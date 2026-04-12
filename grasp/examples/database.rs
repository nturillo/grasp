use std::{collections::HashSet, path::Path};

use grasp::{algorithms::search::{Dijkstra, ShortestPath}, graph::prelude::*};
use gdraw::app::GraspApp;
use rusqlite::{self, Connection};

const GRAPH_SIZE: usize = 6;

/// Graph backed by a file on disk
pub struct DatabaseGraph{
    db_connection: Connection
}
impl DatabaseGraph{
    pub fn open(path: impl AsRef<Path>) -> Result<Self, rusqlite::Error>{
        let db_connection = Connection::open(path)?;
        // Setup graph struct
        db_connection.execute(
            "CREATE TABLE IF NOT EXISTS vertices (
                ID BIGINT UNSIGNED PRIMARY KEY
            );",
            ()
        )?;
        // Setup edges
        db_connection.execute(
            "CREATE TABLE IF NOT EXISTS edges (
                Start BIGINT UNSIGNED,
                End BIGINT UNSIGNED,
                PRIMARY KEY (Start, End)
                FOREIGN KEY (Start) REFERENCES vertices(ID) ON DELETE CASCADE
                FOREIGN KEY (End) REFERENCES vertices(ID) ON DELETE CASCADE
            );", 
            ()
        )?;
        Ok(Self{db_connection})
    }
    pub fn in_memory() -> Result<Self, rusqlite::Error>{
        let db_connection = Connection::open_in_memory()?;
        // Setup graph struct
        db_connection.execute(
            "CREATE TABLE IF NOT EXISTS vertices (
                ID BIGINT UNSIGNED PRIMARY KEY
            );",
            ()
        )?;
        // Setup edges
        db_connection.execute(
            "CREATE TABLE IF NOT EXISTS edges (
                Start BIGINT UNSIGNED,
                End BIGINT UNSIGNED,
                PRIMARY KEY (Start, End)
                FOREIGN KEY (Start) REFERENCES vertices(ID) ON DELETE CASCADE
                FOREIGN KEY (End) REFERENCES vertices(ID) ON DELETE CASCADE
            );", 
            ()
        )?;
        Ok(Self{db_connection})
    }
}
impl GraphTrait for DatabaseGraph{
    fn vertex_count(&self) -> usize {
        let mut stmt = self.db_connection.prepare("SELECT COUNT(ID) FROM vertices;").expect("");
        let count = stmt.query_one((), |r| r.get::<usize, isize>(0)).expect("");
        count as usize
    }

    fn edge_count(&self) -> usize {
        let mut stmt = self.db_connection.prepare("SELECT COUNT(*) FROM edges;").expect("");
        let count = stmt.query_one((), |r| r.get::<usize, isize>(0)).expect("");
        count as usize
    }

    fn has_vertex(&self, v: VertexID) -> bool {
        let mut stmt = self.db_connection.prepare("SELECT ID FROM vertices WHERE vertices.ID = (?1);").expect("");
        stmt.exists([v.to_string()]).expect("")
    }

    fn has_edge(&self, e: EdgeID) -> bool {
        let mut stmt = self.db_connection.prepare(
            "SELECT 1 FROM edges WHERE edges.Start = (?1) AND edges.End = (?2);"
        ).expect("");
        stmt.exists([e.0.to_string(), e.1.to_string()]).expect("")
    }

    fn vertices(&self) -> impl Iterator<Item=VertexID> {
        let mut stmt = self.db_connection.prepare(
            "SELECT ID FROM vertices;"
        ).expect("");
        stmt.query_map((), |r| r.get::<usize, isize>(0)).expect("")
            .filter_map(|id| id.ok().map(|i| i as VertexID))
            .collect::<Vec<usize>>().into_iter()
    }

    fn edges(&self) -> impl Iterator<Item=EdgeID> {
        let mut stmt = self.db_connection.prepare(
            "SELECT Start, End FROM edges;"
        ).expect("");
        stmt.query_map((), |r| {
            let start = r.get::<usize, isize>(0)? as usize;
            let end = r.get::<usize, isize>(1)? as usize;
            Ok((start, end))
        }).expect("")
            .filter_map(|id| id.ok())
            .collect::<Vec<EdgeID>>().into_iter()
    }

    fn neighbors(&self, v: VertexID) -> impl Set<Item = VertexID> {
        let mut stmt = self.db_connection.prepare(
            "SELECT End FROM edges WHERE edges.Start = (?1);"
        ).expect("");
        stmt.query_map([v.to_string()], |r| r.get::<usize, isize>(0)).expect("")
            .filter_map(|id| id.ok().map(|i| i as VertexID))
            .collect::<HashSet<VertexID>>()
    }

    fn vertex_set(&self) -> impl Set<Item = VertexID> {
        self.vertices().collect::<HashSet<VertexID>>()
    }
}
impl GraphMut for DatabaseGraph{
    fn create_vertex(&mut self) -> VertexID {
        // Find smallest unused vertex
        let mut stmt = self.db_connection.prepare("
            SELECT 0 WHERE NOT EXISTS (SELECT 1 FROM vertices WHERE ID = 0)

            UNION ALL

            SELECT MIN(u.ID + 1)
            FROM vertices u LEFT JOIN vertices v
                ON u.ID + 1 = v.ID
            WHERE v.ID IS NULL;
        ").expect("");
        // Get first
        let unused_id = stmt.query_row((), |r| r.get::<usize, isize>(0)).expect("");
        // Insert unused_id into vertices table
        self.db_connection.execute("INSERT INTO vertices (ID) VALUES (?1);", [unused_id.to_string()]).expect("");
        // return id
        unused_id as usize
    }

    fn remove_vertex(&mut self, v: VertexID) -> impl Iterator<Item = EdgeID> {
        // Get effected edges
        let mut edge_stmt = self.db_connection.prepare("
            SELECT Start, End FROM edges WHERE Start = (?1) OR End = (?1);
        ").expect("");
        let edge_iter = edge_stmt.query_map([v.to_string()], |r| {
            let start = r.get::<usize, isize>(0)? as usize;
            let end = r.get::<usize, isize>(1)? as usize;
            Ok((start, end))
        }).expect("").filter_map(|r| r.ok()).collect::<Vec<EdgeID>>().into_iter();
        // Delete edges Again could be done automatically with Foreign Key constraints
        self.db_connection.execute(
            "DELETE FROM edges WHERE Start = (?1) OR End = (?1)", 
            [v.to_string()]
        ).expect("");
        // Delete vertex
        self.db_connection.execute("
            DELETE FROM vertices WHERE ID = (?1);
        ", [v.to_string()]).expect("");
        // Return edges
        edge_iter
    }

    fn try_add_edge(&mut self, edge: EdgeID) -> Result<(), GraphError> {
        // Ensure valid. Could also do this with Foreign Key constraint
        if !self.has_vertex(edge.0) {
            if !self.has_vertex(edge.1) {
                return Err(GraphError::NeitherVertexInGraph(edge.0, edge.1));
            }else {
                return Err(GraphError::VertexNotInGraph(edge.0));
            }
        } else if !self.has_vertex(edge.1) {
            return Err(GraphError::VertexNotInGraph(edge.1));
        }
        // Attempt to add edge
        let mut stmt = self.db_connection.prepare("
            INSERT INTO edges (Start, End) VALUES ((?1), (?2));
        ").expect("");
        stmt.execute([edge.0.to_string(), edge.1.to_string()]).expect("");
        Ok(())
    }

    fn remove_edge(&mut self, e: EdgeID) -> bool {
        self.db_connection.execute(
            "DELETE FROM edges WHERE Start = (?1) AND End = (?2)", 
            [e.0.to_string(), e.1.to_string()]
        ).expect("") > 0
    }
}
impl DiGraph for DatabaseGraph{
    fn in_neighbors(&self, v: VertexID) -> impl Set<Item = VertexID> {
        let mut stmt = self.db_connection.prepare(
            "SELECT Start FROM edges WHERE edges.End = (?1);"
        ).expect("");
        stmt.query_map([v.to_string()], |r| r.get::<usize, isize>(0)).expect("")
            .filter_map(|id| id.ok().map(|i| i as VertexID))
            .collect::<HashSet<VertexID>>()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>>{
    let database_path = "db_example_graph.db3";
    let mut graph = DatabaseGraph::open(database_path)?;
    println!("Vertex Count: {}, Edge Count: {}", graph.vertex_count(), graph.edge_count());
    // Build complete graph
    println!("Building K{}", GRAPH_SIZE);
    let mut vertex_set = Vec::with_capacity(GRAPH_SIZE);
    for _ in 0..GRAPH_SIZE{
        let vertex = graph.create_vertex();
        let _ = graph.try_add_neighbors(vertex, vertex_set.iter().copied());
        // Add backedges
        for u in vertex_set.iter(){let _ = graph.try_add_edge((*u, vertex));}
        vertex_set.push(vertex);
    }
    println!("Vertex Count: {}, Edge Count: {}", graph.vertex_count(), graph.edge_count());
    // Print all edges
    for edge in graph.edges(){
        println!("Edge: {:?}", edge);
    }
    // Close and reopen graph
    println!("Closing and Reopening Graph");
    drop(graph);
    let mut graph = DatabaseGraph::open(database_path)?;
    println!("Vertex Count: {}, Edge Count: {}", graph.vertex_count(), graph.edge_count());
    // Remove some edges and vertices
    println!("Deleting vertex {}", vertex_set[0]);
    for edge in graph.remove_vertex(vertex_set[0]) {
        println!("Deleted edge {:?} by consequence", edge);
    }
    // Delete and edge
    let edge = graph.edges().next().unwrap();
    assert!(graph.remove_edge(edge));
    assert!(graph.remove_edge(edge.inv()));
    println!("Removed edge {:?}", edge);
    // Construct Cycle
    println!("Building C{}", GRAPH_SIZE);
    let start_vertex = graph.create_vertex();
    let second_vertex = graph.create_vertex();
    let _ = graph.try_add_edge((start_vertex, second_vertex));
    let mut cur_vertex = second_vertex;
    print!("Cycle: {}, {}, ", start_vertex, second_vertex);
    for _ in 2..GRAPH_SIZE{
        let new_vertex = graph.create_vertex();
        let _ = graph.try_add_edge((cur_vertex, new_vertex));
        cur_vertex = new_vertex;
        print!("{}, ", cur_vertex);
    }
    println!();
    let _ = graph.try_add_edge((cur_vertex, start_vertex));
    // Find minimal path between start vertex and next vertex
    let mut distance_from_start = Dijkstra::from_source(start_vertex, &graph, |_, _| Some(1))?;
    let mut distance_from_second = Dijkstra::from_source(second_vertex, &graph, |_, _| Some(1))?;
    // iterate dijkstras to end
    while let Some(_) = distance_from_start.next() {}
    while let Some(_) = distance_from_second.next() {}
    // Get shortest path info
    let start_to_second_dist = distance_from_start.distance_to(second_vertex).unwrap();
    let start_to_second_path = distance_from_start.shortest_path_to(second_vertex).unwrap();
    let second_to_start_dist = distance_from_second.distance_to(start_vertex).unwrap();
    let second_to_start_path = distance_from_second.shortest_path_to(start_vertex).unwrap();
    println!("Distance from start->second: {}, Distance from second->start: {}", start_to_second_dist, second_to_start_dist);
    println!("Path from start->second: {:?}", start_to_second_path);
    println!("Path from second->start: {:?}", second_to_start_path);
    // Convert to std form and display in visualizer
    let mut app = GraspApp::new();
    app.load(&graph);
    app.start()?;
    Ok(())
}



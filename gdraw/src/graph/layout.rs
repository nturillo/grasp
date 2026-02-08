use core::f32;
use std::collections::{HashMap, btree_map::Range};

use eframe::egui::Vec2;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

use crate::graph::storage::Graph;

pub enum LayoutType {
    FruchtermanReingold,
}

pub struct LayoutConfig {
    pub layout_type: LayoutType,
    pub area: (f32, f32),
    pub iterations: usize,
    pub temperature_decay_factor: f32,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            layout_type: LayoutType::FruchtermanReingold,
            area: (2.0, 2.0),
            iterations: 200,
            temperature_decay_factor: 0.95,
        }
    }
}

pub(crate) fn apply(graph: &mut Graph, config: &LayoutConfig) {
    match config.layout_type {
        LayoutType::FruchtermanReingold => fruchterman_reingold(graph, config),
    }
}

// Algorithm from https://reingold.co/force-directed.pdf
fn fruchterman_reingold(graph: &mut Graph, config: &LayoutConfig) {
    let area_size = config.area.0 * config.area.1;
    let edge_length = (area_size / graph.vertices() as f32).sqrt();
    let mut temp = f32::min(config.area.0, config.area.1) / 10.0;

    let force_rep = |v: Vec2, u: Vec2| {
        let displacement = v - u;
        displacement * (edge_length / displacement.length()).powi(2)
    };
    let force_att = |v: Vec2, u: Vec2| {
        let displacement = v - u;
        displacement * displacement.length() / edge_length
    };

    let mut vertex_displacement: HashMap<usize, Vec2> = HashMap::new();

    place_randomly(graph, config);

    for _ in 0..config.iterations {
        for (v_id, v_vertex) in &graph.vertex_list {
            let v = v_vertex.center;

            for (u_id, u_vertex) in &graph.vertex_list {
                let u = u_vertex.center;

                if v_id != u_id {
                    *vertex_displacement.entry(*v_id).or_default() += force_rep(v, u);
                }
            }
        }

        for (edge, _) in &graph.edge_list {
            if let (Some(v_vertex), Some(u_vertex)) = (
                graph.vertex_list.get(&edge[0]),
                graph.vertex_list.get(&edge[1]),
            ) {
                vertex_displacement
                    .entry(edge[0])
                    .and_modify(|d| *d -= force_att(v_vertex.center, u_vertex.center));

                vertex_displacement
                    .entry(edge[1])
                    .and_modify(|d| *d += force_att(v_vertex.center, u_vertex.center));
            }
        }

        for (id, vertex) in &mut graph.vertex_list {
            let disp = vertex_displacement.get(id).unwrap_or(&Vec2::ZERO);

            vertex.center += disp.normalized() * f32::min(disp.length(), temp);
            vertex.center.x = f32::min(
                config.area.0 / 2.0,
                f32::max(-config.area.0 / 2.0, vertex.center.x),
            );
            vertex.center.y = f32::min(
                config.area.1 / 2.0,
                f32::max(-config.area.1 / 2.0, vertex.center.y),
            );
        }

        temp *= config.temperature_decay_factor;
        vertex_displacement.clear();
    }
}

fn place_randomly(graph: &mut Graph, config: &LayoutConfig) {
    let mut rng = ChaCha8Rng::seed_from_u64(graph.get_hash());
    let area = Vec2::from(config.area);
    let offset = Vec2::splat(0.5);

    graph.vertex_list.values_mut().for_each(|vertex| {
        vertex.center = area * (Vec2::from(rng.random::<(f32, f32)>()) - offset)
    });
}

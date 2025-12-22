use crate::graph::graph_traits::GraphTrait;



pub fn to_dot<G: GraphTrait>(g: G) -> String {
    let mut s= "Graph {".to_string();
    for (u,v) in g.edges() {
        s.push_str(&format!("{} -- {}\n", u, v))
    }
    s.push('}');
    s
}

#[cfg(test)]
mod tests {
    #[test]
    fn butterfly_dot() {
        todo!();
    }
}
use core::f32;
use std::{collections::HashMap, fmt};

use eframe::egui::Vec2;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

use crate::graph::storage::Graph;

#[derive(Clone, Copy)]
pub(crate) enum PartialLayout {
    None,
    FruchtermanReingold(f32),
}

#[derive(Clone, Copy, PartialEq)]
pub enum LayoutType {
    FruchtermanReingold,
}

impl fmt::Display for LayoutType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            LayoutType::FruchtermanReingold => write!(f, "Fruchterman-Reingold"),
        }
    }
}

#[derive(Clone, Copy)]
pub struct LayoutConfig {
    pub layout_type: LayoutType,
    pub area: (f32, f32),
    pub iterations: usize,
    pub iterations_per_update: usize,
    pub temperature_decay_factor: f32,
    pub temperature_decay_factor_per_update: f32,
    pub run_per_update: bool,

    pub(crate) partial_data: PartialLayout,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            layout_type: LayoutType::FruchtermanReingold,
            area: (2.0, 2.0),
            iterations: 3000,
            iterations_per_update: 100,
            temperature_decay_factor: 0.95,
            temperature_decay_factor_per_update: 0.9999,
            run_per_update: false,
            partial_data: PartialLayout::None,
        }
    }
}

pub(crate) fn apply(graph: &mut Graph) {
    place_randomly(graph);
    graph.layout_config.partial_data = PartialLayout::None;
    reapply(graph);
}

pub(crate) fn reapply(graph: &mut Graph) {
    graph.layout_config.partial_data = match graph.layout_config.layout_type {
        LayoutType::FruchtermanReingold => fruchterman_reingold(graph),
    };
}

// Algorithm from https://reingold.co/force-directed.pdf
fn fruchterman_reingold(graph: &mut Graph) -> PartialLayout {
    let config = graph.layout_config;
    let (decay_factor, iterations) = if config.run_per_update {
        (
            config.temperature_decay_factor_per_update,
            config.iterations_per_update,
        )
    } else {
        (config.temperature_decay_factor, config.iterations)
    };

    let area_size = config.area.0 * config.area.1;
    let edge_length = (area_size / graph.vertices() as f32).sqrt();

    let mut temp = match config.partial_data {
        PartialLayout::FruchtermanReingold(t) => t,
        _ => f32::min(config.area.0, config.area.1) / 10.0,
    };

    let force_rep = |v: Vec2, u: Vec2| {
        let displacement = v - u;
        displacement * (edge_length / displacement.length()).powi(2)
    };
    let force_att = |v: Vec2, u: Vec2| {
        let displacement = v - u;
        displacement * displacement.length() / edge_length
    };

    let mut vertex_displacement: HashMap<usize, Vec2> = HashMap::new();

    for _ in 0..iterations {
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

        temp *= decay_factor;
        vertex_displacement.clear();
    }

    if config.run_per_update {
        PartialLayout::FruchtermanReingold(temp)
    } else {
        PartialLayout::None
    }
}

fn place_randomly(graph: &mut Graph) {
    //let mut rng = ChaCha8Rng::seed_from_u64(graph.get_hash()); // May implement later if needed
    let mut rng = rand::rng();
    let area = Vec2::from(graph.layout_config.area);
    let offset = Vec2::splat(0.5);

    graph.vertex_list.values_mut().for_each(|vertex| {
        vertex.center = area * (Vec2::from(rng.random::<(f32, f32)>()) - offset)
    });
}

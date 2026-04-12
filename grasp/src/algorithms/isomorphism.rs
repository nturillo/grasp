use std::collections::HashMap;
use crate::graph::prelude::*;
use wgpu::{
    BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutEntry, 
    BufferUsages, ComputePassDescriptor, ComputePipeline, ComputePipelineDescriptor, 
    Device, InstanceDescriptor, PipelineCompilationOptions, 
    PipelineLayoutDescriptor, Queue, ShaderStages, include_wgsl, 
    util::{BufferInitDescriptor, DeviceExt}, 
    wgt::{BufferDescriptor, CommandEncoderDescriptor, DeviceDescriptor}
};

const ITERATION_COUNT: usize = 8;

pub struct GPUIsomorphismDevice{
    device: Device,
    queue: Queue,
    layout: BindGroupLayout,
    pipeline: ComputePipeline
}
impl GPUIsomorphismDevice{
    // Sets up wgpu, and creates the gpu structures necessary for GPU Isomorphism
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let instance = wgpu::Instance::new(InstanceDescriptor::new_without_display_handle());
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await?;
        let (device, queue) = adapter
            .request_device(&DeviceDescriptor{
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_defaults(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                memory_hints: wgpu::MemoryHints::MemoryUsage,
                trace: wgpu::Trace::Off
            })
            .await?;
        Ok(Self::from_device_and_queue(device, queue))
    }
    // Creates the gpu structures necessary for GPU Isomorphism
    pub fn from_device_and_queue(device: Device, queue: Queue) -> Self{
        // Compile shader
        let shader = device.create_shader_module(include_wgsl!("../../shaders/isomorphism.wgsl"));
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { label: None, entries: &[
            BindGroupLayoutEntry{
                binding: 0,
                visibility: ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: true }, has_dynamic_offset: false, min_binding_size: None },
                count: None
            },
            BindGroupLayoutEntry{
                binding: 1,
                visibility: ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: true }, has_dynamic_offset: false, min_binding_size: None },
                count: None
            }, 
            BindGroupLayoutEntry{
                binding: 2,
                visibility: ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: true }, has_dynamic_offset: false, min_binding_size: None },
                count: None
            },
            BindGroupLayoutEntry{
                binding: 3,
                visibility: ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: false }, has_dynamic_offset: false, min_binding_size: None },
                count: None
            },
        ] });
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor{
            label: None, bind_group_layouts: &[Some(&layout)], immediate_size: 0
        });
        let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor{
            label: None,
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("calculate_color"),
            cache: None,
            compilation_options: PipelineCompilationOptions{constants: &[], zero_initialize_workgroup_memory: false}
        });
        Self{device, queue, layout, pipeline}
    }
    /// Tests if two graphs are isomorphic. Will return true for certain non isomorphic graphs
    pub async fn estimate_isomorphic(&self, graph_a: &impl GraphTrait, graph_b: &impl GraphTrait) -> bool{
        // Easy short circuit case
        if graph_a.vertex_count() != graph_b.vertex_count() || graph_a.edge_count() != graph_b.edge_count() {
            return false
        }
        // Don't do work when we have empty graphs
        if graph_a.is_empty() {return true;}
        // Also don't do work if we have no edges
        if graph_a.edge_count() == 0 {return true;}
        // Convert to CSR
        let (_, vertices_a, adj_a, color_a) = to_csr_format(graph_a);
        let (_, vertices_b, adj_b, color_b) = to_csr_format(graph_b);
        // Upload data to the gpu
        let vertices_a_buffer = self.device.create_buffer_init(&BufferInitDescriptor{
            label: None,
            contents: bytemuck::cast_slice::<u32, u8>(&vertices_a),
            usage: BufferUsages::STORAGE,
        });
        let adj_a_buffer = self.device.create_buffer_init(&BufferInitDescriptor{
            label: None,
            contents: bytemuck::cast_slice::<u32, u8>(&adj_a),
            usage: BufferUsages::STORAGE,
        });
        let color_a1_buffer = self.device.create_buffer_init(&BufferInitDescriptor{
            label: None,
            contents: bytemuck::cast_slice::<u32, u8>(&color_a),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
        });
        let color_a2_buffer = self.device.create_buffer_init(&BufferInitDescriptor{
            label: None,
            contents: bytemuck::cast_slice::<u32, u8>(&color_a),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
        });
        // Used to read data back to cpu
        let color_a_read_buffer = self.device.create_buffer(&BufferDescriptor{
            label: None,
            size: color_a1_buffer.size(),
            usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
            mapped_at_creation: false
        });
        let vertices_b_buffer = self.device.create_buffer_init(&BufferInitDescriptor{
            label: None,
            contents: bytemuck::cast_slice::<u32, u8>(&vertices_b),
            usage: BufferUsages::STORAGE,
        });
        let adj_b_buffer = self.device.create_buffer_init(&BufferInitDescriptor{
            label: None,
            contents: bytemuck::cast_slice::<u32, u8>(&adj_b),
            usage: BufferUsages::STORAGE,
        });
        let color_b1_buffer = self.device.create_buffer_init(&BufferInitDescriptor{
            label: None,
            contents: bytemuck::cast_slice::<u32, u8>(&color_b),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
        });
        let color_b2_buffer = self.device.create_buffer_init(&BufferInitDescriptor{
            label: None,
            contents: bytemuck::cast_slice::<u32, u8>(&color_b),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
        });
        // Used to read data back to cpu
        let color_b_read_buffer = self.device.create_buffer(&BufferDescriptor{
            label: None,
            size: color_b1_buffer.size(),
            usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
            mapped_at_creation: false
        });
        // Create bind group
        let bindgroupa1 = self.device.create_bind_group(&BindGroupDescriptor{
            label: None,
            layout: &self.layout,
            entries: &[
                BindGroupEntry{binding: 0, resource: vertices_a_buffer.as_entire_binding()},
                BindGroupEntry{binding: 1, resource: adj_a_buffer.as_entire_binding()},
                BindGroupEntry{binding: 2, resource: color_a1_buffer.as_entire_binding()},
                BindGroupEntry{binding: 3, resource: color_a2_buffer.as_entire_binding()},
            ]
        });
        let bindgroupa2 = self.device.create_bind_group(&BindGroupDescriptor{
            label: None,
            layout: &self.layout,
            entries: &[
                BindGroupEntry{binding: 0, resource: vertices_a_buffer.as_entire_binding()},
                BindGroupEntry{binding: 1, resource: adj_a_buffer.as_entire_binding()},
                BindGroupEntry{binding: 2, resource: color_a2_buffer.as_entire_binding()},
                BindGroupEntry{binding: 3, resource: color_a1_buffer.as_entire_binding()},
            ]
        });
        let bindgroupb1 = self.device.create_bind_group(&BindGroupDescriptor{
            label: None,
            layout: &self.layout,
            entries: &[
                BindGroupEntry{binding: 0, resource: vertices_b_buffer.as_entire_binding()},
                BindGroupEntry{binding: 1, resource: adj_b_buffer.as_entire_binding()},
                BindGroupEntry{binding: 2, resource: color_b1_buffer.as_entire_binding()},
                BindGroupEntry{binding: 3, resource: color_b2_buffer.as_entire_binding()},
            ]
        });
        let bindgroupb2 = self.device.create_bind_group(&BindGroupDescriptor{
            label: None,
            layout: &self.layout,
            entries: &[
                BindGroupEntry{binding: 0, resource: vertices_b_buffer.as_entire_binding()},
                BindGroupEntry{binding: 1, resource: adj_b_buffer.as_entire_binding()},
                BindGroupEntry{binding: 2, resource: color_b2_buffer.as_entire_binding()},
                BindGroupEntry{binding: 3, resource: color_b1_buffer.as_entire_binding()},
            ]
        });
        // Start compute pass
        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor{label: None});
        let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor{label: None, timestamp_writes: None});
        compute_pass.set_pipeline(&self.pipeline);
        // Start dispatching, switch 
        let a_inv_count = vertices_a.len().div_ceil(64) as u32;
        let b_inv_count = vertices_b.len().div_ceil(64) as u32;
        for _ in 0..ITERATION_COUNT{
            compute_pass.set_bind_group(0, Some(&bindgroupa1), &[]);
            compute_pass.dispatch_workgroups(a_inv_count, 1, 1);
            compute_pass.set_bind_group(0, Some(&bindgroupa2), &[]);
            compute_pass.dispatch_workgroups(a_inv_count, 1, 1);
            compute_pass.set_bind_group(0, Some(&bindgroupb1), &[]);
            compute_pass.dispatch_workgroups(b_inv_count, 1, 1);
            compute_pass.set_bind_group(0, Some(&bindgroupb2), &[]);
            compute_pass.dispatch_workgroups(b_inv_count, 1, 1);
        }
        // Read from a1 and b1 buffers
        drop(compute_pass);
        encoder.copy_buffer_to_buffer(&color_a1_buffer, 0, &color_a_read_buffer, 0, color_a1_buffer.size());
        let (wake_a, wait_a) = futures::channel::oneshot::channel::<()>();
        encoder.map_buffer_on_submit(&color_a_read_buffer, wgpu::MapMode::Read, .., |_| {let _ = wake_a.send(());});
        encoder.copy_buffer_to_buffer(&color_b1_buffer, 0, &color_b_read_buffer, 0, color_b1_buffer.size());
        let (wake_b, wait_b) = futures::channel::oneshot::channel::<()>();
        encoder.map_buffer_on_submit(&color_b_read_buffer, wgpu::MapMode::Read, .., |_| {let _ = wake_b.send(());});
        // Submit work
        let command_buffer = encoder.finish();
        self.queue.submit([command_buffer]);
        let _ = self.device.poll(wgpu::wgt::PollType::Wait { submission_index: None, timeout: None });
        // Wait for buffer map
        let _ = wait_a.await;
        let colors_a: Vec<u32> = bytemuck::cast_slice::<u8, u32>(&color_a_read_buffer.get_mapped_range(..))
            .into_iter().cloned().collect();
        let _ = wait_b.await;
        let colors_b: Vec<u32> = bytemuck::cast_slice::<u8, u32>(&color_b_read_buffer.get_mapped_range(..))
            .into_iter().cloned().collect();
        // build sorted color histogram
        let mut a_multiset_map: HashMap<u32, usize> = HashMap::default();
        let mut multiset_a: Vec<usize> = vec![];
        for color in colors_a {
            if let Some(index) = a_multiset_map.get(&color) {
                multiset_a[*index] += 1;
            }else {
                multiset_a.push(1);
                a_multiset_map.insert(color, multiset_a.len()-1);
            }
        }
        multiset_a.sort();
        let mut b_multiset_map: HashMap<u32, usize> = HashMap::default();
        let mut multiset_b: Vec<usize> = vec![];
        for color in colors_b {
            if let Some(index) = b_multiset_map.get(&color) {
                multiset_b[*index] += 1;
            }else {
                multiset_b.push(1);
                b_multiset_map.insert(color, multiset_b.len()-1);
            }
        }
        multiset_b.sort();
        // Compare shapes
        if multiset_a.len() != multiset_b.len() {return false;}
        for i in 0..multiset_a.len() {
            if multiset_a[i] != multiset_b[i] {return false;}
        }
        true
    }
}

/// Converts a graph to compact sparse row format. Very good format for gpu code. Also returns colors based on degree
fn to_csr_format(graph: &impl GraphTrait) -> (HashMap<VertexID, usize>, Vec<u32>, Vec<u32>, Vec<u32>){
    if graph.is_empty() {return (HashMap::default(), vec![], vec![], vec![])}
    // Create vertex map
    let map: HashMap<VertexID, usize> = graph.vertices().enumerate().map(|(i, v)| (v, i)).collect();
    let mut vertex: Vec<u32> = vec![0; graph.vertex_count()];
    let mut colors: Vec<u32> = vec![0; graph.vertex_count()];
    let mut last_neighbor_count = 0;
    for v in graph.vertices() {
        let neighbor_count = graph.neighbors(v).len();
        if map[&v] == vertex.len()-1 {last_neighbor_count = neighbor_count;}
        vertex[map[&v]] = neighbor_count as u32;
        colors[map[&v]] = neighbor_count as u32;
    }
    // prefix sum scan
    let mut temp = vertex[0]; vertex[0] = 0;
    for i in 1..vertex.len() {std::mem::swap(&mut temp, &mut vertex[i]); temp += vertex[i];}
    // Set adj list
    let mut adj: Vec<u32> = vec![0; *vertex.last().unwrap() as usize+last_neighbor_count];
    for v in graph.vertices() {
        for (i, n) in graph.neighbors(v).iter().enumerate(){
            let v = map[&v]; let n = map[&n];
            adj[i+vertex[v] as usize] = n as u32;
        }
    }
    (map, vertex, adj, colors)
}

#[cfg(all(test, feature="wgpu"))]
mod test{
    use futures::executor::block_on;
    use wgpu::{Backend, BackendOptions, Backends, DeviceDescriptor, InstanceDescriptor, InstanceFlags, MemoryBudgetThresholds};

    use crate::{algorithms::isomorphism::GPUIsomorphismDevice, graph::{AnyVertexGraph, GraphMut, constructors::build_cycle, prelude::{SparseDiGraph, SparseSimpleGraph}}};

    #[test]
    fn test_isomorphism(){
        let mut graph_a = SparseDiGraph::default();
        graph_a.add_vertex(0); graph_a.add_vertex(1); graph_a.add_vertex(2); graph_a.add_vertex(3); graph_a.add_vertex(4);
        graph_a.add_edge((0, 1));
        graph_a.add_edge((1, 2));
        graph_a.add_edge((2, 3));
        graph_a.add_edge((3, 4));
        graph_a.add_edge((4, 0));
        let mut graph_b = SparseDiGraph::default();
        graph_b.add_vertex(0); graph_b.add_vertex(1); graph_b.add_vertex(2); graph_b.add_vertex(3); graph_b.add_vertex(4);
        graph_b.add_edge((1, 0));
        graph_b.add_edge((2, 1));
        graph_b.add_edge((3, 2));
        graph_b.add_edge((4, 3));
        graph_b.add_edge((0, 4));

        let isomorphism = block_on(GPUIsomorphismDevice::new());
        assert!(isomorphism.is_ok());
        let isomorphism = isomorphism.unwrap();
        let result = block_on(isomorphism.estimate_isomorphic(&graph_a, &graph_b));
        assert!(result);
        graph_b.add_edge((0, 1));
        let result = block_on(isomorphism.estimate_isomorphic(&graph_a, &graph_b));
        assert!(!result);
    }

    #[test]
    fn test_large_isomorphism(){
        let mut graph_a: SparseSimpleGraph = build_cycle(1000);
        let mut graph_b: SparseSimpleGraph = graph_a.clone();
        graph_a.add_edge((0, 100));
        graph_b.add_edge((1, 101));
        let isomorphism = block_on(GPUIsomorphismDevice::new());
        assert!(isomorphism.is_ok());
        let isomorphism = isomorphism.unwrap();
        let result = block_on(isomorphism.estimate_isomorphic(&graph_a, &graph_b));
        assert!(result);
        graph_b.remove_edge((1, 101));
        let result = block_on(isomorphism.estimate_isomorphic(&graph_a, &graph_b));
        assert!(!result);
    }
}

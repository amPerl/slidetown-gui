use nif::Nif;
use wgpu::util::DeviceExt;

use super::{instance::OpaqueMeshInstance, vertex::OpaqueMeshVertex};

#[derive(Debug)]
pub struct OpaqueMesh {
    pub label: Option<String>,
    pub vertices: Vec<OpaqueMeshVertex>,
    pub buffers_v_idx_i: Option<(wgpu::Buffer, wgpu::Buffer, wgpu::Buffer)>,
    pub indices: Vec<u32>,
    pub instances: Vec<OpaqueMeshInstance>,
    pub bounds_from_origin: [f32; 3],
}

impl OpaqueMesh {
    pub fn upload(&mut self, device: &wgpu::Device) {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("nif_vertex_buffer"),
            contents: bytemuck::cast_slice(&self.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("nif_index_buffer"),
            contents: bytemuck::cast_slice::<u32, _>(&self.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let instances_data = self
            .instances
            .iter()
            .map(OpaqueMeshInstance::to_raw)
            .collect::<Vec<_>>();
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("nif_instance_buffer"),
            contents: bytemuck::cast_slice(&instances_data),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        self.buffers_v_idx_i = Some((vertex_buffer, index_buffer, instance_buffer));
    }

    pub fn buffers(&self) -> Option<&(wgpu::Buffer, wgpu::Buffer, wgpu::Buffer)> {
        self.buffers_v_idx_i.as_ref()
    }

    pub fn create_from_nif_lod(nif: &Nif, lod_distance: f32) -> Self {
        let mut mesh = nif::collectors::single_mesh::Mesh::default();
        mesh.add_nif(nif, lod_distance).unwrap();

        let vertices: Vec<OpaqueMeshVertex> = mesh
            .vertices
            .into_iter()
            .zip(mesh.normals.into_iter())
            .map(|(position, normal)| OpaqueMeshVertex {
                position: position.into(),
                normal: normal.into(),
            })
            .collect();

        let bounds = vertices
            .iter()
            .map(|v| (v.position))
            .reduce(|bounds, v| {
                let mut new_bounds = bounds;
                new_bounds[0] = new_bounds[0].max(v[0].abs());
                new_bounds[1] = new_bounds[1].max(v[1].abs());
                new_bounds[2] = new_bounds[2].max(v[2].abs());
                new_bounds
            })
            .unwrap_or([0.0; 3]);

        let indices = mesh.indices.clone();

        Self {
            label: None,
            vertices,
            indices,
            instances: Default::default(),
            bounds_from_origin: bounds,
            buffers_v_idx_i: None,
        }
    }

    pub fn merge(&mut self, mut other: OpaqueMesh) {
        self.bounds_from_origin = [
            self.bounds_from_origin[0].max(other.bounds_from_origin[0]),
            self.bounds_from_origin[1].max(other.bounds_from_origin[1]),
            self.bounds_from_origin[2].max(other.bounds_from_origin[2]),
        ];
        let index_base = self.vertices.len() as u32;
        self.vertices.append(&mut other.vertices);
        self.indices
            .extend(other.indices.drain(..).map(|i| i + index_base));
    }
}

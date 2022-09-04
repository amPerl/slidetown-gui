use eframe::wgpu::{self, util::DeviceExt};
use nif::Nif;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct UntexturedMeshVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
}

impl UntexturedMeshVertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 2] =
            wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[allow(dead_code)]
pub struct UntexturedMeshInstanceRaw {
    model: [[f32; 4]; 4],
    normal: [[f32; 3]; 3],
}

impl UntexturedMeshInstanceRaw {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 7] = wgpu::vertex_attr_array![
            2 => Float32x4,
            3 => Float32x4,
            4 => Float32x4,
            5 => Float32x4,
            6 => Float32x3,
            7 => Float32x3,
            8 => Float32x3
        ];
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<UntexturedMeshInstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &ATTRIBUTES,
        }
    }
}

#[derive(Debug, Clone)]
pub struct UntexturedMeshInstance {
    pub position: glam::Vec3,
    pub rotation: glam::Quat,
    pub scale: f32,
}

impl Default for UntexturedMeshInstance {
    fn default() -> Self {
        Self {
            position: glam::Vec3::ZERO,
            rotation: glam::Quat::IDENTITY,
            scale: 1.0,
        }
    }
}

impl UntexturedMeshInstance {
    pub fn to_raw(&self) -> UntexturedMeshInstanceRaw {
        UntexturedMeshInstanceRaw {
            model: glam::Mat4::from_scale_rotation_translation(
                glam::vec3(self.scale, self.scale, self.scale),
                self.rotation,
                self.position,
            )
            .to_cols_array_2d(),
            normal: glam::Mat3::from_quat(self.rotation).to_cols_array_2d(),
        }
    }
}

#[derive(Debug)]
pub struct UntexturedMesh {
    pub vertices: Vec<UntexturedMeshVertex>,
    pub buffers_v_idx_i: Option<(wgpu::Buffer, wgpu::Buffer, wgpu::Buffer)>,
    pub indices: Vec<u32>,
    pub instances: Vec<UntexturedMeshInstance>,
    pub bounds_from_origin: [f32; 3],
}

impl UntexturedMesh {
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
            .map(UntexturedMeshInstance::to_raw)
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

    pub fn create_from_nif_lod(
        nif: &Nif,
        lod_distance: f32,
        instances: Option<Vec<UntexturedMeshInstance>>,
    ) -> Self {
        let mut mesh = nif::collectors::single_mesh::Mesh::default();
        mesh.add_nif(nif, lod_distance).unwrap();

        let vertices: Vec<UntexturedMeshVertex> = mesh
            .vertices
            .into_iter()
            .zip(mesh.normals.into_iter())
            .map(|(position, normal)| UntexturedMeshVertex {
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

        let instances = instances
            .unwrap_or_else(|| vec![UntexturedMeshInstance::default()])
            .to_vec();

        Self {
            vertices,
            indices,
            instances,
            bounds_from_origin: bounds,
            buffers_v_idx_i: None,
        }
    }

    pub fn merge(&mut self, mut other: UntexturedMesh) {
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

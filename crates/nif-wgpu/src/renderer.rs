use nif::Nif;
use slotmap::{new_key_type, DenseSlotMap};

use crate::{
    camera::Camera,
    light::Light,
    pipelines::opaque::{instance::OpaqueMeshInstance, mesh::OpaqueMesh, OpaquePipeline},
};

new_key_type! { pub struct MeshHandle; }

pub struct Renderer {
    pub light: Light,
    pub camera: Camera,
    opaque_pipeline: OpaquePipeline,
    opaque_meshes: DenseSlotMap<MeshHandle, OpaqueMesh>,
}

impl Renderer {
    pub fn new(device: &wgpu::Device, target_format: wgpu::TextureFormat) -> Self {
        let light = Light::new(device);
        let camera = Camera::new(device);
        let opaque_pipeline = OpaquePipeline::new(device, target_format.into(), &light, &camera);
        Self {
            light,
            camera,
            opaque_pipeline,
            opaque_meshes: Default::default(),
        }
    }

    pub fn clear(&mut self) {
        self.opaque_meshes.clear();
    }

    pub fn add_nif(&mut self, nif: &Nif, lod_distance: f32) -> MeshHandle {
        let mesh = OpaqueMesh::create_from_nif_lod(nif, lod_distance);
        self.opaque_meshes.insert(mesh)
    }

    pub fn merge_nif(
        &mut self,
        handle: MeshHandle,
        nif: &Nif,
        lod_distance: f32,
    ) -> Result<(), ()> {
        if let Some(old_mesh) = self.opaque_meshes.get_mut(handle) {
            let new_mesh = OpaqueMesh::create_from_nif_lod(nif, lod_distance);
            old_mesh.merge(new_mesh);
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn add_instance(
        &mut self,
        handle: MeshHandle,
        position: glam::Vec3,
        rotation: glam::Quat,
        scale: f32,
    ) -> Result<(), ()> {
        if let Some(mesh) = self.opaque_meshes.get_mut(handle) {
            mesh.instances.push(OpaqueMeshInstance {
                position,
                rotation,
                scale,
            });
            Ok(())
        } else {
            Err(())
        }
    }

    fn upload_pending(&mut self, device: &wgpu::Device) {
        for mesh in self.opaque_meshes.values_mut() {
            if mesh.buffers().is_none() {
                mesh.upload(device);
            }
        }
    }

    pub fn prepare(&mut self, device: &wgpu::Device) {
        self.upload_pending(device);
    }

    pub fn render<'rpass>(&'rpass self, rpass: &mut wgpu::RenderPass<'rpass>) {
        self.opaque_pipeline.bind(rpass);
        rpass.set_bind_group(0, self.camera.bind_group(), &[]);
        rpass.set_bind_group(1, self.light.bind_group(), &[]);
        for mesh in self.opaque_meshes.values() {
            if mesh.instances.is_empty() {
                continue;
            }
            if let Some((vertex_buffer, index_buffer, instance_buffer)) = mesh.buffers() {
                let has_label = mesh.label.is_some();
                if let Some(label) = mesh.label.as_ref() {
                    rpass.push_debug_group(label);
                }
                rpass.set_vertex_buffer(0, vertex_buffer.slice(..));
                rpass.set_vertex_buffer(1, instance_buffer.slice(..));
                rpass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                rpass.draw_indexed(
                    0..(mesh.indices.len() as _),
                    0,
                    0..mesh.instances.len() as _,
                );
                if has_label {
                    rpass.pop_debug_group();
                }
            }
        }
    }
}

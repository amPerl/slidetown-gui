use std::collections::HashMap;

use eframe::wgpu::{self, util::DeviceExt};
use nif::Nif;

use super::{
    light::Light,
    texture::Texture,
    untextured_mesh::{UntexturedMesh, UntexturedMeshInstance},
    untextured_mesh_pipeline::UntexturedMeshPipeline,
    Camera,
};

pub struct NifRenderResources {
    untextured_mesh_pipeline: UntexturedMeshPipeline,
    pub meshes: HashMap<String, UntexturedMesh>,
    pub combined_bounds: [f32; 3],
}

impl NifRenderResources {
    pub fn new(
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
        light: &Light,
        camera: &Camera,
    ) -> Self {
        let untextured_mesh_pipeline =
            UntexturedMeshPipeline::new(device, target_format.into(), light, camera);
        Self {
            untextured_mesh_pipeline,
            meshes: Default::default(),
            combined_bounds: [0.0; 3],
        }
    }

    pub fn clear_nifs(&mut self) {
        self.meshes.clear();
        self.combined_bounds = [0.0, 0.0, 0.0];
    }

    pub fn add_nif(
        &mut self,
        nif: &Nif,
        lod_distance: f32,
        group: Option<String>,
        instances: Option<Vec<UntexturedMeshInstance>>,
    ) {
        let group = group.unwrap_or_default();
        let mesh = UntexturedMesh::create_from_nif_lod(nif, lod_distance, instances);
        let mesh = match self.meshes.get_mut(&group) {
            Some(existing_mesh) => {
                existing_mesh.merge(mesh);
                existing_mesh
            }
            None => {
                self.meshes.insert(group.clone(), mesh);
                self.meshes.get_mut(&group).unwrap()
            }
        };

        let mut old_bounds = self.combined_bounds;
        if mesh.bounds_from_origin[0] > old_bounds[0] {
            old_bounds[0] = mesh.bounds_from_origin[0];
        }
        if mesh.bounds_from_origin[1] > old_bounds[1] {
            old_bounds[1] = mesh.bounds_from_origin[1];
        }
        if mesh.bounds_from_origin[2] > old_bounds[2] {
            old_bounds[2] = mesh.bounds_from_origin[2];
        }
        self.combined_bounds = old_bounds;
    }

    pub fn set_nif(
        &mut self,
        nif: &Nif,
        lod_distance: f32,
        group: Option<String>,
        instances: Option<Vec<UntexturedMeshInstance>>,
    ) {
        self.meshes.remove(&group.clone().unwrap_or_default());
        self.add_nif(nif, lod_distance, group, instances)
    }

    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        camera: &Camera,
        light: &Light,
        model_rotation: &glam::Quat,
    ) {
        self.untextured_mesh_pipeline.update_camera(queue, camera);
        self.untextured_mesh_pipeline.update_light(queue, light);

        for mesh in self.meshes.values_mut() {
            if mesh.buffers().is_none() {
                mesh.upload(device);
            }
        }

        if let Some(mesh) = self.meshes.values_mut().next() {
            let instance = mesh.instances.get_mut(0).unwrap();
            instance.rotation = *model_rotation;
            let instance = instance.clone();
            if let Some((_, _, instance_buffer)) = mesh.buffers() {
                queue.write_buffer(
                    instance_buffer,
                    0,
                    bytemuck::cast_slice(&[instance.to_raw()]),
                );
            }
        }
    }

    pub fn paint<'rpass>(&'rpass self, rpass: &mut wgpu::RenderPass<'rpass>) {
        self.untextured_mesh_pipeline.set(rpass);

        for mesh in self.meshes.values() {
            if let Some((vertex_buffer, index_buffer, instance_buffer)) = mesh.buffers() {
                rpass.set_vertex_buffer(0, vertex_buffer.slice(..));
                rpass.set_vertex_buffer(1, instance_buffer.slice(..));
                rpass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                rpass.draw_indexed(
                    0..(mesh.indices.len() as _),
                    0,
                    0..mesh.instances.len() as _,
                );
            }
        }
    }
}

use eframe::wgpu;

use super::{
    camera::Camera,
    light::Light,
    untextured_mesh::{UntexturedMeshInstanceRaw, UntexturedMeshVertex},
};

pub struct UntexturedMeshPipeline {
    pipeline: wgpu::RenderPipeline,
    camera_uniform_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    light_uniform_buffer: wgpu::Buffer,
    light_bind_group: wgpu::BindGroup,
}

impl UntexturedMeshPipeline {
    pub fn new(
        device: &wgpu::Device,
        target: wgpu::ColorTargetState,
        light: &Light,
        camera: &Camera,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("nif_shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("./untextured_mesh.wgsl").into()),
        });

        let (light_uniform_buffer, light_bind_group_layout, light_bind_group) =
            light.create(device);

        let (camera_uniform_buffer, camera_bind_group_layout, camera_bind_group) =
            camera.create(device);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("nif_pipeline_layout"),
            bind_group_layouts: &[&camera_bind_group_layout, &light_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("nif_render_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    UntexturedMeshVertex::desc(),
                    UntexturedMeshInstanceRaw::desc(),
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(target)],
            }),
            primitive: wgpu::PrimitiveState {
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        Self {
            pipeline,
            camera_uniform_buffer,
            camera_bind_group,
            light_uniform_buffer,
            light_bind_group,
        }
    }

    pub fn update_light(&self, queue: &wgpu::Queue, light: &Light) {
        queue.write_buffer(
            &self.light_uniform_buffer,
            0,
            bytemuck::cast_slice(&[*light]),
        );
    }

    pub fn update_camera(&self, queue: &wgpu::Queue, camera: &Camera) {
        queue.write_buffer(
            &self.camera_uniform_buffer,
            0,
            bytemuck::cast_slice(&[camera.to_raw()]),
        );
    }

    pub fn set<'rpass>(&'rpass self, rpass: &mut wgpu::RenderPass<'rpass>) {
        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, &self.camera_bind_group, &[]);
        rpass.set_bind_group(1, &self.light_bind_group, &[]);
    }
}

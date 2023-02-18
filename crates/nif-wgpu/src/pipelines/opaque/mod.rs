use crate::{camera::Camera, light::Light};

use self::{instance::OpaqueMeshInstanceRaw, vertex::OpaqueMeshVertex};

pub mod instance;
pub mod mesh;
pub mod vertex;

pub struct OpaquePipeline {
    pipeline: wgpu::RenderPipeline,
}

impl OpaquePipeline {
    pub fn new(
        device: &wgpu::Device,
        target: wgpu::ColorTargetState,
        light: &Light,
        camera: &Camera,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("nif_shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("./opaque.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("nif_pipeline_layout"),
            bind_group_layouts: &[camera.layout(), light.layout()],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("nif_render_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[OpaqueMeshVertex::desc(), OpaqueMeshInstanceRaw::desc()],
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

        Self { pipeline }
    }

    pub fn bind<'rpass>(&'rpass self, rpass: &mut wgpu::RenderPass<'rpass>) {
        rpass.set_pipeline(&self.pipeline);
    }
}

use eframe::wgpu::{self, util::DeviceExt};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    position: [f32; 4],
    view_proj: [[f32; 4]; 4],
}

#[derive(Debug, Clone)]
pub struct Camera {
    pub eye: glam::Vec3,
    pub target: glam::Vec3,
    pub up: glam::Vec3,
    pub aspect_ratio: f32,
    fov_y: f32,
    z_near: f32,
    z_far: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            eye: (0.0, 5.0, 10.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: glam::Vec3::Z,
            aspect_ratio: 1.0,
            fov_y: 60.0,
            z_near: 1.0,
            z_far: 100000.0,
        }
    }
}

impl Camera {
    pub fn build_view_matrix(&self) -> glam::Mat4 {
        glam::Mat4::look_at_rh(self.eye, self.target, self.up)
    }

    pub fn build_projection_matrix(&self) -> glam::Mat4 {
        glam::Mat4::perspective_rh(
            self.fov_y.to_radians(),
            self.aspect_ratio,
            self.z_near,
            self.z_far,
        )
    }

    pub fn to_raw(&self) -> CameraUniform {
        let view_proj = self.build_projection_matrix() * self.build_view_matrix();
        CameraUniform {
            position: self.eye.extend(1.0).into(),
            view_proj: view_proj.to_cols_array_2d(),
        }
    }

    pub fn create(
        &self,
        device: &wgpu::Device,
    ) -> (wgpu::Buffer, wgpu::BindGroupLayout, wgpu::BindGroup) {
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("nif_camera_uniform_buffer"),
            contents: bytemuck::cast_slice(&[self.to_raw()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("nif_camera_bind_group_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("nif_camera_bind_group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        (uniform_buffer, bind_group_layout, bind_group)
    }
}

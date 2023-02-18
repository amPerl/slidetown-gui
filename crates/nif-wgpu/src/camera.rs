use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    position: [f32; 4],
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new(position: glam::Vec3, view_proj: glam::Mat4) -> Self {
        Self {
            position: position.extend(1.0).into(),
            view_proj: view_proj.to_cols_array_2d(),
        }
    }
}

#[derive(Debug)]
pub struct Camera {
    pub eye: glam::Vec3,
    pub target: glam::Vec3,
    pub up: glam::Vec3,
    pub aspect_ratio: f32,
    fov_y: f32,
    z_near: f32,
    z_far: f32,
    buffer: wgpu::Buffer,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
}

impl Camera {
    pub fn new(device: &wgpu::Device) -> Self {
        let eye = (5.0, -5.0, 5.0).into();
        let target = (0.0, 0.0, 0.0).into();
        let up = glam::Vec3::Z;
        let aspect_ratio = 1.0;
        let fov_y = 60.0;
        let z_near = 1.0;
        let z_far = 100000.0;

        let view = build_view_matrix(eye, target, up);
        let proj = build_projection_matrix(fov_y, aspect_ratio, z_near, z_far);
        let view_proj = build_view_proj_matrix(view, proj);

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("nif_camera_uniform_buffer"),
            contents: bytemuck::cast_slice(&[CameraUniform::new(eye, view_proj)]),
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
                resource: buffer.as_entire_binding(),
            }],
        });

        Self {
            eye,
            target,
            up,
            aspect_ratio,
            fov_y,
            z_near,
            z_far,
            buffer,
            bind_group_layout,
            bind_group,
        }
    }

    pub fn to_raw(&self) -> CameraUniform {
        let view = build_view_matrix(self.eye, self.target, self.up);
        let proj = build_projection_matrix(self.fov_y, self.aspect_ratio, self.z_near, self.z_far);
        let view_proj = build_view_proj_matrix(view, proj);
        CameraUniform::new(self.eye, view_proj)
    }

    pub fn layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub fn update(&self, queue: &wgpu::Queue) {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.to_raw()]));
    }
}

fn build_view_proj_matrix(view: glam::Mat4, proj: glam::Mat4) -> glam::Mat4 {
    proj * view
}

fn build_view_matrix(eye: glam::Vec3, target: glam::Vec3, up: glam::Vec3) -> glam::Mat4 {
    glam::Mat4::look_at_rh(eye, target, up)
}

fn build_projection_matrix(fov_y: f32, aspect_ratio: f32, z_near: f32, z_far: f32) -> glam::Mat4 {
    glam::Mat4::perspective_rh(fov_y.to_radians(), aspect_ratio, z_near, z_far)
}

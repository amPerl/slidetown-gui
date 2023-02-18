use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    position: [f32; 3],
    _padding: u32,
    color: [f32; 3],
    _padding2: u32,
}

impl LightUniform {
    pub fn new(position: glam::Vec3, color: glam::Vec3) -> Self {
        Self {
            position: position.into(),
            _padding: Default::default(),
            color: color.into(),
            _padding2: Default::default(),
        }
    }
}

#[derive(Debug)]
pub struct Light {
    pub position: glam::Vec3,
    pub color: glam::Vec3,
    buffer: wgpu::Buffer,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
}

impl Light {
    pub fn new(device: &wgpu::Device) -> Self {
        let position = glam::vec3(5.0, 0.0, 10.0);
        let color = glam::vec3(1.0, 1.0, 1.0);

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("nif_light_uniform_buffer"),
            contents: bytemuck::cast_slice(&[LightUniform::new(position, color)]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("nif_light_bind_group_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("nif_light_bind_group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        Self {
            buffer,
            bind_group_layout,
            bind_group,
            position,
            color,
        }
    }

    fn to_raw(&self) -> LightUniform {
        LightUniform::new(self.position, self.color)
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

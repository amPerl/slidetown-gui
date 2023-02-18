#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[allow(dead_code)]
pub struct OpaqueMeshInstanceRaw {
    model: [[f32; 4]; 4],
    normal: [[f32; 3]; 3],
}

impl OpaqueMeshInstanceRaw {
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
            array_stride: std::mem::size_of::<OpaqueMeshInstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &ATTRIBUTES,
        }
    }
}

#[derive(Debug, Clone)]
pub struct OpaqueMeshInstance {
    pub position: glam::Vec3,
    pub rotation: glam::Quat,
    pub scale: f32,
}

impl Default for OpaqueMeshInstance {
    fn default() -> Self {
        Self {
            position: glam::Vec3::ZERO,
            rotation: glam::Quat::IDENTITY,
            scale: 1.0,
        }
    }
}

impl OpaqueMeshInstance {
    pub fn to_raw(&self) -> OpaqueMeshInstanceRaw {
        OpaqueMeshInstanceRaw {
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

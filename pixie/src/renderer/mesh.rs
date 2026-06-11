pub struct Mesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub instance_buffer: Option<wgpu::Buffer>,
    pub instance_capacity: usize,
    pub num_indices: u32,
    pub num_instances: u32,
}

impl Mesh {
    pub fn replace_instance(
        &mut self,
        buffer: wgpu::Buffer,
        instance_capacity: usize,
        num_instances: u32,
    ) {
        self.instance_buffer = Some(buffer);
        self.instance_capacity = instance_capacity;
        self.num_instances = num_instances;
    }
}

impl std::fmt::Debug for Mesh {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Mesh")
            .field("index_count", &self.num_indices)
            .field("instance_count", &self.num_instances)
            .field("instance_capacity", &self.instance_capacity)
            .finish()
    }
}

pub(crate) fn required_instance_capacity(current: usize, required: usize) -> usize {
    if required <= current {
        current
    } else {
        required.next_power_of_two()
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SpriteInstanceRaw {
    pub(crate) uv: [f32; 4],
    pub(crate) model: [[f32; 4]; 4],
}

impl SpriteInstanceRaw {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<SpriteInstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ColorSpriteInstanceRaw {
    pub(crate) uv: [f32; 4],
    pub(crate) model: [[f32; 4]; 4],
    pub(crate) color: [f32; 3],
}

impl ColorSpriteInstanceRaw {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<ColorSpriteInstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 20]>() as wgpu::BufferAddress,
                    shader_location: 9,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::required_instance_capacity;

    #[test]
    fn instance_capacity_grows_to_next_power_of_two() {
        assert_eq!(required_instance_capacity(0, 1), 1);
        assert_eq!(required_instance_capacity(0, 3), 4);
        assert_eq!(required_instance_capacity(4, 5), 8);
        assert_eq!(required_instance_capacity(8, 17), 32);
    }

    #[test]
    fn instance_capacity_does_not_shrink() {
        assert_eq!(required_instance_capacity(16, 0), 16);
        assert_eq!(required_instance_capacity(16, 7), 16);
        assert_eq!(required_instance_capacity(16, 16), 16);
    }
}

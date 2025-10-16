use crate::renderer::mesh::InstanceTileRaw;
use std::sync::Arc;

#[derive(Clone)]
pub struct TextRenderData {
    pub content: Arc<String>,  // Arc makes clone O(1) instead of O(n)
    pub color : [f32;3],
    pub position: [f32; 3],
    pub size: [f32; 2],
}

pub struct TileRenderData {
    pub uv: [f32; 4],
    pub position: [f32; 3],
    pub size: [f32; 2],
}

impl TileRenderData {
    pub fn get_instance_matrix(&self) -> InstanceTileRaw {
        let position = cgmath::Vector3 { x: self.position[0], y: self.position[1], z: self.position[2] };
        let translation_matrix = cgmath::Matrix4::from_translation(position);
        let scale_matrix = cgmath::Matrix4::from_nonuniform_scale(self.size[0], self.size[1], 1.0);
        let model = (translation_matrix * scale_matrix).into();

        InstanceTileRaw {
            uv: self.uv,
            model,
        }

    }
}
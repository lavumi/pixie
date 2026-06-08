use crate::renderer::mesh::SpriteInstanceRaw;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct TextRenderData {
    pub content: Arc<String>, // Arc makes clone O(1) instead of O(n)
    pub color: [f32; 3],
    pub position: [f32; 3],
    pub size: [f32; 2],
}

pub struct SpriteRenderData {
    pub uv: [f32; 4],
    pub position: [f32; 3],
    pub size: [f32; 2],
    pub rotation: f32,
}

pub struct RenderFrame<'a> {
    pub camera_uniform: [[f32; 4]; 4],
    pub sprite_render_data: &'a HashMap<String, Vec<SpriteRenderData>>,
    pub sprite_atlases: &'a [String],
    pub texts: &'a [TextRenderData],
}

impl SpriteRenderData {
    pub fn get_instance_matrix(&self) -> SpriteInstanceRaw {
        let position = cgmath::Vector3 {
            x: self.position[0],
            y: self.position[1],
            z: self.position[2],
        };
        let translation_matrix = cgmath::Matrix4::from_translation(position);
        let rotation_matrix = cgmath::Matrix4::from_angle_z(cgmath::Rad(self.rotation));
        let scale_matrix = cgmath::Matrix4::from_nonuniform_scale(self.size[0], self.size[1], 1.0);
        let model = (translation_matrix * rotation_matrix * scale_matrix).into();

        SpriteInstanceRaw { uv: self.uv, model }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cgmath::{Matrix4, Vector4};
    use std::f32::consts::FRAC_PI_2;

    fn assert_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < 0.0001,
            "expected {expected}, got {actual}"
        );
    }

    #[test]
    fn zero_rotation_matches_translation_and_scale() {
        let data = SpriteRenderData {
            uv: [0.0, 1.0, 0.0, 1.0],
            position: [2.0, 3.0, 0.5],
            size: [4.0, 2.0],
            rotation: 0.0,
        };
        let raw = data.get_instance_matrix();
        let expected: [[f32; 4]; 4] =
            (Matrix4::from_translation(cgmath::Vector3::new(2.0, 3.0, 0.5))
                * Matrix4::from_nonuniform_scale(4.0, 2.0, 1.0))
            .into();

        for (actual_column, expected_column) in raw.model.iter().zip(expected.iter()) {
            for (actual, expected) in actual_column.iter().zip(expected_column.iter()) {
                assert_close(*actual, *expected);
            }
        }
    }

    #[test]
    fn rotates_around_sprite_center() {
        let data = SpriteRenderData {
            uv: [0.0, 1.0, 0.0, 1.0],
            position: [2.0, 3.0, 0.0],
            size: [2.0, 1.0],
            rotation: FRAC_PI_2,
        };
        let raw = data.get_instance_matrix();
        let model: Matrix4<f32> = raw.model.into();

        let center = model * Vector4::new(0.0, 0.0, 0.0, 1.0);
        assert_close(center.x, 2.0);
        assert_close(center.y, 3.0);

        let right_edge = model * Vector4::new(0.5, 0.0, 0.0, 1.0);
        assert_close(right_edge.x, 2.0);
        assert_close(right_edge.y, 4.0);
    }
}

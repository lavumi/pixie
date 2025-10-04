use specs::*;
use specs_derive::Component;

#[allow(dead_code)]
#[derive(Debug, Clone, Eq, PartialEq, Hash, Copy)]
pub enum BodyType { Static, Kinematic, Dynamic }

#[derive(Component, Clone)]
pub struct Collider {
    pub aabb_offset: [f32; 4],
}
impl Default for Collider {
    fn default() -> Self {
        Collider {
            aabb_offset: [-1.0, 0.0, -0.25, 0.25],
        }
    }
}

#[derive(Component, Clone)]
pub struct Tile {
    pub uv: [f32; 4],
    pub atlas: String,
}

#[derive(Component, Clone)]
pub struct Transform {
    pub position: [f32; 3],
    pub size: [f32; 2],
}

#[derive(Component, Clone)]
pub struct Text {
    pub content: String,
    pub color : [f32;3]
}

#[derive(Component, Clone, Default)]
pub struct Animation {
    pub index : u32,
    pub delta : f32,
}

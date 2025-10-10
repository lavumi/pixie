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

#[derive(Component, Clone)]
pub struct Animation {
    pub current_frame: u32,
    pub frame_count: u32,
    pub frame_duration: f32,
    pub elapsed_time: f32,
    pub loop_animation: bool,
    pub finished: bool,
    pub atlas_columns: u32,
    pub atlas_rows: u32,
}

impl Default for Animation {
    fn default() -> Self {
        Animation {
            current_frame: 0,
            frame_count: 1,
            frame_duration: 1.0,
            elapsed_time: 0.0,
            loop_animation: true,
            finished: false,
            atlas_columns: 1,
            atlas_rows: 1,
        }
    }
}

// Physics components
#[derive(Component, Clone, Debug)]
pub struct RigidBody {
    pub body_type: BodyType,
    pub mass: f32,
    pub restitution: f32,  // 탄성 (0.0 = 완전 비탄성, 1.0 = 완전 탄성)
}

impl Default for RigidBody {
    fn default() -> Self {
        RigidBody {
            body_type: BodyType::Dynamic,
            mass: 1.0,
            restitution: 0.0,
        }
    }
}

#[derive(Component, Clone, Default, Debug)]
pub struct Velocity {
    pub linear: [f32; 2],
    pub angular: f32,
}

#[derive(Component, Clone, Default)]
pub struct Force {
    pub linear: [f32; 2],
    pub torque: f32,
}

#[derive(Component, Clone)]
pub struct CircleCollider {
    pub radius: f32,
}

impl Default for CircleCollider {
    fn default() -> Self {
        CircleCollider { radius: 0.5 }
    }
}

#[derive(Component, Clone)]
pub struct BoxCollider {
    pub width: f32,
    pub height: f32,
}

impl Default for BoxCollider {
    fn default() -> Self {
        BoxCollider { width: 1.0, height: 1.0 }
    }
}

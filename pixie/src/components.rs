// Components in hecs don't require any special derives or imports
// Any type that implements Send + Sync can be used as a component

#[allow(dead_code)]
#[derive(Debug, Clone, Eq, PartialEq, Hash, Copy)]
pub enum BodyType { Static, Kinematic, Dynamic }

#[derive(Clone)]
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

#[derive(Clone)]
pub struct Tile {
    pub uv: [f32; 4],
    pub atlas: String,
}

#[derive(Clone)]
pub struct Transform {
    pub position: [f32; 3],
    pub size: [f32; 2],
}

#[derive(Clone)]
pub struct Text {
    pub content: String,
    pub version: u64,  // Increment when content changes
}

impl Default for Text {
    fn default() -> Self {
        Text {
            content: String::new(),
            version: 0,
        }
    }
}

impl Text {
    pub fn set_content(&mut self, content: String) {
        if self.content != content {
            self.content = content;
            self.version = self.version.wrapping_add(1);
        }
    }
}

#[derive(Clone)]
pub struct TextStyle {
    pub size: [f32; 2],
    pub color: [f32; 3],
    pub z_index: f32,
}

impl Default for TextStyle {
    fn default() -> Self {
        TextStyle {
            size: [1.0, 1.0],
            color: [1.0, 1.0, 1.0],
            z_index: 1.0,
        }
    }
}

#[derive(Clone)]
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
#[derive(Clone, Debug)]
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

#[derive(Clone, Default, Debug)]
pub struct Velocity {
    pub linear: [f32; 2],
    pub angular: f32,
}

#[derive(Clone, Default)]
pub struct Force {
    pub linear: [f32; 2],
    pub torque: f32,
}

#[derive(Clone)]
pub struct CircleCollider {
    pub radius: f32,
}

impl Default for CircleCollider {
    fn default() -> Self {
        CircleCollider { radius: 0.5 }
    }
}

#[derive(Clone)]
pub struct BoxCollider {
    pub width: f32,
    pub height: f32,
}

impl Default for BoxCollider {
    fn default() -> Self {
        BoxCollider { width: 1.0, height: 1.0 }
    }
}

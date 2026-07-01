// Components in hecs don't require any special derives or imports
// Any type that implements Send + Sync can be used as a component

#[allow(dead_code)]
#[derive(Debug, Clone, Eq, PartialEq, Hash, Copy)]
pub enum BodyType {
    Static,
    Kinematic,
    Dynamic,
}

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
pub struct Sprite {
    pub uv: [f32; 4],
    pub atlas: AtlasId,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Transform {
    pub position: [f32; 3],
    pub size: [f32; 2],
    /// Counter-clockwise rotation around the Z axis, in radians.
    pub rotation: f32,
}

impl Transform {
    pub fn new(position: [f32; 3], size: [f32; 2]) -> Self {
        Self {
            position,
            size,
            rotation: 0.0,
        }
    }

    pub fn with_rotation(position: [f32; 3], size: [f32; 2], rotation: f32) -> Self {
        Self {
            position,
            size,
            rotation,
        }
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::new([0.0, 0.0, 0.0], [1.0, 1.0])
    }
}

#[derive(Clone, Default)]
pub struct Text {
    pub content: String,
}

#[derive(Clone)]
pub struct TextStyle {
    /// Size of the font em square rasterized at 48 pixels.
    ///
    /// Interpreted in world units for `TextCoordinateSpace::World` and pixels
    /// for `TextCoordinateSpace::Screen`.
    pub size: [f32; 2],
    pub color: [f32; 3],
    pub z_index: f32,
    pub coordinate_space: TextCoordinateSpace,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TextCoordinateSpace {
    World,
    Screen,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum UiAnchor {
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    Center,
    CenterRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

impl UiAnchor {
    pub fn factor(self) -> [f32; 2] {
        match self {
            Self::TopLeft => [0.0, 0.0],
            Self::TopCenter => [0.5, 0.0],
            Self::TopRight => [1.0, 0.0],
            Self::CenterLeft => [0.0, 0.5],
            Self::Center => [0.5, 0.5],
            Self::CenterRight => [1.0, 0.5],
            Self::BottomLeft => [0.0, 1.0],
            Self::BottomCenter => [0.5, 1.0],
            Self::BottomRight => [1.0, 1.0],
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub enum UiRoot {
    #[default]
    RenderViewport,
    Window,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiTransform {
    /// Parent reference point in the selected UI root.
    pub anchor: UiAnchor,
    /// Reference point inside the widget's rendered bounds.
    pub pivot: UiAnchor,
    /// Pixel offset from the anchor. Positive Y points down.
    pub offset: [f32; 2],
    pub root: UiRoot,
}

impl UiTransform {
    pub fn new(anchor: UiAnchor, pivot: UiAnchor, offset: [f32; 2]) -> Self {
        Self {
            anchor,
            pivot,
            offset,
            root: UiRoot::RenderViewport,
        }
    }

    pub fn with_root(mut self, root: UiRoot) -> Self {
        self.root = root;
        self
    }
}

impl Default for UiTransform {
    fn default() -> Self {
        Self::new(UiAnchor::Center, UiAnchor::Center, [0.0, 0.0])
    }
}

impl Default for TextStyle {
    fn default() -> Self {
        TextStyle {
            size: [1.0, 1.0],
            color: [1.0, 1.0, 1.0],
            z_index: 1.0,
            coordinate_space: TextCoordinateSpace::World,
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
    pub restitution: f32, // 탄성 (0.0 = 완전 비탄성, 1.0 = 완전 탄성)
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
        BoxCollider {
            width: 1.0,
            height: 1.0,
        }
    }
}
use crate::AtlasId;

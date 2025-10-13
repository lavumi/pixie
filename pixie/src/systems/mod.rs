// System modules - now as free functions instead of System trait impls
pub mod update_camera;
pub mod update_animation;
pub mod update_physics;
pub mod apply_gravity;
pub mod collision_system;

// Re-export system functions
pub use update_camera::update_camera;
pub use update_animation::update_animation;
pub use update_physics::update_physics;
pub use apply_gravity::{apply_gravity, Gravity};
pub use collision_system::collision_system;

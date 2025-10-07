// Re-export generic systems from engine
pub use pixie::systems::*;
pub use pixie::dispatcher::UnifiedDispatcher;

// Define physics demo system execution order
pixie::construct_dispatcher!(
    (UpdateCamera, "update_camera", &[]),
    (ApplyGravity, "apply_gravity", &[]),
    (UpdatePhysics, "update_physics", &["apply_gravity"]),
    (CollisionSystem, "collision", &["update_physics"])
);

pub fn build() -> Box<dyn UnifiedDispatcher + 'static> {
    new_dispatch()
}

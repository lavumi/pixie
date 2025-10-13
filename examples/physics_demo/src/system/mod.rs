// Re-export generic systems from engine
use pixie::systems::*;
pub use pixie::dispatcher::UnifiedDispatcher;

// Define physics demo system execution order
pixie::construct_dispatcher!(
    update_camera,
    apply_gravity,
    update_physics,
    collision_system
);

pub fn build() -> Box<dyn UnifiedDispatcher + 'static> {
    new_dispatch()
}

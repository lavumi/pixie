use super::UnifiedDispatcher;
use hecs::World;
use crate::resources::ResourceContainer;

/// System function type - takes world and resources, returns nothing
pub type SystemFn = fn(&mut World, &mut ResourceContainer);

/// Macro to construct a single-threaded dispatcher with given system functions
///
/// Example usage:
/// ```ignore
/// construct_dispatcher!(
///     update_physics,
///     update_animation,
///     collision_system
/// );
/// ```
#[macro_export]
macro_rules! construct_dispatcher {
    ( $( $system_fn:expr ),* $(,)? ) => {
        pub fn new_dispatch() -> Box<dyn $crate::dispatcher::UnifiedDispatcher> {
            let mut systems: Vec<$crate::dispatcher::SystemFn> = Vec::new();
            $(
                systems.push($system_fn);
            )*
            Box::new($crate::dispatcher::SingleThreadedDispatcher { systems })
        }
    };
}

/// Single-threaded dispatcher for WASM and simple use cases
pub struct SingleThreadedDispatcher {
    pub systems: Vec<SystemFn>,
}

impl UnifiedDispatcher for SingleThreadedDispatcher {
    fn run_now(&mut self, world: &mut World, resources: &mut ResourceContainer) {
        for system_fn in &self.systems {
            system_fn(world, resources);
        }
    }
}

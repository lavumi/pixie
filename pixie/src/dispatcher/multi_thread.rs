use super::UnifiedDispatcher;
use hecs::World;
use crate::resources::ResourceContainer;

// TODO: Multi-threaded dispatcher implementation for hecs
//
// The multi-threaded dispatcher is not yet implemented for hecs.
// This will require manual parallelization using rayon or similar crates.
//
// Implementation notes:
// 1. hecs doesn't have automatic parallel scheduling like specs
// 2. Need to manually split queries to run in parallel
// 3. Can use rayon::par_iter() for parallel processing
// 4. Must ensure no data races (separate mut queries, or use atomic operations)
//
// Example approach:
// - Use rayon to parallelize independent systems
// - Split entity batches for parallel processing within systems
// - Carefully manage mutable access to world and resources
//
// For now, use the single-threaded dispatcher (automatically selected on non-WASM)

/// System function type - takes world and resources, returns nothing
pub type SystemFn = fn(&mut World, &mut ResourceContainer);

/// Macro to construct a multi-threaded dispatcher with given system functions
///
/// Currently falls back to single-threaded execution.
/// TODO: Implement proper multi-threaded execution
#[macro_export]
macro_rules! construct_dispatcher {
    ( $( $system_fn:expr ),* $(,)? ) => {
        pub fn new_dispatch() -> Box<dyn $crate::dispatcher::UnifiedDispatcher> {
            log::warn!("Multi-threaded dispatcher not yet implemented for hecs. Using single-threaded fallback.");
            let mut systems: Vec<$crate::dispatcher::SystemFn> = Vec::new();
            $(
                systems.push($system_fn);
            )*
            Box::new($crate::dispatcher::MultiThreadedDispatcher { systems })
        }
    };
}

/// Multi-threaded dispatcher (placeholder - currently single-threaded)
///
/// TODO: Implement parallel execution using rayon
pub struct MultiThreadedDispatcher {
    pub systems: Vec<SystemFn>,
}

impl UnifiedDispatcher for MultiThreadedDispatcher {
    fn run_now(&mut self, world: &mut World, resources: &mut ResourceContainer) {
        // TODO: Implement parallel execution
        // For now, just run sequentially like single-threaded
        for system_fn in &self.systems {
            system_fn(world, resources);
        }
    }
}

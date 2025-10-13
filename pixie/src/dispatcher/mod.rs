use hecs::World;
use crate::resources::ResourceContainer;

#[cfg(not(target_arch = "wasm32"))]
pub use multi_thread::*;
#[cfg(target_arch = "wasm32")]
pub use single_thread::*;

#[cfg(target_arch = "wasm32")]
#[macro_use]
mod single_thread;

#[cfg(not(target_arch = "wasm32"))]
#[macro_use]
mod multi_thread;

/// Unified dispatcher trait for running systems
/// In hecs, systems are just functions, so dispatcher stores function pointers
pub trait UnifiedDispatcher {
    fn run_now(&mut self, world: &mut World, resources: &mut ResourceContainer);
}

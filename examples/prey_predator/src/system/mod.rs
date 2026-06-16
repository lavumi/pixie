pub use pixie::dispatcher::UnifiedDispatcher;
use pixie::systems::*;

pixie::construct_dispatcher!(update_camera);

pub fn build() -> Box<dyn UnifiedDispatcher + 'static> {
    new_dispatch()
}

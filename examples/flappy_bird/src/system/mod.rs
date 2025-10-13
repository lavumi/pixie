// Re-export generic systems from engine
pub use pixie::systems::*;
pub use pixie::dispatcher::UnifiedDispatcher;

// Re-export game-specific system functions
pub use check_collision::check_collision;
pub use scroll_background::scroll_background;
pub use scroll_pipe::scroll_pipe;
pub use update_player::update_player;
pub use check_game_stage::check_game_stage;
pub use process_nn::process_nn;

mod check_collision;
mod scroll_background;
mod scroll_pipe;
mod update_player;
mod check_game_stage;
mod process_nn;

// Define game-specific system execution order with function pointers
pixie::construct_dispatcher!(
    update_camera,
    scroll_background,
    scroll_pipe,
    process_nn,
    update_player,
    check_collision,
    check_game_stage,
    update_animation
);

pub fn build() -> Box<dyn UnifiedDispatcher> {
    new_dispatch()
}

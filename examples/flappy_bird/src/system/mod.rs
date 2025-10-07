// Re-export generic systems from engine
pub use pixie::systems::*;
pub use pixie::dispatcher::UnifiedDispatcher;

pub use check_collision::CheckCollision;
pub use scroll_background::ScrollBackground;
pub use scroll_pipe::UpdatePipe;
pub use update_player::UpdatePlayer;
pub use check_game_stage::CheckGameStage;
pub use process_nn::ProcessNN;

mod check_collision;
mod scroll_background;
mod scroll_pipe;
mod update_player;
mod check_game_stage;
mod process_nn;

// Define game-specific system execution order
pixie::construct_dispatcher!(
    (UpdateCamera, "update_camera", &[]),
    (ScrollBackground, "update_scroll", &[]),
    (UpdatePipe, "update_pipe", &[]),
    (ProcessNN, "process_nn", &[]),
    (UpdatePlayer, "update_player", &[]),
    (CheckCollision, "check_collision", &[]),
    (CheckGameStage, "check_game_stage", &[]),
    (UpdateAnimation, "update_animation", &[])
);

pub fn build() -> Box<dyn UnifiedDispatcher + 'static> {
    new_dispatch()
}
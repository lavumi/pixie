#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use crate::winit_state::WinitState;
use crate::flappy_app::FlappyApplication;

// Re-export renderer from engine
pub use engine::renderer;

pub mod winit_state;
pub mod flappy_app;
mod components;
mod resources;
mod system;
mod builder;
mod game_configs;

#[cfg(target_arch = "wasm32")]
mod wasm_bindings;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn start(){
    let title = "wgpu_wasm";
    let width = game_configs::SCREEN_SIZE[0];
    let height = game_configs::SCREEN_SIZE[1];

    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
        } else {
            env_logger::init();
        }
    }

    let (wb, event_loop) = WinitState::create(title, width, height );

    let app = FlappyApplication::default();
    let mut engine = engine::Engine::new(app, wb, &event_loop).await;

    // Load game-specific textures
    engine.get_render_state_mut().load_texture_atlas("tile", include_bytes!("../assets/img/tile.png"));
    engine.get_render_state_mut().load_texture_atlas("bg", include_bytes!("../assets/img/bg.png"));
    engine.get_render_state_mut().load_texture_atlas("player", include_bytes!("../assets/img/player.png"));

    event_loop.run_app(&mut engine).unwrap();
}


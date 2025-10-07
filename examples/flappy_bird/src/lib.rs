#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use crate::flappy_app::FlappyApplication;

// Re-export renderer from pixie
pub use pixie::renderer;

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

    let app = FlappyApplication::default();
    let dispatcher = system::build();
    
    // Load game-specific textures
    let mut textures = std::collections::HashMap::new();
    textures.insert("tile".to_string(), include_bytes!("../assets/img/tile.png") as &[u8]);
    textures.insert("bg".to_string(), include_bytes!("../assets/img/bg.png") as &[u8]);
    textures.insert("player".to_string(), include_bytes!("../assets/img/player.png") as &[u8]);
    
    pixie::Engine::start(app, title, width, height, Some(textures), dispatcher).await;
}


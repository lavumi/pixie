#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use crate::physics_app::PhysicsApp;

pub use pixie::renderer;

pub mod physics_app;
mod components;
mod system;
mod config;

// #[cfg(target_arch = "wasm32")]
// mod wasm_bindings;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn start() {
    let title = "Physics Demo - Bouncing Balls";
    let width = config::SCREEN_SIZE[0];
    let height = config::SCREEN_SIZE[1];

    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Info).expect("Couldn't initialize logger");
        } else {
            env_logger::init();
        }
    }

    let app = PhysicsApp::default();
    let dispatcher = system::build();
    
    // Load textures
    let mut textures = std::collections::HashMap::new();
    textures.insert("ball".to_string(), include_bytes!("../assets/circle.png") as &[u8]);
    textures.insert("box".to_string(), include_bytes!("../assets/box.png") as &[u8]);
    
    pixie::Engine::start(app, title, width, height, Some(textures), dispatcher).await;
}

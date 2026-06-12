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

async fn run() -> Result<(), pixie::EngineError> {
    let title = "wgpu_wasm";
    let width = game_configs::SCREEN_SIZE[0];
    let height = game_configs::SCREEN_SIZE[1];

    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn)
                .map_err(|error| pixie::EngineError::Startup(
                    format!("failed to initialize logger: {error}")
                ))?;
        } else {
            env_logger::try_init().map_err(|error| pixie::EngineError::Startup(
                format!("failed to initialize logger: {error}")
            ))?;
        }
    }

    let app = FlappyApplication::default();
    let dispatcher = system::build();

    let texture_atlases = vec![
        pixie::TextureAtlasAsset::from_static("tile", include_bytes!("../assets/img/tile.png")),
        pixie::TextureAtlasAsset::from_static("bg", include_bytes!("../assets/img/bg.png")),
        pixie::TextureAtlasAsset::from_static("player", include_bytes!("../assets/img/player.png")),
    ];

    pixie::Engine::start(app, title, width, height, texture_atlases, dispatcher).await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn start() -> Result<(), pixie::EngineError> {
    run().await
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub async fn start() -> Result<(), wasm_bindgen::JsError> {
    run().await.map_err(Into::into)
}

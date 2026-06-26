#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use crate::prey_predator_app::PreyPredatorApp;

pub use pixie::renderer;

mod components;
mod config;
pub mod prey_predator_app;
mod resources;
mod system;

async fn run() -> Result<(), pixie::EngineError> {
    let title = "Prey/Predator Simulation";
    let width = config::SCREEN_SIZE[0];
    let height = config::SCREEN_SIZE[1];

    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Info)
                .map_err(|error| pixie::EngineError::Startup(
                    format!("failed to initialize logger: {error}")
                ))?;
        } else {
            env_logger::try_init().map_err(|error| pixie::EngineError::Startup(
                format!("failed to initialize logger: {error}")
            ))?;
        }
    }

    let app = PreyPredatorApp::default();
    let dispatcher = system::build();

    let texture_atlases = vec![
        pixie::TextureAtlasAsset::from_static(
            "area_fill",
            include_bytes!("../assets/area_fill.png"),
        ),
        pixie::TextureAtlasAsset::from_static(
            "area_border",
            include_bytes!("../assets/area_border.png"),
        ),
        pixie::TextureAtlasAsset::from_static("prey", include_bytes!("../assets/prey.png")),
        pixie::TextureAtlasAsset::from_static("predator", include_bytes!("../assets/predator.png")),
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

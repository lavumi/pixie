# Pixie - 2D WebGPU Engine

A lightweight 2D game engine built with Rust, WebGPU, winit, and hecs. Pixie runs natively on desktop platforms and can be built for browsers with WebAssembly.

## Features

- Cross-platform runtime for native and browser targets
- hecs-based entity-component-system architecture
- WebGPU renderer with sprite/tile and text rendering
- Fixed timestep update loop for physics-style systems
- Runtime TTF font rasterization with fontdue
- Reusable engine crate plus game-specific examples

## Workspace

```text
pixie/
├── pixie/                    # Generic engine crate
│   ├── src/
│   │   ├── application.rs    # Application trait
│   │   ├── engine.rs         # winit runtime and frame loop
│   │   ├── components.rs     # Shared ECS components
│   │   ├── systems/          # Generic systems
│   │   ├── resources/        # Camera, delta time, resource container
│   │   └── renderer/         # WebGPU renderer
│   └── assets/               # Shaders and fonts
├── examples/
│   ├── flappy_bird/          # Neural-network Flappy Bird demo
│   └── physics_demo/         # 2D physics demo
└── Cargo.toml                # Workspace manifest
```

## Build And Run

### Native

```bash
# Flappy Bird AI demo
cargo run --bin flappy

# Physics demo
cargo run --bin physics

# Release build
cargo build --release --bin flappy
./target/release/flappy
```

### WebAssembly

Build an example from its directory:

```bash
cd examples/flappy_bird
wasm-pack build --target web

# Serve locally
python -m http.server 8000
# or
npx http-server -p 8000
```

Then open `http://localhost:8000/index.html`.

## Testing And Linting

```bash
cargo test
cargo test -p pixie
cargo fmt
cargo clippy --all-targets
```

## Architecture

Games implement the `Application` trait from `pixie/src/application.rs`:

```rust
use hecs::World;
use pixie::ResourceContainer;
use winit::event::WindowEvent;

pub trait Application {
    fn init(&mut self, world: &mut World, resources: &mut ResourceContainer);
    fn update(&mut self, world: &mut World, resources: &mut ResourceContainer, dt: f32);
    fn fixed_update(&mut self, world: &mut World, resources: &mut ResourceContainer, fixed_dt: f32) {}
    fn handle_input(&mut self, world: &mut World, resources: &mut ResourceContainer, event: &WindowEvent) -> bool;
    fn should_run_fixed(&self, world: &World, resources: &ResourceContainer) -> bool { true }
}
```

The engine owns the `hecs::World`, a `ResourceContainer`, a dispatcher, and the WebGPU `RenderState`. Applications create entities and resources in `init`, run game logic in `update` or `fixed_update`, and handle input through `handle_input`.

Rendering data is collected directly from ECS components:

- `Transform + Sprite` entities become sprite instances
- `Transform + Text + TextStyle` entities become text instances
- `Camera` is stored as an engine-managed resource

The runtime uses a variable update for general game logic and a fixed 60 Hz step for physics-style systems. The dispatcher abstraction is platform-specific, but the current native `MultiThreadedDispatcher` is a sequential fallback after the migration to hecs.

## Core Files

- `pixie/src/engine.rs` - winit event loop, frame timing, renderer integration
- `pixie/src/application.rs` - game-facing application trait
- `pixie/src/components.rs` - shared ECS components
- `pixie/src/resources/` - shared resources and typed resource container
- `pixie/src/systems/` - generic systems such as physics, animation, gravity, collision
- `pixie/src/renderer/` - WebGPU renderer, pipelines, textures, font manager
- `examples/flappy_bird/src/flappy_app.rs` - AI Flappy Bird application
- `examples/physics_demo/src/physics_app.rs` - physics demo application

## Creating A New Game

1. Add a new workspace member under `examples/` or a separate crate.
2. Depend on `pixie = { path = "../../pixie" }`.
3. Implement `Application`.
4. Spawn entities with shared components such as `Transform`, `Sprite`, `Text`, and game-specific components.
5. Insert resources through `ResourceContainer`.
6. Build a dispatcher with the systems your game needs.
7. Start the engine with `TextureAtlasAsset` values passed to `Engine::start`.

See `examples/flappy_bird/src/main.rs`, `examples/flappy_bird/src/lib.rs`, and `examples/physics_demo/src/main.rs` for working startup patterns.

## Asset Loading

Textures are registered with an `AtlasId` and loaded through `TextureAtlasAsset`:

```rust
let atlases = vec![TextureAtlasAsset::from_static(
    "player",
    include_bytes!("../assets/player.png"),
)];
```

A sprite references the same ID with `Sprite { atlas: "player".into(), uv }`.
Games can register runtime-owned image bytes through the engine-managed
`TextureAtlasRegistry` resource. Duplicate IDs, invalid images, and sprites
referencing unloaded IDs produce typed atlas errors instead of renderer panics.

Fonts are loaded by the engine and rasterized at runtime through `FontManager`. Text rendering is driven by ECS `Text` components and uses a version field so unchanged text can be cached.

## Tech Stack

- Rust
- wgpu
- winit
- hecs
- fontdue
- cgmath
- instant
- wasm-bindgen and web-sys for browser builds

## Status

Pixie is an active engine prototype. The hecs migration is in place, examples compile and tests pass, and the next cleanup areas are dispatcher parallelism, renderer error handling, broader engine tests, and keeping docs aligned with code.

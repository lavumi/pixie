# AGENTS.md

This file provides guidance to Codex when working in this repository.

## Project Overview

Pixie is a lightweight 2D game engine built with Rust, wgpu, winit, and hecs. It supports native desktop builds and WebAssembly browser builds.

## Workspace Structure

This Cargo workspace has three members:

- `pixie/` - generic engine library
- `examples/flappy_bird/` - neural-network Flappy Bird demo
- `examples/physics_demo/` - 2D physics demo

## Build And Run Commands

### Native Development

```bash
cargo run --bin flappy
cargo run --bin physics
```

### Release Build

```bash
cargo build --release --bin flappy
./target/release/flappy
```

### WebAssembly Build

```bash
cd examples/flappy_bird
wasm-pack build --target web

python -m http.server 8000
# or
npx http-server -p 8000
```

Then open `http://localhost:8000/index.html`.

### Testing And Linting

```bash
cargo test
cargo test -p pixie
cargo fmt
cargo clippy --all-targets
```

## Core Architecture

### Application Trait

Games implement `pixie::Application` from `pixie/src/application.rs`:

```rust
pub trait Application {
    fn init(&mut self, world: &mut World, resources: &mut ResourceContainer);
    fn update(&mut self, world: &mut World, resources: &mut ResourceContainer, dt: f32);
    fn fixed_update(&mut self, world: &mut World, resources: &mut ResourceContainer, fixed_dt: f32) {}
    fn handle_input(&mut self, world: &mut World, resources: &mut ResourceContainer, event: &WindowEvent) -> bool;
    fn should_run_fixed(&self, world: &World, resources: &ResourceContainer) -> bool { true }
}
```

The engine owns the `hecs::World`, a typed `ResourceContainer`, the dispatcher, the window, and the WebGPU renderer. Applications spawn entities, insert resources, run game logic, and handle input.

### Engine Runtime

`Engine<A: Application>` lives in `pixie/src/engine.rs` and manages:

- winit window and event loop
- ECS world and resources
- fixed 60 Hz update loop for systems
- variable update loop for app logic
- camera and render data collection
- WebGPU `RenderState`

Rendering data is collected from ECS components:

- `Transform + Tile` for sprites/tiles
- `Transform + Text + TextStyle` for text
- `Camera` from resources for the view-projection matrix

### Dispatcher

Dispatcher code lives in `pixie/src/dispatcher/`.

- WASM uses the single-threaded dispatcher.
- Native currently exposes `MultiThreadedDispatcher`, but it is still a sequential fallback after the hecs migration.
- Systems are free functions with the signature `fn(&mut World, &mut ResourceContainer)`.

### Rendering Pipeline

Renderer code lives in `pixie/src/renderer/`.

Key components:

- `RenderState` - main renderer interface
- `GPUResourceManager` - buffers, bind groups, meshes
- `PipelineManager` - shader pipeline setup
- `FontManager` - runtime TTF rasterization
- `Texture` - image loading and GPU texture setup

Shaders live in `pixie/assets/shader/`.

## Components And Resources

Generic components are in `pixie/src/components.rs`, including:

- `Transform`
- `Tile`
- `Text`
- `TextStyle`
- `Animation`
- `RigidBody`
- `Velocity`
- `Force`
- `CircleCollider`
- `BoxCollider`

Generic resources are in `pixie/src/resources/`, including:

- `Camera`
- `DeltaTime`
- `ResourceContainer`

Generic systems are in `pixie/src/systems/`, including:

- `update_physics`
- `update_animation`
- `update_camera`
- `apply_gravity`
- `collision_system`

## Creating A New Game

1. Create a new crate or example in the workspace.
2. Depend on `pixie`.
3. Implement `Application`.
4. Spawn ECS entities with shared and game-specific components.
5. Insert resources through `ResourceContainer`.
6. Build a dispatcher from the systems needed by the game.
7. Start the engine and pass texture atlases through the startup texture map.

Use `examples/flappy_bird/src/flappy_app.rs` and `examples/physics_demo/src/physics_app.rs` as references.

## Current State Notes

- The project uses hecs, not specs.
- The engine crate path is `pixie/`, not `engine/`.
- The physics demo is already part of the workspace.
- Renderer/resource code still has some panic-based error paths.
- Test coverage exists but is concentrated in `ResourceContainer`, neural-network processing, and gene handling.

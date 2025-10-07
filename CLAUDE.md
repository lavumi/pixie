# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Pixie is a lightweight 2D game engine built with Rust and WebGPU, featuring an ECS (Entity-Component-System) architecture. It runs both natively (Windows/macOS/Linux) and in browsers via WebAssembly.

## Workspace Structure

This is a Cargo workspace with the following members:
- `engine/` - Generic game engine library (reusable core)
- `examples/flappy_bird/` - Neural network AI demo game

## Build and Run Commands

### Native Development
```bash
# Run example (development build)
cargo run --bin flappy

# Release build (optimized)
cargo build --release --bin flappy
./target/release/flappy
```

### WebAssembly Build
```bash
cd examples/flappy_bird
wasm-pack build --target web

# Serve locally (choose one):
python -m http.server 8000
# or
npx http-server -p 8000

# Then open http://localhost:8000/index.html
```

### Testing and Linting
```bash
# Run tests
cargo test

# Run tests for specific package
cargo test -p engine

# Format code
cargo fmt

# Lint code
cargo clippy
```

## Core Architecture

### Application Trait Pattern

The engine uses a trait-based abstraction where games implement the `Application` trait:

```rust
pub trait Application {
    fn init(&mut self, world: &mut World);
    fn update(&mut self, world: &mut World, dt: f32);
    fn handle_input(&mut self, world: &mut World, event: &WindowEvent) -> bool;
    fn get_camera_uniform(&self, world: &World) -> [[f32; 4]; 4];
    fn get_tile_instances(&self, world: &World) -> HashMap<String, Vec<TileRenderData>>;
    fn get_text_instances(&self, world: &World) -> Vec<TextRenderData>;
}
```

Located in: `engine/src/application.rs`

### Engine Runtime

The `Engine<A: Application>` struct (in `engine/src/engine.rs`) manages:
- Window and event loop (using winit)
- ECS World (using specs)
- RenderState (WebGPU rendering)
- Frame timing and input handling

Key flow:
1. `Engine::new()` initializes window, world, and calls `app.init()`
2. Event loop calls `app.update()` each frame
3. Rendering data retrieved via trait methods
4. `RenderState` handles WebGPU rendering pipeline

### ECS System Execution

**Platform-specific dispatcher** (`engine/src/dispatcher/`):
- Native builds use multi-threaded dispatcher
- WASM builds use single-threaded dispatcher (web constraints)
- Abstracted via `UnifiedDispatcher` trait

Systems are defined per-game (not in engine core). See `examples/flappy_bird/src/system/` for examples.

### Rendering Pipeline

Located in `engine/src/renderer/`:

**Key components:**
- `RenderState` - Main renderer interface
- `GpuResourceManager` - Manages buffers, bind groups
- `PipelineManager` - Shader pipeline management
- `FontManager` - Runtime TTF font rasterization with fontdue
- `TextureAtlas` - Sprite texture management

**Shaders:** WebGPU shaders in `engine/assets/shader/`:
- `texture.wgsl` - Sprite/tile rendering
- `font.wgsl` - Text rendering

**Rendering flow:**
1. Application provides rendering data via trait methods
2. Engine updates camera buffer
3. Mesh instances updated (sprites/tiles)
4. Text instances updated (dynamic text)
5. `RenderState::render()` executes draw calls

### Cross-Platform Considerations

**WASM-specific code:**
- Canvas setup in `Engine::new()` (`#[cfg(target_arch = "wasm32")]`)
- WebGL fallback enabled via wgpu features
- Single-threaded dispatcher for web compatibility
- Platform-specific dependencies in Cargo.toml

**Time handling:**
- Uses `instant` crate for cross-platform timing
- WASM feature flag: `instant = { features = ["wasm-bindgen"] }`

## Engine Components and Resources

**Generic Components** (`engine/src/components.rs`):
- `Position` - World position
- `Velocity` - Movement velocity
- `AnimationConfig` - Sprite animation
- `Rect` - Collision bounds
- `Renderable` - Sprite rendering data

**Generic Resources** (`engine/src/resources/`):
- `Camera` - 2D orthographic camera with zoom
- `DeltaTime` - Frame delta time
- `InputHandler` - Keyboard/mouse input state

**Generic Systems** (`engine/src/systems/`):
- `UpdatePhysicsSystem` - Apply velocity to position
- `UpdateAnimationSystem` - Frame-based sprite animation
- `UpdateCameraSystem` - Camera matrix calculation

## Creating a New Game

1. Create new binary in workspace
2. Implement `Application` trait
3. Register game-specific components in `init()`
4. Create game systems and add to dispatcher
5. Provide rendering data in trait methods
6. Load textures via `engine.get_render_state_mut().load_texture_atlas()`

See `examples/flappy_bird/src/flappy_app.rs` for reference implementation.

## Key Files to Know

**Engine Core:**
- `engine/src/engine.rs` - Main engine runtime and event loop
- `engine/src/application.rs` - Application trait definition
- `engine/src/renderer/renderer.rs` - RenderState and rendering logic
- `engine/src/dispatcher/` - Platform-specific system execution

**Example Game:**
- `examples/flappy_bird/src/flappy_app.rs` - Application implementation
- `examples/flappy_bird/src/system/process_nn.rs` - Neural network system
- `examples/flappy_bird/src/resources/gene_handler.rs` - Genetic algorithm logic

## Asset Loading

**Textures:**
- Use `RenderState::load_texture_atlas(name, bytes)`
- Atlas name used in `Renderable::atlas_name`
- Supports PNG, JPEG via `image` crate

**Fonts:**
- Engine includes default font (loaded automatically)
- TTF fonts rasterized at runtime by FontManager
- Text rendered via `TextRenderData` returned from `get_text_instances()`

## Dependencies

**Core:**
- `wgpu` - WebGPU graphics API
- `winit` - Cross-platform windowing
- `specs` - ECS framework
- `fontdue` - TTF font rasterization

**Math/Utilities:**
- `cgmath` - Linear algebra
- `instant` - Cross-platform timing
- `bytemuck` - Pod types for GPU

**WASM:**
- `wasm-pack` required for web builds
- `wasm-bindgen` - JS interop
- `web-sys` - Web API bindings

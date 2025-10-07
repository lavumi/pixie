# Pixie - Render Engine

A lightweight 2D game engine built with Rust and WebGPU, featuring an ECS architecture and cross-platform support.

## Features

- **Cross-platform**: Runs natively (Windows/macOS/Linux) and in browsers via WebAssembly
- **ECS Architecture**: Clean entity-component-system design using specs
- **WebGPU Renderer**: Modern graphics API supporting Vulkan/Metal/DX12/WebGL
- **Generic Engine Core**: Reusable components for any 2D game
- **Runtime Font Rendering**: Dynamic TTF font rasterization with fontdue

## Installation

### Prerequisites

1. **Install Rust** (if not already installed)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **For WASM builds**, install wasm-pack:
   ```bash
   cargo install wasm-pack
   ```

### Clone the Repository
```bash
git clone https://github.com/lavumi/pixie.git
cd pixie
```

## Building and Running

### Native Build

Build and run examples:

```bash
# Development build
cargo run --bin flappy

# Release build (optimized)
cargo build --release --bin flappy
./target/release/flappy
```

### WebAssembly Build

Build for the web:

```bash
cd examples/flappy_bird
wasm-pack build --target web
```

**To run locally:**

1. Serve the directory with any HTTP server:
   ```bash
   # Using Python
   python -m http.server 8000

   # Using Node.js (npx)
   npx http-server -p 8000
   ```

2. Open `http://localhost:8000/index.html` in your browser

## Project Structure

```
pixie/
├── engine/                    # Generic game engine crate
│   ├── src/
│   │   ├── application.rs     # Application trait (game logic interface)
│   │   ├── engine.rs          # Engine runtime
│   │   ├── components/        # Reusable ECS components
│   │   ├── systems/           # Generic systems (physics, animation)
│   │   ├── resources/         # Shared resources (camera, input)
│   │   └── renderer/          # WebGPU renderer
│   └── assets/                # Engine assets (shaders, fonts)
│
└── examples/
    └── flappy_bird/           # Neural network Flappy Bird demo
        ├── README.md          # Game-specific documentation
        ├── src/
        │   ├── flappy_app.rs  # Game logic (implements Application)
        │   ├── components/    # Game-specific components
        │   ├── systems/       # AI & game systems
        │   └── resources/     # Gene pool management
        └── assets/            # Game assets (sprites)
```

## Creating a New Game

To create a new game, implement the `Application` trait:

```rust
use pixie::{Application, Engine};
use specs::World;
use winit::event::WindowEvent;

struct MyGame {
    // Your game state
}

impl Application for MyGame {
    fn init(&mut self, world: &mut World) {
        // Register components, insert resources, create entities
        world.register::<MyComponent>();
        world.insert(MyResource::default());
    }

    fn update(&mut self, world: &mut World, dt: f32) {
        // Run game logic and systems
    }

    fn handle_input(&mut self, world: &mut World, event: &WindowEvent) -> bool {
        // Handle input events
        false
    }

    fn get_camera_uniform(&self, world: &World) -> [[f32; 4]; 4] {
        // Provide camera matrix
    }

    fn get_tile_instances(&self, world: &World) -> HashMap<String, Vec<TileRenderData>> {
        // Provide sprite rendering data
    }

    fn get_text_instances(&self, world: &World) -> Vec<TextRenderData> {
        // Provide text rendering data
    }
}

// In your main function:
let app = MyGame::default();
let engine = Engine::new(app, window_attrs, &event_loop).await;

// Load game textures
engine.get_render_state_mut().load_texture_atlas("sprites", sprite_bytes);

event_loop.run_app(&mut engine).unwrap();
```

## Tech Stack

- **Rust**: Systems programming language
- **wgpu**: Cross-platform graphics API (WebGPU/Vulkan/Metal/DX12)
- **winit**: Window creation and event handling
- **specs**: Entity-Component-System framework
- **fontdue**: TTF font rasterization
- **instant**: Cross-platform time measurement

## Examples

### Flappy Bird with Neural Network AI

See [examples/flappy_bird/README.md](examples/flappy_bird/README.md) for a complete example featuring:
- Neural network AI agents
- Genetic algorithm evolution
- Real-time visualization

🎮 **[Live Demo](https://lavumi.net/wasm01)**

## Architecture

This engine follows a **generic runtime + game implementation** pattern:

### Engine Responsibilities
- Window management and event loop
- WebGPU rendering pipeline
- ECS world management
- Resource loading (textures, fonts)
- Input handling
- Camera and viewport management

### Application Responsibilities
- Game logic and rules
- Component registration
- System execution
- Entity creation
- Rendering data preparation

This separation allows the engine to be completely game-agnostic while providing all necessary infrastructure.

## Roadmap

See [REFACTOR_PLAN.md](REFACTOR_PLAN.md) for the complete refactoring plan.

- [x] Workspace structure separation
- [x] Component/System separation
- [x] Asset organization
- [x] Dynamic font rendering
- [x] Application trait abstraction
- [ ] Physics demo example
- [ ] Additional game examples

## Contributing

Contributions are welcome! Feel free to:
- Report bugs
- Suggest features
- Submit pull requests

## License

This project is open source and available under the MIT License.

---

**Built with ❤️ and Rust**

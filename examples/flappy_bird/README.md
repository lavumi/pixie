# Flappy Bird - Neural Network AI Demo

A Flappy Bird clone where AI agents learn to play using neural networks and genetic algorithms.

🎮 **[Live Demo](https://lavumi.net/wasm01)**

## Overview

Watch 100 AI-controlled birds learn to navigate through pipes over successive generations. Each bird has its own neural network brain, and the population evolves through natural selection and mutation.

## How It Works

### Neural Network

Each bird is controlled by a simple feed-forward neural network:

**Network Architecture:**
- **Input Layer** (2 neurons):
  - Distance to next pipe (X axis)
  - Height difference to pipe opening (Y axis)

- **Hidden Layer** (configurable neurons):
  - Weighted connections from input
  - Activation function applied

- **Output Layer** (1 neuron):
  - Binary decision: Jump or Don't Jump
  - Threshold determines action

**Decision Making:**
```
inputs = [pipe_x - bird_x, pipe_y - bird_y]
hidden = activate(weights_hidden × inputs + bias)
output = activate(weights_output × hidden + bias)
if output > threshold: jump()
```

### Genetic Algorithm

The population evolves through these steps:

1. **Initialization**
   - 100 birds spawn with random neural network weights
   - Each bird has unique "DNA" (network weights)

2. **Evaluation**
   - Birds play simultaneously
   - Fitness = survival time + distance traveled
   - Generation ends when all birds crash

3. **Selection**
   - Top performing birds selected as parents
   - Fitness-proportional selection ensures better genes survive

4. **Reproduction**
   - Parent genes combined (crossover)
   - Random mutations applied to weights
   - New generation created

5. **Iteration**
   - Process repeats
   - Each generation typically performs better than the last

**Evolution Parameters:**
```rust
POPULATION_SIZE: 100
MUTATION_RATE: 0.1
ELITE_COUNT: 10       // Best performers always survive
CROSSOVER_RATE: 0.7
```

## Game Mechanics

**Physics:**
- Gravity: Constant downward acceleration
- Jump: Upward velocity impulse
- Collision: Hit pipe or ground = death

**Pipes:**
- Scroll left at constant speed
- Random height variation
- Fixed gap size
- Respawn when off-screen

**Scoring:**
- Each pipe passed = +1 score
- Survival time contributes to fitness
- Generation tracks best score

## Controls

- **Any Key**: Start/Resume game
- **P**: Pause
- **R**: Force restart generation
- **ESC**: Quit

## Building and Running

### Native
```bash
# From repository root
cargo run --bin flappy --release
```

### WebAssembly
```bash
cd examples/flappy_bird
wasm-pack build --target web

# Serve with any HTTP server
python -m http.server 8000
# Open http://localhost:8000/index.html
```

## Code Structure

```
src/
├── flappy_app.rs       # Main game logic (implements Application trait)
├── components/         # Game-specific ECS components
│   ├── player.rs       # Bird component
│   ├── pipe.rs         # Pipe obstacle
│   └── dna.rs          # Neural network weights
├── systems/            # Game systems
│   ├── process_nn.rs   # Neural network processing
│   ├── update_player.rs# Bird physics
│   ├── update_pipe.rs  # Pipe movement
│   └── check_collision.rs # Collision detection
├── resources/          # Global resources
│   ├── gene_handler.rs # Genetic algorithm logic
│   └── score.rs        # Score tracking
└── builder.rs          # Entity creation helpers
```

## Observing Evolution

**Generation 1:**
- Random flapping
- Most birds crash immediately
- Few lucky survivors

**Generation 5-10:**
- Some birds learn basic timing
- Average survival increases
- Still many failures

**Generation 20+:**
- Consistent pipe navigation
- Multiple birds survive long
- Occasional perfect runs

**Generation 50+:**
- Optimized strategies emerge
- Minimal unnecessary jumps
- High success rate

## Customization

**Adjust Evolution Speed:**
```rust
// In game_configs.rs
const POPULATION_SIZE: usize = 100;  // More birds = slower but more diverse
const MUTATION_RATE: f32 = 0.1;      // Higher = more exploration
```

**Change Network Complexity:**
```rust
// In components/dna.rs
const HIDDEN_NEURONS: usize = 4;     // More neurons = more complex behavior
```

**Modify Physics:**
```rust
// In systems/update_player.rs
const GRAVITY: f32 = -0.5;
const JUMP_FORCE: f32 = 0.2;
```

## Performance

- **Native**: 60 FPS with 100+ birds
- **WASM**: 30-60 FPS depending on browser
- **Memory**: ~50MB typical usage

## Future Enhancements

- [ ] Save/load best neural networks
- [ ] Visualize neural network activations
- [ ] Multiple difficulty levels
- [ ] Tournament mode between generations
- [ ] Export trained networks

## Implementation Details

This demo showcases the engine's capabilities:
- **ECS Architecture**: Clean separation of data and logic
- **Real-time AI**: Neural networks processed every frame
- **Cross-platform**: Identical behavior native and web
- **Performance**: Handles 100+ entities at 60 FPS

See the [engine documentation](../../README.md) to build your own games!

---

**Watch evolution in action! 🐦🧬**

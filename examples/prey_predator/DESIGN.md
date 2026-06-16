# Prey/Predator Simulation Design

## Goal

This example is an ecosystem simulation built on Pixie. It models two agent
species:

- `Prey`: survives, avoids predators, and reproduces after living long enough.
- `Predator`: hunts prey and reproduces after eating enough prey.

The first implementation should prove the simulation shape, not the final
evolution system. It should create a stable, observable world where agents move
with directional sprites, perceive nearby agents through ray-style vision, and
update population counts through simple lifecycle rules.

## V1 Scope

V1 should include:

- A new workspace example crate at `examples/prey_predator`.
- A runnable native binary named `prey_predator`.
- Directional sprites for prey and predators.
- A torus world where agents wrap around the bounds.
- Prey and predator entities with species-specific lifecycle rules.
- Vision data collection with a fixed input shape per agent.
- A placeholder brain that uses the same input/output interface as the future
  neural network.
- HUD text showing basic simulation state.
- Unit tests for deterministic logic that does not require rendering.

V1 should not include:

- Full genetic evolution.
- A parameter tuning UI.
- Runtime asset editing.
- Primitive shape rendering in the engine.
- Debug rendering for every vision ray.
- Engine-level raycast or physics changes.

## Visual Direction

Agents must visibly communicate their facing direction. Simple circles are not
enough because the user cannot tell where an agent is looking.

Use small top-down marker sprites:

- `assets/prey.png`
  - Green or cyan body.
  - Small circular or oval body.
  - Short forward nose, bar, or triangle.
  - Softer shape, visually weaker than predators.
- `assets/predator.png`
  - Red or orange body.
  - Slightly larger body.
  - Longer forward nose or sharper triangular head.
  - More aggressive silhouette than prey.

Sprite orientation contract:

- The image's forward direction is positive local X.
- `Transform.rotation` is the agent facing direction in radians.
- Movement uses the same facing direction unless the brain outputs a negative
  speed, in which case the agent moves backward while still looking forward.
- The sprite should be centered around its body so rotation feels natural.

Pixie currently does not support per-sprite tinting. V1 should use separate PNG
atlases for prey and predator instead of a single white sprite recolored in
code.

Recommended initial asset style:

- Transparent PNG.
- Square canvas, for example `64x64`.
- Body centered around `(32, 32)`.
- Nose points to the right.
- Enough transparent padding so rotation does not clip.

Minimum acceptable V1 sprite:

- A filled body shape.
- A clearly visible forward marker attached to the body.
- A transparent background.
- No animation frames.
- Readable at approximately `0.6` to `1.0` world units on screen.

The simplest acceptable design is a circular body with a short rectangular or
triangular nose. This keeps the asset cheap to make while still making facing
direction obvious during simulation.

## World Model

The world is a 2D torus.

If an agent crosses a world bound, it appears on the opposite side:

- `x > half_width` wraps to `-half_width`.
- `x < -half_width` wraps to `half_width`.
- `y > half_height` wraps to `-half_height`.
- `y < -half_height` wraps to `half_height`.

This avoids agents getting stuck against walls and keeps the simulation focused
on agent behavior rather than boundary handling.

Rendering should use an orthographic camera with enough zoom to see the full
ecosystem. V1 does not need camera panning or zoom controls.

## Agents

Each agent has:

- A species.
- A position, size, and rotation through Pixie's `Transform`.
- A visible sprite through Pixie's `Sprite`.
- Movement state.
- Vision parameters.
- Brain parameters.
- Lifecycle state.
- Reproduction state.

Species:

```rust
pub enum Species {
    Prey,
    Predator,
}
```

Agent movement:

- Brain output controls `rotation_delta` and `speed`.
- Rotation is integrated each fixed update.
- Speed can be negative.
- Position changes along the current facing direction.
- Species can have different max speed and turn speed.

## Vision Model

Each agent has a fan of ray-like samples.

Vision base parameters:

- `max_distance`: maximum visible distance.
- `total_angle`: total field-of-view angle in radians.
- `ray_interval`: angle between adjacent rays in radians.

Ray count:

```text
floor(total_angle / ray_interval) + 1
```

The fan is centered on the agent's current facing direction. For example, with a
90 degree total angle and 15 degree interval, the rays cover:

```text
-45, -30, -15, 0, 15, 30, 45 degrees
```

V1 vision is not an engine physics raycast. It is a simulation query:

1. For each agent, inspect other agents in the world.
2. Convert each target into local polar coordinates relative to the viewer.
3. Assign the target to the closest matching ray if it is within `max_distance`
   and inside the vision fan.
4. Keep only the nearest target per ray.

Torus distance should be considered when measuring agent distance, so an agent
near the right edge can perceive an agent near the left edge through the wrap.

## Brain Interface

The future neural network should use the same interface from the start, even if
V1 uses a placeholder brain.

For each ray, the input contains:

```text
[target_type, normalized_distance]
```

`target_type` values:

- `0.0`: no target detected.
- `1.0`: prey detected.
- `-1.0`: predator detected.

`normalized_distance`:

- `distance / max_distance` when a target is detected.
- `1.0` when no target is detected.

Input length:

```text
ray_count * 2
```

Brain output:

```text
[rotation_delta, speed]
```

Output interpretation:

- `rotation_delta` is clamped to the species turn limit.
- `speed` is clamped to the species movement speed limit.
- Negative speed is allowed.

V1 placeholder behavior:

- Prey should turn away from visible predators.
- Predator should turn toward visible prey.
- If no relevant target is visible, agents wander with deterministic or seeded
  pseudo-random turning.

The placeholder brain should be isolated behind the same function shape that a
neural network brain will later use.

## Lifecycle Rules

Prey:

- Can reproduce after surviving for `prey_reproduction_age` seconds.
- Dies when eaten by a predator.
- Dies when older than `prey_max_age`.
- Cannot reproduce again until its reproduction cooldown expires.

Predator:

- Gains one food count when it eats a prey.
- Can reproduce after eating `predator_food_to_reproduce` prey.
- Dies when older than `predator_max_age`.
- Dies when it has not eaten for `predator_starvation_time` seconds.
- Cannot reproduce again until its reproduction cooldown expires.

Population control:

- Each species has a maximum population cap.
- Reproduction does nothing if the species is already at cap.
- New children spawn near the parent with a small random offset.
- Children start with age `0`, reproduction cooldown active, and species default
  lifecycle counters.

Predation:

- Predator eats prey when their positions are within `predation_radius`.
- If multiple prey are within range, the nearest prey is eaten.
- A prey can only be eaten once in a fixed update.

## Suggested Default Parameters

These are starting points, not final balance.

World:

| Parameter | Value |
| --- | ---: |
| Width | `40.0` |
| Height | `24.0` |
| Fixed timestep | Pixie default 60 Hz |

Initial population:

| Species | Value |
| --- | ---: |
| Prey | `80` |
| Predator | `12` |

Population caps:

| Species | Value |
| --- | ---: |
| Prey | `200` |
| Predator | `60` |

Movement:

| Species | Max speed | Max turn per second |
| --- | ---: | ---: |
| Prey | `5.0` | `4.0 rad` |
| Predator | `4.2` | `3.2 rad` |

Vision:

| Species | Max distance | Total angle | Ray interval |
| --- | ---: | ---: | ---: |
| Prey | `7.0` | `160 deg` | `20 deg` |
| Predator | `9.0` | `120 deg` | `15 deg` |

Lifecycle:

| Parameter | Value |
| --- | ---: |
| Prey reproduction age | `8.0 s` |
| Prey reproduction cooldown | `5.0 s` |
| Prey max age | `60.0 s` |
| Predator food to reproduce | `3` |
| Predator reproduction cooldown | `8.0 s` |
| Predator max age | `80.0 s` |
| Predator starvation time | `12.0 s` |
| Predation radius | `0.6` |

## ECS Design

Expected game-specific components:

```rust
pub enum Species {
    Prey,
    Predator,
}

pub struct Agent {
    pub species: Species,
    pub max_speed: f32,
    pub max_turn_speed: f32,
}

pub struct Vision {
    pub max_distance: f32,
    pub total_angle: f32,
    pub ray_interval: f32,
}

pub struct Brain {
    pub kind: BrainKind,
}

pub enum BrainKind {
    Placeholder,
    NeuralNetwork,
}

pub struct LifeCycle {
    pub age: f32,
    pub max_age: f32,
    pub time_since_food: f32,
    pub food_eaten: u32,
}

pub struct Reproduction {
    pub cooldown_remaining: f32,
    pub required_age: f32,
    pub required_food: u32,
}
```

Pixie components used directly:

- `Transform`
- `Sprite`
- `Text`
- `TextStyle`

Avoid using Pixie's current physics collision system for V1 predation. The
existing collision system is useful for demos, but predator/prey interaction is
better expressed as simple simulation logic at this stage.

## Resources

Expected resources:

```rust
pub struct SimulationConfig {
    pub world_width: f32,
    pub world_height: f32,
    pub initial_prey: usize,
    pub initial_predators: usize,
    pub max_prey: usize,
    pub max_predators: usize,
}

pub struct SimulationStats {
    pub elapsed_time: f32,
    pub prey_alive: usize,
    pub predators_alive: usize,
    pub prey_born: usize,
    pub predators_born: usize,
    pub prey_eaten: usize,
    pub prey_died_of_age: usize,
    pub predators_died: usize,
}

pub struct SpawnQueue {
    pub requests: Vec<SpawnRequest>,
}
```

The config should start as code constants. Runtime editing can be added later.

## System Order

The fixed update should run simulation systems in this order:

1. `collect_vision`
2. `process_brains`
3. `move_agents`
4. `wrap_world`
5. `process_predation`
6. `update_lifecycle`
7. `process_reproduction`
8. `spawn_queued_agents`
9. `cleanup_dead_agents`
10. `update_stats`

HUD can update in `Application::update`, because it only needs to reflect the
latest simulation state.

Reasoning:

- Vision should read the previous frame's positions.
- Movement should happen before predation.
- Predation should mark prey as dead before reproduction is processed.
- Spawns and despawns should happen after systems finish collecting decisions.

## Application Behavior

Input:

- `Space`: pause or resume.
- `R`: reset the simulation.
- `D`: reserved for future vision debug toggle.

Startup:

- Set camera zoom so the full torus world is visible.
- Insert config and stats resources.
- Spawn HUD text.
- Spawn initial prey and predator populations.

Reset:

- Despawn all agent entities.
- Reset stats.
- Spawn fresh initial populations.
- Keep HUD text entities.

## HUD

HUD text should show:

```text
Prey Predator Simulation
Time: 00.0
Prey: 80 of 200
Predators: 12 of 60
Born: P 0 Pred 0
Eaten: 0
Deaths: P 0 Pred 0
Space Pause R Reset
```

Use `Pred` for predator in compact fields. Avoid single-letter `R`, because it
is easy to confuse with reset or reproduction.

Pixie's current font atlas only rasterizes letters, digits, spaces, newlines,
colon, and period. HUD strings should avoid punctuation such as `/`, `|`, `-`,
and parentheses until the font atlas character set is expanded.

## Testing Plan

Unit tests:

- Ray count calculation handles exact and non-exact angle division.
- Vision input length is `ray_count * 2`.
- Target type encoding is stable.
- Torus wrapping maps coordinates to the opposite side.
- Torus distance picks the shorter wrapped delta.
- Prey reproduction becomes available only after survival age and cooldown.
- Predator reproduction becomes available only after enough food and cooldown.
- Species population cap prevents extra spawn requests.
- Predator starvation marks predator for death.
- Predation chooses the nearest prey inside radius.

Manual checks:

- `cargo run --bin prey_predator` opens a window.
- Prey and predator sprites visibly show facing direction.
- Agents wrap around world bounds.
- Predator count and prey count change over time.
- HUD updates without flicker or panics.

Workspace checks:

```bash
cargo check --workspace
cargo test --workspace
cargo clippy --all-targets
```

## Future Work

Neural evolution:

- Replace placeholder brain with feed-forward neural network evaluation.
- Define genome size from `input_size`, hidden layers, and output size.
- Add mutation and crossover.
- Track species-specific fitness.
- Decide whether prey and predators evolve independently or share a common
  brain format with species-specific fitness.

Debug visualization:

- Add `D` toggle for vision rays.
- Render rays with thin directional sprites or future primitive line support.
- Highlight detected target per ray.

Simulation tooling:

- Runtime parameter panel.
- Fast-forward mode.
- Seeded simulation replay.
- CSV or JSON export for population metrics.

Engine candidates:

- Optional sprite tinting.
- Primitive line rendering.
- Shared math helpers for torus distance and angle normalization.

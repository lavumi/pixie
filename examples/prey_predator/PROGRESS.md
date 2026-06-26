# Prey/Predator Progress

## Current Branch

- Branch: `example/prey_predator`
- Latest implementation commit: `be9bce5` - `✨ Add prey predator movement and camera controller`

## Implemented

### Example Structure

- Added the `examples/prey_predator` workspace example.
- Added a native runnable binary named `prey_predator`.
- Registered the example texture atlases through Pixie's `TextureAtlasAsset`.

### Visuals

- Added dedicated PNG sprites for prey and predators.
- The sprite forward direction is positive local X.
- Prey and predator are currently separate atlases because Pixie does not yet
  support per-sprite tinting.
- Added `area_fill.png` and `area_border.png` to make the simulation world
  bounds visible in world space.

### HUD

- Added a screen-space HUD for simulation status.
- HUD text remains fixed on screen while the world camera zooms or pans.
- Current HUD fields:
  - status
  - elapsed time
  - camera zoom
  - prey count
  - predator count
  - controls

### Agent Movement

- Added `AgentMotion` with:
  - signed speed
  - max absolute speed
  - signed angular velocity
  - max absolute angular velocity
- Agents spawn with randomized initial rotation, speed, and angular velocity.
- Negative speed is supported.
- Movement runs in `fixed_update`.
- Agents wrap around the configured torus world bounds.

### Camera

- Added an engine-level `CameraController` resource.
- Default camera controls:
  - mouse wheel zoom
  - left mouse drag pan
- Added an engine-level `WindowSize` resource so pixel drag movement can be
  converted into world-space camera movement.
- Removed camera zoom and pan input handling from the prey/predator app. The
  example now relies on the engine's default camera controller.

## Verified

The latest implementation was verified with:

```bash
cargo check --workspace
cargo test --workspace
cargo clippy --all-targets
```

All passed.

## Known Gaps

- The current movement is still random wandering, not brain-driven behavior.
- No vision/raycast-style perception has been implemented yet.
- No prey reproduction rule has been implemented yet.
- No predator eating, starvation, or reproduction rule has been implemented yet.
- No deterministic unit tests exist for prey/predator simulation logic yet.
- Camera controls are engine-level now, but there is not yet a public preset API
  beyond mutating the `CameraController` resource directly.

## Suggested Next Steps

1. Add vision parameter components and deterministic vision-query tests.
2. Add a placeholder brain interface returning `[rotation_delta, speed]`.
3. Replace random angular velocity with brain output while keeping clamps in
   `AgentMotion`.
4. Implement prey/predator interaction rules:
   - predator eats prey on contact
   - prey reproduces after survival time
   - predator reproduces after food count
   - predator starvation
5. Add debug visualization for selected agent vision only, not every agent by
   default.

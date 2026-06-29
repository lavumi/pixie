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

### Vision

- Added species-specific `Vision` distance, field-of-view, and ray interval.
- Vision runs before movement in each fixed update.
- Rays use circle intersections against snapshotted agent bounds.
- Each ray keeps only its nearest target and excludes the viewer itself.
- `VisionOutput` preserves the target entity, species, and surface distance.
- Neural-network input per ray is `[normalized_distance, prey, predator]`.
- The no-target input is `[1.0, 0.0, 0.0]`.
- V1 vision does not detect across the torus boundary.

### Camera

- Added an engine-level `CameraController` resource.
- Default camera controls:
  - mouse wheel zoom
  - left mouse drag pan
- Added an engine-level `WindowSize` resource so pixel drag movement can be
  converted into world-space camera movement.
- Removed camera zoom and pan input handling from the prey/predator app. The
  example now relies on the engine's default camera controller.

### Debug Rendering Foundation

- Added the engine-level `DebugDraw` transient resource.
- Added world-space debug lines with RGBA color and world-space thickness.
- Added a dedicated shader, pipeline, and reusable dynamic vertex buffer.
- Debug lines are expanded into triangles for consistent thickness across
  native and WebGL targets.
- Debug submissions are cleared after each render attempt.
- Added `SELECTION_DEBUG_PLAN.md` for selection, camera follow, deselection, and
  selected-agent vision visualization.

## Verified

The latest implementation was verified with:

```bash
cargo check --workspace
cargo test --workspace
cargo clippy --all-targets
```

All passed.

Vision also has deterministic tests for ray counts, ray-circle detection,
distance and angle exclusion, nearest-target selection, self exclusion, species
encoding, and the V1 non-wrapped boundary.

## Known Gaps

- The current movement is still random wandering, not brain-driven behavior.
- No prey reproduction rule has been implemented yet.
- No predator eating, starvation, or reproduction rule has been implemented yet.
- Camera controls are engine-level now, but there is not yet a public preset API
  beyond mutating the `CameraController` resource directly.

## Suggested Next Steps

1. Add a placeholder brain interface returning `[rotation_delta, speed]`.
2. Replace random angular velocity with brain output while keeping clamps in
   `AgentMotion`.
3. Implement prey/predator interaction rules:
   - predator eats prey on contact
   - prey reproduces after survival time
   - predator reproduces after food count
   - predator starvation
4. Add debug visualization for selected agent vision only, not every agent by
   default.

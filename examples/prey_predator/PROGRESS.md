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
- Vision runs before movement at a configurable sensing interval.
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

### Agent Selection

- Added engine-managed `RenderViewport` data shared by rendering and input.
- Added orthographic `Camera::screen_to_world` with letterbox handling.
- Added `SelectionState` to track cursor, click state, and selected entity.
- Left click selects the nearest agent center under the cursor.
- Empty-space and out-of-viewport clicks clear selection.
- Click detection coexists with the existing left-drag camera pan.
- The HUD displays the selected species alongside the visual outline.

### Selection Follow and Vision Debug

- The camera follows the selected agent while preserving zoom.
- Starting a left-button drag clears selection and resumes manual pan.
- `Escape` clears selection before falling through to engine exit behavior.
- Despawned selections are cleared automatically.
- A yellow circular outline marks the selected agent.
- Only the selected agent submits vision rays:
  - gray for no hit
  - green for prey
  - red for predator
- Hit rays stop at the recorded ray-circle intersection distance.

### Lifecycle, Predation, and Reproduction

- Added `LifeCycle` state for age, food timer, and food count.
- Added per-agent reproduction cooldown state.
- Added queue-based spawn and death decisions.
- Prey:
  - reproduce after the configured survival age and cooldown
  - die at the configured maximum age
- Predators:
  - eat the nearest prey inside the configured predation radius
  - wait for a configured minimum interval between successful attacks
  - reset starvation time and gain food when eating
  - consume the configured food count when reproducing
  - die from maximum age or starvation
- Predation uses the shortest torus distance at world boundaries.
- Species population caps include queued births and pending deaths.
- Children spawn near parents with configured random offset and active
  reproduction cooldown.
- HUD now reports births, prey eaten, prey age deaths, and predator deaths.

### Anchored UI Layout

- Added nine-position `UiAnchor` values from top-left through bottom-right.
- Added `UiTransform` with independent anchor and widget pivot.
- UI offsets use pixels with positive Y pointing down.
- UI roots can target the game `RenderViewport` or the full window.
- Screen text uses measured glyph bounds for pivot alignment.
- Screen UI renders through a depth-independent full-window layer.
- Migrated the simulation HUD to `TopLeft` anchor and `TopLeft` pivot.
- Added deterministic resize and letterbox margin tests.

### Responsive Viewport

- Added engine-level `ViewportMode`.
- Engine default remains `ViewportMode::FixedAspect` with letterboxing.
- The prey/predator example selects `ViewportMode::Expand`.
- Expand mode uses the full resized window without letterboxing.
- Orthographic camera vertical zoom stays fixed while horizontal visibility
  follows the new window aspect ratio.

### Fixed-Topology Neural Network

- Added a configurable fully connected feed-forward network.
- The default topology is `27 -> 16 -> 2` for the current 9-ray vision setup.
- Added Xavier-uniform initial weight generation and zero biases.
- Added `tanh` activation for hidden and output layers.
- Brain output controls normalized angular velocity and signed speed.
- Added per-agent `Brain` and `BrainOutput` components.
- Fixed update order is now vision, brain processing, then movement.
- Children clone the parent's genome and apply configurable Gaussian mutation
  and low-probability gene reset.
- Gene values are clamped to a configurable absolute limit.
- Evolution remains asynchronous and single-parent; NEAT and crossover are
  deferred.

### Population Performance

- Initial genomes now use a configurable positive speed-output bias, so the
  initial population favors forward movement while negative speed remains valid.
- Replaced all-agent vision scans with a uniform spatial grid.
- Added a distance broad-phase before exact ray-circle intersections.
- Predation now uses a torus-aware spatial grid instead of scanning every prey
  for every predator.
- Reuses vision output, candidate, neural input, and activation buffers.
- Vision and brain inference default to 15 Hz while movement remains at 60 Hz.
- Added regression coverage for forward-biased initialization, grid-boundary
  detection, and 1,024-agent vision collection.

## Verified

The latest implementation was verified with:

```bash
cargo check --workspace
cargo test --workspace
cargo clippy --all-targets -- -D warnings
```

All passed.

Vision also has deterministic tests for ray counts, ray-circle detection,
distance and angle exclusion, nearest-target selection, self exclusion, species
encoding, and the V1 non-wrapped boundary.

## Known Gaps

- There is no runtime brain inspector or genome export.
- NEAT, mate selection, and crossover are not implemented.
- Camera controls are engine-level now, but there is not yet a public preset API
  beyond mutating the `CameraController` resource directly.

## Suggested Next Steps

1. Add selected-agent brain outputs and lifecycle values to the debug HUD.
2. Add deterministic simulation seeds and genome serialization for experiments.
3. Tune lifecycle and mutation parameters from observed population behavior.

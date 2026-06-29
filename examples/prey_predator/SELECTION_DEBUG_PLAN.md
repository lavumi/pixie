# Agent Selection and Debug Plan

## Goal

Allow one simulation agent to be inspected without changing simulation
behavior. Selecting an agent should expose its vision state, keep it visible,
and make selection state easy to enter and leave.

## Interaction Contract

### Select

- Left click converts the cursor from screen space to world space.
- Agents whose circular bounds contain the cursor are selection candidates.
- If bounds overlap, select the candidate whose center is nearest to the
  cursor.
- Draw a visible selection outline around the selected agent.

### Camera Follow

- Move the world camera target to the selected agent every variable update.
- Mouse-wheel zoom remains available while following.
- Starting a manual camera drag clears the selection and stops following.

### Deselect

- Left clicking empty world space clears the selection.
- `Escape` clears the selection when one exists.
- With no selection, `Escape` keeps its engine-level exit behavior.
- Clearing selection also removes the outline and vision debug drawing.

### Vision Debug

Only the selected agent submits vision debug primitives.

- No hit: gray ray ending at `Vision::max_distance`.
- Prey hit: green ray ending at the ray-circle intersection.
- Predator hit: red ray ending at the ray-circle intersection.
- A selection outline identifies the ray origin.

## Implementation Phases

### Phase 1: Engine DebugLine Renderer

Status: complete

- Add an engine-managed `DebugDraw` resource.
- Support world-space start/end positions, RGBA color, and world-space
  thickness.
- Expand each line into triangles for portable thickness on native and WebGL.
- Add a dedicated shader, dynamic GPU vertex buffer, and render pipeline.
- Extract submitted lines into immutable frame data.
- Clear submissions after each render attempt.
- Add CPU-side resource, extraction, and geometry tests.

Completion criteria:

- Applications can submit a line through `DebugDraw`.
- Lines use the world camera and coexist with sprites and text.
- Empty frames issue no debug draw call.
- Repeated frames reuse allocated CPU and GPU capacity.

### Phase 2: Coordinate Conversion and Selection

Status: complete

- Add orthographic `Camera::screen_to_world`.
- Account for the actual render viewport and window size.
- Track the latest cursor position.
- Add a prey/predator `SelectionState` resource.
- Implement nearest candidate selection and empty-space deselection.
- Add deterministic coordinate conversion and selection tests.

Implemented interaction details:

- Selection occurs on left-button release.
- Pointer movement up to 4 physical pixels is treated as a click.
- Larger movement remains available to the existing left-drag camera pan.
- Clicking outside the render viewport or outside every agent clears selection.
- The HUD shows `Selected: None`, `Selected: Prey`, or `Selected: Predator`.

### Phase 3: Camera Follow

Status: complete

- Center the camera on the selected agent during variable updates.
- Preserve the current zoom.
- Clear selection when left-button camera dragging starts.
- Clear stale selection when the selected entity no longer exists.

Implemented interaction details:

- Camera position follows the selected transform every variable update.
- Camera zoom is not modified by follow.
- Crossing the 4-pixel click threshold clears selection before pan continues.
- `Escape` clears an active selection; a second `Escape` exits through the
  engine's default input handling.

### Phase 4: Selected-Agent Debug Drawing

Status: complete

- Submit a selection outline using short line segments.
- Submit rays from the selected agent's current `VisionOutput`.
- Use gray, green, and red hit-state colors.
- Stop hit rays at their recorded intersection distance.
- Submit no agent debug primitives when selection is empty.

Implemented visual details:

- Selection outline: yellow 24-segment circle.
- Missed ray: gray and drawn to maximum vision distance.
- Prey hit: green and drawn to the recorded intersection.
- Predator hit: red and drawn to the recorded intersection.
- Debug geometry is resubmitted only for the selected agent each frame.

### Phase 5: Manual Verification

Status: pending

- Select prey and predator agents at multiple zoom levels.
- Verify overlap selection chooses the nearest center.
- Verify wheel zoom continues while following.
- Verify drag and empty-space click deselect.
- Verify `Escape` deselects before it exits the application.
- Verify ray colors and endpoints match detected targets.

## Deliberate Constraints

- Debug line thickness is measured in world units, so it scales with camera
  zoom.
- Debug lines are transient submissions, not ECS entities.
- V1 vision and its debug rays do not cross the torus boundary.
- Only one agent can be selected at a time.

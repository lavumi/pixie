use hecs::World;
use crate::components::{Animation, Tile};
use crate::resources::{DeltaTime, ResourceContainer};

/// Update animation system - advances sprite animation frames based on time
pub fn update_animation(world: &mut World, resources: &mut ResourceContainer) {
    let dt = resources.get::<DeltaTime>()
        .expect("DeltaTime resource not found");

    // Query for entities with both Tile and Animation components
    for (_entity, (tile, animation)) in world.query_mut::<(&mut Tile, &mut Animation)>() {
        // Skip if animation is finished and not looping
        if animation.finished && !animation.loop_animation {
            continue;
        }

        animation.elapsed_time += dt.0;

        // Check if it's time to advance frame
        if animation.elapsed_time >= animation.frame_duration {
            animation.elapsed_time = 0.0;
            animation.current_frame += 1;

            // Handle frame overflow
            if animation.current_frame >= animation.frame_count {
                if animation.loop_animation {
                    animation.current_frame = 0;
                } else {
                    animation.current_frame = animation.frame_count - 1;
                    animation.finished = true;
                    continue;
                }
            }

            // Calculate UV coordinates based on atlas layout
            let frame_x = animation.current_frame % animation.atlas_columns;
            let frame_y = animation.current_frame / animation.atlas_columns;

            let u_size = 1.0 / animation.atlas_columns as f32;
            let v_size = 1.0 / animation.atlas_rows as f32;

            tile.uv = [
                frame_x as f32 * u_size,           // u_min
                (frame_x + 1) as f32 * u_size,     // u_max
                frame_y as f32 * v_size,           // v_min
                (frame_y + 1) as f32 * v_size,     // v_max
            ];
        }
    }
}
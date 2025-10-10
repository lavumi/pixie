use specs::{Join, Read, System, WriteStorage};

use crate::components::{Animation, Tile};
use crate::resources::DeltaTime;

pub struct UpdateAnimation;

impl<'a> System<'a> for UpdateAnimation {
    type SystemData = (
        WriteStorage<'a, Tile>,
        WriteStorage<'a, Animation>,
        Read<'a, DeltaTime>
    );

    fn run(&mut self, (mut tile, mut anime, dt): Self::SystemData) {
        for (t, a) in (&mut tile, &mut anime).join() {
            // Skip if animation is finished and not looping
            if a.finished && !a.loop_animation {
                continue;
            }

            a.elapsed_time += dt.0;

            // Check if it's time to advance frame
            if a.elapsed_time >= a.frame_duration {
                a.elapsed_time = 0.0;
                a.current_frame += 1;

                // Handle frame overflow
                if a.current_frame >= a.frame_count {
                    if a.loop_animation {
                        a.current_frame = 0;
                    } else {
                        a.current_frame = a.frame_count - 1;
                        a.finished = true;
                        continue;
                    }
                }

                // Calculate UV coordinates based on atlas layout
                let frame_x = a.current_frame % a.atlas_columns;
                let frame_y = a.current_frame / a.atlas_columns;

                let u_size = 1.0 / a.atlas_columns as f32;
                let v_size = 1.0 / a.atlas_rows as f32;

                t.uv = [
                    frame_x as f32 * u_size,           // u_min
                    (frame_x + 1) as f32 * u_size,     // u_max
                    frame_y as f32 * v_size,           // v_min
                    (frame_y + 1) as f32 * v_size,     // v_max
                ];
            }
        }
    }
}
use hecs::World;
use pixie::ResourceContainer;
use rand::Rng;
use rand::rngs::ThreadRng;

use crate::components::{Pipe, Transform};
use crate::resources::Score;
use pixie::DeltaTime;
use crate::game_configs::{GAME_SPEED, HOLE_SIZE};
use crate::flappy_app::Stage;

/// Scroll pipes and respawn them when off-screen
pub fn scroll_pipe(world: &mut World, resources: &mut ResourceContainer) {
    let dt_value = resources.get::<DeltaTime>().expect("DeltaTime resource not found").0;
    let stage = resources.get::<Stage>().expect("Stage resource not found");

    // Only run when game is in Run stage
    if *stage != Stage::Run {
        return;
    }

    // Update score based on time
    if let Some(score) = resources.get_mut::<Score>() {
        score.0 += dt_value;
    }

    // Collect entities that need repositioning and normal updates
    let mut to_reposition: Vec<(hecs::Entity, u8, [f32; 3], [f32; 2], f32)> = Vec::new();
    let mut to_update: Vec<(hecs::Entity, [f32; 3])> = Vec::new();

    for (entity, (pipe, transform)) in world.query::<(&Pipe, &Transform)>().iter() {
        let mut new_transform = transform.clone();
        new_transform.position[0] -= dt_value * GAME_SPEED;

        if new_transform.position[0] + new_transform.size[0] / 2.0 < -6.0 {
            to_reposition.push((
                entity,
                pipe.pipe_index,
                new_transform.position,
                new_transform.size,
                pipe.reposition_size
            ));
        } else {
            // Collect for update after query is done
            to_update.push((entity, new_transform.position));
        }
    }

    // Apply normal position updates
    for (entity, position) in to_update {
        if let Ok(mut t) = world.get::<&mut Transform>(entity) {
            t.position = position;
        }
    }

    // Reposition pipes with new random height
    if !to_reposition.is_empty() {
        let rng = resources.get_mut::<ThreadRng>().expect("ThreadRng resource not found");
        let rand = rng.gen_range(1.0..9.0);

        for (entity, pipe_index, mut position, mut size, reposition_size) in to_reposition {
            position[0] += reposition_size;

            match pipe_index {
                0 => {
                    position[1] = rand - 6.0;
                }
                1 => {
                    position[1] = (rand - 6.0) * 0.5 - 4.0;
                    size[1] = rand;
                }
                2 => {
                    position[1] = rand + HOLE_SIZE - 4.0;
                }
                3 => {
                    position[1] = (rand + HOLE_SIZE - 4.0) * 0.5 + 5.5;
                    size[1] = 13.0 - (rand + HOLE_SIZE);
                }
                _ => {}
            }

            // Update transform
            if let Ok(mut transform) = world.get::<&mut Transform>(entity) {
                transform.position = position;
                transform.size = size;
            }
        }
    }
}

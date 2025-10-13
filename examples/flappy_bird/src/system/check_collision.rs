use hecs::World;
use pixie::ResourceContainer;

use crate::components::{DNA, Pipe, Player, Transform};
use crate::resources::{GeneHandler, Score};
use crate::flappy_app::Stage;

/// Check collisions between players and pipes/boundaries
pub fn check_collision(world: &mut World, resources: &mut ResourceContainer) {
    let stage = resources.get::<Stage>().expect("Stage resource not found");

    // Only run when game is in Run stage
    if *stage != Stage::Run {
        return;
    }

    let score = resources.get::<Score>().expect("Score resource not found").0;

    // Collect entities to delete (can't delete while iterating)
    let mut entities_to_delete = Vec::new();

    // Check each player
    for (entity, (_player, player_tr, dna)) in world.query::<(&Player, &Transform, &DNA)>().iter() {
        let pt = player_tr.position;

        // Check boundary collision
        if pt[1] < -7.0 || pt[1] > 9.0 {
            entities_to_delete.push((entity, dna.index));
            continue;
        }

        // Check pipe collision
        for (_pipe_entity, (_pipe, pipe_tr)) in world.query::<(&Pipe, &Transform)>().iter() {
            // Find closest point on pipe to player
            let obstacle_point = [
                if pt[0] > pipe_tr.position[0] + pipe_tr.size[0] * 0.5 {
                    pipe_tr.position[0] + pipe_tr.size[0] * 0.5
                } else if pt[0] < pipe_tr.position[0] - pipe_tr.size[0] * 0.5 {
                    pipe_tr.position[0] - pipe_tr.size[0] * 0.5
                } else {
                    pt[0]
                },
                if pt[1] > pipe_tr.position[1] + pipe_tr.size[1] * 0.5 {
                    pipe_tr.position[1] + pipe_tr.size[1] * 0.5
                } else if pt[1] < pipe_tr.position[1] - pipe_tr.size[1] * 0.5 {
                    pipe_tr.position[1] - pipe_tr.size[1] * 0.5
                } else {
                    pt[1]
                }
            ];

            let dist_pow = (obstacle_point[0] - pt[0]) * (obstacle_point[0] - pt[0])
                         + (obstacle_point[1] - pt[1]) * (obstacle_point[1] - pt[1]);

            if dist_pow < 0.2 {
                entities_to_delete.push((entity, dna.index));
                break;
            }
        }
    }

    // Delete collided entities and record scores
    if let Some(gene_handler) = resources.get_mut::<GeneHandler>() {
        for (entity, dna_index) in entities_to_delete {
            gene_handler.set_score(dna_index, score);
            let _ = world.despawn(entity);
        }
    }
}

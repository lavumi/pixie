use hecs::World;
use pixie::ResourceContainer;

use crate::components::{Player, Transform};
use crate::game_configs::{GRAVITY, JUMP_FORCE};
use pixie::DeltaTime;
use crate::flappy_app::Stage;

/// Update player physics - applies gravity and jump force
pub fn update_player(world: &mut World, resources: &mut ResourceContainer) {
    let dt = resources.get::<DeltaTime>().expect("DeltaTime resource not found");
    let stage = resources.get::<Stage>().expect("Stage resource not found");

    // Only run when game is in Run stage
    if *stage != Stage::Run {
        return;
    }

    for (_entity, (player, transform)) in world.query_mut::<(&mut Player, &mut Transform)>() {
        player.force = if player.jump {
            player.jump = false;
            JUMP_FORCE * dt.0
        } else {
            player.force - GRAVITY * dt.0
        };

        transform.position[1] += player.force;
    }
}

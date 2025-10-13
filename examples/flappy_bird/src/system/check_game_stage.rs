use hecs::World;
use pixie::ResourceContainer;

use crate::components::Player;
use crate::resources::GameFinished;

/// Check if game should end (no players left)
pub fn check_game_stage(world: &mut World, resources: &mut ResourceContainer) {
    let player_count = world.query::<&Player>().iter().count();

    if player_count == 0 {
        resources.insert(GameFinished(true));
    }
}

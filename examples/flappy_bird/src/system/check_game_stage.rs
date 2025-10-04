use specs::{ReadStorage, System, Write};
use crate::components::Player;

use crate::resources::{GameFinished};

pub struct CheckGameStage;

impl<'a> System<'a> for CheckGameStage {
    type SystemData = (
        Write<'a, GameFinished>,
        ReadStorage<'a, Player>
    );

    fn run(&mut self, (mut stage, players): Self::SystemData) {
        if players.is_empty() {
            *stage = GameFinished(true);
        }
    }
}
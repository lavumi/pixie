use specs::{Join, Read, ReadStorage, System, WriteStorage};

use crate::components::{Background, Transform};
use crate::game_configs::GAME_SPEED;
use crate::resources::{DeltaTime};

pub struct ScrollBackground;

impl<'a> System<'a> for ScrollBackground {
    type SystemData = (
        ReadStorage<'a, Background>,
        WriteStorage<'a, Transform>,
        Read<'a, DeltaTime>
    );

    fn run(&mut self, (sc, mut tf, dt): Self::SystemData) {
        for ( scroll, transform) in ( &sc, &mut tf).join() {
            transform.position[0] -= dt.0 * GAME_SPEED;
            if transform.position[0] + transform.size[0]  / 2.0 < -6.0 {
                transform.position[0] += scroll.reposition_size;
            }
        }
    }
}
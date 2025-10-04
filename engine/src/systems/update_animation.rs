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

    fn run(&mut self, (mut tile, mut anime,  dt): Self::SystemData) {
        for ( t, a) in ( &mut tile, &mut anime).join() {
            a.delta += dt.0;
            if a.delta > 0.2 {
                a.delta = 0.;
                a.index += 1;
                if a.index >= 4 {
                    a.index = 0;
                }

                t.uv = [
                    0.25 * a.index as f32,
                    0.25 * (a.index + 1) as f32,
                    0.0,
                    1.0
                ];
            }


        }
    }
}
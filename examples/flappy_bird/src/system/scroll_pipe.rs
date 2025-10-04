use specs::{Join, Read, ReadStorage, System, Write, WriteStorage};

use crate::components::{ Pipe, Transform};
use crate::resources::{DeltaTime, Score};
use rand::Rng;
use rand::rngs::ThreadRng;
use crate::game_configs::{GAME_SPEED, HOLE_SIZE};

pub struct UpdatePipe;



impl<'a> System<'a> for UpdatePipe {
    type SystemData = (
        ReadStorage<'a, Pipe>,
        WriteStorage<'a, Transform>,
        Read<'a, DeltaTime>,
        Write<'a, ThreadRng>,
        Write<'a, Score>
    );

    fn run(&mut self, (pipes, mut tf, dt, mut rng, mut score): Self::SystemData) {
        let mut rand = -1.0f32;
        score.0 += dt.0;
        for (p, transform) in ( &pipes, &mut tf).join() {
            transform.position[0] -= dt.0 * GAME_SPEED;
            if transform.position[0] + transform.size[0]  / 2.0 < -6.0 {
                if rand < 0.0 {
                    rand = rng.gen_range(1.0..9.0);
                }
                transform.position[0] += p.reposition_size;
                match p.pipe_index {
                    0 => {
                        transform.position[1] = rand - 6.0;
                    }
                    1 => {
                        transform.position[1] = (rand - 6.0) * 0.5 - 4.0;
                        transform.size[1] = rand;
                    }
                    2 => {
                        transform.position[1] = rand + HOLE_SIZE - 4.0;
                    }
                    3 => {
                        transform.position[1] = (rand + HOLE_SIZE -4.0)  * 0.5 + 5.5;
                        transform.size[1] = 13.0 - (rand + HOLE_SIZE );
                    }
                    _ => {}
                }
            }
        }
    }
}
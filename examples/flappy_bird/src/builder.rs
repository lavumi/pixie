use rand::rngs::ThreadRng;
use specs::{Builder, World, WorldExt};
use crate::components::*;
use crate::game_configs::HOLE_SIZE;
use crate::resources::GeneHandler;

pub fn background(world: &mut World) {
    world.create_entity()
        .with(Tile {
            uv: [0.0, 1.0, 0.0, 1.0],
            atlas: "bg".to_string(),
        })
        .with(Transform {
            position: [0., 1., 0.2],
            size: [16.0, 16.0],
        })
        .with(Background {
            reposition_size: 32.0,
        })
        .build();
    world.create_entity()
        .with(Tile {
            uv: [0.0, 1.0, 0.0, 1.0],
            atlas: "bg".to_string(),
        })
        .with(Transform {
            position: [16., 1., 0.2],
            size: [16.0, 16.0],
        })
        .with(Background {
            reposition_size: 32.0,
        })
        .build();
    for i in 0..16 {
        let pos = i as f32 - 7.5;
        world.create_entity()
            .with(Tile {
                uv: [0.0, 0.125, 0.75, 1.0],
                atlas: "tile".to_string(),
            })
            .with(Transform {
                position: [pos, -8., 0.2],
                size: [1.0, 2.0],
            })
            .with(Background {
                reposition_size: 16.0,
            })
            .build();
    }
}

pub fn pipe(world: &mut World, pos: f32) {
    use rand::Rng;
    let rand;
    {
        let mut rng = world.write_resource::<ThreadRng>();
        rand = rng.gen_range(3.0..7.0);
    }
    world.create_entity()
        .with(Tile {
            uv: [0.0, 0.25, 0., 0.25],
            atlas: "tile".to_string(),
        })
        .with(Transform {
            position: [pos, rand - 6.0, 0.2],
            size: [2.0, 2.0],
        })
        .with(Pipe {
            reposition_size: 16.0,
            pipe_index: 0,
        })
        .with(PipeTarget {})
        .build();
    world.create_entity()
        .with(Tile {
            uv: [0.0, 0.25, 0.25, 0.25],
            atlas: "tile".to_string(),
        })
        .with(Transform {
            position: [pos, rand * 0.5 - 7.0, 0.2],
            size: [2.0, rand],
        })
        .with(Pipe {
            reposition_size: 16.0,
            pipe_index: 1,
        })
        .build();

    world.create_entity()
        .with(Tile {
            uv: [0.0, 0.25, 0.5, 0.75],
            atlas: "tile".to_string(),
        })
        .with(Transform {
            position: [pos, rand + HOLE_SIZE - 4.0, 0.2],
            size: [2.0, 2.0],
        })
        .with(Pipe {
            reposition_size: 16.0,
            pipe_index: 2,
        })
        .build();
    world.create_entity()
        .with(Tile {
            uv: [0.0, 0.25, 0.5, 0.5],
            atlas: "tile".to_string(),
        })
        .with(Transform {
            position: [pos, (rand + HOLE_SIZE - 4.0) * 0.5 + 5.5, 0.2],
            size: [2.0, 13.0 - (rand + HOLE_SIZE)],
        })
        .with(Pipe {
            reposition_size: 16.0,
            pipe_index: 3,
        })
        .build();
}

pub fn player(world: &mut World) {
    world.create_entity()
        .with(Tile {
            uv: [0.0, 0.25, 0.0, 1.0],
            atlas: "player".to_string(),
        })
        .with(Transform {
            position: [0., 0., 0.3],
            size: [1., 1.],
        })
        .with(Player::default())
        .with(Animation::default())
        .build();
}

pub fn ai_player(world: &mut World) {
    let dna = world.write_resource::<GeneHandler>().get_dna();
    world.create_entity()
        .with(Tile {
            uv: [0.0, 0.25, 0.0, 1.0],
            atlas: "player".to_string(),
        })
        .with(Transform {
            position: [0., 0., 0.3],
            size: [1., 1.],
        })
        .with(Player::default())
        .with(Animation::default())
        .with(dna)
        .build();
}
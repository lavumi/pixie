use rand::rngs::ThreadRng;
use rand::Rng;
use hecs::World;
use pixie::ResourceContainer;
use crate::components::*;
use crate::game_configs::HOLE_SIZE;
use crate::resources::GeneHandler;

pub fn background(world: &mut World) {
    // Background sprites
    world.spawn((
        Tile {
            uv: [0.0, 1.0, 0.0, 1.0],
            atlas: "bg".to_string(),
        },
        Transform {
            position: [0., 1., 0.2],
            size: [16.0, 16.0],
        },
        Background {
            reposition_size: 32.0,
        },
    ));

    world.spawn((
        Tile {
            uv: [0.0, 1.0, 0.0, 1.0],
            atlas: "bg".to_string(),
        },
        Transform {
            position: [16., 1., 0.2],
            size: [16.0, 16.0],
        },
        Background {
            reposition_size: 32.0,
        },
    ));

    // Ground tiles
    for i in 0..16 {
        let pos = i as f32 - 7.5;
        world.spawn((
            Tile {
                uv: [0.0, 0.125, 0.75, 1.0],
                atlas: "tile".to_string(),
            },
            Transform {
                position: [pos, -8., 0.2],
                size: [1.0, 2.0],
            },
            Background {
                reposition_size: 16.0,
            },
        ));
    }
}

pub fn pipe(world: &mut World, pos: f32) {
    // Generate random pipe height
    // Note: We need to pass ThreadRng from resources since we can't access resources here
    // For now, create a temporary RNG - this should be refactored to accept resources
    let mut rng = ThreadRng::default();
    let rand = rng.gen_range(3.0..7.0);

    // Top pipe cap
    world.spawn((
        Tile {
            uv: [0.0, 0.25, 0., 0.25],
            atlas: "tile".to_string(),
        },
        Transform {
            position: [pos, rand - 6.0, 0.3],
            size: [2.0, 2.0],
        },
        Pipe {
            reposition_size: 16.0,
            pipe_index: 0,
        },
        PipeTarget {},
    ));

    // Top pipe body
    world.spawn((
        Tile {
            uv: [0.0, 0.25, 0.25, 0.25],
            atlas: "tile".to_string(),
        },
        Transform {
            position: [pos, rand * 0.5 - 7.0, 0.3],
            size: [2.0, rand],
        },
        Pipe {
            reposition_size: 16.0,
            pipe_index: 1,
        },
    ));

    // Bottom pipe cap
    world.spawn((
        Tile {
            uv: [0.0, 0.25, 0.5, 0.75],
            atlas: "tile".to_string(),
        },
        Transform {
            position: [pos, rand + HOLE_SIZE - 4.0, 0.3],
            size: [2.0, 2.0],
        },
        Pipe {
            reposition_size: 16.0,
            pipe_index: 2,
        },
    ));

    // Bottom pipe body
    world.spawn((
        Tile {
            uv: [0.0, 0.25, 0.5, 0.5],
            atlas: "tile".to_string(),
        },
        Transform {
            position: [pos, (rand + HOLE_SIZE - 4.0) * 0.5 + 5.5, 0.3],
            size: [2.0, 13.0 - (rand + HOLE_SIZE)],
        },
        Pipe {
            reposition_size: 16.0,
            pipe_index: 3,
        },
    ));
}

pub fn ai_player(world: &mut World) {
    // TODO: This needs to access GeneHandler from resources
    // For now, create a default DNA - will be overridden by proper gene handler
    let dna = DNA {
        hidden_layers: [4, 3],
        genes: [0.0; crate::game_configs::GENE_SIZE],
        index: 0,
    };

    world.spawn((
        Tile {
            uv: [0.0, 0.25, 0.0, 1.0],
            atlas: "player".to_string(),
        },
        Transform {
            position: [0., 0., 0.3],
            size: [1., 1.],
        },
        Player::default(),
        Animation {
            current_frame: 0,
            frame_count: 4,
            frame_duration: 0.2,
            elapsed_time: 0.0,
            loop_animation: true,
            finished: false,
            atlas_columns: 4,
            atlas_rows: 1,
        },
        dna,
    ));
}

/// Helper to spawn ai_player with proper DNA from resources
pub fn ai_player_with_resources(world: &mut World, resources: &mut ResourceContainer) {
    let dna = if let Some(gene_handler) = resources.get_mut::<GeneHandler>() {
        gene_handler.get_dna()
    } else {
        DNA {
            hidden_layers: [4, 3],
            genes: [0.0; crate::game_configs::GENE_SIZE],
            index: 0,
        }
    };

    world.spawn((
        Tile {
            uv: [0.0, 0.25, 0.0, 1.0],
            atlas: "player".to_string(),
        },
        Transform {
            position: [0., 0., 0.3],
            size: [1., 1.],
        },
        Player::default(),
        Animation {
            current_frame: 0,
            frame_count: 4,
            frame_duration: 0.2,
            elapsed_time: 0.0,
            loop_animation: true,
            finished: false,
            atlas_columns: 4,
            atlas_rows: 1,
        },
        dna,
    ));
}

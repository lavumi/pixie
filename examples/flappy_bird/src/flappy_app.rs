use std::collections::HashMap;
use rand::rngs::ThreadRng;
use specs::{Join, World, WorldExt};
use winit::event::{ElementState, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

use pixie::{Application, Camera, DeltaTime};
use pixie::renderer::{TileRenderData, TextRenderData};

use crate::builder::{background, pipe, ai_player};
use crate::components::*;
use crate::game_configs::GENE_SIZE;
use crate::resources::*;
// systems are now built and owned by the engine

#[allow(dead_code)]
#[derive(Debug, Clone, Eq, PartialEq, Hash, Copy)]
pub enum Stage { Ready, Run, Pause, End }

impl Default for Stage {
    fn default() -> Self {
        Stage::Ready
    }
}

pub struct FlappyApplication { stage: Stage }

impl Default for FlappyApplication {
    fn default() -> Self {
        FlappyApplication {
            stage: Stage::Ready,
        }
    }
}

impl FlappyApplication {
    fn init_game(&mut self, world: &mut World) {
        world.delete_all();
        background(world);

        pipe(world, 16.);
        pipe(world, 8.);

        for _ in 0..100 {
            ai_player(world);
        }

        let mut finished = world.write_resource::<GameFinished>();
        *finished = GameFinished(false);

        let mut inputs = world.write_resource::<pixie::InputHandler>();
        *inputs = pixie::InputHandler::default();

        let mut score = world.write_resource::<Score>();
        *score = Score::default();

        self.stage = Stage::Ready;
    }

    fn check_game_finished(&mut self, world: &World) {
        let finished = world.read_resource::<GameFinished>();
        if finished.0 {
            self.stage = Stage::End;
        }
    }

    pub fn get_gene_data(&self, world: &World) -> ([f32; GENE_SIZE], [f32; 2]) {
        let gene_handler = world.read_resource::<GeneHandler>();

        let player = world.read_storage::<Player>();
        let transform = world.read_storage::<pixie::Transform>();
        let dna = world.read_storage::<DNA>();
        let pipe = world.read_storage::<PipeTarget>();
        let last_player = (&player, &transform, &dna).join().last();

        let mut pipe_position = [99.0, 0.0];
        for (_, pipe_tr) in (&pipe, &transform).join() {
            if pipe_tr.position[0] > -3.0 && pipe_position[0] > pipe_tr.position[0] {
                pipe_position = [pipe_tr.position[0], pipe_tr.position[1]];
            }
        }

        let (position, index) = match last_player {
            None => ([0.0, 0.0], 0),
            Some(p) => ([p.1.position[0], p.1.position[1]], p.2.index),
        };

        let input_data = [
            pipe_position[0] - position[0],
            pipe_position[1] - position[1],
        ];

        let gene = gene_handler.get_alive_gene(index);
        (gene, input_data)
    }
}

impl Application for FlappyApplication {
    fn init(&mut self, world: &mut World) {
        // Register components
        world.register::<pixie::Transform>();
        world.register::<pixie::Collider>();
        world.register::<pixie::Tile>();
        world.register::<Background>();
        world.register::<Player>();
        world.register::<Pipe>();
        world.register::<PipeTarget>();
        world.register::<pixie::Animation>();
        world.register::<pixie::Text>();
        world.register::<DNA>();

        // Insert resources
        world.insert(Camera::init_orthographic(9, 500.0 / 900.0));
        world.insert(DeltaTime(0.05));
        world.insert(GameFinished(false));
        world.insert(ThreadRng::default());
        world.insert(Score::default());
        world.insert(pixie::InputHandler::default());
        world.insert(GeneHandler::default());

        // Initialize game
        self.init_game(world);
    }

    fn update(&mut self, world: &mut World, dt: f32) {
        self.check_game_finished(world);

        if self.stage == Stage::End {
            world.write_resource::<GeneHandler>().process_generation();
            self.init_game(world);
            self.stage = Stage::Run;
        }

        if self.stage != Stage::Run {
            return;
        }

        let mut delta = world.write_resource::<DeltaTime>();
        *delta = DeltaTime(dt);
        drop(delta);
    }

    fn handle_input(&mut self, world: &mut World, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput { event: key_event, .. } => {
                let physical_key = key_event.physical_key;
                let state = key_event.state;

                match self.stage {
                    Stage::End => {
                        if state == ElementState::Released {
                            self.init_game(world);
                        }
                        true
                    }
                    Stage::Ready | Stage::Pause => {
                        if state == ElementState::Released {
                            self.stage = Stage::Run;
                        }
                        true
                    }
                    Stage::Run => {
                        match physical_key {
                            PhysicalKey::Code(KeyCode::KeyP) => {
                                if state == ElementState::Released {
                                    self.stage = Stage::Pause;
                                }
                                true
                            }
                            PhysicalKey::Code(KeyCode::KeyR) => {
                                if state == ElementState::Released {
                                    world.delete_all();
                                }
                                true
                            }
                            PhysicalKey::Code(_) => {
                                let mut input_handler = world.write_resource::<pixie::InputHandler>();
                                input_handler.receive_keyboard_input(state, physical_key)
                            }
                            _ => false,
                        }
                    }
                }
            }
            _ => false,
        }
    }

    fn get_camera_uniform(&self, world: &World) -> [[f32; 4]; 4] {
        let camera = world.read_resource::<Camera>();
        camera.get_view_proj()
    }

    fn get_tile_instances(&self, world: &World) -> HashMap<String, Vec<TileRenderData>> {
        let tiles = world.read_storage::<pixie::Tile>();
        let transforms = world.read_storage::<pixie::Transform>();
        let rt_data = (&tiles, &transforms).join().collect::<Vec<_>>();

        let mut tile_instance_data_hashmap = HashMap::new();
        for (tile, transform) in rt_data {
            let atlas = tile.atlas.clone();
            let instance = TileRenderData {
                uv: tile.uv,
                position: transform.position,
                size: transform.size,
            };

            tile_instance_data_hashmap
                .entry(atlas)
                .or_insert_with(Vec::new)
                .push(instance);
        }

        tile_instance_data_hashmap
    }

    fn get_text_instances(&self, world: &World) -> Vec<TextRenderData> {
        let gene_handler = world.read_resource::<GeneHandler>();
        let score = world.read_resource::<Score>();
        let players = world.read_storage::<Player>().join().count();

        let mut text_render_data = vec![
            TextRenderData {
                content: format!("Generation: {}\nScore: {:.3}\nSurvive: {}", gene_handler.generation, score.0, players),
                position: [-4.5, 8.5, 1.],
                size: [0.5, 0.5],
                color: [0.0, 0.0, 0.0],
            }
        ];

        if self.stage == Stage::Ready {
            text_render_data.push(
                TextRenderData {
                    content: "Press any key to start".to_string(),
                    position: [-3., 1., 1.],
                    size: [0.5, 0.5],
                    color: [0.0, 0.0, 0.0],
                }
            );
        }

        text_render_data
    }
    fn should_run_fixed(&self, _world: &World) -> bool { self.stage == Stage::Run }
}

use std::collections::HashMap;
use rand::rngs::ThreadRng;
use hecs::World;
use winit::event::{ElementState, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

use pixie::{Application, ResourceContainer};
use pixie::renderer::{TileRenderData, TextRenderData};

use crate::builder::{background, pipe, ai_player_with_resources};
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
    fn init_game(&mut self, world: &mut World, resources: &mut ResourceContainer) {
        // Clear all entities
        world.clear();

        background(world);
        pipe(world, 16.);
        pipe(world, 8.);

        for _ in 0..100 {
            ai_player_with_resources(world, resources);
        }

        // Reset resources
        resources.insert(GameFinished(false));
        resources.insert(Score::default());

        self.stage = Stage::Ready;
    }

    fn check_game_finished(&mut self, resources: &ResourceContainer) {
        if let Some(finished) = resources.get::<GameFinished>() {
            if finished.0 {
                self.stage = Stage::End;
            }
        }
    }

    pub fn get_gene_data(&self, world: &World, resources: &ResourceContainer) -> ([f32; GENE_SIZE], [f32; 2]) {
        let gene_handler = resources.get::<GeneHandler>()
            .expect("GeneHandler resource not found");

        // Find last player
        let mut last_player: Option<([f32; 3], usize)> = None;
        for (_entity, (transform, dna, _player)) in world.query::<(&pixie::Transform, &DNA, &Player)>().iter() {
            last_player = Some((transform.position, dna.index));
        }

        // Find nearest pipe
        let mut pipe_position = [99.0, 0.0];
        for (_entity, (transform, _pipe_target)) in world.query::<(&pixie::Transform, &PipeTarget)>().iter() {
            if transform.position[0] > -3.0 && pipe_position[0] > transform.position[0] {
                pipe_position = [transform.position[0], transform.position[1]];
            }
        }

        let (position, index) = match last_player {
            None => ([0.0, 0.0], 0),
            Some((pos, idx)) => ([pos[0], pos[1]], idx),
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
    fn init(&mut self, world: &mut World, resources: &mut ResourceContainer) {
        // In hecs, we don't need to register components

        // Set camera zoom for flappy bird
        if let Some(camera) = resources.get_mut::<pixie::Camera>() {
            camera.set_zoom(9.0);
        }

        // Insert resources (Camera and DeltaTime are created automatically by Engine)
        resources.insert(GameFinished(false));
        resources.insert(ThreadRng::default());
        resources.insert(Score::default());
        resources.insert(GeneHandler::default());
        resources.insert(self.stage);

        // Initialize game
        self.init_game(world, resources);
    }

    fn update(&mut self, world: &mut World, resources: &mut ResourceContainer, _dt: f32) {
        self.check_game_finished(resources);

        if self.stage == Stage::End {
            if let Some(gene_handler) = resources.get_mut::<GeneHandler>() {
                gene_handler.process_generation();
            }
            self.init_game(world, resources);
            self.stage = Stage::Run;
        }

        // Always update stage resource for systems to read
        resources.insert(self.stage);
    }

    fn handle_input(&mut self, world: &mut World, resources: &mut ResourceContainer, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput { event: key_event, .. } => {
                let physical_key = key_event.physical_key;
                let state = key_event.state;

                match self.stage {
                    Stage::End => {
                        if state == ElementState::Released {
                            self.init_game(world, resources);
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
                                    world.clear();
                                }
                                true
                            }
                            _ => false,
                        }
                    }
                }
            }
            _ => false,
        }
    }

    fn get_tile_instances(&self, world: &World, _resources: &ResourceContainer) -> HashMap<String, Vec<TileRenderData>> {
        let mut tile_instance_data_hashmap = HashMap::new();

        // Query for entities with Tile and Transform components
        for (_entity, (tile, transform)) in world.query::<(&pixie::Tile, &pixie::Transform)>().iter() {
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

    fn get_text_instances(&self, world: &World, resources: &ResourceContainer) -> Vec<TextRenderData> {
        let gene_handler = resources.get::<GeneHandler>()
            .expect("GeneHandler resource not found");
        let score = resources.get::<Score>()
            .expect("Score resource not found");

        // Count players
        let players = world.query::<&Player>().iter().count();

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

    fn should_run_fixed(&self, _world: &World, _resources: &ResourceContainer) -> bool {
        self.stage == Stage::Run
    }
}

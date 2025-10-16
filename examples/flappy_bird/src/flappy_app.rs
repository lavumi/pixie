use rand::rngs::ThreadRng;
use hecs::World;
use winit::event::{ElementState, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

use pixie::{Application, ResourceContainer, Transform, Text, TextStyle};

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

pub struct FlappyApplication {
    stage: Stage,
    stats_text_entity: Option<hecs::Entity>,
    instruction_text_entity: Option<hecs::Entity>,
}

impl Default for FlappyApplication {
    fn default() -> Self {
        FlappyApplication {
            stage: Stage::Ready,
            stats_text_entity: None,
            instruction_text_entity: None,
        }
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

        // Create text entities
        self.stats_text_entity = Some(world.spawn((
            Transform {
                position: [-4.5, 8.5, 0.0],
                size: [1.0, 1.0],
            },
            Text {
                content: "Generation: 0\nScore: 0.000\nSurvive: 0".to_string(),
                version: 1,
            },
            TextStyle {
                size: [0.5, 0.5],
                color: [0.0, 0.0, 0.0],
                z_index: 1.0,
            },
        )));

        self.instruction_text_entity = Some(world.spawn((
            Transform {
                position: [-3.0, 1.0, 0.0],
                size: [1.0, 1.0],
            },
            Text {
                content: "Press any key to start".to_string(),
                version: 1,
            },
            TextStyle {
                size: [0.5, 0.5],
                color: [0.0, 0.0, 0.0],
                z_index: 1.0,
            },
        )));

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

        // Update text entities
        self.update_texts(world, resources);
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

    fn should_run_fixed(&self, _world: &World, _resources: &ResourceContainer) -> bool {
        self.stage == Stage::Run
    }
}

impl FlappyApplication {
    fn init_game(&mut self, world: &mut World, resources: &mut ResourceContainer) {
        // Clear all entities except text entities
        let text_entities = vec![self.stats_text_entity, self.instruction_text_entity]
            .into_iter()
            .flatten()
            .collect::<std::collections::HashSet<_>>();

        let to_delete: Vec<hecs::Entity> = world.iter()
            .map(|entity_ref| entity_ref.entity())
            .filter(|entity| !text_entities.contains(entity))
            .collect();

        for entity in to_delete {
            let _ = world.despawn(entity);
        }

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

    fn update_texts(&mut self, world: &mut World, resources: &ResourceContainer) {
        // Update stats text
        if let Some(entity) = self.stats_text_entity {
            if let Ok(mut text) = world.get::<&mut Text>(entity) {
                let gene_handler = resources.get::<GeneHandler>()
                    .expect("GeneHandler resource not found");
                let score = resources.get::<Score>()
                    .expect("Score resource not found");

                // Count players
                let players = world.query::<&Player>().iter().count();

                let new_content = format!("Generation: {}\nScore: {:.3}\nSurvive: {}",
                    gene_handler.generation, score.0, players);

                // Use set_content helper which auto-increments version
                text.set_content(new_content);
            }
        }

        // Update instruction text visibility based on stage
        if let Some(entity) = self.instruction_text_entity {
            // Hide/show instruction text based on stage
            if self.stage == Stage::Ready {
                // Make sure it exists
                if world.get::<&Text>(entity).is_err() {
                    // Re-spawn if it was removed
                    self.instruction_text_entity = Some(world.spawn((
                        Transform {
                            position: [-3.0, 1.0, 0.0],
                            size: [1.0, 1.0],
                        },
                        Text {
                            content: "Press any key to start".to_string(),
                            version: 1,
                        },
                        TextStyle {
                            size: [0.5, 0.5],
                            color: [0.0, 0.0, 0.0],
                            z_index: 1.0,
                        },
                    )));
                }
            } else {
                // Remove instruction text when not in Ready stage
                let _ = world.despawn(entity);
            }
        }
    }
}

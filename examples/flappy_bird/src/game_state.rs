use std::collections::HashMap;

use rand::rngs::ThreadRng;
use specs::{Join, World, WorldExt};
use winit::event::ElementState;
use winit::keyboard::{KeyCode, PhysicalKey};
use crate::builder::{background, pipe, ai_player};

use crate::components::*;
use crate::game_configs::GENE_SIZE;
use crate::resources::*;
use crate::system;
use crate::system::UnifiedDispatcher;
use engine::renderer::*;


#[allow(dead_code)]
#[derive(Debug, Clone, Eq, PartialEq, Hash, Copy)]
pub enum Stage { Ready, Run, Pause, End }

impl Default for Stage {
    fn default() -> Self {
        Stage::Ready
    }
}

pub struct GameState {
    pub world: World,
    dispatcher: Box<dyn UnifiedDispatcher + 'static>,
    stage: Stage,
}

impl Default for GameState {
    fn default() -> Self {
        GameState {
            world: World::new(),
            dispatcher: system::build(),
            stage: Stage::Ready,
        }
    }
}

impl GameState {
    pub fn init(&mut self) {
        self.world.register::<Transform>();
        self.world.register::<Collider>();
        self.world.register::<Tile>();
        self.world.register::<Background>();
        self.world.register::<Player>();
        self.world.register::<Pipe>();
        self.world.register::<PipeTarget>();
        self.world.register::<Animation>();
        self.world.register::<Text>();
        self.world.register::<DNA>();

        self.world.insert(Camera::init_orthographic(9));
        self.world.insert(DeltaTime(0.05));
        self.world.insert(GameFinished(false));
        self.world.insert(ThreadRng::default());
        self.world.insert(Score::default());
        self.world.insert(InputHandler::default());
        self.world.insert(GeneHandler::default());


        self.init_game();
    }

    fn init_game(&mut self) {
        self.world.delete_all();
        background(&mut self.world);


        pipe(&mut self.world, 16.);
        pipe(&mut self.world, 8.);

        for _ in 0..100 {
            ai_player(&mut self.world);
        }
        // player(&mut self.world);


        let mut finished = self.world.write_resource::<GameFinished>();
        *finished = GameFinished(false);

        let mut inputs = self.world.write_resource::<InputHandler>();
        *inputs = InputHandler::default();

        let mut score = self.world.write_resource::<Score>();
        *score = Score::default();

        self.stage = Stage::Ready;
    }

    fn check_game_finished(&mut self) {
        let finished = self.world.read_resource::<GameFinished>();
        if finished.0 == true {
            self.stage = Stage::End;
        }
    }

    fn update_delta_time(&mut self, dt: f32) {
        let mut delta = self.world.write_resource::<DeltaTime>();
        *delta = DeltaTime(dt);
    }

    pub fn update(&mut self, dt: f32) {
        self.check_game_finished();

        if self.stage == Stage::End {
            self.world.write_resource::<GeneHandler>().process_generation();
            self.init_game();
            self.stage = Stage::Run;
        }

        if self.stage != Stage::Run {
            return;
        }


        self.update_delta_time(dt);
        self.dispatcher.run_now(&mut self.world);
        self.world.maintain();
    }

    pub fn handle_keyboard_input(&mut self, physical_key: PhysicalKey, state: ElementState) -> bool {
        match self.stage {
            Stage::End => {
                if state == ElementState::Released {
                    self.init_game();
                }
                return true;
            }
            Stage::Ready | Stage::Pause => {
                if state == ElementState::Released {
                    self.stage = Stage::Run;
                }
                return true;
            }
            Stage::Run => {
                match physical_key {
                    PhysicalKey::Code(KeyCode::KeyP) => {
                        if state == ElementState::Released {
                            self.stage = Stage::Pause;
                        }
                        return true;
                    }
                    PhysicalKey::Code(KeyCode::KeyR) => {
                        if state == ElementState::Released {
                            self.force_restart();
                        }
                        return true;
                    }
                    PhysicalKey::Code(_) => {
                        let mut input_handler = self.world.write_resource::<InputHandler>();
                        input_handler.receive_keyboard_input(state, physical_key)
                    }
                    _ => {
                        return false;
                    }
                }
            }
        }
    }

    pub fn get_camera_uniform(&self) -> [[f32; 4]; 4] {
        let camera = self.world.read_resource::<Camera>();
        let camera_uniform = camera.get_view_proj();
        return camera_uniform;
    }

    pub fn get_tile_instance(&self) -> HashMap<String, Vec<TileRenderData>> {
        let tiles = self.world.read_storage::<Tile>();
        let transforms = self.world.read_storage::<Transform>();
        let rt_data = (&tiles, &transforms).join().collect::<Vec<_>>();

        let mut tile_instance_data_hashmap = HashMap::new();
        for (tile, transform) in rt_data {
            let atlas = tile.atlas.clone();
            let instance = TileRenderData {
                uv: tile.uv.clone(),
                position: transform.position.clone(),
                size: transform.size.clone(),
            };


            tile_instance_data_hashmap
                .entry(atlas)
                .or_insert_with(Vec::new)
                .push(instance);
        }

        tile_instance_data_hashmap
    }

    // pub fn get_font_instance(&self) -> Vec<TextRenderData> {
    //     let texts = self.world.read_storage::<Text>();
    //     let transforms = self.world.read_storage::<Transform>();
    //     let rt_data = (&texts, &transforms).join().collect::<Vec<_>>();
    //
    //     let mut text_render_data = Vec::new();
    //     for (text, transform) in rt_data {
    //         let instance = TextRenderData {
    //             content: text.content.clone(),
    //             position : transform.position.clone(),
    //             size : transform.size.clone(),
    //             color : text.color
    //         };
    //
    //         text_render_data.push( instance );
    //     }
    //
    //     text_render_data
    // }
    pub fn set_score_text(&self) -> Vec<TextRenderData> {
        let gene_handler = self.world.read_resource::<GeneHandler>();
        let score = self.world.read_resource::<Score>();
        let players =  self.world.read_storage::<Player>().join().count();
        let mut text_render_data = vec![
            TextRenderData {
                content: format!("Generation:{}\nScore:{:.3}\nSurvive:{}", gene_handler.generation, score.0, players),
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
    pub fn get_gene_data(&self) -> ([f32; GENE_SIZE], [f32; 2]) {
        let gene_handler = self.world.read_resource::<GeneHandler>();


        let player = self.world.read_storage::<Player>();
        let transform = self.world.read_storage::<Transform>();
        let dna = self.world.read_storage::<DNA>();
        let pipe = self.world.read_storage::<PipeTarget>();
        let last_player = (&player, &transform, &dna).join().last();

        let mut pipe_position = [99.0, 0.0];
        for (_, pipe_tr) in (&pipe, &transform).join() {
            if pipe_tr.position[0] > -3.0 &&  pipe_position[0] > pipe_tr.position[0] {
                pipe_position = [pipe_tr.position[0], pipe_tr.position[1]];
            }
        }




        let (position, index) = match last_player {
            None => {
                ([0.0, 0.0],0)
            }
            Some(p) => {
                ([p.1.position[0], p.1.position[1]],p.2.index)
            }
        };

        let input_data = [
            pipe_position[0] - position[0],
            pipe_position[1] - position[1],
        ];

        let gene = gene_handler.get_alive_gene(index);
        return (gene, input_data);
    }

    pub fn force_restart(&mut self) {
        self.world.delete_all();
    }
}
use hecs::{Entity, World};
use rand::rngs::ThreadRng;
use rand::{thread_rng, Rng};
use winit::event::{ElementState, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

use pixie::{Application, Camera, ResourceContainer, Sprite, Text, TextStyle, Transform};

use crate::components::{Agent, Species};
use crate::config;
use crate::resources::{SimulationConfig, SimulationStats};

pub struct PreyPredatorApp {
    paused: bool,
    hud_text_entity: Option<Entity>,
    rng: ThreadRng,
}

impl Default for PreyPredatorApp {
    fn default() -> Self {
        Self {
            paused: false,
            hud_text_entity: None,
            rng: thread_rng(),
        }
    }
}

impl Application for PreyPredatorApp {
    fn init(&mut self, world: &mut World, resources: &mut ResourceContainer) {
        if let Some(camera) = resources.get_mut::<Camera>() {
            camera.set_zoom(24.0);
        }

        resources.insert(SimulationConfig {
            world_width: config::WORLD_WIDTH,
            world_height: config::WORLD_HEIGHT,
            initial_prey: config::INITIAL_PREY,
            initial_predators: config::INITIAL_PREDATORS,
            max_prey: config::MAX_PREY,
            max_predators: config::MAX_PREDATORS,
        });
        resources.insert(SimulationStats::default());

        self.create_hud(world);
        self.spawn_initial_agents(world, resources);
        self.refresh_stats(world, resources, 0.0);
        self.update_hud(world, resources);
    }

    fn update(&mut self, world: &mut World, resources: &mut ResourceContainer, dt: f32) {
        self.refresh_stats(world, resources, dt);
        self.update_hud(world, resources);
    }

    fn handle_input(
        &mut self,
        world: &mut World,
        resources: &mut ResourceContainer,
        event: &WindowEvent,
    ) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                event: key_event, ..
            } if key_event.state == ElementState::Pressed => match key_event.physical_key {
                PhysicalKey::Code(KeyCode::Space) => {
                    self.paused = !self.paused;
                    true
                }
                PhysicalKey::Code(KeyCode::KeyR) => {
                    self.reset(world, resources);
                    true
                }
                _ => false,
            },
            _ => false,
        }
    }

    fn should_run_fixed(&self, _world: &World, _resources: &ResourceContainer) -> bool {
        !self.paused
    }
}

impl PreyPredatorApp {
    fn create_hud(&mut self, world: &mut World) {
        self.hud_text_entity = Some(world.spawn((
            Transform::new([-41.5, 22.5, 0.0], [1.0, 1.0]),
            Text::default(),
            TextStyle {
                size: [0.55, 0.55],
                color: [1.0, 1.0, 1.0],
                z_index: 2.0,
            },
        )));
    }

    fn spawn_initial_agents(&mut self, world: &mut World, resources: &ResourceContainer) {
        let config = resources
            .get::<SimulationConfig>()
            .expect("SimulationConfig resource not found")
            .clone();

        for _ in 0..config.initial_prey {
            self.spawn_agent(world, Species::Prey, &config);
        }

        for _ in 0..config.initial_predators {
            self.spawn_agent(world, Species::Predator, &config);
        }
    }

    fn spawn_agent(&mut self, world: &mut World, species: Species, config: &SimulationConfig) {
        let half_width = config.world_width * 0.5;
        let half_height = config.world_height * 0.5;
        let x = self.rng.gen_range(-half_width..half_width);
        let y = self.rng.gen_range(-half_height..half_height);
        let rotation = self.rng.gen_range(0.0..std::f32::consts::TAU);

        let (atlas, size, z) = match species {
            Species::Prey => ("prey", config::PREY_SIZE, 0.3),
            Species::Predator => ("predator", config::PREDATOR_SIZE, 0.4),
        };

        world.spawn((
            Transform::with_rotation([x, y, z], size, rotation),
            Sprite {
                uv: [0.0, 1.0, 0.0, 1.0],
                atlas: atlas.into(),
            },
            Agent { species },
        ));
    }

    fn reset(&mut self, world: &mut World, resources: &mut ResourceContainer) {
        let hud_entity = self.hud_text_entity;
        let to_delete: Vec<Entity> = world
            .iter()
            .map(|entity_ref| entity_ref.entity())
            .filter(|entity| Some(*entity) != hud_entity)
            .collect();

        for entity in to_delete {
            let _ = world.despawn(entity);
        }

        resources.insert(SimulationStats::default());
        self.spawn_initial_agents(world, resources);
        self.refresh_stats(world, resources, 0.0);
        self.update_hud(world, resources);
    }

    fn refresh_stats(&self, world: &World, resources: &mut ResourceContainer, dt: f32) {
        let mut prey_alive = 0;
        let mut predators_alive = 0;

        for (_entity, agent) in world.query::<&Agent>().iter() {
            match agent.species {
                Species::Prey => prey_alive += 1,
                Species::Predator => predators_alive += 1,
            }
        }

        if let Some(stats) = resources.get_mut::<SimulationStats>() {
            if !self.paused {
                stats.elapsed_time += dt;
            }
            stats.prey_alive = prey_alive;
            stats.predators_alive = predators_alive;
        }
    }

    fn update_hud(&self, world: &mut World, resources: &ResourceContainer) {
        let Some(entity) = self.hud_text_entity else {
            return;
        };

        let Some(config) = resources.get::<SimulationConfig>() else {
            return;
        };

        let Some(stats) = resources.get::<SimulationStats>() else {
            return;
        };

        if let Ok(mut text) = world.get::<&mut Text>(entity) {
            let status = if self.paused { "Paused" } else { "Running" };
            text.content = format!(
                "Prey Predator Simulation\n\nStatus: {status}\nTime: {:.1}\nPrey: {} of {}\nPredators: {} of {}\n\nSpace Pause R Reset",
                stats.elapsed_time,
                stats.prey_alive,
                config.max_prey,
                stats.predators_alive,
                config.max_predators,
            );
        }
    }
}

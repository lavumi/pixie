use hecs::{Entity, World};
use rand::rngs::ThreadRng;
use rand::{thread_rng, Rng};
use winit::event::{ElementState, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

use pixie::{
    Application, Camera, ResourceContainer, Sprite, Text, TextCoordinateSpace, TextStyle, Transform,
};

use crate::components::{Agent, AgentMotion, Species};
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

        self.create_world_area(world);
        self.create_hud(world);
        self.spawn_initial_agents(world, resources);
        self.refresh_stats(world, resources, 0.0);
        self.update_hud(world, resources);
    }

    fn update(&mut self, world: &mut World, resources: &mut ResourceContainer, dt: f32) {
        self.refresh_stats(world, resources, dt);
        self.update_hud(world, resources);
    }

    fn fixed_update(
        &mut self,
        world: &mut World,
        resources: &mut ResourceContainer,
        fixed_dt: f32,
    ) {
        self.update_agent_motion(world, resources, fixed_dt);
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
    fn create_world_area(&self, world: &mut World) {
        let width = config::WORLD_WIDTH;
        let height = config::WORLD_HEIGHT;
        let border = config::WORLD_BORDER_THICKNESS;
        let half_width = width * 0.5;
        let half_height = height * 0.5;

        world.spawn((
            Transform::new([0.0, 0.0, 0.0], [width, height]),
            Sprite {
                uv: [0.0, 1.0, 0.0, 1.0],
                atlas: "area_fill".into(),
            },
        ));

        for (position, size) in [
            (
                [0.0, half_height + border * 0.5, 0.1],
                [width + border, border],
            ),
            (
                [0.0, -half_height - border * 0.5, 0.1],
                [width + border, border],
            ),
            ([-half_width - border * 0.5, 0.0, 0.1], [border, height]),
            ([half_width + border * 0.5, 0.0, 0.1], [border, height]),
        ] {
            world.spawn((
                Transform::new(position, size),
                Sprite {
                    uv: [0.0, 1.0, 0.0, 1.0],
                    atlas: "area_border".into(),
                },
            ));
        }
    }

    fn create_hud(&mut self, world: &mut World) {
        self.hud_text_entity = Some(world.spawn((
            Transform::new([20.0, 32.0, 0.0], [1.0, 1.0]),
            Text::default(),
            TextStyle {
                size: [18.0, 18.0],
                color: [1.0, 1.0, 1.0],
                z_index: -10.0,
                coordinate_space: TextCoordinateSpace::Screen,
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

        let (atlas, size, z, max_abs_speed, max_abs_angular_velocity) = match species {
            Species::Prey => (
                "prey",
                config::PREY_SIZE,
                0.3,
                config::PREY_MAX_ABS_SPEED,
                config::PREY_MAX_ABS_ANGULAR_VELOCITY,
            ),
            Species::Predator => (
                "predator",
                config::PREDATOR_SIZE,
                0.4,
                config::PREDATOR_MAX_ABS_SPEED,
                config::PREDATOR_MAX_ABS_ANGULAR_VELOCITY,
            ),
        };
        let speed = self.random_signed_min_magnitude(max_abs_speed, 0.35);
        let angular_velocity = self
            .rng
            .gen_range(-max_abs_angular_velocity..max_abs_angular_velocity);

        world.spawn((
            Transform::with_rotation([x, y, z], size, rotation),
            Sprite {
                uv: [0.0, 1.0, 0.0, 1.0],
                atlas: atlas.into(),
            },
            Agent { species },
            AgentMotion::new(
                speed,
                max_abs_speed,
                angular_velocity,
                max_abs_angular_velocity,
            ),
        ));
    }

    fn random_signed_min_magnitude(&mut self, max_abs: f32, min_ratio: f32) -> f32 {
        let magnitude = self.rng.gen_range(max_abs * min_ratio..max_abs);
        if self.rng.gen_bool(0.5) {
            magnitude
        } else {
            -magnitude
        }
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
        self.create_world_area(world);
        self.spawn_initial_agents(world, resources);
        self.refresh_stats(world, resources, 0.0);
        self.update_hud(world, resources);
    }

    fn update_agent_motion(&self, world: &mut World, resources: &ResourceContainer, fixed_dt: f32) {
        let Some(config) = resources.get::<SimulationConfig>() else {
            return;
        };

        for (_entity, (transform, motion)) in
            world.query::<(&mut Transform, &mut AgentMotion)>().iter()
        {
            motion.speed = motion
                .speed
                .clamp(-motion.max_abs_speed, motion.max_abs_speed);
            motion.angular_velocity = motion.angular_velocity.clamp(
                -motion.max_abs_angular_velocity,
                motion.max_abs_angular_velocity,
            );

            transform.rotation += motion.angular_velocity * fixed_dt;
            let direction = [transform.rotation.cos(), transform.rotation.sin()];
            transform.position[0] += direction[0] * motion.speed * fixed_dt;
            transform.position[1] += direction[1] * motion.speed * fixed_dt;

            Self::wrap_position(
                &mut transform.position,
                config.world_width,
                config.world_height,
            );
        }
    }

    fn wrap_position(position: &mut [f32; 3], world_width: f32, world_height: f32) {
        let half_width = world_width * 0.5;
        let half_height = world_height * 0.5;

        if position[0] > half_width {
            position[0] -= world_width;
        } else if position[0] < -half_width {
            position[0] += world_width;
        }

        if position[1] > half_height {
            position[1] -= world_height;
        } else if position[1] < -half_height {
            position[1] += world_height;
        }
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

        let zoom = resources
            .get::<Camera>()
            .map(|camera| camera.zoom())
            .unwrap_or_default();

        if let Ok(mut text) = world.get::<&mut Text>(entity) {
            let status = if self.paused { "Paused" } else { "Running" };
            text.content = format!(
                "Prey Predator Simulation\n\nStatus: {status}\nTime: {:.1}\nZoom: {:.1}\nPrey: {} of {}\nPredators: {} of {}\n\nSpace Pause R Reset\nMouse Wheel Zoom\nLeft Drag Pan",
                stats.elapsed_time,
                zoom,
                stats.prey_alive,
                config.max_prey,
                stats.predators_alive,
                config.max_predators,
            );
        }
    }
}

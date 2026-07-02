use hecs::{Entity, World};
use rand::rngs::ThreadRng;
use rand::{thread_rng, Rng};
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

use pixie::{
    Application, Camera, DebugDraw, DebugLine, RenderViewport, ResourceContainer, Sprite, Text,
    TextCoordinateSpace, TextStyle, Transform, UiAnchor, UiTransform, ViewportMode,
};

use crate::components::{
    Agent, AgentMotion, Brain, BrainOutput, LifeCycle, Reproduction, Species, Vision, VisionOutput,
};
use crate::config;
use crate::neural_network::{Genome, MutationConfig, NetworkShape};
use crate::resources::{
    DeathQueue, DeathReason, SelectionState, SimulationConfig, SimulationStats, SpawnQueue,
};
use crate::system::brain::process_brains;
use crate::system::lifecycle::{process_predation, process_reproduction, update_lifecycles};
use crate::system::vision::collect_vision;

const DEBUG_RAY_THICKNESS: f32 = 0.04;
const SELECTION_OUTLINE_THICKNESS: f32 = 0.07;
const SELECTION_OUTLINE_PADDING: f32 = 0.15;
const SELECTION_OUTLINE_SEGMENTS: usize = 24;
const DEBUG_RAY_MISS_COLOR: [f32; 4] = [0.55, 0.55, 0.55, 0.7];
const DEBUG_RAY_PREY_COLOR: [f32; 4] = [0.2, 1.0, 0.25, 0.85];
const DEBUG_RAY_PREDATOR_COLOR: [f32; 4] = [1.0, 0.2, 0.2, 0.85];
const SELECTION_OUTLINE_COLOR: [f32; 4] = [1.0, 0.85, 0.1, 1.0];

pub struct PreyPredatorApp {
    paused: bool,
    hud_text_entity: Option<Entity>,
    vision_elapsed: f32,
    rng: ThreadRng,
}

impl Default for PreyPredatorApp {
    fn default() -> Self {
        Self {
            paused: false,
            hud_text_entity: None,
            vision_elapsed: 0.0,
            rng: thread_rng(),
        }
    }
}

impl Application for PreyPredatorApp {
    fn init(&mut self, world: &mut World, resources: &mut ResourceContainer) {
        resources.insert(ViewportMode::Expand);
        if let Some(camera) = resources.get_mut::<Camera>() {
            camera.set_zoom(24.0);
        }

        let simulation_config = SimulationConfig::default();
        self.vision_elapsed = simulation_config.vision_update_interval;
        resources.insert(simulation_config);
        resources.insert(SimulationStats::default());
        resources.insert(SelectionState::default());
        resources.insert(DeathQueue::default());
        resources.insert(SpawnQueue::default());

        self.create_world_area(world);
        self.create_hud(world);
        self.spawn_initial_agents(world, resources);
        self.refresh_stats(world, resources, 0.0);
        self.update_hud(world, resources);
    }

    fn update(&mut self, world: &mut World, resources: &mut ResourceContainer, dt: f32) {
        self.follow_selected_agent(world, resources);
        self.draw_selected_agent_debug(world, resources);
        self.refresh_stats(world, resources, dt);
        self.update_hud(world, resources);
    }

    fn fixed_update(
        &mut self,
        world: &mut World,
        resources: &mut ResourceContainer,
        fixed_dt: f32,
    ) {
        let Some(simulation_config) = resources.get::<SimulationConfig>().cloned() else {
            return;
        };
        self.vision_elapsed += fixed_dt;
        if self.vision_elapsed >= simulation_config.vision_update_interval {
            self.vision_elapsed %= simulation_config.vision_update_interval;
            collect_vision(world);
            process_brains(world);
        }
        self.update_agent_motion(world, resources, fixed_dt);

        let mut deaths = resources.remove::<DeathQueue>().unwrap_or_default();
        let mut spawns = resources.remove::<SpawnQueue>().unwrap_or_default();
        deaths.requests.clear();
        spawns.requests.clear();

        process_predation(world, &simulation_config, &mut deaths);
        update_lifecycles(world, &simulation_config, fixed_dt, &mut deaths);
        process_reproduction(world, &simulation_config, &deaths, &mut spawns);
        self.cleanup_dead_agents(world, resources, &mut deaths);
        self.spawn_queued_agents(world, resources, &simulation_config, &mut spawns);

        resources.insert(deaths);
        resources.insert(spawns);
    }

    fn handle_input(
        &mut self,
        world: &mut World,
        resources: &mut ResourceContainer,
        event: &WindowEvent,
    ) -> bool {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                if let Some(selection) = resources.get_mut::<SelectionState>() {
                    selection.update_cursor([position.x as f32, position.y as f32]);
                }
                false
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => {
                let click_position = if let Some(selection) = resources.get_mut::<SelectionState>()
                {
                    match state {
                        ElementState::Pressed => {
                            selection.begin_primary_press();
                            None
                        }
                        ElementState::Released => selection.finish_primary_press(),
                    }
                } else {
                    None
                };

                if let Some(click_position) = click_position {
                    self.select_agent_at_screen(world, resources, click_position);
                }
                false
            }
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
                PhysicalKey::Code(KeyCode::Escape) => {
                    if let Some(selection) = resources.get_mut::<SelectionState>() {
                        if selection.selected.is_some() {
                            selection.clear();
                            return true;
                        }
                    }
                    false
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
            Text::default(),
            TextStyle {
                size: [18.0, 18.0],
                color: [1.0, 1.0, 1.0],
                z_index: -10.0,
                coordinate_space: TextCoordinateSpace::Screen,
            },
            UiTransform::new(
                UiAnchor::TopLeft,
                UiAnchor::TopLeft,
                config::HUD_OFFSET_PIXELS,
            ),
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
        self.spawn_agent_at(world, species, [x, y], 0.0, config, None);
    }

    fn spawn_agent_at(
        &mut self,
        world: &mut World,
        species: Species,
        position: [f32; 2],
        reproduction_cooldown: f32,
        simulation_config: &SimulationConfig,
        inherited_brain: Option<Brain>,
    ) {
        let rotation = self.rng.gen_range(0.0..std::f32::consts::TAU);

        let (atlas, size, z, max_abs_speed, max_abs_angular_velocity, vision) = match species {
            Species::Prey => (
                "prey",
                config::PREY_SIZE,
                0.3,
                config::PREY_MAX_ABS_SPEED,
                config::PREY_MAX_ABS_ANGULAR_VELOCITY,
                Vision::new(
                    config::PREY_VISION_MAX_DISTANCE,
                    config::PREY_VISION_TOTAL_ANGLE,
                    config::PREY_VISION_RAY_INTERVAL,
                ),
            ),
            Species::Predator => (
                "predator",
                config::PREDATOR_SIZE,
                0.4,
                config::PREDATOR_MAX_ABS_SPEED,
                config::PREDATOR_MAX_ABS_ANGULAR_VELOCITY,
                Vision::new(
                    config::PREDATOR_VISION_MAX_DISTANCE,
                    config::PREDATOR_VISION_TOTAL_ANGLE,
                    config::PREDATOR_VISION_RAY_INTERVAL,
                ),
            ),
        };
        let speed = self.random_signed_min_magnitude(max_abs_speed, 0.35);
        let angular_velocity = self
            .rng
            .gen_range(-max_abs_angular_velocity..max_abs_angular_velocity);
        let brain = inherited_brain.unwrap_or_else(|| {
            let shape = NetworkShape::new(
                vision.input_len(),
                &simulation_config.brain_hidden_layers,
                simulation_config.brain_output_size,
            );
            let genome = Genome::random_with_output_biases(
                &shape,
                &[0.0, simulation_config.brain_initial_speed_bias],
                &mut self.rng,
            );
            Brain::new(shape, genome)
        });
        assert_eq!(
            brain.shape.input_size(),
            vision.input_len(),
            "inherited brain input size does not match offspring vision"
        );
        assert_eq!(
            brain.shape.output_size(),
            2,
            "prey predator brains must produce rotation and speed outputs"
        );

        world.spawn((
            Transform::with_rotation([position[0], position[1], z], size, rotation),
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
            brain,
            BrainOutput::default(),
            vision,
            VisionOutput::empty(&vision),
            LifeCycle::default(),
            Reproduction::new(reproduction_cooldown),
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
        resources.insert(DeathQueue::default());
        resources.insert(SpawnQueue::default());
        self.vision_elapsed = resources
            .get::<SimulationConfig>()
            .map(|config| config.vision_update_interval)
            .unwrap_or_default();
        if let Some(selection) = resources.get_mut::<SelectionState>() {
            selection.clear();
        }
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

    fn cleanup_dead_agents(
        &self,
        world: &mut World,
        resources: &mut ResourceContainer,
        deaths: &mut DeathQueue,
    ) {
        for request in deaths.requests.drain(..) {
            if world.despawn(request.entity).is_err() {
                continue;
            }

            if let Some(selection) = resources.get_mut::<SelectionState>() {
                if selection.selected == Some(request.entity) {
                    selection.clear();
                }
            }
            if let Some(stats) = resources.get_mut::<SimulationStats>() {
                match (request.species, request.reason) {
                    (Species::Prey, DeathReason::Eaten) => stats.prey_eaten += 1,
                    (Species::Prey, DeathReason::Age) => stats.prey_died_of_age += 1,
                    (Species::Predator, DeathReason::Age | DeathReason::Starvation) => {
                        stats.predators_died += 1;
                    }
                    (Species::Prey, DeathReason::Starvation)
                    | (Species::Predator, DeathReason::Eaten) => {}
                }
            }
        }
    }

    fn spawn_queued_agents(
        &mut self,
        world: &mut World,
        resources: &mut ResourceContainer,
        config: &SimulationConfig,
        spawns: &mut SpawnQueue,
    ) {
        for request in spawns.requests.drain(..) {
            let Ok(parent_brain) = world.get::<&Brain>(request.parent) else {
                continue;
            };
            let mutation_config = MutationConfig {
                mutation_rate: config.brain_mutation_rate,
                mutation_sigma: config.brain_mutation_sigma,
                reset_rate: config.brain_reset_rate,
                max_abs_gene: config.brain_max_abs_gene,
            };
            let child_brain = Brain::new(
                parent_brain.shape.clone(),
                parent_brain.genome.mutated(&mutation_config, &mut self.rng),
            );
            drop(parent_brain);

            let angle = self.rng.gen_range(0.0..std::f32::consts::TAU);
            let distance = self.rng.gen_range(0.0..config.offspring_spawn_offset);
            let mut position = [
                request.parent_position[0] + angle.cos() * distance,
                request.parent_position[1] + angle.sin() * distance,
                0.0,
            ];
            Self::wrap_position(&mut position, config.world_width, config.world_height);
            let cooldown = match request.species {
                Species::Prey => config.prey_reproduction_cooldown,
                Species::Predator => config.predator_reproduction_cooldown,
            };
            self.spawn_agent_at(
                world,
                request.species,
                [position[0], position[1]],
                cooldown,
                config,
                Some(child_brain),
            );

            if let Some(stats) = resources.get_mut::<SimulationStats>() {
                match request.species {
                    Species::Prey => stats.prey_born += 1,
                    Species::Predator => stats.predators_born += 1,
                }
            }
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
        let selected = resources
            .get::<SelectionState>()
            .and_then(|selection| selection.selected)
            .and_then(|entity| world.get::<&Agent>(entity).ok())
            .map(|agent| match agent.species {
                Species::Prey => "Prey",
                Species::Predator => "Predator",
            })
            .unwrap_or("None");

        if let Ok(mut text) = world.get::<&mut Text>(entity) {
            let status = if self.paused { "Paused" } else { "Running" };
            text.content = format!(
                "Prey Predator Simulation\n\nStatus: {status}\nTime: {:.1}\nZoom: {:.1}\nSelected: {selected}\nPrey: {} of {}\nPredators: {} of {}\nBorn: P {} Pred {}\nEaten: {}\nDeaths: P {} Pred {}\n\nSpace Pause R Reset\nMouse Wheel Zoom\nLeft Click Select\nLeft Drag Pan",
                stats.elapsed_time,
                zoom,
                stats.prey_alive,
                config.max_prey,
                stats.predators_alive,
                config.max_predators,
                stats.prey_born,
                stats.predators_born,
                stats.prey_eaten,
                stats.prey_died_of_age,
                stats.predators_died,
            );
        }
    }

    fn select_agent_at_screen(
        &self,
        world: &World,
        resources: &mut ResourceContainer,
        screen_position: [f32; 2],
    ) {
        let world_position = resources
            .get::<RenderViewport>()
            .copied()
            .and_then(|viewport| {
                resources
                    .get::<Camera>()
                    .and_then(|camera| camera.screen_to_world(screen_position, viewport))
            });
        let selected = world_position.and_then(|position| Self::nearest_agent_at(world, position));

        if let Some(selection) = resources.get_mut::<SelectionState>() {
            selection.selected = selected;
        }
    }

    fn nearest_agent_at(world: &World, position: [f32; 2]) -> Option<Entity> {
        world
            .query::<(&Transform, &Agent)>()
            .iter()
            .filter_map(|(entity, (transform, _agent))| {
                let delta = [
                    transform.position[0] - position[0],
                    transform.position[1] - position[1],
                ];
                let distance_squared = delta[0] * delta[0] + delta[1] * delta[1];
                let radius = transform.size[0].abs().max(transform.size[1].abs()) * 0.5;

                (distance_squared <= radius * radius).then_some((entity, distance_squared))
            })
            .min_by(|left, right| left.1.total_cmp(&right.1))
            .map(|(entity, _distance_squared)| entity)
    }

    fn follow_selected_agent(&self, world: &World, resources: &mut ResourceContainer) {
        let Some(selected) = resources
            .get::<SelectionState>()
            .and_then(|selection| selection.selected)
        else {
            return;
        };
        let position = world
            .get::<&Transform>(selected)
            .ok()
            .map(|transform| [transform.position[0], transform.position[1]]);

        match position {
            Some(position) => {
                if let Some(camera) = resources.get_mut::<Camera>() {
                    camera.move_camera(position);
                }
            }
            None => {
                if let Some(selection) = resources.get_mut::<SelectionState>() {
                    selection.clear();
                }
            }
        }
    }

    fn draw_selected_agent_debug(&self, world: &World, resources: &mut ResourceContainer) {
        let Some(selected) = resources
            .get::<SelectionState>()
            .and_then(|selection| selection.selected)
        else {
            return;
        };
        let debug_lines = {
            let Ok(transform) = world.get::<&Transform>(selected) else {
                return;
            };
            let Ok(vision) = world.get::<&Vision>(selected) else {
                return;
            };
            let Ok(output) = world.get::<&VisionOutput>(selected) else {
                return;
            };

            Self::build_selected_agent_debug_lines(&transform, &vision, &output)
        };

        if let Some(debug_draw) = resources.get_mut::<DebugDraw>() {
            for line in debug_lines {
                debug_draw.submit(line);
            }
        }
    }

    fn build_selected_agent_debug_lines(
        transform: &Transform,
        vision: &Vision,
        output: &VisionOutput,
    ) -> Vec<DebugLine> {
        let mut lines = Vec::with_capacity(vision.ray_count() + SELECTION_OUTLINE_SEGMENTS);
        let z = transform.position[2] + 0.2;
        let origin = [transform.position[0], transform.position[1], z];

        for ray_index in 0..vision.ray_count() {
            let hit = output.hits.get(ray_index).copied().flatten();
            let (distance, color) = match hit {
                Some(hit) => (
                    hit.distance.min(vision.max_distance),
                    match hit.species {
                        Species::Prey => DEBUG_RAY_PREY_COLOR,
                        Species::Predator => DEBUG_RAY_PREDATOR_COLOR,
                    },
                ),
                None => (vision.max_distance, DEBUG_RAY_MISS_COLOR),
            };
            let angle = transform.rotation + vision.ray_angle(ray_index);
            let end = [
                origin[0] + angle.cos() * distance,
                origin[1] + angle.sin() * distance,
                z,
            ];
            lines.push(DebugLine::new(origin, end, color, DEBUG_RAY_THICKNESS));
        }

        let radius =
            transform.size[0].abs().max(transform.size[1].abs()) * 0.5 + SELECTION_OUTLINE_PADDING;
        for segment in 0..SELECTION_OUTLINE_SEGMENTS {
            let start_angle =
                std::f32::consts::TAU * segment as f32 / SELECTION_OUTLINE_SEGMENTS as f32;
            let end_angle =
                std::f32::consts::TAU * (segment + 1) as f32 / SELECTION_OUTLINE_SEGMENTS as f32;
            let start = [
                origin[0] + start_angle.cos() * radius,
                origin[1] + start_angle.sin() * radius,
                z + 0.01,
            ];
            let end = [
                origin[0] + end_angle.cos() * radius,
                origin[1] + end_angle.sin() * radius,
                z + 0.01,
            ];
            lines.push(DebugLine::new(
                start,
                end,
                SELECTION_OUTLINE_COLOR,
                SELECTION_OUTLINE_THICKNESS,
            ));
        }

        lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::VisionHit;
    use crate::resources::{DeathRequest, SpawnRequest};

    fn spawn_agent(world: &mut World, position: [f32; 2], size: f32) -> Entity {
        world.spawn((
            Transform::new([position[0], position[1], 0.0], [size, size]),
            Agent {
                species: Species::Prey,
            },
        ))
    }

    #[test]
    fn nearest_overlapping_agent_is_selected() {
        let mut world = World::new();
        let farther = spawn_agent(&mut world, [0.3, 0.0], 1.0);
        let nearer = spawn_agent(&mut world, [0.1, 0.0], 1.0);

        assert_eq!(
            PreyPredatorApp::nearest_agent_at(&world, [0.0, 0.0]),
            Some(nearer)
        );
        assert_ne!(nearer, farther);
    }

    #[test]
    fn hud_uses_top_left_anchor_and_pivot() {
        let mut app = PreyPredatorApp::default();
        let mut world = World::new();

        app.create_hud(&mut world);

        let entity = app.hud_text_entity.unwrap();
        let ui_transform = world.get::<&UiTransform>(entity).unwrap();
        assert_eq!(ui_transform.anchor, UiAnchor::TopLeft);
        assert_eq!(ui_transform.pivot, UiAnchor::TopLeft);
        assert_eq!(ui_transform.offset, config::HUD_OFFSET_PIXELS);
        assert!(world.get::<&Transform>(entity).is_err());
    }

    #[test]
    fn empty_space_clears_selection_candidate() {
        let mut world = World::new();
        spawn_agent(&mut world, [0.0, 0.0], 1.0);

        assert_eq!(PreyPredatorApp::nearest_agent_at(&world, [2.0, 2.0]), None);
    }

    #[test]
    fn camera_follows_selected_agent_without_changing_zoom() {
        let app = PreyPredatorApp::default();
        let mut world = World::new();
        let selected = spawn_agent(&mut world, [4.0, -3.0], 1.0);
        let mut resources = ResourceContainer::new();
        let mut camera = Camera::init_orthographic(10.0, 1.0);
        camera.set_zoom(7.0);
        resources.insert(camera);
        resources.insert(SelectionState::with_selected(selected));

        app.follow_selected_agent(&world, &mut resources);

        let camera = resources.get::<Camera>().unwrap();
        assert_eq!(camera.zoom(), 7.0);
        let center = camera
            .screen_to_world([100.0, 100.0], RenderViewport::new(0.0, 0.0, 200.0, 200.0))
            .unwrap();
        assert!((center[0] - 4.0).abs() < 0.0001);
        assert!((center[1] + 3.0).abs() < 0.0001);
    }

    #[test]
    fn camera_follow_clears_despawned_selection() {
        let app = PreyPredatorApp::default();
        let mut world = World::new();
        let selected = spawn_agent(&mut world, [0.0, 0.0], 1.0);
        world.despawn(selected).unwrap();
        let mut resources = ResourceContainer::new();
        resources.insert(Camera::init_orthographic(10.0, 1.0));
        resources.insert(SelectionState::with_selected(selected));

        app.follow_selected_agent(&world, &mut resources);

        assert_eq!(resources.get::<SelectionState>().unwrap().selected, None);
    }

    #[test]
    fn selected_debug_lines_use_hit_distance_and_species_color() {
        let mut world = World::new();
        let target = spawn_agent(&mut world, [5.0, 2.0], 1.0);
        let vision = Vision::new(10.0, 0.0, 1.0);
        let output = VisionOutput {
            hits: vec![Some(VisionHit {
                target,
                species: Species::Predator,
                distance: 4.0,
            })],
        };
        let transform = Transform::with_rotation([1.0, 2.0, 0.3], [1.0, 1.0], 0.0);

        let lines = PreyPredatorApp::build_selected_agent_debug_lines(&transform, &vision, &output);

        assert_eq!(lines.len(), 1 + SELECTION_OUTLINE_SEGMENTS);
        assert_eq!(lines[0].start, [1.0, 2.0, 0.5]);
        assert_eq!(lines[0].end, [5.0, 2.0, 0.5]);
        assert_eq!(lines[0].color, DEBUG_RAY_PREDATOR_COLOR);
        assert!(lines[1..]
            .iter()
            .all(|line| line.color == SELECTION_OUTLINE_COLOR));
    }

    #[test]
    fn missed_debug_ray_uses_max_distance_and_miss_color() {
        let vision = Vision::new(6.0, 0.0, 1.0);
        let output = VisionOutput::empty(&vision);
        let transform =
            Transform::with_rotation([0.0, 0.0, 0.3], [1.0, 1.0], std::f32::consts::FRAC_PI_2);

        let lines = PreyPredatorApp::build_selected_agent_debug_lines(&transform, &vision, &output);

        assert!(lines[0].end[0].abs() < 0.0001);
        assert!((lines[0].end[1] - 6.0).abs() < 0.0001);
        assert_eq!(lines[0].color, DEBUG_RAY_MISS_COLOR);
    }

    #[test]
    fn cleanup_applies_death_stats_and_clears_selection() {
        let app = PreyPredatorApp::default();
        let mut world = World::new();
        let prey = spawn_agent(&mut world, [0.0, 0.0], 1.0);
        let predator = world.spawn((
            Transform::default(),
            Agent {
                species: Species::Predator,
            },
        ));
        let mut resources = ResourceContainer::new();
        resources.insert(SimulationStats::default());
        resources.insert(SelectionState::with_selected(prey));
        let mut deaths = DeathQueue {
            requests: vec![
                DeathRequest {
                    entity: prey,
                    species: Species::Prey,
                    reason: DeathReason::Eaten,
                },
                DeathRequest {
                    entity: predator,
                    species: Species::Predator,
                    reason: DeathReason::Starvation,
                },
            ],
        };

        app.cleanup_dead_agents(&mut world, &mut resources, &mut deaths);

        assert!(!world.contains(prey));
        assert!(!world.contains(predator));
        assert_eq!(resources.get::<SelectionState>().unwrap().selected, None);
        let stats = resources.get::<SimulationStats>().unwrap();
        assert_eq!(stats.prey_eaten, 1);
        assert_eq!(stats.predators_died, 1);
    }

    #[test]
    fn queued_child_starts_with_species_cooldown_and_updates_stats() {
        let mut app = PreyPredatorApp::default();
        let mut world = World::new();
        let mut resources = ResourceContainer::new();
        resources.insert(SimulationStats::default());
        let config = SimulationConfig {
            offspring_spawn_offset: 0.1,
            brain_mutation_rate: 0.0,
            brain_reset_rate: 0.0,
            ..SimulationConfig::default()
        };
        let vision = Vision::new(
            config::PREY_VISION_MAX_DISTANCE,
            config::PREY_VISION_TOTAL_ANGLE,
            config::PREY_VISION_RAY_INTERVAL,
        );
        let shape = NetworkShape::new(
            vision.input_len(),
            &config.brain_hidden_layers,
            config.brain_output_size,
        );
        let parent_brain = Brain::new(
            shape.clone(),
            Genome::from_genes(&shape, vec![0.25; shape.parameter_count()]),
        );
        let parent = world.spawn((parent_brain.clone(),));
        let mut spawns = SpawnQueue {
            requests: vec![SpawnRequest {
                parent,
                species: Species::Prey,
                parent_position: [2.0, 3.0],
            }],
        };

        app.spawn_queued_agents(&mut world, &mut resources, &config, &mut spawns);

        let mut query = world.query::<(&LifeCycle, &Reproduction, &Brain)>();
        let (_, (lifecycle, reproduction, child_brain)) = query.iter().next().unwrap();
        assert_eq!(*lifecycle, LifeCycle::default());
        assert_eq!(
            reproduction.cooldown_remaining,
            config.prey_reproduction_cooldown
        );
        assert_eq!(*child_brain, parent_brain);
        assert_eq!(resources.get::<SimulationStats>().unwrap().prey_born, 1);
        assert!(spawns.requests.is_empty());
    }
}

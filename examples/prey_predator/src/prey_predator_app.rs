use hecs::{Entity, World};
use rand::rngs::ThreadRng;
use rand::{thread_rng, Rng};
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

use pixie::{
    Application, Camera, DebugDraw, DebugLine, RenderViewport, ResourceContainer, Sprite, Text,
    TextCoordinateSpace, TextStyle, Transform,
};

use crate::components::{Agent, AgentMotion, Species, Vision, VisionOutput};
use crate::config;
use crate::resources::{SelectionState, SimulationConfig, SimulationStats};
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
        resources.insert(SelectionState::default());

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
        collect_vision(world);
        self.update_agent_motion(world, resources, fixed_dt);
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
            vision,
            VisionOutput::empty(&vision),
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
                "Prey Predator Simulation\n\nStatus: {status}\nTime: {:.1}\nZoom: {:.1}\nSelected: {selected}\nPrey: {} of {}\nPredators: {} of {}\n\nSpace Pause R Reset\nMouse Wheel Zoom\nLeft Click Select\nLeft Drag Pan",
                stats.elapsed_time,
                zoom,
                stats.prey_alive,
                config.max_prey,
                stats.predators_alive,
                config.max_predators,
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
}

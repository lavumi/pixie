use std::collections::HashMap;
use specs::{Join, World, WorldExt, Builder};
use winit::event::{WindowEvent, ElementState, MouseButton};
use winit::keyboard::{KeyCode, PhysicalKey};

use pixie::{Application, Camera, DeltaTime, Transform, Tile, Gravity};
use pixie::{RigidBody, Velocity, Force, CircleCollider, BodyType, BoxCollider};
use pixie::renderer::{TileRenderData, TextRenderData};

// systems are now built and owned by the engine; keep module private here
use crate::config;

pub struct PhysicsApp {
    gravity_enabled: bool,
    ball_state: BallState,
    balls_to_shoot: Vec<[f32; 2]>, // Store positions for balls to be shot
    shot_index: usize,
    shoot_timer: f32,
    shoot_interval: f32,

    // Ball shooting configuration
    ball_count: usize,
    shoot_speed: f32,
    shoot_angle: f32,
    start_x: f32,
    start_y: f32,
    ball_sizes: [f32; 3], // Three different ball sizes
    ball_mass: f32,
    ball_restitution: f32,
}

#[derive(Debug, Clone, Copy)]
enum BallState {
    Ready,
    Shooting,
    Complete,
}

impl Default for PhysicsApp {
    fn default() -> Self {
        PhysicsApp {
            gravity_enabled: true,
            ball_state: BallState::Ready,
            balls_to_shoot: Vec::new(),
            shot_index: 0,
            shoot_timer: 0.0,
            shoot_interval: 0.05, // Shoot one ball every 0.05 seconds

            // Ball shooting configuration
            ball_count: 100,
            shoot_speed: 25.0,
            shoot_angle: -45.0, // degrees
            start_x: -15.0,
            start_y: 15.0,
            ball_sizes: [0.3, 0.5, 0.7], // Three different ball sizes
            ball_mass: 1.0,  // Increase mass to resist gravity better
            ball_restitution: 0.6,  // Increase bounce
        }
    }
}

impl Application for PhysicsApp {
    fn init(&mut self, world: &mut World) {
        // Register components
        world.register::<Transform>();
        world.register::<Tile>();
        world.register::<RigidBody>();
        world.register::<Velocity>();
        world.register::<Force>();
        world.register::<CircleCollider>();
        world.register::<BoxCollider>();

        // Insert resources
        let aspect_ratio = config::SCREEN_SIZE[0] as f32 / config::SCREEN_SIZE[1] as f32;
        world.insert(Camera::init_orthographic(15, aspect_ratio));
        world.insert(DeltaTime(0.0));
        world.insert(Gravity::default());


        // Create boundaries (static walls)
        self.create_boundary(world, 0.0, -12.5, config::BOX_SIZE[0], 1.0);   // Bottom
        // self.create_boundary(world, 0.0, 9.5, 20.0, 1.0);    // Top
        self.create_boundary(world, -config::BOX_SIZE[0] / 2.0 - 0.5,  -5.5, 1.0, config::BOX_SIZE[1]);   // Left
        self.create_boundary(world, config::BOX_SIZE[0] / 2.0 + 0.5, -5.5, 1.0, config::BOX_SIZE[1]);    // Right



        self.start_ball_shooting();
        log::info!("Physics demo initialized - 10 balls + 4 walls created");
    }

    fn update(&mut self, world: &mut World, dt: f32) {
        // Handle ball shooting sequence
        if matches!(self.ball_state, BallState::Shooting) {
            self.process_ball_shooting(world, dt);
        }
    }

    // No dispatcher building in app anymore

    fn handle_input(&mut self, world: &mut World, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Left, .. } => {
                // Only allow shooting when in Ready state
                // if matches!(self.ball_state, BallState::Ready) {
                //     self.start_ball_shooting();
                //     true
                // } else {
                //     false // Ignore clicks when already shooting or complete
                // }
                false
            }
            WindowEvent::KeyboardInput { event: key_event, .. } => {
                if key_event.state == ElementState::Pressed {
                    match key_event.physical_key {
                        PhysicalKey::Code(KeyCode::KeyR) => {
                            self.reset(world);
                            true
                        }
                        PhysicalKey::Code(KeyCode::Space) => {
                            // self.gravity_enabled = !self.gravity_enabled;

                            // // Toggle gravity in the world resource
                            // let mut gravity = world.write_resource::<Gravity>();
                            // if self.gravity_enabled {
                            //     gravity.value = config::GRAVITY;
                            // } else {
                            //     gravity.value = [0.0, 0.0];
                            // }

                            // log::info!("Gravity: {}", self.gravity_enabled);
                            true
                        }
                        _ => false
                    }
                } else {
                    false
                }
            }
            _ => false
        }
    }

    fn get_camera_uniform(&self, world: &World) -> [[f32; 4]; 4] {
        let camera = world.read_resource::<Camera>();
        camera.get_view_proj()
    }

    fn get_tile_instances(&self, world: &World) -> HashMap<String, Vec<TileRenderData>> {
        let transforms = world.read_storage::<Transform>();
        let tiles = world.read_storage::<Tile>();

        let mut instances = HashMap::new();

        for (transform, tile) in (&transforms, &tiles).join() {
            instances
                .entry(tile.atlas.clone())
                .or_insert_with(Vec::new)
                .push(TileRenderData {
                    position: transform.position,
                    size: transform.size,
                    uv: tile.uv,
                });
        }

        instances
    }

    fn get_text_instances(&self, _world: &World) -> Vec<TextRenderData> {
        vec![]
    }
}

impl PhysicsApp {
    fn create_boundary(&self, world: &mut World, x: f32, y: f32, width: f32, height: f32) {
        world.create_entity()
            .with(Transform {
                position: [x, y, 0.5],
                size: [width, height],
            })
            .with(Tile {
                uv: [0.0, 1.0, 0.0, 1.0],
                atlas: "box".to_string(),
            })
            .with(RigidBody {
                body_type: BodyType::Static,
                mass: f32::INFINITY,
                restitution: 0.9,
            })
            .with(Velocity::default())
            .with(Force::default())
            .with(BoxCollider { width: width, height: height })
            .build();
    }

    fn reset(&mut self, world: &mut World) {
        use specs::Join;

        // Reset ball state FIRST to stop any ongoing shooting
        self.ball_state = BallState::Ready;
        self.balls_to_shoot.clear();
        self.shot_index = 0;
        self.shoot_timer = 0.0;

        let to_delete: Vec<_> = {
            let entities = world.entities();
            let bodies = world.read_storage::<RigidBody>();

            (&entities, &bodies)
                .join()
                .filter(|(_, body)| body.body_type == BodyType::Dynamic)
                .map(|(entity, _)| entity)
                .collect()
        };

        log::info!("Deleting {} dynamic entities", to_delete.len());

        for entity in to_delete {
            if let Err(e) = world.delete_entity(entity) {
                log::error!("Failed to delete entity: {:?}", e);
            }
        }

        self.start_ball_shooting();

        world.maintain();

        log::info!("Physics demo reset complete - state: {:?}", self.ball_state);
    }

    fn start_ball_shooting(&mut self) {
        self.ball_state = BallState::Shooting;
        self.balls_to_shoot.clear();
        self.shot_index = 0;

        // Just store the same starting position for all balls
        for _ in 0..self.ball_count {
            self.balls_to_shoot.push([self.start_x, self.start_y]);
        }

        log::info!("Started ball shooting sequence - {} balls from ({}, {}) at {}° ({}s intervals)", 
                  self.ball_count, self.start_x, self.start_y, self.shoot_angle, self.shoot_interval);
    }

    fn process_ball_shooting(&mut self, world: &mut World, dt: f32) {
        self.shoot_timer += dt;

        if self.shoot_timer >= self.shoot_interval && self.shot_index < self.balls_to_shoot.len() {
            let pos = self.balls_to_shoot[self.shot_index];

            // Cycle through ball sizes: 0.3, 0.5, 0.7
            let size_index = self.shot_index % 3;
            let radius = self.ball_sizes[size_index];

            log::info!("Shooting ball {} at ({}, {}) with radius {}",
                self.shot_index, pos[0], pos[1], radius);

            self.shoot_ball(world, pos, radius);

            self.shot_index += 1;
            self.shoot_timer = 0.0;

            if self.shot_index >= self.balls_to_shoot.len() {
                self.ball_state = BallState::Complete;
                log::info!("Ball shooting sequence completed");
            }
        }
    }

    fn shoot_ball(&self, world: &mut World, pos: [f32; 2], radius: f32) {
        let angle_rad = self.shoot_angle.to_radians();
        let velocity_x = angle_rad.cos() * self.shoot_speed;
        let velocity_y = angle_rad.sin() * self.shoot_speed;
        let ball_size = radius * 2.0;

        // Mass proportional to area (radius²) for 2D physics
        let mass = self.ball_mass * (radius * radius) / (0.5 * 0.5);

        world.create_entity()
            .with(Transform {
                position: [pos[0], pos[1], 0.5],
                size: [ball_size, ball_size],
            })
            .with(Tile {
                uv: [0.0, 1.0, 0.0, 1.0],
                atlas: "ball".to_string(),
            })
            .with(RigidBody {
                body_type: BodyType::Dynamic,
                mass,
                restitution: self.ball_restitution,
            })
            .with(Velocity {
                linear: [velocity_x, velocity_y],
                angular: 0.0,
            })
            .with(Force::default())
            .with(CircleCollider { radius })
            .build();
        world.create_entity()
            .with(Transform {
                position: [pos[0]+ ball_size * 0.5, pos[1] - ball_size, 0.5],
                size: [ball_size, ball_size],
            })
            .with(Tile {
                uv: [0.0, 1.0, 0.0, 1.0],
                atlas: "ball".to_string(),
            })
            .with(RigidBody {
                body_type: BodyType::Dynamic,
                mass,
                restitution: self.ball_restitution,
            })
            .with(Velocity {
                linear: [velocity_x, velocity_y],
                angular: 0.0,
            })
            .with(Force::default())
            .with(CircleCollider { radius })
            .build();
        world.create_entity()
            .with(Transform {
                position: [pos[0]- ball_size * 0.5, pos[1] + ball_size, 0.5],
                size: [ball_size, ball_size],
            })
            .with(Tile {
                uv: [0.0, 1.0, 0.0, 1.0],
                atlas: "ball".to_string(),
            })
            .with(RigidBody {
                body_type: BodyType::Dynamic,
                mass,
                restitution: self.ball_restitution,
            })
            .with(Velocity {
                linear: [velocity_x, velocity_y],
                angular: 0.0,
            })
            .with(Force::default())
            .with(CircleCollider { radius })
            .build();
    }
}

pub const SCREEN_SIZE: [u32; 2] = [1920, 1080];
pub const HUD_OFFSET_PIXELS: [f32; 2] = [10.0, 10.0];

pub const WORLD_WIDTH: f32 = 40.0;
pub const WORLD_HEIGHT: f32 = 40.0;

pub const INITIAL_PREY: usize = 80;
pub const INITIAL_PREDATORS: usize = 12;

pub const MAX_PREY: usize = 1000;
pub const MAX_PREDATORS: usize = 1000;

pub const PREY_SIZE: [f32; 2] = [0.7, 0.7];
pub const PREDATOR_SIZE: [f32; 2] = [0.9, 0.9];

pub const WORLD_BORDER_THICKNESS: f32 = 0.18;

pub const PREY_MAX_ABS_SPEED: f32 = 4.5;
pub const PREDATOR_MAX_ABS_SPEED: f32 = 3.8;

pub const PREY_MAX_ABS_ANGULAR_VELOCITY: f32 = 1.6;
pub const PREDATOR_MAX_ABS_ANGULAR_VELOCITY: f32 = 1.3;

pub const PREY_VISION_MAX_DISTANCE: f32 = 7.0;
pub const PREY_VISION_TOTAL_ANGLE: f32 = 160.0 * std::f32::consts::PI / 180.0;
pub const PREY_VISION_RAY_INTERVAL: f32 = 20.0 * std::f32::consts::PI / 180.0;

pub const PREDATOR_VISION_MAX_DISTANCE: f32 = 9.0;
pub const PREDATOR_VISION_TOTAL_ANGLE: f32 = 120.0 * std::f32::consts::PI / 180.0;
pub const PREDATOR_VISION_RAY_INTERVAL: f32 = 15.0 * std::f32::consts::PI / 180.0;

pub const PREY_REPRODUCTION_AGE: f32 = 8.0;
pub const PREY_REPRODUCTION_COOLDOWN: f32 = 5.0;
pub const PREY_MAX_AGE: f32 = 60.0;

pub const PREDATOR_FOOD_TO_REPRODUCE: u32 = 3;
pub const PREDATOR_REPRODUCTION_COOLDOWN: f32 = 8.0;
pub const PREDATOR_MAX_AGE: f32 = 80.0;
pub const PREDATOR_STARVATION_TIME: f32 = 12.0;

pub const PREDATION_RADIUS: f32 = 0.6;
pub const OFFSPRING_SPAWN_OFFSET: f32 = 0.8;

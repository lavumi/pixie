use hecs::Entity;

use crate::neural_network::{Genome, NetworkShape};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Species {
    Prey,
    Predator,
}

#[derive(Debug, Clone)]
pub struct Agent {
    pub species: Species,
}

#[derive(Debug, Clone)]
pub struct AgentMotion {
    pub speed: f32,
    pub max_abs_speed: f32,
    pub angular_velocity: f32,
    pub max_abs_angular_velocity: f32,
}

impl AgentMotion {
    pub fn new(
        speed: f32,
        max_abs_speed: f32,
        angular_velocity: f32,
        max_abs_angular_velocity: f32,
    ) -> Self {
        Self {
            speed: speed.clamp(-max_abs_speed, max_abs_speed),
            max_abs_speed,
            angular_velocity: angular_velocity
                .clamp(-max_abs_angular_velocity, max_abs_angular_velocity),
            max_abs_angular_velocity,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Brain {
    pub shape: NetworkShape,
    pub genome: Genome,
}

impl Brain {
    pub fn new(shape: NetworkShape, genome: Genome) -> Self {
        Self { shape, genome }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct BrainOutput {
    pub angular_velocity: f32,
    pub speed: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LifeCycle {
    pub age: f32,
    pub time_since_food: f32,
    pub food_eaten: u32,
}

impl Default for LifeCycle {
    fn default() -> Self {
        Self {
            age: 0.0,
            time_since_food: 0.0,
            food_eaten: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Reproduction {
    pub cooldown_remaining: f32,
}

impl Reproduction {
    pub fn new(cooldown_remaining: f32) -> Self {
        Self {
            cooldown_remaining: cooldown_remaining.max(0.0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vision {
    pub max_distance: f32,
    pub total_angle: f32,
    pub ray_interval: f32,
}

impl Vision {
    pub fn new(max_distance: f32, total_angle: f32, ray_interval: f32) -> Self {
        assert!(
            max_distance.is_finite() && max_distance > 0.0,
            "vision max distance must be finite and positive"
        );
        assert!(
            total_angle.is_finite() && (0.0..=std::f32::consts::TAU).contains(&total_angle),
            "vision total angle must be finite and between zero and TAU"
        );
        assert!(
            ray_interval.is_finite() && ray_interval > 0.0,
            "vision ray interval must be finite and positive"
        );

        Self {
            max_distance,
            total_angle,
            ray_interval,
        }
    }

    pub fn ray_count(&self) -> usize {
        (self.total_angle / self.ray_interval).floor() as usize + 1
    }

    pub fn ray_angle(&self, index: usize) -> f32 {
        assert!(index < self.ray_count(), "vision ray index out of bounds");
        let sampled_angle = (self.ray_count() - 1) as f32 * self.ray_interval;
        -sampled_angle * 0.5 + index as f32 * self.ray_interval
    }

    pub fn input_len(&self) -> usize {
        self.ray_count() * VisionOutput::CHANNELS_PER_RAY
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VisionHit {
    pub target: Entity,
    pub species: Species,
    pub distance: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VisionOutput {
    pub hits: Vec<Option<VisionHit>>,
}

impl VisionOutput {
    pub const CHANNELS_PER_RAY: usize = 3;

    pub fn empty(vision: &Vision) -> Self {
        Self {
            hits: vec![None; vision.ray_count()],
        }
    }

    pub fn inputs(&self, vision: &Vision) -> Vec<f32> {
        assert_eq!(
            self.hits.len(),
            vision.ray_count(),
            "vision output does not match vision parameters"
        );

        let mut inputs = Vec::with_capacity(vision.input_len());
        for hit in &self.hits {
            match hit {
                Some(hit) => {
                    inputs.push((hit.distance / vision.max_distance).clamp(0.0, 1.0));
                    inputs.push(f32::from(hit.species == Species::Prey));
                    inputs.push(f32::from(hit.species == Species::Predator));
                }
                None => inputs.extend_from_slice(&[1.0, 0.0, 0.0]),
            }
        }
        inputs
    }
}

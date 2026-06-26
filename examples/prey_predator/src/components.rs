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

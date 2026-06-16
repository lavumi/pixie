#[derive(Debug, Clone)]
pub struct SimulationConfig {
    pub world_width: f32,
    pub world_height: f32,
    pub initial_prey: usize,
    pub initial_predators: usize,
    pub max_prey: usize,
    pub max_predators: usize,
}

#[derive(Debug, Clone, Default)]
pub struct SimulationStats {
    pub elapsed_time: f32,
    pub prey_alive: usize,
    pub predators_alive: usize,
}

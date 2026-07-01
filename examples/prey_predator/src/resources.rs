use hecs::Entity;

use crate::components::Species;
use crate::config;

#[derive(Debug, Clone)]
pub struct SimulationConfig {
    pub world_width: f32,
    pub world_height: f32,
    pub initial_prey: usize,
    pub initial_predators: usize,
    pub max_prey: usize,
    pub max_predators: usize,
    pub prey_reproduction_age: f32,
    pub prey_reproduction_cooldown: f32,
    pub prey_max_age: f32,
    pub predator_food_to_reproduce: u32,
    pub predator_reproduction_cooldown: f32,
    pub predator_max_age: f32,
    pub predator_starvation_time: f32,
    pub predation_radius: f32,
    pub offspring_spawn_offset: f32,
    pub brain_hidden_layers: Vec<usize>,
    pub brain_output_size: usize,
    pub brain_mutation_rate: f32,
    pub brain_mutation_sigma: f32,
    pub brain_reset_rate: f32,
    pub brain_max_abs_gene: f32,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            world_width: config::WORLD_WIDTH,
            world_height: config::WORLD_HEIGHT,
            initial_prey: config::INITIAL_PREY,
            initial_predators: config::INITIAL_PREDATORS,
            max_prey: config::MAX_PREY,
            max_predators: config::MAX_PREDATORS,
            prey_reproduction_age: config::PREY_REPRODUCTION_AGE,
            prey_reproduction_cooldown: config::PREY_REPRODUCTION_COOLDOWN,
            prey_max_age: config::PREY_MAX_AGE,
            predator_food_to_reproduce: config::PREDATOR_FOOD_TO_REPRODUCE,
            predator_reproduction_cooldown: config::PREDATOR_REPRODUCTION_COOLDOWN,
            predator_max_age: config::PREDATOR_MAX_AGE,
            predator_starvation_time: config::PREDATOR_STARVATION_TIME,
            predation_radius: config::PREDATION_RADIUS,
            offspring_spawn_offset: config::OFFSPRING_SPAWN_OFFSET,
            brain_hidden_layers: config::BRAIN_HIDDEN_LAYERS.to_vec(),
            brain_output_size: config::BRAIN_OUTPUT_SIZE,
            brain_mutation_rate: config::BRAIN_MUTATION_RATE,
            brain_mutation_sigma: config::BRAIN_MUTATION_SIGMA,
            brain_reset_rate: config::BRAIN_RESET_RATE,
            brain_max_abs_gene: config::BRAIN_MAX_ABS_GENE,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SimulationStats {
    pub elapsed_time: f32,
    pub prey_alive: usize,
    pub predators_alive: usize,
    pub prey_born: usize,
    pub predators_born: usize,
    pub prey_eaten: usize,
    pub prey_died_of_age: usize,
    pub predators_died: usize,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum DeathReason {
    Eaten,
    Age,
    Starvation,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DeathRequest {
    pub entity: Entity,
    pub species: Species,
    pub reason: DeathReason,
}

#[derive(Debug, Default)]
pub struct DeathQueue {
    pub requests: Vec<DeathRequest>,
}

impl DeathQueue {
    pub fn contains(&self, entity: Entity) -> bool {
        self.requests.iter().any(|request| request.entity == entity)
    }

    pub fn push(&mut self, request: DeathRequest) {
        if !self.contains(request.entity) {
            self.requests.push(request);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpawnRequest {
    pub parent: Entity,
    pub species: Species,
    pub parent_position: [f32; 2],
}

#[derive(Debug, Default)]
pub struct SpawnQueue {
    pub requests: Vec<SpawnRequest>,
}

const CLICK_DRAG_THRESHOLD_SQUARED: f32 = 16.0;

#[derive(Debug, Default)]
pub struct SelectionState {
    pub selected: Option<Entity>,
    pub cursor_screen_position: Option<[f32; 2]>,
    primary_press_origin: Option<[f32; 2]>,
}

impl SelectionState {
    #[cfg(test)]
    pub fn with_selected(entity: Entity) -> Self {
        Self {
            selected: Some(entity),
            ..Self::default()
        }
    }

    pub fn update_cursor(&mut self, position: [f32; 2]) {
        self.cursor_screen_position = Some(position);
        let Some(origin) = self.primary_press_origin else {
            return;
        };
        let delta = [position[0] - origin[0], position[1] - origin[1]];
        let distance_squared = delta[0] * delta[0] + delta[1] * delta[1];
        if distance_squared > CLICK_DRAG_THRESHOLD_SQUARED {
            self.selected = None;
        }
    }

    pub fn begin_primary_press(&mut self) {
        self.primary_press_origin = self.cursor_screen_position;
    }

    pub fn finish_primary_press(&mut self) -> Option<[f32; 2]> {
        let origin = self.primary_press_origin.take()?;
        let current = self.cursor_screen_position?;
        let delta = [current[0] - origin[0], current[1] - origin[1]];
        let distance_squared = delta[0] * delta[0] + delta[1] * delta[1];

        (distance_squared <= CLICK_DRAG_THRESHOLD_SQUARED).then_some(current)
    }

    pub fn clear(&mut self) {
        self.selected = None;
        self.primary_press_origin = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primary_press_distinguishes_click_from_drag() {
        let mut world = hecs::World::new();
        let selected = world.spawn(());
        let mut selection = SelectionState::with_selected(selected);
        selection.update_cursor([10.0, 20.0]);
        selection.begin_primary_press();
        selection.update_cursor([12.0, 22.0]);
        assert_eq!(selection.finish_primary_press(), Some([12.0, 22.0]));
        assert_eq!(selection.selected, Some(selected));

        selection.begin_primary_press();
        selection.update_cursor([20.0, 22.0]);
        assert_eq!(selection.finish_primary_press(), None);
        assert_eq!(selection.selected, None);
    }
}

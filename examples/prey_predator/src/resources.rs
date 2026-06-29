use hecs::Entity;

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

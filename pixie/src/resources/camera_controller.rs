use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};

use super::{Camera, WindowSize};

const DEFAULT_ZOOM_FACTOR: f32 = 1.15;
const DEFAULT_PAN_SPEED: f32 = 1.0;

#[derive(Debug, Clone)]
pub struct CameraController {
    pub enabled: bool,
    pub zoom_enabled: bool,
    pub pan_enabled: bool,
    pub zoom_factor: f32,
    pub pan_button: MouseButton,
    pub pan_speed: f32,
    is_dragging: bool,
    last_cursor_position: Option<[f64; 2]>,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            enabled: true,
            zoom_enabled: true,
            pan_enabled: true,
            zoom_factor: DEFAULT_ZOOM_FACTOR,
            pan_button: MouseButton::Left,
            pan_speed: DEFAULT_PAN_SPEED,
            is_dragging: false,
            last_cursor_position: None,
        }
    }
}

impl CameraController {
    pub fn handle_event(
        &mut self,
        event: &WindowEvent,
        camera: &mut Camera,
        window_size: WindowSize,
    ) -> bool {
        if !self.enabled {
            return false;
        }

        match event {
            WindowEvent::MouseWheel { delta, .. } if self.zoom_enabled => {
                self.handle_mouse_wheel(camera, delta)
            }
            WindowEvent::MouseInput { state, button, .. }
                if self.pan_enabled && *button == self.pan_button =>
            {
                self.is_dragging = *state == ElementState::Pressed;
                if !self.is_dragging {
                    self.last_cursor_position = None;
                }
                true
            }
            WindowEvent::CursorMoved { position, .. } if self.pan_enabled => {
                self.handle_cursor_moved(camera, window_size, [position.x, position.y])
            }
            _ => false,
        }
    }

    fn handle_mouse_wheel(&self, camera: &mut Camera, delta: &MouseScrollDelta) -> bool {
        let scroll_y = match delta {
            MouseScrollDelta::LineDelta(_, y) => *y,
            MouseScrollDelta::PixelDelta(position) => position.y as f32,
        };

        if scroll_y.abs() < f32::EPSILON {
            return false;
        }

        if scroll_y > 0.0 {
            camera.zoom_in(self.zoom_factor);
        } else {
            camera.zoom_out(self.zoom_factor);
        }

        true
    }

    fn handle_cursor_moved(
        &mut self,
        camera: &mut Camera,
        window_size: WindowSize,
        position: [f64; 2],
    ) -> bool {
        if !self.is_dragging {
            self.last_cursor_position = Some(position);
            return false;
        }

        let Some(previous_position) = self.last_cursor_position.replace(position) else {
            return true;
        };

        let delta = [
            position[0] - previous_position[0],
            position[1] - previous_position[1],
        ];

        if delta[0].abs() < f64::EPSILON && delta[1].abs() < f64::EPSILON {
            return true;
        }

        let world_units_per_pixel = camera.zoom() * 2.0 / window_size.height.max(1) as f32;
        camera.move_camera_delta([
            -(delta[0] as f32) * world_units_per_pixel * self.pan_speed,
            (delta[1] as f32) * world_units_per_pixel * self.pan_speed,
        ]);

        true
    }
}

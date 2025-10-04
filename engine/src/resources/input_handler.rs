use winit::event::ElementState;
use winit::keyboard::{KeyCode, PhysicalKey};

pub struct InputHandler{
    pub jump: bool,
}


impl Default for InputHandler {
    fn default() -> Self {
        InputHandler {
            jump: false
        }
    }
}


impl InputHandler {
    pub fn receive_keyboard_input(&mut self, state : ElementState, physical_key: PhysicalKey) -> bool {
        match physical_key {
            PhysicalKey::Code(KeyCode::Space) => {
                match state {
                    ElementState::Pressed => {
                        self.jump = true;
                    }
                    ElementState::Released => {
                        self.jump = false;
                    }
                }
                true
            }
            _ => {
                false
            }
        }
    }
}
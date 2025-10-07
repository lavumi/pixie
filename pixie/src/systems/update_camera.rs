use specs::{System, Write};

use crate::resources::Camera;

pub struct UpdateCamera;

impl<'a> System<'a> for UpdateCamera {
    type SystemData = (
        Write<'a, Camera>,
    );

    fn run(&mut self, _: Self::SystemData) {
        // let ( mut camera) = data;
        // let player_pos = [pos.0, pos.1];
        // camera.move_camera(player_pos);
    }
}
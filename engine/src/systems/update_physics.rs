use specs::{Entities, System, WriteStorage};

use crate::components::{Collider, Transform};

pub struct UpdatePhysics;

impl<'a> System<'a> for UpdatePhysics {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Collider>,
        WriteStorage<'a, Transform>
    );

    fn run(&mut self, (_entities, mut _col, mut _tf): Self::SystemData) {}
}
use specs::{Join, System, WriteStorage, ReadStorage, Read};
use crate::components::{Force, RigidBody, BodyType};

// Gravity resource
pub struct Gravity {
    pub value: [f32; 2],
}

impl Default for Gravity {
    fn default() -> Self {
        Gravity {
            value: [0.0, -9.8],
        }
    }
}

pub struct ApplyGravity;

impl Default for ApplyGravity {
    fn default() -> Self {
        ApplyGravity
    }
}

impl<'a> System<'a> for ApplyGravity {
    type SystemData = (
        WriteStorage<'a, Force>,
        ReadStorage<'a, RigidBody>,
        Read<'a, Gravity>,
    );

    fn run(&mut self, (mut forces, bodies, gravity): Self::SystemData) {
        for (force, body) in (&mut forces, &bodies).join() {
            if body.body_type == BodyType::Dynamic {
                force.linear[0] += gravity.value[0] * body.mass;
                force.linear[1] += gravity.value[1] * body.mass;
            }
        }
    }
}

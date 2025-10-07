use specs::{Join, Read, System, WriteStorage};
use crate::components::{RigidBody, Velocity, Force, Transform, BodyType};
use crate::resources::DeltaTime;

pub struct UpdatePhysics;

impl<'a> System<'a> for UpdatePhysics {
    type SystemData = (
        WriteStorage<'a, Transform>,
        WriteStorage<'a, Velocity>,
        WriteStorage<'a, Force>,
        WriteStorage<'a, RigidBody>,
        Read<'a, DeltaTime>,
    );

    fn run(&mut self, (mut transforms, mut velocities, mut forces, bodies, dt): Self::SystemData) {
        for (transform, velocity, force, body) in (&mut transforms, &mut velocities, &mut forces, &bodies).join() {
            if body.body_type != BodyType::Dynamic {
                continue;
            }

            // Apply forces (F = ma -> a = F/m)
            let acceleration = [
                force.linear[0] / body.mass,
                force.linear[1] / body.mass,
            ];

            // Update velocity
            velocity.linear[0] += acceleration[0] * dt.0;
            velocity.linear[1] += acceleration[1] * dt.0;

            // Update position
            transform.position[0] += velocity.linear[0] * dt.0;
            transform.position[1] += velocity.linear[1] * dt.0;

            // Clear forces
            force.linear = [0.0, 0.0];
            force.torque = 0.0;
        }
    }
}
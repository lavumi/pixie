use hecs::World;
use crate::components::{RigidBody, Velocity, Force, Transform, BodyType};
use crate::resources::{DeltaTime, ResourceContainer};

/// Update physics system - applies forces and updates positions
pub fn update_physics(world: &mut World, resources: &mut ResourceContainer) {
    let dt = resources.get::<DeltaTime>()
        .expect("DeltaTime resource not found");

    // Query for entities with physics components
    for (_entity, (transform, velocity, force, body)) in
        world.query_mut::<(&mut Transform, &mut Velocity, &mut Force, &RigidBody)>()
    {
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
use hecs::World;
use crate::components::{Force, RigidBody, BodyType};
use crate::resources::ResourceContainer;

/// Gravity resource
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

/// Apply gravity system - adds gravitational force to dynamic bodies
pub fn apply_gravity(world: &mut World, resources: &mut ResourceContainer) {
    let gravity = resources.get::<Gravity>()
        .expect("Gravity resource not found");

    // Query for entities with Force and RigidBody components
    for (_entity, (force, body)) in world.query_mut::<(&mut Force, &RigidBody)>() {
        if body.body_type == BodyType::Dynamic {
            force.linear[0] += gravity.value[0] * body.mass;
            force.linear[1] += gravity.value[1] * body.mass;
        }
    }
}

use crate::components::{BodyType, Force, RigidBody, Transform, Velocity};
use crate::resources::{DeltaTime, ResourceContainer};
use hecs::World;

/// Update physics system - applies forces and updates positions
pub fn update_physics(world: &mut World, resources: &mut ResourceContainer) {
    let dt = resources
        .get::<DeltaTime>()
        .expect("DeltaTime resource not found");

    // Query for entities with physics components
    for (_entity, (transform, velocity, force, body)) in
        world.query_mut::<(&mut Transform, &mut Velocity, &mut Force, &RigidBody)>()
    {
        if body.body_type != BodyType::Dynamic {
            continue;
        }

        // Apply forces (F = ma -> a = F/m)
        let acceleration = [force.linear[0] / body.mass, force.linear[1] / body.mass];

        // Update velocity
        velocity.linear[0] += acceleration[0] * dt.0;
        velocity.linear[1] += acceleration[1] * dt.0;
        velocity.angular += force.torque / body.mass * dt.0;

        // Update transform using semi-implicit Euler integration
        transform.position[0] += velocity.linear[0] * dt.0;
        transform.position[1] += velocity.linear[1] * dt.0;
        transform.rotation += velocity.angular * dt.0;

        // Clear forces
        force.linear = [0.0, 0.0];
        force.torque = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dynamic_body() -> RigidBody {
        RigidBody {
            body_type: BodyType::Dynamic,
            mass: 2.0,
            restitution: 0.0,
        }
    }

    #[test]
    fn integrates_angular_velocity_and_torque() {
        let mut world = World::new();
        let entity = world.spawn((
            Transform::new([0.0, 0.0, 0.0], [1.0, 1.0]),
            Velocity {
                linear: [0.0, 0.0],
                angular: 1.0,
            },
            Force {
                linear: [0.0, 0.0],
                torque: 4.0,
            },
            dynamic_body(),
        ));
        let mut resources = ResourceContainer::new();
        resources.insert(DeltaTime(0.5));

        update_physics(&mut world, &mut resources);

        let transform = world.get::<&Transform>(entity).unwrap();
        let velocity = world.get::<&Velocity>(entity).unwrap();
        let force = world.get::<&Force>(entity).unwrap();
        assert!((velocity.angular - 2.0).abs() < f32::EPSILON);
        assert!((transform.rotation - 1.0).abs() < f32::EPSILON);
        assert_eq!(force.torque, 0.0);
    }

    #[test]
    fn does_not_integrate_non_dynamic_bodies() {
        let mut world = World::new();
        let static_entity = world.spawn((
            Transform::with_rotation([0.0, 0.0, 0.0], [1.0, 1.0], 0.25),
            Velocity {
                linear: [1.0, 0.0],
                angular: 2.0,
            },
            Force {
                linear: [2.0, 0.0],
                torque: 3.0,
            },
            RigidBody {
                body_type: BodyType::Static,
                mass: f32::INFINITY,
                restitution: 0.0,
            },
        ));
        let kinematic_entity = world.spawn((
            Transform::with_rotation([0.0, 0.0, 0.0], [1.0, 1.0], -0.5),
            Velocity {
                linear: [1.0, 0.0],
                angular: 2.0,
            },
            Force {
                linear: [2.0, 0.0],
                torque: 3.0,
            },
            RigidBody {
                body_type: BodyType::Kinematic,
                mass: 1.0,
                restitution: 0.0,
            },
        ));
        let mut resources = ResourceContainer::new();
        resources.insert(DeltaTime(0.5));

        update_physics(&mut world, &mut resources);

        assert_eq!(
            world.get::<&Transform>(static_entity).unwrap().rotation,
            0.25
        );
        assert_eq!(
            world.get::<&Transform>(kinematic_entity).unwrap().rotation,
            -0.5
        );
    }
}

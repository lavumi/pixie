use hecs::{World, Entity};
use crate::components::{Transform, Velocity, RigidBody, CircleCollider, BoxCollider, BodyType};
use crate::resources::ResourceContainer;

#[derive(Debug, Clone)]
pub struct CollisionInfo {
    pub entity1: Entity,
    pub entity2: Entity,
    pub normal: [f32; 2],
    pub penetration: f32,
    pub contact_point: [f32; 2],
    pub mass1: f32,
    pub mass2: f32,
    pub restitution: f32,
    pub body_type1: BodyType,
    pub body_type2: BodyType,
}

struct EntityCollisionData {
    entity: Entity,
    position: [f32; 2],
    mass: f32,
    restitution: f32,
    body_type: BodyType,
    collider: ColliderType,
}

enum ColliderType {
    Circle { radius: f32 },
    Box { width: f32, height: f32 },
}

/// Collision system - detects and resolves collisions between rigid bodies
pub fn collision_system(world: &mut World, _resources: &mut ResourceContainer) {
    // Collision iteration for stability
    const ITERATIONS: usize = 8;

    for _ in 0..ITERATIONS {
        let mut collisions = Vec::new();

        // Collect entity data
        let mut entity_data = Vec::new();

        // Collect circle collider entities
        for (entity, (transform, body, collider)) in world.query::<(&Transform, &RigidBody, &CircleCollider)>().iter() {
            entity_data.push(EntityCollisionData {
                entity,
                position: [transform.position[0], transform.position[1]],
                mass: body.mass,
                restitution: body.restitution,
                body_type: body.body_type,
                collider: ColliderType::Circle { radius: collider.radius },
            });
        }

        // Collect box collider entities
        for (entity, (transform, body, collider)) in world.query::<(&Transform, &RigidBody, &BoxCollider)>().iter() {
            entity_data.push(EntityCollisionData {
                entity,
                position: [transform.position[0], transform.position[1]],
                mass: body.mass,
                restitution: body.restitution,
                body_type: body.body_type,
                collider: ColliderType::Box {
                    width: collider.width,
                    height: collider.height
                },
            });
        }

        // Detect collisions
        for i in 0..entity_data.len() {
            for j in (i + 1)..entity_data.len() {
                let data1 = &entity_data[i];
                let data2 = &entity_data[j];

                // Skip if both are static
                if data1.body_type == BodyType::Static && data2.body_type == BodyType::Static {
                    continue;
                }

                if let Some(collision) = detect_collision(data1, data2) {
                    collisions.push(collision);
                }
            }
        }

        // Resolve collisions
        resolve_collisions(world, collisions);
    }
}

fn detect_collision(data1: &EntityCollisionData, data2: &EntityCollisionData) -> Option<CollisionInfo> {
    match (&data1.collider, &data2.collider) {
        (ColliderType::Circle { radius: r1 }, ColliderType::Circle { radius: r2 }) => {
            detect_circle_circle(data1, data2, *r1, *r2)
        }
        (ColliderType::Circle { radius }, ColliderType::Box { width, height }) => {
            detect_circle_box(data1, data2, *radius, *width, *height)
        }
        (ColliderType::Box { width, height }, ColliderType::Circle { radius }) => {
            detect_box_circle(data1, data2, *width, *height, *radius)
        }
        (ColliderType::Box { width: w1, height: h1 }, ColliderType::Box { width: w2, height: h2 }) => {
            detect_box_box(data1, data2, *w1, *h1, *w2, *h2)
        }
    }
}

fn detect_circle_circle(
    data1: &EntityCollisionData,
    data2: &EntityCollisionData,
    radius1: f32,
    radius2: f32,
) -> Option<CollisionInfo> {
    let dx = data2.position[0] - data1.position[0];
    let dy = data2.position[1] - data1.position[1];
    let distance_squared = dx * dx + dy * dy;
    let min_distance = radius1 + radius2;
    let min_distance_squared = min_distance * min_distance;

    if distance_squared < min_distance_squared && distance_squared > 0.00001 {
        let distance = distance_squared.sqrt();
        let penetration = min_distance - distance;
        let normal = [dx / distance, dy / distance];

        Some(CollisionInfo {
            entity1: data1.entity,
            entity2: data2.entity,
            normal,
            penetration,
            contact_point: [
                data1.position[0] + normal[0] * radius1,
                data1.position[1] + normal[1] * radius1,
            ],
            mass1: data1.mass,
            mass2: data2.mass,
            restitution: data1.restitution.min(data2.restitution),
            body_type1: data1.body_type,
            body_type2: data2.body_type,
        })
    } else {
        None
    }
}

fn detect_circle_box(
    circle_data: &EntityCollisionData,
    box_data: &EntityCollisionData,
    radius: f32,
    box_width: f32,
    box_height: f32,
) -> Option<CollisionInfo> {
    let half_width = box_width / 2.0;
    let half_height = box_height / 2.0;

    // Find closest point on box to circle center
    let closest_x = (circle_data.position[0] - box_data.position[0])
        .max(-half_width)
        .min(half_width);
    let closest_y = (circle_data.position[1] - box_data.position[1])
        .max(-half_height)
        .min(half_height);

    // Convert to world space
    let closest_point_x = box_data.position[0] + closest_x;
    let closest_point_y = box_data.position[1] + closest_y;

    // Calculate distance from circle center to closest point
    let dx = circle_data.position[0] - closest_point_x;
    let dy = circle_data.position[1] - closest_point_y;
    let distance_squared = dx * dx + dy * dy;

    if distance_squared < radius * radius {
        let distance = distance_squared.sqrt();

        // Handle case where circle center is inside box
        let (normal, penetration) = if distance < -0.0001 {
            // Circle center is inside box - push along closest axis
            let overlap_x = half_width - (circle_data.position[0] - box_data.position[0]).abs();
            let overlap_y = half_height - (circle_data.position[1] - box_data.position[1]).abs();

            if overlap_x < overlap_y {
                let dir = if circle_data.position[0] < box_data.position[0] { -1.0 } else { 1.0 };
                ([dir, 0.0], radius + overlap_x)
            } else {
                let dir = if circle_data.position[1] < box_data.position[1] { -1.0 } else { 1.0 };
                ([0.0, dir], radius + overlap_y)
            }
        } else {
            // Normal case
            let normal = [-dx / distance, - dy / distance];
            let penetration = radius - distance;
            (normal, penetration)
        };

        Some(CollisionInfo {
            entity1: circle_data.entity,
            entity2: box_data.entity,
            normal,
            penetration,
            contact_point: [closest_point_x, closest_point_y],
            mass1: circle_data.mass,
            mass2: box_data.mass,
            restitution: circle_data.restitution.min(box_data.restitution),
            body_type1: circle_data.body_type,
            body_type2: box_data.body_type,
        })
    } else {
        None
    }
}

fn detect_box_circle(
    box_data: &EntityCollisionData,
    circle_data: &EntityCollisionData,
    box_width: f32,
    box_height: f32,
    radius: f32,
) -> Option<CollisionInfo> {
    // Reuse circle-box detection but flip the result
    let mut collision = detect_circle_box(circle_data, box_data, radius, box_width, box_height)?;

    // Swap entities and flip normal
    std::mem::swap(&mut collision.entity1, &mut collision.entity2);
    std::mem::swap(&mut collision.mass1, &mut collision.mass2);
    std::mem::swap(&mut collision.body_type1, &mut collision.body_type2);
    collision.normal[0] = -collision.normal[0];
    collision.normal[1] = -collision.normal[1];

    Some(collision)
}

fn detect_box_box(
    data1: &EntityCollisionData,
    data2: &EntityCollisionData,
    width1: f32,
    height1: f32,
    width2: f32,
    height2: f32,
) -> Option<CollisionInfo> {
    let half_width1 = width1 / 2.0;
    let half_height1 = height1 / 2.0;
    let half_width2 = width2 / 2.0;
    let half_height2 = height2 / 2.0;

    let left1 = data1.position[0] - half_width1;
    let right1 = data1.position[0] + half_width1;
    let top1 = data1.position[1] + half_height1;
    let bottom1 = data1.position[1] - half_height1;

    let left2 = data2.position[0] - half_width2;
    let right2 = data2.position[0] + half_width2;
    let top2 = data2.position[1] + half_height2;
    let bottom2 = data2.position[1] - half_height2;

    if right1 >= left2 && left1 <= right2 && top1 >= bottom2 && bottom1 <= top2 {
        let overlap_x = (right1 - left2).min(right2 - left1);
        let overlap_y = (top1 - bottom2).min(top2 - bottom1);

        let (normal, penetration) = if overlap_x < overlap_y {
            let dir = if data1.position[0] < data2.position[0] { -1.0 } else { 1.0 };
            ([dir, 0.0], overlap_x)
        } else {
            let dir = if data1.position[1] < data2.position[1] { -1.0 } else { 1.0 };
            ([0.0, dir], overlap_y)
        };

        Some(CollisionInfo {
            entity1: data1.entity,
            entity2: data2.entity,
            normal,
            penetration,
            contact_point: [
                (data1.position[0] + data2.position[0]) / 2.0,
                (data1.position[1] + data2.position[1]) / 2.0,
            ],
            mass1: data1.mass,
            mass2: data2.mass,
            restitution: data1.restitution.min(data2.restitution),
            body_type1: data1.body_type,
            body_type2: data2.body_type,
        })
    } else {
        None
    }
}

fn resolve_collisions(
    world: &mut World,
    collisions: Vec<CollisionInfo>,
) {
    const CORRECTION_PERCENT: f32 = 1.2;  // Increase position correction even more
    const SLOP: f32 = 0.0001;  // Reduce slop for better separation

    for collision in collisions {
        if collision.penetration < 0.00001 {
            continue;
        }

        let normal = collision.normal;

        // Position correction
        let correction = ((collision.penetration - SLOP).max(0.0) * CORRECTION_PERCENT) /
            match (collision.body_type1, collision.body_type2) {
                (BodyType::Dynamic, BodyType::Dynamic) => {
                    1.0 / collision.mass1 + 1.0 / collision.mass2
                }
                (BodyType::Dynamic, BodyType::Static) => 1.0 / collision.mass1,
                (BodyType::Static, BodyType::Dynamic) => 1.0 / collision.mass2,
                _ => continue,
            };

        // Apply position corrections
        match (collision.body_type1, collision.body_type2) {
            (BodyType::Dynamic, BodyType::Dynamic) => {
                let ratio1 = (1.0 / collision.mass1) / (1.0 / collision.mass1 + 1.0 / collision.mass2);
                let ratio2 = (1.0 / collision.mass2) / (1.0 / collision.mass1 + 1.0 / collision.mass2);

                if let Ok(mut transform) = world.get::<&mut Transform>(collision.entity1) {
                    transform.position[0] -= normal[0] * correction * ratio1;
                    transform.position[1] -= normal[1] * correction * ratio1;
                }
                if let Ok(mut transform) = world.get::<&mut Transform>(collision.entity2) {
                    transform.position[0] += normal[0] * correction * ratio2;
                    transform.position[1] += normal[1] * correction * ratio2;
                }
            }
            (BodyType::Dynamic, BodyType::Static) => {
                if let Ok(mut transform) = world.get::<&mut Transform>(collision.entity1) {
                    transform.position[0] -= normal[0] * correction;
                    transform.position[1] -= normal[1] * correction;
                }
            }
            (BodyType::Static, BodyType::Dynamic) => {
                if let Ok(mut transform) = world.get::<&mut Transform>(collision.entity2) {
                    transform.position[0] += normal[0] * correction;
                    transform.position[1] += normal[1] * correction;
                }
            }
            _ => {}
        }

        // Velocity resolution - need to get velocities first to calculate relative velocity
        let vel1_linear = world.get::<&Velocity>(collision.entity1)
            .ok()
            .map(|v| v.linear);
        let vel2_linear = world.get::<&Velocity>(collision.entity2)
            .ok()
            .map(|v| v.linear);

        if let (Some(v1_linear), Some(v2_linear)) = (vel1_linear, vel2_linear) {
            let relative_vel = [
                v2_linear[0] - v1_linear[0],
                v2_linear[1] - v1_linear[1],
            ];
            let vel_along_normal = relative_vel[0] * normal[0] + relative_vel[1] * normal[1];

            // Don't resolve if separating (with small threshold for stability)
            if vel_along_normal > -0.001 {
                continue;
            }

            let impulse_scalar = -(1.0 + collision.restitution) * vel_along_normal /
                match (collision.body_type1, collision.body_type2) {
                    (BodyType::Dynamic, BodyType::Dynamic) => {
                        1.0 / collision.mass1 + 1.0 / collision.mass2
                    }
                    (BodyType::Dynamic, BodyType::Static) => 1.0 / collision.mass1,
                    (BodyType::Static, BodyType::Dynamic) => 1.0 / collision.mass2,
                    _ => continue,
                };

            let impulse = [impulse_scalar * normal[0], impulse_scalar * normal[1]];

            if collision.body_type1 == BodyType::Dynamic {
                if let Ok(mut velocity) = world.get::<&mut Velocity>(collision.entity1) {
                    velocity.linear[0] -= impulse[0] / collision.mass1;
                    velocity.linear[1] -= impulse[1] / collision.mass1;
                }
            }

            if collision.body_type2 == BodyType::Dynamic {
                if let Ok(mut velocity) = world.get::<&mut Velocity>(collision.entity2) {
                    velocity.linear[0] += impulse[0] / collision.mass2;
                    velocity.linear[1] += impulse[1] / collision.mass2;
                }
            }
        }
    }
}

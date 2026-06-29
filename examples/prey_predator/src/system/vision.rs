use hecs::{Entity, World};
use pixie::Transform;

use crate::components::{Agent, Species, Vision, VisionHit, VisionOutput};

#[derive(Debug, Clone, Copy)]
struct AgentSnapshot {
    entity: Entity,
    position: [f32; 2],
    rotation: f32,
    radius: f32,
    species: Species,
}

pub fn collect_vision(world: &mut World) {
    let snapshots: Vec<AgentSnapshot> = world
        .query::<(&Transform, &Agent)>()
        .iter()
        .map(|(entity, (transform, agent))| AgentSnapshot {
            entity,
            position: [transform.position[0], transform.position[1]],
            rotation: transform.rotation,
            radius: transform.size[0].abs().max(transform.size[1].abs()) * 0.5,
            species: agent.species,
        })
        .collect();

    let viewers: Vec<(Entity, Vision)> = world
        .query::<&Vision>()
        .iter()
        .map(|(entity, vision)| (entity, *vision))
        .collect();

    for (entity, vision) in viewers {
        let Some(viewer) = snapshots.iter().find(|snapshot| snapshot.entity == entity) else {
            continue;
        };
        let output = cast_vision(*viewer, vision, &snapshots);
        let has_output = world.get::<&VisionOutput>(entity).is_ok();

        if has_output {
            let mut current = world
                .get::<&mut VisionOutput>(entity)
                .expect("vision output disappeared while collecting vision");
            *current = output;
        } else {
            world
                .insert_one(entity, output)
                .expect("vision viewer disappeared while collecting vision");
        }
    }
}

fn cast_vision(viewer: AgentSnapshot, vision: Vision, targets: &[AgentSnapshot]) -> VisionOutput {
    let mut output = VisionOutput::empty(&vision);

    for ray_index in 0..vision.ray_count() {
        let angle = viewer.rotation + vision.ray_angle(ray_index);
        let direction = [angle.cos(), angle.sin()];

        output.hits[ray_index] = targets
            .iter()
            .filter(|target| target.entity != viewer.entity)
            .filter_map(|target| {
                ray_circle_distance(
                    viewer.position,
                    direction,
                    target.position,
                    target.radius,
                    vision.max_distance,
                )
                .map(|distance| VisionHit {
                    target: target.entity,
                    species: target.species,
                    distance,
                })
            })
            .min_by(|left, right| left.distance.total_cmp(&right.distance));
    }

    output
}

fn ray_circle_distance(
    origin: [f32; 2],
    direction: [f32; 2],
    center: [f32; 2],
    radius: f32,
    max_distance: f32,
) -> Option<f32> {
    let offset = [origin[0] - center[0], origin[1] - center[1]];
    let projection = offset[0] * direction[0] + offset[1] * direction[1];
    let offset_squared = offset[0] * offset[0] + offset[1] * offset[1];
    let discriminant = projection * projection - (offset_squared - radius * radius);

    if discriminant < 0.0 {
        return None;
    }

    let root = discriminant.sqrt();
    let near = -projection - root;
    let far = -projection + root;
    let distance = if near >= 0.0 { near } else { far };

    (distance >= 0.0 && distance <= max_distance).then_some(distance)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spawn_agent(
        world: &mut World,
        position: [f32; 2],
        rotation: f32,
        size: f32,
        species: Species,
        vision: Option<Vision>,
    ) -> Entity {
        let transform =
            Transform::with_rotation([position[0], position[1], 0.0], [size, size], rotation);
        match vision {
            Some(vision) => world.spawn((
                transform,
                Agent { species },
                vision,
                VisionOutput::empty(&vision),
            )),
            None => world.spawn((transform, Agent { species })),
        }
    }

    fn forward_vision() -> Vision {
        Vision::new(10.0, 0.0, 0.5)
    }

    #[test]
    fn ray_count_handles_exact_and_non_exact_intervals() {
        let exact = Vision::new(10.0, 1.0, 0.25);
        let non_exact = Vision::new(10.0, 1.0, 0.3);

        assert_eq!(exact.ray_count(), 5);
        assert_eq!(non_exact.ray_count(), 4);
        assert_eq!(exact.ray_angle(0), -0.5);
        assert_eq!(exact.ray_angle(4), 0.5);
    }

    #[test]
    fn detects_target_in_front_and_encodes_input() {
        let mut world = World::new();
        let vision = forward_vision();
        let viewer = spawn_agent(
            &mut world,
            [0.0, 0.0],
            0.0,
            1.0,
            Species::Prey,
            Some(vision),
        );
        let target = spawn_agent(&mut world, [5.0, 0.0], 0.0, 2.0, Species::Predator, None);

        collect_vision(&mut world);

        let output = world.get::<&VisionOutput>(viewer).unwrap();
        assert_eq!(output.hits[0].unwrap().target, target);
        assert_eq!(output.hits[0].unwrap().distance, 4.0);
        assert_eq!(output.inputs(&vision), vec![0.4, 0.0, 1.0]);
    }

    #[test]
    fn excludes_targets_outside_distance_angle_and_behind_viewer() {
        let mut world = World::new();
        let vision = Vision::new(5.0, std::f32::consts::FRAC_PI_2, 0.25);
        let viewer = spawn_agent(
            &mut world,
            [0.0, 0.0],
            0.0,
            0.5,
            Species::Prey,
            Some(vision),
        );
        spawn_agent(&mut world, [8.0, 0.0], 0.0, 0.5, Species::Predator, None);
        spawn_agent(&mut world, [0.0, 3.0], 0.0, 0.5, Species::Predator, None);
        spawn_agent(&mut world, [-2.0, 0.0], 0.0, 0.5, Species::Predator, None);

        collect_vision(&mut world);

        let output = world.get::<&VisionOutput>(viewer).unwrap();
        assert!(output.hits.iter().all(Option::is_none));
        assert_eq!(output.inputs(&vision).len(), vision.input_len());
    }

    #[test]
    fn keeps_nearest_target_and_excludes_self() {
        let mut world = World::new();
        let vision = forward_vision();
        let viewer = spawn_agent(
            &mut world,
            [0.0, 0.0],
            0.0,
            1.0,
            Species::Predator,
            Some(vision),
        );
        let nearest = spawn_agent(&mut world, [3.0, 0.0], 0.0, 1.0, Species::Prey, None);
        spawn_agent(&mut world, [6.0, 0.0], 0.0, 1.0, Species::Predator, None);

        collect_vision(&mut world);

        let output = world.get::<&VisionOutput>(viewer).unwrap();
        assert_eq!(output.hits[0].unwrap().target, nearest);
        assert_eq!(output.inputs(&vision), vec![0.25, 1.0, 0.0]);
    }

    #[test]
    fn does_not_see_across_world_wrap_in_v1() {
        let mut world = World::new();
        let vision = Vision::new(3.0, 0.0, 0.5);
        let viewer = spawn_agent(
            &mut world,
            [19.0, 0.0],
            0.0,
            0.5,
            Species::Prey,
            Some(vision),
        );
        spawn_agent(&mut world, [-19.0, 0.0], 0.0, 0.5, Species::Predator, None);

        collect_vision(&mut world);

        let output = world.get::<&VisionOutput>(viewer).unwrap();
        assert_eq!(output.inputs(&vision), vec![1.0, 0.0, 0.0]);
    }
}

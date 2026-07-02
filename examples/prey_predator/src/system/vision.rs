use std::collections::HashMap;

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

struct SpatialGrid {
    cell_size: f32,
    max_target_radius: f32,
    cells: HashMap<(i32, i32), Vec<usize>>,
}

impl SpatialGrid {
    fn new(snapshots: &[AgentSnapshot], max_vision_distance: f32) -> Self {
        let max_target_radius = snapshots
            .iter()
            .map(|snapshot| snapshot.radius)
            .fold(0.0, f32::max);
        let cell_size = (max_vision_distance * 0.25)
            .max(max_target_radius * 2.0)
            .max(f32::EPSILON);
        let mut cells: HashMap<(i32, i32), Vec<usize>> = HashMap::new();
        for (index, snapshot) in snapshots.iter().enumerate() {
            cells
                .entry(Self::cell(snapshot.position, cell_size))
                .or_default()
                .push(index);
        }
        Self {
            cell_size,
            max_target_radius,
            cells,
        }
    }

    fn cell(position: [f32; 2], cell_size: f32) -> (i32, i32) {
        (
            (position[0] / cell_size).floor() as i32,
            (position[1] / cell_size).floor() as i32,
        )
    }

    fn search_radius(&self, vision_distance: f32) -> i32 {
        ((vision_distance + self.max_target_radius) / self.cell_size).ceil() as i32
    }

    fn collect_candidates(
        &self,
        viewer: AgentSnapshot,
        vision_distance: f32,
        snapshots: &[AgentSnapshot],
        candidates: &mut Vec<usize>,
    ) {
        candidates.clear();
        let viewer_cell = Self::cell(viewer.position, self.cell_size);
        let search_radius = self.search_radius(vision_distance);

        for cell_y in viewer_cell.1 - search_radius..=viewer_cell.1 + search_radius {
            for cell_x in viewer_cell.0 - search_radius..=viewer_cell.0 + search_radius {
                let Some(indices) = self.cells.get(&(cell_x, cell_y)) else {
                    continue;
                };
                for target_index in indices {
                    let target = snapshots[*target_index];
                    if target.entity == viewer.entity {
                        continue;
                    }
                    let delta = [
                        target.position[0] - viewer.position[0],
                        target.position[1] - viewer.position[1],
                    ];
                    let maximum_distance = vision_distance + target.radius;
                    if delta[0] * delta[0] + delta[1] * delta[1]
                        <= maximum_distance * maximum_distance
                    {
                        candidates.push(*target_index);
                    }
                }
            }
        }
    }
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
    if snapshots.is_empty() {
        return;
    }

    let viewers: Vec<(Entity, Vision)> = world
        .query::<&Vision>()
        .iter()
        .map(|(entity, vision)| (entity, *vision))
        .collect();
    let max_vision_distance = viewers
        .iter()
        .map(|(_entity, vision)| vision.max_distance)
        .fold(0.0, f32::max);
    let grid = SpatialGrid::new(&snapshots, max_vision_distance);
    let snapshot_indices: HashMap<Entity, usize> = snapshots
        .iter()
        .enumerate()
        .map(|(index, snapshot)| (snapshot.entity, index))
        .collect();
    let mut candidates = Vec::new();

    for (entity, vision) in viewers {
        let Some(viewer_index) = snapshot_indices.get(&entity).copied() else {
            continue;
        };
        grid.collect_candidates(
            snapshots[viewer_index],
            vision.max_distance,
            &snapshots,
            &mut candidates,
        );
        let has_output = world.get::<&VisionOutput>(entity).is_ok();

        if has_output {
            let mut current = world
                .get::<&mut VisionOutput>(entity)
                .expect("vision output disappeared while collecting vision");
            cast_vision(
                snapshots[viewer_index],
                vision,
                &snapshots,
                &candidates,
                &mut current,
            );
        } else {
            let mut output = VisionOutput::empty(&vision);
            cast_vision(
                snapshots[viewer_index],
                vision,
                &snapshots,
                &candidates,
                &mut output,
            );
            world
                .insert_one(entity, output)
                .expect("vision viewer disappeared while collecting vision");
        }
    }
}

fn cast_vision(
    viewer: AgentSnapshot,
    vision: Vision,
    targets: &[AgentSnapshot],
    candidates: &[usize],
    output: &mut VisionOutput,
) {
    output.hits.clear();
    output.hits.resize(vision.ray_count(), None);

    for ray_index in 0..vision.ray_count() {
        let angle = viewer.rotation + vision.ray_angle(ray_index);
        let direction = [angle.cos(), angle.sin()];
        let mut nearest: Option<VisionHit> = None;

        for target_index in candidates {
            let target = targets[*target_index];
            let Some(distance) = ray_circle_distance(
                viewer.position,
                direction,
                target.position,
                target.radius,
                vision.max_distance,
            ) else {
                continue;
            };
            if nearest.as_ref().is_some_and(|hit| hit.distance <= distance) {
                continue;
            }
            nearest = Some(VisionHit {
                target: target.entity,
                species: target.species,
                distance,
            });
        }
        output.hits[ray_index] = nearest;
    }
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
    fn spatial_grid_detects_targets_across_cell_boundaries() {
        let mut world = World::new();
        let vision = Vision::new(4.0, 0.0, 0.5);
        let viewer = spawn_agent(
            &mut world,
            [0.9, 0.0],
            0.0,
            1.0,
            Species::Prey,
            Some(vision),
        );
        let target = spawn_agent(&mut world, [1.1, 0.0], 0.0, 1.0, Species::Predator, None);

        collect_vision(&mut world);

        let output = world.get::<&VisionOutput>(viewer).unwrap();
        assert_eq!(output.hits[0].unwrap().target, target);
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

    #[test]
    fn dense_population_collects_vision_for_every_agent() {
        let mut world = World::new();
        let vision = Vision::new(7.0, std::f32::consts::FRAC_PI_2, 0.2);
        for index in 0..1_024 {
            let x = (index % 32) as f32 * 1.25 - 19.375;
            let y = (index / 32) as f32 * 1.25 - 19.375;
            spawn_agent(&mut world, [x, y], 0.0, 0.5, Species::Prey, Some(vision));
        }

        collect_vision(&mut world);

        assert_eq!(world.query::<&VisionOutput>().iter().count(), 1_024);
    }
}

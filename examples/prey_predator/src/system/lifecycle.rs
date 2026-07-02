use std::collections::{HashMap, HashSet};

use hecs::{Entity, World};
use pixie::Transform;

use crate::components::{Agent, LifeCycle, Reproduction, Species};
use crate::resources::{
    DeathQueue, DeathReason, DeathRequest, SimulationConfig, SpawnQueue, SpawnRequest,
};

#[derive(Debug, Clone, Copy)]
struct AgentPosition {
    entity: Entity,
    position: [f32; 2],
}

pub fn process_predation(world: &mut World, config: &SimulationConfig, deaths: &mut DeathQueue) {
    let mut prey = Vec::new();
    let mut predators = Vec::new();
    for (entity, (transform, agent)) in world.query::<(&Transform, &Agent)>().iter() {
        let position = [transform.position[0], transform.position[1]];
        match agent.species {
            Species::Prey => prey.push(AgentPosition { entity, position }),
            Species::Predator => predators.push(AgentPosition { entity, position }),
        }
    }
    predators.retain(|predator| {
        world
            .get::<&LifeCycle>(predator.entity)
            .is_ok_and(|lifecycle| lifecycle.time_since_food >= config.predator_min_feed_interval)
    });

    let cell_columns = (config.world_width / config.predation_radius)
        .floor()
        .max(1.0) as i32;
    let cell_rows = (config.world_height / config.predation_radius)
        .floor()
        .max(1.0) as i32;
    let cell_width = config.world_width / cell_columns as f32;
    let cell_height = config.world_height / cell_rows as f32;
    let mut prey_cells: HashMap<(i32, i32), Vec<AgentPosition>> = HashMap::new();
    for candidate in prey {
        prey_cells
            .entry(torus_cell(
                candidate.position,
                config.world_width,
                config.world_height,
                cell_columns,
                cell_rows,
            ))
            .or_default()
            .push(candidate);
    }

    let mut eaten = HashSet::new();
    let radius_squared = config.predation_radius * config.predation_radius;
    for predator in predators {
        let predator_cell = torus_cell(
            predator.position,
            config.world_width,
            config.world_height,
            cell_columns,
            cell_rows,
        );
        let search_columns = (config.predation_radius / cell_width).ceil() as i32;
        let search_rows = (config.predation_radius / cell_height).ceil() as i32;
        let mut nearest: Option<(Entity, f32)> = None;

        for cell_y in predator_cell.1 - search_rows..=predator_cell.1 + search_rows {
            for cell_x in predator_cell.0 - search_columns..=predator_cell.0 + search_columns {
                let cell = (
                    cell_x.rem_euclid(cell_columns),
                    cell_y.rem_euclid(cell_rows),
                );
                let Some(candidates) = prey_cells.get(&cell) else {
                    continue;
                };
                for candidate in candidates {
                    if eaten.contains(&candidate.entity) {
                        continue;
                    }
                    let distance_squared = torus_distance_squared(
                        predator.position,
                        candidate.position,
                        config.world_width,
                        config.world_height,
                    );
                    if distance_squared > radius_squared
                        || nearest
                            .as_ref()
                            .is_some_and(|current| current.1 <= distance_squared)
                    {
                        continue;
                    }
                    nearest = Some((candidate.entity, distance_squared));
                }
            }
        }

        let Some((prey_entity, _distance_squared)) = nearest else {
            continue;
        };
        eaten.insert(prey_entity);
        deaths.push(DeathRequest {
            entity: prey_entity,
            species: Species::Prey,
            reason: DeathReason::Eaten,
        });

        if let Ok(mut lifecycle) = world.get::<&mut LifeCycle>(predator.entity) {
            lifecycle.food_eaten += 1;
            lifecycle.time_since_food = 0.0;
        }
    }
}

fn torus_cell(
    position: [f32; 2],
    world_width: f32,
    world_height: f32,
    columns: i32,
    rows: i32,
) -> (i32, i32) {
    let normalized_x = (position[0] + world_width * 0.5) / world_width;
    let normalized_y = (position[1] + world_height * 0.5) / world_height;
    (
        (normalized_x * columns as f32)
            .floor()
            .clamp(0.0, (columns - 1) as f32) as i32,
        (normalized_y * rows as f32)
            .floor()
            .clamp(0.0, (rows - 1) as f32) as i32,
    )
}

pub fn update_lifecycles(
    world: &mut World,
    config: &SimulationConfig,
    fixed_dt: f32,
    deaths: &mut DeathQueue,
) {
    for (entity, (agent, lifecycle, reproduction)) in world
        .query::<(&Agent, &mut LifeCycle, &mut Reproduction)>()
        .iter()
    {
        if deaths.contains(entity) {
            continue;
        }

        lifecycle.age += fixed_dt;
        reproduction.cooldown_remaining = (reproduction.cooldown_remaining - fixed_dt).max(0.0);

        match agent.species {
            Species::Prey if lifecycle.age >= config.prey_max_age => {
                deaths.push(DeathRequest {
                    entity,
                    species: Species::Prey,
                    reason: DeathReason::Age,
                });
            }
            Species::Predator => {
                lifecycle.time_since_food += fixed_dt;
                let reason = if lifecycle.age >= config.predator_max_age {
                    Some(DeathReason::Age)
                } else if lifecycle.time_since_food >= config.predator_starvation_time {
                    Some(DeathReason::Starvation)
                } else {
                    None
                };
                if let Some(reason) = reason {
                    deaths.push(DeathRequest {
                        entity,
                        species: Species::Predator,
                        reason,
                    });
                }
            }
            Species::Prey => {}
        }
    }
}

pub fn process_reproduction(
    world: &mut World,
    config: &SimulationConfig,
    deaths: &DeathQueue,
    spawns: &mut SpawnQueue,
) {
    let mut prey_count = 0;
    let mut predator_count = 0;
    for (entity, agent) in world.query::<&Agent>().iter() {
        if deaths.contains(entity) {
            continue;
        }
        match agent.species {
            Species::Prey => prey_count += 1,
            Species::Predator => predator_count += 1,
        }
    }

    for (entity, (transform, agent, lifecycle, reproduction)) in world
        .query::<(&Transform, &Agent, &mut LifeCycle, &mut Reproduction)>()
        .iter()
    {
        if deaths.contains(entity) {
            continue;
        }

        let can_reproduce = match agent.species {
            Species::Prey => {
                lifecycle.age >= config.prey_reproduction_age
                    && reproduction.cooldown_remaining <= 0.0
                    && prey_count < config.max_prey
            }
            Species::Predator => {
                lifecycle.food_eaten >= config.predator_food_to_reproduce
                    && reproduction.cooldown_remaining <= 0.0
                    && predator_count < config.max_predators
            }
        };
        if !can_reproduce {
            continue;
        }

        spawns.requests.push(SpawnRequest {
            parent: entity,
            species: agent.species,
            parent_position: [transform.position[0], transform.position[1]],
        });
        match agent.species {
            Species::Prey => {
                prey_count += 1;
                reproduction.cooldown_remaining = config.prey_reproduction_cooldown;
            }
            Species::Predator => {
                predator_count += 1;
                lifecycle.food_eaten -= config.predator_food_to_reproduce;
                reproduction.cooldown_remaining = config.predator_reproduction_cooldown;
            }
        }
    }
}

pub fn torus_distance_squared(
    left: [f32; 2],
    right: [f32; 2],
    world_width: f32,
    world_height: f32,
) -> f32 {
    let mut delta_x = (left[0] - right[0]).abs();
    let mut delta_y = (left[1] - right[1]).abs();
    if delta_x > world_width * 0.5 {
        delta_x = world_width - delta_x;
    }
    if delta_y > world_height * 0.5 {
        delta_y = world_height - delta_y;
    }
    delta_x * delta_x + delta_y * delta_y
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spawn_agent(
        world: &mut World,
        species: Species,
        position: [f32; 2],
        lifecycle: LifeCycle,
        cooldown: f32,
    ) -> Entity {
        world.spawn((
            Transform::new([position[0], position[1], 0.0], [1.0, 1.0]),
            Agent { species },
            lifecycle,
            Reproduction::new(cooldown),
        ))
    }

    #[test]
    fn predator_eats_nearest_prey_and_updates_food_state() {
        let mut world = World::new();
        let predator = spawn_agent(
            &mut world,
            Species::Predator,
            [0.0, 0.0],
            LifeCycle {
                time_since_food: 5.0,
                ..LifeCycle::default()
            },
            0.0,
        );
        let nearest = spawn_agent(
            &mut world,
            Species::Prey,
            [0.2, 0.0],
            LifeCycle::default(),
            0.0,
        );
        let farther = spawn_agent(
            &mut world,
            Species::Prey,
            [0.5, 0.0],
            LifeCycle::default(),
            0.0,
        );
        let config = SimulationConfig {
            predation_radius: 1.0,
            ..SimulationConfig::default()
        };
        let mut deaths = DeathQueue::default();

        process_predation(&mut world, &config, &mut deaths);

        assert_eq!(deaths.requests.len(), 1);
        assert_eq!(deaths.requests[0].entity, nearest);
        assert_ne!(deaths.requests[0].entity, farther);
        let lifecycle = world.get::<&LifeCycle>(predator).unwrap();
        assert_eq!(lifecycle.food_eaten, 1);
        assert_eq!(lifecycle.time_since_food, 0.0);
    }

    #[test]
    fn predation_uses_shortest_torus_distance() {
        let mut world = World::new();
        spawn_agent(
            &mut world,
            Species::Predator,
            [19.8, 0.0],
            LifeCycle {
                time_since_food: 1.0,
                ..LifeCycle::default()
            },
            0.0,
        );
        let prey = spawn_agent(
            &mut world,
            Species::Prey,
            [-19.8, 0.0],
            LifeCycle::default(),
            0.0,
        );
        let config = SimulationConfig {
            world_width: 40.0,
            predation_radius: 0.5,
            ..SimulationConfig::default()
        };
        let mut deaths = DeathQueue::default();

        process_predation(&mut world, &config, &mut deaths);

        assert_eq!(deaths.requests[0].entity, prey);
    }

    #[test]
    fn predator_waits_for_minimum_feed_interval() {
        let mut world = World::new();
        let predator = spawn_agent(
            &mut world,
            Species::Predator,
            [0.0, 0.0],
            LifeCycle {
                time_since_food: 0.5,
                ..LifeCycle::default()
            },
            0.0,
        );
        let prey = spawn_agent(
            &mut world,
            Species::Prey,
            [0.2, 0.0],
            LifeCycle::default(),
            0.0,
        );
        let config = SimulationConfig {
            predation_radius: 1.0,
            predator_min_feed_interval: 0.75,
            ..SimulationConfig::default()
        };
        let mut deaths = DeathQueue::default();

        process_predation(&mut world, &config, &mut deaths);
        assert!(deaths.requests.is_empty());

        world
            .get::<&mut LifeCycle>(predator)
            .unwrap()
            .time_since_food = 0.75;
        process_predation(&mut world, &config, &mut deaths);

        assert_eq!(deaths.requests.len(), 1);
        assert_eq!(deaths.requests[0].entity, prey);
        let lifecycle = world.get::<&LifeCycle>(predator).unwrap();
        assert_eq!(lifecycle.food_eaten, 1);
        assert_eq!(lifecycle.time_since_food, 0.0);
    }

    #[test]
    fn lifecycle_marks_old_prey_and_starving_predator_for_death() {
        let mut world = World::new();
        let prey = spawn_agent(
            &mut world,
            Species::Prey,
            [0.0, 0.0],
            LifeCycle {
                age: 9.5,
                ..LifeCycle::default()
            },
            1.0,
        );
        let predator = spawn_agent(
            &mut world,
            Species::Predator,
            [2.0, 0.0],
            LifeCycle {
                time_since_food: 4.5,
                ..LifeCycle::default()
            },
            1.0,
        );
        let config = SimulationConfig {
            prey_max_age: 10.0,
            predator_max_age: 20.0,
            predator_starvation_time: 5.0,
            ..SimulationConfig::default()
        };
        let mut deaths = DeathQueue::default();

        update_lifecycles(&mut world, &config, 0.5, &mut deaths);

        assert!(deaths
            .requests
            .iter()
            .any(|request| request.entity == prey && request.reason == DeathReason::Age));
        assert!(deaths.requests.iter().any(|request| {
            request.entity == predator && request.reason == DeathReason::Starvation
        }));
    }

    #[test]
    fn prey_reproduction_respects_age_cooldown_and_population_cap() {
        let mut world = World::new();
        let parent = spawn_agent(
            &mut world,
            Species::Prey,
            [1.0, 2.0],
            LifeCycle {
                age: 8.0,
                ..LifeCycle::default()
            },
            0.0,
        );
        let mut config = SimulationConfig {
            prey_reproduction_age: 8.0,
            prey_reproduction_cooldown: 5.0,
            max_prey: 2,
            ..SimulationConfig::default()
        };
        let deaths = DeathQueue::default();
        let mut spawns = SpawnQueue::default();

        process_reproduction(&mut world, &config, &deaths, &mut spawns);

        assert_eq!(
            spawns.requests,
            vec![SpawnRequest {
                parent,
                species: Species::Prey,
                parent_position: [1.0, 2.0],
            }]
        );
        assert_eq!(
            world
                .get::<&Reproduction>(parent)
                .unwrap()
                .cooldown_remaining,
            5.0
        );

        spawns.requests.clear();
        world
            .get::<&mut Reproduction>(parent)
            .unwrap()
            .cooldown_remaining = 0.0;
        config.max_prey = 1;
        process_reproduction(&mut world, &config, &deaths, &mut spawns);
        assert!(spawns.requests.is_empty());
    }

    #[test]
    fn predator_reproduction_consumes_food_and_starts_cooldown() {
        let mut world = World::new();
        let parent = spawn_agent(
            &mut world,
            Species::Predator,
            [3.0, 4.0],
            LifeCycle {
                food_eaten: 3,
                ..LifeCycle::default()
            },
            0.0,
        );
        let config = SimulationConfig {
            predator_food_to_reproduce: 3,
            predator_reproduction_cooldown: 8.0,
            max_predators: 2,
            ..SimulationConfig::default()
        };
        let deaths = DeathQueue::default();
        let mut spawns = SpawnQueue::default();

        process_reproduction(&mut world, &config, &deaths, &mut spawns);

        assert_eq!(spawns.requests.len(), 1);
        assert_eq!(world.get::<&LifeCycle>(parent).unwrap().food_eaten, 0);
        assert_eq!(
            world
                .get::<&Reproduction>(parent)
                .unwrap()
                .cooldown_remaining,
            8.0
        );
    }
}

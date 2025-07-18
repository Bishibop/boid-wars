use bevy::prelude::*;
use boid_wars_shared::*;
use crate::spatial_grid::SpatialGrid;
use crate::flocking::FlockingConfig;
use crate::groups::{GroupLOD, LODLevel, BoidGroupConfig, calculate_formation_positions};
use std::collections::HashMap;

/// Plugin for group movement systems
pub struct GroupMovementPlugin;

impl Plugin for GroupMovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                group_movement_system,
                formation_flocking_system.after(group_movement_system),
                sync_group_boid_velocities.after(formation_flocking_system),
            ),
        );
    }
}

/// High-level group movement decisions
fn group_movement_system(
    mut groups: Query<(&mut BoidGroup, &Position, &mut GroupVelocity, &GroupLOD)>,
    players: Query<&Position, With<Player>>,
    time: Res<Time>,
    config: Res<BoidGroupConfig>,
) {
    for (mut group, pos, mut velocity, lod) in groups.iter_mut() {
        // Skip update based on LOD
        if !should_update_group(lod, &time) {
            continue;
        }
        
        // Calculate target velocity based on behavior state
        let target_velocity = match &mut group.behavior_state {
            GroupBehavior::Patrolling { route: _, current_waypoint: _ } => {
                // For now, let groups stay relatively still when patrolling
                // This allows individual flocking to be the primary movement
                Vec2::ZERO
            },
            GroupBehavior::Engaging { primary_target, .. } => {
                // Find target player by ID
                if let Some(target_pos) = players.iter()
                    .find(|p| {
                        // In a real implementation, we'd match player ID
                        true // For now, engage first player
                    })
                    .map(|p| p.0) {
                    
                    let direction = (target_pos - pos.0).normalize_or_zero();
                    let preferred_range = match group.archetype {
                        GroupArchetype::Assault { preferred_range, .. } => preferred_range,
                        _ => 200.0,
                    };
                    
                    let distance = pos.0.distance(target_pos);
                    if distance > preferred_range {
                        direction * 150.0
                    } else {
                        // Orbit at preferred range
                        let tangent = Vec2::new(-direction.y, direction.x);
                        tangent * 100.0
                    }
                } else {
                    // Lost target, return to patrol
                    group.behavior_state = GroupBehavior::Patrolling {
                        route: group.home_territory.patrol_points.clone(),
                        current_waypoint: 0,
                    };
                    Vec2::ZERO
                }
            },
            GroupBehavior::Retreating { rally_point, speed_multiplier } => {
                let direction = (*rally_point - pos.0).normalize_or_zero();
                direction * 200.0 * *speed_multiplier
            },
            GroupBehavior::Defending { position, .. } => {
                let direction = (*position - pos.0).normalize_or_zero();
                let distance = pos.0.distance(*position);
                
                if distance > 20.0 {
                    direction * 100.0
                } else {
                    Vec2::ZERO
                }
            },
        };
        
        // Smooth velocity changes
        velocity.0 = velocity.0.lerp(target_velocity, time.delta_secs() * 2.0);
    }
}

/// Individual boid movement within group constraints
fn formation_flocking_system(
    mut boids: Query<(Entity, &BoidGroupMember, &Position, &mut Velocity), With<Boid>>,
    groups: Query<(&BoidGroup, &Position, &GroupVelocity)>,
    obstacle_query: Query<(&Position, &boid_wars_shared::Obstacle), Without<Boid>>,
    player_query: Query<(&Position, &Velocity), (With<boid_wars_shared::Player>, Without<Boid>)>,
    spatial_grid: Res<SpatialGrid>,
    config: Res<FlockingConfig>,
    group_config: Res<BoidGroupConfig>,
    time: Res<Time>,
) {
    use std::collections::HashMap;
    
    // Collect boid data first to avoid borrow issues - INCLUDING VELOCITIES
    let mut boid_data: Vec<(Entity, Entity, u32, Vec2, Vec2, Option<FormationSlot>)> = Vec::new();
    for (entity, member, pos, vel) in boids.iter() {
        boid_data.push((
            entity,
            member.group_entity,
            member.group_id,
            pos.0,
            vel.0, // Add velocity to boid data
            member.formation_slot,
        ));
    }
    
    // Group boids by their group entity
    let mut groups_map: HashMap<Entity, Vec<(Entity, Vec2, Option<FormationSlot>)>> = HashMap::new();
    for (entity, group_entity, _, pos, vel, slot) in &boid_data {
        groups_map.entry(*group_entity)
            .or_insert_with(Vec::new)
            .push((*entity, *pos, *slot));
    }
    
    // Process each group
    for (group_entity, members) in groups_map {
        if let Ok((group, group_pos, group_vel)) = groups.get(group_entity) {
            // Skip formation calculations - just update velocities
            
            // BYPASS GROUP MOVEMENT - Use pure individual flocking
            // Keep the group structure but ignore group velocity entirely
            for (_i, (entity, pos, _slot)) in members.iter().enumerate() {
                if let Ok((_, _, position, mut vel)) = boids.get_mut(*entity) {
                    // Get search radius including avoidance radii
                    let search_radius = config.separation_radius
                        .max(config.alignment_radius)
                        .max(config.cohesion_radius)
                        .max(config.obstacle_avoidance_radius)
                        .max(config.player_avoidance_radius);
                    
                    // Get nearby entities
                    let neighbors = spatial_grid.get_nearby_entities(*pos, search_radius);
                    
                    // Calculate full flocking forces
                    let flocking_force = calculate_full_flocking_forces(
                        *entity,
                        position,
                        &vel,
                        &neighbors,
                        &boid_data,
                        &obstacle_query,
                        &player_query,
                        &config,
                        time.delta_secs(),
                    );
                    
                    // IGNORE GROUP VELOCITY - Use only individual flocking
                    vel.0 = flocking_force.clamp_length_max(config.max_speed);
                }
            }
        }
    }
}

/// Calculate full flocking forces including avoidance
fn calculate_full_flocking_forces(
    entity: Entity,
    position: &Position,
    velocity: &Velocity,
    neighbors: &[Entity],
    boid_data: &[(Entity, Entity, u32, Vec2, Vec2, Option<FormationSlot>)],
    obstacle_query: &Query<(&Position, &boid_wars_shared::Obstacle), Without<Boid>>,
    player_query: &Query<(&Position, &Velocity), (With<boid_wars_shared::Player>, Without<Boid>)>,
    config: &FlockingConfig,
    delta_time: f32,
) -> Vec2 {
    let game_config = &*boid_wars_shared::GAME_CONFIG;
    
    // Basic flocking forces
    let mut separation = Vec2::ZERO;
    let mut alignment = Vec2::ZERO;
    let mut cohesion = Vec2::ZERO;
    let mut sep_count = 0;
    let mut align_count = 0;
    let mut cohesion_count = 0;
    
    // Calculate flocking forces with other boids
    for &other_entity in neighbors {
        if other_entity == entity {
            continue;
        }
        
        // Find in boid data
        if let Some((_, _, _, other_pos, other_vel, _)) = boid_data.iter().find(|(e, _, _, _, _, _)| *e == other_entity) {
            let diff = position.0 - *other_pos;
            let distance = diff.length();
            
            // Separation
            if distance > 0.0 && distance < config.separation_radius {
                let force = diff.normalize() / distance;
                separation += force;
                sep_count += 1;
            }
            
            // Alignment - use actual velocity
            if distance < config.alignment_radius {
                alignment += *other_vel;
                align_count += 1;
            }
            
            // Cohesion
            if distance < config.cohesion_radius {
                cohesion += *other_pos;
                cohesion_count += 1;
            }
        }
    }
    
    let mut total_force = Vec2::ZERO;
    
    // Apply separation
    if sep_count > 0 {
        separation = (separation / sep_count as f32).normalize_or_zero() * config.max_speed;
        total_force += separation * config.separation_weight;
    }
    
    // Apply alignment
    if align_count > 0 {
        alignment = (alignment / align_count as f32).normalize_or_zero() * config.max_speed;
        total_force += alignment * config.alignment_weight;
    }
    
    // Apply cohesion
    if cohesion_count > 0 {
        let center = cohesion / cohesion_count as f32;
        let desired = (center - position.0).normalize_or_zero() * config.max_speed;
        total_force += desired * config.cohesion_weight;
    }
    
    // Obstacle avoidance
    let obstacle_force = calculate_obstacle_avoidance(
        position,
        velocity,
        obstacle_query,
        config,
        delta_time,
    );
    total_force += obstacle_force;
    
    // Player avoidance
    let player_force = calculate_player_avoidance(
        position,
        velocity,
        player_query,
        config,
        delta_time,
    );
    total_force += player_force;
    
    // Wall avoidance
    let wall_force = calculate_wall_avoidance(
        position,
        velocity,
        config,
        game_config,
        delta_time,
    );
    total_force += wall_force;
    
    total_force
}

/// Calculate obstacle avoidance force
fn calculate_obstacle_avoidance(
    position: &Position,
    velocity: &Velocity,
    obstacle_query: &Query<(&Position, &boid_wars_shared::Obstacle), Without<Boid>>,
    config: &FlockingConfig,
    delta_time: f32,
) -> Vec2 {
    let mut avoidance_force = Vec2::ZERO;
    
    // Predict future position
    let future_pos = position.0 + velocity.0 * config.obstacle_prediction_time;
    
    for (obstacle_pos, obstacle) in obstacle_query.iter() {
        let diff = position.0 - obstacle_pos.0;
        let distance = diff.length();
        
        // Check if obstacle is within avoidance range
        if distance < config.obstacle_avoidance_radius {
            // Calculate obstacle bounds
            let half_width = obstacle.width / 2.0;
            let half_height = obstacle.height / 2.0;
            
            // Check if future position would collide
            let future_diff = future_pos - obstacle_pos.0;
            let will_collide = future_diff.x.abs() < half_width + config.obstacle_danger_zone &&
                               future_diff.y.abs() < half_height + config.obstacle_danger_zone;
            
            if will_collide || distance < config.obstacle_danger_zone {
                // Calculate avoidance direction
                let avoidance_direction = if diff.length() > 0.0 {
                    diff.normalize()
                } else {
                    // If directly on obstacle, pick a random direction
                    Vec2::new(1.0, 0.0)
                };
                
                // Stronger avoidance when closer
                let force_magnitude = config.obstacle_avoidance_weight * (1.0 - (distance / config.obstacle_avoidance_radius));
                avoidance_force += avoidance_direction * force_magnitude * config.max_speed;
            }
        }
    }
    
    avoidance_force
}

/// Calculate player avoidance force
fn calculate_player_avoidance(
    position: &Position,
    velocity: &Velocity,
    player_query: &Query<(&Position, &Velocity), (With<boid_wars_shared::Player>, Without<Boid>)>,
    config: &FlockingConfig,
    delta_time: f32,
) -> Vec2 {
    let mut avoidance_force = Vec2::ZERO;
    
    for (player_pos, player_vel) in player_query.iter() {
        let diff = position.0 - player_pos.0;
        let distance = diff.length();
        
        if distance < config.player_avoidance_radius && distance > 0.0 {
            // Predict where player will be
            let future_player_pos = player_pos.0 + player_vel.0 * config.obstacle_prediction_time;
            let future_diff = position.0 - future_player_pos;
            
            // Avoidance direction
            let avoidance_direction = if diff.length() > 0.0 {
                diff.normalize()
            } else {
                Vec2::new(1.0, 0.0)
            };
            
            // Stronger avoidance when closer
            let force_magnitude = config.player_avoidance_weight * (1.0 - (distance / config.player_avoidance_radius));
            avoidance_force += avoidance_direction * force_magnitude * config.max_speed;
        }
    }
    
    avoidance_force
}

/// Calculate wall avoidance force
fn calculate_wall_avoidance(
    position: &Position,
    velocity: &Velocity,
    config: &FlockingConfig,
    game_config: &boid_wars_shared::GameConfig,
    delta_time: f32,
) -> Vec2 {
    let mut avoidance_force = Vec2::ZERO;
    
    // Check proximity to walls
    let margin = config.boundary_margin;
    let pos = position.0;
    
    // Left wall
    if pos.x < margin {
        let force = (margin - pos.x) * config.boundary_turn_force;
        avoidance_force.x += force;
    }
    
    // Right wall
    if pos.x > game_config.game_width - margin {
        let force = (pos.x - (game_config.game_width - margin)) * config.boundary_turn_force;
        avoidance_force.x -= force;
    }
    
    // Top wall
    if pos.y < margin {
        let force = (margin - pos.y) * config.boundary_turn_force;
        avoidance_force.y += force;
    }
    
    // Bottom wall
    if pos.y > game_config.game_height - margin {
        let force = (pos.y - (game_config.game_height - margin)) * config.boundary_turn_force;
        avoidance_force.y -= force;
    }
    
    avoidance_force * config.wall_avoidance_weight
}

/// Calculate local flocking forces for a boid (original version - not used)
fn _calculate_local_flocking(
    entity: Entity,
    pos: &Position,
    neighbors: &[Entity],
    boids: &Query<(Entity, &BoidGroupMember, &Position, &mut Velocity), With<Boid>>,
    config: &FlockingConfig,
) -> Vec2 {
    let mut separation = Vec2::ZERO;
    let mut alignment = Vec2::ZERO;
    let mut cohesion = Vec2::ZERO;
    let mut sep_count = 0;
    let mut align_count = 0;
    let mut cohesion_count = 0;
    
    for &other_entity in neighbors {
        if other_entity == entity {
            continue;
        }
        
        if let Ok((_, _, other_pos, other_vel)) = boids.get(other_entity) {
            let diff = pos.0 - other_pos.0;
            let distance = diff.length();
            
            // Separation
            if distance > 0.0 && distance < config.separation_radius {
                let force = diff.normalize() / distance;
                separation += force;
                sep_count += 1;
            }
            
            // Alignment
            if distance < config.alignment_radius {
                alignment += other_vel.0;
                align_count += 1;
            }
            
            // Cohesion
            if distance < config.cohesion_radius {
                cohesion += other_pos.0;
                cohesion_count += 1;
            }
        }
    }
    
    let mut total_force = Vec2::ZERO;
    
    // Apply separation
    if sep_count > 0 {
        separation = (separation / sep_count as f32).normalize_or_zero() * config.max_speed;
        total_force += separation * config.separation_weight;
    }
    
    // Apply alignment
    if align_count > 0 {
        alignment = (alignment / align_count as f32).normalize_or_zero() * config.max_speed;
        total_force += alignment * config.alignment_weight;
    }
    
    // Apply cohesion
    if cohesion_count > 0 {
        let center = cohesion / cohesion_count as f32;
        let desired = (center - pos.0).normalize_or_zero() * config.max_speed;
        total_force += desired * config.cohesion_weight;
    }
    
    total_force
}

/// Sync boid velocities to physics with smoothing
fn sync_group_boid_velocities(
    mut boids: Query<(&Velocity, &mut bevy_rapier2d::dynamics::Velocity), With<BoidGroupMember>>,
    time: Res<Time>,
) {
    let smooth_factor = 10.0; // Higher = faster response, lower = more smoothing
    let dt = time.delta_secs();
    
    for (network_vel, mut physics_vel) in boids.iter_mut() {
        // Smooth the velocity transition to prevent jittering
        physics_vel.linvel = physics_vel.linvel.lerp(network_vel.0, smooth_factor * dt);
        physics_vel.angvel = 0.0;
    }
}

/// Check if group should update based on LOD
fn should_update_group(lod: &GroupLOD, time: &Time) -> bool {
    let update_rate = match lod.level {
        LODLevel::Near => 0.016,     // Every frame (60Hz)
        LODLevel::Medium => 0.1,     // 10Hz
        LODLevel::Far => 0.2,        // 5Hz
        LODLevel::Distant => 1.0,    // 1Hz
    };
    
    time.elapsed_secs() - lod.last_update > update_rate
}
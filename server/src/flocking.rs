use crate::spatial_grid::SpatialGrid;
use bevy::prelude::*;
use boid_wars_shared::{
    Boid, BoidGroup, BoidGroupMember, BoidRole, GroupArchetype, GroupBehavior, Player, Position, Velocity,
};

/// Simplified configuration for flocking behavior
#[derive(Resource, Debug, Clone)]
pub struct FlockingConfig {
    // Detection radii
    pub separation_radius: f32,
    pub alignment_radius: f32,
    pub cohesion_radius: f32,

    // Force weights
    pub separation_weight: f32,
    pub alignment_weight: f32,
    pub cohesion_weight: f32,

    // Movement parameters
    pub max_speed: f32,
    pub max_force: f32,

    // Boundary behavior
    pub boundary_margin: f32,
    pub boundary_turn_force: f32,
    pub wall_avoidance_weight: f32,

    // Obstacle avoidance
    pub obstacle_avoidance_radius: f32,
    pub obstacle_avoidance_weight: f32,
    pub obstacle_prediction_time: f32,

    // Player avoidance
    pub player_avoidance_radius: f32,
    pub player_avoidance_weight: f32,

    // Inter-group separation
    pub inter_group_separation_radius: f32,
    pub inter_group_separation_weight: f32,

    // Avoidance thresholds and constants
    pub obstacle_danger_zone: f32,
    pub collision_threshold: f32,
    pub corner_boost_multiplier: f32,
    pub wall_prediction_time: f32,
    pub min_velocity_threshold: f32,
}

impl Default for FlockingConfig {
    fn default() -> Self {
        Self {
            // Detection radii
            separation_radius: 50.0,
            alignment_radius: 80.0,
            cohesion_radius: 100.0,

            // Force weights
            separation_weight: 1.5,
            alignment_weight: 1.0,
            cohesion_weight: 1.0,

            // Movement parameters - increased 50% more for even faster movement
            max_speed: 1050.0, // 50% increase from 700.0
            max_force: 1800.0, // 50% increase from 1200.0

            // Boundary behavior - increased for better wall avoidance
            boundary_margin: 80.0,      // Increased from 50.0
            boundary_turn_force: 3.0,   // Increased from 2.0
            wall_avoidance_weight: 5.0, // Balanced for flow vs safety

            // Obstacle avoidance - slightly reduced weights for better flow
            obstacle_avoidance_radius: 100.0, // Increased from 80.0
            obstacle_avoidance_weight: 4.0,   // Reduced from 5.0 for better flow
            obstacle_prediction_time: 0.6,    // Reduced from 0.8 for less early avoidance

            // Player avoidance
            player_avoidance_radius: 120.0, // Increased from 100.0
            player_avoidance_weight: 2.5,   // Reduced from 3.0 for better flow

            // Inter-group separation
            inter_group_separation_radius: 150.0, // Keep groups well separated
            inter_group_separation_weight: 2.0,   // Strong enough to prevent merging

            // Avoidance thresholds and constants
            obstacle_danger_zone: 40.0,
            collision_threshold: 30.0,
            corner_boost_multiplier: 1.5,
            wall_prediction_time: 1.0,
            min_velocity_threshold: 0.1,
        }
    }
}

/// Simple flocking system that updates boid velocities
#[allow(clippy::type_complexity)]
pub fn update_flocking(
    mut boids: Query<(Entity, &Position, &mut Velocity, Option<&BoidGroupMember>), With<Boid>>,
    obstacle_query: Query<(&Position, &boid_wars_shared::Obstacle), Without<Boid>>,
    player_query: Query<
        (&Position, &Velocity, &Player),
        (With<boid_wars_shared::Player>, Without<Boid>),
    >,
    group_query: Query<&BoidGroup>,
    spatial_grid: Res<SpatialGrid>,
    config: Res<FlockingConfig>,
    time: Res<Time>,
) {
    let game_config = &*boid_wars_shared::GAME_CONFIG;
    let delta = time.delta_secs();

    // Use largest radius for spatial query - include avoidance radii
    let search_radius = config
        .separation_radius
        .max(config.alignment_radius)
        .max(config.cohesion_radius)
        .max(config.obstacle_avoidance_radius)
        .max(config.player_avoidance_radius)
        .max(config.inter_group_separation_radius);

    // Collect all boid data first to avoid borrow checker issues
    // Now includes group_id for inter-group separation
    let boid_data: Vec<(Entity, Vec2, Vec2, Option<u32>)> = boids
        .iter()
        .map(|(entity, pos, vel, group_member)| {
            let group_id = group_member.as_ref().map(|m| m.group_id);
            (entity, pos.0, vel.0, group_id)
        })
        .collect();

    // Create a HashSet for O(1) boid lookups
    let boid_entities: std::collections::HashSet<Entity> =
        boid_data.iter().map(|(e, _, _, _)| *e).collect();

    // Update each boid
    for (entity, pos, mut vel, group_member) in boids.iter_mut() {
        let mut separation = Vec2::ZERO;
        let mut alignment = Vec2::ZERO;
        let mut cohesion = Vec2::ZERO;
        let mut inter_group_separation = Vec2::ZERO;
        let mut sep_count = 0;
        let mut align_count = 0;
        let mut cohesion_count = 0;
        let mut inter_group_count = 0;

        // Get this boid's group ID
        let my_group_id = group_member.as_ref().map(|m| m.group_id);

        // Get nearby entities from spatial grid
        let nearby = spatial_grid.get_nearby_entities(pos.0, search_radius);

        // Calculate flocking forces from neighbors
        for &other_entity in &nearby {
            if other_entity == entity {
                continue;
            }

            // Find the other boid's data in our collected data
            if let Some((_, other_pos, other_vel, other_group_id)) =
                boid_data.iter().find(|(e, _, _, _)| *e == other_entity)
            {
                let diff = pos.0 - *other_pos;
                let distance = diff.length();

                // Check if boids are in the same group
                let same_group = my_group_id.is_some()
                    && other_group_id.is_some()
                    && my_group_id == *other_group_id;

                // Separation: avoid crowding (applies to all nearby boids)
                if distance > 0.0 && distance < config.separation_radius {
                    let force = diff.normalize() / distance; // Inverse square law
                    separation += force;
                    sep_count += 1;
                }

                // Only apply alignment and cohesion to same-group boids
                if same_group {
                    // Alignment: match velocity
                    if distance < config.alignment_radius {
                        alignment += *other_vel;
                        align_count += 1;
                    }

                    // Cohesion: move toward center
                    if distance < config.cohesion_radius {
                        cohesion += *other_pos;
                        cohesion_count += 1;
                    }
                } else if my_group_id.is_some() && other_group_id.is_some() {
                    // Inter-group separation: push away from other groups
                    if distance > 0.0 && distance < config.inter_group_separation_radius {
                        let force = diff.normalize() / (distance * 0.5); // Stronger repulsion
                        inter_group_separation += force;
                        inter_group_count += 1;
                    }
                }
            }
        }

        // Calculate steering forces
        let mut acceleration = Vec2::ZERO;

        // Apply separation with archetype-specific preferences
        if sep_count > 0 {
            separation = (separation / sep_count as f32).normalize_or_zero() * config.max_speed;
            separation = (separation - vel.0).clamp_length_max(config.max_force);
            
            // Archetype-specific separation behavior
            let separation_multiplier = if let Some(member) = group_member {
                if let Ok(group) = group_query.get(member.group_entity) {
                    match &group.archetype {
                        GroupArchetype::Assault { .. } => 0.7, // Prefer tighter formations for aggressive attacks
                        GroupArchetype::Defensive { .. } => 1.3, // Want more personal space for defensive positioning
                        GroupArchetype::Recon { .. } => 1.1, // Slightly more spread for scouting flexibility
                    }
                } else {
                    1.0
                }
            } else {
                1.0
            };
            
            acceleration += separation * config.separation_weight * separation_multiplier;
        }

        // Apply alignment with archetype-specific coordination
        if align_count > 0 {
            alignment = (alignment / align_count as f32).normalize_or_zero() * config.max_speed;
            alignment = (alignment - vel.0).clamp_length_max(config.max_force);
            
            // Archetype-specific alignment behavior
            let alignment_multiplier = if let Some(member) = group_member {
                if let Ok(group) = group_query.get(member.group_entity) {
                    match &group.archetype {
                        GroupArchetype::Assault { .. } => 1.4, // Strong coordination for unified attacks
                        GroupArchetype::Defensive { .. } => 1.2, // Good coordination for defensive lines
                        GroupArchetype::Recon { .. } => 0.6, // More independent movement for scouting
                    }
                } else {
                    1.0
                }
            } else {
                1.0
            };
            
            acceleration += alignment * config.alignment_weight * alignment_multiplier;
        }

        // Apply cohesion with reduced weight for Defensive groups
        if cohesion_count > 0 {
            let center = cohesion / cohesion_count as f32;
            let desired = (center - pos.0).normalize_or_zero() * config.max_speed;
            cohesion = (desired - vel.0).clamp_length_max(config.max_force);
            
            // Reduce cohesion for Defensive groups to make them spread farther apart
            let cohesion_multiplier = if let Some(member) = group_member {
                if let Ok(group) = group_query.get(member.group_entity) {
                    match &group.archetype {
                        GroupArchetype::Defensive { .. } => 0.3, // Much weaker cohesion for spreading
                        _ => 1.0, // Normal cohesion for other groups
                    }
                } else {
                    1.0
                }
            } else {
                1.0
            };
            
            acceleration += cohesion * config.cohesion_weight * cohesion_multiplier;
        }

        // Apply inter-group separation
        if inter_group_count > 0 {
            inter_group_separation = (inter_group_separation / inter_group_count as f32)
                .normalize_or_zero()
                * config.max_speed;
            inter_group_separation =
                (inter_group_separation - vel.0).clamp_length_max(config.max_force);
            acceleration += inter_group_separation * config.inter_group_separation_weight;
        }

        // Early exit optimization: if only boids nearby, skip avoidance calculations
        let mut non_boid_count = 0;
        for &other_entity in &nearby {
            if other_entity != entity && !boid_entities.contains(&other_entity) {
                non_boid_count += 1;
                break; // Found at least one non-boid, proceed with avoidance
            }
        }

        let mut obstacle_force = Vec2::ZERO;
        let mut player_force = Vec2::ZERO;
        let mut obstacle_count = 0;
        let mut player_count = 0;

        // Only calculate avoidance if there are non-boid entities nearby
        if non_boid_count > 0 {
            for &other_entity in &nearby {
                // Skip self and other boids (handled by flocking)
                if other_entity == entity || boid_entities.contains(&other_entity) {
                    continue;
                }

                // Check for obstacles
                if let Ok((obs_pos, obs)) = obstacle_query.get(other_entity) {
                    let force = calculate_obstacle_avoidance(
                        pos.0,
                        vel.0,
                        obs_pos.0,
                        Vec2::new(obs.width / 2.0, obs.height / 2.0),
                        config.obstacle_prediction_time,
                        config.obstacle_danger_zone,
                    );
                    obstacle_force += force;
                    obstacle_count += 1;
                }

                // Check for players
                if let Ok((player_pos, player_vel, player)) = player_query.get(other_entity) {
                    let distance = pos.0.distance(player_pos.0);
                    if distance < config.player_avoidance_radius {
                        let force = calculate_dynamic_avoidance(
                            pos.0,
                            vel.0,
                            player_pos.0,
                            player_vel.0,
                            config.player_avoidance_radius,
                            config.collision_threshold,
                        );
                        player_force += force;
                        player_count += 1;
                    }
                }
            }
        }

        // Check for pursuit behavior if boid is in an engaging group
        let mut pursuit_force = Vec2::ZERO;
        let mut is_pursuing = false;
        let mut is_defensive_keeping_distance = false;

        if let Some(member) = group_member {
            if let Ok(group) = group_query.get(member.group_entity) {
                match &group.behavior_state {
                    GroupBehavior::Retreating { rally_point, speed_multiplier } => {
                        // Move toward rally point with enhanced speed
                        let direction = (*rally_point - pos.0).normalize_or_zero();
                        pursuit_force = direction * config.max_speed * speed_multiplier;
                        is_pursuing = true;
                    }
                    GroupBehavior::Engaging { primary_target, .. } => {
                        // Find the target player
                        for (target_pos, _, target_player) in player_query.iter() {
                            if target_player.id as u32 == *primary_target {
                                let distance_to_target = pos.0.distance(target_pos.0);
                                
                                match &group.archetype {
                                    GroupArchetype::Defensive { protection_radius, .. } => {
                                        // Defensive groups maintain distance and shoot from afar - much slower and spread out
                                        let preferred_distance = protection_radius * 0.8; // Stay at 80% of protection radius (farther)
                                        
                                        if distance_to_target < preferred_distance {
                                            // Too close - back away while maintaining sight (very slow)
                                            let direction = (pos.0 - target_pos.0).normalize_or_zero();
                                            pursuit_force = direction * config.max_speed * 0.4; // Much slower retreat
                                            is_defensive_keeping_distance = true;
                                        } else if distance_to_target > preferred_distance * 1.8 {
                                            // Too far - move closer very slowly
                                            let direction = (target_pos.0 - pos.0).normalize_or_zero();
                                            pursuit_force = direction * config.max_speed * 0.3; // Very slow approach
                                            is_pursuing = true;
                                        }
                                        // If within optimal range, don't move much (let flocking handle minor adjustments)
                                    }
                                    GroupArchetype::Assault { preferred_range, .. } => {
                                        // Assault groups: Direct aggressive pursuit with flanking
                                        if distance_to_target > *preferred_range {
                                            let mut direction = (target_pos.0 - pos.0).normalize_or_zero();
                                            
                                            // Add flanking behavior based on boid role
                                            if let Some(member) = group_member {
                                                match member.role_in_group {
                                                    BoidRole::Flanker => {
                                                        // Flankers approach from sides
                                                        let perpendicular = Vec2::new(-direction.y, direction.x);
                                                        let side_offset = if (pos.0.x + pos.0.y) % 2.0 > 1.0 { 1.0 } else { -1.0 };
                                                        direction += perpendicular * side_offset * 0.3;
                                                        direction = direction.normalize_or_zero();
                                                    }
                                                    BoidRole::Leader => {
                                                        // Leaders charge directly
                                                        pursuit_force = direction * config.max_speed * 1.4; // Extra aggressive
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            
                                            if pursuit_force.length_squared() == 0.0 {
                                                pursuit_force = direction * config.max_speed * 1.2; // Default aggressive pursuit
                                            }
                                            is_pursuing = true;
                                        }
                                    }
                                    GroupArchetype::Recon { flee_speed_bonus, .. } => {
                                        // Recon groups: Hit-and-run with circling patterns
                                        if distance_to_target > 250.0 {
                                            // Too far - approach for harassment
                                            let direction = (target_pos.0 - pos.0).normalize_or_zero();
                                            pursuit_force = direction * config.max_speed * flee_speed_bonus;
                                            is_pursuing = true;
                                        } else if distance_to_target < 180.0 {
                                            // Too close - circle strafe
                                            let to_target = target_pos.0 - pos.0;
                                            let perpendicular = Vec2::new(-to_target.y, to_target.x).normalize_or_zero();
                                            let circle_direction = perpendicular * 0.7 + to_target.normalize_or_zero() * 0.3;
                                            pursuit_force = circle_direction * config.max_speed * flee_speed_bonus;
                                            is_pursuing = true;
                                        }
                                        // If in optimal range (180-250), maintain position
                                    }
                                }
                                break;
                            }
                        }
                    }
                    _ => {} // Other group behaviors (Patrolling, Defending, etc.)
                }
            }
        }

        // Apply averaged avoidance forces with archetype-specific behavior
        if obstacle_count > 0 {
            let avg_force =
                (obstacle_force / obstacle_count as f32).normalize_or_zero() * config.max_speed;
            let steering = (avg_force - vel.0).clamp_length_max(config.max_force);
            
            // Archetype-specific obstacle avoidance
            let obstacle_multiplier = if let Some(member) = group_member {
                if let Ok(group) = group_query.get(member.group_entity) {
                    match &group.archetype {
                        GroupArchetype::Assault { .. } => 0.8, // More aggressive, less cautious around obstacles
                        GroupArchetype::Defensive { .. } => 1.4, // Very cautious, avoid obstacles early
                        GroupArchetype::Recon { .. } => 1.0, // Standard avoidance for mobility
                    }
                } else {
                    1.0
                }
            } else {
                1.0
            };
            
            acceleration += steering * config.obstacle_avoidance_weight * obstacle_multiplier;
        }

        // Apply player avoidance only if not pursuing or keeping distance
        if player_count > 0 && !is_pursuing && !is_defensive_keeping_distance {
            let avg_force =
                (player_force / player_count as f32).normalize_or_zero() * config.max_speed;
            let steering = (avg_force - vel.0).clamp_length_max(config.max_force);
            acceleration += steering * config.player_avoidance_weight;
        }

        // Apply pursuit force if pursuing
        if is_pursuing {
            let steering = (pursuit_force - vel.0).clamp_length_max(config.max_force);
            acceleration += steering * 2.0; // Strong pursuit force
        }

        // Apply defensive positioning force if keeping distance
        if is_defensive_keeping_distance {
            let steering = (pursuit_force - vel.0).clamp_length_max(config.max_force);
            acceleration += steering * 1.5; // Moderate retreat force
        }

        // Apply enhanced wall avoidance
        let wall_force = calculate_wall_avoidance(
            pos.0,
            vel.0,
            game_config.game_width,
            game_config.game_height,
            config.boundary_margin,
            config.wall_prediction_time,
            config.corner_boost_multiplier,
            config.min_velocity_threshold,
        );
        if wall_force.length_squared() > 0.0 {
            let desired_vel = wall_force * config.max_speed;
            let steering = (desired_vel - vel.0).clamp_length_max(config.max_force);
            acceleration += steering * config.wall_avoidance_weight;
        }

        // Update velocity with archetype-specific speed and agility
        let (speed_multiplier, agility_multiplier) = if let Some(member) = group_member {
            if let Ok(group) = group_query.get(member.group_entity) {
                match &group.archetype {
                    GroupArchetype::Assault { .. } => (1.1, 1.3), // Fast and agile for aggressive maneuvers
                    GroupArchetype::Defensive { .. } => (0.6, 0.7), // Much slower and less agile, methodical
                    GroupArchetype::Recon { .. } => (1.4, 1.6), // Fastest and most agile for hit-and-run
                }
            } else {
                (1.0, 1.0)
            }
        } else {
            (1.0, 1.0)
        };
        
        vel.0 += acceleration * delta * agility_multiplier;
        vel.0 = vel.0.clamp_length_max(config.max_speed * speed_multiplier);
    }
}

/// Sync network velocity to physics velocity
pub fn sync_boid_velocities(
    mut boids: Query<(&Velocity, &mut bevy_rapier2d::dynamics::Velocity), With<Boid>>,
) {
    for (network_vel, mut physics_vel) in boids.iter_mut() {
        physics_vel.linvel = network_vel.0;
        physics_vel.angvel = 0.0;
    }
}

/// Move boids based on their velocity (handled by physics now)
pub fn move_boids(mut boids: Query<(&mut Position, &Transform), With<Boid>>) {
    // Sync position from physics transform
    for (mut pos, transform) in boids.iter_mut() {
        pos.0 = transform.translation.truncate();
    }
}

pub struct FlockingPlugin;

impl Plugin for FlockingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FlockingConfig>();
        // Note: SpatialGrid is initialized by SpatialGridPlugin

        app.add_systems(
            FixedUpdate,
            (
                // Note: spatial grid update is handled by SpatialGridPlugin
                update_flocking.in_set(crate::spatial_grid::SpatialGridSet::Read),
                sync_boid_velocities.after(update_flocking),
                // Physics will move the boids
            ),
        );

        // Sync positions after physics
        app.add_systems(Update, move_boids);

        info!("Flocking plugin initialized");
    }
}

/// Calculate avoidance force for static obstacles
fn calculate_obstacle_avoidance(
    boid_pos: Vec2,
    boid_vel: Vec2,
    obstacle_pos: Vec2,
    obstacle_half_size: Vec2,
    prediction_time: f32,
    danger_zone: f32,
) -> Vec2 {
    // Predict where boid will be
    let future_pos = boid_pos + boid_vel * prediction_time;

    // Find closest point on obstacle AABB
    let closest = Vec2::new(
        future_pos.x.clamp(
            obstacle_pos.x - obstacle_half_size.x,
            obstacle_pos.x + obstacle_half_size.x,
        ),
        future_pos.y.clamp(
            obstacle_pos.y - obstacle_half_size.y,
            obstacle_pos.y + obstacle_half_size.y,
        ),
    );

    // Calculate avoidance force
    let diff = future_pos - closest;
    let distance = diff.length();

    if distance < 0.001 {
        // Inside obstacle, push out strongly
        Vec2::new(1.0, 0.0) * 2.0 // Strong push
    } else if distance < danger_zone * 0.5 {
        // Very close - emergency avoidance
        diff.normalize() * 2.0
    } else if distance < danger_zone {
        // Exponential repulsion
        diff.normalize() * (1.0 - distance / danger_zone).powi(2)
    } else {
        Vec2::ZERO
    }
}

/// Calculate avoidance force for dynamic entities (players)
fn calculate_dynamic_avoidance(
    boid_pos: Vec2,
    boid_vel: Vec2,
    target_pos: Vec2,
    target_vel: Vec2,
    avoidance_radius: f32,
    collision_threshold: f32,
) -> Vec2 {
    let relative_pos = target_pos - boid_pos;
    let distance = relative_pos.length();

    if distance > avoidance_radius || distance < 0.001 {
        return Vec2::ZERO;
    }

    // Calculate relative velocity
    let relative_vel = target_vel - boid_vel;

    // Time to closest approach
    let time_to_closest = if relative_vel.length_squared() > 0.01 {
        -(relative_pos.dot(relative_vel)) / relative_vel.length_squared()
    } else {
        0.0
    };

    // Only avoid if approaching
    if !(0.0..=2.0).contains(&time_to_closest) {
        // Not approaching or too far in future - simple repulsion
        return -relative_pos.normalize_or_zero() * (1.0 - distance / avoidance_radius).powi(2);
    }

    // Predict closest approach distance
    let future_distance = (relative_pos + relative_vel * time_to_closest).length();

    if future_distance < collision_threshold {
        // Calculate perpendicular avoidance
        let avoidance_direction = if relative_pos.perp_dot(relative_vel) > 0.0 {
            Vec2::new(-relative_pos.y, relative_pos.x).normalize()
        } else {
            Vec2::new(relative_pos.y, -relative_pos.x).normalize()
        };

        avoidance_direction * (1.0 - future_distance / collision_threshold).powi(2)
    } else {
        Vec2::ZERO
    }
}

/// Calculate enhanced wall avoidance with prediction
#[allow(clippy::too_many_arguments)]
fn calculate_wall_avoidance(
    pos: Vec2,
    vel: Vec2,
    width: f32,
    height: f32,
    margin: f32,
    prediction_time: f32,
    corner_boost: f32,
    min_velocity: f32,
) -> Vec2 {
    let mut force = Vec2::ZERO;
    let speed = vel.length();

    // Edge case: very low velocity, use position-based repulsion only
    if speed < min_velocity {
        return calculate_static_wall_repulsion(pos, width, height, margin);
    }

    let future_pos = pos + vel.normalize_or_zero() * speed.min(margin) * prediction_time;

    // Check each wall with stronger exponential repulsion
    // Left wall
    if future_pos.x < margin {
        let distance_to_wall = future_pos.x.max(0.1); // Avoid division by zero
        let strength = if distance_to_wall < margin * 0.3 {
            // Emergency repulsion when very close
            3.0
        } else {
            (1.0 - distance_to_wall / margin).powi(2)
        };
        force.x += strength;
    }
    // Right wall
    else if future_pos.x > width - margin {
        let distance_to_wall = (width - future_pos.x).max(0.1);
        let strength = if distance_to_wall < margin * 0.3 {
            3.0
        } else {
            (1.0 - distance_to_wall / margin).powi(2)
        };
        force.x -= strength;
    }

    // Top wall
    if future_pos.y < margin {
        let distance_to_wall = future_pos.y.max(0.1);
        let strength = if distance_to_wall < margin * 0.3 {
            3.0
        } else {
            (1.0 - distance_to_wall / margin).powi(2)
        };
        force.y += strength;
    }
    // Bottom wall
    else if future_pos.y > height - margin {
        let distance_to_wall = (height - future_pos.y).max(0.1);
        let strength = if distance_to_wall < margin * 0.3 {
            3.0
        } else {
            (1.0 - distance_to_wall / margin).powi(2)
        };
        force.y -= strength;
    }

    // Add corner handling - stronger force when approaching corners
    let x_near_wall = future_pos.x < margin || future_pos.x > width - margin;
    let y_near_wall = future_pos.y < margin || future_pos.y > height - margin;
    if x_near_wall && y_near_wall {
        force *= corner_boost;
    }

    force.normalize_or_zero()
}

/// Calculate static wall repulsion for zero/low velocity cases
fn calculate_static_wall_repulsion(pos: Vec2, width: f32, height: f32, margin: f32) -> Vec2 {
    let mut force = Vec2::ZERO;

    // Simple position-based repulsion
    if pos.x < margin {
        force.x += (margin - pos.x) / margin;
    } else if pos.x > width - margin {
        force.x -= (pos.x - (width - margin)) / margin;
    }

    if pos.y < margin {
        force.y += (margin - pos.y) / margin;
    } else if pos.y > height - margin {
        force.y -= (pos.y - (height - margin)) / margin;
    }

    force.normalize_or_zero()
}

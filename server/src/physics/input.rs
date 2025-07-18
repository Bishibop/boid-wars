use bevy::prelude::*;
use boid_wars_shared::{Position, Boid};
use crate::physics::core::{self, *};
use crate::config::PhysicsConfig;

/// AI system for automated testing - EXTRACTED FROM CORE.RS
pub fn ai_player_system(
    mut ai_players: Query<(&mut core::PlayerInput, &mut core::AIPlayer, &Transform), With<core::Player>>,
    other_players: Query<&Transform, (With<core::Player>, Without<core::AIPlayer>)>,
    time: Res<Time>,
    arena_config: Res<ArenaConfig>,
    physics_config: Res<PhysicsConfig>,
) {
    for (mut input, mut ai, transform) in ai_players.iter_mut() {
        ai.behavior_timer += time.delta_secs();
        ai.shoot_timer += time.delta_secs();

        let pos = transform.translation.truncate();

        match ai.ai_type {
            AIType::Circler => {
                // Move in a circle around starting position
                let circle_radius = physics_config.ai_circle_radius;
                let circle_speed = physics_config.ai_circle_speed;

                let center_x = 100.0; // Starting position
                let center_y = 100.0;

                let angle = ai.behavior_timer * circle_speed;
                let target_x = center_x + angle.cos() * circle_radius;
                let target_y = center_y + angle.sin() * circle_radius;

                let direction = Vec2::new(target_x - pos.x, target_y - pos.y).normalize_or_zero();

                input.movement = direction;
                input.aim_direction = direction;
                input.thrust = 1.0;
                input.shooting = false;
            }

            AIType::Bouncer => {
                // Bounce around randomly within arena bounds
                if ai.behavior_timer > 3.0 {
                    ai.behavior_timer = 0.0;
                    ai.target_position = Vec2::new(
                        rand::random::<f32>() * arena_config.width * 0.8 + arena_config.width * 0.1,
                        rand::random::<f32>() * arena_config.height * 0.8
                            + arena_config.height * 0.1,
                    );
                }

                let direction = (ai.target_position - pos).normalize_or_zero();
                input.movement = direction;
                input.aim_direction = direction;
                input.thrust = 1.0;

                // Shoot occasionally
                input.shooting = rand::random::<f32>() < 0.05;
            }

            AIType::Shooter => {
                // Stay in place and shoot in all directions
                input.movement = Vec2::ZERO;
                input.thrust = 0.0;

                // Rotate aim direction
                let aim_angle = ai.behavior_timer * 2.0;
                input.aim_direction = Vec2::new(aim_angle.cos(), aim_angle.sin());

                // Shoot constantly
                input.shooting = ai.shoot_timer > 0.1;
                if input.shooting {
                    ai.shoot_timer = 0.0;
                }
            }

            AIType::Chaser => {
                // Chase the nearest non-AI player
                if let Some(nearest_player) = other_players.iter().min_by(|a, b| {
                    let dist_a = a.translation.distance(transform.translation);
                    let dist_b = b.translation.distance(transform.translation);
                    dist_a
                        .partial_cmp(&dist_b)
                        .unwrap_or(std::cmp::Ordering::Equal)
                }) {
                    let target_pos = nearest_player.translation.truncate();
                    input.movement = (target_pos - pos).normalize_or_zero();
                    input.aim_direction = input.movement;
                    input.thrust = 1.0;

                    // Shoot at target
                    let distance = pos.distance(target_pos);
                    input.shooting =
                        distance < physics_config.ai_chase_shoot_distance && ai.shoot_timer > 0.3;
                    if input.shooting {
                        ai.shoot_timer = 0.0;
                    }
                } else {
                    // No target, circle around
                    input.movement = Vec2::new(
                        (ai.behavior_timer * 0.5).cos(),
                        (ai.behavior_timer * 0.5).sin(),
                    );
                    input.thrust = 0.5;
                    input.shooting = false;
                }
            }
        }
    }
}

/// System to process player input and set velocity directly - EXTRACTED FROM CORE.RS
pub fn player_input_system(
    mut player_query: Query<(
        &mut core::PlayerInput,
        &core::Player,
        &mut bevy_rapier2d::dynamics::Velocity,
        &Transform,
    )>,
    config: Res<PhysicsConfig>,
    time: Res<Time>,
) {
    for (mut input, _player, mut velocity, _transform) in player_query.iter_mut() {
        // Calculate desired velocity based on input
        let movement = input.movement.normalize_or_zero();
        let thrust_force = input.thrust;

        // Apply velocity based on movement and thrust
        let desired_velocity = movement * config.player_max_speed * thrust_force;

        // Set velocity directly for responsive movement
        velocity.linvel = desired_velocity;

        // No angular velocity for top-down movement
        velocity.angvel = 0.0;

        // Input validation - clamp values to prevent exploits
        if input.movement.length() > 1.1 {
            warn!("Invalid movement input detected: {:?}", input.movement);
            input.movement = input.movement.normalize_or_zero();
        }

        if input.aim_direction.length() > 1.1 {
            warn!("Invalid aim direction detected: {:?}", input.aim_direction);
            input.aim_direction = input.aim_direction.normalize_or_zero();
        }

        input.thrust = input.thrust.clamp(0.0, 1.0);
    }
}

/// System for swarm communication - alerts nearby boids when one is attacked - EXTRACTED FROM CORE.RS
pub fn swarm_communication_system(
    mut boid_aggression: ResMut<BoidAggression>,
    mut buffers: ResMut<PhysicsBuffers>,
    spatial_grid: Res<crate::spatial_grid::SpatialGrid>,
    boid_query: Query<(Entity, &Position), With<Boid>>,
) {
    // Clear and reuse the pre-allocated buffer
    buffers.alert_buffer.clear();
    
    // Find boids that were recently attacked and need to alert their neighbors
    for (boid_entity, boid_pos) in boid_query.iter() {
        if boid_aggression.needs_alert(boid_entity) {
            buffers.alert_buffer.push((boid_entity, boid_pos.0));
        }
    }
    
    // For each boid that needs to send an alert, find nearby boids and share the threat
    for &(alerting_boid, alerting_pos) in &buffers.alert_buffer {
        if let Some(attacker) = boid_aggression.get_attacker(alerting_boid) {
            // Find nearby boids within alert radius
            let nearby_entities =
                spatial_grid.get_nearby_entities(alerting_pos, boid_aggression.alert_radius);
            
            // Alert all nearby boids about the threat
            for entity in nearby_entities {
                if let Ok((nearby_boid, _)) = boid_query.get(entity) {
                    // Don't alert the boid to itself
                    if nearby_boid != alerting_boid {
                        // Only alert if the nearby boid isn't already tracking this attacker
                        if !boid_aggression.is_aggressive(nearby_boid) {
                            boid_aggression.record_attack(nearby_boid, attacker);
                        }
                    }
                }
            }
            
            // Mark that this boid has sent its alert
            boid_aggression.mark_alert_sent(alerting_boid);
        }
    }
}
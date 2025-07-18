use bevy::prelude::*;
use boid_wars_shared::*;
use crate::groups::BoidGroupConfig;
use crate::physics::BoidAggression;

/// Plugin for group combat coordination
pub struct GroupCombatPlugin;

impl Plugin for GroupCombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                group_target_selection,
                group_combat_coordinator.after(group_target_selection),
                rotate_active_shooters,
            ),
        );
        
        // Timer for shooter rotation
        app.insert_resource(ShooterRotationTimer(Timer::from_seconds(3.0, TimerMode::Repeating)));
    }
}

/// Timer for rotating active shooters
#[derive(Resource)]
struct ShooterRotationTimer(Timer);

/// Select targets for groups based on their behavior
fn group_target_selection(
    mut groups: Query<(&mut BoidGroup, &Position)>,
    players: Query<(Entity, &Position, &Player), Without<Boid>>,
    aggression: Res<BoidAggression>,
    config: Res<BoidGroupConfig>,
) {
    for (mut group, group_pos) in groups.iter_mut() {
        match &mut group.behavior_state {
            GroupBehavior::Patrolling { .. } => {
                // Check for nearby threats
                let detection_range = match group.archetype {
                    GroupArchetype::Recon { detection_range, .. } => detection_range,
                    _ => config.group_aggression_range,
                };
                
                // Find nearest player
                if let Some((player_entity, player_pos, player)) = players.iter()
                    .map(|(e, p, pl)| (e, p.0.distance(group_pos.0), pl))
                    .filter(|(_, dist, _)| *dist < detection_range)
                    .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
                    .map(|(e, _, pl)| (e, players.get(e).unwrap().1, pl)) {
                    
                    // Check if any boid in the group has been attacked by this player
                    let group_under_attack = aggression.boid_aggression.values()
                        .any(|data| {
                            if let Ok((_, _, p)) = players.get(data.attacker) {
                                p.id == player.id
                            } else {
                                false
                            }
                        });
                    
                    if group_under_attack {
                        // Switch to engaging
                        group.behavior_state = GroupBehavior::Engaging {
                            primary_target: player.id as u32,
                            secondary_targets: vec![],
                        };
                    }
                }
            },
            GroupBehavior::Engaging { primary_target, .. } => {
                // Check if target still exists and is in range
                let target_exists = players.iter().any(|(_, _, p)| p.id as u32 == *primary_target);
                
                if !target_exists {
                    // Return to patrolling
                    group.behavior_state = GroupBehavior::Patrolling {
                        route: group.home_territory.patrol_points.clone(),
                        current_waypoint: 0,
                    };
                } else {
                    // Check if should retreat
                    let member_count = count_group_members(&group.id);
                    let retreat_threshold = match group.archetype {
                        GroupArchetype::Defensive { retreat_threshold, .. } => retreat_threshold,
                        _ => 0.3, // Default 30% losses
                    };
                    
                    if member_count < (50.0 * (1.0 - retreat_threshold)) as usize {
                        // Retreat to home territory
                        group.behavior_state = GroupBehavior::Retreating {
                            rally_point: group.home_territory.center,
                            speed_multiplier: 1.5,
                        };
                    }
                }
            },
            GroupBehavior::Retreating { rally_point, .. } => {
                // Check if reached rally point
                if group_pos.0.distance(*rally_point) < 100.0 {
                    // Switch to defending
                    group.behavior_state = GroupBehavior::Defending {
                        position: *rally_point,
                        radius: 200.0,
                    };
                }
            },
            _ => {},
        }
    }
}

/// Coordinate combat for groups
fn group_combat_coordinator(
    mut groups: Query<(&mut BoidGroup, &Position)>,
    mut boids: Query<(Entity, &BoidGroupMember, &mut BoidCombat, &Position), With<Boid>>,
    players: Query<(&Position, &Player), Without<Boid>>,
) {
    for (mut group, group_pos) in groups.iter_mut() {
        if let GroupBehavior::Engaging { primary_target, .. } = &group.behavior_state {
            // Find target player position
            if let Some((target_pos, _)) = players.iter()
                .find(|(_, p)| p.id as u32 == *primary_target) {
                
                // Update active shooters set
                update_active_shooters(&mut group, &boids, target_pos.0);
                
                // Update combat state for all group members
                for (entity, member, mut combat, pos) in boids.iter_mut() {
                    if member.group_id == group.id {
                        if group.active_shooters.contains(&entity) {
                            // Active shooter - enable combat with very slow fire rate
                            // Don't reset last_shot_time - let natural cooldown happen
                            
                            // Adjust fire rate based on role (much slower rates)
                            combat.fire_rate = match member.role_in_group {
                                BoidRole::Leader => 0.15,    // 1 shot every ~6.7 seconds
                                BoidRole::Flanker => 0.12,   // 1 shot every ~8.3 seconds
                                _ => 0.1,                    // 1 shot every 10 seconds
                            };
                        } else {
                            // Non-shooter - disable combat by setting very low fire rate
                            combat.fire_rate = 0.01; // 1 shot every 100 seconds (effectively never)
                        }
                    }
                }
            }
        } else {
            // Not engaging - clear active shooters
            group.active_shooters.clear();
        }
    }
}

/// Update the set of active shooters for a group
fn update_active_shooters(
    group: &mut BoidGroup,
    boids: &Query<(Entity, &BoidGroupMember, &mut BoidCombat, &Position), With<Boid>>,
    target_pos: Vec2,
) {
    // Get all eligible shooters
    let mut eligible_shooters: Vec<(Entity, f32, BoidRole)> = boids.iter()
        .filter(|(_, member, _, _)| member.group_id == group.id)
        .map(|(entity, member, _, pos)| {
            let distance = pos.0.distance(target_pos);
            (entity, distance, member.role_in_group)
        })
        .collect();
    
    // Sort by priority (role and distance)
    eligible_shooters.sort_by(|a, b| {
        // Prioritize by role first
        let role_priority_a = match a.2 {
            BoidRole::Leader => 0,
            BoidRole::Flanker => 1,
            BoidRole::Support => 2,
            BoidRole::Scout => 3,
        };
        let role_priority_b = match b.2 {
            BoidRole::Leader => 0,
            BoidRole::Flanker => 1,
            BoidRole::Support => 2,
            BoidRole::Scout => 3,
        };
        
        if role_priority_a != role_priority_b {
            role_priority_a.cmp(&role_priority_b)
        } else {
            // Then by distance
            a.1.partial_cmp(&b.1).unwrap()
        }
    });
    
    // Keep existing shooters if still eligible
    let mut new_shooters = std::collections::HashSet::new();
    let desired_count = group.max_shooters as usize;
    
    // First, retain existing shooters that are still eligible
    for (entity, _, _) in &eligible_shooters {
        if group.active_shooters.contains(entity) && new_shooters.len() < desired_count {
            new_shooters.insert(*entity);
        }
    }
    
    // Then add new shooters if needed
    for (entity, _, _) in &eligible_shooters {
        if new_shooters.len() >= desired_count {
            break;
        }
        new_shooters.insert(*entity);
    }
    
    group.active_shooters = new_shooters;
}

/// Periodically rotate active shooters
fn rotate_active_shooters(
    mut groups: Query<&mut BoidGroup>,
    mut timer: ResMut<ShooterRotationTimer>,
    time: Res<Time>,
) {
    timer.0.tick(time.delta());
    
    if timer.0.just_finished() {
        for mut group in groups.iter_mut() {
            if matches!(group.behavior_state, GroupBehavior::Engaging { .. }) {
                // Force rotation by clearing one shooter
                if group.active_shooters.len() > 1 {
                    // Remove the first shooter (oldest)
                    if let Some(&first) = group.active_shooters.iter().next() {
                        group.active_shooters.remove(&first);
                    }
                }
            }
        }
    }
}

/// Count members of a group (placeholder - in real implementation would query)
fn count_group_members(group_id: &u32) -> usize {
    // This is a simplified version - in reality would query all boids with matching group_id
    50 // Default group size
}
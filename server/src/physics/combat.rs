use bevy::prelude::*;
use boid_wars_shared::{self, BoidCombat, Position, Boid};
use boid_wars_shared::{Player as SharedPlayer, Projectile as SharedProjectile, Velocity as SharedVelocity};
use bevy_rapier2d::prelude::{*, Velocity as RapierVelocity};
use lightyear::prelude::server::Replicate;
use std::time::{Duration, Instant};
use crate::physics::core::{
    Projectile, ProjectilePhysics, ProjectileType, BoidAggression, 
    PooledProjectile, PROJECTILE_NAME, BOID_PROJECTILE_NAME, Player, PlayerInput,
    WeaponStats, PlayerAggression, GameCollisionGroups
};
use crate::physics::{ProjectilePool, BoidProjectilePool};
use crate::config::PhysicsConfig;

/// System to handle player shooting - EXTRACTED FROM CORE.RS
pub fn shooting_system(
    mut commands: Commands,
    mut player_query: Query<(Entity, &PlayerInput, &mut Player, &WeaponStats, &Transform)>,
    mut pool: ResMut<ProjectilePool>,
    mut player_aggression: ResMut<PlayerAggression>,
    time: Res<Time>,
    config: Res<PhysicsConfig>,
) {
    for (entity, input, mut player, weapon, transform) in player_query.iter_mut() {
        player.weapon_cooldown.tick(time.delta());

        if input.shooting && player.weapon_cooldown.finished() {
            // Reset cooldown
            player.weapon_cooldown.reset();

            // Mark player as aggressive
            player_aggression.mark_aggressive(entity);

            // Spawn projectile - offset in the aim direction to avoid self-collision
            let spawn_offset = config.projectile_spawn_offset;
            let projectile_spawn_pos =
                transform.translation.truncate() + input.aim_direction * spawn_offset; // Spawn in aim direction

            let projectile_velocity = input.aim_direction * weapon.projectile_speed;

            // Try to get a projectile from the pool
            let projectile_entity = if let Some(pooled_handle) = pool.acquire() {
                let _status = pool.status();

                // Update existing projectile components
                commands.entity(pooled_handle.entity).insert((
                    // Update projectile data
                    Projectile {
                        damage: weapon.damage,
                        owner: Some(entity), // Use actual player entity
                        projectile_type: ProjectileType::Basic,
                        lifetime: {
                            let mut timer = Timer::new(weapon.projectile_lifetime, TimerMode::Once);
                            timer.unpause(); // Make sure timer is running
                            timer
                        },
                        speed: weapon.projectile_speed,
                    },
                    ProjectilePhysics {
                        velocity: projectile_velocity,
                        spawn_time: Instant::now(),
                        max_lifetime: weapon.projectile_lifetime,
                    },
                    // Reset physics state
                    Transform::from_translation(projectile_spawn_pos.extend(0.0)),
                    RapierVelocity::linear(projectile_velocity),
                    // Re-enable collision detection
                    ActiveEvents::COLLISION_EVENTS,
                ));

                // Store the pooled handle for later release
                commands
                    .entity(pooled_handle.entity)
                    .insert(PooledProjectile(pooled_handle));

                pooled_handle.entity
            } else {
                warn!("[POOL] Pool exhausted! Spawning new projectile (this may cause performance issues)");

                // Pool is empty, spawn a new projectile
                commands
                    .spawn((
                        // Physics projectile component (server-only)
                        Projectile {
                            damage: weapon.damage,
                            owner: Some(entity), // Use actual player entity
                            projectile_type: ProjectileType::Basic,
                            lifetime: Timer::new(weapon.projectile_lifetime, TimerMode::Once),
                            speed: weapon.projectile_speed,
                        },
                        ProjectilePhysics {
                            velocity: projectile_velocity,
                            spawn_time: Instant::now(),
                            max_lifetime: weapon.projectile_lifetime,
                        },
                        // Rapier2D components
                        RigidBody::Dynamic,
                        Collider::ball(config.projectile_collider_radius),
                        Sensor, // Make it a sensor so it doesn't bounce
                        GameCollisionGroups::projectile(),
                        ActiveEvents::COLLISION_EVENTS, // Enable collision events
                        RapierVelocity::linear(projectile_velocity),
                        Transform::from_translation(projectile_spawn_pos.extend(0.0)),
                        GlobalTransform::default(),
                        bevy_rapier2d::dynamics::GravityScale(0.0), // Disable gravity for projectiles
                        Name::new(PROJECTILE_NAME),
                    ))
                    .id()
            };

            // Add network components for client replication
            commands.entity(projectile_entity).insert((
                // Network components for replication
                SharedProjectile {
                    id: projectile_entity.index(), // Use entity index as ID
                    damage: weapon.damage,
                    owner_id: player.player_id,
                },
                Position(projectile_spawn_pos),
                SharedVelocity(projectile_velocity),
                Replicate::default(),
            ));
        }
    }
}

/// System for boid shooting behavior - EXTRACTED FROM CORE.RS
pub fn boid_shooting_system(
    mut commands: Commands,
    mut boid_query: Query<
        (
            Entity,
            &Transform,
            &mut BoidCombat,
            &Position,
        ),
        With<Boid>,
    >,
    player_query: Query<(Entity, &Position), With<SharedPlayer>>,
    boid_aggression: Res<BoidAggression>,
    spatial_grid: Res<crate::spatial_grid::SpatialGrid>,
    mut boid_pool: ResMut<BoidProjectilePool>,
    time: Res<Time>,
    config: Res<PhysicsConfig>,
) {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    for (boid_entity, transform, mut combat, boid_pos) in boid_query.iter_mut() {
        // Update shooting timer
        combat.last_shot_time += time.delta_secs();

        // Check if boid can shoot (cooldown finished)
        let cooldown_time = 1.0 / combat.fire_rate; // Convert fire rate to cooldown
        if combat.last_shot_time < cooldown_time {
            continue;
        }

        // Find target player
        let target_pos = find_boid_target(
            boid_entity,
            boid_pos,
            &player_query,
            &boid_aggression,
            &spatial_grid,
            combat.aggression_range,
        );

        if let Some(target_position) = target_pos {
            // Reset shooting timer
            combat.last_shot_time = 0.0;

            // Calculate aim direction with spread
            let base_direction = (target_position - boid_pos.0).normalize();
            let spread_angle = rng.gen_range(-combat.spread_angle..combat.spread_angle);
            let rotation = std::f32::consts::PI * spread_angle;
            let cos_rot = rotation.cos();
            let sin_rot = rotation.sin();

            // Apply rotation to direction vector
            let aim_direction = Vec2::new(
                base_direction.x * cos_rot - base_direction.y * sin_rot,
                base_direction.x * sin_rot + base_direction.y * cos_rot,
            );

            // Spawn projectile
            let spawn_offset = config.projectile_spawn_offset;
            let projectile_spawn_pos =
                transform.translation.truncate() + aim_direction * spawn_offset;
            let projectile_velocity = aim_direction * combat.projectile_speed;

            // Try to get a projectile from the boid pool
            let projectile_entity = if let Some(pooled_handle) = boid_pool.acquire() {
                // Update existing projectile components
                commands.entity(pooled_handle.entity).insert((
                    Projectile {
                        damage: combat.damage,
                        owner: Some(boid_entity),
                        projectile_type: ProjectileType::Basic,
                        lifetime: {
                            let mut timer = Timer::new(Duration::from_secs(2), TimerMode::Once);
                            timer.unpause();
                            timer
                        },
                        speed: combat.projectile_speed,
                    },
                    ProjectilePhysics {
                        velocity: projectile_velocity,
                        spawn_time: Instant::now(),
                        max_lifetime: Duration::from_secs(2),
                    },
                    Transform::from_translation(projectile_spawn_pos.extend(0.0)),
                    RapierVelocity::linear(projectile_velocity),
                    ActiveEvents::COLLISION_EVENTS,
                ));

                commands
                    .entity(pooled_handle.entity)
                    .insert(PooledProjectile(pooled_handle));

                pooled_handle.entity
            } else {
                // Pool exhausted, spawn new projectile
                commands
                    .spawn((
                        Projectile {
                            damage: combat.damage,
                            owner: Some(boid_entity),
                            projectile_type: ProjectileType::Basic,
                            lifetime: Timer::new(Duration::from_secs(2), TimerMode::Once),
                            speed: combat.projectile_speed,
                        },
                        ProjectilePhysics {
                            velocity: projectile_velocity,
                            spawn_time: Instant::now(),
                            max_lifetime: Duration::from_secs(2),
                        },
                        RigidBody::Dynamic,
                        Collider::ball(config.projectile_collider_radius),
                        Sensor,
                        GameCollisionGroups::boid_projectile(),
                        ActiveEvents::COLLISION_EVENTS,
                        RapierVelocity::linear(projectile_velocity),
                        Transform::from_translation(projectile_spawn_pos.extend(0.0)),
                        GlobalTransform::default(),
                        bevy_rapier2d::dynamics::GravityScale(0.0),
                        Name::new(BOID_PROJECTILE_NAME),
                    ))
                    .id()
            };

            // Add network components for client replication
            commands.entity(projectile_entity).insert((
                SharedProjectile {
                    id: projectile_entity.index(),
                    damage: combat.damage,
                    owner_id: boid_entity.index() as u64, // Use boid entity as owner
                },
                Position(projectile_spawn_pos),
                SharedVelocity(projectile_velocity),
                Replicate::default(),
            ));
        }
    }
}

/// Find target for boid shooting - EXTRACTED FROM CORE.RS
fn find_boid_target(
    boid_entity: Entity,
    boid_pos: &Position,
    players: &Query<(Entity, &Position), With<SharedPlayer>>,
    aggression: &BoidAggression,
    spatial_grid: &crate::spatial_grid::SpatialGrid,
    range: f32,
) -> Option<Vec2> {
    // 1. Check if boid has a remembered attacker
    if let Some(attacker_entity) = aggression.get_attacker(boid_entity) {
        if let Ok((_, player_pos)) = players.get(attacker_entity) {
            let distance = boid_pos.0.distance(player_pos.0);
            if distance <= range {
                return Some(player_pos.0);
            }
        }
    }

    // 2. Use spatial grid to find nearby players
    let nearby_entities = spatial_grid.get_nearby_entities(boid_pos.0, range);

    let mut closest_player = None;
    let mut closest_distance = range;

    for entity in nearby_entities {
        if let Ok((_, player_pos)) = players.get(entity) {
            let distance = boid_pos.0.distance(player_pos.0);
            if distance < closest_distance {
                closest_distance = distance;
                closest_player = Some(player_pos.0);
            }
        }
    }

    closest_player
}
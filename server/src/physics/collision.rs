use bevy::prelude::*;
use bevy_rapier2d::prelude::{*, Velocity as RapierVelocity};
use boid_wars_shared::{Health, Boid, Obstacle};
use boid_wars_shared::{Projectile as SharedProjectile, Position as SharedPosition, Velocity as SharedVelocity};
use crate::physics::core::{
    self, Despawning, PhysicsBuffers, BoidAggression, PooledProjectile, 
    Projectile, ProjectileTemplate, BoidProjectileTemplate
};
use crate::physics::{ProjectilePool, BoidProjectilePool};
use crate::config::PhysicsConfig;

/// System to handle collisions - EXTRACTED FROM CORE.RS
pub fn collision_system(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    mut buffers: ResMut<PhysicsBuffers>,
    mut health_queries: ParamSet<(
        Query<(&mut core::Player, Option<&mut Health>)>,
        Query<&mut Health, With<Boid>>,
    )>,
    projectile_query: Query<&Projectile>,
    boid_entity_query: Query<Entity, With<Boid>>,
    _obstacle_query: Query<Entity, With<Obstacle>>,
    mut boid_aggression: ResMut<BoidAggression>,
) {
    // Clear and reuse pre-allocated buffers
    buffers.player_collision_buffer.clear();
    buffers.boid_collision_buffer.clear();

    // Process collision events directly without intermediate collection
    for collision_event in collision_events.read() {
        if let CollisionEvent::Started(entity1, entity2, _) = collision_event {
            // Check if entity1 is a projectile
            if let Ok(projectile) = projectile_query.get(*entity1) {
                if health_queries.p0().get(*entity2).is_ok() {
                    // Projectile hit player
                    buffers.player_collision_buffer.push((
                        *entity1,
                        *entity2,
                        projectile.damage,
                        projectile.owner,
                    ));
                } else if boid_entity_query.get(*entity2).is_ok() {
                    // Projectile hit boid
                    buffers.boid_collision_buffer.push((
                        *entity1,
                        *entity2,
                        projectile.damage,
                        projectile.owner,
                    ));
                }
            }

            // Check if entity2 is a projectile
            if let Ok(projectile) = projectile_query.get(*entity2) {
                if health_queries.p0().get(*entity1).is_ok() {
                    // Projectile hit player
                    buffers.player_collision_buffer.push((
                        *entity2,
                        *entity1,
                        projectile.damage,
                        projectile.owner,
                    ));
                } else if boid_entity_query.get(*entity1).is_ok() {
                    // Projectile hit boid
                    buffers.boid_collision_buffer.push((
                        *entity2,
                        *entity1,
                        projectile.damage,
                        projectile.owner,
                    ));
                }
            }
        }
    }

    // Process player collisions
    for &(projectile_entity, player_entity, damage, _owner) in &buffers.player_collision_buffer {
        if let Ok((mut player, health_opt)) = health_queries.p0().get_mut(player_entity) {
            // Hit a player - apply damage with clamping
            player.health = (player.health - damage).max(0.0);

            // Sync to Health component if it exists
            if let Some(mut health) = health_opt {
                health.current = player.health;
            }

            if player.health <= 0.0 {
                handle_player_death(&mut commands, player_entity);
            }

            // Mark projectile for despawn
            commands.entity(projectile_entity).insert(Despawning);
        }
    }

    // Process boid collisions
    for &(projectile_entity, boid_entity, damage, owner) in &buffers.boid_collision_buffer {
        // Check if owner is a player first (before borrowing health)
        let owner_is_player = if let Some(owner_entity) = owner {
            health_queries.p0().get(owner_entity).is_ok()
        } else {
            false
        };

        if let Ok(mut health) = health_queries.p1().get_mut(boid_entity) {
            // Hit a boid - apply damage
            health.current = (health.current - damage).max(0.0);

            // Track aggression if projectile came from a player
            if owner_is_player {
                if let Some(owner_entity) = owner {
                    boid_aggression.record_attack(boid_entity, owner_entity);
                }
            }

            // Handle boid death
            if health.current <= 0.0 {
                commands.entity(boid_entity).insert(Despawning);
            }

            // Mark projectile for despawn
            commands.entity(projectile_entity).insert(Despawning);
        }
    }
}

/// Handle player death - EXTRACTED FROM CORE.RS
fn handle_player_death(commands: &mut Commands, player_entity: Entity) {
    // In battle royale mode, death is permanent
    // Mark the player entity for cleanup
    commands.entity(player_entity).insert(Despawning);

    // TODO: Emit death event for UI/spectator mode
    // TODO: Update game state for battle royale (track remaining players)
    // TODO: Trigger death visual/audio effects

    info!("Player {:?} has been eliminated", player_entity);
}

/// System to return projectiles to pool instead of despawning - EXTRACTED FROM CORE.RS
#[allow(clippy::type_complexity)]
pub fn return_projectiles_to_pool(
    mut commands: Commands,
    mut player_pool: ResMut<ProjectilePool>,
    mut boid_pool: ResMut<BoidProjectilePool>,
    mut projectiles: Query<
        (
            Entity,
            &mut Transform,
            &mut RapierVelocity,
            &mut Projectile,
            Option<&PooledProjectile>,
            Option<&Despawning>,
            Option<&ProjectileTemplate>,
            Option<&BoidProjectileTemplate>,
        ),
        With<Projectile>,
    >,
    config: Res<PhysicsConfig>,
) {
    for (
        entity,
        mut transform,
        mut velocity,
        mut projectile,
        pooled,
        despawning,
        player_template,
        boid_template,
    ) in projectiles.iter_mut()
    {
        // Check if this projectile should be returned to pool
        let should_return = despawning.is_some() || projectile.lifetime.finished();

        if should_return {
            // Only process pooled projectiles
            if let Some(PooledProjectile(pooled_handle)) = pooled {
                // Determine which pool to use based on template type
                let pool_valid = if player_template.is_some() {
                    player_pool.is_valid(*pooled_handle)
                } else if boid_template.is_some() {
                    boid_pool.is_valid(*pooled_handle)
                } else {
                    // No template component, assume it's not pooled
                    false
                };

                // Validate the handle is still valid
                if !pool_valid {
                    warn!(
                        "[POOL] Invalid pooled handle for entity {:?}, removing from world",
                        entity
                    );
                    commands.entity(entity).despawn();
                    continue;
                }

                // Reset projectile state
                transform.translation = config.projectile_pool_offscreen_position;
                *velocity = RapierVelocity::zero();
                projectile.lifetime.reset();
                projectile.lifetime.pause(); // Pause the timer so it doesn't tick while pooled

                // Remove network components to stop replication (with error handling)
                if let Ok(mut entity_commands) = commands.get_entity(entity) {
                    entity_commands.remove::<SharedProjectile>();
                    entity_commands.remove::<SharedPosition>();
                    entity_commands.remove::<SharedVelocity>();
                    entity_commands.remove::<lightyear::prelude::server::Replicate>();

                    // Remove Despawning marker if present
                    if despawning.is_some() {
                        entity_commands.remove::<Despawning>();
                    }

                    // Make sure it stays a sensor
                    entity_commands.insert(Sensor);

                    // Return to appropriate pool based on template type
                    let released = if player_template.is_some() {
                        player_pool.release(*pooled_handle)
                    } else if boid_template.is_some() {
                        boid_pool.release(*pooled_handle)
                    } else {
                        false
                    };

                    if !released {
                        warn!("[POOL] Failed to return projectile {:?} to pool - possible double-release", entity);
                    }
                } else {
                    warn!("[POOL] Failed to return projectile {:?} to pool - entity may have been despawned", entity);
                }
            } else if despawning.is_some() {
                // Non-pooled projectile marked for despawn
                commands.entity(entity).despawn();
            }
        }
    }
}
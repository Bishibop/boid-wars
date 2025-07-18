use bevy::prelude::*;
use crate::physics::core::{self, *};
use crate::config::PhysicsConfig;

/// System to handle player movement physics - EXTRACTED FROM CORE.RS
pub fn player_movement_system(
    mut player_query: Query<(&core::Player, &Ship, &mut bevy_rapier2d::dynamics::Velocity, &Transform), With<core::Player>>,
    _time: Res<Time>,
    config: Res<PhysicsConfig>,
) {
    for (_player, ship, mut velocity, _transform) in player_query.iter_mut() {
        // Apply damping for momentum feel
        velocity.linvel *= config.player_damping_factor;
        velocity.angvel *= config.player_damping_factor;

        // Clamp max speed
        if velocity.linvel.length() > ship.max_speed {
            velocity.linvel = velocity.linvel.normalize() * ship.max_speed;
        }
    }
}

/// System to update projectiles - EXTRACTED FROM CORE.RS
pub fn projectile_system(
    mut commands: Commands,
    mut projectile_query: Query<(Entity, &mut core::Projectile, &core::ProjectilePhysics, &Transform)>,
    time: Res<Time>,
    arena_config: Res<ArenaConfig>,
    mut debug_timer: Local<f32>,
) {
    *debug_timer += time.delta_secs();

    let _active_projectiles = projectile_query
        .iter()
        .filter(|(_, _, _, transform)| {
            let pos = transform.translation.truncate();
            pos.x > -500.0 && pos.y > -500.0 // Only count projectiles not in pool area
        })
        .count();

    for (entity, mut projectile, _physics, transform) in projectile_query.iter_mut() {
        // Skip pooled projectiles (they're positioned off-screen)
        let pos = transform.translation.truncate();
        if pos.x < -500.0 || pos.y < -500.0 {
            continue; // This is a pooled projectile, skip processing
        }

        // Update lifetime only for active projectiles
        projectile.lifetime.tick(time.delta());

        // Mark for despawn if lifetime expired (will be returned to pool)
        if projectile.lifetime.finished() {
            commands.entity(entity).insert(Despawning);
            continue;
        }

        // Check world bounds (using top-left origin coordinate system)
        if pos.x < 0.0 || pos.x > arena_config.width || pos.y < 0.0 || pos.y > arena_config.height {
            commands.entity(entity).insert(Despawning);
        }
    }
}
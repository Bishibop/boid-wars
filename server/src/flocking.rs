use crate::spatial_grid::SpatialGrid;
use bevy::prelude::*;
use boid_wars_shared::{Boid, Position, Velocity};

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

            // Movement parameters
            max_speed: 200.0,
            max_force: 400.0,

            // Boundary behavior
            boundary_margin: 50.0,
            boundary_turn_force: 2.0,
        }
    }
}

/// Simple flocking system that updates boid velocities
pub fn update_flocking(
    mut boids: Query<(Entity, &Position, &mut Velocity), With<Boid>>,
    spatial_grid: Res<SpatialGrid>,
    config: Res<FlockingConfig>,
    time: Res<Time>,
) {
    let game_config = &*boid_wars_shared::GAME_CONFIG;
    let delta = time.delta_secs();

    // Use largest radius for spatial query
    let search_radius = config
        .separation_radius
        .max(config.alignment_radius)
        .max(config.cohesion_radius);

    // Collect all boid data first to avoid borrow checker issues
    let boid_data: Vec<(Entity, Vec2, Vec2)> = boids
        .iter()
        .map(|(entity, pos, vel)| (entity, pos.0, vel.0))
        .collect();

    // Update each boid
    for (entity, pos, mut vel) in boids.iter_mut() {
        let mut separation = Vec2::ZERO;
        let mut alignment = Vec2::ZERO;
        let mut cohesion = Vec2::ZERO;
        let mut sep_count = 0;
        let mut align_count = 0;
        let mut cohesion_count = 0;

        // Get nearby entities from spatial grid
        let nearby = spatial_grid.get_nearby_entities(pos.0, search_radius);

        // Calculate flocking forces from neighbors
        for &other_entity in &nearby {
            if other_entity == entity {
                continue;
            }

            // Find the other boid's data in our collected data
            if let Some((_, other_pos, other_vel)) =
                boid_data.iter().find(|(e, _, _)| *e == other_entity)
            {
                let diff = pos.0 - *other_pos;
                let distance = diff.length();

                // Separation: avoid crowding
                if distance > 0.0 && distance < config.separation_radius {
                    let force = diff.normalize() / distance; // Inverse square law
                    separation += force;
                    sep_count += 1;
                }

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
            }
        }

        // Calculate steering forces
        let mut acceleration = Vec2::ZERO;

        // Apply separation
        if sep_count > 0 {
            separation = (separation / sep_count as f32).normalize_or_zero() * config.max_speed;
            separation = (separation - vel.0).clamp_length_max(config.max_force);
            acceleration += separation * config.separation_weight;
        }

        // Apply alignment
        if align_count > 0 {
            alignment = (alignment / align_count as f32).normalize_or_zero() * config.max_speed;
            alignment = (alignment - vel.0).clamp_length_max(config.max_force);
            acceleration += alignment * config.alignment_weight;
        }

        // Apply cohesion
        if cohesion_count > 0 {
            let center = cohesion / cohesion_count as f32;
            let desired = (center - pos.0).normalize_or_zero() * config.max_speed;
            cohesion = (desired - vel.0).clamp_length_max(config.max_force);
            acceleration += cohesion * config.cohesion_weight;
        }

        // Apply boundary avoidance
        let margin = config.boundary_margin;
        let turn_force = config.boundary_turn_force;

        if pos.0.x < margin {
            acceleration.x += turn_force;
        } else if pos.0.x > game_config.game_width - margin {
            acceleration.x -= turn_force;
        }

        if pos.0.y < margin {
            acceleration.y += turn_force;
        } else if pos.0.y > game_config.game_height - margin {
            acceleration.y -= turn_force;
        }

        // Update velocity
        vel.0 += acceleration * delta;
        vel.0 = vel.0.clamp_length_max(config.max_speed);
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
        app.init_resource::<crate::spatial_grid::SpatialGrid>();

        app.add_systems(
            FixedUpdate,
            (
                crate::spatial_grid::update_spatial_grid,
                update_flocking,
                sync_boid_velocities,
                // Physics will move the boids
            )
                .chain(),
        );

        // Sync positions after physics
        app.add_systems(Update, move_boids);

        info!("Flocking plugin initialized with spatial grid");
    }
}

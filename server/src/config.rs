use bevy::prelude::*;
use std::time::Duration;

/// Physics configuration
#[derive(Resource)]
pub struct PhysicsConfig {
    // Player physics
    pub player_thrust_force: f32,
    pub player_turn_rate: f32,
    pub player_forward_speed_multiplier: f32,
    pub player_max_speed: f32,
    pub player_acceleration: f32,
    pub player_deceleration: f32,
    pub player_collider_size: f32, // Half-size for cuboid (31.2 for 62.4x62.4 sprite)
    pub player_damping_factor: f32,

    // Projectile physics
    pub projectile_speed: f32,
    pub projectile_lifetime: Duration,
    pub projectile_damage: f32,
    pub projectile_fire_rate: f32,
    pub projectile_spawn_offset: f32,
    pub projectile_collider_radius: f32,

    // Pool configuration
    pub projectile_pool_size: usize,
    pub projectile_pool_initial_spawn: usize,
    pub projectile_pool_offscreen_position: Vec3,

    // Arena configuration
    pub arena_wall_thickness: f32,

    // AI configuration
    pub ai_circle_radius: f32,
    pub ai_circle_speed: f32,
    pub ai_shoot_interval: f32,
    pub ai_aim_spread: f32,
    pub ai_chase_shoot_distance: f32,

    // Boid physics
    pub boid_radius: f32, // Radius for ball collider (12.0 for 24x24 sprite)

    // Boid aggression
    pub boid_aggression_memory_duration: Duration,
    pub boid_aggression_alert_radius: f32,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            // Player physics
            player_thrust_force: 50000.0,
            player_turn_rate: 3.0,
            player_forward_speed_multiplier: 1.5,
            player_max_speed: 800.0,
            player_acceleration: 800.0,
            player_deceleration: 400.0,
            player_collider_size: 31.2, // Half of 62.4x62.4 sprite size
            player_damping_factor: 0.98,

            // Projectile physics
            projectile_speed: 900.0, // Increased from 600.0 for faster bullets
            projectile_lifetime: Duration::from_secs(3),
            projectile_damage: 25.0,
            projectile_fire_rate: 8.0, // Increased from 4.0 for faster firing
            projectile_spawn_offset: 50.0, // Increased from 15.0 to spawn farther from ship
            projectile_collider_radius: 9.0, // Radius for 18x18 sprite

            // Pool configuration
            projectile_pool_size: 500,
            projectile_pool_initial_spawn: 100,
            projectile_pool_offscreen_position: Vec3::new(-1000.0, -1000.0, -100.0),

            // Arena configuration
            arena_wall_thickness: 25.0,

            // AI configuration
            ai_circle_radius: 100.0,
            ai_circle_speed: 1.0,
            ai_shoot_interval: 2.0,
            ai_aim_spread: 0.2,
            ai_chase_shoot_distance: 300.0,

            // Boid physics
            boid_radius: 12.0, // Radius for 24x24 sprite (diameter = 24)

            // Boid aggression
            boid_aggression_memory_duration: Duration::from_secs(5),
            boid_aggression_alert_radius: 150.0,
        }
    }
}

/// Performance monitoring configuration
#[derive(Resource)]
pub struct MonitoringConfig {
    pub pool_health_check_interval: f32,
    pub pool_low_threshold: usize,
    pub pool_high_utilization_threshold: f32,
    pub status_log_interval: f32,
    pub projectile_log_interval: f32,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            pool_health_check_interval: 10.0,
            pool_low_threshold: 10,
            pool_high_utilization_threshold: 80.0,
            status_log_interval: 5.0,
            projectile_log_interval: 5.0,
        }
    }
}

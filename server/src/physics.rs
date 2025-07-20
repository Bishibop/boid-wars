use crate::config::{MonitoringConfig, PhysicsConfig};
use crate::pool::{BoundedPool, PooledEntity};
use crate::spatial_grid::SpatialGridSet;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use boid_wars_shared;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::f32::consts::PI;
use std::time::{Duration, Instant};

// Static names to avoid runtime allocations
const POOLED_PROJECTILE_NAME: &str = "Pooled Projectile";
const POOLED_BOID_PROJECTILE_NAME: &str = "Pooled Boid Projectile";
const PROJECTILE_NAME: &str = "Projectile";
const BOID_PROJECTILE_NAME: &str = "Boid Projectile";
const TOP_WALL_NAME: &str = "Top Wall";
const BOTTOM_WALL_NAME: &str = "Bottom Wall";
const LEFT_WALL_NAME: &str = "Left Wall";
const RIGHT_WALL_NAME: &str = "Right Wall";

// BOID_RADIUS constant now in PhysicsConfig.boid_radius

/// Pre-allocated buffers for hot path operations
#[derive(Resource)]
pub struct PhysicsBuffers {
    /// Buffer for swarm communication alerts
    pub alert_buffer: Vec<(Entity, Vec2)>,
    /// Buffer for player collision data
    pub player_collision_buffer: Vec<(Entity, Entity, f32, Option<Entity>)>,
    /// Buffer for boid collision data
    pub boid_collision_buffer: Vec<(Entity, Entity, f32, Option<Entity>)>,
}

impl Default for PhysicsBuffers {
    fn default() -> Self {
        Self {
            // Pre-allocate reasonable capacities
            alert_buffer: Vec::with_capacity(128),
            player_collision_buffer: Vec::with_capacity(64),
            boid_collision_buffer: Vec::with_capacity(256),
        }
    }
}

/// Marker component to indicate entity is being despawned
#[derive(Component)]
pub struct Despawning;

/// Component to track pooled projectiles
#[derive(Component)]
pub struct PooledProjectile(PooledEntity);

/// Resource to track player aggression for boid AI
#[derive(Resource)]
pub struct PlayerAggression {
    /// Maps player entity to the time they last attacked
    pub aggressive_players: HashMap<Entity, Instant>,
    /// How long a player remains "aggressive" after attacking
    pub aggression_duration: Duration,
    /// Last time cleanup was performed
    last_cleanup: Instant,
    /// How often to cleanup expired entries (in seconds)
    cleanup_interval: f32,
}

impl Default for PlayerAggression {
    fn default() -> Self {
        Self {
            aggressive_players: HashMap::new(),
            aggression_duration: Duration::from_secs(5), // Players stay aggressive for 5 seconds
            last_cleanup: Instant::now(),
            cleanup_interval: 1.0,
        }
    }
}

impl PlayerAggression {
    /// Mark a player as aggressive
    pub fn mark_aggressive(&mut self, player: Entity) {
        // Enforce maximum size to prevent unbounded growth
        const MAX_TRACKED_PLAYERS: usize = 100;

        // If we're at capacity and this is a new player, remove the oldest entry
        if self.aggressive_players.len() >= MAX_TRACKED_PLAYERS
            && !self.aggressive_players.contains_key(&player)
        {
            // Find and remove the oldest entry
            if let Some(&oldest_player) = self
                .aggressive_players
                .iter()
                .min_by_key(|(_, &time)| time)
                .map(|(entity, _)| entity)
            {
                self.aggressive_players.remove(&oldest_player);
            }
        }

        self.aggressive_players.insert(player, Instant::now());
    }

    /// Check if a player is currently aggressive
    pub fn is_aggressive(&self, player: Entity) -> bool {
        if let Some(&last_attack) = self.aggressive_players.get(&player) {
            last_attack.elapsed() < self.aggression_duration
        } else {
            false
        }
    }

    /// Clean up expired aggression entries (only runs periodically)
    pub fn cleanup_expired(&mut self) {
        let now = Instant::now();

        // Only cleanup if enough time has passed
        if now.duration_since(self.last_cleanup).as_secs_f32() < self.cleanup_interval {
            return;
        }

        self.last_cleanup = now;
        self.aggressive_players.retain(|_, &mut last_attack| {
            now.duration_since(last_attack) < self.aggression_duration
        });
    }
}

/// Resource to track boid aggression and enable swarm communication
#[derive(Resource)]
pub struct BoidAggression {
    /// Maps boid entity to aggression data
    pub boid_aggression: HashMap<Entity, BoidAggressionData>,
    /// How long boids remember their attackers
    pub aggression_duration: Duration,
    /// Radius within which boids alert their neighbors
    pub alert_radius: f32,
    /// Last time cleanup was performed
    last_cleanup: Instant,
    /// How often to cleanup expired entries (in seconds)
    cleanup_interval: f32,
}

/// Data structure for tracking boid aggression
#[derive(Debug, Clone)]
pub struct BoidAggressionData {
    /// The player entity that attacked this boid
    pub attacker: Entity,
    /// When the attack occurred
    pub attack_time: Instant,
    /// Whether this boid has alerted its neighbors
    pub alert_sent: bool,
}

impl FromWorld for BoidAggression {
    fn from_world(world: &mut World) -> Self {
        let config = world.resource::<PhysicsConfig>();
        Self {
            boid_aggression: HashMap::new(),
            aggression_duration: config.boid_aggression_memory_duration,
            alert_radius: config.boid_aggression_alert_radius,
            last_cleanup: Instant::now(),
            cleanup_interval: 1.0, // Cleanup every second instead of every frame
        }
    }
}

impl BoidAggression {
    /// Record that a boid was attacked by a player
    pub fn record_attack(&mut self, boid: Entity, attacker: Entity) {
        // Enforce maximum size to prevent unbounded growth
        const MAX_TRACKED_BOIDS: usize = 1000;

        // If we're at capacity and this is a new boid, remove the oldest entry
        if self.boid_aggression.len() >= MAX_TRACKED_BOIDS
            && !self.boid_aggression.contains_key(&boid)
        {
            // Find and remove the oldest entry
            if let Some(&oldest_boid) = self
                .boid_aggression
                .iter()
                .min_by_key(|(_, data)| data.attack_time)
                .map(|(entity, _)| entity)
            {
                self.boid_aggression.remove(&oldest_boid);
            }
        }

        self.boid_aggression.insert(
            boid,
            BoidAggressionData {
                attacker,
                attack_time: Instant::now(),
                alert_sent: false,
            },
        );
    }

    /// Get the attacker of a boid (if any)
    pub fn get_attacker(&self, boid: Entity) -> Option<Entity> {
        self.boid_aggression.get(&boid).map(|data| data.attacker)
    }

    /// Check if a boid is currently aggressive (has recent attacker)
    pub fn is_aggressive(&self, boid: Entity) -> bool {
        if let Some(data) = self.boid_aggression.get(&boid) {
            data.attack_time.elapsed() < self.aggression_duration
        } else {
            false
        }
    }

    /// Mark that a boid has sent an alert to its neighbors
    pub fn mark_alert_sent(&mut self, boid: Entity) {
        if let Some(data) = self.boid_aggression.get_mut(&boid) {
            data.alert_sent = true;
        }
    }

    /// Check if a boid needs to send an alert to its neighbors
    pub fn needs_alert(&self, boid: Entity) -> bool {
        if let Some(data) = self.boid_aggression.get(&boid) {
            self.is_aggressive(boid) && !data.alert_sent
        } else {
            false
        }
    }

    /// Clean up expired aggression entries (only runs periodically)
    pub fn cleanup_expired(&mut self) {
        let now = Instant::now();

        // Only cleanup if enough time has passed
        if now.duration_since(self.last_cleanup).as_secs_f32() < self.cleanup_interval {
            return;
        }

        self.last_cleanup = now;
        self.boid_aggression
            .retain(|_, data| now.duration_since(data.attack_time) < self.aggression_duration);
    }
}

// Re-export for convenience
pub use bevy_rapier2d::prelude::{
    ActiveEvents, Collider, CollisionEvent, ExternalForce, ExternalImpulse,
    RapierPhysicsPlugin, RigidBody, Sensor, Velocity,
};

#[cfg(debug_assertions)]
pub use bevy_rapier2d::prelude::RapierDebugRenderPlugin;

/// System sets for explicit ordering
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum PhysicsSet {
    /// Input collection and processing
    Input,
    /// AI decision making
    AI,
    /// Movement and physics updates
    Movement,
    /// Weapon systems and shooting
    Combat,
    /// Collision detection and response
    Collision,
    /// Resource management (pooling, cleanup)
    ResourceManagement,
    /// Network synchronization
    NetworkSync,
}

/// Physics plugin that sets up Rapier2D and physics systems
pub struct PhysicsPlugin {
    /// Whether to enable debug rendering (requires full rendering setup)
    pub enable_debug_render: bool,
}

impl Default for PhysicsPlugin {
    fn default() -> Self {
        Self {
            enable_debug_render: true,
        }
    }
}

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        // Initialize configuration
        let physics_config = PhysicsConfig::default();
        let pool_size = physics_config.projectile_pool_size;
        let collider_radius = physics_config.projectile_collider_radius;

        app
            // Add Rapier2D physics plugin with no gravity for top-down space game
            .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0));

        // Only add debug render plugin in debug builds and if enabled
        #[cfg(debug_assertions)]
        if self.enable_debug_render {
            app.add_plugins(RapierDebugRenderPlugin::default());
        }

        app
            // Add configuration resources
            .insert_resource(physics_config)
            .init_resource::<MonitoringConfig>()
            .init_resource::<ArenaConfig>()
            .init_resource::<PlayerAggression>()
            .init_resource::<BoidAggression>()
            .init_resource::<PhysicsBuffers>()
            .insert_resource(ProjectilePool::new(
                ProjectileTemplate { collider_radius },
                pool_size,
            ))
            .insert_resource(BoidProjectilePool::new(
                BoidProjectileTemplate { collider_radius },
                200, // Smaller pool for boids
            ))
            .add_systems(
                Startup,
                (
                    setup_arena,
                    setup_projectile_pool,
                    setup_boid_projectile_pool,
                )
                    .chain(),
            )
            // Configure system sets with explicit ordering
            .configure_sets(
                FixedUpdate,
                (
                    PhysicsSet::Input,
                    PhysicsSet::AI
                        .after(PhysicsSet::Input)
                        .after(SpatialGridSet::Update),
                    PhysicsSet::Movement.after(PhysicsSet::AI),
                    PhysicsSet::Combat.after(PhysicsSet::Movement),
                    PhysicsSet::Collision.after(PhysicsSet::Combat),
                    PhysicsSet::ResourceManagement.after(PhysicsSet::Collision),
                    PhysicsSet::NetworkSync.after(PhysicsSet::ResourceManagement),
                )
                    .chain()
                    .after(bevy_rapier2d::plugin::PhysicsSet::SyncBackend),
            )
            // Add systems to appropriate sets
            .add_systems(
                FixedUpdate,
                (
                    player_input_system.in_set(PhysicsSet::Input),
                    ai_player_system.in_set(PhysicsSet::AI),
                    swarm_communication_system
                        .in_set(PhysicsSet::AI)
                        .in_set(SpatialGridSet::Read),
                    player_movement_system.in_set(PhysicsSet::Movement),
                    projectile_system.in_set(PhysicsSet::Movement),
                    collision_system.in_set(PhysicsSet::Collision),
                    return_projectiles_to_pool.in_set(PhysicsSet::ResourceManagement),
                    cleanup_system.in_set(PhysicsSet::ResourceManagement),
                ),
            )
            // Combat system runs in Update for responsive shooting
            .add_systems(
                Update,
                (
                    shooting_system.in_set(PhysicsSet::Combat),
                    boid_shooting_system.in_set(PhysicsSet::Combat),
                    monitor_pool_health,
                    monitor_boid_pool_health,
                    cleanup_player_aggression,
                    cleanup_boid_aggression,
                ),
            );
    }
}

/// Arena configuration
#[derive(Resource)]
pub struct ArenaConfig {
    pub width: f32,
    pub height: f32,
    pub wall_thickness: f32,
}

impl FromWorld for ArenaConfig {
    fn from_world(world: &mut World) -> Self {
        let game_config = &*boid_wars_shared::GAME_CONFIG;
        let physics_config = world.resource::<PhysicsConfig>();
        Self {
            width: game_config.game_width,
            height: game_config.game_height,
            wall_thickness: physics_config.arena_wall_thickness,
        }
    }
}

/// Collision groups for different entity types
///
/// ## Group Allocation:
/// - `GROUP_1`: Players - can be hit by all projectiles, collide with walls and other players
/// - `GROUP_2`: Player projectiles - hit players, boids, and walls
/// - `GROUP_3`: Walls - block all entities and projectiles
/// - `GROUP_4`: Boids - can be hit by player projectiles, collide with walls and other boids
/// - `GROUP_5`: Boid projectiles - only hit players and walls (not other boids)
/// - `GROUP_6-32`: Reserved for future use (power-ups, obstacles, etc.)
pub struct GameCollisionGroups {
    pub players: Group,
    pub projectiles: Group,
    pub walls: Group,
    pub boids: Group,
    pub boid_projectiles: Group, // Separate group for boid projectiles
}

impl Default for GameCollisionGroups {
    fn default() -> Self {
        Self {
            players: Group::GROUP_1,
            projectiles: Group::GROUP_2,
            walls: Group::GROUP_3,
            boids: Group::GROUP_4,
            boid_projectiles: Group::GROUP_5, // New group for boid projectiles
        }
    }
}

impl GameCollisionGroups {
    pub fn player() -> bevy_rapier2d::geometry::CollisionGroups {
        let groups = Self::default();
        bevy_rapier2d::geometry::CollisionGroups::new(
            groups.players,
            groups.players | groups.projectiles | groups.walls | groups.boid_projectiles, // Players collide with both types of projectiles
        )
    }

    pub fn projectile() -> bevy_rapier2d::geometry::CollisionGroups {
        let groups = Self::default();
        bevy_rapier2d::geometry::CollisionGroups::new(
            groups.projectiles,
            groups.players | groups.walls | groups.boids, // Projectiles hit players, walls, and boids
        )
    }

    pub fn wall() -> bevy_rapier2d::geometry::CollisionGroups {
        let groups = Self::default();
        bevy_rapier2d::geometry::CollisionGroups::new(
            groups.walls,
            groups.players | groups.projectiles | groups.boids,
        )
    }

    pub fn boid() -> bevy_rapier2d::geometry::CollisionGroups {
        let groups = Self::default();
        bevy_rapier2d::geometry::CollisionGroups::new(
            groups.boids,
            groups.projectiles | groups.walls | groups.boids, // Boids only collide with player projectiles, not boid projectiles
        )
    }

    pub fn boid_projectile() -> bevy_rapier2d::geometry::CollisionGroups {
        let groups = Self::default();
        bevy_rapier2d::geometry::CollisionGroups::new(
            groups.boid_projectiles,
            groups.players | groups.walls, // Boid projectiles only hit players and walls, not other boids
        )
    }
}

/// Player component with physics and game stats
#[derive(Component, Clone, Debug)]
pub struct Player {
    pub player_id: u64,
    pub health: f32,
    pub max_health: f32,
    pub thrust_force: f32,
    pub turn_rate: f32,
    pub forward_speed_multiplier: f32,
    pub weapon_cooldown: Timer,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            player_id: 0,
            health: 100.0,
            max_health: 100.0,
            thrust_force: 50000.0,
            turn_rate: 5.0,
            forward_speed_multiplier: 1.5,
            weapon_cooldown: Timer::new(Duration::from_millis(250), TimerMode::Once),
        }
    }
}

/// Player input component for twin-stick controls
#[derive(Component, Clone, Debug, Default, Serialize, Deserialize)]
pub struct PlayerInput {
    pub movement: Vec2,      // Normalized movement vector
    pub aim_direction: Vec2, // Normalized aim direction
    pub thrust: f32,         // 0.0 to 1.0
    pub shooting: bool,
    pub input_sequence: u32, // For network synchronization
}

/// AI player component for automated testing
#[derive(Component, Clone, Debug)]
pub struct AIPlayer {
    pub ai_type: AIType,
    pub behavior_timer: f32,
    pub target_position: Vec2,
    pub shoot_timer: f32,
}

impl Default for AIPlayer {
    fn default() -> Self {
        Self {
            ai_type: AIType::Circler,
            behavior_timer: 0.0,
            target_position: Vec2::ZERO,
            shoot_timer: 0.0,
        }
    }
}

/// Different AI behavior types for testing
#[derive(Clone, Debug, Copy)]
pub enum AIType {
    Circler, // Moves in circles
    Bouncer, // Bounces around randomly
    Shooter, // Focuses on shooting
    Chaser,  // Chases other players
}

/// Ship physics properties
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Ship {
    pub facing_direction: Vec2,
    pub max_speed: f32,
    pub acceleration: f32,
    pub deceleration: f32,
    pub angular_velocity: f32,
}

impl Default for Ship {
    fn default() -> Self {
        Self {
            facing_direction: Vec2::Y,
            max_speed: 800.0,
            acceleration: 800.0,
            deceleration: 400.0,
            angular_velocity: 0.0,
        }
    }
}

/// Projectile component
#[derive(Component, Clone, Debug)]
pub struct Projectile {
    pub damage: f32,
    pub owner: Option<Entity>,
    pub projectile_type: ProjectileType,
    pub lifetime: Timer,
    pub speed: f32,
}

/// Different projectile types for weapon variety
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ProjectileType {
    Basic,
    Plasma,
    Laser,
}

/// Weapon statistics
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct WeaponStats {
    pub damage: f32,
    pub fire_rate: f32,
    pub projectile_speed: f32,
    pub projectile_lifetime: Duration,
    pub spread: f32,
}

impl Default for WeaponStats {
    fn default() -> Self {
        Self {
            damage: 10.0,            // Updated for combat design: players deal 10 damage
            fire_rate: 8.0,          // Increased from 4.0 for faster firing
            projectile_speed: 900.0, // Increased from 600.0 for faster bullets
            projectile_lifetime: Duration::from_secs(3),
            spread: 0.0,
        }
    }
}

/// Type alias for our projectile pool
pub type ProjectilePool = BoundedPool<ProjectileTemplate>;

/// Type alias for boid projectile pool
pub type BoidProjectilePool = BoundedPool<BoidProjectileTemplate>;

/// Template for spawning projectiles
#[derive(Component, Clone)]
pub struct ProjectileTemplate {
    pub collider_radius: f32,
}

/// Template for spawning boid projectiles
#[derive(Component, Clone)]
pub struct BoidProjectileTemplate {
    pub collider_radius: f32,
}

/// Setup projectile pool with pre-spawned projectiles
fn setup_projectile_pool(
    mut commands: Commands,
    mut pool: ResMut<ProjectilePool>,
    config: Res<PhysicsConfig>,
) {
    // Pre-spawn initial batch of projectiles
    pool.pre_spawn(
        &mut commands,
        config.projectile_pool_initial_spawn,
        |cmds, template| {
            let mut timer = Timer::from_seconds(1.0, TimerMode::Once);
            timer.pause();

            cmds.spawn((
                // Core components that won't change
                RigidBody::Dynamic,
                Collider::ball(template.collider_radius),
                Sensor,
                GameCollisionGroups::projectile(),
                bevy_rapier2d::dynamics::GravityScale(0.0),
                Name::new(POOLED_PROJECTILE_NAME),
                // Position far off-screen
                Transform::from_translation(config.projectile_pool_offscreen_position),
                GlobalTransform::default(),
                // Placeholder components that will be updated when used
                Projectile {
                    damage: 0.0,
                    owner: None, // No owner for pooled projectiles
                    projectile_type: ProjectileType::Basic,
                    lifetime: timer,
                    speed: 0.0,
                },
                Velocity::zero(),
                ProjectileTemplate {
                    collider_radius: template.collider_radius,
                },
            ))
            .id()
        },
    );

    let _status = pool.status();
}

/// Setup boid projectile pool with pre-spawned projectiles
fn setup_boid_projectile_pool(
    mut commands: Commands,
    mut pool: ResMut<BoidProjectilePool>,
    config: Res<PhysicsConfig>,
) {
    // Pre-spawn initial batch of boid projectiles
    pool.pre_spawn(
        &mut commands,
        50, // Initial spawn count for boids
        |cmds, template| {
            let mut timer = Timer::from_seconds(2.0, TimerMode::Once);
            timer.pause();

            cmds.spawn((
                // Core components that won't change
                RigidBody::Dynamic,
                Collider::ball(template.collider_radius),
                Sensor,
                GameCollisionGroups::boid_projectile(),
                bevy_rapier2d::dynamics::GravityScale(0.0),
                Name::new(POOLED_BOID_PROJECTILE_NAME),
                // Position far off-screen
                Transform::from_translation(config.projectile_pool_offscreen_position),
                GlobalTransform::default(),
                // Placeholder components that will be updated when used
                Projectile {
                    damage: 5.0, // Boid projectiles deal 5 damage
                    owner: None, // No owner for pooled projectiles
                    projectile_type: ProjectileType::Basic,
                    lifetime: timer,
                    speed: 400.0, // Slower than player projectiles
                },
                Velocity::zero(),
                BoidProjectileTemplate {
                    collider_radius: template.collider_radius,
                },
            ))
            .id()
        },
    );

    let _status = pool.status();
}

/// Setup the arena with walls - using top-left origin like network coordinates
fn setup_arena(mut commands: Commands, arena_config: Res<ArenaConfig>) {
    let collision_groups = GameCollisionGroups::default();

    // Top wall (y = 0)
    commands.spawn((
        RigidBody::Fixed,
        Collider::cuboid(arena_config.width / 2.0, arena_config.wall_thickness / 2.0),
        Transform::from_xyz(
            arena_config.width / 2.0,
            -arena_config.wall_thickness / 2.0,
            0.0,
        ),
        bevy_rapier2d::geometry::CollisionGroups::new(collision_groups.walls, Group::ALL),
        Name::new(TOP_WALL_NAME),
    ));

    // Bottom wall (y = height)
    commands.spawn((
        RigidBody::Fixed,
        Collider::cuboid(arena_config.width / 2.0, arena_config.wall_thickness / 2.0),
        Transform::from_xyz(
            arena_config.width / 2.0,
            arena_config.height + arena_config.wall_thickness / 2.0,
            0.0,
        ),
        bevy_rapier2d::geometry::CollisionGroups::new(collision_groups.walls, Group::ALL),
        Name::new(BOTTOM_WALL_NAME),
    ));

    // Left wall (x = 0)
    commands.spawn((
        RigidBody::Fixed,
        Collider::cuboid(arena_config.wall_thickness / 2.0, arena_config.height / 2.0),
        Transform::from_xyz(
            -arena_config.wall_thickness / 2.0,
            arena_config.height / 2.0,
            0.0,
        ),
        bevy_rapier2d::geometry::CollisionGroups::new(collision_groups.walls, Group::ALL),
        Name::new(LEFT_WALL_NAME),
    ));

    // Right wall (x = width)
    commands.spawn((
        RigidBody::Fixed,
        Collider::cuboid(arena_config.wall_thickness / 2.0, arena_config.height / 2.0),
        Transform::from_xyz(
            arena_config.width + arena_config.wall_thickness / 2.0,
            arena_config.height / 2.0,
            0.0,
        ),
        bevy_rapier2d::geometry::CollisionGroups::new(collision_groups.walls, Group::ALL),
        Name::new(RIGHT_WALL_NAME),
    ));
}

/// AI system for automated testing
fn ai_player_system(
    mut ai_players: Query<(&mut PlayerInput, &mut AIPlayer, &Transform), With<Player>>,
    other_players: Query<&Transform, (With<Player>, Without<AIPlayer>)>,
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

/// System to process player input and set velocity directly
fn player_input_system(
    mut player_query: Query<(
        &mut PlayerInput,
        &Player,
        &mut bevy_rapier2d::dynamics::Velocity,
        &Transform,
    )>,
    time: Res<Time>,
    mut debug_timer: Local<f32>,
) {
    *debug_timer += time.delta_secs();

    for (input, player, mut velocity, transform) in player_query.iter_mut() {
        // Store old velocity for comparison
        let _old_velocity = velocity.linvel;

        // Set velocity directly like the old network system did
        if input.thrust > 0.0 {
            let movement_direction = input.movement.normalize_or_zero();
            let target_speed = 200.0; // pixels/second - similar to old system

            // Set velocity directly
            velocity.linvel = movement_direction * target_speed;
        } else {
            // Stop when no input
            velocity.linvel = Vec2::ZERO;
        }

        // Handle rotation
        if input.aim_direction.length() > 0.1 {
            let target_angle = input.aim_direction.y.atan2(input.aim_direction.x) - PI / 2.0;
            let current_angle = transform.rotation.to_euler(EulerRot::ZYX).0;
            let angle_diff = (target_angle - current_angle + PI) % (2.0 * PI) - PI;

            velocity.angvel = angle_diff * player.turn_rate;
        }
    }

    if *debug_timer > 1.0 {
        *debug_timer = 0.0;
    }
}

/// System to handle player movement physics
fn player_movement_system(
    mut player_query: Query<(&Player, &Ship, &mut Velocity, &Transform), With<Player>>,
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

/// System to handle shooting
fn shooting_system(
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
            let spawn_offset = 50.0; // Further increased offset for player bullets
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
                    // Reset physics state
                    Transform::from_translation(projectile_spawn_pos.extend(0.0)),
                    Velocity::linear(projectile_velocity),
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
                        // Rapier2D components
                        RigidBody::Dynamic,
                        Collider::ball(config.projectile_collider_radius),
                        Sensor, // Make it a sensor so it doesn't bounce
                        GameCollisionGroups::projectile(),
                        ActiveEvents::COLLISION_EVENTS, // Enable collision events
                        Velocity::linear(projectile_velocity),
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
                boid_wars_shared::Projectile {
                    id: projectile_entity.index(), // Use entity index as ID
                    damage: weapon.damage,
                    owner_id: player.player_id,
                },
                boid_wars_shared::Position(projectile_spawn_pos),
                boid_wars_shared::Velocity(projectile_velocity),
                lightyear::prelude::server::Replicate::default(),
            ));
        }
    }
}

/// System for boid shooting behavior
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn boid_shooting_system(
    mut commands: Commands,
    mut boid_query: Query<
        (
            Entity,
            &Transform,
            &boid_wars_shared::BoidCombatStats,
            &mut boid_wars_shared::BoidCombatState,
            &boid_wars_shared::Position,
        ),
        With<boid_wars_shared::Boid>,
    >,
    player_query: Query<(Entity, &boid_wars_shared::Position), With<boid_wars_shared::Player>>,
    boid_aggression: Res<BoidAggression>,
    spatial_grid: Res<crate::spatial_grid::SpatialGrid>,
    mut boid_pool: ResMut<BoidProjectilePool>,
    time: Res<Time>,
    config: Res<PhysicsConfig>,
) {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    for (boid_entity, transform, combat_stats, mut combat_state, boid_pos) in boid_query.iter_mut() {
        // Update shooting timer
        combat_state.last_shot_time += time.delta_secs();

        // Check if boid can shoot (cooldown finished)
        let cooldown_time = 1.0 / combat_stats.fire_rate; // Convert fire rate to cooldown
        if combat_state.last_shot_time < cooldown_time {
            continue;
        }

        // Find target player
        let target_pos = find_boid_target(
            boid_entity,
            boid_pos,
            &player_query,
            &boid_aggression,
            &spatial_grid,
            combat_stats.aggression_range,
        );

        if let Some(target_position) = target_pos {
            // Reset shooting timer
            combat_state.last_shot_time = 0.0;

            // Calculate aim direction with spread
            let base_direction = (target_position - boid_pos.0).normalize();
            let spread_angle = rng.gen_range(-combat_stats.spread_angle..combat_stats.spread_angle);
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
            let projectile_velocity = aim_direction * combat_stats.projectile_speed;

            // Try to get a projectile from the boid pool
            let projectile_entity = if let Some(pooled_handle) = boid_pool.acquire() {
                // Update existing projectile components
                commands.entity(pooled_handle.entity).insert((
                    Projectile {
                        damage: combat_stats.damage,
                        owner: Some(boid_entity),
                        projectile_type: ProjectileType::Basic,
                        lifetime: {
                            let mut timer = Timer::new(Duration::from_secs(2), TimerMode::Once);
                            timer.unpause();
                            timer
                        },
                        speed: combat_stats.projectile_speed,
                    },
                    Transform::from_translation(projectile_spawn_pos.extend(0.0)),
                    Velocity::linear(projectile_velocity),
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
                            damage: combat_stats.damage,
                            owner: Some(boid_entity),
                            projectile_type: ProjectileType::Basic,
                            lifetime: Timer::new(Duration::from_secs(2), TimerMode::Once),
                            speed: combat_stats.projectile_speed,
                        },
                        RigidBody::Dynamic,
                        Collider::ball(config.projectile_collider_radius),
                        Sensor,
                        GameCollisionGroups::boid_projectile(),
                        ActiveEvents::COLLISION_EVENTS,
                        Velocity::linear(projectile_velocity),
                        Transform::from_translation(projectile_spawn_pos.extend(0.0)),
                        GlobalTransform::default(),
                        bevy_rapier2d::dynamics::GravityScale(0.0),
                        Name::new(BOID_PROJECTILE_NAME),
                    ))
                    .id()
            };

            // Add network components for client replication
            commands.entity(projectile_entity).insert((
                boid_wars_shared::Projectile {
                    id: projectile_entity.index(),
                    damage: combat_stats.damage,
                    owner_id: boid_entity.index() as u64, // Use boid entity as owner
                },
                boid_wars_shared::Position(projectile_spawn_pos),
                boid_wars_shared::Velocity(projectile_velocity),
                lightyear::prelude::server::Replicate::default(),
            ));
        }
    }
}

/// Find target for boid shooting
fn find_boid_target(
    boid_entity: Entity,
    boid_pos: &boid_wars_shared::Position,
    players: &Query<(Entity, &boid_wars_shared::Position), With<boid_wars_shared::Player>>,
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

/// System for swarm communication - alerts nearby boids when one is attacked
fn swarm_communication_system(
    mut boid_aggression: ResMut<BoidAggression>,
    mut buffers: ResMut<PhysicsBuffers>,
    spatial_grid: Res<crate::spatial_grid::SpatialGrid>,
    boid_query: Query<(Entity, &boid_wars_shared::Position), With<boid_wars_shared::Boid>>,
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

            // Mark this boid as having sent its alert
            boid_aggression.mark_alert_sent(alerting_boid);
        }
    }
}

/// System to update projectiles
fn projectile_system(
    mut commands: Commands,
    mut projectile_query: Query<(Entity, &mut Projectile, &Transform)>,
    time: Res<Time>,
    arena_config: Res<ArenaConfig>,
    mut debug_timer: Local<f32>,
) {
    *debug_timer += time.delta_secs();

    let _active_projectiles = projectile_query
        .iter()
        .filter(|(_, _, transform)| {
            let pos = transform.translation.truncate();
            pos.x > -500.0 && pos.y > -500.0 // Only count projectiles not in pool area
        })
        .count();

    for (entity, mut projectile, transform) in projectile_query.iter_mut() {
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

/// System to handle collisions
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn collision_system(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    mut buffers: ResMut<PhysicsBuffers>,
    mut health_queries: ParamSet<(
        Query<(&mut Player, Option<&mut boid_wars_shared::Health>)>,
        Query<&mut boid_wars_shared::Health, With<boid_wars_shared::Boid>>,
    )>,
    projectile_query: Query<&Projectile>,
    boid_entity_query: Query<Entity, With<boid_wars_shared::Boid>>,
    _obstacle_query: Query<Entity, With<boid_wars_shared::Obstacle>>,
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
                } else if _obstacle_query.get(*entity2).is_ok() {
                    // Projectile hit obstacle - despawn projectile
                    commands.entity(*entity1).insert(Despawning);
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
                } else if _obstacle_query.get(*entity1).is_ok() {
                    // Projectile hit obstacle - despawn projectile
                    commands.entity(*entity2).insert(Despawning);
                }
            }
        }
    }

    // Process player collisions
    for &(projectile_entity, player_entity, damage, owner) in &buffers.player_collision_buffer {
        // Skip if player is hitting themselves
        if let Some(owner_entity) = owner {
            if owner_entity == player_entity {
                continue;
            }
        }

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

/// Handle player death
fn handle_player_death(commands: &mut Commands, player_entity: Entity) {
    // In battle royale mode, death is permanent
    // Mark the player entity for cleanup
    commands.entity(player_entity).insert(Despawning);

    // TODO: Emit death event for UI/spectator mode
    // TODO: Update game state for battle royale (track remaining players)
    // TODO: Trigger death visual/audio effects

    info!("Player {:?} has been eliminated", player_entity);
}

/// System to return projectiles to pool instead of despawning
#[allow(clippy::type_complexity)]
fn return_projectiles_to_pool(
    mut commands: Commands,
    mut player_pool: ResMut<ProjectilePool>,
    mut boid_pool: ResMut<BoidProjectilePool>,
    mut projectiles: Query<
        (
            Entity,
            &mut Transform,
            &mut Velocity,
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
                velocity.linvel = Vec2::ZERO;
                velocity.angvel = 0.0;
                projectile.lifetime.reset();
                projectile.lifetime.pause(); // Pause the timer so it doesn't tick while pooled

                // Remove network components to stop replication (with error handling)
                if let Ok(mut entity_commands) = commands.get_entity(entity) {
                    entity_commands.remove::<boid_wars_shared::Projectile>();
                    entity_commands.remove::<boid_wars_shared::Position>();
                    entity_commands.remove::<boid_wars_shared::Velocity>();
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

/// System to clean up orphaned entities
fn cleanup_system(
    mut commands: Commands,
    projectile_query: Query<Entity, (With<Projectile>, Without<RigidBody>)>,
    despawning_entities: Query<Entity, (With<Despawning>, Without<Projectile>)>,
) {
    // Clean up projectiles that lost their physics body
    for entity in projectile_query.iter() {
        commands.entity(entity).despawn();
    }

    // Clean up non-projectile entities marked for despawning
    for entity in despawning_entities.iter() {
        commands.entity(entity).despawn();
    }
}

/// System to monitor pool health and performance
fn monitor_pool_health(
    pool: Res<ProjectilePool>,
    mut debug_timer: Local<f32>,
    time: Res<Time>,
    config: Res<MonitoringConfig>,
) {
    *debug_timer += time.delta_secs();

    // Log pool statistics at configured interval
    if *debug_timer > config.pool_health_check_interval {
        let status = pool.status();
        let utilization = (status.active as f32 / status.total.max(1) as f32) * 100.0;

        if utilization > config.pool_high_utilization_threshold {
            warn!("[PLAYER POOL] High utilization detected! Consider increasing pool size or reducing projectile spawn rate");
        }

        *debug_timer = 0.0;
    }
}

/// Monitor boid projectile pool health
fn monitor_boid_pool_health(
    pool: Res<BoidProjectilePool>,
    mut debug_timer: Local<f32>,
    time: Res<Time>,
    config: Res<MonitoringConfig>,
) {
    *debug_timer += time.delta_secs();

    // Log pool statistics at configured interval
    if *debug_timer > config.pool_health_check_interval {
        let status = pool.status();
        let utilization = (status.active as f32 / status.total.max(1) as f32) * 100.0;

        if utilization > config.pool_high_utilization_threshold {
            warn!("[BOID POOL] High utilization detected! Consider increasing pool size or reducing boid projectile spawn rate");
        }

        *debug_timer = 0.0;
    }
}

/// Spawn a player with physics components
pub fn spawn_player(commands: &mut Commands, player_id: u64, spawn_position: Vec2) -> Entity {
    let collision_groups = GameCollisionGroups::default();

    // Spawn with minimal components first
    let entity = commands
        .spawn((
            RigidBody::Dynamic,
            Collider::cuboid(4.0, 4.0), // Match 8x8 visual size
            Transform::from_translation(spawn_position.extend(0.0)),
            GlobalTransform::default(),
        ))
        .id();

    // Add physics properties
    commands.entity(entity).insert((
        Velocity::default(),
        ExternalForce::default(),
        ExternalImpulse::default(),
        bevy_rapier2d::dynamics::Sleeping::disabled(),
        Damping {
            linear_damping: 2.0,
            angular_damping: 5.0,
        },
        bevy_rapier2d::dynamics::AdditionalMassProperties::Mass(10.0),
        bevy_rapier2d::dynamics::GravityScale(0.0),
    ));

    // Add game components
    commands.entity(entity).insert((
        Player {
            player_id,
            ..Default::default()
        },
        PlayerInput::default(),
        Ship::default(),
        WeaponStats::default(),
        bevy_rapier2d::geometry::CollisionGroups::new(
            collision_groups.players,
            collision_groups.projectiles | collision_groups.walls,
        ),
        Name::new(format!("Player {player_id}")),
    ));

    entity
}

/// Spawn an AI player for testing
pub fn spawn_ai_player(
    commands: &mut Commands,
    player_id: u64,
    spawn_position: Vec2,
    ai_type: AIType,
) -> Entity {
    let collision_groups = GameCollisionGroups::default();

    // Note: spawn_position is in physics coordinates (centered)
    // The sync system will convert to network coordinates
    // Spawn with minimal components first
    let entity = commands
        .spawn((
            // Core physics components
            RigidBody::Dynamic,
            Collider::cuboid(4.0, 4.0), // Match 8x8 visual size
            Transform::from_translation(spawn_position.extend(0.0)),
            GlobalTransform::default(),
        ))
        .id();

    // Add physics properties
    commands.entity(entity).insert((
        Velocity::default(),
        ExternalForce::default(),
        ExternalImpulse::default(),
        bevy_rapier2d::dynamics::Sleeping::disabled(),
    ));

    // Add game components
    commands.entity(entity).insert((
        // Networking components for replication
        boid_wars_shared::Player {
            id: player_id,
            name: format!("AI {player_id} ({ai_type:?})"),
        },
        boid_wars_shared::Position(bevy::math::Vec2::ZERO),
        boid_wars_shared::Velocity(bevy::math::Vec2::ZERO),
        boid_wars_shared::Rotation { angle: 0.0 },
        boid_wars_shared::Health::default(),
        // Physics components
        Player {
            player_id,
            ..Default::default()
        },
        PlayerInput::default(),
        Ship::default(),
        WeaponStats::default(),
        AIPlayer {
            ai_type,
            ..Default::default()
        },
    ));

    // Add physics modifiers
    commands.entity(entity).insert((
        bevy_rapier2d::geometry::CollisionGroups::new(
            collision_groups.players,
            collision_groups.projectiles | collision_groups.walls,
        ),
        Damping {
            linear_damping: 0.5, // Reduced damping to allow movement
            angular_damping: 1.0,
        },
        bevy_rapier2d::dynamics::AdditionalMassProperties::Mass(10.0),
        bevy_rapier2d::dynamics::GravityScale(0.0),
        lightyear::prelude::server::Replicate::default(),
        Name::new(format!("AI Player {player_id} ({ai_type:?})")),
    ));

    entity
}


/// System to clean up expired player aggression entries
fn cleanup_player_aggression(mut player_aggression: ResMut<PlayerAggression>) {
    player_aggression.cleanup_expired();
}

/// System to clean up expired boid aggression entries
fn cleanup_boid_aggression(mut boid_aggression: ResMut<BoidAggression>) {
    boid_aggression.cleanup_expired();
}

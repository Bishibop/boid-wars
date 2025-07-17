use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use boid_wars_shared;
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;
use std::time::{Duration, Instant};

// Constants
pub const BOID_RADIUS: f32 = 4.0;

/// Marker component to indicate entity is being despawned
#[derive(Component)]
pub struct Despawning;

// Re-export for convenience
pub use bevy_rapier2d::prelude::{
    ActiveEvents, Collider, CollisionEvent, ExternalForce, ExternalImpulse,
    RapierDebugRenderPlugin, RapierPhysicsPlugin, RigidBody, Sensor, Velocity,
};

/// Physics plugin that sets up Rapier2D and physics systems
pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app
            // Add Rapier2D physics plugin with no gravity for top-down space game
            .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
            .add_plugins(RapierDebugRenderPlugin::default())
            // Physics configuration is now handled by the plugin directly
            .init_resource::<ArenaConfig>()
            .init_resource::<ProjectilePool>()
            .add_systems(Startup, (setup_arena, setup_projectile_pool).chain())
            .add_systems(Update, shooting_system) // Move to Update for responsive input
            .add_systems(
                FixedUpdate,
                (
                    // Player and AI systems
                    (
                        ai_player_system,
                        player_input_system,
                        player_movement_system,
                    )
                        .chain(),
                    // Projectile lifecycle systems
                    (projectile_system, collision_system),
                    // Pool management before cleanup
                    return_projectiles_to_pool,
                    cleanup_system,
                    // Network sync after all physics updates
                    (sync_physics_to_network, sync_projectile_physics_to_network),
                )
                    .chain(),
            )
            .add_systems(Update, monitor_pool_health);
    }
}

/// Arena configuration
#[derive(Resource)]
pub struct ArenaConfig {
    pub width: f32,
    pub height: f32,
    pub wall_thickness: f32,
}

impl Default for ArenaConfig {
    fn default() -> Self {
        let game_config = &*boid_wars_shared::GAME_CONFIG;
        Self {
            width: game_config.game_width,
            height: game_config.game_height,
            wall_thickness: 25.0, // Scaled down proportionally
        }
    }
}

/// Collision groups for different entity types
pub struct GameCollisionGroups {
    pub players: Group,
    pub projectiles: Group,
    pub walls: Group,
    pub boids: Group, // Future use
}

impl Default for GameCollisionGroups {
    fn default() -> Self {
        Self {
            players: Group::GROUP_1,
            projectiles: Group::GROUP_2,
            walls: Group::GROUP_3,
            boids: Group::GROUP_4,
        }
    }
}

impl GameCollisionGroups {
    pub fn player() -> bevy_rapier2d::geometry::CollisionGroups {
        let groups = Self::default();
        bevy_rapier2d::geometry::CollisionGroups::new(
            groups.players,
            groups.players | groups.projectiles | groups.walls, // Players now collide with other players
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
            groups.projectiles | groups.walls | groups.boids, // Boids collide with projectiles, walls, and each other
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
            thrust_force: 50000.0, // High thrust for responsive movement
            turn_rate: 5.0,
            forward_speed_multiplier: 1.5,
            weapon_cooldown: Timer::new(Duration::from_millis(100), TimerMode::Once),
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
            max_speed: 800.0, // High max speed for responsive movement
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
    pub owner: Entity,
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

/// Projectile physics properties
#[derive(Component, Clone, Debug)]
pub struct ProjectilePhysics {
    pub velocity: Vec2,
    pub spawn_time: Instant,
    pub max_lifetime: Duration,
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
            damage: 25.0,
            fire_rate: 4.0, // Shots per second
            projectile_speed: 600.0,
            projectile_lifetime: Duration::from_secs(3),
            spread: 0.0,
        }
    }
}

/// Object pool for projectiles to avoid allocation/deallocation
#[derive(Resource)]
pub struct ProjectilePool {
    available: Vec<Entity>,
    active: std::collections::HashSet<Entity>,
    _pool_size: usize,
}

impl Default for ProjectilePool {
    fn default() -> Self {
        Self {
            available: Vec::with_capacity(500),
            active: std::collections::HashSet::with_capacity(500),
            _pool_size: 500,
        }
    }
}

impl ProjectilePool {
    pub fn get_projectile(&mut self) -> Option<Entity> {
        if let Some(entity) = self.available.pop() {
            self.active.insert(entity);
            Some(entity)
        } else {
            None
        }
    }

    pub fn return_projectile(&mut self, entity: Entity) {
        if self.active.remove(&entity) {
            self.available.push(entity);
        }
    }
}

/// Setup projectile pool with pre-spawned projectiles
fn setup_projectile_pool(mut commands: Commands, mut pool: ResMut<ProjectilePool>) {
    info!("[POOL] Pre-spawning 100 projectiles for pool");

    // Create a dummy entity to use as placeholder owner
    let dummy_entity = commands
        .spawn((
            Name::new("Projectile Pool Dummy Owner"),
            Transform::from_translation(Vec3::new(-2000.0, -2000.0, -200.0)), // Far off-screen
        ))
        .id();

    // Pre-spawn projectiles for the pool
    for i in 0..100 {
        let mut timer = Timer::from_seconds(1.0, TimerMode::Once);
        timer.pause(); // Start paused

        let entity = commands
            .spawn((
                // Core components that won't change
                RigidBody::Dynamic,
                Collider::ball(2.0),
                Sensor,
                GameCollisionGroups::projectile(),
                bevy_rapier2d::dynamics::GravityScale(0.0),
                Name::new(format!("Pooled Projectile {i}")),
                // Position far off-screen
                Transform::from_translation(Vec3::new(-1000.0, -1000.0, -100.0)),
                GlobalTransform::default(),
                // Placeholder components that will be updated when used
                Projectile {
                    damage: 0.0,
                    owner: dummy_entity, // Use dummy entity instead of PLACEHOLDER
                    projectile_type: ProjectileType::Basic,
                    lifetime: timer,
                    speed: 0.0,
                },
                ProjectilePhysics {
                    velocity: Vec2::ZERO,
                    spawn_time: Instant::now(),
                    max_lifetime: Duration::from_secs(1),
                },
                Velocity::zero(),
            ))
            .id();

        pool.available.push(entity);
    }

    info!(
        "[POOL] Projectile pool initialized with {} projectiles",
        pool.available.len()
    );
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
        Name::new("Top Wall"),
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
        Name::new("Bottom Wall"),
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
        Name::new("Left Wall"),
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
        Name::new("Right Wall"),
    ));

    // Removed arena setup log
}

/// AI system for automated testing
fn ai_player_system(
    mut ai_players: Query<(&mut PlayerInput, &mut AIPlayer, &Transform), With<Player>>,
    other_players: Query<&Transform, (With<Player>, Without<AIPlayer>)>,
    time: Res<Time>,
    arena_config: Res<ArenaConfig>,
) {
    for (mut input, mut ai, transform) in ai_players.iter_mut() {
        ai.behavior_timer += time.delta_secs();
        ai.shoot_timer += time.delta_secs();

        let pos = transform.translation.truncate();

        match ai.ai_type {
            AIType::Circler => {
                // Move in a circle around starting position
                let circle_radius = 100.0;
                let circle_speed = 1.0; // radians per second

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

                // Removed AI debug logs
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

                // Removed AI debug logs
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
                    dist_a.partial_cmp(&dist_b).unwrap()
                }) {
                    let target_pos = nearest_player.translation.truncate();
                    input.movement = (target_pos - pos).normalize_or_zero();
                    input.aim_direction = input.movement;
                    input.thrust = 1.0;

                    // Shoot at target
                    let distance = pos.distance(target_pos);
                    input.shooting = distance < 300.0 && ai.shoot_timer > 0.3;
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

        // Removed debug logs

        // Set velocity directly like the old network system did
        if input.thrust > 0.0 {
            let movement_direction = input.movement.normalize_or_zero();
            let target_speed = 200.0; // pixels/second - similar to old system

            // Set velocity directly
            velocity.linvel = movement_direction * target_speed;

            // Removed debug logs
        } else {
            // Stop when no input
            velocity.linvel = Vec2::ZERO;
            // Removed debug logs
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
) {
    for (_player, ship, mut velocity, _transform) in player_query.iter_mut() {
        // Apply damping for momentum feel
        let damping_factor = 0.98; // Higher value = less damping = faster movement
        velocity.linvel *= damping_factor;
        velocity.angvel *= damping_factor;

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
    time: Res<Time>,
) {
    // Debug: Log all players and their input state
    let player_count = player_query.iter().count();
    info!("🔍 SHOOTING: Processing {} players", player_count);

    for (entity, input, mut player, weapon, transform) in player_query.iter_mut() {
        player.weapon_cooldown.tick(time.delta());

        // Debug each player's state
        info!(
            "🔍 Player {} - shooting={} cooldown_finished={}",
            player.player_id,
            input.shooting,
            player.weapon_cooldown.finished()
        );

        if input.shooting && player.weapon_cooldown.finished() {
            info!(
                "🚀 PHYSICS: Player {} shooting! Spawning projectile",
                player.player_id
            );
            // Reset cooldown
            player.weapon_cooldown.reset();

            // Spawn projectile - offset in the aim direction to avoid self-collision
            let spawn_offset = 15.0; // Distance from player center
            let projectile_spawn_pos =
                transform.translation.truncate() + input.aim_direction * spawn_offset; // Spawn in aim direction

            let projectile_velocity = input.aim_direction * weapon.projectile_speed;

            // Debug velocity calculation
            info!(
                "🎯 PHYSICS: Player {} aim_direction=({:.1}, {:.1}) projectile_speed={:.1}",
                player.player_id,
                input.aim_direction.x,
                input.aim_direction.y,
                weapon.projectile_speed
            );

            // Monitor pool size and warn if running low
            if pool.available.len() < 10 {
                warn!(
                    "[POOL] Pool running low: {} projectiles available (active: {})",
                    pool.available.len(),
                    pool.active.len()
                );
            }

            // Try to get a projectile from the pool
            let projectile_entity = if let Some(pooled_entity) = pool.get_projectile() {
                info!(
                    "[POOL] Reusing projectile {:?} from pool ({} available)",
                    pooled_entity,
                    pool.available.len()
                );

                // Update existing projectile components
                commands.entity(pooled_entity).insert((
                    // Update projectile data
                    Projectile {
                        damage: weapon.damage,
                        owner: entity, // Use actual player entity
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
                    Velocity::linear(projectile_velocity),
                    // Re-enable collision detection
                    ActiveEvents::COLLISION_EVENTS,
                ));

                pooled_entity
            } else {
                warn!("[POOL] Pool exhausted! Spawning new projectile (this may cause performance issues)");

                // Pool is empty, spawn a new projectile
                commands
                    .spawn((
                        // Physics projectile component (server-only)
                        Projectile {
                            damage: weapon.damage,
                            owner: entity, // Use actual player entity
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
                        Collider::ball(2.0), // Small bullet collider
                        Sensor,              // Make it a sensor so it doesn't bounce
                        GameCollisionGroups::projectile(),
                        ActiveEvents::COLLISION_EVENTS, // Enable collision events
                        Velocity::linear(projectile_velocity),
                        Transform::from_translation(projectile_spawn_pos.extend(0.0)),
                        GlobalTransform::default(),
                        bevy_rapier2d::dynamics::GravityScale(0.0), // Disable gravity for projectiles
                        Name::new("Projectile"),
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

            info!(
                "✅ PHYSICS: Projectile spawned at ({:.1}, {:.1}) with velocity ({:.1}, {:.1})",
                projectile_spawn_pos.x,
                projectile_spawn_pos.y,
                projectile_velocity.x,
                projectile_velocity.y
            );
        }
    }
}

/// System to update projectiles
fn projectile_system(
    mut commands: Commands,
    mut projectile_query: Query<(Entity, &mut Projectile, &ProjectilePhysics, &Transform)>,
    time: Res<Time>,
    arena_config: Res<ArenaConfig>,
    mut debug_timer: Local<f32>,
) {
    *debug_timer += time.delta_secs();

    let active_projectiles = projectile_query
        .iter()
        .filter(|(_, _, _, transform)| {
            let pos = transform.translation.truncate();
            pos.x > -500.0 && pos.y > -500.0 // Only count projectiles not in pool area
        })
        .count();

    if *debug_timer > 5.0 {
        info!(
            "🎯 PROJECTILE SYSTEM: {} total projectiles, {} active",
            projectile_query.iter().count(),
            active_projectiles
        );
        *debug_timer = 0.0;
    }

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

/// System to handle collisions
fn collision_system(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    mut player_query: Query<&mut Player>,
    projectile_query: Query<&Projectile>,
    _boid_query: Query<Entity, With<boid_wars_shared::Boid>>,
    _obstacle_query: Query<Entity, With<boid_wars_shared::Obstacle>>,
) {
    for collision_event in collision_events.read() {
        if let CollisionEvent::Started(entity1, entity2, _) = collision_event {
            // Check if entity1 is a projectile
            if let Ok(projectile) = projectile_query.get(*entity1) {
                // Projectile hit something - mark for despawn (will be returned to pool)
                info!(
                    "[COLLISION] Projectile {:?} hit something, marking for despawn",
                    entity1
                );
                commands.entity(*entity1).insert(Despawning);

                // Check what it hit
                if let Ok(mut player) = player_query.get_mut(*entity2) {
                    // Hit a player - apply damage
                    player.health -= projectile.damage;
                    if player.health <= 0.0 {
                        handle_player_death(&mut commands, *entity2);
                    }
                }
                // Could add boid damage here if needed
            }

            // Check if entity2 is a projectile (reverse collision)
            if let Ok(projectile) = projectile_query.get(*entity2) {
                // Projectile hit something - mark for despawn (will be returned to pool)
                info!(
                    "[COLLISION] Projectile {:?} hit something, marking for despawn",
                    entity2
                );
                commands.entity(*entity2).insert(Despawning);

                // Check what it hit
                if let Ok(mut player) = player_query.get_mut(*entity1) {
                    // Hit a player - apply damage
                    player.health -= projectile.damage;
                    if player.health <= 0.0 {
                        handle_player_death(&mut commands, *entity1);
                    }
                }
                // Could add boid damage here if needed
            }
        }
    }
}

/// Handle player death
fn handle_player_death(_commands: &mut Commands, _player_entity: Entity) {
    // TODO: Implement respawn logic or game over state
    // For now, just log the death
    info!("Player died!");
}

/// System to return projectiles to pool instead of despawning
#[allow(clippy::type_complexity)]
fn return_projectiles_to_pool(
    mut commands: Commands,
    mut pool: ResMut<ProjectilePool>,
    mut projectiles: Query<
        (
            Entity,
            &mut Transform,
            &mut Velocity,
            &mut Projectile,
            Option<&Despawning>,
        ),
        With<Projectile>,
    >,
) {
    for (entity, mut transform, mut velocity, mut projectile, despawning) in projectiles.iter_mut()
    {
        // Check if this projectile should be returned to pool
        let should_return = despawning.is_some() || projectile.lifetime.finished();

        if should_return {
            // Only log occasionally to reduce spam
            if entity.index() % 10 == 0 {
                info!("[POOL] Returning projectile {:?} to pool", entity);
            }

            // Reset projectile state
            transform.translation = Vec3::new(-1000.0, -1000.0, -100.0); // Move far off-screen
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

                // Return to pool
                pool.return_projectile(entity);
            } else {
                warn!("[POOL] Failed to return projectile {:?} to pool - entity may have been despawned", entity);
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
fn monitor_pool_health(pool: Res<ProjectilePool>, mut debug_timer: Local<f32>, time: Res<Time>) {
    *debug_timer += time.delta_secs();

    // Log pool statistics every 10 seconds
    if *debug_timer > 10.0 {
        let available = pool.available.len();
        let active = pool.active.len();
        let total = available + active;
        let utilization = (active as f32 / total as f32) * 100.0;

        info!(
            "[POOL] Health check - Available: {}, Active: {}, Utilization: {:.1}%",
            available, active, utilization
        );

        if utilization > 80.0 {
            warn!("[POOL] High utilization detected! Consider increasing pool size or reducing projectile spawn rate");
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
        bevy_rapier2d::dynamics::GravityScale(0.0), // Set reasonable mass
        bevy_rapier2d::dynamics::GravityScale(0.0),
        lightyear::prelude::server::Replicate::default(),
        Name::new(format!("AI Player {player_id} ({ai_type:?})")),
    ));

    // Removed AI spawn log

    entity
}

/// Sync physics Transform positions to networked Position components - NO CONVERSION NEEDED!
fn sync_physics_to_network(
    mut query: Query<
        (
            &Transform,
            &mut boid_wars_shared::Position,
            &bevy_rapier2d::dynamics::Velocity,
            &mut boid_wars_shared::Velocity,
            &Player,
        ),
        With<Player>,
    >,
    time: Res<Time>,
    mut debug_timer: Local<f32>,
) {
    *debug_timer += time.delta_secs();

    // Removed debug logs

    for (transform, mut position, physics_vel, mut net_vel, _player) in query.iter_mut() {
        // Direct copy - both systems use same coordinate system now!
        let physics_pos = transform.translation.truncate();
        position.0 = physics_pos;

        // Sync velocity from physics
        net_vel.0 = physics_vel.linvel;

        // Removed debug logs
    }

    if *debug_timer > 2.0 {
        *debug_timer = 0.0;
    }
}

/// Sync projectile physics positions to networked Position components
#[allow(clippy::type_complexity)]
fn sync_projectile_physics_to_network(
    mut projectiles: Query<
        (
            &Transform,
            &mut boid_wars_shared::Position,
            &bevy_rapier2d::dynamics::Velocity,
            &mut boid_wars_shared::Velocity,
        ),
        (With<Projectile>, With<boid_wars_shared::Projectile>),
    >,
) {
    for (transform, mut position, physics_vel, mut net_vel) in projectiles.iter_mut() {
        // Sync position from physics to network
        position.0 = transform.translation.truncate();

        // Sync velocity from physics to network
        net_vel.0 = physics_vel.linvel;
    }
}

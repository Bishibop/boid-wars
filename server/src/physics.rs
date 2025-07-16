use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use std::f32::consts::PI;

// Re-export for convenience
pub use bevy_rapier2d::prelude::{
    Collider, ExternalForce, ExternalImpulse,
    RapierPhysicsPlugin, RapierDebugRenderPlugin, RigidBody, Velocity,
};

/// Physics plugin that sets up Rapier2D and physics systems
pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app
            // Add Rapier2D physics plugin
            .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
            .add_plugins(RapierDebugRenderPlugin::default())
            // Physics configuration is now handled by the plugin directly
            .init_resource::<ArenaConfig>()
            .init_resource::<ProjectilePool>()
            .add_systems(Startup, setup_arena)
            .add_systems(FixedUpdate, (
                ai_player_system,
                player_input_system,
                player_movement_system,
                shooting_system,
                projectile_system,
                collision_system,
                cleanup_system,
            ).chain());
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
        Self {
            width: 2500.0,
            height: 1500.0,
            wall_thickness: 50.0,
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
            thrust_force: 500.0,
            turn_rate: 5.0,
            forward_speed_multiplier: 1.5,
            weapon_cooldown: Timer::new(Duration::from_millis(250), TimerMode::Once),
        }
    }
}

/// Player input component for twin-stick controls
#[derive(Component, Clone, Debug, Default, Serialize, Deserialize)]
pub struct PlayerInput {
    pub movement: Vec2,        // Normalized movement vector
    pub aim_direction: Vec2,   // Normalized aim direction
    pub thrust: f32,           // 0.0 to 1.0
    pub shooting: bool,
    pub input_sequence: u32,   // For network synchronization
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
    Circler,    // Moves in circles
    Bouncer,    // Bounces around randomly
    Shooter,    // Focuses on shooting
    Chaser,     // Chases other players
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
            max_speed: 300.0,
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
    pool_size: usize,
}

impl Default for ProjectilePool {
    fn default() -> Self {
        Self {
            available: Vec::with_capacity(500),
            active: std::collections::HashSet::with_capacity(500),
            pool_size: 500,
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

/// Setup the arena with walls
fn setup_arena(mut commands: Commands, arena_config: Res<ArenaConfig>) {
    let collision_groups = GameCollisionGroups::default();
    
    // Top wall
    commands.spawn((
        RigidBody::Fixed,
        Collider::cuboid(arena_config.width / 2.0, arena_config.wall_thickness / 2.0),
        Transform::from_xyz(0.0, arena_config.height / 2.0 + arena_config.wall_thickness / 2.0, 0.0),
        bevy_rapier2d::geometry::CollisionGroups::new(
            collision_groups.walls,
            Group::ALL
        ),
        Name::new("Top Wall"),
    ));
    
    // Bottom wall
    commands.spawn((
        RigidBody::Fixed,
        Collider::cuboid(arena_config.width / 2.0, arena_config.wall_thickness / 2.0),
        Transform::from_xyz(0.0, -arena_config.height / 2.0 - arena_config.wall_thickness / 2.0, 0.0),
        bevy_rapier2d::geometry::CollisionGroups::new(
            collision_groups.walls,
            Group::ALL
        ),
        Name::new("Bottom Wall"),
    ));
    
    // Left wall
    commands.spawn((
        RigidBody::Fixed,
        Collider::cuboid(arena_config.wall_thickness / 2.0, arena_config.height / 2.0),
        Transform::from_xyz(-arena_config.width / 2.0 - arena_config.wall_thickness / 2.0, 0.0, 0.0),
        bevy_rapier2d::geometry::CollisionGroups::new(
            collision_groups.walls,
            Group::ALL
        ),
        Name::new("Left Wall"),
    ));
    
    // Right wall
    commands.spawn((
        RigidBody::Fixed,
        Collider::cuboid(arena_config.wall_thickness / 2.0, arena_config.height / 2.0),
        Transform::from_xyz(arena_config.width / 2.0 + arena_config.wall_thickness / 2.0, 0.0, 0.0),
        bevy_rapier2d::geometry::CollisionGroups::new(
            collision_groups.walls,
            Group::ALL
        ),
        Name::new("Right Wall"),
    ));
    
    info!("Arena setup complete: {}x{}", arena_config.width, arena_config.height);
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
                // Move in circles around center
                let center = Vec2::ZERO;
                let radius = 200.0;
                let speed = 1.0;
                
                let angle = ai.behavior_timer * speed;
                let target = center + Vec2::new(angle.cos(), angle.sin()) * radius;
                
                input.movement = (target - pos).normalize_or_zero();
                input.aim_direction = Vec2::new(1.0, 0.0); // Aim right
                input.thrust = 1.0;
                
                // Shoot periodically
                input.shooting = ai.shoot_timer > 0.5;
                if input.shooting {
                    ai.shoot_timer = 0.0;
                }
            }
            
            AIType::Bouncer => {
                // Bounce around randomly
                if ai.behavior_timer > 2.0 {
                    ai.behavior_timer = 0.0;
                    ai.target_position = Vec2::new(
                        (rand::random::<f32>() - 0.5) * arena_config.width * 0.8,
                        (rand::random::<f32>() - 0.5) * arena_config.height * 0.8,
                    );
                }
                
                input.movement = (ai.target_position - pos).normalize_or_zero();
                input.aim_direction = input.movement;
                input.thrust = 0.8;
                
                // Shoot randomly
                input.shooting = rand::random::<f32>() < 0.1;
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
                if let Some(nearest_player) = other_players.iter()
                    .min_by(|a, b| {
                        let dist_a = a.translation.distance(transform.translation);
                        let dist_b = b.translation.distance(transform.translation);
                        dist_a.partial_cmp(&dist_b).unwrap()
                    })
                {
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
                        (ai.behavior_timer * 0.5).sin()
                    );
                    input.thrust = 0.5;
                    input.shooting = false;
                }
            }
        }
    }
}

/// System to process player input and apply forces
fn player_input_system(
    mut player_query: Query<(&mut PlayerInput, &Player, &mut ExternalForce, &mut ExternalImpulse, &Transform)>,
    time: Res<Time>,
) {
    for (input, player, mut force, mut impulse, transform) in player_query.iter_mut() {
        // Reset forces
        force.force = Vec2::ZERO;
        force.torque = 0.0;
        
        // Calculate thrust force
        if input.thrust > 0.0 {
            let movement_direction = input.movement.normalize_or_zero();
            let facing_direction = (transform.rotation * Vec3::Y).truncate();
            
            // Forward speed boost when moving in facing direction
            let forward_alignment = movement_direction.dot(facing_direction);
            let speed_multiplier = if forward_alignment > 0.0 {
                1.0 + (player.forward_speed_multiplier - 1.0) * forward_alignment
            } else {
                1.0
            };
            
            force.force = movement_direction * player.thrust_force * input.thrust * speed_multiplier;
        }
        
        // Handle rotation
        if input.aim_direction.length() > 0.1 {
            let target_angle = input.aim_direction.y.atan2(input.aim_direction.x) - PI/2.0;
            let current_angle = transform.rotation.to_euler(EulerRot::ZYX).0;
            let angle_diff = (target_angle - current_angle + PI) % (2.0 * PI) - PI;
            
            impulse.torque_impulse = angle_diff * player.turn_rate * time.delta_secs();
        }
    }
}

/// System to handle player movement physics
fn player_movement_system(
    mut player_query: Query<(&Player, &Ship, &mut Velocity, &Transform), With<Player>>,
    _time: Res<Time>,
) {
    for (_player, ship, mut velocity, _transform) in player_query.iter_mut() {
        // Apply damping for momentum feel
        let damping_factor = 0.95; // Adjust for desired momentum feel
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
    mut player_query: Query<(&PlayerInput, &mut Player, &WeaponStats, &Transform)>,
    time: Res<Time>,
) {
    for (input, mut player, weapon, transform) in player_query.iter_mut() {
        player.weapon_cooldown.tick(time.delta());
        
        if input.shooting && player.weapon_cooldown.finished() {
            // Reset cooldown
            player.weapon_cooldown.reset();
            
            // Spawn projectile
            let projectile_spawn_pos = transform.translation.truncate() + 
                (transform.rotation * Vec3::Y).truncate() * 30.0; // Offset from ship center
            
            let projectile_velocity = input.aim_direction * weapon.projectile_speed;
            let collision_groups = GameCollisionGroups::default();
            
            commands.spawn((
                Projectile {
                    damage: weapon.damage,
                    owner: Entity::PLACEHOLDER, // TODO: Get actual player entity
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
                bevy_rapier2d::geometry::CollisionGroups::new(
                    collision_groups.projectiles,
                    collision_groups.players | collision_groups.walls
                ),
                Velocity::linear(projectile_velocity),
                Transform::from_translation(projectile_spawn_pos.extend(0.0)),
                GlobalTransform::default(),
                Name::new("Projectile"),
            ));
        }
    }
}

/// System to update projectiles
fn projectile_system(
    mut commands: Commands,
    mut projectile_query: Query<(Entity, &mut Projectile, &ProjectilePhysics, &Transform)>,
    time: Res<Time>,
    arena_config: Res<ArenaConfig>,
) {
    for (entity, mut projectile, _physics, transform) in projectile_query.iter_mut() {
        // Update lifetime
        projectile.lifetime.tick(time.delta());
        
        // Despawn if lifetime expired
        if projectile.lifetime.finished() {
            commands.entity(entity).despawn();
            continue;
        }
        
        // Check world bounds
        let pos = transform.translation.truncate();
        if pos.x.abs() > arena_config.width / 2.0 || pos.y.abs() > arena_config.height / 2.0 {
            commands.entity(entity).despawn();
        }
    }
}

/// System to handle collisions
fn collision_system(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    mut player_query: Query<&mut Player>,
    projectile_query: Query<&Projectile>,
) {
    for collision_event in collision_events.read() {
        if let CollisionEvent::Started(entity1, entity2, _) = collision_event {
            // Check for player-projectile collision
            if let (Ok(mut player), Ok(projectile)) = (
                player_query.get_mut(*entity1),
                projectile_query.get(*entity2)
            ) {
                // Apply damage
                player.health -= projectile.damage;
                
                // Despawn projectile
                commands.entity(*entity2).despawn();
                
                // Handle player death
                if player.health <= 0.0 {
                    handle_player_death(&mut commands, *entity1);
                }
            }
            
            // Check for projectile-wall collision
            if let Ok(_projectile) = projectile_query.get(*entity1) {
                // Despawn projectile on wall hit
                commands.entity(*entity1).despawn();
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

/// System to clean up orphaned entities
fn cleanup_system(
    mut commands: Commands,
    projectile_query: Query<Entity, (With<Projectile>, Without<RigidBody>)>,
) {
    // Clean up projectiles that lost their physics body
    for entity in projectile_query.iter() {
        commands.entity(entity).despawn();
    }
}

/// Spawn a player with physics components
pub fn spawn_player(
    commands: &mut Commands,
    player_id: u64,
    spawn_position: Vec2,
) -> Entity {
    let collision_groups = GameCollisionGroups::default();
    
    commands.spawn((
        Player {
            player_id,
            ..Default::default()
        },
        PlayerInput::default(),
        Ship::default(),
        WeaponStats::default(),
        // Rapier2D components
        RigidBody::Dynamic,
        Collider::cuboid(15.0, 25.0), // Rectangular ship collider
        bevy_rapier2d::geometry::CollisionGroups::new(
            collision_groups.players,
            collision_groups.projectiles | collision_groups.walls
        ),
        Transform::from_translation(spawn_position.extend(0.0)),
        GlobalTransform::default(),
        Velocity::default(),
        ExternalForce::default(),
        ExternalImpulse::default(),
        Damping {
            linear_damping: 0.1,
            angular_damping: 0.3,
        },
        Name::new(format!("Player {}", player_id)),
    )).id()
}

/// Spawn an AI player for testing
pub fn spawn_ai_player(
    commands: &mut Commands,
    player_id: u64,
    spawn_position: Vec2,
    ai_type: AIType,
) -> Entity {
    let collision_groups = GameCollisionGroups::default();
    
    commands.spawn((
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
        // Rapier2D components
        RigidBody::Dynamic,
        Collider::cuboid(15.0, 25.0), // Rectangular ship collider
        bevy_rapier2d::geometry::CollisionGroups::new(
            collision_groups.players,
            collision_groups.projectiles | collision_groups.walls
        ),
        Transform::from_translation(spawn_position.extend(0.0)),
        GlobalTransform::default(),
        Velocity::default(),
        ExternalForce::default(),
        ExternalImpulse::default(),
        Damping {
            linear_damping: 0.1,
            angular_damping: 0.3,
        },
        Name::new(format!("AI Player {} ({:?})", player_id, ai_type)),
    )).id()
}
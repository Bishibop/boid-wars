use bevy::prelude::*;
use boid_wars_shared::*;
use lightyear::prelude::server::*;
use lightyear::prelude::{NetworkTarget, SharedConfig};
use lightyear::server::message::ReceiveMessage;
use std::net::SocketAddr;
use tracing::info;

pub mod despawn_utils;
pub mod physics;
pub mod position_sync;
use bevy_rapier2d::prelude::{Collider, ExternalForce, ExternalImpulse, RigidBody};
use despawn_utils::SafeDespawnExt;
use physics::{GameCollisionGroups, PhysicsPlugin, Ship, WeaponStats};
use position_sync::{PositionSyncPlugin, SyncPosition};

fn main() {
    info!("🚀 Boid Wars Server Starting...");

    // Load configuration
    let network_config = &*NETWORK_CONFIG;
    let game_config = &*GAME_CONFIG;
    let _server_settings = &*SERVER_CONFIG;

    // Configure server address
    let server_addr: SocketAddr = network_config
        .server_addr
        .parse()
        .expect("Failed to parse server address");

    info!("📡 Server will listen on {}", server_addr);
    info!(
        "🎮 Game area: {}x{}",
        game_config.game_width, game_config.game_height
    );

    // Create server config
    let lightyear_config = create_websocket_config(server_addr, network_config);

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ServerPlugins::new(lightyear_config))
        .add_plugins(ProtocolPlugin)
        .add_plugins(PhysicsPlugin)
        .add_plugins(PositionSyncPlugin)
        .add_plugins(BoidWarsServerPlugin)
        .run();
}

fn create_websocket_config(
    server_addr: SocketAddr,
    network_config: &NetworkConfig,
) -> lightyear::prelude::server::ServerConfig {
    info!("🔧 Creating WebSocket server config...");

    // WebSocket transport - NO CERTIFICATES!
    let transport = ServerTransport::WebSocketServer { server_addr };
    let io = IoConfig::from_transport(transport);

    // Use Netcode auth with a shared key for dev
    let netcode_config = NetcodeConfig::default()
        .with_protocol_id(network_config.protocol_id)
        .with_key(network_config.dev_key);

    let net_config = NetConfig::Netcode {
        config: netcode_config,
        io,
    };

    lightyear::prelude::server::ServerConfig {
        shared: SharedConfig::default(),
        net: vec![net_config],
        packet: Default::default(),
        replication: Default::default(),
        ping: Default::default(),
    }
}

// Server-specific plugin
pub struct BoidWarsServerPlugin;

impl Plugin for BoidWarsServerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameState>();

        // Add startup system to spawn server
        app.add_systems(Startup, setup_server);

        // Connection handling
        app.add_systems(
            Update,
            (
                handle_connections,
                handle_player_input,
                handle_reset_ai_message,
            ),
        );

        // Add game systems
        app.add_systems(Update, (log_status, spawn_collision_objects_delayed));
        app.add_systems(FixedUpdate, (move_boids, update_boid_ai));

        // Note: Physics systems (player_input_system, etc.) are added by PhysicsPlugin in FixedUpdate
    }
}

fn setup_server(mut commands: Commands) {
    info!("✅ Server initialized");

    // Load configuration
    let game_config = &*GAME_CONFIG;
    let _server_settings = &*SERVER_CONFIG;

    // Start the Lightyear server
    commands.queue(|world: &mut World| {
        world.start_server();
        info!("🚀 Lightyear server started and listening!");
    });

    // Create status timer
    commands.insert_resource(StatusTimer(Timer::from_seconds(
        _server_settings.status_log_interval,
        TimerMode::Repeating,
    )));

    // Spawn the single boid for Iteration 0
    commands.spawn((
        BoidBundle::new(1, game_config.spawn_x, game_config.spawn_y),
        Replicate::default(),
    ));

    info!("🤖 Spawned initial boid with replication");

    info!("🌐 Server ready for client connections");
}

// Spawn AI players when a human player connects
fn spawn_collision_objects_delayed(
    mut commands: Commands,
    players: Query<&boid_wars_shared::Player>,
    mut spawned: Local<bool>,
) {
    // Wait until at least one player is connected and we haven't spawned yet
    if !players.is_empty() && !*spawned {
        *spawned = true;
        // Spawn peaceful boids instead of AI players

        spawn_boid_flock(&mut commands);
        spawn_static_obstacles(&mut commands);
    }
}

// Helper function to spawn peaceful boids
fn spawn_boid_flock(commands: &mut Commands) {
    let game_config = &*GAME_CONFIG;

    // Spawn 8 boids scattered around the arena
    for i in 0..8 {
        let boid_id = 100 + i; // Use IDs 100-107

        // Scatter them around the arena
        let x = (i as f32 * 100.0) % game_config.game_width;
        let y = (i as f32 * 80.0) % game_config.game_height;

        commands.spawn((
            BoidBundle::new(boid_id, x, y),
            Replicate::default(),
            // Add physics components for collision
            RigidBody::Dynamic,
            Collider::ball(physics::BOID_RADIUS), // Small boid collider
            GameCollisionGroups::boid(),
            bevy_rapier2d::prelude::ActiveEvents::COLLISION_EVENTS, // Enable collision detection
            Transform::from_xyz(x, y, 0.0),
            GlobalTransform::default(),
            bevy_rapier2d::dynamics::Velocity::zero(),
            bevy_rapier2d::dynamics::GravityScale(0.0),
            SyncPosition, // Mark for position sync
        ));
    }
}

// Helper function to spawn static obstacles
fn spawn_static_obstacles(commands: &mut Commands) {
    let _game_config = &*GAME_CONFIG;
    let collision_groups = GameCollisionGroups::wall();

    // Create some obstacles scattered around the arena
    let obstacles = [
        (200.0, 200.0, 30.0, 30.0), // Square obstacle
        (600.0, 150.0, 40.0, 20.0), // Rectangular obstacle
        (300.0, 400.0, 25.0, 25.0), // Small square
        (500.0, 350.0, 35.0, 35.0), // Medium square
        (150.0, 450.0, 50.0, 15.0), // Long rectangle
    ];

    for (i, (x, y, width, height)) in obstacles.iter().enumerate() {
        commands.spawn((
            RigidBody::Fixed,
            Collider::cuboid(width / 2.0, height / 2.0), // Rapier uses half-extents
            Transform::from_xyz(*x, *y, 0.0),
            GlobalTransform::default(),
            collision_groups,
            bevy_rapier2d::prelude::ActiveEvents::COLLISION_EVENTS, // Enable collision detection
            Name::new(format!("Obstacle {}", i + 1)),
            // Add network components to make it visible to clients
            boid_wars_shared::Position(Vec2::new(*x, *y)),
            boid_wars_shared::Obstacle {
                id: i as u32,
                width: *width,
                height: *height,
            },
            boid_wars_shared::Health::default(),
            lightyear::prelude::server::Replicate::default(),
            SyncPosition, // Mark for position sync
        ));
    }
}

// Handle reset message from client (respawn AI players and obstacles)
fn handle_reset_ai_message(
    mut commands: Commands,
    mut reset_messages: EventReader<ReceiveMessage<ResetAIMessage>>,
    boids: Query<Entity, With<Boid>>,
    obstacles: Query<Entity, With<Name>>,
) {
    for _message in reset_messages.read() {
        // Reset debug logs

        // Despawn all boids
        for entity in boids.iter() {
            commands.safe_despawn(entity);
        }

        // Despawn all obstacles (they have Name components)
        for entity in obstacles.iter() {
            commands.safe_despawn(entity);
        }

        // Spawn new boids and obstacles
        spawn_boid_flock(&mut commands);
        spawn_static_obstacles(&mut commands);

        // Reset complete log
    }
}

// Handle new client connections
fn handle_connections(mut commands: Commands, mut connections: EventReader<ConnectEvent>) {
    let game_config = &*GAME_CONFIG;

    for event in connections.read() {
        let client_id = event.client_id;
        // Connection log

        // Spawn a player for the connected client with both networking and physics
        let player_entity = commands
            .spawn((
                PlayerBundle::new(
                    client_id.to_bits(),
                    format!("Player {}", client_id.to_bits()),
                    game_config.spawn_x,
                    game_config.spawn_y,
                ),
                // Networking
                Replicate {
                    controlled_by: ControlledBy {
                        target: NetworkTarget::Single(client_id),
                        ..default()
                    },
                    ..default()
                },
            ))
            .id();

        // Add physics components separately to avoid tuple size limits
        commands.entity(player_entity).insert((
            physics::Player {
                player_id: client_id.to_bits(),
                ..Default::default()
            },
            physics::PlayerInput::default(),
            Ship::default(),
            WeaponStats::default(),
        ));

        // Add physics body components
        commands.entity(player_entity).insert((
            RigidBody::Dynamic,
            Collider::cuboid(5.0, 5.0), // Match 10x10 visual size
            GameCollisionGroups::player(),
            physics::Velocity::zero(),
            ExternalForce::default(),
            ExternalImpulse::default(),
            Transform::from_xyz(game_config.spawn_x, game_config.spawn_y, 0.0), // Back to original spawn
            GlobalTransform::default(),
            bevy_rapier2d::dynamics::GravityScale(0.0), // Disable gravity for top-down space game
            bevy_rapier2d::dynamics::Sleeping::disabled(),
            bevy_rapier2d::dynamics::Damping {
                linear_damping: 0.0, // No damping for immediate response
                angular_damping: 0.0,
            },
            bevy_rapier2d::dynamics::AdditionalMassProperties::Mass(1.0), // Light mass
            SyncPosition,                                                 // Mark for position sync
        ));

        // Spawn logs
    }
}

// Handle player input messages - update physics input properly
fn handle_player_input(
    mut message_events: EventReader<ReceiveMessage<boid_wars_shared::PlayerInput>>,
    mut players: Query<
        (&boid_wars_shared::Player, &mut physics::PlayerInput),
        With<physics::Player>,
    >,
) {
    for event in message_events.read() {
        let client_id = event.from;
        let input = &event.message;

        // Input validation
        if !validate_player_input(input) {
            warn!(
                "Invalid input from client {:?}: movement={:?}, aim={:?}",
                client_id, input.movement, input.aim
            );
            continue;
        }

        // Find the player for this client and update their physics input
        for (player, mut physics_input) in players.iter_mut() {
            if player.id == client_id.to_bits() {
                // Update physics input - this feeds into the physics input system
                physics_input.movement = input.movement.normalize_or_zero(); // Ensure normalized
                physics_input.aim_direction = input.aim.normalize_or_zero(); // Ensure normalized
                physics_input.thrust = if input.movement.length() > 0.0 {
                    1.0
                } else {
                    0.0
                };
                physics_input.shooting = input.fire;
            }
        }
    }
}

/// Validate player input to prevent malicious or malformed data
fn validate_player_input(input: &boid_wars_shared::PlayerInput) -> bool {
    // Check movement vector is valid
    if !input.movement.is_finite() || input.movement.length() > 1.1 {
        return false;
    }

    // Check aim direction is valid
    if !input.aim.is_finite() || input.aim.length() > 1.1 {
        return false;
    }

    true
}

#[allow(clippy::too_many_arguments)]
fn log_status(
    time: Res<Time>,
    mut status_timer: ResMut<StatusTimer>,
    players: Query<&Position, With<boid_wars_shared::Player>>,
    boids: Query<&Position, With<Boid>>,
    physics_players: Query<&Transform, With<physics::Player>>,
    projectiles: Query<&Transform, With<Projectile>>,
    all_entities: Query<Entity>,
    all_physics_components: Query<Entity, With<physics::Player>>,
) {
    if status_timer.0.tick(time.delta()).just_finished() {
        let player_count = players.iter().len();
        let boid_count = boids.iter().len();
        let physics_player_count = physics_players.iter().len();
        let projectile_count = projectiles.iter().len();
        let total_entities = all_entities.iter().len();
        let physics_component_count = all_physics_components.iter().len();

        info!(
            "📊 Server - Uptime: {:.1}s | Network Players: {} | Boids: {} | Physics Players: {} | Projectiles: {} | Total Entities: {} | Physics Components: {}",
            time.elapsed_secs(),
            player_count,
            boid_count,
            physics_player_count,
            projectile_count,
            total_entities,
            physics_component_count
        );
    }
}

// Player movement is handled by the physics system in physics.rs

fn move_boids(
    time: Res<Time>,
    mut boids: Query<
        (
            &mut Transform,
            &boid_wars_shared::Velocity,
            Option<&mut bevy_rapier2d::dynamics::Velocity>,
        ),
        With<Boid>,
    >,
) {
    let game_config = &*GAME_CONFIG;
    let delta = time.delta_secs();

    for (mut transform, vel, physics_vel) in boids.iter_mut() {
        // Update transform position (physics)
        transform.translation.x += vel.0.x * delta;
        transform.translation.y += vel.0.y * delta;

        // Keep boid in bounds
        transform.translation.x = transform.translation.x.clamp(0.0, game_config.game_width);
        transform.translation.y = transform.translation.y.clamp(0.0, game_config.game_height);

        // Also update physics velocity if present
        if let Some(mut phys_vel) = physics_vel {
            phys_vel.linvel = vel.0;
        }
    }
}

fn update_boid_ai(
    mut boids: Query<(Entity, &Transform, &mut boid_wars_shared::Velocity), With<Boid>>,
    _players: Query<&Transform, (With<boid_wars_shared::Player>, Without<Boid>)>,
    mut debug_timer: Local<f32>,
    time: Res<Time>,
) {
    *debug_timer += time.delta_secs();

    let game_config = &*GAME_CONFIG;
    let max_speed = game_config.boid_speed * 0.4; // Slower, more peaceful movement

    let boid_count = boids.iter().count();
    if *debug_timer > 2.0 {
        info!(
            "🐦 BOID AI: Processing {} boids with max_speed={:.1}",
            boid_count, max_speed
        );
        *debug_timer = 0.0;
    }

    // Collect all boid data for flocking calculations
    let boid_data: Vec<(Entity, Vec2, Vec2)> = boids
        .iter()
        .map(|(entity, transform, vel)| (entity, transform.translation.truncate(), vel.0))
        .collect();

    for (entity, boid_transform, mut boid_vel) in boids.iter_mut() {
        let boid_pos = boid_transform.translation.truncate();
        let mut separation = Vec2::ZERO;
        let mut alignment = Vec2::ZERO;
        let mut cohesion = Vec2::ZERO;
        let mut neighbor_count = 0;

        // Flocking parameters
        let separation_radius = 50.0;
        let alignment_radius = 80.0;
        let cohesion_radius = 100.0;

        for (other_entity, other_pos, other_vel) in &boid_data {
            if *other_entity == entity {
                continue;
            } // Skip self

            let diff = boid_pos - *other_pos;
            let distance = diff.length();

            // Separation: avoid crowding neighbors
            if distance > 0.0 && distance < separation_radius {
                let normalized_diff = diff / distance;
                separation += normalized_diff / distance; // Stronger when closer
            }

            // Alignment and Cohesion: only with nearby neighbors
            if distance < alignment_radius {
                neighbor_count += 1;

                // Alignment: steer towards average heading of neighbors
                alignment += *other_vel;

                // Cohesion: steer towards average position of neighbors
                if distance < cohesion_radius {
                    cohesion += *other_pos;
                }
            }
        }

        // Calculate steering forces
        let mut steering = Vec2::ZERO;

        // Apply separation (strongest force)
        if separation.length() > 0.0 {
            steering += separation.normalize() * max_speed * 1.5;
        }

        // Apply alignment
        if neighbor_count > 0 {
            let avg_velocity = alignment / neighbor_count as f32;
            if avg_velocity.length() > 0.0 {
                steering += (avg_velocity.normalize() * max_speed - boid_vel.0) * 0.5;
            }

            // Apply cohesion
            let avg_position = cohesion / neighbor_count as f32;
            let cohesion_force = (avg_position - boid_pos).normalize_or_zero() * max_speed;
            steering += (cohesion_force - boid_vel.0) * 0.3;
        }

        // Add some wandering for natural movement
        let wander_strength = 0.2;
        let time_factor = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f32();
        let wander_angle = (entity.index() as f32 + time_factor * 0.5).sin() * 2.0;
        let wander =
            Vec2::new(wander_angle.cos(), wander_angle.sin()) * max_speed * wander_strength;
        steering += wander;

        // Apply steering with smooth acceleration
        boid_vel.0 += steering * 0.02; // Gentle acceleration

        // Limit speed and add boundaries
        if boid_vel.0.length() > max_speed {
            boid_vel.0 = boid_vel.0.normalize() * max_speed;
        }

        // Boundary behavior: gently turn away from edges
        let margin = 50.0;
        let mut boundary_force = Vec2::ZERO;

        if boid_pos.x < margin {
            boundary_force.x += (margin - boid_pos.x) * 0.5;
        } else if boid_pos.x > game_config.game_width - margin {
            boundary_force.x -= (boid_pos.x - (game_config.game_width - margin)) * 0.5;
        }

        if boid_pos.y < margin {
            boundary_force.y += (margin - boid_pos.y) * 0.5;
        } else if boid_pos.y > game_config.game_height - margin {
            boundary_force.y -= (boid_pos.y - (game_config.game_height - margin)) * 0.5;
        }

        boid_vel.0 += boundary_force * 0.1;

        // Debug velocity changes
        if *debug_timer < 0.1 && entity.index() == 1 {
            // Debug first boid every 2 seconds
            info!(
                "🐦 Boid {:?}: pos=({:.1}, {:.1}) vel=({:.1}, {:.1}) steering=({:.1}, {:.1})",
                entity, boid_pos.x, boid_pos.y, boid_vel.0.x, boid_vel.0.y, steering.x, steering.y
            );
        }
    }
}

#[derive(Resource)]
struct StatusTimer(Timer);

#[derive(Resource, Default)]
struct GameState;

use bevy::prelude::*;
use boid_wars_shared::*;
use lightyear::prelude::server::*;
use lightyear::prelude::{NetworkTarget, SharedConfig};
use lightyear::server::message::ReceiveMessage;
use std::net::SocketAddr;
use tracing::{info, warn};

// Camera2dBundle should be in prelude

pub mod config;
pub mod debug_ui;
pub mod despawn_utils;
pub mod flocking;
pub mod physics;
pub mod pool;
pub mod position_sync;
pub mod spatial_grid;
use bevy_rapier2d::prelude::{Collider, ExternalForce, ExternalImpulse, RigidBody};
use config::PhysicsConfig;
use debug_ui::DebugUIPlugin;
use flocking::FlockingPlugin;
use physics::{GameCollisionGroups, PhysicsPlugin, Ship, WeaponStats};
use position_sync::{PositionSyncPlugin, SyncPosition};

fn main() {
    info!("Starting Boid Wars server...");

    // Load configuration
    let network_config = &*NETWORK_CONFIG;
    let game_config = &*GAME_CONFIG;

    // Configure server address
    let server_addr: SocketAddr = network_config
        .server_addr
        .parse()
        .expect("Failed to parse server address");

    info!(
        "Server listening on {} | Game area: {}x{}",
        server_addr, game_config.game_width, game_config.game_height
    );

    // Create server config
    let lightyear_config = create_websocket_config(server_addr, network_config);

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Boid Wars Server".to_string(),
                resolution: (1800.0, 1350.0).into(),
                ..default()
            }),
            ..default()
        }))
        // Add debug UI early so it gets input events first
        .add_plugins(DebugUIPlugin)
        .add_plugins(ServerPlugins::new(lightyear_config))
        .add_plugins(ProtocolPlugin)
        .add_plugins(PhysicsPlugin)
        .add_plugins(PositionSyncPlugin)
        .add_plugins(FlockingPlugin)
        .add_plugins(BoidWarsServerPlugin)
        .run();
}

fn create_websocket_config(
    server_addr: SocketAddr,
    network_config: &NetworkConfig,
) -> lightyear::prelude::server::ServerConfig {
    // Create WebSocket server config

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
        app.add_systems(Update, (handle_connections, handle_player_input));

        // Add game systems
        app.add_systems(Update, (log_status, spawn_collision_objects_delayed));
        // Boid movement is handled by FlockingPlugin
        // Note: Physics systems (player_input_system, etc.) are added by PhysicsPlugin in FixedUpdate
    }
}

fn setup_server(mut commands: Commands) {
    // Start the Lightyear server
    commands.queue(|world: &mut World| {
        world.start_server();
    });

    // Create status timer
    commands.insert_resource(StatusTimer(Timer::from_seconds(
        5.0, // Default status log interval
        TimerMode::Repeating,
    )));
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

    // Spawn 30 boids scattered around the arena
    for i in 0..30 {
        let boid_id = 100 + i; // Use IDs 100-129

        // Scatter them around the arena with some randomness
        let base_x = ((i % 6) as f32 * game_config.game_width / 6.0) + 50.0;
        let base_y = ((i / 6) as f32 * game_config.game_height / 5.0) + 50.0;

        // Add some random offset to avoid perfect grid
        let x = base_x + (rand::random::<f32>() - 0.5) * 60.0;
        let y = base_y + (rand::random::<f32>() - 0.5) * 60.0;

        // Clamp to ensure they spawn within bounds
        let x = x.clamp(20.0, game_config.game_width - 20.0);
        let y = y.clamp(20.0, game_config.game_height - 20.0);

        // Create boid with random initial velocity
        let mut bundle = BoidBundle::new(boid_id, x, y);
        let angle = rand::random::<f32>() * std::f32::consts::TAU;
        let speed = 100.0; // Initial speed
        bundle.velocity = boid_wars_shared::Velocity::new(angle.cos() * speed, angle.sin() * speed);

        // Set boid health to 20 (standard boid)
        bundle.health = boid_wars_shared::Health {
            current: 20.0,
            max: 20.0,
        };

        commands.spawn((
            bundle,
            Replicate::default(),
            // Add physics components for collision
            RigidBody::Dynamic,
            Collider::ball(physics::BOID_RADIUS), // Use constant
            GameCollisionGroups::boid(),
            bevy_rapier2d::prelude::ActiveEvents::COLLISION_EVENTS, // Enable collision detection
            Transform::from_xyz(x, y, 0.0),
            GlobalTransform::default(),
            bevy_rapier2d::dynamics::Velocity {
                linvel: Vec2::new(angle.cos() * speed, angle.sin() * speed),
                angvel: 0.0,
            },
            bevy_rapier2d::dynamics::GravityScale(0.0),
            bevy_rapier2d::dynamics::Damping {
                linear_damping: 0.0, // No damping for free movement
                angular_damping: 1.0,
            },
            bevy_rapier2d::dynamics::AdditionalMassProperties::Mass(0.5), // Light boids
            SyncPosition,                                                 // Mark for position sync
        ));
    }
}

// Helper function to spawn static obstacles
fn spawn_static_obstacles(commands: &mut Commands) {
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

// Handle new client connections
fn handle_connections(
    mut commands: Commands,
    mut connections: EventReader<ConnectEvent>,
    physics_config: Res<PhysicsConfig>,
) {
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
            // Add Health component for replication
            boid_wars_shared::Health {
                current: 100.0,
                max: 100.0,
            },
        ));

        // Add physics body components
        commands.entity(player_entity).insert((
            RigidBody::Dynamic,
            Collider::cuboid(
                physics_config.player_collider_size,
                physics_config.player_collider_size,
            ),
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
    projectiles: Query<&Transform, With<Projectile>>,
    all_entities: Query<Entity>,
) {
    if status_timer.0.tick(time.delta()).just_finished() {
        let player_count = players.iter().len();
        let boid_count = boids.iter().len();
        let projectile_count = projectiles.iter().len();
        let total_entities = all_entities.iter().len();

        info!(
            "[Status] Players: {} | Boids: {} | Projectiles: {} | Entities: {}",
            player_count, boid_count, projectile_count, total_entities
        );
    }
}

#[derive(Resource)]
struct StatusTimer(Timer);

#[derive(Resource, Default)]
struct GameState;

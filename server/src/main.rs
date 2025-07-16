use bevy::prelude::*;
use boid_wars_shared::*;
use lightyear::prelude::server::*;
use lightyear::prelude::{NetworkTarget, SharedConfig};
use lightyear::server::message::ReceiveMessage;
use std::net::SocketAddr;
use tracing::info;

pub mod physics;
use physics::{PhysicsPlugin, Ship, WeaponStats, GameCollisionGroups, Projectile, AIPlayer};
use bevy_rapier2d::prelude::{RigidBody, Collider, ExternalForce, ExternalImpulse};

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
        app.add_systems(Update, (handle_connections, handle_player_input, handle_reset_ai_message));

        // Add game systems
        app.add_systems(Update, (log_status, spawn_collision_objects_delayed, debug_player_components));
        app.add_systems(FixedUpdate, (move_players, move_boids, update_boid_ai));
        
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
        // Removed AI spawn log
        
        spawn_ai_players(&mut commands);
        spawn_static_obstacles(&mut commands);
    }
}


// Helper function to spawn AI players
fn spawn_ai_players(commands: &mut Commands) {
    let _game_config = &*GAME_CONFIG;
    
    // Spawn 3 AI players with different behaviors around the arena
    let ai_spawns = [
        (100.0, 100.0, physics::AIType::Circler),
        (700.0, 100.0, physics::AIType::Bouncer), 
        (400.0, 500.0, physics::AIType::Chaser),
    ];
    
    for (i, (x, y, ai_type)) in ai_spawns.iter().enumerate() {
        let ai_id = 1000 + i as u64; // Use high IDs to avoid conflicts
        
        physics::spawn_ai_player(commands, ai_id, Vec2::new(*x, *y), *ai_type);
        
        // Removed AI spawn log
    }
    
    // Removed AI spawn complete log
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
        ));
        
        // Removed obstacle spawn log
    }
    
    // Removed obstacle spawn complete log
}

// Handle reset message from client (respawn AI players and obstacles)
fn handle_reset_ai_message(
    mut commands: Commands,
    mut reset_messages: EventReader<ReceiveMessage<ResetAIMessage>>,
    ai_players: Query<Entity, With<AIPlayer>>,
    obstacles: Query<Entity, With<Name>>,
) {
    for message in reset_messages.read() {
        // Removed reset debug logs
        
        // Despawn all AI players
        for entity in ai_players.iter() {
            commands.entity(entity).despawn();
        }
        
        // Despawn all obstacles (they have Name components)
        for entity in obstacles.iter() {
            commands.entity(entity).despawn();
        }
        
        // Spawn new AI players and obstacles
        spawn_ai_players(&mut commands);
        spawn_static_obstacles(&mut commands);
        
        // Removed reset complete log
    }
}

// Handle new client connections
fn handle_connections(mut commands: Commands, mut connections: EventReader<ConnectEvent>) {
    let game_config = &*GAME_CONFIG;

    for event in connections.read() {
        let client_id = event.client_id;
        // Removed connection log

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
                linear_damping: 0.0,  // No damping for immediate response
                angular_damping: 0.0,
            },
            bevy_rapier2d::dynamics::AdditionalMassProperties::Mass(1.0),  // Light mass
        ));

        // Removed spawn logs
    }
}

// Handle player input messages - update physics input properly
fn handle_player_input(
    mut message_events: EventReader<ReceiveMessage<boid_wars_shared::PlayerInput>>,
    mut players: Query<(&boid_wars_shared::Player, &mut physics::PlayerInput), With<physics::Player>>,
) {
    for event in message_events.read() {
        let client_id = event.from;
        let input = &event.message;
        
        // Removed debug logs for cleaner output

        let mut found_player = false;
        // Find the player for this client and update their physics input
        for (player, mut physics_input) in players.iter_mut() {
            if player.id == client_id.to_bits() {
                found_player = true;
                
                // Store old values for comparison
                let old_movement = physics_input.movement;
                let old_thrust = physics_input.thrust;
                
                // Update physics input - this feeds into the physics input system
                physics_input.movement = input.movement;
                physics_input.aim_direction = input.aim;
                physics_input.thrust = if input.movement.length() > 0.0 { 1.0 } else { 0.0 };
                physics_input.shooting = input.fire;

                // Removed debug logs
            }
        }
        
        // Removed debug logs
    }
}

fn log_status(
    time: Res<Time>,
    mut status_timer: ResMut<StatusTimer>,
    players: Query<&Position, With<boid_wars_shared::Player>>,
    boids: Query<&Position, With<Boid>>,
    physics_players: Query<&Transform, With<physics::Player>>,
    ai_players: Query<&Transform, With<AIPlayer>>,
    projectiles: Query<&Transform, With<Projectile>>,
    all_entities: Query<Entity>,
    all_physics_components: Query<Entity, With<physics::Player>>,
) {
    if status_timer.0.tick(time.delta()).just_finished() {
        let player_count = players.iter().len();
        let boid_count = boids.iter().len();
        let physics_player_count = physics_players.iter().len();
        let ai_player_count = ai_players.iter().len();
        let projectile_count = projectiles.iter().len();
        let total_entities = all_entities.iter().len();
        let physics_component_count = all_physics_components.iter().len();
        
        info!(
            "📊 Server - Uptime: {:.1}s | Network Players: {} | Boids: {} | Physics Players: {} | AI Players: {} | Projectiles: {} | Total Entities: {} | Physics Components: {}",
            time.elapsed_secs(),
            player_count,
            boid_count,
            physics_player_count,
            ai_player_count,
            projectile_count,
            total_entities,
            physics_component_count
        );
    }
}

fn move_players(time: Res<Time>, mut players: Query<(&mut Position, &boid_wars_shared::Velocity), With<boid_wars_shared::Player>>) {
    let game_config = &*GAME_CONFIG;
    let delta = time.delta_secs();

    for (mut pos, vel) in players.iter_mut() {
        // For now, just apply velocity until we have input handling working
        // Update position
        pos.0.x += vel.0.x * delta;
        pos.0.y += vel.0.y * delta;

        // Keep player in bounds
        pos.0.x = pos.0.x.clamp(0.0, game_config.game_width);
        pos.0.y = pos.0.y.clamp(0.0, game_config.game_height);
    }
}

fn move_boids(time: Res<Time>, mut boids: Query<(&mut Position, &boid_wars_shared::Velocity), With<Boid>>) {
    let game_config = &*GAME_CONFIG;
    let delta = time.delta_secs();

    for (mut pos, vel) in boids.iter_mut() {
        // Update position
        pos.0.x += vel.0.x * delta;
        pos.0.y += vel.0.y * delta;

        // Keep boid in bounds
        pos.0.x = pos.0.x.clamp(0.0, game_config.game_width);
        pos.0.y = pos.0.y.clamp(0.0, game_config.game_height);
    }
}

fn update_boid_ai(
    mut boids: Query<(&Position, &mut boid_wars_shared::Velocity), With<Boid>>,
    players: Query<&Position, (With<boid_wars_shared::Player>, Without<Boid>)>,
) {
    let game_config = &*GAME_CONFIG;

    for (boid_pos, mut boid_vel) in boids.iter_mut() {
        // Find nearest player
        let nearest_player = players.iter().min_by(|a, b| {
            let dist_a = (a.0.x - boid_pos.0.x).powi(2) + (a.0.y - boid_pos.0.y).powi(2);
            let dist_b = (b.0.x - boid_pos.0.x).powi(2) + (b.0.y - boid_pos.0.y).powi(2);
            dist_a
                .partial_cmp(&dist_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        if let Some(player_pos) = nearest_player {
            // Move towards nearest player
            let dx = player_pos.0.x - boid_pos.0.x;
            let dy = player_pos.0.y - boid_pos.0.y;
            let distance = (dx * dx + dy * dy).sqrt();

            if distance > 0.0 {
                boid_vel.0.x = (dx / distance) * game_config.boid_speed;
                boid_vel.0.y = (dy / distance) * game_config.boid_speed;
            }
        }
    }
}

#[derive(Resource)]
struct StatusTimer(Timer);


#[derive(Resource, Default)]
struct GameState;

/// Debug system to check what components each player entity has
fn debug_player_components(
    all_players: Query<Entity, With<boid_wars_shared::Player>>,
    transforms: Query<&Transform>,
    physics_players: Query<&physics::Player>,
    positions: Query<&boid_wars_shared::Position>,
    velocities: Query<&boid_wars_shared::Velocity>,
    physics_velocities: Query<&bevy_rapier2d::dynamics::Velocity>,
    mut debug_timer: Local<f32>,
    time: Res<Time>,
) {
    *debug_timer += time.delta_secs();
    
    if *debug_timer > 3.0 {
        *debug_timer = 0.0;
        
        info!("🔍 COMPONENT DEBUG: Checking player entities...");
        
        for entity in all_players.iter() {
            let has_transform = transforms.contains(entity);
            let has_physics_player = physics_players.contains(entity);
            let has_position = positions.contains(entity);
            let has_velocity = velocities.contains(entity);
            let has_physics_velocity = physics_velocities.contains(entity);
            
            info!("🔍 Entity {:?}: Transform={}, PhysicsPlayer={}, Position={}, Velocity={}, PhysicsVelocity={}", 
                entity, has_transform, has_physics_player, has_position, has_velocity, has_physics_velocity);
                
            if has_transform && has_physics_player && has_position && has_velocity && has_physics_velocity {
                info!("✅ Entity {:?} has ALL required components for sync!", entity);
            } else {
                info!("❌ Entity {:?} is MISSING components for sync!", entity);
            }
        }
    }
}
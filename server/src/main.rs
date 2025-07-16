use bevy::prelude::*;
use boid_wars_shared::*;
use lightyear::prelude::server::*;
use lightyear::prelude::{NetworkTarget, SharedConfig};
use lightyear::server::message::ReceiveMessage;
use std::net::SocketAddr;
use tracing::{debug, info};

mod physics;
use physics::*;

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
        app.add_systems(Update, (handle_connections, handle_player_input));

        // Add game systems
        app.add_systems(Update, log_status);
        app.add_systems(FixedUpdate, (move_players, move_boids, update_boid_ai));
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
    
    // Also spawn AI players for physics testing
    spawn_ai_player(&mut commands, 100, Vec2::new(300.0, 0.0), AIType::Circler);
    spawn_ai_player(&mut commands, 101, Vec2::new(-300.0, 0.0), AIType::Bouncer);
    spawn_ai_player(&mut commands, 102, Vec2::new(0.0, 300.0), AIType::Shooter);
    spawn_ai_player(&mut commands, 103, Vec2::new(0.0, -300.0), AIType::Chaser);
    info!("🤖 Spawned 4 AI players for physics testing");
    
    info!("🌐 Server ready for client connections");
}

// Handle new client connections
fn handle_connections(mut commands: Commands, mut connections: EventReader<ConnectEvent>) {
    let game_config = &*GAME_CONFIG;

    for event in connections.read() {
        let client_id = event.client_id;
        info!("🎮 Client {} connected!", client_id);

        // Spawn a player for the connected client with both networking and physics
        let player_entity = commands
            .spawn((
                PlayerBundle::new(
                    client_id.to_bits(),
                    format!("Player {}", client_id.to_bits()),
                    game_config.spawn_x,
                    game_config.spawn_y,
                ),
                // Add physics components
                physics::Player {
                    player_id: client_id.to_bits(),
                    ..Default::default()
                },
                PlayerInput::default(),
                Ship::default(),
                WeaponStats::default(),
                // Physics body
                RigidBody::Dynamic,
                Collider::cuboid(24.0, 32.0),
                GameCollisionGroups::player(),
                physics::Velocity::zero(),
                ExternalForce::default(),
                ExternalImpulse::default(),
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

        info!(
            "✅ Spawned player entity {:?} for client {} with physics",
            player_entity, client_id
        );
    }
}

// Handle player input messages
fn handle_player_input(
    mut message_events: EventReader<ReceiveMessage<PlayerInput>>,
    mut players: Query<(&Player, &mut Velocity), With<Player>>,
) {
    let game_config = &*GAME_CONFIG;

    for event in message_events.read() {
        let client_id = event.from;
        let input = &event.message;

        // Find the player for this client
        for (player, mut velocity) in players.iter_mut() {
            if player.id == client_id.to_bits() {
                // Apply movement input to velocity
                velocity.0.x = input.movement.x * game_config.player_speed;
                velocity.0.y = input.movement.y * game_config.player_speed;

                if input.movement.length() > 0.0 {
                    debug!("📍 Player {} moving: {:?}", player.id, input.movement);
                }
            }
        }
    }
}

fn log_status(
    time: Res<Time>,
    mut status_timer: ResMut<StatusTimer>,
    players: Query<&Position, With<Player>>,
    boids: Query<&Position, With<Boid>>,
    physics_players: Query<&Transform, With<physics::Player>>,
    ai_players: Query<&Transform, With<AIPlayer>>,
    projectiles: Query<&Transform, With<Projectile>>,
) {
    if status_timer.0.tick(time.delta()).just_finished() {
        let player_count = players.iter().len();
        let boid_count = boids.iter().len();
        let physics_player_count = physics_players.iter().len();
        let ai_player_count = ai_players.iter().len();
        let projectile_count = projectiles.iter().len();
        
        info!(
            "📊 Server - Uptime: {:.1}s | Network Players: {} | Boids: {} | Physics Players: {} | AI Players: {} | Projectiles: {}",
            time.elapsed_secs(),
            player_count,
            boid_count,
            physics_player_count,
            ai_player_count,
            projectile_count
        );
    }
}

fn move_players(time: Res<Time>, mut players: Query<(&mut Position, &Velocity), With<Player>>) {
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

fn move_boids(time: Res<Time>, mut boids: Query<(&mut Position, &Velocity), With<Boid>>) {
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
    mut boids: Query<(&Position, &mut Velocity), With<Boid>>,
    players: Query<&Position, (With<Player>, Without<Boid>)>,
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
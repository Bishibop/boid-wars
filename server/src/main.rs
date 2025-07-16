use bevy::prelude::*;
use boid_wars_shared::*;
use lightyear::prelude::server::*;
use lightyear::prelude::{NetworkTarget, SharedConfig};
use lightyear::server::message::ReceiveMessage;
use std::net::SocketAddr;
use tracing::{debug, info};

// Server configuration constants
const SERVER_ADDR: &str = "127.0.0.1:5001";
const STATUS_LOG_INTERVAL: f32 = 5.0;
const DEV_KEY: [u8; 32] = [0u8; 32];
const PROTOCOL_ID: u64 = 12345;

// Game spawn constants
const SPAWN_X: f32 = 400.0;
const SPAWN_Y: f32 = 300.0;

fn main() {
    info!("🚀 Boid Wars Server Starting...");

    // Configure server address
    let server_addr: SocketAddr = SERVER_ADDR.parse().expect("Failed to parse server address");

    info!("📡 Server will listen on {}", server_addr);

    // Create server config
    let server_config = create_websocket_config(server_addr);

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ServerPlugins::new(server_config))
        .add_plugins(ProtocolPlugin)
        .add_plugins(BoidWarsServerPlugin)
        .run();
}

fn create_websocket_config(server_addr: SocketAddr) -> ServerConfig {
    info!("🔧 Creating WebSocket server config...");

    // WebSocket transport - NO CERTIFICATES!
    let transport = ServerTransport::WebSocketServer { server_addr };
    let io = IoConfig::from_transport(transport);

    // Use Netcode auth with a shared key for dev
    let netcode_config = NetcodeConfig::default()
        .with_protocol_id(PROTOCOL_ID)
        .with_key(DEV_KEY);

    let net_config = NetConfig::Netcode {
        config: netcode_config,
        io,
    };

    ServerConfig {
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

    // Start the Lightyear server
    commands.queue(|world: &mut World| {
        world.start_server();
        info!("🚀 Lightyear server started and listening!");
    });

    // Create status timer
    commands.spawn(StatusTimer {
        timer: Timer::from_seconds(STATUS_LOG_INTERVAL, TimerMode::Repeating),
    });

    // Spawn the single boid for Iteration 0
    commands.spawn((BoidBundle::new(1, SPAWN_X, SPAWN_Y), Replicate::default()));

    info!("🤖 Spawned initial boid with replication");
    info!("🌐 Server ready for client connections");
}

// Handle new client connections
fn handle_connections(mut commands: Commands, mut connections: EventReader<ConnectEvent>) {
    for event in connections.read() {
        let client_id = event.client_id;
        info!("🎮 Client {} connected!", client_id);

        // Spawn a player for the connected client
        let player_entity = commands
            .spawn((
                PlayerBundle::new(
                    client_id.to_bits(),
                    format!("Player {}", client_id.to_bits()),
                    SPAWN_X,
                    SPAWN_Y,
                ),
                Replicate {
                    // Player is controlled by the connected client
                    controlled_by: ControlledBy {
                        target: NetworkTarget::Single(client_id),
                        ..default()
                    },
                    ..default()
                },
            ))
            .id();

        info!(
            "✅ Spawned player entity {:?} for client {}",
            player_entity, client_id
        );
    }
}

// Handle player input messages
fn handle_player_input(
    mut message_events: EventReader<ReceiveMessage<PlayerInput>>,
    mut players: Query<(&Player, &mut Velocity), With<Player>>,
) {
    for event in message_events.read() {
        let client_id = event.from;
        let input = &event.message;

        // Find the player for this client
        for (player, mut velocity) in players.iter_mut() {
            if player.id == client_id.to_bits() {
                // Apply movement input to velocity
                velocity.0.x = input.movement.x * PLAYER_SPEED;
                velocity.0.y = input.movement.y * PLAYER_SPEED;

                if input.movement.length() > 0.0 {
                    debug!("📍 Player {} moving: {:?}", player.id, input.movement);
                }
            }
        }
    }
}

fn log_status(
    time: Res<Time>,
    mut query: Query<&mut StatusTimer>,
    players: Query<&Position, With<Player>>,
    boids: Query<&Position, With<Boid>>,
) {
    for mut status in query.iter_mut() {
        if status.timer.tick(time.delta()).just_finished() {
            let player_count = players.iter().len();
            let boid_count = boids.iter().len();
            info!(
                "📊 Server running - Uptime: {:.1}s | Players: {} | Boids: {}",
                time.elapsed_secs(),
                player_count,
                boid_count
            );
        }
    }
}

fn move_players(time: Res<Time>, mut players: Query<(&mut Position, &Velocity), With<Player>>) {
    let delta = time.delta_secs();

    for (mut pos, vel) in players.iter_mut() {
        // For now, just apply velocity until we have input handling working
        // Update position
        pos.0.x += vel.0.x * delta;
        pos.0.y += vel.0.y * delta;

        // Keep player in bounds
        pos.0.x = pos.0.x.clamp(0.0, GAME_WIDTH);
        pos.0.y = pos.0.y.clamp(0.0, GAME_HEIGHT);
    }
}

fn move_boids(time: Res<Time>, mut boids: Query<(&mut Position, &Velocity), With<Boid>>) {
    let delta = time.delta_secs();

    for (mut pos, vel) in boids.iter_mut() {
        // Update position
        pos.0.x += vel.0.x * delta;
        pos.0.y += vel.0.y * delta;

        // Keep boid in bounds
        pos.0.x = pos.0.x.clamp(0.0, GAME_WIDTH);
        pos.0.y = pos.0.y.clamp(0.0, GAME_HEIGHT);
    }
}

fn update_boid_ai(
    mut boids: Query<(&Position, &mut Velocity), With<Boid>>,
    players: Query<&Position, (With<Player>, Without<Boid>)>,
) {
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
                boid_vel.0.x = (dx / distance) * BOID_SPEED;
                boid_vel.0.y = (dy / distance) * BOID_SPEED;
            }
        }
    }
}

#[derive(Component)]
struct StatusTimer {
    timer: Timer,
}

#[derive(Resource, Default)]
struct GameState;

use bevy::prelude::*;
use boid_wars_shared::*;
use lightyear::prelude::*;
use lightyear::prelude::server::*;
use lightyear::server::netcode::*;
use lightyear::server::replication::*;
use std::time::Duration;

fn main() {
    println!("🚀 Boid Wars Server Starting...");

    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(ServerPlugins {
            tick_duration: Duration::from_secs_f64(1.0 / 30.0), // 30Hz tick rate
        })
        .add_plugins(SharedPlugin)
        .add_plugins(GameServerPlugin)
        .run();
}

// Server-specific plugin
pub struct GameServerPlugin;

impl Plugin for GameServerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameState>();
        
        // Add startup system to spawn server
        app.add_systems(Startup, setup_server);
        
        // Add connection handling observers (Lightyear 0.21 pattern)
        app.add_observer(handle_client_connect);
        app.add_observer(handle_client_disconnect);
        
        // Add game systems
        app.add_systems(Update, log_status);
        app.add_systems(FixedUpdate, (
            move_players,
            move_boids,
            update_boid_ai,
        ));
    }
}

fn setup_server(mut commands: Commands) {
    println!("✅ Server initialized");
    
    // Spawn the Lightyear server with proper configuration
    commands.spawn(NetcodeServer::new(
        NetcodeServerConfig::default()
            .with_addr("127.0.0.1:3000".parse().unwrap())
    ));
    println!("🌐 Lightyear server spawned on 127.0.0.1:3000");

    // Create status timer
    commands.spawn(StatusTimer {
        timer: Timer::from_seconds(5.0, TimerMode::Repeating),
    });

    // Spawn the single boid for Iteration 0
    commands.spawn(BoidBundle::new(1, 400.0, 300.0));
    
    println!("🤖 Spawned initial boid");
    
    // Spawn a test player for now
    commands.spawn(PlayerBundle::new(
        PeerId::Local(0), // Fake peer ID for testing
        "TestPlayer".to_string(),
        200.0,
        300.0,
    ));
    println!("🎮 Spawned test player");
}

/// Handle client connections using Lightyear 0.21 observer pattern
fn handle_client_connect(trigger: Trigger<OnAdd, Connected>, mut commands: Commands) {
    let client_entity = trigger.target();
    println!("🔗 Client connected: {:?}", client_entity);
    
    // Add replication sender to the new client
    commands.entity(client_entity).insert(
        ReplicationSender::new(
            Duration::from_millis(100),
            SendUpdatesMode::SinceLastAck,
            false
        )
    );
}

fn handle_client_disconnect(trigger: Trigger<OnAdd, Disconnected>) {
    let client_entity = trigger.target();
    println!("❌ Client disconnected: {:?}", client_entity);
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
            println!("📊 Server running - Uptime: {:.1}s | Players: {} | Boids: {}", 
                     time.elapsed_secs(), player_count, boid_count);
        }
    }
}

fn move_players(
    time: Res<Time>,
    mut players: Query<
        (&mut Position, &Velocity),
        With<Player>,
    >,
) {
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

fn move_boids(
    time: Res<Time>,
    mut boids: Query<(&mut Position, &Velocity), With<Boid>>,
) {
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
        let nearest_player = players.iter()
            .min_by(|a, b| {
                let dist_a = (a.0.x - boid_pos.0.x).powi(2) + (a.0.y - boid_pos.0.y).powi(2);
                let dist_b = (b.0.x - boid_pos.0.x).powi(2) + (b.0.y - boid_pos.0.y).powi(2);
                dist_a.partial_cmp(&dist_b).unwrap()
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
struct GameState {
    next_player_id: u32,
}

// Shared plugin for common functionality
pub struct SharedPlugin;

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ProtocolPlugin);
    }
}
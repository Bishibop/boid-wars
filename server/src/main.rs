use bevy::prelude::*;
use boid_wars_shared::*;
use lightyear::server::config::ServerConfig;
use lightyear::server::plugin::ServerPlugins;

fn main() {
    println!("🚀 Boid Wars Server Starting...");

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ServerPlugins::new(ServerConfig::default()))
        .add_plugins(ProtocolPlugin)
        .add_plugins(BoidWarsServerPlugin)
        .run();
}

// Server-specific plugin
pub struct BoidWarsServerPlugin;

impl Plugin for BoidWarsServerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameState>();

        // Add startup system to spawn server
        app.add_systems(Startup, setup_server);

        // Connection handling will be added later once we have working basic server

        // Add game systems
        app.add_systems(Update, log_status);
        app.add_systems(FixedUpdate, (move_players, move_boids, update_boid_ai));
    }
}

fn setup_server(mut commands: Commands) {
    println!("✅ Server initialized");

    // No need to spawn NetcodeServer entity - it's handled by ServerPlugins

    // Create status timer
    commands.spawn(StatusTimer {
        timer: Timer::from_seconds(5.0, TimerMode::Repeating),
    });

    // Spawn the single boid for Iteration 0
    commands.spawn(BoidBundle::new(1, 400.0, 300.0));

    println!("🤖 Spawned initial boid");

    // Don't spawn test player - wait for real clients to connect
    println!("🌐 Lightyear server ready for connections");
}

// Connection handling will be added later

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
            println!(
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
    _next_player_id: u32,
}

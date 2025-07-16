use bevy::prelude::*;
use boid_wars_shared::*;
use lightyear::prelude::client::*;
use lightyear::prelude::SharedConfig;
use std::net::SocketAddr;
use tracing::info;
use wasm_bindgen::prelude::*;

// Setup panic hook for better error messages in browser console
#[wasm_bindgen]
pub fn run() {
    console_error_panic_hook::set_once();

    let mut app = App::new();

    // Add Bevy plugins optimized for WASM (disable audio to avoid browser warnings)
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Boid Wars - Bevy WASM Client".into(),
                    resolution: (800.0, 600.0).into(),
                    canvas: Some("#bevy-canvas".into()),
                    fit_canvas_to_parent: true,
                    prevent_default_event_handling: false,
                    ..default()
                }),
                ..default()
            })
            .disable::<bevy::audio::AudioPlugin>(),
    );

    // Add Lightyear client plugins
    let lightyear_config = create_client_config();
    app.add_plugins(ClientPlugins::new(lightyear_config));

    // Add shared protocol
    app.add_plugins(ProtocolPlugin);

    // Initialize performance timer
    let client_settings = &*CLIENT_CONFIG;
    app.insert_resource(PerformanceTimer(Timer::from_seconds(
        client_settings.performance_log_interval,
        TimerMode::Repeating,
    )));

    // Add systems
    app.add_systems(Startup, (setup_scene, connect_to_server));
    app.add_systems(
        Update,
        (
            performance_monitor,
            handle_connection_events,
            render_networked_entities,
            sync_position_to_transform,
            send_player_input,
        ),
    );

    // Run the app
    app.run();
}

// Configuration is now loaded from the shared config system

/// Create Lightyear client configuration
fn create_client_config() -> lightyear::prelude::client::ClientConfig {
    info!("🔧 Creating client config for WebSocket connection...");

    let network_config = &*NETWORK_CONFIG;
    let server_addr: SocketAddr = network_config
        .server_addr
        .parse()
        .expect("Failed to parse server address");
    info!("📡 Will connect to server: {}", server_addr);

    // Use WebSocket (no certificates needed!)
    let transport = ClientTransport::WebSocketClient { server_addr };
    let io = IoConfig::from_transport(transport);

    // Use Netcode auth with matching key and protocol
    let net_config = NetConfig::Netcode {
        config: NetcodeConfig::default(),
        io,
        auth: Authentication::Manual {
            server_addr,
            client_id: 1,
            private_key: network_config.dev_key,
            protocol_id: network_config.protocol_id,
        },
    };

    lightyear::prelude::client::ClientConfig {
        shared: SharedConfig::default(),
        net: net_config,
        replication: Default::default(),
        packet: Default::default(),
        ping: Default::default(),
        interpolation: Default::default(),
        prediction: Default::default(),
        sync: Default::default(),
    }
}

/// Scene setup
fn setup_scene(mut commands: Commands, _asset_server: Res<AssetServer>) {
    // Spawn a 2D camera
    commands.spawn(Camera2d);

    info!("📷 Camera ready - waiting for server entities...");
}

/// Performance monitoring timer resource
#[derive(Resource)]
struct PerformanceTimer(Timer);

/// Simple performance monitoring system
fn performance_monitor(
    time: Res<Time>,
    mut perf_timer: ResMut<PerformanceTimer>,
    players: Query<Entity, With<Player>>,
    boids: Query<Entity, With<Boid>>,
) {
    // Log performance at regular intervals using proper timer
    if perf_timer.0.tick(time.delta()).just_finished() {
        let player_count = players.iter().count();
        let boid_count = boids.iter().count();
        let fps = 1.0 / time.delta_secs();
        info!(
            "📊 Performance: {} players, {} boids, {:.1} FPS",
            player_count, boid_count, fps
        );
    }
}

/// Connect to the game server
fn connect_to_server(mut commands: Commands) {
    info!("🌐 Initiating connection to server...");

    commands.queue(|world: &mut World| {
        world.connect_client();
        info!("✅ Client connection initiated!");
    });
}

/// Handle connection events from Lightyear
fn handle_connection_events(
    mut connection_events: EventReader<ConnectEvent>,
    mut disconnect_events: EventReader<DisconnectEvent>,
) {
    for event in connection_events.read() {
        info!("✅ Connected to server! Client ID: {:?}", event.client_id());
    }

    for event in disconnect_events.read() {
        info!("❌ Disconnected from server: {:?}", event.reason);
    }
}

// Type aliases to simplify complex queries
type UnrenderedPlayer = (With<Player>, Without<Sprite>);
type UnrenderedBoid = (With<Boid>, Without<Sprite>);

/// Render networked entities (players and boids from server)
fn render_networked_entities(
    mut commands: Commands,
    players: Query<(Entity, &Position), UnrenderedPlayer>,
    boids: Query<(Entity, &Position), UnrenderedBoid>,
) {
    // Add visual representation to networked players
    for (entity, position) in players.iter() {
        commands.entity(entity).insert((
            Sprite::from_color(Color::srgb(0.0, 1.0, 0.0), Vec2::new(10.0, 10.0)),
            Transform::from_translation(Vec3::new(position.x, position.y, 1.0)),
        ));
    }

    // Add visual representation to networked boids
    for (entity, position) in boids.iter() {
        commands.entity(entity).insert((
            Sprite::from_color(Color::srgb(1.0, 0.0, 0.0), Vec2::new(8.0, 8.0)),
            Transform::from_translation(Vec3::new(position.x, position.y, 1.0)),
        ));
    }
}

/// Sync Position component to Transform for rendering
fn sync_position_to_transform(mut query: Query<(&Position, &mut Transform), Changed<Position>>) {
    for (position, mut transform) in query.iter_mut() {
        transform.translation.x = position.x;
        transform.translation.y = position.y;
    }
}

/// Send player input to server
fn send_player_input(mut connection: ResMut<ConnectionManager>, keys: Res<ButtonInput<KeyCode>>) {
    let mut movement = Vec2::ZERO;
    let fire = false;

    // Basic WASD movement
    if keys.pressed(KeyCode::KeyW) {
        movement.y += 1.0;
    }
    if keys.pressed(KeyCode::KeyS) {
        movement.y -= 1.0;
    }
    if keys.pressed(KeyCode::KeyA) {
        movement.x -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) {
        movement.x += 1.0;
    }

    // Normalize movement
    if movement.length() > 0.0 {
        movement = movement.normalize();
    }

    // For now, aim in the same direction as movement
    let aim = movement;

    let input = PlayerInput {
        movement,
        aim,
        fire,
    };

    // Send input to server as a message
    if let Err(e) = connection.send_message::<UnreliableChannel, PlayerInput>(&input) {
        info!("⚠️ Failed to send input: {:?}", e);
    }
}

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
                    prevent_default_event_handling: true,
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
    app.add_systems(Startup, (setup_scene, connect_to_server, setup_ui));
    app.add_systems(
        Update,
        (
            performance_monitor,
            handle_connection_events,
            render_networked_entities,
            sync_position_to_transform,
            send_player_input,
            debug_player_count,
            button_system,
        ),
    );

    // Run the app
    app.run();
}

// Configuration is now loaded from the shared config system

/// Create Lightyear client configuration
fn create_client_config() -> lightyear::prelude::client::ClientConfig {
    let network_config = &*NETWORK_CONFIG;
    let server_addr: SocketAddr = network_config
        .server_addr
        .parse()
        .expect("Failed to parse server address");

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
    // Spawn a 2D camera centered on the game area
    let game_config = &*GAME_CONFIG;
    commands.spawn((
        Camera2d,
        Transform::from_xyz(
            game_config.game_width / 2.0,
            game_config.game_height / 2.0,
            1000.0,
        ),
    ));

    // Add arena boundary visualization
    let boundary_color = Color::srgb(0.5, 0.5, 0.5);
    let boundary_width = 5.0;

    // Top boundary
    commands.spawn((
        Sprite::from_color(
            boundary_color,
            Vec2::new(game_config.game_width, boundary_width),
        ),
        Transform::from_xyz(
            game_config.game_width / 2.0,
            game_config.game_height - boundary_width / 2.0,
            0.0,
        ),
    ));

    // Bottom boundary
    commands.spawn((
        Sprite::from_color(
            boundary_color,
            Vec2::new(game_config.game_width, boundary_width),
        ),
        Transform::from_xyz(game_config.game_width / 2.0, boundary_width / 2.0, 0.0),
    ));

    // Left boundary
    commands.spawn((
        Sprite::from_color(
            boundary_color,
            Vec2::new(boundary_width, game_config.game_height),
        ),
        Transform::from_xyz(boundary_width / 2.0, game_config.game_height / 2.0, 0.0),
    ));

    // Right boundary
    commands.spawn((
        Sprite::from_color(
            boundary_color,
            Vec2::new(boundary_width, game_config.game_height),
        ),
        Transform::from_xyz(
            game_config.game_width - boundary_width / 2.0,
            game_config.game_height / 2.0,
            0.0,
        ),
    ));
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
    commands.queue(|world: &mut World| {
        world.connect_client();
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
type UnrenderedObstacle = (With<Obstacle>, Without<Sprite>);
type UnrenderedProjectile = (With<Projectile>, Without<Sprite>);

/// Render networked entities (players, boids, obstacles, and projectiles from server)
fn render_networked_entities(
    mut commands: Commands,
    players: Query<(Entity, &Position, &Player), UnrenderedPlayer>,
    boids: Query<(Entity, &Position), UnrenderedBoid>,
    obstacles: Query<(Entity, &Position, &Obstacle), UnrenderedObstacle>,
    projectiles: Query<(Entity, &Position), UnrenderedProjectile>,
) {
    // Add visual representation to networked players
    for (entity, position, _player) in players.iter() {
        commands.entity(entity).insert((
            Sprite::from_color(Color::srgb(0.0, 1.0, 0.0), Vec2::new(10.0, 10.0)), // Original small size
            Transform::from_translation(Vec3::new(position.x, position.y, 1.0)),
        ));
    }

    // Add visual representation to networked boids (includes AI players)
    for (entity, position) in boids.iter() {
        commands.entity(entity).insert((
            Sprite::from_color(Color::srgb(1.0, 0.0, 0.0), Vec2::new(8.0, 8.0)), // Original small size
            Transform::from_translation(Vec3::new(position.x, position.y, 1.0)),
        ));
    }

    // Add visual representation to networked obstacles
    for (entity, position, obstacle) in obstacles.iter() {
        commands.entity(entity).insert((
            Sprite::from_color(
                Color::srgb(0.5, 0.3, 0.1),
                Vec2::new(obstacle.width, obstacle.height),
            ), // Brown obstacles
            Transform::from_translation(Vec3::new(position.x, position.y, 0.5)), // Slightly behind other entities
        ));
    }

    // Add visual representation to networked projectiles
    for (entity, position) in projectiles.iter() {
        commands.entity(entity).insert((
            Sprite::from_color(Color::srgb(0.0, 1.0, 1.0), Vec2::new(6.0, 6.0)), // Cyan bullets, slightly larger
            Transform::from_translation(Vec3::new(position.x, position.y, 2.0)), // In front of other entities
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
fn send_player_input(
    mut connection: ResMut<ConnectionManager>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    players: Query<&Position, With<Player>>,
) {
    let mut movement = Vec2::ZERO;
    let fire = keys.pressed(KeyCode::Space) || mouse_buttons.pressed(MouseButton::Left);

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

    // Calculate aim direction from mouse position
    let mut aim = movement; // Fallback to movement direction

    if let (Ok(window), Ok((camera, camera_transform))) = (windows.single(), cameras.single()) {
        if let Some(cursor_pos) = window.cursor_position() {
            // Convert cursor position to world coordinates
            if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
                // Get player position (assume we're the first/only player for now)
                if let Ok(player_pos) = players.single() {
                    // Calculate direction from player to mouse
                    let direction = (world_pos - player_pos.0).normalize_or_zero();
                    if direction.length() > 0.1 {
                        // Only use mouse aim if it's valid
                        aim = direction;
                    }
                }
            }
        }
    }

    let input = PlayerInput::new(movement, aim, fire);

    // Removed debug logs

    // Removed debug logs

    // Send input to server as a message - no debug logs
    let _ = connection.send_message::<UnreliableChannel, PlayerInput>(&input);
}

/// Debug system to count players and their positions
fn debug_player_count(
    all_players: Query<(&Player, &Position, &Transform), With<Player>>,
    rendered_players: Query<&Player, With<Sprite>>,
    mut timer: Local<f32>,
    time: Res<Time>,
) {
    *timer += time.delta_secs();
    if *timer > 2.0 {
        *timer = 0.0;
        info!(
            "🔍 Total players: {} | Rendered players: {}",
            all_players.iter().count(),
            rendered_players.iter().count()
        );
    }
}

/// Component to mark the reset button
#[derive(Component)]
struct ResetButton;

/// Set up UI elements
fn setup_ui(mut commands: Commands) {
    // Create a simple button
    commands
        .spawn((
            Button,
            Node {
                width: Val::Px(150.0),
                height: Val::Px(50.0),
                position_type: PositionType::Absolute,
                left: Val::Px(10.0),
                top: Val::Px(10.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
            ResetButton,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Reset (R/Enter)"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
            ));
        });
}

/// Handle button clicks and reset
#[allow(clippy::type_complexity)]
fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<ResetButton>),
    >,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut connection: ResMut<ConnectionManager>,
) {
    let mut should_reset = false;

    for (interaction, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *color = BackgroundColor(Color::srgb(0.35, 0.75, 0.35));
                should_reset = true;
            }
            Interaction::Hovered => {
                *color = BackgroundColor(Color::srgb(0.25, 0.25, 0.25));
            }
            Interaction::None => {
                *color = BackgroundColor(Color::srgb(0.15, 0.15, 0.15));
            }
        }
    }

    // Also allow R key or Enter to reset
    if keyboard.just_pressed(KeyCode::KeyR) || keyboard.just_pressed(KeyCode::Enter) {
        should_reset = true;
    }

    if should_reset {
        // Send reset message to server
        let reset_message = ResetAIMessage;
        if let Err(e) = connection.send_message::<ReliableChannel, ResetAIMessage>(&reset_message) {
            warn!("Failed to send reset message: {:?}", e);
        }
    }
}

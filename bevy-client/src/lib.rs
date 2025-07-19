use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
use boid_wars_shared::*;
use lightyear::prelude::client::*;
use lightyear::prelude::SharedConfig;
use std::net::SocketAddr;
use tracing::info;
use wasm_bindgen::prelude::*;

// Health bar components
#[derive(Component)]
struct PlayerHealthBar;

#[derive(Component)]
struct HealthBarBackground;

#[derive(Component)]
struct HealthBarFill;

#[derive(Component)]
struct BoidHealthBar {
    owner: Entity,
}

#[derive(Component)]
struct HealthBarLink {
    background: Entity,
    fill: Entity,
}

// Client-side smooth interpolation component
#[derive(Component)]
struct SmoothTransform {
    previous_position: Vec2,
    target_position: Vec2,
    previous_rotation: f32,
    target_rotation: f32,
    interpolation_speed: f32,
    // Enhanced smoothing
    velocity: Vec2,
    angular_velocity: f32,
    smoothing_factor: f32,
}

// Setup panic hook for better error messages in browser console
#[wasm_bindgen]
pub fn run() {
    console_error_panic_hook::set_once();

    let mut app = App::new();

    // Add Bevy plugins optimized for WASM
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Boid Wars - Bevy WASM Client".into(),
                    resolution: (1200.0, 900.0).into(),
                    canvas: Some("#bevy-canvas".into()),
                    fit_canvas_to_parent: true,
                    prevent_default_event_handling: true,
                    ..default()
                }),
                ..default()
            })
            .set(bevy::render::RenderPlugin {
                synchronous_pipeline_compilation: true,
                ..default()
            })
            .set(AssetPlugin {
                // Disable .meta file loading to avoid 404 errors
                meta_check: AssetMetaCheck::Never,
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
            update_health_bars,
            update_boid_health_bar_positions,
            cleanup_health_bars,
            handle_camera_zoom,
            smooth_interpolation_system,
        ),
    );

    // Run the app
    app.run();
}

// Configuration is now loaded from the shared config system

/// Create Lightyear client configuration
fn create_client_config() -> lightyear::prelude::client::ClientConfig {
    let network_config = &*NETWORK_CONFIG;
    
    // Dynamically construct WebSocket URL based on environment and page protocol
    let server_addr: SocketAddr = if cfg!(debug_assertions) {
        // Development: always use localhost with ws://
        "127.0.0.1:8080".parse().expect("Failed to parse dev address")
    } else {
        // Production: construct address using page's host and port
        let window = web_sys::window().expect("Should have window");
        let location = window.location();
        
        let host = location.hostname()
            .unwrap_or_else(|_| "boid-wars.fly.dev".to_string());
            
        let protocol = location.protocol()
            .unwrap_or_else(|_| "https:".to_string());
            
        // Get the current page's port, or use standard ports
        let port = location.port()
            .ok()
            .and_then(|p| if p.is_empty() { None } else { Some(p) })
            .unwrap_or_else(|| {
                // If no port specified, use standard ports
                if protocol == "https:" { "443".to_string() } else { "80".to_string() }
            });
            
        info!("🔗 Detected page protocol: {}, host: {}, port: {}", protocol, host, port);
        
        // Try to parse the current host with detected port
        // If it fails, fall back to dummy IP (browser will use page host anyway)
        format!("{}:{}", host, port)
            .parse::<SocketAddr>()
            .unwrap_or_else(|_| {
                info!("⚠️  Could not parse host as IP, using fallback");
                // Use standard port in fallback too
                let fallback_port = if protocol == "https:" { 443 } else { 80 };
                format!("127.0.0.1:{}", fallback_port).parse().unwrap()
            })
    };

    info!("🔗 Client WebSocket config address: {}", server_addr);

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

/// UI setup for health bars
fn setup_ui(mut commands: Commands) {
    // Player health bar container
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(20.0),
                left: Val::Px(20.0),
                width: Val::Px(200.0),
                height: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
            PlayerHealthBar,
            HealthBarBackground,
        ))
        .with_children(|parent| {
            // Health bar fill
            parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.8, 0.2, 0.2)),
                HealthBarFill,
            ));
        });
}

/// Scene setup
fn setup_scene(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("Loading game sprites...");

    // Load player sprite texture - using Ship_01 Level 1
    let player_texture = asset_server.load("game-assets/sprites/Ship_LVL_1.png");
    commands.insert_resource(PlayerSprite(player_texture));

    // Load enemy sprite texture - using Pirate Ship 04
    let enemy_texture = asset_server.load("game-assets/sprites/Ship_04.png");
    commands.insert_resource(EnemySprite(enemy_texture));

    // Load projectile sprite - using craftpix laser
    let projectile_texture = asset_server.load("game-assets/sprites/laser1_small.png");
    commands.insert_resource(ProjectileSprite(projectile_texture));

    // Load background textures - using the derelict ship copy images
    let background1 = asset_server.load("game-assets/backgrounds/derelict_ship_main.png");
    let background2 = asset_server.load("game-assets/backgrounds/derelict_ship_2.png");
    let background3 = asset_server.load("game-assets/backgrounds/derelict_ship_3.png");

    // Spawn a 2D camera centered on the game area
    let game_config = &*GAME_CONFIG;

    // Spawn the three background derelict ships with offsets and random rotations
    let center_x = game_config.game_width * 0.5;
    let center_y = game_config.game_height * 0.5;
    let offset_distance = 1500.0;

    // First ship - top-left direction
    commands.spawn((
        Sprite {
            image: background1,
            color: Color::srgba(0.25, 0.25, 0.25, 1.0), // Dark overlay
            ..default()
        },
        Transform::from_xyz(
            center_x - offset_distance * 0.7,
            center_y + offset_distance * 0.7,
            1.0, // Background layer
        )
        .with_rotation(Quat::from_rotation_z(15.0_f32.to_radians())),
        Background,
    ));

    // Second ship - bottom direction
    commands.spawn((
        Sprite {
            image: background2,
            color: Color::srgba(0.25, 0.25, 0.25, 1.0), // Dark overlay
            ..default()
        },
        Transform::from_xyz(
            center_x,
            center_y - offset_distance,
            1.0, // Background layer
        )
        .with_rotation(Quat::from_rotation_z(-30.0_f32.to_radians())),
        Background,
    ));

    // Third ship - right direction
    commands.spawn((
        Sprite {
            image: background3,
            color: Color::srgba(0.25, 0.25, 0.25, 1.0), // Dark overlay
            ..default()
        },
        Transform::from_xyz(
            center_x + offset_distance,
            center_y + offset_distance * 0.2,
            1.0, // Background layer
        )
        .with_rotation(Quat::from_rotation_z(45.0_f32.to_radians())),
        Background,
    ));

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
            2.0,
        ),
    ));

    // Bottom boundary
    commands.spawn((
        Sprite::from_color(
            boundary_color,
            Vec2::new(game_config.game_width, boundary_width),
        ),
        Transform::from_xyz(game_config.game_width / 2.0, boundary_width / 2.0, 2.0),
    ));

    // Left boundary
    commands.spawn((
        Sprite::from_color(
            boundary_color,
            Vec2::new(boundary_width, game_config.game_height),
        ),
        Transform::from_xyz(boundary_width / 2.0, game_config.game_height / 2.0, 2.0),
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
            2.0,
        ),
    ));
}

/// Performance monitoring timer resource
#[derive(Resource)]
struct PerformanceTimer(Timer);

/// Resource to hold player sprite texture
#[derive(Resource)]
struct PlayerSprite(Handle<Image>);

/// Resource to hold enemy sprite texture
#[derive(Resource)]
struct EnemySprite(Handle<Image>);

/// Resource to hold projectile sprite texture
#[derive(Resource)]
struct ProjectileSprite(Handle<Image>);

/// Marker component for background entities
#[derive(Component)]
struct Background;

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
    info!("🚀 Attempting to connect to server...");
    commands.queue(|world: &mut World| {
        world.connect_client();
        info!("📡 Connection request sent to server");
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
    player_sprite: Res<PlayerSprite>,
    enemy_sprite: Res<EnemySprite>,
    projectile_sprite: Res<ProjectileSprite>,
    asset_server: Res<AssetServer>,
    players: Query<(Entity, &Position, &Rotation, &Player, Option<&Velocity>), UnrenderedPlayer>,
    boids: Query<(Entity, &Position, Option<&Rotation>, Option<&Velocity>), UnrenderedBoid>,
    obstacles: Query<(Entity, &Position, &Obstacle), UnrenderedObstacle>,
    projectiles: Query<(Entity, &Position, Option<&Velocity>), UnrenderedProjectile>,
) {
    // Check if sprites are loaded
    let player_loaded = asset_server.is_loaded(&player_sprite.0);
    let enemy_loaded = asset_server.is_loaded(&enemy_sprite.0);

    if !player_loaded || !enemy_loaded {
        info!(
            "Waiting for sprites to load... Player: {}, Enemy: {}",
            player_loaded, enemy_loaded
        );
        return;
    }
    // Add visual representation to networked players
    for (entity, position, rotation, _player, _velocity) in players.iter() {
        commands.entity(entity).insert((
            Sprite {
                image: player_sprite.0.clone(),
                custom_size: Some(Vec2::new(48.0, 48.0)), // Set explicit size
                ..default()
            },
            Transform::from_translation(Vec3::new(position.x, position.y, 3.0))
                .with_rotation(Quat::from_rotation_z(rotation.angle)),
        ));
    }

    // Add visual representation to networked boids (includes AI players)
    for (entity, position, rotation, velocity) in boids.iter() {
        // Use velocity direction if available and significant, otherwise use rotation
        let angle = if let Some(vel) = velocity {
            if vel.length_squared() > 0.1 {
                vel.y.atan2(vel.x) - std::f32::consts::FRAC_PI_2 // Subtract 90 degrees to face forward
            } else if let Some(rot) = rotation {
                rot.angle
            } else {
                0.0
            }
        } else if let Some(rot) = rotation {
            rot.angle
        } else {
            0.0
        };

        commands.entity(entity).insert((
            Sprite {
                image: enemy_sprite.0.clone(),
                custom_size: Some(Vec2::new(24.0, 24.0)), // Set explicit size for enemies
                ..default()
            },
            Transform::from_translation(Vec3::new(position.x, position.y, 3.0))
                .with_rotation(Quat::from_rotation_z(angle)),
        ));

        // Spawn health bar for boid
        let health_bar_bg = commands
            .spawn((
                Sprite::from_color(Color::srgb(0.2, 0.2, 0.2), Vec2::new(20.0, 3.0)),
                Transform::from_translation(Vec3::new(position.x, position.y + 15.0, 1.5)),
                BoidHealthBar { owner: entity },
            ))
            .id();

        let health_bar_fill = commands
            .spawn((
                Sprite::from_color(Color::srgb(0.8, 0.2, 0.2), Vec2::new(20.0, 3.0)),
                Transform::from_translation(Vec3::new(position.x, position.y + 15.0, 1.6)),
                BoidHealthBar { owner: entity },
                HealthBarFill,
            ))
            .id();

        // Store health bar references on the boid entity
        commands.entity(entity).insert(HealthBarLink {
            background: health_bar_bg,
            fill: health_bar_fill,
        });
    }

    // Add visual representation to networked obstacles
    for (entity, position, obstacle) in obstacles.iter() {
        commands.entity(entity).insert((
            Sprite::from_color(
                Color::srgb(0.5, 0.3, 0.1),
                Vec2::new(obstacle.width, obstacle.height),
            ), // Brown obstacles
            Transform::from_translation(Vec3::new(position.x, position.y, 2.5)), // Slightly behind other entities
        ));
    }

    // Add visual representation to networked projectiles
    for (entity, position, velocity) in projectiles.iter() {
        // Calculate rotation from velocity for projectiles
        let angle = if let Some(vel) = velocity {
            if vel.length_squared() > 0.1 {
                vel.y.atan2(vel.x) - std::f32::consts::FRAC_PI_2 // Subtract 90 degrees to face forward
            } else {
                0.0
            }
        } else {
            0.0
        };

        commands.entity(entity).insert((
            Sprite {
                image: projectile_sprite.0.clone(),
                custom_size: Some(Vec2::new(18.0, 18.0)), // Projectile size
                ..default()
            },
            Transform::from_translation(Vec3::new(position.x, position.y, 4.0)) // In front of other entities
                .with_rotation(Quat::from_rotation_z(angle)),
        ));
    }
}

/// Sync Position and Rotation components to Transform for rendering
fn sync_position_to_transform(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &Position,
            Option<&Rotation>,
            Option<&Velocity>,
            &mut Transform,
            Option<&Player>,
            Option<&mut SmoothTransform>,
        ),
        Or<(Changed<Position>, Changed<Rotation>, Changed<Velocity>)>,
    >,
) {
    for (entity, position, rotation, velocity, mut transform, player, smooth_transform) in
        query.iter_mut()
    {
        // Check if this is a boid that needs smooth interpolation
        let is_boid = player.is_none(); // Non-players are boids

        if is_boid {
            // Handle smooth interpolation for boids
            if let Some(mut smooth) = smooth_transform {
                // Update targets
                smooth.previous_position = smooth.target_position;
                smooth.target_position = position.0;

                // Calculate target rotation
                let target_rotation = if let Some(vel) = velocity {
                    if vel.length_squared() > 0.1 {
                        vel.y.atan2(vel.x) - std::f32::consts::FRAC_PI_2
                    } else if let Some(rot) = rotation {
                        rot.angle
                    } else {
                        smooth.target_rotation
                    }
                } else if let Some(rot) = rotation {
                    rot.angle
                } else {
                    smooth.target_rotation
                };

                smooth.previous_rotation = smooth.target_rotation;
                smooth.target_rotation = target_rotation;
            } else {
                // First time seeing this boid - add SmoothTransform component
                let target_rotation = if let Some(vel) = velocity {
                    if vel.length_squared() > 0.1 {
                        vel.y.atan2(vel.x) - std::f32::consts::FRAC_PI_2
                    } else if let Some(rot) = rotation {
                        rot.angle
                    } else {
                        0.0
                    }
                } else if let Some(rot) = rotation {
                    rot.angle
                } else {
                    0.0
                };

                commands.entity(entity).insert(SmoothTransform {
                    previous_position: position.0,
                    target_position: position.0,
                    previous_rotation: target_rotation,
                    target_rotation,
                    interpolation_speed: 8.0, // Slower for more smoothness
                    velocity: Vec2::ZERO,
                    angular_velocity: 0.0,
                    smoothing_factor: 0.85, // Higher = more smoothing
                });
            }
        } else {
            // Players get direct updates (no smoothing for player)
            transform.translation.x = position.x;
            transform.translation.y = position.y;

            // Player: always use rotation component (aim direction)
            let angle = if let Some(rot) = rotation {
                rot.angle
            } else {
                transform.rotation.to_euler(EulerRot::ZYX).0 // Keep current rotation
            };

            transform.rotation = Quat::from_rotation_z(angle);
        }
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

    // Send input to server as a message
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

/// Update health bars based on entity health
fn update_health_bars(
    // Query for the main player's health bar
    mut health_bar_query: Query<&mut Node, (With<HealthBarFill>, With<PlayerHealthBar>)>,
    // Query for boid health bars
    mut boid_fill_query: Query<(&mut Sprite, &BoidHealthBar), With<HealthBarFill>>,
    // Query for player health using Health component
    player_query: Query<&Health, With<Player>>,
    // Query for boid health
    boid_query: Query<&Health, With<Boid>>,
) {
    // Update player health bar
    for health in player_query.iter().take(1) {
        for mut health_bar in health_bar_query.iter_mut().take(1) {
            let health_percentage = (health.current / health.max).clamp(0.0, 1.0);
            health_bar.width = Val::Percent(health_percentage * 100.0);
        }
    }

    // Update boid health bars
    for (mut sprite, health_bar) in boid_fill_query.iter_mut() {
        if let Ok(health) = boid_query.get(health_bar.owner) {
            let health_percentage = (health.current / health.max).clamp(0.0, 1.0);
            sprite.custom_size = Some(Vec2::new(20.0 * health_percentage, 3.0));

            // Optionally hide health bar if at full health
            if health_percentage >= 1.0 {
                sprite.color = Color::srgba(0.8, 0.2, 0.2, 0.0); // Make transparent
            } else {
                sprite.color = Color::srgba(0.8, 0.2, 0.2, 1.0); // Make visible
            }
        }
    }
}

/// Update boid health bar positions to follow boids
fn update_boid_health_bar_positions(
    boid_query: Query<(&Transform, &HealthBarLink), With<Boid>>,
    mut health_bar_query: Query<&mut Transform, (With<BoidHealthBar>, Without<Boid>)>,
) {
    for (boid_transform, health_link) in boid_query.iter() {
        // Update background position
        if let Ok(mut bar_transform) = health_bar_query.get_mut(health_link.background) {
            bar_transform.translation.x = boid_transform.translation.x;
            bar_transform.translation.y = boid_transform.translation.y + 15.0;
        }

        // Update fill position
        if let Ok(mut bar_transform) = health_bar_query.get_mut(health_link.fill) {
            bar_transform.translation.x = boid_transform.translation.x;
            bar_transform.translation.y = boid_transform.translation.y + 15.0;
        }
    }
}

/// Clean up health bars when entities are removed
fn cleanup_health_bars(
    mut commands: Commands,
    mut removed_boids: RemovedComponents<Boid>,
    health_bar_query: Query<(Entity, &BoidHealthBar)>,
) {
    for removed_boid in removed_boids.read() {
        // Find and despawn health bars associated with removed boids
        for (entity, health_bar) in health_bar_query.iter() {
            if health_bar.owner == removed_boid {
                commands.entity(entity).despawn();
            }
        }
    }
}

/// Enhanced smooth interpolation system for boids with velocity-based smoothing
fn smooth_interpolation_system(
    mut query: Query<
        (&mut Transform, &mut SmoothTransform),
        (With<SmoothTransform>, Without<Player>),
    >,
    time: Res<Time>,
) {
    let delta_time = time.delta_secs();

    for (mut transform, mut smooth) in query.iter_mut() {
        let current_pos = Vec2::new(transform.translation.x, transform.translation.y);
        let current_rotation = transform.rotation.to_euler(EulerRot::ZYX).0;

        // Calculate distance to target for adaptive smoothing
        let distance_to_target = (smooth.target_position - current_pos).length();

        // Adaptive smoothing - less smoothing when far from target, more when close
        let adaptive_factor = (distance_to_target / 100.0).clamp(0.2, 1.0);
        let base_lerp_factor = smooth.interpolation_speed * delta_time * adaptive_factor;

        // Velocity-based smoothing for position
        let target_velocity = (smooth.target_position - current_pos) * smooth.interpolation_speed;
        smooth.velocity = smooth.velocity.lerp(target_velocity, base_lerp_factor);

        // Apply velocity with additional smoothing
        let velocity_factor = smooth.smoothing_factor;
        let new_pos = current_pos + smooth.velocity * delta_time * velocity_factor;

        // Additional exponential smoothing towards target
        let final_pos = new_pos.lerp(smooth.target_position, base_lerp_factor * 0.3);

        transform.translation.x = final_pos.x;
        transform.translation.y = final_pos.y;

        // Enhanced rotation smoothing with angular velocity
        let mut rotation_diff = smooth.target_rotation - current_rotation;

        // Handle rotation wrap-around (shortest path between angles)
        while rotation_diff > std::f32::consts::PI {
            rotation_diff -= 2.0 * std::f32::consts::PI;
        }
        while rotation_diff < -std::f32::consts::PI {
            rotation_diff += 2.0 * std::f32::consts::PI;
        }

        // Angular velocity smoothing
        let target_angular_velocity = rotation_diff * smooth.interpolation_speed;
        smooth.angular_velocity = smooth.angular_velocity * smooth.smoothing_factor
            + target_angular_velocity * (1.0 - smooth.smoothing_factor);

        // Apply angular velocity with damping
        let new_rotation = current_rotation + smooth.angular_velocity * delta_time * 0.7;
        transform.rotation = Quat::from_rotation_z(new_rotation);
    }
}

/// Handle camera zoom with mouse wheel
fn handle_camera_zoom(
    mut scroll_evr: EventReader<bevy::input::mouse::MouseWheel>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
) {
    for ev in scroll_evr.read() {
        if let Ok(mut transform) = camera_query.single_mut() {
            // Zoom in/out with scroll wheel
            let zoom_delta = ev.y * 0.1;
            let current_scale = transform.scale.x;
            let new_scale = (current_scale - zoom_delta).clamp(0.5, 5.0);
            transform.scale = Vec3::splat(new_scale);
        }
    }
}

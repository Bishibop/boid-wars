use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
use boid_wars_shared::*;
use lightyear::prelude::client::*;
use lightyear::prelude::SharedConfig;
use lightyear::client::message::ReceiveMessage;
use std::net::SocketAddr;
use tracing::{info, warn};
use wasm_bindgen::prelude::*;

mod health_events;
use health_events::HealthEventsPlugin;

// Constants
const PLAYER_SPRITE_SIZE: f32 = 64.0; // Actual sprite size after optimization
const BOID_SPRITE_SIZE: f32 = 32.0; // Actual boid sprite size
const PROJECTILE_SPRITE_SIZE: f32 = 18.0; // Actual projectile sprite size

// Client-side components
#[derive(Component)]
struct LocalPlayer;

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

// Client-side projectile tracking
#[derive(Resource, Default)]
struct ClientProjectileTracker {
    // Map network ID to local entity
    projectiles: std::collections::HashMap<u32, Entity>,
}

// Projectile sprite pool for performance
#[derive(Resource)]
struct ProjectileSpritePool {
    available: Vec<Entity>,
    active: std::collections::HashMap<u32, Entity>, // network_id -> entity
    sprite_texture: Handle<Image>,
    max_size: usize,
}

impl ProjectileSpritePool {
    fn new(max_size: usize) -> Self {
        Self {
            available: Vec::with_capacity(max_size),
            active: std::collections::HashMap::with_capacity(max_size),
            sprite_texture: Handle::default(),
            max_size,
        }
    }
    
    fn get(&mut self) -> Option<Entity> {
        self.available.pop()
    }
    
    fn return_entity(&mut self, entity: Entity) {
        self.available.push(entity);
    }
    
    fn is_full(&self) -> bool {
        self.active.len() + self.available.len() >= self.max_size
    }
}

// Component for client-side projectile simulation
#[derive(Component)]
struct ClientProjectile {
    #[allow(dead_code)]
    network_id: u32,
    velocity: Vec2,
    #[allow(dead_code)]
    owner_id: u64,
    #[allow(dead_code)]
    is_boid_projectile: bool,
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
                    prevent_default_event_handling: false,
                    ..default()
                }),
                ..default()
            })
            .set(bevy::render::RenderPlugin {
                synchronous_pipeline_compilation: false,
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
    let (lightyear_config, client_id) = create_client_config();
    app.insert_resource(MyClientId(client_id));
    app.add_plugins(ClientPlugins::new(lightyear_config));

    // Add shared protocol
    app.add_plugins(ProtocolPlugin);
    
    // Add health events handling
    app.add_plugins(HealthEventsPlugin);

    // Initialize performance timer
    let client_settings = &*CLIENT_CONFIG;
    app.insert_resource(PerformanceTimer(Timer::from_seconds(
        client_settings.performance_log_interval,
        TimerMode::Repeating,
    )));

    // Initialize debug settings
    app.init_resource::<DebugSettings>();

    // Initialize client projectile tracker
    app.init_resource::<ClientProjectileTracker>();
    
    // Initialize projectile sprite pool
    app.insert_resource(ProjectileSpritePool::new(500)); // Match server pool size

    // Add systems with proper ordering
    app.add_systems(Startup, (setup_scene, connect_to_server, setup_ui));
    app.add_systems(Startup, setup_projectile_pool.after(setup_scene));
    app.add_systems(
        Update,
        (
            performance_monitor,
            handle_connection_events,
            mark_local_player,
            handle_projectile_spawn_events,
            handle_projectile_despawn_events,
            update_client_projectiles,
            render_networked_entities,
            sync_position_to_transform,
            update_player_rotation_to_mouse,
            send_player_input,
            debug_player_count,
            update_health_bars,
            update_boid_health_bar_positions,
            cleanup_health_bars,
            // handle_camera_zoom, // Removed mouse scroll zoom
            smooth_interpolation_system,
            toggle_debug_display,
            debug_collision_system,
        ),
    );

    // Run the app
    app.run();
}

// Configuration is now loaded from the shared config system

/// Create Lightyear client configuration
fn create_client_config() -> (lightyear::prelude::client::ClientConfig, u64) {
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
    // Generate a unique client ID with full timestamp + large random component to minimize collision risk
    let client_id = (js_sys::Date::now() as u64) * 10000 + (js_sys::Math::random() * 10000.0) as u64;

    let net_config = NetConfig::Netcode {
        config: NetcodeConfig::default(),
        io,
        auth: Authentication::Manual {
            server_addr,
            client_id,
            private_key: network_config.dev_key,
            protocol_id: network_config.protocol_id,
        },
    };

    let config = lightyear::prelude::client::ClientConfig {
        shared: SharedConfig::default(),
        net: net_config,
        replication: Default::default(),
        packet: Default::default(),
        ping: Default::default(),
        interpolation: Default::default(),
        prediction: Default::default(),
        sync: Default::default(),
    };
    
    (config, client_id)
}

/// UI setup for health bars
fn setup_ui(mut commands: Commands) {
    // Player health bar container with border
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(20.0),
                left: Val::Px(20.0),
                width: Val::Px(204.0), // Increased for border
                height: Val::Px(24.0), // Increased for border
                border: UiRect::all(Val::Px(2.0)), // 2px border all around
                ..default()
            },
            BackgroundColor(Color::srgb(1.0, 1.0, 1.0)), // White border
            BorderColor(Color::srgb(1.0, 1.0, 1.0)), // White border color
            PlayerHealthBar,
        ))
        .with_children(|parent| {
            // Health bar background (black)
            parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.0, 0.0, 0.0)), // Black background
                HealthBarBackground,
            ))
            .with_children(|parent| {
                // Health bar fill (red)
                parent.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.8, 0.2, 0.2)), // Red fill
                    HealthBarFill,
                ));
            });
        });
}

/// Check if WebP is supported by the browser
fn supports_webp() -> bool {
    // Use web-sys to check WebP support
    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            if let Ok(canvas) = document.create_element("canvas") {
                if let Ok(canvas) = canvas.dyn_into::<web_sys::HtmlCanvasElement>() {
                    canvas.set_width(1);
                    canvas.set_height(1);
                    if let Some(ctx) = canvas.get_context("2d").ok().flatten() {
                        if let Ok(_ctx) = ctx.dyn_into::<web_sys::CanvasRenderingContext2d>() {
                            // Try to create a WebP data URL
                            return canvas.to_data_url_with_type("image/webp").is_ok();
                        }
                    }
                }
            }
        }
    }
    false
}

/// Load image with WebP fallback to PNG
fn load_image_with_fallback(asset_server: &AssetServer, path: &str) -> Handle<Image> {
    let extension = if supports_webp() { "webp" } else { "png" };
    let full_path = format!("{}.{}", path.trim_end_matches(".png").trim_end_matches(".webp"), extension);
    asset_server.load(full_path)
}

/// Scene setup
fn setup_scene(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("Loading game sprites...");
    info!("WebP support: {}", supports_webp());

    // Load player sprite texture - using Ship_01 Level 1
    let player_texture = load_image_with_fallback(&asset_server, "sprites/Ship_LVL_1");
    commands.insert_resource(PlayerSprite(player_texture));

    // Load player 2 sprite texture
    let player2_texture = load_image_with_fallback(&asset_server, "sprites/Ship_player_2");
    commands.insert_resource(Player2Sprite(player2_texture));

    // Load enemy sprite texture - using Pirate Ship 04
    let enemy_texture = load_image_with_fallback(&asset_server, "sprites/Ship_04");
    commands.insert_resource(EnemySprite(enemy_texture));

    // Load projectile sprite - using craftpix laser
    let projectile_texture = load_image_with_fallback(&asset_server, "sprites/laser1_small");
    commands.insert_resource(ProjectileSprite(projectile_texture));

    // Spawn a 2D camera centered on the game area
    let game_config = &*GAME_CONFIG;
    let center_x = game_config.game_width * 0.5;
    let center_y = game_config.game_height * 0.5;

    // Load starfield background (behind everything)
    let starfield_texture = load_image_with_fallback(&asset_server, "backgrounds/starfield_background");
    commands.spawn((
        Sprite {
            image: starfield_texture,
            ..default()
        },
        Transform::from_translation(Vec3::new(center_x, center_y, 0.5)), // Behind everything
    ));

    // Load spaceship background (on top of starfield)
    let spaceship_texture = load_image_with_fallback(&asset_server, "backgrounds/angled_bg");
    commands.spawn((
        Sprite {
            image: spaceship_texture,
            color: Color::srgba(1.0, 1.0, 1.0, 0.7), // Slight opacity
            ..default()
        },
        Transform::from_translation(Vec3::new(
            center_x + (game_config.game_width * 0.3),  // 30% right
            center_y - (game_config.game_height * 0.3), // 30% down
            1.0  // On top of starfield but behind game entities
        )),
    ));

    commands.spawn((
        Camera2d,
        Transform::from_xyz(
            game_config.game_width / 2.0,
            game_config.game_height / 2.0,
            1000.0,
        ),
    ));

    // Arena boundary visualization removed for cleaner appearance
}

/// Setup projectile pool with pre-allocated entities
fn setup_projectile_pool(
    mut commands: Commands,
    mut pool: ResMut<ProjectileSpritePool>,
    projectile_sprite: Res<ProjectileSprite>,
) {
    info!("Pre-allocating projectile pool with 100 entities...");
    
    // Store the sprite texture in the pool
    pool.sprite_texture = projectile_sprite.0.clone();
    
    // Pre-allocate 100 projectile entities
    for _ in 0..100 {
        let entity = commands.spawn((
            ClientProjectile {
                network_id: 0,
                velocity: Vec2::ZERO,
                owner_id: 0,
                is_boid_projectile: false,
            },
            Sprite {
                image: projectile_sprite.0.clone(),
                custom_size: Some(Vec2::splat(PROJECTILE_SPRITE_SIZE)),
                ..default()
            },
            Transform::from_translation(Vec3::new(-1000.0, -1000.0, 14.0)), // Off-screen
            Visibility::Hidden, // Hidden by default
        )).id();
        
        pool.available.push(entity);
    }
    
    info!("Projectile pool initialized with {} entities", pool.available.len());
}

/// Performance monitoring timer resource
#[derive(Resource)]
struct PerformanceTimer(Timer);

/// Store our client ID
#[derive(Resource)]
struct MyClientId(u64);


/// Resource to hold player sprite texture
#[derive(Resource)]
struct PlayerSprite(Handle<Image>);

/// Resource to hold player 2 sprite texture
#[derive(Resource)]
struct Player2Sprite(Handle<Image>);

/// Resource to hold enemy sprite texture
#[derive(Resource)]
struct EnemySprite(Handle<Image>);

/// Resource to hold projectile sprite texture
#[derive(Resource)]
struct ProjectileSprite(Handle<Image>);

/// Debug settings for collision visualization
#[derive(Resource)]
struct DebugSettings {
    show_collisions: bool,
    collision_color: Color,
    player_scale: f32,
    boid_scale: f32,
}

impl Default for DebugSettings {
    fn default() -> Self {
        Self {
            show_collisions: false,
            collision_color: Color::srgba(0.0, 1.0, 0.0, 0.5), // Semi-transparent green
            player_scale: 1.0, // 1.0 = actual size, 2.0 = double size, etc.
            boid_scale: 1.0,   // Separate scale for boids
        }
    }
}

/// Component to mark collision outline entities
#[derive(Component)]
struct CollisionOutline {
    entity: Entity,  // The entity this outline belongs to
    is_player: bool, // true for player, false for boid
}


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
        let client_id = event.client_id();
        info!("✅ Connected to server! Client ID: {:?}", client_id);
    }

    for event in disconnect_events.read() {
        info!("❌ Disconnected from server: {:?}", event.reason);
    }
}

/// Mark the local player entity
fn mark_local_player(
    my_client_id: Res<MyClientId>,
    mut commands: Commands,
    players: Query<(Entity, &Player), Without<LocalPlayer>>,
) {
    // Get our client ID from the resource
    let our_id = my_client_id.0;
    
    // Find and mark our player
    for (entity, player) in players.iter() {
        if player.id == our_id {
            commands.entity(entity).insert(LocalPlayer);
        }
    }
}

// Type aliases to simplify complex queries
type UnrenderedPlayer = (With<Player>, Without<Sprite>);
type UnrenderedBoid = (With<Boid>, Without<Sprite>);
type UnrenderedObstacle = (With<Obstacle>, Without<Sprite>);
type UnrenderedProjectile = (With<Projectile>, Without<Sprite>);

/// Render networked entities (players, boids, obstacles, and projectiles from server)
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn render_networked_entities(
    mut commands: Commands,
    player_sprite: Res<PlayerSprite>,
    player2_sprite: Res<Player2Sprite>,
    enemy_sprite: Res<EnemySprite>,
    projectile_sprite: Res<ProjectileSprite>,
    asset_server: Res<AssetServer>,
    players: Query<
        (
            Entity,
            &Position,
            Option<&Rotation>,
            &Player,
            Option<&Velocity>,
            Option<&PlayerNumber>,
        ),
        UnrenderedPlayer,
    >,
    boids: Query<(Entity, &Position, Option<&Rotation>, Option<&Velocity>), UnrenderedBoid>,
    obstacles: Query<(Entity, &Position, &Obstacle), UnrenderedObstacle>,
    projectiles: Query<(Entity, &Position, Option<&Velocity>), UnrenderedProjectile>,
) {
    // Check if sprites are loaded
    let player_loaded = asset_server.is_loaded(&player_sprite.0);
    let player2_loaded = asset_server.is_loaded(&player2_sprite.0);
    let enemy_loaded = asset_server.is_loaded(&enemy_sprite.0);

    if !player_loaded || !player2_loaded || !enemy_loaded {
        info!(
            "Waiting for sprites to load... Player: {}, Player2: {}, Enemy: {}",
            player_loaded, player2_loaded, enemy_loaded
        );
        return;
    }
    // Add visual representation to networked players
    for (entity, position, rotation, _player, _velocity, player_number) in players.iter() {
        let mut transform = Transform::from_translation(Vec3::new(position.x, position.y, 13.0));

        // Apply rotation if available
        if let Some(rot) = rotation {
            transform = transform.with_rotation(Quat::from_rotation_z(
                rot.angle - std::f32::consts::FRAC_PI_2,
            ));
        }

        // Choose sprite based on player number
        let sprite_handle = match player_number {
            Some(PlayerNumber::Player1) => player_sprite.0.clone(),
            Some(PlayerNumber::Player2) => player2_sprite.0.clone(),
            None => player_sprite.0.clone(), // Default to player 1 sprite if no number
        };

        // Add visual components
        commands.entity(entity).insert((
            Sprite {
                image: sprite_handle,
                custom_size: Some(Vec2::splat(PLAYER_SPRITE_SIZE)),
                ..default()
            },
            transform,
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
                custom_size: Some(Vec2::splat(BOID_SPRITE_SIZE)),
                ..default()
            },
            Transform::from_translation(Vec3::new(position.x, position.y, 13.0))
                .with_rotation(Quat::from_rotation_z(angle)),
        ));

        // Spawn health bar for boid
        let health_bar_bg = commands
            .spawn((
                Sprite::from_color(Color::srgb(0.2, 0.2, 0.2), Vec2::new(20.0, 3.0)),
                Transform::from_translation(Vec3::new(position.x, position.y + 15.0, 11.5)),
                BoidHealthBar { owner: entity },
            ))
            .id();

        let health_bar_fill = commands
            .spawn((
                Sprite::from_color(Color::srgb(0.8, 0.2, 0.2), Vec2::new(20.0, 3.0)),
                Transform::from_translation(Vec3::new(position.x, position.y + 15.0, 11.6)),
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
            Transform::from_translation(Vec3::new(position.x, position.y, 12.5)), // Slightly behind other entities
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
                custom_size: Some(Vec2::splat(PROJECTILE_SPRITE_SIZE)),
                ..default()
            },
            Transform::from_translation(Vec3::new(position.x, position.y, 14.0)) // In front of other entities
                .with_rotation(Quat::from_rotation_z(angle)),
        ));
    }
}

/// Sync Position and Rotation components to Transform for rendering
#[allow(clippy::type_complexity)]
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
            Option<&Projectile>,
            Option<&mut SmoothTransform>,
            Option<&LocalPlayer>,
        ),
        Or<(Changed<Position>, Changed<Rotation>, Changed<Velocity>)>,
    >,
) {
    for (
        entity,
        position,
        rotation,
        velocity,
        mut transform,
        player,
        projectile,
        smooth_transform,
        local_player,
    ) in query.iter_mut()
    {
        // Check if this entity needs smooth interpolation (only boids, not players or projectiles)
        let needs_smoothing = player.is_none() && projectile.is_none();

        if needs_smoothing {
            // Handle smooth interpolation for boids only
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
            // Players and projectiles get position updates
            if player.is_some() && local_player.is_none() {
                // Remote players get light interpolation for 30Hz updates
                let target_pos = position.0;
                let current_pos = transform.translation.truncate();
                
                // Smooth position interpolation (10% per frame)
                let smoothed_pos = current_pos.lerp(target_pos, 0.1);
                transform.translation.x = smoothed_pos.x;
                transform.translation.y = smoothed_pos.y;
            } else {
                // Local players and projectiles get direct updates
                transform.translation.x = position.x;
                transform.translation.y = position.y;
            }

            // Handle rotation for projectiles
            if projectile.is_some() {
                // Use velocity direction for projectile rotation
                if let Some(vel) = velocity {
                    if vel.length_squared() > 0.1 {
                        let angle = vel.y.atan2(vel.x) - std::f32::consts::FRAC_PI_2;
                        transform.rotation = Quat::from_rotation_z(angle);
                    }
                }
            }

            // Sync rotation from server for remote players only
            // (Local player rotation is handled by update_player_rotation_to_mouse)
            if let Some(rot) = rotation {
                if player.is_some() && local_player.is_none() {
                    // Remote players get light interpolation for smoother 30Hz updates
                    let target_rotation = rot.angle;
                    let current_rotation = transform.rotation.to_euler(EulerRot::ZYX).0;
                    
                    // Smooth rotation interpolation (15% per frame)
                    let angle_diff = (target_rotation - current_rotation).rem_euclid(std::f32::consts::TAU);
                    let interpolated_angle = if angle_diff > std::f32::consts::PI {
                        current_rotation - (std::f32::consts::TAU - angle_diff) * 0.15
                    } else {
                        current_rotation + angle_diff * 0.15
                    };
                    
                    transform.rotation = Quat::from_rotation_z(interpolated_angle);
                }
            }
        }
    }
}

/// Update player sprite rotation to face mouse cursor
#[allow(clippy::type_complexity)]
fn update_player_rotation_to_mouse(
    mut players: Query<(&Position, &mut Transform), (With<Player>, With<LocalPlayer>)>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
) {
    if let (Ok(window), Ok((camera, camera_transform))) = (windows.single(), cameras.single()) {
        if let Some(cursor_pos) = window.cursor_position() {
            // Convert cursor position to world coordinates
            if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
                // Update rotation for local player only
                for (position, mut transform) in players.iter_mut() {
                    // Calculate direction from player to mouse
                    let direction = (world_pos - position.0).normalize_or_zero();
                    if direction.length() > 0.1 {
                        // Calculate angle and apply sprite offset
                        let angle = direction.y.atan2(direction.x) - std::f32::consts::FRAC_PI_2;
                        transform.rotation = Quat::from_rotation_z(angle);
                    }
                }
            }
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
    players: Query<&Position, (With<Player>, With<LocalPlayer>)>,
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
    let mut aim = Vec2::ZERO; // Default to no aim

    if let (Ok(window), Ok((camera, camera_transform))) = (windows.single(), cameras.single()) {
        if let Some(cursor_pos) = window.cursor_position() {
            // Convert cursor position to world coordinates
            if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
                // Get local player position
                if let Ok(player_pos) = players.single() {
                    // Calculate direction from player to mouse
                    let direction = (world_pos - player_pos.0).normalize_or_zero();
                    if direction.length() > 0.1 {
                        aim = direction;
                    }
                }
            }
        }
    }

    // Only fallback to movement direction if no valid mouse aim was calculated
    if aim.length() < 0.1 && movement.length() > 0.1 {
        aim = movement;
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
    // Query for all players with health
    player_query: Query<(&Health, &Player), With<Player>>,
    // Query for boid health
    boid_query: Query<&Health, With<Boid>>,
    // Get our client ID
    my_client_id: Res<MyClientId>,
    mut log_timer: Local<f32>,
    time: Res<Time>,
) {
    // Log every second
    *log_timer += time.delta_secs();
    let should_log = *log_timer >= 1.0;
    if should_log {
        *log_timer = 0.0;
    }
    
    // Get our client ID
    let our_id = my_client_id.0;
    
    // Find our player by matching IDs
    let mut found_local_player = false;
    for (health, player) in player_query.iter() {
        if player.id == our_id {
            found_local_player = true;
            if should_log {
                info!("Local player health: {}/{}", health.current, health.max);
            }
            for mut health_bar in health_bar_query.iter_mut() {
                let health_percentage = (health.current / health.max).clamp(0.0, 1.0);
                health_bar.width = Val::Percent(health_percentage * 100.0);
                if should_log {
                    info!("Health bar updated to {}%", health_percentage * 100.0);
                }
            }
            break;
        }
    }
    
    if !found_local_player && should_log {
        info!("No local player found for health bar update (our ID: {})", our_id);
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
            bar_transform.translation.y = boid_transform.translation.y + 18.0;
        }

        // Update fill position
        if let Ok(mut bar_transform) = health_bar_query.get_mut(health_link.fill) {
            bar_transform.translation.x = boid_transform.translation.x;
            bar_transform.translation.y = boid_transform.translation.y + 18.0;
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
#[allow(clippy::type_complexity)]
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

/// Handle camera zoom with mouse wheel (DISABLED)
/*
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
*/

/// Toggle debug display with C key
fn toggle_debug_display(
    keys: Res<ButtonInput<KeyCode>>,
    mut debug_settings: ResMut<DebugSettings>,
) {
    if keys.just_pressed(KeyCode::KeyC) {
        debug_settings.show_collisions = !debug_settings.show_collisions;
        info!(
            "Debug collision display: {}",
            debug_settings.show_collisions
        );
    }
}

/// Consolidated debug collision system that handles creation, updates, and cleanup
fn debug_collision_system(
    mut commands: Commands,
    debug_settings: Res<DebugSettings>,
    mut collision_outlines: Query<(Entity, &mut Sprite, &mut Transform, &CollisionOutline)>,
    players: Query<(Entity, &Position), With<Player>>,
    boids: Query<(Entity, &Position), With<Boid>>,
) {
    if !debug_settings.show_collisions {
        // Remove all collision outlines when debug is disabled
        for (outline_entity, _, _, _) in collision_outlines.iter() {
            commands.entity(outline_entity).despawn();
        }
        return;
    }

    // Track which entities have outlines
    let mut outlined_entities = std::collections::HashSet::new();
    let mut outlines_to_remove = Vec::new();

    // Update existing outlines (position, size, color) and remove orphaned ones
    for (outline_entity, mut sprite, mut transform, outline) in collision_outlines.iter_mut() {
        // Try to find the target entity and update position
        let mut found = false;

        if let Ok((_, position)) = players.get(outline.entity) {
            transform.translation.x = position.x;
            transform.translation.y = position.y;
            found = true;
            outlined_entities.insert(outline.entity);
        } else if let Ok((_, position)) = boids.get(outline.entity) {
            transform.translation.x = position.x;
            transform.translation.y = position.y;
            found = true;
            outlined_entities.insert(outline.entity);
        }

        if !found {
            outlines_to_remove.push(outline_entity);
        } else {
            // Update size based on scale settings
            let size = if outline.is_player {
                PLAYER_SPRITE_SIZE * debug_settings.player_scale
            } else {
                BOID_SPRITE_SIZE * debug_settings.boid_scale
            };
            sprite.custom_size = Some(Vec2::new(size, size));
            sprite.color = debug_settings.collision_color;
        }
    }

    // Remove orphaned outlines
    for entity in outlines_to_remove {
        commands.entity(entity).despawn();
    }

    // Create outlines for new players
    for (entity, position) in players.iter() {
        if !outlined_entities.contains(&entity) {
            let size = PLAYER_SPRITE_SIZE * debug_settings.player_scale;
            commands.spawn((
                Sprite {
                    color: debug_settings.collision_color,
                    custom_size: Some(Vec2::new(size, size)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(position.x, position.y, 20.0)),
                CollisionOutline {
                    entity,
                    is_player: true,
                },
            ));
        }
    }

    // Create outlines for new boids (circular colliders)
    for (entity, position) in boids.iter() {
        if !outlined_entities.contains(&entity) {
            let size = BOID_SPRITE_SIZE * debug_settings.boid_scale;
            commands.spawn((
                Sprite {
                    color: debug_settings.collision_color,
                    custom_size: Some(Vec2::new(size, size)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(position.x, position.y, 20.0)),
                CollisionOutline {
                    entity,
                    is_player: false,
                },
            ));
        }
    }
}

/// Handle projectile spawn events from server
fn handle_projectile_spawn_events(
    mut commands: Commands,
    mut message_events: EventReader<ReceiveMessage<ProjectileSpawnEvent>>,
    mut tracker: ResMut<ClientProjectileTracker>,
    mut pool: ResMut<ProjectileSpritePool>,
    mut query: Query<(&mut Transform, &mut ClientProjectile, &mut Visibility)>,
) {
    // Receive all projectile spawn events
    for message_event in message_events.read() {
        let event = &message_event.message;
        
        // Try to get an entity from the pool
        let entity = if let Some(pooled_entity) = pool.get() {
            pooled_entity
        } else if !pool.is_full() {
            // Create new entity if pool not at max capacity
            commands.spawn((
                ClientProjectile {
                    network_id: 0,
                    velocity: Vec2::ZERO,
                    owner_id: 0,
                    is_boid_projectile: false,
                },
                Sprite {
                    image: pool.sprite_texture.clone(),
                    custom_size: Some(Vec2::splat(PROJECTILE_SPRITE_SIZE)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(-1000.0, -1000.0, 14.0)),
                Visibility::Hidden,
            )).id()
        } else {
            warn!("Projectile pool exhausted!");
            continue;
        };
        
        // Update the entity with new projectile data
        if let Ok((mut transform, mut projectile, mut visibility)) = query.get_mut(entity) {
            // Update projectile data
            projectile.network_id = event.id;
            projectile.velocity = event.velocity;
            projectile.owner_id = event.owner_id;
            projectile.is_boid_projectile = event.is_boid_projectile;
            
            // Update position and make visible
            transform.translation.x = event.position.x;
            transform.translation.y = event.position.y;
            *visibility = Visibility::Visible;
            
            // Track the projectile
            pool.active.insert(event.id, entity);
            tracker.projectiles.insert(event.id, entity);
        }
    }
}

/// Handle projectile despawn events from server
fn handle_projectile_despawn_events(
    mut message_events: EventReader<ReceiveMessage<ProjectileDespawnEvent>>,
    mut tracker: ResMut<ClientProjectileTracker>,
    mut pool: ResMut<ProjectileSpritePool>,
    mut query: Query<(&mut Transform, &mut Visibility), With<ClientProjectile>>,
) {
    // Receive all projectile despawn events
    for message_event in message_events.read() {
        let event = &message_event.message;
        
        // Find the local projectile entity
        if let Some(entity) = tracker.projectiles.remove(&event.id) {
            // Remove from pool's active tracking
            pool.active.remove(&event.id);
            
            // Return entity to pool instead of despawning
            if let Ok((mut transform, mut visibility)) = query.get_mut(entity) {
                // Hide the entity and move it off-screen
                *visibility = Visibility::Hidden;
                transform.translation.x = -1000.0;
                transform.translation.y = -1000.0;
                
                // Return to available pool
                pool.return_entity(entity);
            }
        }
    }
}

/// Update client-side projectile positions based on velocity
fn update_client_projectiles(
    mut projectiles: Query<(&mut Transform, &ClientProjectile)>,
    time: Res<Time>,
) {
    let delta = time.delta_secs();
    let game_config = &*GAME_CONFIG;
    
    for (mut transform, projectile) in projectiles.iter_mut() {
        // Update position based on velocity
        transform.translation.x += projectile.velocity.x * delta;
        transform.translation.y += projectile.velocity.y * delta;
        
        // Update rotation to match velocity direction
        if projectile.velocity.length_squared() > 0.1 {
            let angle = projectile.velocity.y.atan2(projectile.velocity.x) - std::f32::consts::FRAC_PI_2;
            transform.rotation = Quat::from_rotation_z(angle);
        }
        
        // Check bounds and mark for removal if out of bounds
        let pos = transform.translation.truncate();
        if pos.x < 0.0 || pos.x > game_config.game_width || 
           pos.y < 0.0 || pos.y > game_config.game_height {
            // Note: We don't despawn here - we wait for the server's despawn event
            // This prevents desync between client and server
        }
    }
}

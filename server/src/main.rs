use bevy::app::PluginGroupBuilder;
use bevy::prelude::*;
use boid_wars_shared::*;
use lightyear::connection::id::ClientId;
use lightyear::prelude::server::*;
use lightyear::prelude::{MessageSend, NetworkTarget, SharedConfig};
use lightyear::server::message::ReceiveMessage;
use std::net::SocketAddr;
use tracing::{info, warn};

// Camera2dBundle should be in prelude

pub mod config;
pub mod debug_ui;
pub mod despawn_utils;
pub mod flocking;
pub mod groups;
pub mod health_sync;
pub mod physics;
pub mod pool;
pub mod position_sync;
pub mod spatial_grid;
use bevy_rapier2d::prelude::{Collider, ExternalForce, ExternalImpulse, RigidBody};
use config::PhysicsConfig;
use debug_ui::DebugUIPlugin;
use health_sync::HealthSyncPlugin;
use physics::{GameCollisionGroups, PhysicsPlugin, Ship, WeaponStats};
use position_sync::{PositionSyncPlugin, SyncPosition};
use spatial_grid::SpatialGridPlugin;

/// Returns the appropriate base plugins depending on whether we're in debug or release mode
fn get_base_plugins() -> PluginGroupBuilder {
    #[cfg(debug_assertions)]
    {
        // In debug mode, use DefaultPlugins with a window for the debug UI
        DefaultPlugins.build().set(WindowPlugin {
            primary_window: Some(Window {
                title: "Boid Wars Server (Debug)".to_string(),
                resolution: (1800.0, 1350.0).into(),
                ..default()
            }),
            ..default()
        })
    }

    #[cfg(not(debug_assertions))]
    {
        // In release mode, use DefaultPlugins but disable windowing
        DefaultPlugins.build().disable::<WindowPlugin>()
    }
}

fn main() {
    // Initialize logging first
    tracing_subscriber::fmt::init();

    info!("🚀 Starting Boid Wars server...");
    info!(
        "🔧 Bevy plugins: {}",
        if cfg!(debug_assertions) {
            "DefaultPlugins (debug)"
        } else {
            "DefaultPlugins (headless)"
        }
    );

    // Load configuration
    let network_config = &*NETWORK_CONFIG;
    let game_config = &*GAME_CONFIG;

    // Configure server address
    info!(
        "📡 Parsing server bind address: {}",
        network_config.server_bind_addr
    );
    let server_addr: SocketAddr = network_config
        .server_bind_addr
        .parse()
        .expect("Failed to parse server bind address");

    info!(
        "🌐 Server will listen on {} | Game area: {}x{}",
        server_addr, game_config.game_width, game_config.game_height
    );

    info!("🎯 Protocol ID: {}", network_config.protocol_id);

    // Create server config
    info!("⚙️  Creating Lightyear server configuration...");
    let lightyear_config = create_websocket_config(server_addr, network_config);

    info!("🎮 Building Bevy app with plugins...");
    let mut app = App::new();

    info!("🔌 Adding base plugins...");
    app.add_plugins(get_base_plugins());

    info!("🔌 Adding game plugins...");
    app.add_plugins(DebugUIPlugin)
        .add_plugins(ServerPlugins::new(lightyear_config))
        .add_plugins(ProtocolPlugin)
        .add_plugins(SpatialGridPlugin) // Must be before systems that use it
        .add_plugins(PhysicsPlugin::default())
        .add_plugins(PositionSyncPlugin)
        .add_plugins(HealthSyncPlugin) // Event-based health synchronization
        .add_plugins(flocking::FlockingPlugin) // Add flocking behavior
        .add_plugins(groups::BoidGroupPlugin)
        .add_plugins(BoidWarsServerPlugin);

    info!("🚀 Starting Bevy app...");
    app.run();
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
        app.add_systems(
            Update,
            (
                handle_connections,
                handle_disconnections,
                handle_player_input,
                handle_player_ready,
                send_game_state_updates,
                check_start_game,
            ),
        );

        // Add game systems
        app.add_systems(Update, (log_status, spawn_collision_objects_delayed));
        // Note: Physics systems (player_input_system, etc.) are added by PhysicsPlugin in FixedUpdate
    }
}

fn setup_server(mut commands: Commands) {
    info!("🌐 Starting Lightyear server...");

    // Start the Lightyear server
    commands.queue(|world: &mut World| {
        world.start_server();
        info!("✅ Lightyear server started successfully");
    });

    // Create status timer
    commands.insert_resource(StatusTimer(Timer::from_seconds(
        5.0, // Default status log interval
        TimerMode::Repeating,
    )));

    info!("⏰ Status timer configured (5s intervals)");

    // Initialize player slots
    commands.insert_resource(PlayerSlots::default());
}

// Spawn AI players when the game starts
fn spawn_collision_objects_delayed(
    mut commands: Commands,
    game_state: Res<GameState>,
    mut spawned: Local<bool>,
    physics_config: Res<PhysicsConfig>,
) {
    // Only spawn when game is in InGame phase and we haven't spawned yet
    if game_state.phase == boid_wars_shared::GamePhase::InGame && !*spawned {
        *spawned = true;
        // Spawn peaceful boids instead of AI players

        // Re-enabled to test boid synchronization
        spawn_boid_flock(&mut commands, &physics_config);
        spawn_static_obstacles(&mut commands);
    }
}

// Helper function to spawn peaceful boids using the group system
fn spawn_boid_flock(commands: &mut Commands, physics_config: &PhysicsConfig) {
    let _game_config = &*GAME_CONFIG;
    // Get resources
    let mut group_id_counter = groups::GroupIdCounter::default();
    let mut boid_id_counter = groups::BoidIdCounter::default();

    // Spawn groups in different zones
    let mut spawned_groups = 0;

    // Create a simple territory for testing (no complex generation)
    let _simple_territory = TerritoryData {
        center: Vec2::new(300.0, 300.0), // Just place it in the arena
        radius: 100.0,
        zone: ArenaZone::Outer,
        patrol_points: vec![
            // Simple patrol points
            Vec2::new(250.0, 250.0),
            Vec2::new(350.0, 250.0),
            Vec2::new(350.0, 350.0),
            Vec2::new(250.0, 350.0),
        ],
        neighboring_territories: vec![],
    };

    // Original Recon group (commented out - kept as backup)
    /*
    groups::spawn_boid_group(
        commands,
        GroupArchetype::Recon {
            detection_range: 300.0,
            flee_speed_bonus: 1.2,
        },
        20, // Small group for testing
        simple_territory,
        &mut group_id_counter,
        &mut boid_id_counter,
        physics_config,
    );
    */

    // First group: Assault group in center
    let assault_territory = TerritoryData {
        center: Vec2::new(800.0, 600.0), // Arena center
        radius: 400.0,
        zone: ArenaZone::Inner,
        patrol_points: vec![
            Vec2::new(600.0, 400.0),
            Vec2::new(1000.0, 400.0),
            Vec2::new(1000.0, 800.0),
            Vec2::new(600.0, 800.0),
        ],
        neighboring_territories: vec![],
    };

    groups::spawn_boid_group(
        commands,
        GroupArchetype::Assault {
            aggression_multiplier: 1.0,
            preferred_range: 150.0,
        },
        15,
        assault_territory,
        &mut group_id_counter,
        &mut boid_id_counter,
        physics_config,
    );
    spawned_groups += 1;

    // Second group: Defensive group in upper area
    let defensive_territory = TerritoryData {
        center: Vec2::new(800.0, 300.0), // Upper center
        radius: 300.0,
        zone: ArenaZone::Middle,
        patrol_points: vec![
            Vec2::new(500.0, 200.0),
            Vec2::new(1100.0, 200.0),
            Vec2::new(1100.0, 400.0),
            Vec2::new(500.0, 400.0),
        ],
        neighboring_territories: vec![],
    };

    groups::spawn_boid_group(
        commands,
        GroupArchetype::Defensive {
            protection_radius: 400.0,
            retreat_threshold: 0.4,
        },
        20,
        defensive_territory,
        &mut group_id_counter,
        &mut boid_id_counter,
        physics_config,
    );
    spawned_groups += 1;

    // Third group: Recon group patrolling outer edges
    let recon_territory = TerritoryData {
        center: Vec2::new(800.0, 900.0), // Lower area
        radius: 500.0,
        zone: ArenaZone::Outer,
        patrol_points: vec![
            Vec2::new(200.0, 700.0),
            Vec2::new(1400.0, 700.0),
            Vec2::new(1400.0, 1100.0),
            Vec2::new(200.0, 1100.0),
        ],
        neighboring_territories: vec![],
    };

    groups::spawn_boid_group(
        commands,
        GroupArchetype::Recon {
            detection_range: 400.0,
            flee_speed_bonus: 1.3,
        },
        12,
        recon_territory,
        &mut group_id_counter,
        &mut boid_id_counter,
        physics_config,
    );
    spawned_groups += 1;

    // Comment out other groups for now
    /*
    // Spawn Defensive groups in middle zone
    for territory in territories.iter().filter(|t| t.zone == ArenaZone::Middle).take(1) {
        groups::spawn_boid_group(
            commands,
            GroupArchetype::Defensive {
                protection_radius: 600.0,
                retreat_threshold: 0.4,
            },
            30, // Smaller groups for testing
            territory.clone(),
            &mut group_id_counter,
            &mut boid_id_counter,
            physics_config,
        );
        spawned_groups += 1;
    }
    */

    info!("Spawned {} boid groups with territories", spawned_groups);

    // Update resource counters
    commands.insert_resource(group_id_counter);
    commands.insert_resource(boid_id_counter);
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
    mut connections: EventReader<ConnectEvent>,
    mut player_slots: ResMut<PlayerSlots>,
    mut connection_manager: ResMut<ConnectionManager>,
    mut game_state: ResMut<GameState>,
) {
    let game_config = &*GAME_CONFIG;

    for event in connections.read() {
        let client_id = event.client_id;

        // Determine which player slot to assign
        let player_number = if player_slots.player1.is_none() {
            PlayerNumber::Player1
        } else if player_slots.player2.is_none() {
            PlayerNumber::Player2
        } else {
            // Server is full - send rejection message then disconnect
            info!("Server full: rejecting client {:?}", client_id);
            
            let server_full_msg = boid_wars_shared::ServerFullMessage {
                current_players: 2,
                max_players: 2,
                message: "Server is full (2/2 players). Please try again later.".to_string(),
            };
            
            // Send message to the specific client
            if let Err(e) = connection_manager
                .send_message_to_target::<boid_wars_shared::ReliableChannel, _>(
                    &server_full_msg,
                    NetworkTarget::Single(client_id),
                ) {
                warn!("Failed to send ServerFull message: {:?}", e);
            }
            
            // Client will disconnect themselves after receiving ServerFull message
            continue;
        };

        info!("Client {:?} connected as {:?}", client_id, player_number);

        // Store player slot assignment WITHOUT spawning yet
        // We'll use a placeholder entity until they're ready
        match player_number {
            PlayerNumber::Player1 => {
                player_slots.player1 = Some((client_id, Entity::PLACEHOLDER));
                // Force a change detection for immediate update
                game_state.set_changed();
                if player_slots.player2.is_some() {
                    // Both players connected, move to lobby phase
                    game_state.phase = boid_wars_shared::GamePhase::Lobby;
                    info!("Both players connected, entering lobby phase");
                }
            }
            PlayerNumber::Player2 => {
                player_slots.player2 = Some((client_id, Entity::PLACEHOLDER));
                // Force a change detection for immediate update
                game_state.set_changed();
                if player_slots.player1.is_some() {
                    // Both players connected, move to lobby phase
                    game_state.phase = boid_wars_shared::GamePhase::Lobby;
                    info!("Both players connected, entering lobby phase");
                }
            }
        }

        // Player spawning code moved to check_start_game system
        info!(
            "Player slot assigned for client {:?}, waiting for ready signal",
            client_id
        );
    }
}

// Handle client disconnections
fn handle_disconnections(
    mut commands: Commands,
    mut disconnections: EventReader<DisconnectEvent>,
    mut player_slots: ResMut<PlayerSlots>,
    mut game_state: ResMut<GameState>,
) {
    for event in disconnections.read() {
        let client_id = event.client_id;

        // Find and remove player from slots
        if let Some((stored_id, entity)) = player_slots.player1 {
            if stored_id == client_id {
                info!("Player 1 (client {:?}) disconnected", client_id);
                player_slots.player1 = None;
                game_state.player1_ready = false;
                
                // Only despawn if entity was actually spawned (not placeholder)
                if entity != Entity::PLACEHOLDER {
                    commands.entity(entity).despawn();
                }
            }
        }

        if let Some((stored_id, entity)) = player_slots.player2 {
            if stored_id == client_id {
                info!("Player 2 (client {:?}) disconnected", client_id);
                player_slots.player2 = None;
                game_state.player2_ready = false;
                
                // Only despawn if entity was actually spawned (not placeholder)
                if entity != Entity::PLACEHOLDER {
                    commands.entity(entity).despawn();
                }
            }
        }
        
        // Reset to waiting phase if any player disconnects
        if player_slots.player1.is_none() || player_slots.player2.is_none() {
            game_state.phase = boid_wars_shared::GamePhase::WaitingForPlayers;
            info!("Player disconnected, returning to waiting phase");
        }
    }
}

// Handle player input messages - update physics input properly
fn handle_player_input(
    mut message_events: EventReader<ReceiveMessage<boid_wars_shared::PlayerInput>>,
    mut players: Query<
        (
            &boid_wars_shared::Player,
            &mut physics::PlayerInput,
            &PlayerNumber,
        ),
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
        for (player, mut physics_input, _player_number) in players.iter_mut() {
            if player.id == client_id.to_bits() {
                // Both players now have full functionality
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

        debug!(
            "[Status] Players: {} | Boids: {} | Projectiles: {} | Entities: {}",
            player_count, boid_count, projectile_count, total_entities
        );
    }
}

#[derive(Resource)]
struct StatusTimer(Timer);

#[derive(Resource)]
struct GameState {
    phase: boid_wars_shared::GamePhase,
    player1_ready: bool,
    player2_ready: bool,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            phase: boid_wars_shared::GamePhase::WaitingForPlayers,
            player1_ready: false,
            player2_ready: false,
        }
    }
}

#[derive(Resource, Default)]
struct PlayerSlots {
    player1: Option<(ClientId, Entity)>,
    player2: Option<(ClientId, Entity)>,
}

// Handle player ready messages
fn handle_player_ready(
    mut message_events: EventReader<ReceiveMessage<boid_wars_shared::PlayerReady>>,
    mut game_state: ResMut<GameState>,
    player_slots: Res<PlayerSlots>,
) {
    for event in message_events.read() {
        let client_id = event.from;
        
        // Only process ready in lobby phase
        if game_state.phase != boid_wars_shared::GamePhase::Lobby {
            continue;
        }
        
        // Mark the appropriate player as ready
        if let Some((stored_id, _)) = player_slots.player1 {
            if stored_id == client_id {
                game_state.player1_ready = true;
                info!("Player 1 is ready!");
            }
        }
        
        if let Some((stored_id, _)) = player_slots.player2 {
            if stored_id == client_id {
                game_state.player2_ready = true;
                info!("Player 2 is ready!");
            }
        }
    }
}

// Send game state updates to all clients
fn send_game_state_updates(
    game_state: Res<GameState>,
    player_slots: Res<PlayerSlots>,
    mut connection_manager: ResMut<ConnectionManager>,
) {
    // Always send updates when game state or player slots change
    if !game_state.is_changed() && !player_slots.is_changed() {
        return;
    }
    
    // Count connected players
    let player_count = match (player_slots.player1.is_some(), player_slots.player2.is_some()) {
        (true, true) => 2,
        (true, false) | (false, true) => 1,
        (false, false) => 0,
    };
    
    let update = boid_wars_shared::GameStateUpdate {
        phase: game_state.phase.clone(),
        player_count,
        player1_ready: game_state.player1_ready,
        player2_ready: game_state.player2_ready,
    };
    
    // Send to all connected clients
    if let Err(e) = connection_manager
        .send_message_to_target::<boid_wars_shared::ReliableChannel, _>(
            &update,
            NetworkTarget::All,
        ) {
        warn!("Failed to send game state update: {:?}", e);
    }
}

// Check if we should start the game
fn check_start_game(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    mut player_slots: ResMut<PlayerSlots>,
    physics_config: Res<PhysicsConfig>,
) {
    // Only check in lobby phase
    if game_state.phase != boid_wars_shared::GamePhase::Lobby {
        return;
    }
    
    // Check if both players are ready
    if !game_state.player1_ready || !game_state.player2_ready {
        return;
    }
    
    info!("Both players ready! Starting game...");
    
    // Spawn player 1
    if let Some((client_id, ref mut entity)) = player_slots.player1.as_mut() {
        let player_entity = spawn_player(
            &mut commands,
            &physics_config,
            *client_id,
            100.0, // spawn_x
            100.0, // spawn_y
            boid_wars_shared::PlayerNumber::Player1,
        );
        *entity = player_entity;
    }
    
    // Spawn player 2
    if let Some((client_id, ref mut entity)) = player_slots.player2.as_mut() {
        let game_config = &*GAME_CONFIG;
        let player_entity = spawn_player(
            &mut commands,
            &physics_config,
            *client_id,
            game_config.game_width - 100.0,  // spawn_x
            game_config.game_height - 100.0, // spawn_y
            boid_wars_shared::PlayerNumber::Player2,
        );
        *entity = player_entity;
    }
    
    // Move to in-game phase
    game_state.phase = boid_wars_shared::GamePhase::InGame;
    info!("Game started!");
}

// Helper function to spawn a player
fn spawn_player(
    commands: &mut Commands,
    physics_config: &PhysicsConfig,
    client_id: ClientId,
    spawn_x: f32,
    spawn_y: f32,
    player_number: boid_wars_shared::PlayerNumber,
) -> Entity {
    let game_config = &*GAME_CONFIG;
    
    let player_entity = commands
        .spawn((
            PlayerBundle::new(
                client_id.to_bits(),
                format!("Player {}", client_id.to_bits()),
                spawn_x,
                spawn_y,
                player_number,
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
    
    // Add physics components
    commands.entity(player_entity).insert((
        physics::Player {
            player_id: client_id.to_bits(),
            ..Default::default()
        },
        physics::PlayerInput::default(),
        Ship::default(),
        WeaponStats::default(),
        boid_wars_shared::Health {
            current: game_config.default_health,
            max: game_config.default_health,
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
        Transform::from_xyz(spawn_x, spawn_y, 0.0),
        GlobalTransform::default(),
        bevy_rapier2d::dynamics::GravityScale(0.0),
        bevy_rapier2d::dynamics::Sleeping::disabled(),
        bevy_rapier2d::dynamics::Damping {
            linear_damping: 0.0,
            angular_damping: 0.0,
        },
        bevy_rapier2d::dynamics::AdditionalMassProperties::Mass(1.0),
        SyncPosition,
    ));
    
    info!("Player spawned at ({}, {}) for client {:?}", spawn_x, spawn_y, client_id);
    
    player_entity
}

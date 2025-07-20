use bevy::ecs::entity::MapEntities;
use bevy::prelude::*;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::GAME_CONFIG;

// Re-export Vec2 for use in other crates
pub use bevy::prelude::Vec2;

// Components

/// Player entity component
#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Player {
    pub id: u64,
    pub name: String,
}

/// Position component (replicated)
#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq, Deref, DerefMut)]
pub struct Position(pub Vec2);

impl Position {
    pub fn new(x: f32, y: f32) -> Self {
        Self(Vec2::new(x, y))
    }
}

/// Rotation component (replicated)
#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Rotation {
    pub angle: f32,
}

/// Simple boid entity for Iteration 0
#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Boid {
    pub id: u32,
}

/// Static combat capabilities for boids (replicated once)
#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct BoidCombatStats {
    pub damage: f32,
    pub fire_rate: f32,
    pub projectile_speed: f32,
    pub aggression_range: f32,
    pub spread_angle: f32,
}

impl Default for BoidCombatStats {
    fn default() -> Self {
        Self {
            damage: 5.0,             // Half of player damage
            fire_rate: 0.2,          // 1 shot every 5 seconds (much slower)
            projectile_speed: 400.0, // Slower than player (600)
            aggression_range: 200.0, // Detect players within 200 units
            spread_angle: 0.087,     // ~5 degrees in radians (much more accurate)
        }
    }
}

/// Dynamic combat state for boids (server-only, not replicated)
#[derive(Component, Clone, Debug)]
pub struct BoidCombatState {
    pub last_shot_time: f32,
}

impl Default for BoidCombatState {
    fn default() -> Self {
        Self {
            last_shot_time: 0.0,
        }
    }
}


/// Static obstacle component
#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Obstacle {
    pub id: u32,
    pub width: f32,
    pub height: f32,
}

/// Projectile component for network replication
#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Projectile {
    pub id: u32,
    pub damage: f32,
    pub owner_id: u64,
}

/// Velocity component for movement
#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq, Deref, DerefMut)]
pub struct Velocity(pub Vec2);

impl Velocity {
    pub fn new(x: f32, y: f32) -> Self {
        Self(Vec2::new(x, y))
    }
}

/// Health component
#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Default for Health {
    fn default() -> Self {
        let default_health = GAME_CONFIG.default_health;
        Self {
            current: default_health,
            max: default_health,
        }
    }
}

// Group System Components

/// Core group component for managing boid groups
#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct BoidGroup {
    pub id: u32,
    pub archetype: GroupArchetype,
    pub home_territory: TerritoryData,
    pub current_formation: Formation,
    pub behavior_state: GroupBehavior,
    #[serde(skip)]
    pub active_shooters: HashSet<Entity>,
    pub max_shooters: u8,
    pub initial_size: u32,
}

/// Boid membership component
#[derive(Component, Clone, Debug)]
pub struct BoidGroupMember {
    pub group_entity: Entity,
    pub group_id: u32,
    pub formation_slot: Option<FormationSlot>,
    pub role_in_group: BoidRole,
}

/// Formation slot identifier
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FormationSlot(pub usize);

/// Role a boid plays in its group
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum BoidRole {
    Leader,
    Flanker,
    Support,
    Scout,
}

/// Group archetypes with distinct behaviors
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum GroupArchetype {
    Assault {
        aggression_multiplier: f32,
        preferred_range: f32,
    },
    Defensive {
        protection_radius: f32,
        retreat_threshold: f32,
    },
    Recon {
        detection_range: f32,
        flee_speed_bonus: f32,
    },
}

/// Dynamic formations
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Formation {
    VFormation { 
        angle: f32, 
        spacing: f32,
        leader_boost: f32,
    },
    CircleDefense { 
        radius: f32,
        layers: u8,
        rotation_speed: f32,
    },
    SwarmAttack { 
        spread: f32,
        convergence_point: Vec2,
    },
    PatrolLine {
        length: f32,
        wave_amplitude: f32,
    },
}

impl Formation {
    pub fn default_for_archetype(archetype: &GroupArchetype) -> Self {
        match archetype {
            GroupArchetype::Assault { .. } => Formation::VFormation {
                angle: 45.0_f32.to_radians(),
                spacing: 30.0,
                leader_boost: 1.2,
            },
            GroupArchetype::Defensive { .. } => Formation::CircleDefense {
                radius: 80.0,
                layers: 2,
                rotation_speed: 0.5,
            },
            GroupArchetype::Recon { .. } => Formation::PatrolLine {
                length: 200.0,
                wave_amplitude: 50.0,
            },
        }
    }
}

/// Group AI states
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum GroupBehavior {
    Patrolling { 
        route: Vec<Vec2>, 
        current_waypoint: usize,
    },
    Engaging { 
        primary_target: u32, // Target player ID instead of Entity
        #[serde(skip)]
        secondary_targets: Vec<u32>,
    },
    Retreating { 
        rally_point: Vec2,
        speed_multiplier: f32,
    },
    Defending { 
        position: Vec2,
        radius: f32,
    },
}

/// Territory data for group home areas
#[derive(Component, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TerritoryData {
    pub center: Vec2,
    pub radius: f32,
    pub zone: ArenaZone,
    pub patrol_points: Vec<Vec2>,
    pub neighboring_territories: Vec<u32>,
}

/// Arena zones for territory placement
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum ArenaZone {
    Outer,  // Recon groups
    Middle, // Defensive groups  
    Inner,  // Assault groups
    Center, // Boss groups (future)
}

/// Group velocity for hierarchical movement
#[derive(Component, Clone, Debug, Serialize, Deserialize, PartialEq, Deref, DerefMut)]
pub struct GroupVelocity(pub Vec2);

/// Replicated group data for network optimization
#[derive(Component, Serialize, Deserialize, Clone, PartialEq)]
pub struct ReplicatedGroup {
    pub id: u32,
    pub position: Vec2,
    pub formation: Formation,
    pub member_count: u32,
    pub archetype: GroupArchetype,
}

// Inputs

/// Player input from client
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Reflect)]
pub struct PlayerInput {
    /// Movement direction (normalized)
    pub movement: Vec2,
    /// Aim direction (normalized)  
    pub aim: Vec2,
    /// Is firing
    pub fire: bool,
}

impl PlayerInput {
    /// Create a new PlayerInput
    pub fn new(movement: Vec2, aim: Vec2, fire: bool) -> Self {
        Self {
            movement,
            aim,
            fire,
        }
    }
}

// Messages

impl Default for PlayerInput {
    fn default() -> Self {
        Self {
            movement: Vec2::ZERO,
            aim: Vec2::ZERO,
            fire: false,
        }
    }
}

// Implement MapEntities for input (required by Lightyear)
impl MapEntities for PlayerInput {
    fn map_entities<M: EntityMapper>(&mut self, _entity_mapper: &mut M) {}
}

// Bundles

#[derive(Bundle)]
pub struct PlayerBundle {
    pub player: Player,
    pub position: Position,
    pub rotation: Rotation,
    pub velocity: Velocity,
    pub health: Health,
}

impl PlayerBundle {
    pub fn new(id: u64, name: String, x: f32, y: f32) -> Self {
        Self {
            player: Player { id, name },
            position: Position::new(x, y),
            rotation: Rotation { angle: 0.0 },
            velocity: Velocity::new(0.0, 0.0),
            health: Health::default(),
        }
    }
}

#[derive(Bundle)]
pub struct BoidBundle {
    pub boid: Boid,
    pub position: Position,
    pub velocity: Velocity,
    pub health: Health,
    pub combat_stats: BoidCombatStats,
    pub combat_state: BoidCombatState,
}

impl BoidBundle {
    pub fn new(id: u32, x: f32, y: f32) -> Self {
        Self {
            boid: Boid { id },
            position: Position::new(x, y),
            velocity: Velocity::new(0.0, 0.0),
            health: Health::default(),
            combat_stats: BoidCombatStats::default(),
            combat_state: BoidCombatState::default(),
        }
    }
}

// Channels - use Channel derive macro for Lightyear 0.20
#[derive(Channel)]
pub struct UnreliableChannel;

#[derive(Channel)]
pub struct ReliableChannel;

// Protocol Plugin
#[derive(Clone)]
pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        // Register types with Bevy
        app.register_type::<PlayerInput>();

        // Register components for replication using correct Lightyear 0.20 API
        // Server-authoritative components (unidirectional to save bandwidth)
        app.register_component::<Position>(ChannelDirection::ServerToClient);
        app.register_component::<Rotation>(ChannelDirection::ServerToClient);
        app.register_component::<Velocity>(ChannelDirection::ServerToClient);
        app.register_component::<Health>(ChannelDirection::ServerToClient);
        app.register_component::<Player>(ChannelDirection::ServerToClient);
        app.register_component::<Boid>(ChannelDirection::ServerToClient);
        app.register_component::<BoidCombatStats>(ChannelDirection::ServerToClient);
        // BoidCombatState is server-only and not registered for replication
        app.register_component::<Obstacle>(ChannelDirection::ServerToClient);
        app.register_component::<Projectile>(ChannelDirection::ServerToClient);
        
        // Group system components - NOT replicated to save bandwidth
        // Groups are server-side only for AI coordination

        // Register PlayerInput as message (not input plugin)
        app.register_message::<PlayerInput>(ChannelDirection::ClientToServer);

        // Register channels using correct Lightyear 0.20 API
        app.add_channel::<UnreliableChannel>(ChannelSettings {
            mode: ChannelMode::UnorderedUnreliableWithAcks,
            ..default()
        });

        app.add_channel::<ReliableChannel>(ChannelSettings {
            mode: ChannelMode::OrderedReliable(ReliableSettings::default()),
            ..default()
        });

        // AuthorityChange is automatically registered by Lightyear's SharedPlugin
        // No manual registration needed in Lightyear 0.20

        // Protocol plugin built successfully
    }
}

// Game constants moved to config system - use GAME_CONFIG at runtime

use bevy::ecs::entity::MapEntities;
use bevy::prelude::*;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

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
}

impl BoidBundle {
    pub fn new(id: u32, x: f32, y: f32) -> Self {
        Self {
            boid: Boid { id },
            position: Position::new(x, y),
            velocity: Velocity::new(0.0, 0.0),
            health: Health::default(),
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
        // Only register basic replication for now - no interpolation to avoid missing function errors
        app.register_component::<Position>(ChannelDirection::Bidirectional);
        app.register_component::<Rotation>(ChannelDirection::Bidirectional);
        app.register_component::<Velocity>(ChannelDirection::Bidirectional);
        app.register_component::<Health>(ChannelDirection::ServerToClient);
        app.register_component::<Player>(ChannelDirection::ServerToClient);
        app.register_component::<Boid>(ChannelDirection::ServerToClient);
        app.register_component::<Obstacle>(ChannelDirection::ServerToClient);
        app.register_component::<Projectile>(ChannelDirection::ServerToClient);

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

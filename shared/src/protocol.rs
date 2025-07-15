use bevy::prelude::*;
use bevy::ecs::entity::MapEntities;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

// Components

/// Player entity component
#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Player {
    pub id: PeerId,
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

// Access x and y directly
impl Position {
    pub fn x(&self) -> f32 {
        self.0.x
    }
    
    pub fn y(&self) -> f32 {
        self.0.y
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

/// Velocity component for movement
#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq, Deref, DerefMut)]
pub struct Velocity(pub Vec2);

impl Velocity {
    pub fn new(x: f32, y: f32) -> Self {
        Self(Vec2::new(x, y))
    }
}

// Access x and y directly
impl Velocity {
    pub fn x(&self) -> f32 {
        self.0.x
    }
    
    pub fn y(&self) -> f32 {
        self.0.y
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
        Self {
            current: 100.0,
            max: 100.0,
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
    pub fn new(id: PeerId, name: String, x: f32, y: f32) -> Self {
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

// Channels
#[derive(Debug)]
pub struct UnreliableChannel;

#[derive(Debug)]
pub struct ReliableChannel;

// Protocol Plugin
#[derive(Clone)]
pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        // Register types with Bevy
        app.register_type::<PlayerInput>();
        
        // TODO: Component registration might need to be done after ServerPlugins is added
        // Let's try a simpler approach for now
        
        // Basic component registration (if this fails, we'll move it to server setup)
        // app.register_component::<Player>();
        // app.register_component::<Position>();
        // app.register_component::<Rotation>();
        // app.register_component::<Velocity>();
        // app.register_component::<Health>();
        // app.register_component::<Boid>();
        
        println!("📋 Protocol plugin built - component registration TODO");
    }
}

// Game constants
pub const PLAYER_SPEED: f32 = 200.0;
pub const BOID_SPEED: f32 = 150.0;
pub const GAME_WIDTH: f32 = 800.0;
pub const GAME_HEIGHT: f32 = 600.0;
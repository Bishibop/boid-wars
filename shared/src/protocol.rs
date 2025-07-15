use serde::{Deserialize, Serialize};

/// Messages sent from client to server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    /// Client wants to join the game
    Join { name: String },
    /// Client input update
    Input(PlayerInput),
    /// Client is leaving
    Leave,
}

/// Messages sent from server to client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    /// Welcome message with player ID
    Welcome { player_id: u32 },
    /// Game state update
    StateUpdate(GameState),
    /// Player disconnected
    PlayerLeft { player_id: u32 },
    /// Error message
    Error { message: String },
}

/// Player input state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlayerInput {
    /// Movement direction (normalized)
    pub movement: Vec2,
    /// Aim direction (normalized)
    pub aim: Vec2,
    /// Is firing
    pub fire: bool,
}

/// Simple 2D vector
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

/// Game state snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    /// Current tick
    pub tick: u32,
    /// Player positions
    pub players: Vec<PlayerState>,
    /// For validation demo - simple entity
    pub test_entity: Option<TestEntity>,
}

/// Player state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerState {
    pub id: u32,
    pub position: Vec2,
    pub rotation: f32,
}

/// Test entity for validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestEntity {
    pub position: Vec2,
    pub radius: f32,
}

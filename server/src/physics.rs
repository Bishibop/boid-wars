// Re-export everything from the physics module
// This maintains backward compatibility while organizing code into modules

pub use self::core::*;

pub mod core;

// Decomposed physics modules for better organization
pub mod collision;
pub mod movement; 
pub mod combat;
pub mod input;

// Re-export key functions from decomposed modules
pub use collision::collision_system;
pub use movement::{player_movement_system, projectile_system};
pub use combat::{shooting_system, boid_shooting_system};
pub use input::{player_input_system, ai_player_system, swarm_communication_system};
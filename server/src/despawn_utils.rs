use crate::physics::Despawning;
use bevy::prelude::*;

/// Extension trait for safe entity despawning
pub trait SafeDespawnExt {
    fn safe_despawn(&mut self, entity: Entity);
}

impl<'w, 's> SafeDespawnExt for Commands<'w, 's> {
    /// Safely despawn an entity without panicking if it doesn't exist
    fn safe_despawn(&mut self, entity: Entity) {
        // Get a mutable reference to the entity commands
        if let Ok(mut entity_commands) = self.get_entity(entity) {
            // First mark it as despawning to prevent double-despawn attempts
            entity_commands.insert(Despawning);
        }

        // Note: The actual despawn/return to pool is handled by return_projectiles_to_pool system
        // For non-pooled entities, they will be despawned in cleanup_system
    }
}

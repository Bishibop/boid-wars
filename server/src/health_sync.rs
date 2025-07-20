use bevy::prelude::*;
use boid_wars_shared::*;
use lightyear::prelude::server::*;
use lightyear::prelude::{MessageSend, NetworkTarget};
use std::collections::HashMap;

/// Resource to track previous health values for change detection
#[derive(Resource, Default)]
pub struct HealthTracker {
    previous_health: HashMap<Entity, (f32, f32)>, // (current, max)
}

/// System to detect health changes and send events
pub fn detect_health_changes(
    mut connection: ResMut<ConnectionManager>,
    mut health_tracker: ResMut<HealthTracker>,
    health_query: Query<(Entity, &Health, Option<&Player>, Option<&Boid>), Changed<Health>>,
    mut removed: RemovedComponents<Health>,
) {
    // Handle removed components
    for entity in removed.read() {
        health_tracker.previous_health.remove(&entity);
    }

    // Check for health changes
    for (entity, health, player, boid) in health_query.iter() {
        let current = health.current;
        let max = health.max;
        
        // Check if this is a significant change
        let should_send = if let Some(&(prev_current, prev_max)) = health_tracker.previous_health.get(&entity) {
            // Send if health changed by more than 0.1 or max health changed
            (prev_current - current).abs() > 0.1 || (prev_max - max).abs() > 0.01
        } else {
            // First time seeing this entity, always send
            true
        };

        if should_send {
            // Determine entity ID for the event
            let entity_id = if let Some(player) = player {
                player.id as u32
            } else if let Some(boid) = boid {
                boid.id
            } else {
                // Skip entities without proper IDs
                continue;
            };

            // Create and send health change event
            let event = HealthChangeEvent {
                entity_id,
                new_health: current,
                max_health: max,
            };

            // Send to all connected clients
            let _ = connection.send_message_to_target::<ReliableChannel, _>(
                &event,
                NetworkTarget::All,
            );

            // Update tracker
            health_tracker.previous_health.insert(entity, (current, max));
        }
    }
}

/// System to send initial health state when entities spawn
pub fn send_initial_health(
    mut connection: ResMut<ConnectionManager>,
    mut health_tracker: ResMut<HealthTracker>,
    new_entities: Query<(Entity, &Health, Option<&Player>, Option<&Boid>), Added<Health>>,
) {
    for (entity, health, player, boid) in new_entities.iter() {
        // Determine entity ID
        let entity_id = if let Some(player) = player {
            player.id as u32
        } else if let Some(boid) = boid {
            boid.id
        } else {
            continue;
        };

        // Send initial health state
        let event = HealthChangeEvent {
            entity_id,
            new_health: health.current,
            max_health: health.max,
        };

        let _ = connection.send_message_to_target::<ReliableChannel, _>(
            &event,
            NetworkTarget::All,
        );

        // Track the initial state
        health_tracker.previous_health.insert(entity, (health.current, health.max));
    }
}

/// Plugin to handle health synchronization via events
pub struct HealthSyncPlugin;

impl Plugin for HealthSyncPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HealthTracker>()
            .add_systems(
                Update,
                (
                    send_initial_health,
                    detect_health_changes,
                )
                    .chain(),
            );
    }
}
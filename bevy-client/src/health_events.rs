use bevy::prelude::*;
use boid_wars_shared::*;
use lightyear::client::message::ReceiveMessage;

/// System to handle health change events from the server
pub fn handle_health_change_events(
    mut commands: Commands,
    mut message_events: EventReader<ReceiveMessage<HealthChangeEvent>>,
    mut player_query: Query<&mut Health, With<Player>>,
    mut boid_query: Query<&mut Health, (With<Boid>, Without<Player>)>,
    players: Query<(Entity, &Player)>,
    boids: Query<(Entity, &Boid), Without<Player>>,
) {
    for message_event in message_events.read() {
        let event = &message_event.message;

        // Try to find the entity by ID
        let mut found = false;

        // Check players
        for (entity, player) in players.iter() {
            if player.id == event.entity_id {
                // No cast needed, both are u64 now
                if let Ok(mut health) = player_query.get_mut(entity) {
                    health.current = event.new_health;
                    health.max = event.max_health;
                    found = true;
                    break;
                }
            }
        }

        // If not a player, check boids
        if !found {
            for (entity, boid) in boids.iter() {
                if boid.id as u64 == event.entity_id {
                    // Cast boid.id (u32) to u64 for comparison
                    if let Ok(mut health) = boid_query.get_mut(entity) {
                        health.current = event.new_health;
                        health.max = event.max_health;
                        found = true;
                        break;
                    } else {
                        // Entity doesn't have health component yet, add it
                        commands.entity(entity).insert(Health {
                            current: event.new_health,
                            max: event.max_health,
                        });
                        found = true;
                        break;
                    }
                }
            }
        }

        if !found {
            // This can happen if the entity hasn't been spawned on the client yet
            // This is normal due to network latency
        }
    }
}

/// Plugin to handle health synchronization via events on the client
pub struct HealthEventsPlugin;

impl Plugin for HealthEventsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_health_change_events);
    }
}

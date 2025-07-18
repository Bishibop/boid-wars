use bevy::prelude::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

/// Simple health state for tracking server health
#[derive(Resource, Clone)]
pub struct HealthState {
    pub is_healthy: Arc<AtomicBool>,
    pub player_count: Arc<AtomicUsize>,
    pub entity_count: Arc<AtomicUsize>,
}

impl Default for HealthState {
    fn default() -> Self {
        Self {
            is_healthy: Arc::new(AtomicBool::new(true)),
            player_count: Arc::new(AtomicUsize::new(0)),
            entity_count: Arc::new(AtomicUsize::new(0)),
        }
    }
}

impl HealthState {
    pub fn set_healthy(&self, healthy: bool) {
        self.is_healthy.store(healthy, Ordering::SeqCst);
    }

    pub fn update_player_count(&self, count: usize) {
        self.player_count.store(count, Ordering::SeqCst);
    }

    pub fn update_entity_count(&self, count: usize) {
        self.entity_count.store(count, Ordering::SeqCst);
    }

    pub fn is_healthy(&self) -> bool {
        self.is_healthy.load(Ordering::SeqCst)
    }

    pub fn get_player_count(&self) -> usize {
        self.player_count.load(Ordering::SeqCst)
    }

    pub fn get_entity_count(&self) -> usize {
        self.entity_count.load(Ordering::SeqCst)
    }
}

/// Plugin to manage health state
pub struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HealthState>()
            .add_systems(Update, update_health_metrics);
    }
}

/// System to update health metrics
fn update_health_metrics(
    health_state: Res<HealthState>,
    // You can add queries here to count players and entities
    // For now, we'll just mark as healthy
) {
    health_state.set_healthy(true);
    // TODO: Add actual player and entity counting
    // health_state.update_player_count(player_query.iter().count());
    // health_state.update_entity_count(entity_query.iter().count());
}
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use boid_wars_shared::{Position, Velocity as NetworkVelocity};
use std::time::Duration;
use tracing::info;

// Constants
const ROTATION_SYNC_THRESHOLD: f32 = 0.01; // ~0.5 degrees in radians

/// Marker component for entities that need position sync
#[derive(Component)]
pub struct SyncPosition;

/// Plugin that handles synchronization between physics and network positions
///
/// This implements Option 1: Maintaining both Transform (physics) and Position (network)
/// components with robust synchronization and drift detection.
pub struct PositionSyncPlugin;

impl Plugin for PositionSyncPlugin {
    fn build(&self, app: &mut App) {
        // Configuration resources
        app.insert_resource(SyncConfig::default())
            .insert_resource(DriftMetrics::default())
            .insert_resource(SyncPerformanceMetrics::default());

        // System sets for ordering
        app.configure_sets(
            PostUpdate,
            (
                SyncSet::PhysicsToNetwork.after(bevy_rapier2d::plugin::PhysicsSet::Writeback),
                SyncSet::DriftDetection.after(SyncSet::PhysicsToNetwork),
            ),
        );

        // Core sync systems
        app.add_systems(
            PostUpdate,
            (
                initial_position_sync,
                sync_physics_to_network,
                sync_velocity_to_network,
            )
                .chain()
                .in_set(SyncSet::PhysicsToNetwork),
        );

        // Debug and monitoring systems
        #[cfg(debug_assertions)]
        app.add_systems(
            Last,
            (
                detect_position_drift,
                apply_drift_corrections,
                log_sync_performance,
            )
                .chain()
                .in_set(SyncSet::DriftDetection),
        );
    }
}

/// Configuration for position synchronization
#[derive(Resource)]
pub struct SyncConfig {
    /// Maximum allowed drift before warning (in world units)
    pub drift_threshold: f32,
    /// Minimum position change to trigger sync (optimization)
    pub min_sync_distance: f32,
    /// Minimum velocity change to trigger sync
    pub min_sync_velocity: f32,
    /// Enable drift correction (automatically snap positions if drift detected)
    pub auto_correct_drift: bool,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            drift_threshold: 2.5, // Increased from 0.1 to reduce warning spam
            min_sync_distance: 0.1, // Increased from 0.001 - only sync meaningful movement
            min_sync_velocity: 0.1,  // Increased from 0.001 - reduce velocity spam
            auto_correct_drift: true, // Always auto-correct to prevent drift
        }
    }
}

/// Metrics for tracking position drift
#[derive(Resource, Default)]
pub struct DriftMetrics {
    pub max_drift_detected: f32,
    pub entities_with_drift: usize,
    pub total_drift_corrections: usize,
    pub last_drift_check: Option<std::time::Instant>,
}

/// Performance metrics for sync operations
#[derive(Resource, Default)]
pub struct SyncPerformanceMetrics {
    pub positions_synced: usize,
    pub velocities_synced: usize,
    pub sync_time_ms: f32,
    pub last_frame_syncs: usize,
}

/// System sets for ordering
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum SyncSet {
    PhysicsToNetwork,
    NetworkToPhysics,
    DriftDetection,
}

/// Initial sync for newly spawned entities - ensures Position matches Transform
#[allow(clippy::type_complexity)]
pub fn initial_position_sync(
    mut query: Query<(&Transform, &mut Position), (Added<SyncPosition>, With<Transform>)>,
) {
    for (transform, mut position) in query.iter_mut() {
        // Force sync on spawn
        position.0 = transform.translation.truncate();
    }
}

/// Sync physics Transform to network Position and Rotation (server-side)
#[allow(clippy::type_complexity)]
pub fn sync_physics_to_network(
    mut query: Query<
        (
            &Transform,
            &mut Position,
            &mut boid_wars_shared::Rotation,
            Entity,
        ),
        With<SyncPosition>,
    >,
    config: Res<SyncConfig>,
    mut metrics: ResMut<SyncPerformanceMetrics>,
) {
    let start = std::time::Instant::now();
    let mut sync_count = 0;

    for (transform, mut position, mut rotation, _entity) in query.iter_mut() {
        let new_pos = transform.translation.truncate();
        let old_pos = position.0;

        // Extract rotation angle from transform (Z rotation for 2D)
        let new_angle = transform.rotation.to_euler(bevy::math::EulerRot::ZYX).0;
        let old_angle = rotation.angle;

        // Sync position if it changed significantly
        let position_changed = new_pos.distance(old_pos) > config.min_sync_distance;

        // Sync rotation if it changed significantly (using angular distance)
        let angle_diff = (new_angle - old_angle).abs();
        let rotation_changed = angle_diff > ROTATION_SYNC_THRESHOLD;

        if position_changed {
            position.0 = new_pos;
            sync_count += 1;
        }

        if rotation_changed {
            rotation.angle = new_angle;
            sync_count += 1;

        }
    }

    // Update metrics
    metrics.positions_synced += sync_count;
    metrics.last_frame_syncs = sync_count;
    metrics.sync_time_ms = start.elapsed().as_secs_f32() * 1000.0;
}

/// Sync physics Velocity to network Velocity (server-side)
#[allow(clippy::type_complexity)]
pub fn sync_velocity_to_network(
    mut query: Query<(&Velocity, &mut NetworkVelocity), (Changed<Velocity>, With<SyncPosition>)>,
    config: Res<SyncConfig>,
    mut metrics: ResMut<SyncPerformanceMetrics>,
) {
    let mut sync_count = 0;

    for (physics_vel, mut net_vel) in query.iter_mut() {
        let new_vel = physics_vel.linvel;
        let old_vel = net_vel.0;

        // Only sync if velocity changed significantly
        if new_vel.distance(old_vel) > config.min_sync_velocity {
            net_vel.0 = new_vel;
            sync_count += 1;
        }
    }

    metrics.velocities_synced += sync_count;
}

/// Detect position drift between physics and network
#[cfg(debug_assertions)]
pub fn detect_position_drift(
    query: Query<(&Transform, &Position, Entity), With<SyncPosition>>,
    config: Res<SyncConfig>,
    mut metrics: ResMut<DriftMetrics>,
    mut commands: Commands,
) {
    let mut max_drift = 0.0f32;
    let mut drift_count = 0;

    for (transform, position, entity) in query.iter() {
        let physics_pos = transform.translation.truncate();
        let network_pos = position.0;
        let drift = physics_pos.distance(network_pos);

        if drift > config.drift_threshold {
            drift_count += 1;
            max_drift = max_drift.max(drift);

            warn!(
                "Position drift detected on entity {:?}: {:.3} units (physics: {:?}, network: {:?})",
                entity, drift, physics_pos, network_pos
            );

            // Auto-correct if enabled
            if config.auto_correct_drift {
                commands.entity(entity).insert(DriftCorrection {
                    target_position: physics_pos,
                });
                metrics.total_drift_corrections += 1;
            }
        }
    }

    // Update metrics
    metrics.max_drift_detected = max_drift;
    metrics.entities_with_drift = drift_count;
    metrics.last_drift_check = Some(std::time::Instant::now());
}

/// Component marking entities that need drift correction
#[derive(Component)]
pub struct DriftCorrection {
    pub target_position: Vec2,
}

/// Apply drift corrections
pub fn apply_drift_corrections(
    mut commands: Commands,
    mut query: Query<(Entity, &DriftCorrection, &mut Position)>,
) {
    for (entity, correction, mut position) in query.iter_mut() {
        position.0 = correction.target_position;
        commands.entity(entity).remove::<DriftCorrection>();
    }
}

/// Log sync performance metrics
#[cfg(debug_assertions)]
pub fn log_sync_performance(
    metrics: Res<SyncPerformanceMetrics>,
    mut timer: Local<Timer>,
    time: Res<Time>,
) {
    // Log every 5 seconds
    if timer.duration() == Duration::ZERO {
        *timer = Timer::from_seconds(5.0, TimerMode::Repeating);
    }

    if timer.tick(time.delta()).just_finished() && metrics.last_frame_syncs > 0 {
        info!(
            "Position Sync Performance - Positions: {}, Velocities: {}, Time: {:.2}ms, Last Frame: {}",
            metrics.positions_synced,
            metrics.velocities_synced,
            metrics.sync_time_ms,
            metrics.last_frame_syncs
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_config_defaults() {
        let config = SyncConfig::default();
        assert!(config.drift_threshold > 0.0);
        assert!(config.min_sync_distance > 0.0);
    }

    #[test]
    fn test_drift_detection_threshold() {
        let physics_pos = Vec2::new(100.0, 100.0);
        let network_pos = Vec2::new(100.07, 100.07); // Actually under 0.1 threshold (distance ≈ 0.099)
        let drift = physics_pos.distance(network_pos);
        assert!(drift < SyncConfig::default().drift_threshold);
    }
}

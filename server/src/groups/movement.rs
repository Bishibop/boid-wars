use crate::groups::{GroupLOD, LODLevel};
use crate::spatial_grid::SpatialGridSet;
use bevy::prelude::*;
use boid_wars_shared::*;

/// Plugin for group movement systems
pub struct GroupMovementPlugin;

impl Plugin for GroupMovementPlugin {
    fn build(&self, app: &mut App) {
        // Configure system ordering
        app.add_systems(
            FixedUpdate,
            (
                // Group movement just updates group center positions
                group_movement_system.before(SpatialGridSet::Read),
                // Note: All boid movement is now handled by flocking.rs
            ),
        );
    }
}

/// Update group positions based on member positions
fn group_movement_system(
    mut groups: Query<(&mut BoidGroup, &mut Position, &GroupLOD, Entity), Without<Boid>>,
    members: Query<(&BoidGroupMember, &Position), With<Boid>>,
    players: Query<&Player>,
    time: Res<Time>,
) {
    for (mut group, mut pos, lod, group_entity) in groups.iter_mut() {
        // Skip update based on LOD
        if !should_update_group(lod, &time) {
            continue;
        }

        // Calculate group center from member positions
        let mut center = Vec2::ZERO;
        let mut member_count = 0;

        for (member, member_pos) in members.iter() {
            if member.group_entity == group_entity {
                center += member_pos.0;
                member_count += 1;
            }
        }

        if member_count > 0 {
            // Update group position to center of members
            pos.0 = center / member_count as f32;
        }

        // Update behavior state based on combat (keep existing logic for target detection)
        if let GroupBehavior::Engaging { primary_target, .. } = &mut group.behavior_state {
            // Check if target still exists
            if !players.iter().any(|p| p.id as u32 == *primary_target) {
                // Lost target, return to patrol
                group.behavior_state = GroupBehavior::Patrolling {
                    route: group.home_territory.patrol_points.clone(),
                    current_waypoint: 0,
                };
            }
        }
    }
}

/// Check if group should update based on LOD
fn should_update_group(lod: &GroupLOD, time: &Time) -> bool {
    let update_rate = match lod.level {
        LODLevel::Near => 0.016,  // Every frame (60Hz)
        LODLevel::Medium => 0.1,  // 10Hz
        LODLevel::Far => 0.2,     // 5Hz
        LODLevel::Distant => 1.0, // 1Hz
    };

    time.elapsed_secs() - lod.last_update > update_rate
}

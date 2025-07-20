use crate::config::PhysicsConfig;
use crate::physics::GameCollisionGroups;
use crate::position_sync::SyncPosition;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use boid_wars_shared::*;
use lightyear::prelude::server::*;
use lightyear::shared::replication::components::ReplicationGroup;

pub mod combat;
pub mod formation;
pub mod movement;
pub mod territory;

use formation::*;
use territory::*;

/// Configuration for the boid group system
#[derive(Resource, Debug, Clone)]
pub struct BoidGroupConfig {
    // Group parameters
    pub min_group_size: u32,
    pub max_group_size: u32,
    pub default_group_size: u32,
    pub groups_per_zone: u32,

    // Formation parameters
    pub formation_strength: f32,
    pub formation_transition_speed: f32,
    pub formation_position_tolerance: f32,

    // Combat parameters
    pub max_shooters_percentage: f32,
    pub shooter_rotation_interval: f32,
    pub group_aggression_range: f32,

    // Territory parameters
    pub territory_radius: f32,
    pub patrol_speed: f32,
    pub territory_defense_bonus: f32,

    // LOD parameters
    pub lod_near_distance: f32,
    pub lod_medium_distance: f32,
    pub lod_far_distance: f32,

    // Performance limits
    pub max_groups: u32,
    pub max_total_boids: u32,
}

impl Default for BoidGroupConfig {
    fn default() -> Self {
        Self {
            // Group parameters
            min_group_size: 50,
            max_group_size: 200,
            default_group_size: 100,
            groups_per_zone: 5, // More groups for larger scale

            // Formation parameters
            formation_strength: 0.7,
            formation_transition_speed: 2.0,
            formation_position_tolerance: 8.0, // Slightly more than original

            // Combat parameters
            max_shooters_percentage: 0.1, // Only 10% of group can shoot
            shooter_rotation_interval: 5.0, // Rotate every 5 seconds
            group_aggression_range: 300.0, // Scaled for smaller arena

            // Territory parameters
            territory_radius: 200.0, // Scaled for smaller arena
            patrol_speed: 0.6,       // Reasonable speed
            territory_defense_bonus: 1.5,

            // LOD parameters
            lod_near_distance: 500.0,    // Full detail for close groups
            lod_medium_distance: 1000.0, // Reduced update rate
            lod_far_distance: 1500.0,    // Minimal updates for distant groups

            // Performance limits
            max_groups: 50,         // Support large-scale battles
            max_total_boids: 10000, // Target 10k+ entities
        }
    }
}

/// Counter for generating unique group IDs
#[derive(Resource, Default)]
pub struct GroupIdCounter(pub u32);

/// Counter for generating unique boid IDs
#[derive(Resource)]
pub struct BoidIdCounter(pub u32);

impl Default for BoidIdCounter {
    fn default() -> Self {
        Self(1000) // Start at 1000 to avoid conflicts
    }
}

/// Level of detail for group processing
#[derive(Component, Clone, Copy, Debug)]
pub struct GroupLOD {
    pub level: LODLevel,
    pub last_update: f32,
}

#[derive(Clone, Copy, Debug)]
pub enum LODLevel {
    Near,    // Full individual AI, every frame
    Medium,  // Simplified flocking, 10Hz update
    Far,     // Group-only movement, 5Hz update
    Distant, // Static until player approaches
}

/// Spawn a boid group with the specified parameters
pub fn spawn_boid_group(
    commands: &mut Commands,
    archetype: GroupArchetype,
    size: u32,
    territory: TerritoryData,
    group_id_counter: &mut GroupIdCounter,
    boid_id_counter: &mut BoidIdCounter,
    physics_config: &PhysicsConfig,
) -> Entity {
    // Generate unique group ID
    let group_id = group_id_counter.0;
    group_id_counter.0 += 1;

    // Calculate max shooters based on group size (much more conservative)
    let max_shooters = (size as f32 * 0.1).ceil().clamp(1.0, 3.0) as u8; // 10% of group, max 3 shooters

    // Spawn group entity
    let group = commands
        .spawn((
            BoidGroup {
                id: group_id,
                archetype,
                home_territory: territory.clone(),
                current_formation: Formation::default_for_archetype(&archetype),
                behavior_state: GroupBehavior::Patrolling {
                    route: territory.patrol_points.clone(),
                    current_waypoint: 0,
                },
                active_shooters: std::collections::HashSet::new(),
                max_shooters,
                initial_size: size,
            },
            Position(territory.center),
            GroupVelocity(Vec2::ZERO),
            GroupLOD {
                level: LODLevel::Near,
                last_update: 0.0,
            },
            // No replication - groups are server-side only
        ))
        .id();

    // Calculate formation positions
    let formation = Formation::default_for_archetype(&archetype);
    let formation_positions = calculate_formation_positions(&formation, size as usize);

    // Spawn member boids
    for (i, offset) in formation_positions.iter().enumerate() {
        let boid_id = boid_id_counter.0;
        boid_id_counter.0 += 1;

        // Determine role based on position in formation
        let role = match i {
            0 => BoidRole::Leader,
            n if n < 3 => BoidRole::Flanker,
            n if n < size as usize / 2 => BoidRole::Support,
            _ => BoidRole::Scout,
        };

        // Create boid position
        let x = territory.center.x + offset.x;
        let y = territory.center.y + offset.y;

        // Create boid bundle with enhanced stats based on archetype
        let mut bundle = BoidBundle::new(boid_id, x, y);

        // Dramatically adjust combat stats and size based on archetype
        match archetype {
            GroupArchetype::Assault {
                aggression_multiplier,
                ..
            } => {
                // Assault: High damage, fast fire rate, close range, heavy health
                bundle.combat_stats.damage = 8.0 * aggression_multiplier; // Higher base damage
                bundle.combat_stats.fire_rate = 0.4; // Much faster firing (2.5 shots/sec)
                bundle.combat_stats.projectile_speed = 500.0; // Fast projectiles
                bundle.combat_stats.aggression_range = 180.0; // Close engagement range
                bundle.combat_stats.spread_angle = 0.15; // Less accurate (~8.5 degrees)
                bundle.health.max = 40.0; // Tankier for frontline combat
                bundle.health.current = bundle.health.max;
                bundle.size.scale = 1.2; // 20% larger for heavy armor
            }
            GroupArchetype::Defensive { .. } => {
                // Defensive: Moderate damage, slow fire rate, long range, heavy health
                bundle.combat_stats.damage = 6.0; // Moderate damage
                bundle.combat_stats.fire_rate = 0.25; // Slower, methodical firing (4 sec per shot)
                bundle.combat_stats.projectile_speed = 450.0; // Moderate speed
                bundle.combat_stats.aggression_range = 350.0; // Long engagement range
                bundle.combat_stats.spread_angle = 0.05; // Very accurate (~3 degrees)
                bundle.health.max = 50.0; // Heaviest armor for defensive positions
                bundle.health.current = bundle.health.max;
                bundle.size.scale = 1.4; // 40% larger for heaviest armor
            }
            GroupArchetype::Recon { .. } => {
                // Recon: Low damage, moderate fire rate, very long range, light health
                bundle.combat_stats.damage = 4.0; // Lower damage for harassment
                bundle.combat_stats.fire_rate = 0.3; // Moderate firing (~3.3 sec per shot)
                bundle.combat_stats.projectile_speed = 600.0; // Fastest projectiles
                bundle.combat_stats.aggression_range = 400.0; // Longest range for scouting
                bundle.combat_stats.spread_angle = 0.08; // Good accuracy (~4.5 degrees)
                bundle.health.max = 20.0; // Light armor for mobility
                bundle.health.current = bundle.health.max;
                bundle.size.scale = 0.8; // 20% smaller for light armor and speed
            }
        }

        // Random initial velocity
        let angle = rand::random::<f32>() * std::f32::consts::TAU;
        let speed = 50.0;
        bundle.velocity = boid_wars_shared::Velocity::new(angle.cos() * speed, angle.sin() * speed);

        // Store scale value before bundle is moved
        let size_scale = bundle.size.scale;

        commands.spawn((
            bundle,
            BoidGroupMember {
                group_entity: group,
                group_id,
                formation_slot: None, // Disable formation slots
                role_in_group: role,
            },
            BoidSpriteGroup { group_id }, // Add sprite group for client rendering
            Replicate {
                group: ReplicationGroup::new_id(group_id.into()),
                ..default()
            },
            // Physics components
            RigidBody::Dynamic,
            Collider::ball(physics_config.boid_radius * size_scale),
            GameCollisionGroups::boid(),
            ActiveEvents::COLLISION_EVENTS,
            Transform::from_xyz(x, y, 0.0),
            GlobalTransform::default(),
            bevy_rapier2d::dynamics::Velocity {
                linvel: Vec2::new(angle.cos() * speed, angle.sin() * speed),
                angvel: 0.0,
            },
            GravityScale(0.0),
            Damping {
                linear_damping: 0.0,
                angular_damping: 1.0,
            },
            AdditionalMassProperties::Mass(0.5),
            SyncPosition,
        ));
    }

    group
}

/// Plugin for the boid group system
pub struct BoidGroupPlugin;

impl Plugin for BoidGroupPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BoidGroupConfig>();
        app.init_resource::<GroupIdCounter>();
        app.init_resource::<BoidIdCounter>();
        app.init_resource::<crate::flocking::FlockingConfig>(); // Add missing resource for debug UI

        // Add sub-plugins
        app.add_plugins((
            TerritoryPlugin,
            FormationPlugin,
            movement::GroupMovementPlugin,
            combat::GroupCombatPlugin,
        ));

        // Add systems
        app.add_systems(Update, (update_group_lod, cleanup_empty_groups));

        info!("Boid group system initialized");
    }
}

/// Update LOD levels based on player distance
fn update_group_lod(
    mut groups: Query<(&Position, &mut GroupLOD), With<BoidGroup>>,
    players: Query<&Position, With<Player>>,
    config: Res<BoidGroupConfig>,
) {
    for (group_pos, mut lod) in groups.iter_mut() {
        let nearest_player_dist = players
            .iter()
            .map(|p| p.0.distance(group_pos.0))
            .min_by(|a, b| a.total_cmp(b))
            .unwrap_or(f32::MAX);

        lod.level = match nearest_player_dist {
            d if d < config.lod_near_distance => LODLevel::Near,
            d if d < config.lod_medium_distance => LODLevel::Medium,
            d if d < config.lod_far_distance => LODLevel::Far,
            _ => LODLevel::Distant,
        };
    }
}

/// Clean up empty groups
fn cleanup_empty_groups(
    mut commands: Commands,
    groups: Query<Entity, With<BoidGroup>>,
    members: Query<&BoidGroupMember>,
) {
    for group_entity in groups.iter() {
        let has_members = members.iter().any(|m| m.group_entity == group_entity);

        if !has_members {
            commands.entity(group_entity).despawn();
        }
    }
}

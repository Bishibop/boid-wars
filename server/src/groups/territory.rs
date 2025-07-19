use bevy::prelude::*;
use boid_wars_shared::{ArenaZone, TerritoryData, Vec2};
use rand::Rng;

/// Generate territories for the entire arena
pub fn generate_territories(arena_width: f32, arena_height: f32) -> Vec<TerritoryData> {
    let mut territories = Vec::new();
    let mut territory_id = 0;

    // Generate territories for each zone
    for zone in [ArenaZone::Outer, ArenaZone::Middle, ArenaZone::Inner] {
        let zone_territories = match zone {
            ArenaZone::Outer => generate_ring_territories(
                arena_width,
                arena_height,
                0.7,
                0.9,
                zone,
                &mut territory_id,
            ),
            ArenaZone::Middle => generate_ring_territories(
                arena_width,
                arena_height,
                0.4,
                0.7,
                zone,
                &mut territory_id,
            ),
            ArenaZone::Inner => generate_cluster_territories(
                arena_width,
                arena_height,
                0.0,
                0.4,
                zone,
                &mut territory_id,
            ),
            _ => vec![],
        };
        territories.extend(zone_territories);
    }

    // Calculate neighbors
    calculate_territory_neighbors(&mut territories);

    territories
}

/// Generate territories in a ring pattern
fn generate_ring_territories(
    arena_width: f32,
    arena_height: f32,
    inner_radius_ratio: f32,
    outer_radius_ratio: f32,
    zone: ArenaZone,
    territory_id: &mut u32,
) -> Vec<TerritoryData> {
    let mut territories = Vec::new();
    let center = Vec2::new(arena_width / 2.0, arena_height / 2.0);

    // Calculate ring dimensions with safety margin
    let max_radius = (arena_width.min(arena_height) / 2.0) * 0.6; // Much smaller to stay within bounds
    let inner_radius = max_radius * inner_radius_ratio;
    let outer_radius = max_radius * outer_radius_ratio;

    // Number of territories based on circumference
    let avg_radius = (inner_radius + outer_radius) / 2.0;
    let circumference = 2.0 * std::f32::consts::PI * avg_radius;
    let territory_count = (circumference / 300.0).ceil() as i32; // One territory per ~300 units

    let mut rng = rand::thread_rng();

    for i in 0..territory_count {
        // Calculate base angle with some randomness
        let base_angle = (i as f32 / territory_count as f32) * 2.0 * std::f32::consts::PI;
        let angle_offset = rng.gen_range(-0.2..0.2);
        let angle = base_angle + angle_offset;

        // Random radius within the ring
        let radius = rng.gen_range(inner_radius..outer_radius);

        // Calculate territory center
        let territory_center = center + Vec2::new(angle.cos(), angle.sin()) * radius;

        // Ensure territory center is within bounds (with margin for patrol points)
        let margin = 150.0; // Territory radius + patrol point distance
        let clamped_center = Vec2::new(
            territory_center.x.clamp(margin, arena_width - margin),
            territory_center.y.clamp(margin, arena_height - margin),
        );

        // Generate patrol points around the territory
        let patrol_points = generate_patrol_points(clamped_center, 80.0, arena_width, arena_height);

        territories.push(TerritoryData {
            center: clamped_center,
            radius: 120.0, // Smaller radius to stay within bounds
            zone,
            patrol_points,
            neighboring_territories: vec![],
        });

        *territory_id += 1;
    }

    territories
}

/// Generate territories in a cluster pattern for inner zones
fn generate_cluster_territories(
    arena_width: f32,
    arena_height: f32,
    _inner_radius_ratio: f32,
    outer_radius_ratio: f32,
    zone: ArenaZone,
    territory_id: &mut u32,
) -> Vec<TerritoryData> {
    let mut territories = Vec::new();
    let center = Vec2::new(arena_width / 2.0, arena_height / 2.0);

    // Calculate zone dimensions
    let max_radius = (arena_width.min(arena_height) / 2.0) * 0.9;
    let outer_radius = max_radius * outer_radius_ratio;

    let mut rng = rand::thread_rng();

    // Create 3-4 clusters in the inner zone
    let cluster_count = rng.gen_range(3..=4);

    for _ in 0..cluster_count {
        // Random position within the zone
        let angle = rng.gen_range(0.0..2.0 * std::f32::consts::PI);
        let radius = rng.gen_range(100.0..outer_radius);
        let territory_center = center + Vec2::new(angle.cos(), angle.sin()) * radius;

        // Ensure territory center is within bounds
        let margin = 120.0; // Territory radius + patrol point distance
        let clamped_center = Vec2::new(
            territory_center.x.clamp(margin, arena_width - margin),
            territory_center.y.clamp(margin, arena_height - margin),
        );

        // Generate patrol points
        let patrol_points = generate_patrol_points(clamped_center, 60.0, arena_width, arena_height);

        territories.push(TerritoryData {
            center: clamped_center,
            radius: 100.0, // Smaller radius for inner zones
            zone,
            patrol_points,
            neighboring_territories: vec![],
        });

        *territory_id += 1;
    }

    territories
}

/// Generate patrol points for a territory
fn generate_patrol_points(
    center: Vec2,
    radius: f32,
    arena_width: f32,
    arena_height: f32,
) -> Vec<Vec2> {
    let mut points = Vec::new();
    let point_count = 6;
    let margin = 50.0; // Safety margin from arena edges

    for i in 0..point_count {
        let angle = (i as f32 / point_count as f32) * 2.0 * std::f32::consts::PI;
        let offset = Vec2::new(angle.cos(), angle.sin()) * radius * 0.6; // Smaller radius
        let point = center + offset;

        // Clamp patrol points to stay within arena bounds
        let clamped_point = Vec2::new(
            point.x.clamp(margin, arena_width - margin),
            point.y.clamp(margin, arena_height - margin),
        );

        points.push(clamped_point);
    }

    points
}

/// Calculate neighboring territories based on distance
fn calculate_territory_neighbors(territories: &mut [TerritoryData]) {
    let neighbor_distance = 400.0; // Territories within this distance are neighbors

    for i in 0..territories.len() {
        for j in 0..territories.len() {
            if i != j {
                let dist = territories[i].center.distance(territories[j].center);
                if dist < neighbor_distance {
                    territories[i].neighboring_territories.push(j as u32);
                }
            }
        }
    }
}

/// Plugin for territory management
pub struct TerritoryPlugin;

impl Plugin for TerritoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_territory_ownership);
    }
}

/// Update territory ownership based on group positions
fn update_territory_ownership() {
    // This system could track which groups control which territories
    // For now, it's a placeholder for future expansion
}

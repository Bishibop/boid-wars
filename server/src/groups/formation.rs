use bevy::prelude::*;
use boid_wars_shared::{Formation, Vec2};

/// Calculate positions for boids in a formation
pub fn calculate_formation_positions(formation: &Formation, count: usize) -> Vec<Vec2> {
    match formation {
        Formation::VFormation { angle, spacing, .. } => {
            calculate_v_formation(count, *angle, *spacing)
        }
        Formation::CircleDefense { radius, layers, .. } => {
            calculate_circle_formation(count, *radius, *layers)
        }
        Formation::SwarmAttack {
            spread,
            convergence_point,
        } => calculate_swarm_formation(count, *spread, *convergence_point),
        Formation::PatrolLine {
            length,
            wave_amplitude,
        } => calculate_line_formation(count, *length, *wave_amplitude),
    }
}

/// Calculate V formation positions
fn calculate_v_formation(count: usize, angle: f32, spacing: f32) -> Vec<Vec2> {
    let mut positions = Vec::with_capacity(count);

    // Leader at the front
    positions.push(Vec2::ZERO);

    if count == 1 {
        return positions;
    }

    // Calculate wing positions
    let half_angle = angle / 2.0;
    let mut row = 1;
    let mut position_in_row = 0;

    for _ in 1..count {
        let side = if position_in_row % 2 == 0 { -1.0 } else { 1.0 };
        let row_offset = (position_in_row / 2 + 1) as f32;

        let x = side * row_offset * spacing * half_angle.sin();
        let y = -row as f32 * spacing;

        positions.push(Vec2::new(x, y));

        position_in_row += 1;
        if position_in_row >= row * 2 {
            row += 1;
            position_in_row = 0;
        }
    }

    positions
}

/// Calculate circle defense formation
fn calculate_circle_formation(count: usize, base_radius: f32, layers: u8) -> Vec<Vec2> {
    let mut positions = Vec::with_capacity(count);

    if count == 0 {
        return positions;
    }

    // Special case for single boid
    if count == 1 {
        positions.push(Vec2::ZERO);
        return positions;
    }

    // Distribute boids across layers
    let boids_per_layer = count / layers as usize;
    let remainder = count % layers as usize;

    let mut boid_index = 0;

    for layer in 0..layers {
        let layer_radius = base_radius * (1.0 + layer as f32 * 0.5);
        let boids_in_this_layer = if layer < remainder as u8 {
            boids_per_layer + 1
        } else {
            boids_per_layer
        };

        if boids_in_this_layer == 0 {
            continue;
        }

        let angle_step = 2.0 * std::f32::consts::PI / boids_in_this_layer as f32;

        for i in 0..boids_in_this_layer {
            let angle = i as f32 * angle_step;
            let x = angle.cos() * layer_radius;
            let y = angle.sin() * layer_radius;
            positions.push(Vec2::new(x, y));

            boid_index += 1;
            if boid_index >= count {
                return positions;
            }
        }
    }

    positions
}

/// Calculate swarm attack formation
fn calculate_swarm_formation(count: usize, spread: f32, convergence_point: Vec2) -> Vec<Vec2> {
    let mut positions = Vec::with_capacity(count);

    // Create a loose cloud formation that converges toward a point
    let mut rng = rand::thread_rng();
    use rand::Rng;

    for i in 0..count {
        // Spiral pattern with randomness
        let t = i as f32 / count as f32;
        let angle = t * 4.0 * std::f32::consts::PI;
        let radius = spread * (1.0 - t * 0.5); // Tighter as we go inward

        let base_x = angle.cos() * radius;
        let base_y = angle.sin() * radius;

        // Add randomness
        let offset_x = rng.gen_range(-spread * 0.2..spread * 0.2);
        let offset_y = rng.gen_range(-spread * 0.2..spread * 0.2);

        let pos = Vec2::new(base_x + offset_x, base_y + offset_y);

        // Bias toward convergence point
        let biased_pos = pos.lerp(convergence_point * 0.1, 0.3);

        positions.push(biased_pos);
    }

    positions
}

/// Calculate patrol line formation
fn calculate_line_formation(count: usize, length: f32, wave_amplitude: f32) -> Vec<Vec2> {
    let mut positions = Vec::with_capacity(count);

    if count == 0 {
        return positions;
    }

    if count == 1 {
        positions.push(Vec2::ZERO);
        return positions;
    }

    let spacing = length / (count - 1) as f32;
    let half_length = length / 2.0;

    for i in 0..count {
        let x = -half_length + i as f32 * spacing;

        // Add wave pattern
        let wave_phase = (i as f32 / count as f32) * 2.0 * std::f32::consts::PI;
        let y = wave_amplitude * wave_phase.sin();

        positions.push(Vec2::new(x, y));
    }

    positions
}

/// Plugin for formation management
pub struct FormationPlugin;

impl Plugin for FormationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_formation_transitions);
    }
}

/// Handle smooth transitions between formations
fn update_formation_transitions(
    mut groups: Query<(
        &mut boid_wars_shared::BoidGroup,
        &boid_wars_shared::Position,
    )>,
) {
    for (mut group, _) in groups.iter_mut() {
        let should_change_formation = matches!(
            (&group.behavior_state, &group.current_formation),
            (
                boid_wars_shared::GroupBehavior::Engaging { .. },
                Formation::VFormation { .. }
            ) | (
                boid_wars_shared::GroupBehavior::Defending { .. },
                Formation::SwarmAttack { .. }
            ) | (
                boid_wars_shared::GroupBehavior::Patrolling { .. },
                Formation::CircleDefense { .. },
            )
        );

        if should_change_formation {
            group.current_formation = match &group.behavior_state {
                boid_wars_shared::GroupBehavior::Engaging { .. } => Formation::SwarmAttack {
                    spread: 150.0,
                    convergence_point: Vec2::ZERO, // Will be updated to target position
                },
                boid_wars_shared::GroupBehavior::Defending { .. } => Formation::CircleDefense {
                    radius: 100.0,
                    layers: 2,
                    rotation_speed: 0.3,
                },
                boid_wars_shared::GroupBehavior::Patrolling { .. } => {
                    Formation::default_for_archetype(&group.archetype)
                }
                _ => group.current_formation.clone(),
            };
        }
    }
}

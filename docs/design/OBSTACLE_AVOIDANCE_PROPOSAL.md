# Obstacle Avoidance System Proposal

## Executive Summary

This proposal outlines the implementation of an obstacle avoidance system for boids in Boid Wars. The system will enable boids to intelligently navigate around static obstacles, players, and arena walls while maintaining their flocking behavior.

## Problem Statement

Currently, boids only avoid each other through flocking behaviors (separation, alignment, cohesion) and have basic boundary avoidance for arena edges. This results in:
- Boids crashing into static obstacles
- Boids colliding with players when not in combat
- Poor navigation around arena walls
- Unrealistic movement patterns

## Proposed Solution: Hybrid Avoidance System

We recommend a hybrid approach that combines:
1. **Spatial grid detection** for performance
2. **Velocity-based prediction** for natural movement
3. **Force-based steering** for smooth integration with flocking

### Why This Approach?

- **Performance**: Leverages existing spatial grid, avoiding expensive raycasting
- **Behavior Quality**: Predictive avoidance creates natural-looking movement
- **Integration**: Works seamlessly with existing flocking forces
- **Flexibility**: Can be tuned per entity type (obstacles vs players)

## Technical Architecture

### Current System Analysis

#### Movement Pipeline
```
FixedUpdate:
  1. update_spatial_grid → Updates entity positions in grid
  2. update_flocking → Calculates flocking forces, updates Velocity
  3. sync_boid_velocities → Syncs to physics Velocity

Update:
  1. move_boids → Syncs Transform to Position

PostUpdate:
  1. PositionSyncPlugin → Handles Transform ↔ Position sync
```

#### Key Components
- **Boids**: `Position`, `Velocity`, `Transform`, `RigidBody::Dynamic`, `Collider::ball(4.0)`
- **Obstacles**: `Position`, `Obstacle`, `Transform`, `RigidBody::Fixed`, `Collider::cuboid`
- **Players**: `Position`, `Player`, `Transform`, `RigidBody::Dynamic`, `Collider::cuboid`

#### Spatial Grid
Currently tracks ALL entities with `Position` component but only used for boid-to-boid flocking.

## Implementation Plan

### Phase 1: Configuration & Data Structures

#### 1.1 Extend FlockingConfig
```rust
pub struct FlockingConfig {
    // ... existing fields ...
    
    // Obstacle avoidance
    pub obstacle_avoidance_radius: f32,     // Detection radius for obstacles
    pub obstacle_avoidance_weight: f32,     // Force multiplier
    pub obstacle_prediction_time: f32,      // Look-ahead time
    
    // Player avoidance  
    pub player_avoidance_radius: f32,       // Detection radius for players
    pub player_avoidance_weight: f32,       // Force multiplier
}

// Suggested defaults:
obstacle_avoidance_radius: 80.0
obstacle_avoidance_weight: 3.0
obstacle_prediction_time: 0.5
player_avoidance_radius: 100.0
player_avoidance_weight: 2.5
```

#### 1.2 Debug UI Integration
Add sliders to `debug_ui.rs` for real-time tuning of avoidance parameters.

### Phase 2: Avoidance System Implementation

#### 2.1 Modify update_flocking System

Add queries for obstacles and players:
```rust
pub fn update_flocking(
    mut boids: Query<(Entity, &Position, &mut Velocity), With<Boid>>,
    obstacle_query: Query<(&Position, &Obstacle), Without<Boid>>,
    player_query: Query<(&Position, &Velocity), (With<Player>, Without<Boid>)>,
    spatial_grid: Res<SpatialGrid>,
    config: Res<FlockingConfig>,
    time: Res<Time>,
) {
    // ... existing code ...
```

#### 2.2 Avoidance Force Calculation

After calculating flocking forces, add:
```rust
// Calculate avoidance forces
let search_radius = search_radius
    .max(config.obstacle_avoidance_radius)
    .max(config.player_avoidance_radius);

let nearby = spatial_grid.get_nearby_entities(pos.0, search_radius);

let mut obstacle_force = Vec2::ZERO;
let mut player_force = Vec2::ZERO;
let mut obstacle_count = 0;
let mut player_count = 0;

for &other_entity in &nearby {
    // Skip self and other boids (handled by flocking)
    if other_entity == entity || boid_data.iter().any(|(e, _, _)| *e == other_entity) {
        continue;
    }
    
    // Check for obstacles
    if let Ok((obs_pos, obs)) = obstacle_query.get(other_entity) {
        let force = calculate_obstacle_avoidance(
            pos.0, vel.0, obs_pos.0, 
            Vec2::new(obs.width / 2.0, obs.height / 2.0),
            config.obstacle_prediction_time
        );
        obstacle_force += force;
        obstacle_count += 1;
    }
    
    // Check for players
    if let Ok((player_pos, player_vel)) = player_query.get(other_entity) {
        let force = calculate_dynamic_avoidance(
            pos.0, vel.0, player_pos.0, player_vel.0,
            config.player_avoidance_radius
        );
        player_force += force;
        player_count += 1;
    }
}

// Apply averaged forces
if obstacle_count > 0 {
    let avg_force = (obstacle_force / obstacle_count as f32)
        .normalize_or_zero() * config.max_speed;
    let steering = (avg_force - vel.0).clamp_length_max(config.max_force);
    acceleration += steering * config.obstacle_avoidance_weight;
}

if player_count > 0 {
    let avg_force = (player_force / player_count as f32)
        .normalize_or_zero() * config.max_speed;
    let steering = (avg_force - vel.0).clamp_length_max(config.max_force);
    acceleration += steering * config.player_avoidance_weight;
}
```

### Phase 3: Avoidance Algorithms

#### 3.1 Static Obstacle Avoidance
```rust
fn calculate_obstacle_avoidance(
    boid_pos: Vec2,
    boid_vel: Vec2,
    obstacle_pos: Vec2,
    obstacle_half_size: Vec2,
    prediction_time: f32,
) -> Vec2 {
    // Predict where boid will be
    let future_pos = boid_pos + boid_vel * prediction_time;
    
    // Find closest point on obstacle AABB
    let closest = Vec2::new(
        future_pos.x.clamp(
            obstacle_pos.x - obstacle_half_size.x,
            obstacle_pos.x + obstacle_half_size.x
        ),
        future_pos.y.clamp(
            obstacle_pos.y - obstacle_half_size.y,
            obstacle_pos.y + obstacle_half_size.y
        )
    );
    
    // Calculate avoidance force
    let diff = future_pos - closest;
    let distance = diff.length();
    
    if distance < 0.001 {
        // Inside obstacle, push out
        Vec2::new(1.0, 0.0) // Default push direction
    } else if distance < 40.0 { // Danger zone
        // Exponential repulsion
        diff.normalize() * (1.0 - distance / 40.0).powi(2)
    } else {
        Vec2::ZERO
    }
}
```

#### 3.2 Dynamic Entity Avoidance
```rust
fn calculate_dynamic_avoidance(
    boid_pos: Vec2,
    boid_vel: Vec2,
    target_pos: Vec2,
    target_vel: Vec2,
    avoidance_radius: f32,
) -> Vec2 {
    let relative_pos = target_pos - boid_pos;
    let distance = relative_pos.length();
    
    if distance > avoidance_radius || distance < 0.001 {
        return Vec2::ZERO;
    }
    
    // Calculate relative velocity
    let relative_vel = target_vel - boid_vel;
    
    // Time to closest approach
    let time_to_closest = if relative_vel.length_squared() > 0.01 {
        -(relative_pos.dot(relative_vel)) / relative_vel.length_squared()
    } else {
        0.0
    };
    
    // Only avoid if approaching
    if time_to_closest < 0.0 || time_to_closest > 2.0 {
        // Not approaching or too far in future
        return relative_pos.normalize_or_zero() * 
            (1.0 - distance / avoidance_radius).powi(2);
    }
    
    // Predict closest approach distance
    let future_distance = (relative_pos + relative_vel * time_to_closest).length();
    
    if future_distance < 30.0 { // Collision threshold
        // Calculate perpendicular avoidance
        let avoidance_direction = if relative_pos.perp_dot(relative_vel) > 0.0 {
            Vec2::new(-relative_pos.y, relative_pos.x).normalize()
        } else {
            Vec2::new(relative_pos.y, -relative_pos.x).normalize()
        };
        
        avoidance_direction * (1.0 - future_distance / 30.0).powi(2)
    } else {
        Vec2::ZERO
    }
}
```

#### 3.3 Enhanced Wall Avoidance
```rust
fn calculate_wall_avoidance(
    pos: Vec2,
    vel: Vec2,
    width: f32,
    height: f32,
    margin: f32,
) -> Vec2 {
    let mut force = Vec2::ZERO;
    let prediction_dist = vel.length() * 0.5; // Half second look-ahead
    
    // Check each wall
    if pos.x - prediction_dist < margin {
        force.x += (margin - pos.x) / margin;
    } else if pos.x + prediction_dist > width - margin {
        force.x -= (pos.x - (width - margin)) / margin;
    }
    
    if pos.y - prediction_dist < margin {
        force.y += (margin - pos.y) / margin;
    } else if pos.y + prediction_dist > height - margin {
        force.y -= (pos.y - (height - margin)) / margin;
    }
    
    force.normalize_or_zero()
}
```

## Performance Considerations

### Optimizations
1. **Spatial Grid Efficiency**
   - Current grid already provides O(1) neighbor lookups
   - Cell size (100.0) is appropriate for avoidance radii

2. **Query Reduction**
   - Component lookups are cached by Bevy
   - Early distance culling before expensive calculations

3. **Computation Limits**
   - Maximum neighbors to consider: 10 obstacles, 5 players
   - Skip distant entities early

### Expected Performance Impact
- Additional cost per boid: ~0.1-0.2ms
- Total for 30 boids: ~3-6ms
- Acceptable for 60 FPS target (16.67ms budget)

## Testing Strategy

### Test Scenarios
1. **Obstacle Navigation**
   - Create maze-like obstacle course
   - Verify boids navigate without getting stuck

2. **Player Interaction**
   - Idle player in boid path
   - Moving player through boid swarm

3. **Corner Cases**
   - Boids trapped between obstacles
   - High-speed collision courses
   - Dense obstacle fields

### Metrics
- Zero collisions with static obstacles
- Smooth, natural movement patterns
- No oscillation or jitter
- Maintained flocking cohesion

## Migration Plan

1. **Phase 1**: Implement basic obstacle avoidance
2. **Phase 2**: Add player avoidance
3. **Phase 3**: Enhance wall avoidance
4. **Phase 4**: Performance optimization
5. **Phase 5**: Fine-tuning and polish

## Risks & Mitigation

### Risk 1: Performance Degradation
**Mitigation**: Profile extensively, add LOD system if needed

### Risk 2: Unnatural Movement
**Mitigation**: Extensive parameter tuning, consider adding noise

### Risk 3: Deadlocks
**Mitigation**: Add deadlock detection and random escape forces

## Future Enhancements

1. **Raycasting Integration**
   - Use Rapier2D's `QueryPipeline` for precise detection
   - Better for complex obstacle shapes

2. **Flow Fields**
   - Pre-calculate navigation fields
   - Excellent for many boids, static obstacles

3. **Behavior Trees**
   - More sophisticated decision making
   - Priority-based avoidance

## Conclusion

This hybrid avoidance system will significantly improve boid navigation while maintaining performance and visual quality. The implementation is straightforward, leveraging existing systems and following established patterns in the codebase.

The modular design allows incremental implementation and testing, reducing risk and enabling quick iteration based on playtesting feedback.
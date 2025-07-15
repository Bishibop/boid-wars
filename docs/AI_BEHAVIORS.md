# Boid AI Behaviors

This document outlines the AI behavior system for the 10,000+ enemy boids in Boid Wars.

## Overview

Boids are the primary antagonists in Boid Wars - swarms of AI-controlled enemies that attack players using emergent flocking behaviors combined with combat AI. The system must handle 10,000+ active boids while maintaining 60 Hz server performance.

## Core Behaviors

### 1. Idle
Default state when no threats are detected.
- **Movement**: Gentle wandering within patrol area
- **Transitions**: → Hunting when player in detection range
- **Performance**: Minimal CPU usage

### 2. Hunting
Active pursuit of the nearest player.
- **Movement**: Direct path to target with obstacle avoidance
- **Attack**: Fire projectiles when in range
- **Group Behavior**: Coordinate with nearby boids to surround target
- **Transitions**: → Fleeing when health low, → Swarming when many boids nearby

### 3. Swarming
Coordinated group movement using classic boid flocking rules.
- **Separation**: Avoid crowding neighbors (highest priority)
- **Alignment**: Steer toward average heading of neighbors
- **Cohesion**: Steer toward average position of neighbors
- **Target Seeking**: Bias swarm movement toward nearest player
- **Transitions**: → Hunting when isolated, → Fleeing when swarm takes heavy damage

### 4. Fleeing
Escape behavior when damaged or outnumbered.
- **Movement**: Away from threat, toward friendly boids
- **Speed**: 150% normal speed for limited duration
- **Transitions**: → Swarming when safe with allies, → Idle when threat gone

### 5. Patrolling
Structured movement along predefined paths.
- **Movement**: Follow waypoint sequence
- **Detection**: Increased sensor range while patrolling
- **Transitions**: → Hunting when player detected

## Implementation Strategy

### State Machine Architecture

```rust
#[derive(Component)]
struct BoidAI {
    current_state: BoidState,
    state_timer: f32,
    target: Option<Entity>,
    home_position: Vec2,
}

#[derive(Debug, Clone)]
enum BoidState {
    Idle { wander_target: Vec2 },
    Hunting { target: Entity, last_seen: Vec2 },
    Swarming { swarm_id: u32 },
    Fleeing { threat_position: Vec2 },
    Patrolling { waypoints: Vec<Vec2>, current_index: usize },
}
```

### Spatial Optimization

To handle 10k+ boids efficiently:

1. **Hierarchical Spatial Grid**
   - Coarse grid (500x500 units) for broad phase
   - Fine grid (50x50 units) for neighbor queries
   - Update only when boid crosses cell boundary

2. **Level of Detail (LOD)**
   - **Near** (< 500 units): Full AI, 30 Hz updates
   - **Medium** (500-1500 units): Simplified AI, 15 Hz updates  
   - **Far** (> 1500 units): Basic movement only, 5 Hz updates

3. **Batch Processing**
   - Group boids by behavior state
   - Process similar behaviors together for cache efficiency
   - Use SIMD for vector calculations

### Flocking Optimization

```rust
// Spatial hash for efficient neighbor queries
const NEIGHBOR_RADIUS: f32 = 100.0;
const MAX_NEIGHBORS: usize = 7; // Limit for performance

fn calculate_flocking_forces(
    boid_pos: Vec2,
    neighbors: &[BoidNeighbor],
) -> FlockingForces {
    let mut separation = Vec2::ZERO;
    let mut alignment = Vec2::ZERO;
    let mut cohesion = Vec2::ZERO;
    let mut count = 0;
    
    // Only consider nearest N neighbors
    for neighbor in neighbors.iter().take(MAX_NEIGHBORS) {
        let diff = boid_pos - neighbor.position;
        let dist_sq = diff.length_squared();
        
        // Separation (inverse square falloff)
        if dist_sq > 0.0 && dist_sq < SEPARATION_RADIUS_SQ {
            separation += diff / dist_sq;
        }
        
        // Alignment and cohesion
        alignment += neighbor.velocity;
        cohesion += neighbor.position;
        count += 1;
    }
    
    // Normalize forces
    if count > 0 {
        alignment /= count as f32;
        cohesion = (cohesion / count as f32) - boid_pos;
    }
    
    FlockingForces {
        separation: separation.normalize_or_zero() * SEPARATION_WEIGHT,
        alignment: alignment.normalize_or_zero() * ALIGNMENT_WEIGHT,
        cohesion: cohesion.normalize_or_zero() * COHESION_WEIGHT,
    }
}
```

## Emergent Behaviors

The combination of simple rules creates complex emergent behaviors:

### Attack Patterns
- **Pincer Movement**: Hunting boids naturally surround players
- **Wave Attacks**: Swarms create pulsing attack patterns
- **Hit and Run**: Damaged boids flee and regroup

### Defensive Formations
- **Scatter**: When area damage detected, swarm disperses
- **Shield Wall**: Healthy boids protect fleeing allies
- **Bait Ball**: Dense defensive formation when heavily outnumbered

## Performance Profiling

Key metrics to monitor:

1. **AI Update Time**: Target < 5ms for 10k boids
2. **Spatial Query Time**: Target < 1ms per frame
3. **Memory Usage**: ~100 bytes per boid (1MB for 10k)
4. **Cache Misses**: Minimize by processing spatially coherent groups

## Testing Strategies

### Unit Tests
```rust
#[test]
fn test_flocking_forces() {
    // Test separation dominates at close range
    // Test alignment at medium range
    // Test cohesion at far range
}

#[test]
fn test_state_transitions() {
    // Verify all valid state transitions
    // Test transition conditions
}
```

### Integration Tests
- Spawn 10k boids and measure frame time
- Test player detection range accuracy
- Verify swarm cohesion under stress

### Behavior Validation
- Record and replay boid behaviors
- Visual debugging overlays
- A/B test different parameter sets

## Tuning Parameters

```rust
// Exposed for runtime tuning
pub struct BoidConfig {
    // Detection
    pub idle_detection_range: f32,      // 300.0
    pub patrol_detection_range: f32,    // 500.0
    
    // Movement
    pub base_speed: f32,                // 100.0
    pub flee_speed_multiplier: f32,     // 1.5
    pub max_acceleration: f32,          // 200.0
    
    // Flocking
    pub separation_weight: f32,         // 2.0
    pub alignment_weight: f32,          // 1.0
    pub cohesion_weight: f32,          // 1.0
    pub separation_radius: f32,         // 30.0
    
    // Combat
    pub attack_range: f32,              // 200.0
    pub projectile_speed: f32,          // 300.0
    pub fire_rate: f32,                 // 0.5
}
```

## Future Enhancements

1. **Adaptive Difficulty**
   - Boids learn from player behavior
   - Difficulty scales with player skill

2. **Special Boid Types**
   - Leaders that buff nearby boids
   - Kamikaze boids with area damage
   - Shield boids that protect others

3. **Environmental Awareness**
   - Use obstacles for ambushes
   - Retreat to defensive positions
   - Coordinate through chokepoints
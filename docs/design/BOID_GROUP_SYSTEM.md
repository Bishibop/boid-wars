# Boid Group System Implementation Plan

## Executive Summary

This document outlines the implementation of a group-based boid system for Boid Wars, enabling 50+ boids to operate as coordinated groups with distinct behaviors, formations, and combat tactics. The system is designed to maintain 60 FPS performance with thousands of entities while creating engaging gameplay through emergent group behaviors.

## Current State Analysis

### Existing Implementation
- **Individual Spawning**: 30 boids spawn individually across the arena
- **Independent Behavior**: Each boid operates autonomously with basic flocking
- **Simple Combat**: Range-based aggression with individual targeting
- **Full Replication**: Every boid is a separate networked entity

### Key Strengths
- Robust flocking algorithm with separation, alignment, and cohesion
- Efficient spatial grid for neighbor queries (100x100 cells)
- Combat system with aggression tracking and memory
- Obstacle avoidance system with predictive steering
- Projectile pooling for performance

### Limitations for Large Scale
- No concept of groups or coordinated behavior
- Each boid requires full network replication (expensive)
- Individual AI calculations for every boid
- No formation or pattern-based movement

## Proposed Architecture

### Core Design Principles
1. **Hierarchical Control**: Groups make high-level decisions, individuals execute
2. **Formation-Based Movement**: Boids maintain formations while flocking locally
3. **Network Efficiency**: Replicate groups, not individuals
4. **Performance First**: LOD systems and batch processing
5. **Emergent Complexity**: Simple rules create sophisticated behaviors

### Group System Components

```rust
// Core group component
#[derive(Component, Serialize, Deserialize, Clone, Debug)]
pub struct BoidGroup {
    pub id: u32,
    pub archetype: GroupArchetype,
    pub home_territory: TerritoryData,
    pub current_formation: Formation,
    pub behavior_state: GroupBehavior,
    pub active_shooters: HashSet<Entity>,
    pub max_shooters: u8,
}

// Boid membership
#[derive(Component, Clone, Debug)]
pub struct BoidGroupMember {
    pub group_entity: Entity,
    pub group_id: u32,
    pub formation_slot: Option<FormationSlot>,
    pub role_in_group: BoidRole,
}

// Group archetypes with distinct behaviors
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum GroupArchetype {
    Assault {
        aggression_multiplier: f32,
        preferred_range: f32,
    },
    Defensive {
        protection_radius: f32,
        retreat_threshold: f32,
    },
    Recon {
        detection_range: f32,
        flee_speed_bonus: f32,
    },
}

// Dynamic formations
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Formation {
    VFormation { 
        angle: f32, 
        spacing: f32,
        leader_boost: f32,
    },
    CircleDefense { 
        radius: f32,
        layers: u8,
        rotation_speed: f32,
    },
    SwarmAttack { 
        spread: f32,
        convergence_point: Vec2,
    },
    PatrolLine {
        length: f32,
        wave_amplitude: f32,
    },
}

// Group AI states
#[derive(Clone, Debug)]
pub enum GroupBehavior {
    Patrolling { 
        route: Vec<Vec2>, 
        current_waypoint: usize 
    },
    Engaging { 
        primary_target: Entity,
        secondary_targets: Vec<Entity>,
    },
    Retreating { 
        rally_point: Vec2,
        speed_multiplier: f32,
    },
    Defending { 
        position: Vec2,
        radius: f32,
    },
}
```

### Territory System

```rust
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct TerritoryData {
    pub center: Vec2,
    pub radius: f32,
    pub zone: ArenaZone,
    pub patrol_points: Vec<Vec2>,
    pub neighboring_territories: Vec<u32>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum ArenaZone {
    Outer,  // Recon groups
    Middle, // Defensive groups  
    Inner,  // Assault groups
    Center, // Boss groups (future)
}
```

## Implementation Plan

### Phase 1: Foundation (Week 1)

#### 1.1 Data Structures
- Add group components to `shared/src/protocol.rs`
- Create `server/src/groups/mod.rs` for group systems
- Extend `BoidBundle` with group membership

#### 1.2 Group Spawning
```rust
fn spawn_boid_group(
    commands: &mut Commands,
    archetype: GroupArchetype,
    size: u32,
    territory: TerritoryData,
) -> Entity {
    // Spawn group entity
    let group = commands.spawn((
        BoidGroup {
            id: generate_group_id(),
            archetype,
            home_territory: territory.clone(),
            current_formation: Formation::default_for_archetype(&archetype),
            behavior_state: GroupBehavior::Patrolling {
                route: territory.patrol_points.clone(),
                current_waypoint: 0,
            },
            active_shooters: HashSet::new(),
            max_shooters: calculate_max_shooters(size),
        },
        Position(territory.center),
        GroupReplicate::default(),
    )).id();

    // Spawn member boids
    let formation_positions = calculate_formation_positions(&formation, size);
    for (i, offset) in formation_positions.iter().enumerate() {
        let boid_type = select_boid_type_for_archetype(&archetype, i);
        
        commands.spawn((
            BoidBundle::new(
                generate_boid_id(),
                territory.center.x + offset.x,
                territory.center.y + offset.y,
            ),
            BoidGroupMember {
                group_entity: group,
                group_id: group.id,
                formation_slot: Some(FormationSlot(i)),
                role_in_group: boid_type.into(),
            },
            boid_type,
        ));
    }
    
    group
}
```

#### 1.3 Territory Generation
```rust
fn generate_territories(arena_width: f32, arena_height: f32) -> Vec<TerritoryData> {
    let mut territories = Vec::new();
    
    // Zone-based generation with organic placement
    for zone in [ArenaZone::Outer, ArenaZone::Middle, ArenaZone::Inner] {
        let zone_territories = match zone {
            ArenaZone::Outer => generate_ring_territories(arena_width, arena_height, 0.7, 0.9),
            ArenaZone::Middle => generate_ring_territories(arena_width, arena_height, 0.4, 0.7),
            ArenaZone::Inner => generate_cluster_territories(arena_width, arena_height, 0.0, 0.4),
            _ => vec![],
        };
        territories.extend(zone_territories);
    }
    
    // Find neighbors for overlap
    calculate_territory_neighbors(&mut territories);
    
    territories
}
```

### Phase 2: Group Movement (Week 1-2)

#### 2.1 Hierarchical Flocking
```rust
// Group-level movement decisions
fn group_movement_system(
    mut groups: Query<(&mut BoidGroup, &Position, &mut GroupVelocity)>,
    players: Query<&Position, With<Player>>,
    spatial_grid: Res<SpatialGrid>,
    time: Res<Time>,
) {
    for (mut group, pos, mut velocity) in groups.iter_mut() {
        // High-level movement decision based on behavior state
        let target_velocity = match &group.behavior_state {
            GroupBehavior::Patrolling { route, current_waypoint } => {
                calculate_patrol_velocity(pos, &route[*current_waypoint])
            },
            GroupBehavior::Engaging { primary_target, .. } => {
                calculate_attack_velocity(pos, primary_target)
            },
            GroupBehavior::Retreating { rally_point, speed_multiplier } => {
                calculate_retreat_velocity(pos, rally_point) * *speed_multiplier
            },
            GroupBehavior::Defending { position, .. } => {
                calculate_defend_velocity(pos, position)
            },
        };
        
        // Smooth velocity changes
        velocity.0 = velocity.0.lerp(target_velocity, time.delta_seconds() * 2.0);
    }
}

// Individual boid movement within group constraints
fn formation_flocking_system(
    mut boids: Query<(&BoidGroupMember, &Position, &mut Velocity), With<Boid>>,
    groups: Query<(&BoidGroup, &Position, &GroupVelocity)>,
    spatial_grid: Res<SpatialGrid>,
    config: Res<FlockingConfig>,
) {
    // Batch process by group for cache efficiency
    let mut groups_map: HashMap<Entity, Vec<(Entity, &Position, &mut Velocity)>> = HashMap::new();
    
    for (member, pos, vel) in boids.iter_mut() {
        groups_map.entry(member.group_entity)
            .or_insert_with(Vec::new)
            .push((entity, pos, vel));
    }
    
    for (group_entity, members) in groups_map {
        let (group, group_pos, group_vel) = groups.get(group_entity).unwrap();
        
        // Apply formation constraints
        let formation_positions = calculate_formation_positions(&group.current_formation, members.len());
        
        for (i, (entity, pos, mut vel)) in members.iter_mut().enumerate() {
            // Blend formation position with local flocking
            let formation_target = *group_pos + formation_positions[i];
            let formation_force = (formation_target - pos.0).normalize_or_zero() * config.formation_strength;
            
            // Regular flocking within local neighborhood
            let flocking_force = calculate_local_flocking(entity, pos, &spatial_grid, &config);
            
            // Combine forces
            vel.0 = (group_vel.0 + formation_force + flocking_force * 0.3)
                .clamp_length_max(config.max_speed);
        }
    }
}
```

#### 2.2 Formation Transitions
```rust
fn formation_transition_system(
    mut groups: Query<(&mut BoidGroup, &Position)>,
    players: Query<&Position, With<Player>>,
    time: Res<Time>,
) {
    for (mut group, pos) in groups.iter_mut() {
        let should_change_formation = match (&group.behavior_state, &group.current_formation) {
            (GroupBehavior::Engaging { .. }, Formation::VFormation { .. }) => true,
            (GroupBehavior::Defending { .. }, Formation::SwarmAttack { .. }) => true,
            (GroupBehavior::Patrolling { .. }, Formation::CircleDefense { .. }) => true,
            _ => false,
        };
        
        if should_change_formation {
            group.current_formation = select_formation_for_behavior(&group.behavior_state);
            // Smooth transition will be handled by formation_flocking_system
        }
    }
}
```

### Phase 3: Combat Integration (Week 2)

#### 3.1 Group Combat Coordinator
```rust
fn group_combat_system(
    mut groups: Query<(&mut BoidGroup, &Position)>,
    mut boids: Query<(&BoidGroupMember, &mut BoidCombat, &Position), With<Boid>>,
    players: Query<(Entity, &Position), With<Player>>,
    aggression: Res<BoidAggression>,
) {
    for (mut group, group_pos) in groups.iter_mut() {
        // Determine group target
        let group_target = match &group.behavior_state {
            GroupBehavior::Engaging { primary_target, .. } => Some(*primary_target),
            _ => find_nearest_threat(&group, group_pos, &players, &aggression),
        };
        
        if let Some(target) = group_target {
            // Select active shooters
            update_active_shooters(&mut group, &boids);
            
            // Update combat state for active shooters
            for (member, mut combat, pos) in boids.iter_mut() {
                if member.group_entity == group_entity && group.active_shooters.contains(&entity) {
                    combat.current_target = Some(target);
                    combat.should_shoot = true;
                } else {
                    combat.should_shoot = false;
                }
            }
        }
    }
}

fn update_active_shooters(
    group: &mut BoidGroup,
    boids: &Query<(&BoidGroupMember, &Position), With<Boid>>,
) {
    // Rotate active shooters every few seconds
    if group.active_shooters.len() < group.max_shooters as usize {
        // Select new shooters based on position, type, health
        let eligible = boids.iter()
            .filter(|(m, _)| m.group_entity == group.entity())
            .filter(|(_, _)| /* has ammo, good angle, etc */)
            .take(group.max_shooters as usize - group.active_shooters.len());
            
        for (entity, _) in eligible {
            group.active_shooters.insert(entity);
        }
    }
}
```

### Phase 4: Performance Optimization (Week 3)

#### 4.1 Level of Detail System
```rust
#[derive(Component)]
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

fn lod_update_system(
    mut groups: Query<(&Position, &mut GroupLOD), With<BoidGroup>>,
    players: Query<&Position, With<Player>>,
    config: Res<LODConfig>,
) {
    for (group_pos, mut lod) in groups.iter_mut() {
        let nearest_player_dist = players.iter()
            .map(|p| p.0.distance(group_pos.0))
            .min()
            .unwrap_or(f32::MAX);
            
        lod.level = match nearest_player_dist {
            d if d < config.near_distance => LODLevel::Near,
            d if d < config.medium_distance => LODLevel::Medium,
            d if d < config.far_distance => LODLevel::Far,
            _ => LODLevel::Distant,
        };
    }
}

// Use LOD to control update frequency
fn should_update_group(lod: &GroupLOD, time: &Time) -> bool {
    let update_rate = match lod.level {
        LODLevel::Near => 0.016,    // Every frame (60Hz)
        LODLevel::Medium => 0.1,     // 10Hz
        LODLevel::Far => 0.2,        // 5Hz
        LODLevel::Distant => 1.0,    // 1Hz
    };
    
    time.elapsed_seconds() - lod.last_update > update_rate
}
```

#### 4.2 Batch Processing
```rust
// Process groups in batches to improve cache locality
fn batched_group_update_system(
    groups: Query<(Entity, &BoidGroup, &GroupLOD)>,
    mut boids: Query<(&BoidGroupMember, &mut Position, &mut Velocity)>,
    time: Res<Time>,
) {
    // Group boids by their group entity for batch processing
    let mut group_members: HashMap<Entity, Vec<(Entity, Mut<Position>, Mut<Velocity>)>> = HashMap::new();
    
    for (entity, member, pos, vel) in boids.iter_mut() {
        group_members.entry(member.group_entity)
            .or_insert_with(Vec::new)
            .push((entity, pos, vel));
    }
    
    // Process each group's members together
    for (group_entity, group, lod) in groups.iter() {
        if !should_update_group(lod, &time) {
            continue;
        }
        
        if let Some(members) = group_members.get_mut(&group_entity) {
            // All memory accesses for this group are sequential
            update_group_members_batch(group, members, &time);
        }
    }
}
```

### Phase 5: Network Optimization (Week 3-4)

#### 5.1 Group-Based Replication
```rust
// Only replicate group state, not individual boids
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct ReplicatedGroup {
    pub id: u32,
    pub position: Vec2,
    pub formation: Formation,
    pub member_count: u32,
    pub archetype: GroupArchetype,
}

// Client-side group representation
#[derive(Component)]
pub struct ClientBoidGroup {
    pub server_group_id: u32,
    pub interpolated_position: Vec2,
    pub target_formation: Formation,
    pub current_formation_blend: f32,
}

// Client spawns and manages individual boids locally
fn client_group_spawn_system(
    mut commands: Commands,
    new_groups: Query<(Entity, &ReplicatedGroup), Added<ReplicatedGroup>>,
) {
    for (entity, replicated) in new_groups.iter() {
        // Spawn visual representations of all boids
        for i in 0..replicated.member_count {
            commands.spawn((
                ClientBoid {
                    group: entity,
                    index: i,
                },
                Position::default(),
                // Visual components...
            ));
        }
    }
}
```

#### 5.2 Client Interpolation
```rust
fn client_boid_interpolation_system(
    groups: Query<(&ClientBoidGroup, &ReplicatedGroup)>,
    mut boids: Query<(&ClientBoid, &mut Position)>,
    time: Res<Time>,
) {
    for (client_group, replicated) in groups.iter() {
        // Interpolate group position
        let group_pos = client_group.interpolated_position
            .lerp(replicated.position, time.delta_seconds() * 5.0);
            
        // Calculate formation positions
        let formation_positions = calculate_formation_positions(
            &replicated.formation,
            replicated.member_count,
        );
        
        // Update individual boid positions
        for (boid, mut pos) in boids.iter_mut() {
            if boid.group == entity {
                let target = group_pos + formation_positions[boid.index];
                pos.0 = pos.0.lerp(target, time.delta_seconds() * 10.0);
            }
        }
    }
}
```

## Testing Strategy

### Performance Testing
```rust
#[cfg(test)]
mod perf_tests {
    #[test]
    fn test_group_spawn_performance() {
        let mut app = create_test_app();
        
        // Spawn 10 groups of 50 boids each
        let start = Instant::now();
        for _ in 0..10 {
            spawn_boid_group(&mut app.world, GroupArchetype::Assault, 50);
        }
        let spawn_time = start.elapsed();
        
        assert!(spawn_time.as_millis() < 50, "Group spawning too slow");
        
        // Run systems and measure frame time
        let frame_start = Instant::now();
        app.update();
        let frame_time = frame_start.elapsed();
        
        assert!(frame_time.as_millis() < 16, "Frame time exceeds 60 FPS target");
    }
}
```

### Behavior Testing
- Formation maintenance under movement
- Combat coordination within groups
- Territory patrol patterns
- Player interaction responses

## Configuration

```rust
pub struct BoidGroupConfig {
    // Group parameters
    pub min_group_size: u32,                    // 30
    pub max_group_size: u32,                    // 100
    pub default_group_size: u32,                // 50
    pub groups_per_zone: u32,                   // 2-3
    
    // Formation parameters
    pub formation_strength: f32,                // 0.7
    pub formation_transition_speed: f32,        // 2.0
    pub formation_position_tolerance: f32,      // 5.0
    
    // Combat parameters
    pub max_shooters_percentage: f32,          // 0.2 (20% of group)
    pub shooter_rotation_interval: f32,         // 3.0 seconds
    pub group_aggression_range: f32,           // 400.0
    
    // Territory parameters
    pub territory_radius: f32,                  // 300.0
    pub patrol_speed: f32,                     // 0.5
    pub territory_defense_bonus: f32,          // 1.5
    
    // LOD parameters
    pub lod_near_distance: f32,                // 500.0
    pub lod_medium_distance: f32,              // 1000.0
    pub lod_far_distance: f32,                 // 1500.0
    
    // Performance limits
    pub max_groups: u32,                       // 10
    pub max_total_boids: u32,                  // 1000
}
```

## Migration Strategy

1. **Phase 1**: Implement alongside existing system
   - Add group spawning as alternative to individual spawning
   - Test with 1-2 groups mixed with individual boids
   
2. **Phase 2**: Performance validation
   - Profile with increasing group counts
   - Optimize bottlenecks
   
3. **Phase 3**: Full deployment
   - Replace individual spawning with group spawning
   - Remove old individual-only systems

## Risk Mitigation

### Technical Risks
1. **Client Prediction Errors**
   - Solution: Extensive interpolation testing
   - Fallback: Show server positions with lag
   
2. **Formation Deadlocks**
   - Solution: Timeout and reform
   - Prevention: Flexible formation constraints

3. **Performance Degradation**
   - Solution: Aggressive LOD, batch processing
   - Monitoring: Built-in performance metrics

### Gameplay Risks
1. **Overwhelming Difficulty**
   - Solution: Careful combat limiting
   - Tuning: Extensive playtesting
   
2. **Predictable Behavior**
   - Solution: Multiple behavior states
   - Variety: Random elements within patterns

## Success Metrics

1. **Performance**
   - Maintain 60 FPS with 500+ boids
   - Network bandwidth < 10KB/s per client
   - Frame time < 16ms (99th percentile)

2. **Gameplay**
   - Player engagement time increased
   - Combat encounters feel threatening but fair
   - Visible coordination in boid groups

3. **Technical**
   - Clean architecture enabling future features
   - Reduced code complexity vs individual management
   - Easier balancing through group parameters

## Future Enhancements

1. **Advanced Formations**
   - Dynamic formation generation
   - Multi-layered formations
   - Terrain-adaptive shapes

2. **Inter-Group Coordination**
   - Temporary alliances
   - Coordinated territory defense
   - Group merging/splitting

3. **Specialized Behaviors**
   - Ambush tactics
   - Flanking maneuvers
   - Retreat and regroup

4. **Visual Polish**
   - Formation trails
   - Group identity markers
   - Coordinated animations

## Conclusion

This group-based boid system transforms the current individual-focused implementation into a scalable, performant architecture capable of supporting thousands of boids while creating engaging, emergent gameplay. The hierarchical approach maintains the appealing flocking behaviors while adding strategic depth through coordinated group actions.

The phased implementation plan allows for iterative development and testing, ensuring each component is solid before building the next layer. With careful attention to performance optimization and network efficiency, this system will enable the ambitious vision of massive space battles with intelligent, coordinated enemies.
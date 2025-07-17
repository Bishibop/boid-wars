# Player and Shooting Physics Implementation Proposal

**STATUS: COMPLETED - Archived 2025-01-17**

This proposal has been fully implemented in the `feature/player-physics` branch and merged. See CHANGELOG.md for implementation details.

## Overview

This document outlines the technical implementation plan for player movement and shooting physics in Boid Wars. The system is designed to support fast-paced arcade-style dogfighting with twin-stick controls while maintaining compatibility with future boid enemy integration.

## Architecture Integration

### Alignment with Existing Systems
- **Server**: Rust/Bevy ECS 0.16 for authoritative physics simulation
- **Physics**: Rapier 2D for unified collision detection and physics
- **Networking**: Lightyear for player input and state replication  
- **Spatial**: R-tree integration for efficient spatial queries
- **Performance**: 60Hz physics simulation with 30Hz network sync

## Design Decisions

### Core Physics Requirements
Based on gameplay requirements analysis:

1. **Movement System**
   - **Momentum/Inertia**: Ships coast when not thrusting for space feel
   - **Independent Rotation**: Ship facing direction separate from movement
   - **Forward Speed Boost**: Faster movement when thrusting in facing direction
   - **Fast Arcade Style**: Responsive controls with quick acceleration

2. **Projectile System**
   - **Constant Velocity**: Bullets travel in straight lines
   - **Travel Time**: Projectiles are not instant hit
   - **Time-Based Lifetime**: Easier for weapon variety implementation
   - **Performance Scale**: Hundreds of simultaneous projectiles

3. **Collision System**
   - **Rectangular Ship Colliders**: More realistic than circles
   - **Players Pass Through**: No player-player collision
   - **Damage Only**: No knockback or physics response on hits
   - **Unified Detection**: Rapier2D for all collision types

4. **World Structure**
   - **Fixed Arena**: Walls with collision response
   - **Configurable Size**: ~2500x1500 default (larger than laptop screen)
   - **Performance Priority**: Optimized for scale over simulation accuracy

### Boid Integration Compatibility

The physics system is designed to seamlessly integrate with future boid enemies:

- **Unified Collision**: Same Rapier2D system for players and boids
- **Interest Management**: Dynamic physics component addition/removal
- **Scalable Architecture**: Handles transition from hundreds to thousands of entities
- **Collision Groups**: Extensible grouping system for different entity types

## Server-Side Implementation (Rust/Bevy ECS)

### Core Components

#### Player Entity Components
```rust
#[derive(Component)]
pub struct Player {
    pub player_id: u64,
    pub health: f32,
    pub max_health: f32,
    pub thrust_force: f32,
    pub turn_rate: f32,
    pub forward_speed_multiplier: f32,
    pub weapon_cooldown: Timer,
}

#[derive(Component)]
pub struct PlayerInput {
    pub movement: Vec2,        // Normalized movement vector
    pub aim_direction: Vec2,   // Normalized aim direction
    pub thrust: f32,           // 0.0 to 1.0
    pub shooting: bool,
    pub input_sequence: u32,   // For network synchronization
}

#[derive(Component)]
pub struct Ship {
    pub facing_direction: Vec2,
    pub max_speed: f32,
    pub acceleration: f32,
    pub deceleration: f32,
    pub angular_velocity: f32,
}
```

#### Projectile Components
```rust
#[derive(Component)]
pub struct Projectile {
    pub damage: f32,
    pub owner: Entity,
    pub projectile_type: ProjectileType,
    pub lifetime: Timer,
    pub speed: f32,
}

#[derive(Component)]
pub struct ProjectilePhysics {
    pub velocity: Vec2,
    pub spawn_time: Instant,
    pub max_lifetime: Duration,
}

#[derive(Component)]
pub struct WeaponStats {
    pub damage: f32,
    pub fire_rate: f32,
    pub projectile_speed: f32,
    pub projectile_lifetime: Duration,
    pub spread: f32,
}
```

### Physics System Implementation

#### Rapier2D Integration
```rust
use bevy_rapier2d::prelude::*;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
            .add_plugins(RapierDebugRenderPlugin::default())
            .insert_resource(RapierConfiguration {
                gravity: Vec2::ZERO, // Space environment
                timestep_mode: TimestepMode::Fixed {
                    dt: 1.0 / 60.0, // 60Hz physics
                    substeps: 1,
                },
                ..default()
            })
            .add_systems(FixedUpdate, (
                player_input_system,
                player_movement_system,
                projectile_system,
                collision_system,
                cleanup_system,
            ).chain());
    }
}
```

#### Collision Groups Configuration
```rust
pub struct CollisionGroups {
    pub players: Group,
    pub projectiles: Group,
    pub walls: Group,
    pub boids: Group, // Future use
}

impl Default for CollisionGroups {
    fn default() -> Self {
        Self {
            players: Group::GROUP_1,
            projectiles: Group::GROUP_2,
            walls: Group::GROUP_3,
            boids: Group::GROUP_4,
        }
    }
}

// Collision matrix
// Players: Collide with Projectiles + Walls (not other players)
// Projectiles: Collide with Players + Walls + Boids
// Walls: Collide with all
// Boids: Collide with Projectiles + Walls (not other boids)
```

### Player Movement System

#### Input Processing
```rust
pub fn player_input_system(
    mut player_query: Query<(&mut PlayerInput, &Player, &mut ExternalForce, &mut ExternalImpulse, &Transform)>,
    time: Res<Time>,
) {
    for (mut input, player, mut force, mut impulse, transform) in player_query.iter_mut() {
        // Reset forces
        force.force = Vec2::ZERO;
        force.torque = 0.0;
        
        // Calculate thrust force
        if input.thrust > 0.0 {
            let movement_direction = input.movement.normalize_or_zero();
            let facing_direction = transform.rotation * Vec2::Y;
            
            // Forward speed boost when moving in facing direction
            let forward_alignment = movement_direction.dot(facing_direction);
            let speed_multiplier = if forward_alignment > 0.0 {
                1.0 + (player.forward_speed_multiplier - 1.0) * forward_alignment
            } else {
                1.0
            };
            
            force.force = movement_direction * player.thrust_force * input.thrust * speed_multiplier;
        }
        
        // Handle rotation
        if input.aim_direction.length() > 0.1 {
            let target_angle = input.aim_direction.y.atan2(input.aim_direction.x) - PI/2.0;
            let current_angle = transform.rotation.to_euler(EulerRot::ZYX).0;
            let angle_diff = (target_angle - current_angle + PI) % (2.0 * PI) - PI;
            
            impulse.torque_impulse = angle_diff * player.turn_rate * time.delta_seconds();
        }
    }
}
```

#### Movement Physics
```rust
pub fn player_movement_system(
    mut player_query: Query<(&Player, &mut Velocity, &Transform), With<Player>>,
    time: Res<Time>,
) {
    for (player, mut velocity, transform) in player_query.iter_mut() {
        // Apply damping for momentum feel
        let damping_factor = 0.95; // Adjust for desired momentum feel
        velocity.linvel *= damping_factor;
        velocity.angvel *= damping_factor;
        
        // Clamp max speed
        if velocity.linvel.length() > player.max_speed {
            velocity.linvel = velocity.linvel.normalize() * player.max_speed;
        }
    }
}
```

### Projectile System

#### Weapon System
```rust
pub fn shooting_system(
    mut commands: Commands,
    mut player_query: Query<(&PlayerInput, &mut Player, &WeaponStats, &Transform)>,
    time: Res<Time>,
) {
    for (input, mut player, weapon, transform) in player_query.iter_mut() {
        player.weapon_cooldown.tick(time.delta());
        
        if input.shooting && player.weapon_cooldown.finished() {
            // Reset cooldown
            player.weapon_cooldown.reset();
            
            // Spawn projectile
            let projectile_spawn_pos = transform.translation.truncate() + 
                (transform.rotation * Vec2::Y) * 30.0; // Offset from ship center
            
            let projectile_velocity = input.aim_direction * weapon.projectile_speed;
            
            commands.spawn((
                Projectile {
                    damage: weapon.damage,
                    owner: player_entity,
                    projectile_type: ProjectileType::Basic,
                    lifetime: Timer::new(weapon.projectile_lifetime, TimerMode::Once),
                    speed: weapon.projectile_speed,
                },
                ProjectilePhysics {
                    velocity: projectile_velocity,
                    spawn_time: Instant::now(),
                    max_lifetime: weapon.projectile_lifetime,
                },
                // Rapier2D components
                RigidBody::Dynamic,
                Collider::ball(2.0), // Small bullet collider
                CollisionGroups::new(
                    CollisionGroups::default().projectiles,
                    CollisionGroups::default().players | CollisionGroups::default().walls
                ),
                Velocity::linear(projectile_velocity),
                Transform::from_translation(projectile_spawn_pos.extend(0.0)),
                GlobalTransform::default(),
            ));
        }
    }
}
```

#### Projectile Physics
```rust
pub fn projectile_system(
    mut commands: Commands,
    mut projectile_query: Query<(Entity, &mut Projectile, &ProjectilePhysics, &Transform)>,
    time: Res<Time>,
) {
    for (entity, mut projectile, physics, transform) in projectile_query.iter_mut() {
        // Update lifetime
        projectile.lifetime.tick(time.delta());
        
        // Despawn if lifetime expired
        if projectile.lifetime.finished() {
            commands.entity(entity).despawn();
            continue;
        }
        
        // Check world bounds
        let pos = transform.translation.truncate();
        if pos.x.abs() > ARENA_WIDTH / 2.0 || pos.y.abs() > ARENA_HEIGHT / 2.0 {
            commands.entity(entity).despawn();
        }
    }
}
```

### Collision Detection System

#### Collision Event Handling
```rust
pub fn collision_system(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    mut player_query: Query<&mut Player>,
    projectile_query: Query<&Projectile>,
) {
    for collision_event in collision_events.read() {
        if let CollisionEvent::Started(entity1, entity2, _) = collision_event {
            // Check for player-projectile collision
            if let (Ok(mut player), Ok(projectile)) = (
                player_query.get_mut(*entity1),
                projectile_query.get(*entity2)
            ) {
                // Apply damage
                player.health -= projectile.damage;
                
                // Despawn projectile
                commands.entity(*entity2).despawn();
                
                // Handle player death
                if player.health <= 0.0 {
                    // Respawn logic or game over
                    handle_player_death(&mut commands, *entity1);
                }
            }
            
            // Check for projectile-wall collision
            if let Ok(projectile) = projectile_query.get(*entity1) {
                // Despawn projectile on wall hit
                commands.entity(*entity1).despawn();
            }
        }
    }
}

fn handle_player_death(commands: &mut Commands, player_entity: Entity) {
    // Reset player health and position
    // Or implement respawn timer
    // Or trigger game over state
}
```

### Arena System

#### World Boundaries
```rust
pub const ARENA_WIDTH: f32 = 2500.0;
pub const ARENA_HEIGHT: f32 = 1500.0;

pub fn setup_arena(mut commands: Commands) {
    let wall_thickness = 50.0;
    
    // Top wall
    commands.spawn((
        RigidBody::Fixed,
        Collider::cuboid(ARENA_WIDTH / 2.0, wall_thickness / 2.0),
        Transform::from_xyz(0.0, ARENA_HEIGHT / 2.0 + wall_thickness / 2.0, 0.0),
        CollisionGroups::new(
            CollisionGroups::default().walls,
            Group::ALL
        ),
    ));
    
    // Bottom wall
    commands.spawn((
        RigidBody::Fixed,
        Collider::cuboid(ARENA_WIDTH / 2.0, wall_thickness / 2.0),
        Transform::from_xyz(0.0, -ARENA_HEIGHT / 2.0 - wall_thickness / 2.0, 0.0),
        CollisionGroups::new(
            CollisionGroups::default().walls,
            Group::ALL
        ),
    ));
    
    // Left wall
    commands.spawn((
        RigidBody::Fixed,
        Collider::cuboid(wall_thickness / 2.0, ARENA_HEIGHT / 2.0),
        Transform::from_xyz(-ARENA_WIDTH / 2.0 - wall_thickness / 2.0, 0.0, 0.0),
        CollisionGroups::new(
            CollisionGroups::default().walls,
            Group::ALL
        ),
    ));
    
    // Right wall
    commands.spawn((
        RigidBody::Fixed,
        Collider::cuboid(wall_thickness / 2.0, ARENA_HEIGHT / 2.0),
        Transform::from_xyz(ARENA_WIDTH / 2.0 + wall_thickness / 2.0, 0.0, 0.0),
        CollisionGroups::new(
            CollisionGroups::default().walls,
            Group::ALL
        ),
    ));
}
```

### Player Spawning System

#### Player Initialization
```rust
pub fn spawn_player(
    commands: &mut Commands,
    player_id: u64,
    spawn_position: Vec2,
) -> Entity {
    commands.spawn((
        Player {
            player_id,
            health: 100.0,
            max_health: 100.0,
            thrust_force: 500.0,
            turn_rate: 5.0,
            forward_speed_multiplier: 1.5,
            weapon_cooldown: Timer::new(Duration::from_millis(250), TimerMode::Once),
        },
        PlayerInput {
            movement: Vec2::ZERO,
            aim_direction: Vec2::Y,
            thrust: 0.0,
            shooting: false,
            input_sequence: 0,
        },
        Ship {
            facing_direction: Vec2::Y,
            max_speed: 300.0,
            acceleration: 800.0,
            deceleration: 400.0,
            angular_velocity: 0.0,
        },
        WeaponStats {
            damage: 25.0,
            fire_rate: 4.0, // Shots per second
            projectile_speed: 600.0,
            projectile_lifetime: Duration::from_secs(3),
            spread: 0.0,
        },
        // Rapier2D components
        RigidBody::Dynamic,
        Collider::cuboid(15.0, 25.0), // Rectangular ship collider
        CollisionGroups::new(
            CollisionGroups::default().players,
            CollisionGroups::default().projectiles | CollisionGroups::default().walls
        ),
        Transform::from_translation(spawn_position.extend(0.0)),
        GlobalTransform::default(),
        Velocity::default(),
        ExternalForce::default(),
        ExternalImpulse::default(),
        Damping {
            linear_damping: 0.1,
            angular_damping: 0.3,
        },
    )).id()
}
```

## Performance Optimizations

### Object Pooling System

#### Projectile Pooling
```rust
#[derive(Resource)]
pub struct ProjectilePool {
    available: Vec<Entity>,
    active: HashSet<Entity>,
    pool_size: usize,
}

impl ProjectilePool {
    pub fn new(pool_size: usize) -> Self {
        Self {
            available: Vec::with_capacity(pool_size),
            active: HashSet::with_capacity(pool_size),
            pool_size,
        }
    }
    
    pub fn get_projectile(&mut self) -> Option<Entity> {
        if let Some(entity) = self.available.pop() {
            self.active.insert(entity);
            Some(entity)
        } else {
            None
        }
    }
    
    pub fn return_projectile(&mut self, entity: Entity) {
        if self.active.remove(&entity) {
            self.available.push(entity);
        }
    }
}

pub fn initialize_projectile_pool(
    mut commands: Commands,
    mut pool: ResMut<ProjectilePool>,
) {
    for _ in 0..pool.pool_size {
        let entity = commands.spawn((
            // Inactive projectile components
            Transform::default(),
            GlobalTransform::default(),
            Visibility::Hidden,
        )).id();
        
        pool.available.push(entity);
    }
}
```

### Spatial Optimization

#### Interest Management Preparation
```rust
pub fn update_player_interest_areas(
    mut commands: Commands,
    player_query: Query<(&Transform, &Player), With<Player>>,
    entity_query: Query<(Entity, &Transform), (Without<Player>, With<RigidBody>)>,
) {
    const INTEREST_RADIUS: f32 = 800.0; // Viewport + margin
    
    for (player_transform, player) in player_query.iter() {
        for (entity, entity_transform) in entity_query.iter() {
            let distance = player_transform.translation.distance(entity_transform.translation);
            
            if distance < INTEREST_RADIUS {
                // Entity is in interest area - ensure physics is active
                if !entity_query.get_component::<RigidBody>(entity).is_ok() {
                    // Add physics components if not present
                    commands.entity(entity).insert(RigidBody::Dynamic);
                }
            } else {
                // Entity is out of interest area - can disable physics
                if entity_query.get_component::<RigidBody>(entity).is_ok() {
                    // Remove physics components to save performance
                    commands.entity(entity).remove::<RigidBody>();
                }
            }
        }
    }
}
```

### Memory Management

#### Entity Cleanup
```rust
pub fn cleanup_system(
    mut commands: Commands,
    projectile_query: Query<Entity, (With<Projectile>, Without<RigidBody>)>,
    dead_player_query: Query<Entity, (With<Player>, Without<Transform>)>,
) {
    // Clean up projectiles that lost their physics body
    for entity in projectile_query.iter() {
        commands.entity(entity).despawn();
    }
    
    // Clean up disconnected players
    for entity in dead_player_query.iter() {
        commands.entity(entity).despawn();
    }
}
```

## Boid Integration Strategy

### Future Boid Components
```rust
#[derive(Component)]
pub struct Boid {
    pub boid_type: BoidType,
    pub health: f32,
    pub speed: f32,
    pub target: Option<Entity>,
    pub ai_state: BoidAIState,
}

// Boids will use same physics system
pub fn spawn_boid_with_physics(
    commands: &mut Commands,
    position: Vec2,
    boid_type: BoidType,
) -> Entity {
    commands.spawn((
        Boid {
            boid_type,
            health: 10.0,
            speed: 100.0,
            target: None,
            ai_state: BoidAIState::Idle,
        },
        // Same Rapier2D components as players
        RigidBody::Dynamic,
        Collider::ball(8.0),
        CollisionGroups::new(
            CollisionGroups::default().boids,
            CollisionGroups::default().projectiles | CollisionGroups::default().walls
        ),
        Transform::from_translation(position.extend(0.0)),
        GlobalTransform::default(),
        Velocity::default(),
    )).id()
}
```

### Dynamic Physics Management
```rust
pub fn manage_boid_physics(
    mut commands: Commands,
    player_query: Query<&Transform, With<Player>>,
    boid_query: Query<(Entity, &Transform), With<Boid>>,
) {
    const PHYSICS_ACTIVATION_DISTANCE: f32 = 1000.0;
    
    for (boid_entity, boid_transform) in boid_query.iter() {
        let near_player = player_query.iter().any(|player_transform| {
            boid_transform.translation.distance(player_transform.translation) < PHYSICS_ACTIVATION_DISTANCE
        });
        
        if near_player {
            // Activate physics for visible boids
            commands.entity(boid_entity).insert((
                RigidBody::Dynamic,
                Collider::ball(8.0),
                Velocity::default(),
            ));
        } else {
            // Deactivate physics for distant boids
            commands.entity(boid_entity).remove::<RigidBody>();
            commands.entity(boid_entity).remove::<Collider>();
            commands.entity(boid_entity).remove::<Velocity>();
        }
    }
}
```

## Implementation Phases

### Phase 1: Core Player Physics (Week 1)
**Goal**: Basic player movement and physics

**Tasks**:
- Implement Player and PlayerInput components
- Create player movement system with momentum
- Set up Rapier2D physics integration
- Add arena boundaries and collision

**Deliverables**:
- Players can move with twin-stick controls
- Ships have momentum and independent rotation
- Forward speed boost working
- Collision with walls

### Phase 2: Projectile System (Week 2)
**Goal**: Shooting mechanics and projectile physics

**Tasks**:
- Implement Projectile components and spawning
- Create shooting system with weapon stats
- Add projectile physics and movement
- Implement projectile cleanup and pooling

**Deliverables**:
- Players can shoot projectiles
- Bullets have proper physics and lifetime
- Weapon cooldown system working
- Object pooling for performance

### Phase 3: Collision Detection (Week 3)
**Goal**: Damage system and collision events

**Tasks**:
- Implement collision event handling
- Add damage calculation system
- Create player death and respawn logic
- Handle projectile-wall collisions

**Deliverables**:
- Player-projectile collisions working
- Damage and health system functional
- Projectiles despawn on wall hits
- Player respawn mechanics

### Phase 4: Input Processing (Week 4)
**Goal**: Network-ready input system

**Tasks**:
- Implement input buffering and validation
- Add input sequence numbering
- Create input prediction foundation
- Handle edge cases and error conditions

**Deliverables**:
- Input system ready for networking
- Smooth and responsive controls
- Input validation and rate limiting
- Foundation for client prediction

### Phase 5: Performance Optimization (Week 5)
**Goal**: Optimize for target performance

**Tasks**:
- Profile physics performance
- Optimize collision detection
- Implement spatial partitioning
- Add performance monitoring

**Deliverables**:
- System handles hundreds of projectiles
- Consistent 60Hz physics performance
- Memory usage optimized
- Performance metrics and monitoring

### Phase 6: Boid Integration Preparation (Week 6)
**Goal**: Prepare for boid system integration

**Tasks**:
- Implement interest management system
- Add dynamic physics component management
- Create boid collision group setup
- Test scalability improvements

**Deliverables**:
- Interest management system working
- Dynamic physics activation/deactivation
- Collision groups ready for boids
- Performance baseline for boid integration

## Testing Strategy

### Unit Tests
- **Component Systems**: Test individual physics components
- **Collision Detection**: Verify collision group interactions
- **Input Processing**: Test input validation and buffering
- **Performance**: Benchmark physics system performance

### Integration Tests
- **Player Movement**: Test complete movement pipeline
- **Shooting System**: Verify projectile spawning and physics
- **Collision Events**: Test damage and death systems
- **Arena Bounds**: Verify boundary collision handling

### Performance Tests
- **Entity Scale**: Test with hundreds of projectiles
- **Physics Simulation**: Verify 60Hz performance
- **Memory Usage**: Monitor memory allocation patterns
- **Network Load**: Test input processing at 60Hz

### Stress Tests
- **Projectile Spam**: Maximum projectile density
- **Rapid Movement**: High-speed player movement
- **Wall Collisions**: Rapid collision event generation
- **Memory Pressure**: Long-running performance

## Risk Assessment

### Technical Risks

#### 1. Physics Performance
**Risk**: Rapier2D performance with hundreds of projectiles
**Mitigation**: 
- Implement object pooling early
- Use collision filtering aggressively
- Profile and optimize hot paths

#### 2. Network Synchronization
**Risk**: Input lag with 60Hz input processing
**Mitigation**:
- Implement input prediction
- Use delta compression
- Optimize network packet size

#### 3. Memory Management
**Risk**: Memory leaks from entity spawning/despawning
**Mitigation**:
- Implement robust object pooling
- Add memory monitoring systems
- Regular cleanup passes

### Performance Risks

#### 1. Collision Detection Scale
**Risk**: Collision detection may not scale to boid levels
**Mitigation**:
- Interest management system
- Spatial partitioning optimization
- Collision group filtering

#### 2. Physics Simulation Stability
**Risk**: Physics instability with fast-moving projectiles
**Mitigation**:
- Tune physics parameters carefully
- Use continuous collision detection
- Implement velocity clamping

## Success Metrics

### Technical Metrics
- **Frame Rate**: Consistent 60Hz physics simulation
- **Entity Count**: Handle 8 players + 500+ projectiles
- **Memory Usage**: <200MB for physics system
- **Input Latency**: <16ms input processing time

### Performance Metrics
- **Collision Detection**: <5ms per frame
- **Physics Simulation**: <10ms per frame
- **Memory Allocation**: <1MB/sec allocation rate
- **Network Bandwidth**: <5KB/s per player for physics

### Quality Metrics
- **Stability**: No physics crashes in 4-hour test
- **Responsiveness**: Input response feels immediate
- **Consistency**: Deterministic physics simulation
- **Scalability**: Ready for boid integration

## Conclusion

This implementation plan provides a comprehensive roadmap for implementing player and shooting physics in Boid Wars. The design prioritizes performance and scalability while maintaining the arcade-style gameplay feel. The unified Rapier2D approach ensures consistency across all entity types and simplifies the integration of future boid enemies.

The phased implementation approach allows for iterative development and testing, ensuring each component is solid before building the next layer. The interest management system provides a clear path for scaling to the massive entity counts required for the full boid system.

The architecture is designed to be networking-friendly with proper input handling and state synchronization preparation, ensuring smooth multiplayer gameplay when the network layer is integrated.

## Next Steps

1. **Team Review**: Review this proposal with the development team
2. **Technical Validation**: Validate Rapier2D performance assumptions
3. **Implementation Planning**: Assign team members to implementation phases
4. **Development Start**: Begin Phase 1 implementation
5. **Progress Tracking**: Establish regular progress reviews and performance benchmarks

---

*This proposal will be updated as implementation progresses and performance characteristics are validated.*
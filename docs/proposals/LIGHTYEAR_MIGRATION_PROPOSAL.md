# Lightyear Migration Proposal: 0.21 → 0.20

## Overview

This document outlines the migration strategy from Lightyear 0.21 to 0.20 for the Boid Wars project. The migration addresses implementation challenges with the 0.21 complete rewrite while maintaining our core architectural goals.

## Executive Summary

**Recommendation**: Migrate from Lightyear 0.21 to 0.20

**Key Benefits**:
- More stable API with fewer breaking changes
- Better documentation and community examples
- Simpler architecture without subcrate complexity
- Faster time to market for networking implementation

**Impact**: Minimal architectural changes required; primarily implementation-level updates

## Version Comparison Analysis

### Lightyear 0.21 (Current Target)
- **Major rewrite** with significant architectural changes
- Split into multiple subcrates (transport, connection, syncing, input, replication)
- Moved from Resources to Entities for networking management
- More component-based networking approach
- Limited documentation and examples for new API

### Lightyear 0.20 (Proposed Target)
- Stable, mature API building on 0.17-0.19 patterns
- Unified architecture without subcrate complexity
- Proven track record with existing projects
- Comprehensive examples and documentation
- Same Bevy 0.16 compatibility

## Architecture Impact Assessment

### Current Architecture Compatibility
Our existing architecture document assumes Lightyear 0.21, but core concepts remain valid:

✅ **Unchanged Elements**:
- Server-authoritative design
- Entity-based replication
- WebTransport/QUIC support
- Interest management patterns
- WASM client bridge approach
- R-tree spatial indexing integration

⚠️ **Implementation Changes Required**:
- Plugin configuration syntax
- Replication component usage
- Connection management APIs
- Input handling patterns

### Technology Stack Updates

**Before (0.21)**:
```
- Networking: Lightyear 0.21 (WebTransport/QUIC)
- Networking: Lightyear 0.21 WASM client (thin bridge)
```

**After (0.20)**:
```
- Networking: Lightyear 0.20 (WebTransport/QUIC)
- Networking: Lightyear 0.20 WASM client (thin bridge)
```

All other technology choices remain identical.

## Required Code Changes

### 1. Dependencies (Cargo.toml)

**Current (0.21)**:
```toml
[dependencies]
lightyear = "0.21"
bevy = "0.16"
```

**Updated (0.20)**:
```toml
[dependencies]
lightyear = "0.20"
bevy = "0.16"
```

### 2. Protocol Definition

**Lightyear 0.20 Pattern**:
```rust
// shared/protocol.rs
use lightyear::*;

#[derive(Component, Message, Serialize, Deserialize)]
pub struct PlayerPosition(pub Vec2);

#[derive(Component, Message, Serialize, Deserialize)]
pub struct PlayerVelocity(pub Vec2);

#[derive(Component, Message, Serialize, Deserialize)]
pub struct BoidPosition(pub Vec2);

#[derive(Serialize, Deserialize)]
pub enum PlayerInput {
    Move(Vec2),
    Shoot(Vec2),
    ToggleAlliance(Entity),
}

#[derive(Serialize, Deserialize)]
pub enum GameMessage {
    PlayerJoined(u64),
    PlayerLeft(u64),
    AllianceFormed(u64, u64),
    BoidSpawned(Entity),
}

pub enum Channel {
    Reliable,
    Unreliable,
}

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        app
            // Messages
            .add_message::<GameMessage>()
            .add_direction(NetworkDirection::ServerToClient)
            
            // Components
            .register_component::<PlayerPosition>()
                .add_prediction(PredictionMode::Full)
                .add_interpolation(InterpolationMode::Full)
                .add_linear_interpolation_fn()
                
            .register_component::<PlayerVelocity>()
                .add_prediction(PredictionMode::Full)
                .add_interpolation(InterpolationMode::Full)
                .add_linear_interpolation_fn()
                
            .register_component::<BoidPosition>()
                .add_interpolation(InterpolationMode::Full)
                .add_linear_interpolation_fn()
            
            // Inputs
            .add_plugins(input::native::InputPlugin::<PlayerInput>::default())
            
            // Channels
            .add_channel::<Channel>(ChannelSettings {
                mode: ChannelMode::OrderedReliable(ReliableSettings::default()),
                ..default()
            });
    }
}
```

### 3. Server Setup

**Lightyear 0.20 Pattern**:
```rust
// server/main.rs
use lightyear::*;

pub struct BoidWarsServerPlugin;

impl Plugin for BoidWarsServerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(FixedUpdate, (
                handle_player_input,
                simulate_boids,
                handle_collisions,
                update_spatial_index,
            ))
            .add_systems(Update, (
                handle_new_connections,
                handle_disconnections,
                manage_interest_areas,
            ))
            .add_observer(handle_player_spawn)
            .add_observer(handle_boid_spawn);
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ProtocolPlugin)
        .add_plugins(ServerPlugins::new(ServerConfig {
            transport: ServerTransport::WebTransport {
                local_port: 5000,
                server_addr: "0.0.0.0:5000".parse().unwrap(),
            },
            ..default()
        }))
        .add_plugins(BoidWarsServerPlugin)
        .run();
}

fn handle_new_connections(
    mut commands: Commands,
    mut new_connections: EventReader<ConnectEvent>,
) {
    for event in new_connections.read() {
        info!("New player connected: {:?}", event.client_id);
        
        // Spawn player entity
        commands.spawn((
            PlayerPosition(Vec2::ZERO),
            PlayerVelocity(Vec2::ZERO),
            PlayerId(event.client_id),
            Replicate {
                sync: SyncTarget::All,
                controlled_by: ControlledBy::Client(event.client_id),
                ..default()
            },
        ));
    }
}
```

### 4. Client Setup

**Lightyear 0.20 Pattern**:
```rust
// client/main.rs
use lightyear::*;

pub struct BoidWarsClientPlugin;

impl Plugin for BoidWarsClientPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(FixedPreUpdate, 
                buffer_input.in_set(InputSet::WriteClientInputs)
            )
            .add_systems(FixedUpdate, (
                predicted_player_movement,
                handle_player_actions,
            ))
            .add_systems(Update, (
                render_players,
                render_boids,
                handle_ui,
            ))
            .add_observer(handle_predicted_spawn)
            .add_observer(handle_interpolated_spawn);
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ProtocolPlugin)
        .add_plugins(ClientPlugins::new(ClientConfig {
            transport: ClientTransport::WebTransport {
                client_addr: "127.0.0.1:0".parse().unwrap(),
                server_addr: "127.0.0.1:5000".parse().unwrap(),
            },
            ..default()
        }))
        .add_plugins(BoidWarsClientPlugin)
        .run();
}

fn buffer_input(
    mut input_query: Query<&mut ActionState<PlayerInput>, With<InputMarker<PlayerInput>>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    cursor: Res<CursorPosition>,
) {
    if let Ok(mut action_state) = input_query.single_mut() {
        let mut movement = Vec2::ZERO;
        
        if keyboard.pressed(KeyCode::KeyW) { movement.y += 1.0; }
        if keyboard.pressed(KeyCode::KeyS) { movement.y -= 1.0; }
        if keyboard.pressed(KeyCode::KeyA) { movement.x -= 1.0; }
        if keyboard.pressed(KeyCode::KeyD) { movement.x += 1.0; }
        
        if movement != Vec2::ZERO {
            action_state.value = Some(PlayerInput::Move(movement.normalize()));
        }
        
        if mouse.just_pressed(MouseButton::Left) {
            if let Some(cursor_pos) = cursor.0 {
                action_state.value = Some(PlayerInput::Shoot(cursor_pos));
            }
        }
    }
}
```

### 5. Boid System Integration

**Lightyear 0.20 Pattern**:
```rust
// server/boid_system.rs
use lightyear::*;

#[derive(Component)]
pub struct Boid {
    pub boid_type: BoidType,
    pub velocity: Vec2,
    pub target: Option<Entity>,
    pub health: f32,
}

fn simulate_boids(
    mut boid_query: Query<(&mut BoidPosition, &mut Boid)>,
    player_query: Query<&PlayerPosition, With<PlayerId>>,
    time: Res<Time>,
) {
    for (mut position, mut boid) in boid_query.iter_mut() {
        // Find nearest player
        let mut nearest_player = None;
        let mut nearest_distance = f32::INFINITY;
        
        for player_pos in player_query.iter() {
            let distance = position.0.distance(player_pos.0);
            if distance < nearest_distance {
                nearest_distance = distance;
                nearest_player = Some(player_pos.0);
            }
        }
        
        // Move towards nearest player
        if let Some(target_pos) = nearest_player {
            let direction = (target_pos - position.0).normalize();
            boid.velocity = direction * 100.0; // Boid speed
            position.0 += boid.velocity * time.delta_seconds();
        }
    }
}

fn spawn_boids(
    mut commands: Commands,
    time: Res<Time>,
    mut spawn_timer: Local<Timer>,
) {
    spawn_timer.tick(time.delta());
    
    if spawn_timer.just_finished() {
        // Spawn new boid
        commands.spawn((
            BoidPosition(Vec2::new(
                fastrand::f32() * 2000.0 - 1000.0,
                fastrand::f32() * 2000.0 - 1000.0,
            )),
            Boid {
                boid_type: BoidType::Swarmer,
                velocity: Vec2::ZERO,
                target: None,
                health: 10.0,
            },
            Replicate {
                sync: SyncTarget::All,
                controlled_by: ControlledBy::Server,
                ..default()
            },
        ));
    }
}
```

### 6. Interest Management

**Lightyear 0.20 Pattern**:
```rust
// server/interest_management.rs
use lightyear::*;

const INTEREST_RADIUS: f32 = 1000.0;

fn manage_interest_areas(
    mut entity_query: Query<(&Transform, &mut Replicate)>,
    player_query: Query<&Transform, (With<PlayerId>, Without<Replicate>)>,
) {
    for (entity_transform, mut replicate) in entity_query.iter_mut() {
        let mut should_replicate = false;
        
        // Check if entity is within interest radius of any player
        for player_transform in player_query.iter() {
            let distance = entity_transform.translation.distance(player_transform.translation);
            if distance < INTEREST_RADIUS {
                should_replicate = true;
                break;
            }
        }
        
        replicate.sync = if should_replicate {
            SyncTarget::All
        } else {
            SyncTarget::None
        };
    }
}
```

## Implementation Phases

### Phase 1: Dependency Migration (Week 1)
**Goal**: Update dependencies and basic setup

**Tasks**:
- Update Cargo.toml dependencies
- Fix compilation errors
- Basic server/client connection
- Protocol definition migration

**Deliverables**:
- Compiling server and client
- Basic connection establishment
- Simple message passing

### Phase 2: Core Systems Migration (Week 2)
**Goal**: Migrate core game systems

**Tasks**:
- Player input handling
- Basic entity replication
- Connection management
- Simple boid spawning

**Deliverables**:
- Player movement working
- Basic boid entities replicating
- Connection lifecycle management

### Phase 3: Game Logic Integration (Week 3)
**Goal**: Integrate full game systems

**Tasks**:
- Boid AI systems
- Collision detection
- Interest management
- Spatial indexing integration

**Deliverables**:
- 1000+ boids simulating
- Collision detection working
- Performance baseline established

### Phase 4: Optimization & Testing (Week 4)
**Goal**: Optimize and validate performance

**Tasks**:
- Performance profiling
- Memory usage optimization
- Network bandwidth measurement
- Stress testing

**Deliverables**:
- Target performance metrics achieved
- Stable 8-player matches
- 10k+ boids support validated

## Risk Assessment

### Technical Risks

#### 1. API Compatibility Issues
**Risk**: Unexpected API differences between versions
**Mitigation**: 
- Comprehensive testing of all networking features
- Fallback to 0.19 if critical issues found
- Gradual migration approach

#### 2. Performance Regression
**Risk**: 0.20 may have different performance characteristics
**Mitigation**:
- Baseline performance measurements
- Continuous performance monitoring
- Optimization strategies prepared

#### 3. WebTransport Support Changes
**Risk**: WebTransport implementation differences
**Mitigation**:
- Test WebTransport early in migration
- WebSocket fallback ready
- Browser compatibility testing

### Migration Risks

#### 1. Timeline Overrun
**Risk**: Migration takes longer than estimated
**Mitigation**:
- Conservative timeline estimates
- Parallel development approach
- Clear milestone definitions

#### 2. Team Productivity Impact
**Risk**: Migration disrupts other development
**Mitigation**:
- Dedicated migration team
- Clear communication channels
- Regular progress updates

## Success Metrics

### Technical Metrics
- **Compilation**: Clean compilation within 2 days
- **Basic Networking**: Client-server communication within 1 week
- **Entity Replication**: 1000+ entities replicating within 2 weeks
- **Performance**: 10k+ boids simulation within 3 weeks

### Quality Metrics
- **Stability**: No networking crashes in 4-hour stress test
- **Latency**: <50ms round-trip time maintained
- **Bandwidth**: <100KB/s per player for 8-player matches
- **Memory**: <500MB total server memory usage

## Benefits Analysis

### Development Benefits
1. **Faster Implementation**: Stable API reduces debugging time
2. **Better Documentation**: More examples and community resources
3. **Reduced Complexity**: Unified architecture easier to understand
4. **Proven Stability**: Lower risk of unexpected issues

### Long-term Benefits
1. **Maintainability**: Simpler codebase easier to maintain
2. **Team Onboarding**: Clearer architecture for new developers
3. **Feature Development**: Focus on game features, not networking issues
4. **Community Support**: More developers familiar with 0.20 patterns

### Business Benefits
1. **Time to Market**: Faster networking implementation
2. **Risk Reduction**: Lower technical risk profile
3. **Resource Efficiency**: Less time spent on networking debugging
4. **Scalability**: Proven architecture for future growth

## Conclusion

Migrating from Lightyear 0.21 to 0.20 represents a strategic decision to prioritize stability and development velocity over cutting-edge features. The 0.20 API provides all the networking capabilities required for Boid Wars while offering a more mature, well-documented development experience.

The migration plan outlined above provides a structured approach to implementing this change with minimal disruption to the overall project timeline. The four-phase approach allows for gradual migration while maintaining development momentum on other project aspects.

**Recommendation**: Proceed with migration to Lightyear 0.20 as outlined in this proposal.

## Next Steps

1. **Team Review**: Review this proposal with the development team
2. **Timeline Approval**: Confirm migration timeline fits project schedule
3. **Resource Allocation**: Assign dedicated migration team members
4. **Implementation Start**: Begin Phase 1 migration tasks
5. **Progress Tracking**: Establish regular migration progress reviews

---

*This proposal can be updated as migration progresses and additional requirements are discovered.*
# Coordinate System Refactor Proposal

## Current Issue

We currently have dual coordinate systems:
1. **Network Position** (`boid_wars_shared::Position`) - Used for network replication
2. **Physics Transform** (`bevy::Transform`) - Used for physics simulation

This creates:
- Potential for drift between systems
- Extra memory overhead
- Synchronization complexity
- Confusion about source of truth

## Proposed Solution

### Option 1: Physics as Source of Truth (Recommended)

Make the physics `Transform` the single source of truth and derive network positions from it.

```rust
// Remove Position component from entities, only add it during replication
fn prepare_replication(
    mut commands: Commands,
    changed_transforms: Query<(Entity, &Transform), (Changed<Transform>, With<Replicate>)>,
) {
    for (entity, transform) in changed_transforms.iter() {
        // Temporarily add Position for network serialization
        commands.entity(entity).insert(
            boid_wars_shared::Position(transform.translation.truncate())
        );
    }
}

fn cleanup_replication(
    mut commands: Commands,
    replicated: Query<Entity, With<boid_wars_shared::Position>>,
) {
    for entity in replicated.iter() {
        commands.entity(entity).remove::<boid_wars_shared::Position>();
    }
}
```

**Pros:**
- Single source of truth
- No synchronization needed
- Less memory usage
- Physics naturally handles interpolation

**Cons:**
- Requires custom Lightyear serialization
- More complex network layer

### Option 2: Network as Source of Truth

Make network `Position` primary and derive physics from it.

```rust
// Update physics from network position
fn sync_network_to_physics(
    mut transforms: Query<(&boid_wars_shared::Position, &mut Transform), Changed<Position>>,
) {
    for (position, mut transform) in transforms.iter_mut() {
        transform.translation.x = position.0.x;
        transform.translation.y = position.0.y;
    }
}
```

**Pros:**
- Simpler network integration
- Client authoritative movement easier

**Cons:**
- Physics becomes secondary
- Potential physics glitches
- Still need sync systems

### Option 3: Component Aliasing (Immediate Fix)

Keep both but ensure they're always synchronized with stricter guarantees.

```rust
// Ensure atomic updates
pub struct PositionSync;

impl Plugin for PositionSync {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (
                sync_physics_to_network
                    .run_if(resource_exists::<PhysicsResource>)
                    .before(NetworkReplicationSet),
                sync_network_to_physics
                    .run_if(not(resource_exists::<PhysicsResource>))
                    .after(NetworkReplicationSet),
            )
        );
    }
}
```

## Implementation Plan

### Phase 1: Immediate Mitigation (Option 3)
1. Add change detection to reduce sync overhead
2. Ensure proper system ordering
3. Add debug assertions for drift detection

### Phase 2: Refactor (Option 1)
1. Create custom Lightyear component mapper
2. Remove Position from entity spawning
3. Update all position queries to use Transform
4. Test network replication thoroughly

## Code Changes for Immediate Fix

```rust
// In physics.rs
fn sync_physics_to_network(
    mut query: Query<
        (&Transform, &mut boid_wars_shared::Position, &Velocity, &mut boid_wars_shared::Velocity),
        Or<(Changed<Transform>, Changed<Velocity>)>
    >,
) {
    for (transform, mut position, velocity, mut net_vel) in query.iter_mut() {
        let new_pos = transform.translation.truncate();
        if position.0.distance(new_pos) > 0.001 { // Only update if actually changed
            position.0 = new_pos;
        }
        if net_vel.0.distance(velocity.linvel) > 0.001 {
            net_vel.0 = velocity.linvel;
        }
    }
}
```

## Metrics for Success

1. Zero position drift in production
2. Reduced CPU usage in sync systems
3. Clearer code architecture
4. Easier debugging of position issues
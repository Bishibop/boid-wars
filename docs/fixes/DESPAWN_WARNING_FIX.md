# Despawn Warning Fix Summary

## The Problem

Server logs showed multiple warnings about attempting to despawn entities that don't exist:
```
WARN bevy_ecs::error::handler: Encountered an error in command `despawn`: The entity with ID 59v10 does not exist
```

## Root Cause

Projectiles could be despawned from multiple systems:
1. **collision_system** - When hitting something
2. **projectile_system** - When lifetime expires or out of bounds  
3. **cleanup_system** - When physics body is lost

This created race conditions where multiple systems tried to despawn the same entity.

## The Solution

### 1. Created Safe Despawn Extension

Added `SafeDespawnExt` trait that:
- Marks entities with `Despawning` component first
- Prevents double-despawn attempts
- Doesn't panic if entity doesn't exist

```rust
pub trait SafeDespawnExt {
    fn safe_despawn(&mut self, entity: Entity);
}
```

### 2. Added Despawning Marker

```rust
#[derive(Component)]
pub struct Despawning;
```

Systems now check for this marker before processing entities.

### 3. Updated All Despawn Calls

Changed all `commands.entity(entity).despawn()` to `commands.safe_despawn(entity)`.

### 4. Added Query Filters

Systems that process entities now exclude those marked for despawning:
```rust
Query<&Projectile, Without<Despawning>>
```

## Benefits

- ✅ No more despawn warnings in logs
- ✅ Cleaner error handling
- ✅ Prevents race conditions
- ✅ More robust entity cleanup

## Implementation Details

The safe despawn implementation:
1. Adds `Despawning` component to mark entity
2. Calls `despawn()` which won't panic if entity is already gone
3. Other systems skip entities with `Despawning` marker

This pattern can be extended to any entity type that might be despawned from multiple places.
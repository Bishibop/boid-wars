# Position Synchronization Architecture

## Overview

Boid Wars uses a dual position system to balance physics simulation accuracy with network efficiency. This document explains the architecture, ownership rules, and best practices.

## Components

### Core Position Components

1. **`Transform` (Bevy/Physics)**
   - Source: `bevy::prelude::Transform`
   - Purpose: Physics simulation and rendering
   - Contains: 3D position, rotation, scale
   - Authority: Physics engine on server, interpolation on client

2. **`Position` (Network)**
   - Source: `boid_wars_shared::Position`
   - Purpose: Network replication
   - Contains: 2D position only (Vec2)
   - Authority: Derived from Transform on server

### Sync Components

- **`SyncConfig`**: Configuration for sync behavior
- **`DriftMetrics`**: Tracks position drift between systems
- **`SyncPerformanceMetrics`**: Monitors sync performance

## System Ordering and Ownership

```
Frame Timeline:
├── Input Collection
├── FixedUpdate
│   ├── Physics Input Processing
│   ├── Physics Simulation (Transform updated)
│   └── Physics Writeback
├── PostUpdate
│   ├── SyncSet::PhysicsToNetwork (Transform → Position)
│   ├── Network Replication
│   └── SyncSet::DriftDetection
└── Render
```

### Ownership Rules

1. **During Physics Simulation** (FixedUpdate)
   - Transform is authoritative
   - Physics engine updates Transform
   - Position is stale until sync

2. **During Network Replication** (PostUpdate)
   - Position is synced from Transform
   - Only changed Transforms trigger sync
   - Network sees consistent Position values

3. **On Client** (No physics)
   - Position is authoritative (from network)
   - Transform is derived from Position
   - Used for rendering only

## Configuration

```rust
// Adjust sync behavior
app.insert_resource(SyncConfig {
    drift_threshold: 0.1,        // Max allowed drift
    min_sync_distance: 0.001,    // Minimum change to sync
    min_sync_velocity: 0.001,    // Velocity sync threshold
    auto_correct_drift: true,    // Auto-fix drift
});
```

## Performance Considerations

### Change Detection
- Only syncs when Transform actually changes
- Configurable minimum change thresholds
- Reduces unnecessary network traffic

### Monitoring
```rust
// Check sync performance
if let Some(metrics) = world.get_resource::<SyncPerformanceMetrics>() {
    println!("Positions synced: {}", metrics.positions_synced);
    println!("Sync time: {:.2}ms", metrics.sync_time_ms);
}
```

### Debug Mode
In debug builds:
- Drift detection runs every frame
- Warnings logged for drift > threshold
- Performance metrics logged every 5 seconds

## Best Practices

### DO
- Trust Transform during physics simulation
- Trust Position during network replication
- Use change detection for queries
- Monitor drift metrics in development

### DON'T
- Modify both Transform and Position manually
- Assume they're always perfectly synced
- Skip the sync systems
- Ignore drift warnings

## Common Issues

### Issue: Position Drift
**Symptom**: Physics and network positions diverge
**Solution**: Check system ordering, ensure sync systems run

### Issue: Jittery Movement
**Symptom**: Entities jump between positions
**Solution**: Increase `min_sync_distance` threshold

### Issue: High CPU Usage
**Symptom**: Sync systems using too much CPU
**Solution**: Use change detection, increase thresholds

## Future Improvements

1. **Predictive Sync**: Predict position changes to reduce latency
2. **Compression**: Delta compression for position updates
3. **Interpolation**: Smooth position transitions on client
4. **Adaptive Thresholds**: Adjust sync thresholds based on entity type

## Testing

Run with drift detection:
```bash
RUST_LOG=warn cargo run --features debug
```

Monitor sync performance:
```bash
RUST_LOG=info cargo run | grep "Position Sync Performance"
```
# Visual Effects Implementation Plan

## Overview
Add client-side visual effects to enhance gameplay feedback without network overhead. All effects are purely cosmetic and rendered locally on each client.

## Available Assets

### Explosion Effects
- `Shmup Final/Particle Effects/explosion1.png` - Sprite sheet with explosion frames
- `craftpix-981156-space-shooter-game-kit/Spaceship-2d-game-sprites/PNG/Ship_01/Explosion/explosion_atlas.png` - Ship explosion animation

### Hit/Impact Effects
- `Shmup Final/Particle Effects/hit.png` - Small hit spark effect

### Engine/Thrust Effects
- `Shmup Final/Boosters/booster1.png` - Engine flame variations (4 colors, 4 sizes)
- `craftpix-981156-space-shooter-game-kit/Spaceship-2d-game-sprites/PNG/Ship_01/Exhaust/exhaust_atlas.png` - Animated exhaust

## Implementation Strategy

### 1. Asset Organization
Copy only the effects we'll use to `game-assets/effects/`:
- `explosion_frames.png` - For death animations
- `hit_spark.png` - For projectile impacts
- `engine_flames.png` - For thrust visualization

### 2. Client-Side Components
Add to `bevy-client/src/lib.rs`:

```rust
// Effect Components (client-only, not networked)
#[derive(Component)]
struct EngineEffect {
    owner: Entity,
    offset: Vec2, // Position relative to ship
}

#[derive(Component)]
struct ExplosionAnimation {
    current_frame: usize,
    total_frames: usize,
    frame_timer: Timer,
    scale: f32,
}

#[derive(Component)]
struct HitEffect {
    lifetime: Timer,
    initial_scale: f32,
}

#[derive(Component)]
struct EffectSprite; // Marker to distinguish from game sprites
```

### 3. Effect Systems

#### Engine Effects System
```rust
fn update_engine_effects(
    mut commands: Commands,
    engine_sprite: Res<EngineSprite>,
    ships: Query<(Entity, &Velocity, &Transform, &Rotation), Changed<Velocity>>,
    engines: Query<(Entity, &EngineEffect)>,
) {
    // Show engine flames when ship has velocity
    // Hide when stopped
    // Position behind ship based on rotation
    // Scale based on velocity magnitude
}
```

#### Death Explosion System
```rust
fn spawn_death_explosions(
    mut commands: Commands,
    explosion_sprite: Res<ExplosionSprite>,
    mut removed: RemovedComponents<Health>,
    transform_query: Query<&Transform>,
) {
    // When entity is removed (health = 0)
    // Spawn explosion at last position
    // Play through animation frames
    // Clean up when complete
}
```

#### Hit Effect System
```rust
fn spawn_hit_effects(
    mut commands: Commands,
    hit_sprite: Res<HitSprite>,
    mut removed_projectiles: RemovedComponents<Projectile>,
    transform_query: Query<&Transform>,
) {
    // When projectile is removed (hit something)
    // Spawn spark effect at impact point
    // Fade out over 0.2 seconds
}
```

### 4. Rendering Considerations

#### Z-Order Layers
- Background: 0.0 - 1.0
- Boundaries: 2.0
- Game entities: 3.0
- Projectiles: 4.0
- **Engine effects: 2.5** (behind ships)
- **Hit effects: 4.5** (above projectiles)
- **Explosions: 5.0** (top layer)

#### Performance
- Pool effect entities when possible
- Limit simultaneous effects
- Use simple fade/scale animations
- Clean up completed effects promptly

### 5. Integration Points

#### Health System
- Monitor for health reaching 0
- Trigger explosion when entity dies
- Hide original sprite during explosion

#### Movement System
- Check velocity changes
- Show/hide engine effects
- Scale effect based on thrust

#### Collision Detection
- Already handled by server
- Client reacts to entity removal
- Spawn effects at last known position

## Benefits of Client-Side Approach

1. **No Network Overhead** - Effects don't affect gameplay
2. **Immediate Response** - No waiting for server
3. **Customizable** - Players could have effect settings
4. **Simple Implementation** - No protocol changes
5. **Better Performance** - Distributed rendering

## Future Enhancements

1. **Particle Systems** - For more complex effects
2. **Sound Effects** - Audio cues for actions
3. **Screen Shake** - Camera effects for explosions
4. **Damage Numbers** - Floating combat text
5. **Trail Effects** - Motion trails for fast objects
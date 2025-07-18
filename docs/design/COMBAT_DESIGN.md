# Combat System Design

This document defines the combat mechanics for Boid Wars, including health systems, damage, death mechanics, and AI combat behavior.

## Health & Damage

### Health Values
**Question**: How much health should entities have?
- Player starting health: `100`
- Basic boid health: `10-30` (scaled to match damage values)
- Should different boid types have different health? `Yes`
  - Scout boid: `10 HP` (one-shot by player)
  - Standard boid: `20 HP` (two-shot by player)
  - Heavy boid: `30-50 HP` (3-5 shots needed)

### Damage Model
**Question**: How much damage do projectiles deal?
- Player projectile damage: `10` (base damage)
- Boid projectile damage: `5` (player survives 20 hits)
- Is damage consistent or variable? `Consistent for now` (can add variance later)
- Can anything be one-shot killed? `Yes - Scout boids (10 HP) die in one player shot`

### Health Regeneration
**Question**: How do players recover health?
- Regeneration type: `Slow regeneration over time`
- Regeneration rate: `2 HP/second after 3 seconds of no damage`
- Maximum health: `100` (cannot exceed starting health)
- Future additions: `Health pickups planned but not in initial implementation`

## Death & Respawning

### Player Death
**Question**: What happens when a player dies?
- Death behavior: `Permanent death in Battle Royale mode`
- Death duration: `Immediate - player is eliminated`
- Lives/continues: `None - one life per match`
- Post-death: `Spectate remaining players (not implemented initially)`
- Other modes: `Co-op would have partner revive (future feature)`

### Respawn Mechanics
**Question**: If players respawn, how does it work?
- Respawn location: `N/A - no respawning in Battle Royale`
- Respawn delay: `N/A`
- Invulnerability period: `N/A`
- Starting equipment: `N/A`
- Note: `Respawning only relevant for future game modes`

### Death Effects
**Question**: What feedback shows when something dies?
- Visual effects: `Explosion particles`
  - Boid death: Small particle burst (performant for many deaths)
  - Player death: Larger explosion effect
- Audio feedback: `Death sounds` (to be determined - simple for performance)
- Drops on death: `None initially` (pickups/drops planned for future)

## Boid Combat Behavior

### Aggression Triggers
**Question**: When do boids attack players?
- Trigger condition: `Range-based` (close proximity triggers shooting)
- Aggression range: `200 units` (starting value - will calibrate for "hectic" gameplay)
- Memory duration: `3 seconds` (continues shooting for 3s after player leaves range)
- Future: `Mixed behaviors based on boid type`
  - Aggressive scouts: Always shoot on sight
  - Defensive heavies: Only when attacked
  - Standard: Current range-based behavior

### Shooting Behavior
**Question**: How do boids shoot?
- Accuracy model: `Random spread` (cone of ±15 degrees from target direction)
- Fire rate: `0.5 shots/second` (one shot every 2 seconds)
- Burst patterns: `Single shot` (one projectile per shot)
- Reload/cooldown: `2 second cooldown between shots`
- Note: `Will need to calibrate spread angle and fire rate for desired difficulty`

### Target Selection
**Question**: How do boids choose targets?
- Priority: `Most aggressive` (targets players who are shooting at them)
- Target switching: `Reactive` (switches to most recent attacker)
- Multi-target capability: `Individual targeting` (each boid tracks its own aggressor)
- Fallback: `If no aggressor, target nearest player within range`
- Memory: `Remember last attacker for 5 seconds`

## Combat Feel

### Hit Feedback
**Question**: How do we show something got hit?
- Visual feedback: `Particle sparks` (small spark effect at impact point)
- Audio feedback: `Hit sound` (distinct from death sound)
- Knockback/stun: `None` (maintains smooth movement flow)
- Performance note: `Keep particles lightweight for many simultaneous hits`

### Kill Feedback
**Question**: How do we reward successful kills?
- Visual effects: `Small explosion or flash` (slightly bigger than hit sparks)
- Score/points: `None` (no point system)
- Kill streaks: `None` (no streak tracking)
- Announcements: `None` (keep it simple)
- Audio: `Small kill confirmation sound`

### Difficulty Progression
**Question**: How does combat difficulty scale?
- Scaling factors: `Zone-based` (harder enemies toward map center)
- Progression trigger: `Distance from center` (concentric difficulty zones)
- Difficulty curve: `Stepped` (distinct zones with different boid types)
  - Outer zone: Scout boids only (10 HP)
  - Mid zone: Mix of scouts and standard boids (20 HP)
  - Inner zone: All types including heavy boids (30-50 HP)
  - Center: Maximum density and aggression

## Technical Specifications

### Performance Limits
- Maximum active boid projectiles: `None initially` (monitor performance)
- Projectile pooling strategy: `Separate pools` (boids have own projectile pool)
- Corpse cleanup delay: `Instant` (no corpses, immediate removal on death)
- Note: `Will add limits if performance issues arise`

### Architecture Decisions
- Health component structure: `Separate components with shared logic`
  - `PlayerHealth` component (with regen timer)
  - `BoidHealth` component (simpler, no regen)
  - Shared damage processing systems
- Damage system: `Event-based` (collision events trigger damage)
  - Projectile collision publishes damage event
  - Health systems listen and apply damage
  - Allows for future damage modifiers/shields
- Team/faction system: `Component tags`
  - `PlayerTeam` and `BoidTeam` components
  - Boids never damage each other (same team)
  - Future: Could have rival boid factions

### Network Synchronization
- Damage authority: `Server-only` (authoritative damage calculation)
- Health sync frequency: `Standard network tick rate` (30Hz with other state)
- Death state sync: `Immediate event` (death is high-priority message)
- Client prediction: `None initially` (can add if latency becomes issue)
- Note: `Start simple, optimize based on playtesting`

## Implementation Priority

### Phase 1: Core Health & Death (MVP)
1. **Health Components**
   - Add `PlayerHealth { current: i32, max: i32 }` component
   - Add `BoidHealth { current: i32 }` component
   - Death detection when health <= 0
2. **Basic Damage System**
   - Collision detection triggers damage
   - Direct health reduction (no events yet)
   - Players take 5 damage from boid projectiles
   - Boids take 10 damage from player projectiles
3. **Death & Cleanup**
   - Instant entity despawn on death
   - No visual effects yet
   - Test: Players can kill boids, boids can kill players
4. **Health Bars**
   - Player health bar (always visible, UI element)
   - Boid health bars (small, above entity, only when damaged)
   - Show current/max health visually

### Phase 2: Boid Combat AI
1. **Boid Shooting**
   - Add shooting capability to boids
   - 200-unit aggression range
   - Random spread (±15 degrees)
   - 2-second cooldown between shots
2. **Target Selection**
   - Track which player shot each boid
   - Boids target their most recent attacker
   - 5-second memory for attackers
   - Fallback to nearest player if no attacker

### Phase 3: Combat Polish
1. **Event-Based Damage**
   - Create `DamageEvent { entity, amount, source }`
   - Refactor to event-driven system
   - Add team components to prevent friendly fire
2. **Visual Feedback**
   - Hit sparks particle effect
   - Death explosion particles (small for boids, large for players)
   - Use existing particle system or simple sprites
3. **Audio Feedback**
   - Hit sound effect
   - Death sound effect
   - Different sounds for player vs boid

### Phase 4: Advanced Features
1. **Health Regeneration**
   - Add regen timer to PlayerHealth
   - 2 HP/second after 3 seconds without damage
   - Cap at max health (100)
2. **Zone-Based Difficulty**
   - Define concentric zones from center
   - Spawn different boid types by zone
   - Outer: Scouts (10 HP)
   - Middle: Standard (20 HP)  
   - Inner: Heavy (30-50 HP)
3. **Separate Projectile Pools**
   - Create `BoidProjectilePool` resource
   - Separate from player projectile pool
   - Monitor performance and add limits if needed

### Phase 5: Multiplayer & Polish
1. **Network Sync**
   - Server-authoritative damage
   - Health state replication
   - Death event priority messages
2. **Performance Optimization**
   - Profile with 10,000 boids
   - Add projectile limits if needed
   - Optimize particle effects
3. **Balance Tuning**
   - Adjust fire rates, damage, ranges
   - Playtest for "hectic" feel
   - Fine-tune zone difficulty

---

## Design Notes
- Start with Phase 1 to get basic combat working
- Each phase builds on the previous one
- Can ship after any phase for incremental releases
- Performance testing critical after Phase 3 (AI shooting)

## Technical Implementation Details

### Phase 1: Core Health & Death - Technical Approach

#### 1.1 Health Components
The `Health` component already exists in `shared/src/protocol.rs`:
```rust
pub struct Health { current: f32, max: f32 }
```

**For Players**: The `Player` component already has health fields. We'll use these instead of adding Health component to players:
- Location: `server/src/physics.rs:378`
- Already has `health: f32` and `max_health: f32`
- Default is 100.0 from GAME_CONFIG

**For Boids**: Already use the shared `Health` component. No changes needed.

#### 1.2 Damage Application
Current collision system (`collision_system` in `physics.rs:809-867`) already handles damage:
- Projectile-Player: `player.health -= projectile.damage`
- Projectile-Boid: `health.current -= projectile.damage`

**Changes needed**:
1. Update projectile damage from 25.0 to design values:
   - Player projectiles: 10 damage
   - Boid projectiles: 5 damage (for Phase 2)
2. Add damage clamping for players (currently only boids have it)

#### 1.3 Death Detection & Cleanup
Current implementation:
- Players: `handle_player_death()` called when health <= 0 (currently TODO)
- Boids: Marked with `Despawning` component when health <= 0

**Implementation tasks**:
1. Implement `handle_player_death()`:
   ```rust
   fn handle_player_death(
       commands: &mut Commands,
       entity: Entity,
       player_id: u64,
   ) {
       // Mark for cleanup
       commands.entity(entity).insert(Despawning);
       // TODO: Emit death event for UI/spectator mode
       // TODO: Update game state for battle royale
   }
   ```

2. Ensure cleanup_system processes dead players properly

#### 1.4 Health Bar Implementation
Since we're using Bevy WASM client, we need to implement health bars on both server and client:

**Server Side**:
- Health values already replicated via `Health` component and `Player` component
- No additional server changes needed

**Client Side** (bevy-client):
1. **Player Health UI**:
   ```rust
   // UI health bar component
   #[derive(Component)]
   struct PlayerHealthBar;
   
   // Spawn UI health bar during player spawn
   fn spawn_player_health_ui(commands: &mut Commands) {
       commands.spawn((
           NodeBundle {
               style: Style {
                   position_type: PositionType::Absolute,
                   bottom: Val::Px(20.0),
                   left: Val::Px(20.0),
                   width: Val::Px(200.0),
                   height: Val::Px(20.0),
                   ..default()
               },
               background_color: Color::rgb(0.2, 0.2, 0.2).into(),
               ..default()
           },
           PlayerHealthBar,
       )).with_children(|parent| {
           // Health fill bar
           parent.spawn(NodeBundle {
               style: Style {
                   width: Val::Percent(100.0), // Updated based on health
                   height: Val::Percent(100.0),
                   ..default()
               },
               background_color: Color::rgb(0.8, 0.2, 0.2).into(),
               ..default()
           });
       });
   }
   ```

2. **Boid Health Bars** (world space):
   ```rust
   // Component to track health bar entity
   #[derive(Component)]
   struct HealthBarLink(Entity);
   
   // Spawn health bar above boid
   fn spawn_boid_health_bar(
       commands: &mut Commands,
       boid_entity: Entity,
       position: Vec3,
   ) {
       let bar = commands.spawn((
           SpriteBundle {
               sprite: Sprite {
                   color: Color::RED,
                   custom_size: Some(Vec2::new(20.0, 3.0)),
                   ..default()
               },
               transform: Transform::from_translation(
                   position + Vec3::new(0.0, 15.0, 0.1) // Above boid
               ),
               ..default()
           },
       )).id();
       
       commands.entity(boid_entity).insert(HealthBarLink(bar));
   }
   ```

3. **Update System**:
   ```rust
   fn update_health_bars(
       player_query: Query<&Player>,
       boid_query: Query<(&Health, &Transform, &HealthBarLink), With<Boid>>,
       mut bar_query: Query<&mut Style, With<PlayerHealthBar>>,
       mut sprite_query: Query<&mut Sprite>,
   ) {
       // Update player UI health bar
       // Update boid world-space health bars
   }
   ```

#### 1.5 Testing Phase 1
Create a test scenario:
1. Spawn player with 100 health (visible health bar)
2. Spawn boids with 10/20/30 health (health bars appear when damaged)
3. Verify player projectiles deal 10 damage (health bars update)
4. Verify boids die when health reaches 0 (health bar disappears)
5. Verify player dies after taking 100 damage (health bar empties)

### Phase 2: Boid Combat AI - Technical Approach

#### 2.1 Boid Combat Components
Add to boids (similar to Player structure):
```rust
// Add to Boid component or create BoidCombat component
struct BoidCombat {
    weapon_stats: WeaponStats,
    shoot_timer: Timer,
    target: Option<Entity>,
    last_attacker: Option<(Entity, Instant)>,
}
```

#### 2.2 Boid Weapon Configuration
Create boid-specific weapon stats:
```rust
impl WeaponStats {
    pub fn boid_default() -> Self {
        Self {
            damage: 5.0,
            fire_rate: 0.5, // Once per 2 seconds
            projectile_speed: 400.0, // Slower than player
            projectile_lifetime: Duration::from_secs(2),
            spread: 0.26, // ~15 degrees in radians
        }
    }
}
```

#### 2.3 Aggression Tracking
Extend existing `PlayerAggression` resource pattern:
```rust
#[derive(Resource)]
struct BoidAggression {
    // Map boid entity to (attacker entity, time)
    aggression_map: HashMap<Entity, (Entity, Instant)>,
}
```

Track when players damage boids in collision_system:
```rust
// When player projectile hits boid
if let Some(mut aggression) = boid_aggression {
    aggression.record_attack(boid_entity, projectile.owner);
}
```

#### 2.4 Boid Shooting System
Create new system following player shooting pattern:
```rust
fn boid_shooting_system(
    mut commands: Commands,
    time: Res<Time>,
    mut boid_query: Query<(
        Entity,
        &Transform,
        &mut BoidCombat,
        &Position,
    ), With<Boid>>,
    player_query: Query<(Entity, &Position), With<Player>>,
    aggression: Res<BoidAggression>,
    mut projectile_pool: ResMut<BoidProjectilePool>,
) {
    for (boid_entity, transform, mut combat, boid_pos) in &mut boid_query {
        // 1. Find target
        let target = find_boid_target(
            boid_entity,
            boid_pos,
            &player_query,
            &aggression,
            200.0, // aggression range
        );
        
        // 2. Update shoot timer
        combat.shoot_timer.tick(time.delta());
        
        // 3. Shoot if ready and has target
        if combat.shoot_timer.finished() && target.is_some() {
            spawn_boid_projectile(
                &mut commands,
                &mut projectile_pool,
                transform,
                target.unwrap(),
                &combat.weapon_stats,
            );
            combat.shoot_timer.reset();
        }
    }
}
```

#### 2.5 Target Selection Logic
```rust
fn find_boid_target(
    boid_entity: Entity,
    boid_pos: &Position,
    players: &Query<(Entity, &Position), With<Player>>,
    aggression: &BoidAggression,
    range: f32,
) -> Option<Vec2> {
    // 1. Check if boid has recent attacker
    if let Some((attacker, _)) = aggression.get_attacker(boid_entity) {
        if let Ok((_, pos)) = players.get(attacker) {
            return Some(pos.0);
        }
    }
    
    // 2. Fallback to nearest player in range
    let mut nearest = None;
    let mut min_dist = range;
    
    for (_, player_pos) in players.iter() {
        let dist = boid_pos.0.distance(player_pos.0);
        if dist < min_dist {
            min_dist = dist;
            nearest = Some(player_pos.0);
        }
    }
    
    nearest
}
```

#### 2.6 Projectile Spawning with Spread
```rust
fn spawn_boid_projectile(
    commands: &mut Commands,
    pool: &mut BoidProjectilePool,
    transform: &Transform,
    target_pos: Vec2,
    weapon: &WeaponStats,
) {
    // Calculate base direction
    let direction = (target_pos - transform.translation.truncate()).normalize();
    
    // Add random spread
    let spread_angle = thread_rng().gen_range(-weapon.spread..weapon.spread);
    let rotation = Quat::from_rotation_z(spread_angle);
    let final_direction = rotation * direction.extend(0.0);
    
    // Spawn using pool or create new
    // Similar to player projectile spawning
}
```

#### 2.7 Integration Points
1. Add `BoidCombat` component during boid spawning
2. Add `boid_shooting_system` to `PhysicsSet::Combat`
3. Create `BoidProjectilePool` resource similar to player pool
4. Update collision groups to handle boid projectiles
5. Track aggression in collision_system

### Key Differences from Player Shooting
1. Boids use spread for inaccuracy
2. Slower fire rate (0.5 vs 4.0 shots/sec)
3. Target selection based on aggression
4. Separate projectile pool for performance isolation
5. No input system - direct target detection

## Implementation Summary

### Phase 1 Quick Start (Minimal Changes)
1. **Update damage values** in `WeaponStats::default()` from 25.0 to 10.0
2. **Implement `handle_player_death()`** to mark players with `Despawning`
3. **Update boid health** in spawn functions to match design (10/20/30 HP)
4. **Add health bars**:
   - Player: UI element showing current/max health
   - Boids: World-space health bars above sprites
5. **Test**: Players can kill boids, boids can kill players, health visible

### Phase 2 Quick Start (Boid Shooting)
1. **Create `BoidCombat` component** with weapon stats and shoot timer
2. **Add `BoidAggression` resource** to track who shot each boid
3. **Create `boid_shooting_system`** that:
   - Finds targets within 200 units
   - Prefers recent attackers
   - Shoots with spread
4. **Spawn boid projectiles** with 5 damage, slower speed
5. **Update collision system** to track boid aggression
6. **Test**: Boids shoot back at players who attack them
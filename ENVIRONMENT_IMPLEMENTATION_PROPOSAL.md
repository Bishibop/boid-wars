# Environment Implementation Proposal: Derelict Megastructure

## Overview

This document outlines the technical implementation plan for the derelict megastructure environment in Boid Wars. The environment will serve as a 3D battlefield with varied terrain features that enhance tactical gameplay while maintaining the fast-paced bullet-hell experience.

## Architecture Integration

### Alignment with Existing Systems
- **Server**: Rust/Bevy ECS for authoritative environment state
- **Client**: TypeScript/Pixi.js for rendering and visual effects
- **Networking**: Lightyear for environment data replication
- **Physics**: Rapier 2D for collision detection and destruction
- **Spatial**: R-tree integration for efficient spatial queries

## Server-Side Implementation (Rust/Bevy ECS)

### Core Components

#### Environment Entity Components
```rust
#[derive(Component)]
pub struct EnvironmentObject {
    pub object_type: EnvironmentType,
    pub health: Option<f32>,        // None = indestructible
    pub collision_shape: CollisionShape,
    pub visual_id: u32,             // Links to client asset
}

#[derive(Component)]
pub struct Destructible {
    pub max_health: f32,
    pub current_health: f32,
    pub destruction_effects: Vec<DestructionEffect>,
    pub drops_debris: bool,
}

#[derive(Component)]
pub struct EnvironmentCollider {
    pub blocks_movement: bool,
    pub blocks_bullets: bool,
    pub damage_on_contact: Option<f32>,
}

#[derive(Component)]
pub struct ZoneAffected {
    pub destroy_on_shrink: bool,
    pub becomes_dangerous: bool,
}
```

#### Environment Types
```rust
pub enum EnvironmentType {
    // Static Structures
    CommTower,
    WeaponTurret,
    AntennaArray,
    HullSection,
    
    // Destructible Elements
    SmallDebris,
    SupportStrut,
    PowerConduit,
    
    // Interactive Features
    BoidNest,
    PowerSource,
    ShieldGenerator,
    
    // Hazards
    EnergyField,
    DebrisCloud,
    PlasmaVent,
}
```

### Spatial Integration

#### R-tree Integration
```rust
// Extend existing spatial system
impl SpatialQuery for EnvironmentObject {
    fn bounds(&self) -> AABB {
        // Return bounding box for R-tree insertion
    }
}

// Environment-specific spatial queries
pub fn query_environment_near_point(
    tree: &RTree<EnvironmentObject>,
    position: Vec2,
    radius: f32,
) -> Vec<EnvironmentObject> {
    // Efficient environment queries for boid AI
}
```

#### Collision Detection
```rust
// Rapier integration for environment collisions
pub fn setup_environment_colliders(
    mut commands: Commands,
    query: Query<(Entity, &EnvironmentObject, &Transform), Added<EnvironmentObject>>,
) {
    for (entity, env_obj, transform) in query.iter() {
        let collider = match env_obj.collision_shape {
            CollisionShape::Rectangle { width, height } => {
                Collider::cuboid(width / 2.0, height / 2.0)
            }
            CollisionShape::Circle { radius } => {
                Collider::ball(radius)
            }
            CollisionShape::Complex { vertices } => {
                Collider::convex_hull(&vertices).unwrap()
            }
        };
        
        commands.entity(entity).insert(collider);
    }
}
```

### Destructible Systems

#### Destruction Logic
```rust
pub fn handle_environment_damage(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Destructible, &EnvironmentObject)>,
    damage_events: EventReader<EnvironmentDamageEvent>,
) {
    for event in damage_events.iter() {
        if let Ok((entity, mut destructible, env_obj)) = query.get_mut(event.target) {
            destructible.current_health -= event.damage;
            
            if destructible.current_health <= 0.0 {
                // Trigger destruction effects
                spawn_destruction_effects(&mut commands, entity, &destructible);
                
                // Remove or replace with debris
                if destructible.drops_debris {
                    spawn_debris_pieces(&mut commands, entity, env_obj);
                }
                
                commands.entity(entity).despawn();
            }
        }
    }
}
```

#### Zone Shrinking Integration
```rust
pub fn apply_zone_effects(
    mut commands: Commands,
    query: Query<(Entity, &ZoneAffected, &Transform)>,
    zone_state: Res<ZoneState>,
) {
    for (entity, zone_affected, transform) in query.iter() {
        if !zone_state.is_position_safe(transform.translation.xy()) {
            if zone_affected.destroy_on_shrink {
                commands.entity(entity).despawn();
            } else if zone_affected.becomes_dangerous {
                // Add damage component
                commands.entity(entity).insert(DamageField { damage_per_second: 10.0 });
            }
        }
    }
}
```

## Client-Side Implementation (TypeScript/Pixi.js)

### Rendering Architecture

#### Multi-Layer Rendering
```typescript
class EnvironmentRenderer {
    private backgroundLayer: PIXI.Container;
    private structureLayer: PIXI.Container;
    private effectsLayer: PIXI.Container;
    private hudLayer: PIXI.Container;
    
    constructor(app: PIXI.Application) {
        this.setupLayers(app);
    }
    
    private setupLayers(app: PIXI.Application) {
        // Background (distant structures, stars)
        this.backgroundLayer = new PIXI.Container();
        this.backgroundLayer.zIndex = 0;
        
        // Main structures (collision objects)
        this.structureLayer = new PIXI.Container();
        this.structureLayer.zIndex = 1;
        
        // Effects (particles, glows)
        this.effectsLayer = new PIXI.Container();
        this.effectsLayer.zIndex = 2;
        
        // HUD elements
        this.hudLayer = new PIXI.Container();
        this.hudLayer.zIndex = 3;
        
        app.stage.addChild(
            this.backgroundLayer,
            this.structureLayer,
            this.effectsLayer,
            this.hudLayer
        );
    }
}
```

#### Level of Detail System
```typescript
class EnvironmentLOD {
    private lodThresholds = {
        high: 500,    // Show full detail
        medium: 1500, // Simplified sprites
        low: 3000,    // Simple shapes
        hidden: 5000  // Don't render
    };
    
    updateLOD(envObject: EnvironmentSprite, cameraDistance: number) {
        const lod = this.calculateLOD(cameraDistance);
        
        switch (lod) {
            case LODLevel.High:
                envObject.texture = this.highDetailTextures[envObject.type];
                envObject.visible = true;
                break;
            case LODLevel.Medium:
                envObject.texture = this.mediumDetailTextures[envObject.type];
                envObject.visible = true;
                break;
            case LODLevel.Low:
                envObject.texture = this.lowDetailTextures[envObject.type];
                envObject.visible = true;
                break;
            case LODLevel.Hidden:
                envObject.visible = false;
                break;
        }
    }
    
    private calculateLOD(distance: number): LODLevel {
        if (distance < this.lodThresholds.high) return LODLevel.High;
        if (distance < this.lodThresholds.medium) return LODLevel.Medium;
        if (distance < this.lodThresholds.low) return LODLevel.Low;
        return LODLevel.Hidden;
    }
}
```

#### Particle Effects System
```typescript
class EnvironmentEffects {
    private particleContainer: PIXI.ParticleContainer;
    private sparkPool: ObjectPool<PIXI.Sprite>;
    private smokePool: ObjectPool<PIXI.Sprite>;
    
    constructor() {
        this.particleContainer = new PIXI.ParticleContainer(1000, {
            scale: true,
            position: true,
            rotation: true,
            alpha: true,
            tint: true
        });
        
        this.sparkPool = new ObjectPool(() => new PIXI.Sprite(sparkTexture), 100);
        this.smokePool = new ObjectPool(() => new PIXI.Sprite(smokeTexture), 50);
    }
    
    createSparks(position: PIXI.Point, count: number) {
        for (let i = 0; i < count; i++) {
            const spark = this.sparkPool.get();
            spark.position.set(position.x, position.y);
            spark.scale.set(0.5 + Math.random() * 0.5);
            spark.rotation = Math.random() * Math.PI * 2;
            
            // Animate spark
            this.animateSpark(spark);
            this.particleContainer.addChild(spark);
        }
    }
    
    private animateSpark(spark: PIXI.Sprite) {
        const velocity = {
            x: (Math.random() - 0.5) * 200,
            y: (Math.random() - 0.5) * 200
        };
        
        const animate = () => {
            spark.position.x += velocity.x * 0.016;
            spark.position.y += velocity.y * 0.016;
            spark.alpha -= 0.02;
            
            if (spark.alpha <= 0) {
                this.particleContainer.removeChild(spark);
                this.sparkPool.release(spark);
            } else {
                requestAnimationFrame(animate);
            }
        };
        
        animate();
    }
}
```

### Performance Optimizations

#### Viewport Culling
```typescript
class EnvironmentCuller {
    private viewport: PIXI.Rectangle;
    private cullMargin = 200; // Extra margin for smooth transitions
    
    updateVisibility(envObjects: EnvironmentSprite[], camera: Camera) {
        this.viewport = camera.getViewport();
        
        for (const obj of envObjects) {
            const bounds = obj.getBounds();
            const isVisible = this.viewport.intersects(bounds);
            
            obj.visible = isVisible;
            obj.renderable = isVisible;
        }
    }
    
    private expandViewport(viewport: PIXI.Rectangle): PIXI.Rectangle {
        return new PIXI.Rectangle(
            viewport.x - this.cullMargin,
            viewport.y - this.cullMargin,
            viewport.width + this.cullMargin * 2,
            viewport.height + this.cullMargin * 2
        );
    }
}
```

#### Texture Atlas Management
```typescript
class EnvironmentAssets {
    private atlases: Map<string, PIXI.Spritesheet> = new Map();
    
    async loadEnvironmentAssets() {
        // Load structure atlas
        const structureAtlas = await PIXI.Assets.load('assets/environment/structures.json');
        this.atlases.set('structures', structureAtlas);
        
        // Load effects atlas
        const effectsAtlas = await PIXI.Assets.load('assets/environment/effects.json');
        this.atlases.set('effects', effectsAtlas);
        
        // Pre-warm texture cache
        this.preloadTextures();
    }
    
    getTexture(category: string, name: string): PIXI.Texture {
        const atlas = this.atlases.get(category);
        return atlas?.textures[name] || PIXI.Texture.EMPTY;
    }
}
```

## Networking Integration

### Environment Data Replication

#### Static vs Dynamic Elements
```rust
// Server-side environment replication
#[derive(Component)]
pub struct StaticEnvironment {
    // Replicated once at match start
    pub replicated: bool,
}

#[derive(Component)]
pub struct DynamicEnvironment {
    // Replicated when changes occur
    pub last_update: Instant,
    pub needs_replication: bool,
}

// Lightyear replication groups
pub fn setup_environment_replication(
    mut commands: Commands,
    static_query: Query<Entity, (With<EnvironmentObject>, With<StaticEnvironment>)>,
    dynamic_query: Query<Entity, (With<EnvironmentObject>, With<DynamicEnvironment>)>,
) {
    // Static elements replicated once
    for entity in static_query.iter() {
        commands.entity(entity).insert(Replicate {
            sync: SyncTarget::All,
            controlled_by: ControlledBy::Server,
            replicate_once: true,
        });
    }
    
    // Dynamic elements replicated on change
    for entity in dynamic_query.iter() {
        commands.entity(entity).insert(Replicate {
            sync: SyncTarget::All,
            controlled_by: ControlledBy::Server,
            replicate_once: false,
        });
    }
}
```

#### Interest Management
```rust
// Environment-specific interest management
pub fn environment_interest_management(
    mut query: Query<(&Transform, &mut Replicate), With<EnvironmentObject>>,
    player_query: Query<&Transform, (With<Player>, Without<EnvironmentObject>)>,
) {
    for (env_transform, mut replicate) in query.iter_mut() {
        let mut should_replicate = false;
        
        // Check if any player is near this environment object
        for player_transform in player_query.iter() {
            let distance = env_transform.translation.distance(player_transform.translation);
            if distance < ENVIRONMENT_REPLICATION_DISTANCE {
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

### Bandwidth Optimization

#### Compression Strategy
```rust
#[derive(Serialize, Deserialize)]
pub struct CompressedEnvironmentUpdate {
    pub entity_id: u32,
    pub position: CompressedVec2,      // 4 bytes instead of 8
    pub rotation: u8,                  // 1 byte instead of 4
    pub health_ratio: u8,              // 1 byte instead of 4
    pub state_flags: u8,               // Multiple booleans in 1 byte
}

impl From<EnvironmentObject> for CompressedEnvironmentUpdate {
    fn from(obj: EnvironmentObject) -> Self {
        // Compress environment data for network transmission
    }
}
```

## Implementation Phases

### Phase 1: MVP Static Environment (Week 1-2)
**Goal**: Basic non-interactive structures for tactical gameplay

**Features**:
- Static collision objects (towers, hull sections)
- Basic Pixi.js rendering
- Simple texture atlas
- R-tree spatial integration

**Deliverables**:
- 5-10 static structure types
- Basic collision detection
- Simple visual representation
- Performance baseline

### Phase 2: Destructible Elements (Week 3-4)
**Goal**: Interactive environment that responds to combat

**Features**:
- Destructible structures with health
- Destruction effects and debris
- Network replication of destruction
- Basic particle effects

**Deliverables**:
- Destructible structure system
- Destruction animations
- Debris physics
- Performance optimization

### Phase 3: Dynamic Environmental Effects (Week 5-6)
**Goal**: Atmospheric and hazardous elements

**Features**:
- Particle effects system
- Environmental hazards
- Zone shrinking integration
- Advanced visual effects

**Deliverables**:
- Particle system
- Hazard mechanics
- Zone interaction
- Visual polish

### Phase 4: Advanced Interactions (Week 7-8)
**Goal**: Complex environmental gameplay

**Features**:
- Boid nest interactions
- Environmental abilities
- Advanced destruction
- Emergent tactics

**Deliverables**:
- Boid-environment interaction
- Player-environment abilities
- Complex destruction chains
- Tactical depth

## Asset Pipeline Integration

### Environment Asset Organization
```
assets/
├── environment/
│   ├── structures/
│   │   ├── comm_tower.png
│   │   ├── weapon_turret.png
│   │   └── hull_section.png
│   ├── effects/
│   │   ├── sparks.png
│   │   ├── smoke.png
│   │   └── explosion.png
│   └── atlases/
│       ├── structures.json
│       ├── structures.png
│       ├── effects.json
│       └── effects.png
```

### Texture Atlas Requirements
- **Structure Atlas**: 2048x2048, all static elements
- **Effects Atlas**: 1024x1024, particle textures
- **Debris Atlas**: 512x512, destruction debris
- **Multiple Resolutions**: @2x, @1x, @0.5x for LOD

### Asset Creation Pipeline
1. **3D Modeling**: Create base structures in Blender
2. **Render to Sprites**: Multiple angles and damage states
3. **Texture Packing**: Use TexturePacker with Pixi.js preset
4. **Integration**: Import into game with consistent naming

## Performance Considerations

### Server Impact
- **CPU**: +15% for collision detection and destruction
- **Memory**: +50MB for environment data
- **Network**: +20% for environment replication
- **Mitigation**: Spatial partitioning, interest management

### Client Impact
- **GPU**: +30% for additional rendering layers
- **Memory**: +100MB for texture atlases
- **CPU**: +10% for particle systems
- **Mitigation**: LOD system, viewport culling, object pooling

### Network Bandwidth
- **Static Elements**: ~50KB per player (one-time)
- **Dynamic Updates**: ~5KB/s per player
- **Destruction Events**: ~1KB per event
- **Total Impact**: ~10% increase in bandwidth

## Testing Strategy

### Performance Benchmarks
- **10k boids + 100 structures**: Target 60 FPS
- **Network latency**: <50ms for destruction events
- **Memory usage**: <500MB total client memory
- **Bandwidth**: <100KB/s per player

### Stress Testing
- **Destruction cascades**: Multiple simultaneous destructions
- **Particle overflow**: Thousands of particles
- **Network congestion**: High destruction event rates
- **Memory pressure**: Long-running matches

## Risk Assessment

### Technical Risks
1. **Performance**: Environment may impact 10k boid performance
   - **Mitigation**: Extensive profiling and optimization
2. **Complexity**: Multi-layer rendering adds complexity
   - **Mitigation**: Gradual implementation, thorough testing
3. **Network**: Environment replication may increase bandwidth
   - **Mitigation**: Aggressive compression and interest management

### Gameplay Risks
1. **Visual Clutter**: Too much detail may hide important elements
   - **Mitigation**: Clear visual hierarchy, playtesting
2. **Balance**: Environment may favor certain playstyles
   - **Mitigation**: Balanced structure placement, multiple strategies
3. **Performance**: Client performance may vary significantly
   - **Mitigation**: Scalable graphics options, LOD system

## Success Metrics

### Technical
- **Frame Rate**: Maintain 60 FPS with full environment
- **Memory Usage**: <500MB total client memory
- **Network**: <10% bandwidth increase
- **Load Time**: <2 seconds additional for environment assets

### Gameplay
- **Engagement**: Environment features used in >80% of matches
- **Balance**: No single structure provides excessive advantage
- **Clarity**: <5% of players report visual confusion
- **Fun**: Positive feedback on tactical depth

## Conclusion

This implementation proposal provides a comprehensive roadmap for adding environmental complexity to Boid Wars while maintaining performance and gameplay clarity. The phased approach allows for iterative development and optimization, ensuring the environment enhances rather than detracts from the core bullet-hell experience.

The technical architecture leverages existing systems (Bevy ECS, Pixi.js, Lightyear) while adding new capabilities for dynamic, interactive environments. Performance considerations are addressed through LOD systems, spatial partitioning, and efficient networking.

The end result will be a rich, atmospheric battlefield that provides tactical depth without overwhelming the fast-paced gameplay that defines Boid Wars.
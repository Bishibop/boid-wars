# Pixi.js Integration Proposal

**Status**: Draft  
**Date**: 2025-01-16  
**Author**: Development Team  
**Target**: Iteration 0-1 (Tech Stack Validation & Core Gameplay)

## Executive Summary

This proposal outlines the plan for properly integrating Pixi.js as our primary rendering engine, building on the existing foundation where Pixi.js is installed and a basic GameClient exists. We'll focus on establishing clean architecture patterns, proper asset management, and a solid foundation for future scaling.

## Current State

### ✅ What's Already Done
- **Pixi.js v8.6.6**: Installed as dependency
- **GameClient Class**: Basic implementation with Pixi.js application
- **Entity Rendering**: Simple circles using PIXI.Graphics
- **Container Structure**: Separation of players, boids, and UI
- **Input Handling**: Mouse and keyboard input working
- **WASM Bridge**: Networking layer providing entity data as JSON

### 🟡 What Needs Improvement
- **No Real Sprites**: Using PIXI.Graphics instead of actual sprite textures
- **No Asset Loading**: No system for loading/managing game assets
- **Basic Rendering**: No animations, effects, or visual polish
- **Limited Entity Types**: Only players and boids, no projectiles or effects
- **No Visual Feedback**: Missing health bars, damage indicators, etc.

## Integration Goals

### Phase 1: Proper Asset System
- Set up asset loading pipeline
- Create basic sprite sheets for entities
- Implement texture management
- Add loading screen

### Phase 2: Enhanced Entity Rendering
- Replace Graphics with actual Sprites
- Add rotation and animation support
- Implement projectile rendering
- Add basic visual effects

### Phase 3: UI/HUD System
- Health bars and player info
- Score/stats display
- Game state indicators
- Debug overlay

### Phase 4: Visual Polish
- Particle effects for explosions
- Trail effects for projectiles
- Screen shake and feedback
- Background and environment

## Technical Architecture

### Asset Loading System

```typescript
// Asset manifest structure
interface AssetManifest {
  sprites: {
    player: string;
    boid: string;
    projectile: string;
  };
  effects: {
    explosion: string;
    trail: string;
  };
  ui: {
    healthBar: string;
    crosshair: string;
  };
}

// Asset loader service
class AssetLoader {
  async loadAssets(manifest: AssetManifest): Promise<void>;
  getTexture(name: string): PIXI.Texture;
  createSprite(textureName: string): PIXI.Sprite;
}
```

### Entity Rendering Architecture

```typescript
// Entity renderer interface
interface EntityRenderer {
  sprite: PIXI.Sprite;
  healthBar?: PIXI.Graphics;
  
  update(entity: Entity): void;
  setPosition(x: number, y: number): void;
  setRotation(angle: number): void;
  destroy(): void;
}

// Specific renderers
class PlayerRenderer implements EntityRenderer {
  constructor(texture: PIXI.Texture);
  showHealthBar(health: number, maxHealth: number): void;
}

class BoidRenderer implements EntityRenderer {
  constructor(texture: PIXI.Texture);
  setAggression(level: number): void; // Changes tint/appearance
}

class ProjectileRenderer implements EntityRenderer {
  constructor(texture: PIXI.Texture);
  addTrail(): void;
}
```

### Rendering System Updates

```typescript
class GameClient {
  private assetLoader: AssetLoader;
  private renderers: Map<string, EntityRenderer>;
  
  async initialize(): Promise<void> {
    // Load all game assets
    await this.assetLoader.loadAssets(ASSET_MANIFEST);
    
    // Initialize Pixi application
    await this.initPixi();
    
    // Set up rendering systems
    this.initRenderers();
  }
  
  private createEntityRenderer(type: EntityType, id: string): EntityRenderer {
    switch(type) {
      case EntityType.Player:
        return new PlayerRenderer(this.assetLoader.getTexture('player'));
      case EntityType.Boid:
        return new BoidRenderer(this.assetLoader.getTexture('boid'));
      case EntityType.Projectile:
        return new ProjectileRenderer(this.assetLoader.getTexture('projectile'));
    }
  }
}
```

## Implementation Plan

### Week 1: Asset Foundation

#### Day 1-2: Asset Creation
- Create basic sprite assets:
  - Player ship (32x32px)
  - Boid enemy (16x16px)
  - Projectile (8x8px)
  - UI elements
- Set up sprite sheet structure
- Create loading screen design

#### Day 3-4: Asset Loading System
- Implement AssetLoader class
- Add loading progress tracking
- Create sprite factory methods
- Handle loading errors gracefully

#### Day 5: Integration
- Update GameClient to use asset system
- Replace PIXI.Graphics with sprites
- Test asset loading pipeline
- Add fallback for missing assets

### Week 2: Entity System Enhancement

#### Day 1-2: Entity Renderers
- Implement PlayerRenderer with rotation
- Implement BoidRenderer with states
- Add ProjectileRenderer for bullets
- Create renderer factory system

#### Day 3-4: Visual Feedback
- Add health bars above players
- Implement damage flash effects
- Add death animations
- Create entity spawn effects

#### Day 5: Testing & Polish
- Test all entity types
- Optimize sprite rendering
- Fix visual bugs
- Document renderer API

### Week 3: UI/HUD Implementation

#### Day 1-2: HUD System
- Create HUD container layer
- Implement health/shield display
- Add score counter
- Create minimap (if needed)

#### Day 3-4: Game State UI
- Add connection status indicator
- Implement death/respawn screen
- Create pause menu
- Add FPS counter

#### Day 5: Integration
- Connect HUD to game state
- Test all UI elements
- Ensure UI scales properly
- Add UI animations

### Week 4: Visual Effects & Polish

#### Day 1-2: Particle System
- Implement basic particle emitter
- Create explosion effects
- Add projectile trails
- Implement thrust effects

#### Day 3-4: Environmental Effects
- Add starfield background
- Implement parallax scrolling
- Create zone boundary visuals
- Add ambient animations

#### Day 5: Final Polish
- Screen shake on explosions
- Hit feedback effects
- Sound integration prep
- Performance validation

## Asset Specifications

### Sprite Requirements

#### Player Ship
- **Size**: 32x32px
- **Format**: PNG with transparency
- **Variations**: Different ship types/colors
- **Animation**: Thrust effect frames

#### Boid Enemy
- **Size**: 16x16px
- **Format**: PNG with transparency
- **States**: Normal, Aggressive, Fleeing, Damaged
- **Animation**: Simple idle animation

#### Projectiles
- **Size**: 8x8px (bullets), 16x16px (missiles)
- **Format**: PNG with transparency
- **Variations**: Player bullets, enemy bullets
- **Effects**: Glow/trail texture

### UI Elements
- **Health Bar**: Scalable 9-slice sprite
- **Crosshair**: 32x32px with center alignment
- **HUD Frame**: Modular UI components
- **Icons**: 16x16px for various indicators

## Performance Considerations

### Baseline Performance
- Target: 60 FPS with current entity count
- No optimization needed yet (that comes later)
- Focus on clean architecture for future scaling

### Memory Management
- Load all assets upfront
- Reuse textures across sprites
- Simple sprite pooling for projectiles
- Clean up unused resources

## Integration with Existing Code

### Minimal Breaking Changes
- Keep existing GameClient interface
- Maintain WASM bridge communication
- Preserve input handling system
- Add asset loading as initialization step

### Progressive Enhancement
- Start with basic sprites
- Add effects incrementally
- Keep Graphics fallback for development
- Maintain hot reload capability

## Testing Plan

### Visual Testing
- All entity types render correctly
- Sprites rotate and position properly
- UI elements display at correct positions
- No visual artifacts or glitches

### Integration Testing
- Asset loading completes successfully
- Entities update from network data
- Input still controls player
- Performance remains stable

### Error Handling
- Missing asset fallbacks work
- Loading errors handled gracefully
- Sprite creation failures don't crash
- Memory cleanup on scene changes

## Success Criteria

### Iteration 0 Completion
- ✅ Sprites replace colored circles
- ✅ Asset loading system functional
- ✅ Basic visual feedback working
- ✅ No performance regression

### Foundation for Iteration 1
- ✅ Projectile system ready
- ✅ Multiple entity types supported
- ✅ Visual effects framework in place
- ✅ Clean architecture for scaling

## Next Steps

1. **Review and approve** this proposal
2. **Create initial sprite assets** (or source from asset packs)
3. **Implement asset loading system**
4. **Update GameClient progressively**
5. **Test with current game functionality**

This integration provides the visual foundation needed for Iteration 1's "Prove the Core" phase while maintaining the simple, working system you've already built.
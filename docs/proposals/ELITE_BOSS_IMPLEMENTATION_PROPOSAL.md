# Elite & Boss Enemy Implementation Proposal

## Overview

This document outlines the technical implementation plan for Elite and Boss enemies in Boid Wars. These special enemies will serve as high-stakes encounters that create dynamic battlefield changes, force player interaction, and provide escalating challenge throughout matches.

## Architecture Integration

### Alignment with Existing Systems
- **Server**: Rust/Bevy ECS for complex AI behaviors and state management
- **Client**: TypeScript/Pixi.js for enhanced visual effects and animations
- **Networking**: Lightyear 0.21 for high-priority entity replication
- **Physics**: Rapier 2D for advanced collision detection and abilities
- **Spatial**: R-tree integration for efficient influence radius queries

## Server-Side Implementation (Rust/Bevy ECS)

### Core Components

#### Elite Enemy Components
```rust
#[derive(Component)]
pub struct Elite {
    pub elite_type: EliteType,
    pub influence_radius: f32,
    pub ability_cooldown: Timer,
    pub spawn_cooldown: Timer,
    pub threat_level: u32,
    pub last_ability_use: Instant,
}

#[derive(Component)]
pub struct BoidCommander {
    pub controlled_boids: Vec<Entity>,
    pub command_range: f32,
    pub coordination_strength: f32,
    pub last_command: Instant,
    pub max_controlled: u32,
}

#[derive(Component)]
pub struct EliteAbility {
    pub ability_type: AbilityType,
    pub cooldown: Duration,
    pub cast_time: Duration,
    pub range: f32,
    pub energy_cost: f32,
}

#[derive(Component)]
pub struct Spawner {
    pub spawn_type: EntityType,
    pub spawn_rate: f32,
    pub max_spawns: u32,
    pub current_spawns: u32,
    pub spawn_timer: Timer,
}
```

#### Boss Enemy Components
```rust
#[derive(Component)]
pub struct Boss {
    pub boss_type: BossType,
    pub current_phase: u32,
    pub phase_health_thresholds: Vec<f32>,
    pub active_abilities: Vec<ActiveAbility>,
    pub enrage_timer: Option<Timer>,
    pub area_of_influence: f32,
}

#[derive(Component)]
pub struct BossPhase {
    pub phase_number: u32,
    pub abilities: Vec<AbilityType>,
    pub movement_pattern: MovementPattern,
    pub spawn_config: SpawnConfig,
    pub duration_limit: Option<Duration>,
}

#[derive(Component)]
pub struct MultiPhaseHealth {
    pub total_health: f32,
    pub phase_thresholds: Vec<f32>,
    pub phase_bonuses: Vec<PhaseBonus>,
    pub regeneration_rate: f32,
}

#[derive(Component)]
pub struct AreaOfEffect {
    pub effect_type: EffectType,
    pub radius: f32,
    pub damage_per_second: f32,
    pub duration: Duration,
    pub affected_entities: HashSet<Entity>,
}
```

### Elite Types Implementation

#### The Broodmother
```rust
#[derive(Component)]
pub struct Broodmother {
    pub spawn_queue: VecDeque<EntityType>,
    pub protective_shield: Shield,
    pub spawn_animation_timer: Timer,
    pub vulnerability_window: Duration,
}

pub fn broodmother_behavior_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Broodmother, &mut Spawner, &Transform)>,
    time: Res<Time>,
    boid_query: Query<&Transform, (With<Boid>, Without<Broodmother>)>,
) {
    for (entity, mut broodmother, mut spawner, transform) in query.iter_mut() {
        // Update spawn timer
        spawner.spawn_timer.tick(time.delta());
        
        // Check if vulnerable (during spawning)
        if broodmother.spawn_animation_timer.tick(time.delta()).just_finished() {
            broodmother.protective_shield.active = false;
        }
        
        // Spawn boids if conditions met
        if spawner.spawn_timer.finished() && spawner.current_spawns < spawner.max_spawns {
            let spawn_position = calculate_spawn_position(transform.translation);
            spawn_swarmer(&mut commands, spawn_position);
            spawner.current_spawns += 1;
            
            // Start vulnerability window
            broodmother.spawn_animation_timer.reset();
            broodmother.protective_shield.active = true;
        }
        
        // Call nearby boids for protection when damaged
        if broodmother.protective_shield.recently_damaged {
            call_boids_for_protection(&mut commands, entity, transform.translation, &boid_query);
            broodmother.protective_shield.recently_damaged = false;
        }
    }
}
```

#### The Stalker
```rust
#[derive(Component)]
pub struct Stalker {
    pub target_player: Option<Entity>,
    pub stealth_active: bool,
    pub stealth_cooldown: Timer,
    pub persistence_timer: Timer,
    pub call_for_backup_threshold: f32,
}

pub fn stalker_behavior_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Stalker, &mut Transform, &Health)>,
    player_query: Query<(Entity, &Transform), (With<Player>, Without<Stalker>)>,
    time: Res<Time>,
) {
    for (entity, mut stalker, mut transform, health) in query.iter_mut() {
        // Update timers
        stalker.stealth_cooldown.tick(time.delta());
        stalker.persistence_timer.tick(time.delta());
        
        // Choose target if none exists
        if stalker.target_player.is_none() {
            stalker.target_player = choose_stalker_target(&player_query, transform.translation);
        }
        
        // Pursue target
        if let Some(target_entity) = stalker.target_player {
            if let Ok((_, target_transform)) = player_query.get(target_entity) {
                let direction = (target_transform.translation - transform.translation).normalize();
                let stalker_speed = if stalker.stealth_active { 1.5 } else { 2.0 };
                
                transform.translation += direction * stalker_speed * time.delta_seconds();
                
                // Activate stealth when not attacking
                let distance_to_target = transform.translation.distance(target_transform.translation);
                if distance_to_target > 100.0 && stalker.stealth_cooldown.finished() {
                    stalker.stealth_active = true;
                } else if distance_to_target < 50.0 {
                    stalker.stealth_active = false;
                    stalker.stealth_cooldown.reset();
                }
            }
        }
        
        // Call for backup when damaged
        if health.current < stalker.call_for_backup_threshold {
            call_nearby_boids(&mut commands, transform.translation, 200.0);
        }
    }
}
```

#### The Coordinator
```rust
#[derive(Component)]
pub struct Coordinator {
    pub aura_radius: f32,
    pub enhancement_strength: f32,
    pub redirect_cooldown: Timer,
    pub affected_boids: HashSet<Entity>,
}

pub fn coordinator_behavior_system(
    mut commands: Commands,
    mut coordinator_query: Query<(Entity, &mut Coordinator, &Transform)>,
    mut boid_query: Query<(Entity, &mut Boid, &Transform), Without<Coordinator>>,
    player_query: Query<&Transform, With<Player>>,
) {
    for (coord_entity, mut coordinator, coord_transform) in coordinator_query.iter_mut() {
        coordinator.affected_boids.clear();
        
        // Find boids within aura
        for (boid_entity, mut boid, boid_transform) in boid_query.iter_mut() {
            let distance = coord_transform.translation.distance(boid_transform.translation);
            
            if distance <= coordinator.aura_radius {
                coordinator.affected_boids.insert(boid_entity);
                
                // Enhance boid capabilities
                boid.speed_multiplier = 1.0 + coordinator.enhancement_strength;
                boid.aggression_level = (boid.aggression_level * 1.5).min(1.0);
                
                // Redirect boids toward priority targets
                if coordinator.redirect_cooldown.finished() {
                    let target_player = find_priority_target(&player_query, coord_transform.translation);
                    if let Some(target_pos) = target_player {
                        boid.target_override = Some(target_pos);
                    }
                }
            }
        }
        
        // Update cooldowns
        coordinator.redirect_cooldown.tick(time.delta());
    }
}
```

### Boss Types Implementation

#### The Hive Mind
```rust
#[derive(Component)]
pub struct HiveMind {
    pub infection_zones: Vec<InfectionZone>,
    pub regeneration_timer: Timer,
    pub pulse_attack_cooldown: Timer,
    pub spawn_wave_config: SpawnWaveConfig,
}

#[derive(Clone)]
pub struct InfectionZone {
    pub position: Vec2,
    pub radius: f32,
    pub spawn_rate: f32,
    pub duration: Duration,
    pub created_at: Instant,
}

pub fn hive_mind_behavior_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut HiveMind, &mut Boss, &mut MultiPhaseHealth, &Transform)>,
    time: Res<Time>,
    player_query: Query<&Transform, With<Player>>,
) {
    for (entity, mut hive_mind, mut boss, mut health, transform) in query.iter_mut() {
        // Phase transitions based on health
        let health_ratio = health.total_health / health.phase_thresholds[0];
        let new_phase = calculate_boss_phase(health_ratio);
        
        if new_phase != boss.current_phase {
            transition_hive_mind_phase(&mut commands, entity, new_phase, &mut boss);
        }
        
        // Phase-specific behaviors
        match boss.current_phase {
            1 => {
                // Stationary spawning
                if hive_mind.spawn_wave_config.timer.tick(time.delta()).just_finished() {
                    spawn_boid_wave(&mut commands, transform.translation, &hive_mind.spawn_wave_config);
                }
            }
            2 => {
                // Begin moving, more aggressive spawning
                let target_position = find_strategic_position(&player_query, transform.translation);
                move_towards_position(&mut commands, entity, target_position, 50.0);
                
                // Increased spawn rate
                hive_mind.spawn_wave_config.spawn_count = (hive_mind.spawn_wave_config.spawn_count * 1.5) as u32;
            }
            3 => {
                // Desperate phase with pulse attacks
                if hive_mind.pulse_attack_cooldown.tick(time.delta()).just_finished() {
                    create_pulse_attack(&mut commands, transform.translation, 300.0, 25.0);
                }
                
                // Rapid spawning
                hive_mind.spawn_wave_config.spawn_rate *= 2.0;
            }
            _ => {}
        }
        
        // Regeneration if not damaged recently
        if hive_mind.regeneration_timer.tick(time.delta()).just_finished() {
            health.total_health = (health.total_health + 10.0).min(health.phase_thresholds[0]);
        }
        
        // Update infection zones
        update_infection_zones(&mut commands, &mut hive_mind.infection_zones, time.delta());
    }
}
```

### Spawn Management System

#### Dynamic Spawn Logic
```rust
#[derive(Resource)]
pub struct EliteSpawnManager {
    pub spawn_timer: Timer,
    pub max_elites: u32,
    pub current_elites: u32,
    pub spawn_weights: HashMap<EliteType, f32>,
    pub player_count_modifier: f32,
    pub zone_size_modifier: f32,
}

pub fn elite_spawn_system(
    mut commands: Commands,
    mut spawn_manager: ResMut<EliteSpawnManager>,
    time: Res<Time>,
    player_query: Query<&Transform, With<Player>>,
    elite_query: Query<&Elite>,
    zone_state: Res<ZoneState>,
) {
    // Update spawn timer
    spawn_manager.spawn_timer.tick(time.delta());
    
    // Update current elite count
    spawn_manager.current_elites = elite_query.iter().count() as u32;
    
    // Check if we should spawn an elite
    if spawn_manager.spawn_timer.finished() && 
       spawn_manager.current_elites < spawn_manager.max_elites {
        
        // Calculate spawn probability based on game state
        let player_count = player_query.iter().count();
        let zone_pressure = calculate_zone_pressure(&zone_state);
        let spawn_probability = calculate_spawn_probability(player_count, zone_pressure);
        
        if thread_rng().gen::<f32>() < spawn_probability {
            let elite_type = choose_elite_type(&spawn_manager.spawn_weights, player_count);
            let spawn_position = choose_spawn_position(&player_query, &zone_state);
            
            spawn_elite(&mut commands, elite_type, spawn_position);
        }
        
        spawn_manager.spawn_timer.reset();
    }
}

fn calculate_spawn_probability(player_count: usize, zone_pressure: f32) -> f32 {
    let base_probability = 0.3;
    let player_modifier = (8.0 - player_count as f32) / 8.0; // Higher with fewer players
    let zone_modifier = zone_pressure * 0.5; // Higher with smaller zone
    
    (base_probability + player_modifier * 0.4 + zone_modifier * 0.3).min(0.8)
}
```

#### Boss Spawn Conditions
```rust
#[derive(Resource)]
pub struct BossSpawnManager {
    pub boss_spawn_conditions: Vec<BossSpawnCondition>,
    pub active_boss: Option<Entity>,
    pub cooldown_timer: Timer,
}

#[derive(Clone)]
pub struct BossSpawnCondition {
    pub boss_type: BossType,
    pub min_players: u32,
    pub max_players: u32,
    pub zone_size_threshold: f32,
    pub elite_count_threshold: u32,
    pub probability: f32,
}

pub fn boss_spawn_system(
    mut commands: Commands,
    mut spawn_manager: ResMut<BossSpawnManager>,
    time: Res<Time>,
    player_query: Query<&Transform, With<Player>>,
    elite_query: Query<&Elite>,
    zone_state: Res<ZoneState>,
) {
    // Only spawn if no boss is active and cooldown is finished
    if spawn_manager.active_boss.is_none() && spawn_manager.cooldown_timer.tick(time.delta()).finished() {
        let player_count = player_query.iter().count() as u32;
        let elite_count = elite_query.iter().count() as u32;
        let zone_size = zone_state.current_radius;
        
        // Check each boss spawn condition
        for condition in &spawn_manager.boss_spawn_conditions {
            if player_count >= condition.min_players &&
               player_count <= condition.max_players &&
               zone_size <= condition.zone_size_threshold &&
               elite_count >= condition.elite_count_threshold {
                
                if thread_rng().gen::<f32>() < condition.probability {
                    let spawn_position = choose_boss_spawn_position(&player_query, &zone_state);
                    let boss_entity = spawn_boss(&mut commands, condition.boss_type, spawn_position);
                    spawn_manager.active_boss = Some(boss_entity);
                    break;
                }
            }
        }
    }
}
```

## Client-Side Implementation (TypeScript/Pixi.js)

### Enhanced Rendering System

#### Elite Visual Effects
```typescript
class EliteRenderer {
    private eliteContainer: PIXI.Container;
    private effectsContainer: PIXI.Container;
    private auraEffects: Map<number, PIXI.Graphics> = new Map();
    private eliteSprites: Map<number, EliteSprite> = new Map();
    
    constructor(app: PIXI.Application) {
        this.setupContainers(app);
        this.setupEliteAssets();
    }
    
    private setupContainers(app: PIXI.Application) {
        this.eliteContainer = new PIXI.Container();
        this.eliteContainer.zIndex = 10; // Above regular boids
        
        this.effectsContainer = new PIXI.Container();
        this.effectsContainer.zIndex = 11; // Above elites
        
        app.stage.addChild(this.eliteContainer, this.effectsContainer);
    }
    
    renderElite(elite: EliteData) {
        let sprite = this.eliteSprites.get(elite.id);
        
        if (!sprite) {
            sprite = this.createEliteSprite(elite);
            this.eliteSprites.set(elite.id, sprite);
            this.eliteContainer.addChild(sprite);
        }
        
        // Update position and rotation
        sprite.position.set(elite.position.x, elite.position.y);
        sprite.rotation = elite.rotation;
        
        // Update visual effects based on elite type
        this.updateEliteEffects(elite, sprite);
        
        // Update health bar
        this.updateEliteHealthBar(elite, sprite);
    }
    
    private createEliteSprite(elite: EliteData): EliteSprite {
        const texture = this.getEliteTexture(elite.type);
        const sprite = new EliteSprite(texture);
        
        // Scale based on elite type
        const scale = this.getEliteScale(elite.type);
        sprite.scale.set(scale);
        
        // Add glow effect
        const glowFilter = new PIXI.filters.GlowFilter({
            color: this.getEliteGlowColor(elite.type),
            outerStrength: 2,
            innerStrength: 1
        });
        sprite.filters = [glowFilter];
        
        // Add health bar
        sprite.healthBar = this.createHealthBar();
        sprite.addChild(sprite.healthBar);
        
        return sprite;
    }
    
    private updateEliteEffects(elite: EliteData, sprite: EliteSprite) {
        switch (elite.type) {
            case EliteType.Broodmother:
                this.updateBroodmotherEffects(elite, sprite);
                break;
            case EliteType.Stalker:
                this.updateStalkerEffects(elite, sprite);
                break;
            case EliteType.Coordinator:
                this.updateCoordinatorEffects(elite, sprite);
                break;
        }
    }
    
    private updateBroodmotherEffects(elite: EliteData, sprite: EliteSprite) {
        // Shield effect when active
        if (elite.shieldActive) {
            if (!sprite.shieldEffect) {
                sprite.shieldEffect = this.createShieldEffect();
                sprite.addChild(sprite.shieldEffect);
            }
            sprite.shieldEffect.visible = true;
        } else if (sprite.shieldEffect) {
            sprite.shieldEffect.visible = false;
        }
        
        // Spawn animation
        if (elite.isSpawning) {
            this.playSpawnAnimation(sprite);
        }
    }
    
    private updateCoordinatorEffects(elite: EliteData, sprite: EliteSprite) {
        // Aura effect
        let aura = this.auraEffects.get(elite.id);
        if (!aura) {
            aura = this.createAuraEffect(elite.influenceRadius);
            this.auraEffects.set(elite.id, aura);
            this.effectsContainer.addChild(aura);
        }
        
        aura.position.set(elite.position.x, elite.position.y);
        aura.alpha = 0.3 + Math.sin(performance.now() * 0.003) * 0.1;
    }
}
```

#### Boss Visual System
```typescript
class BossRenderer {
    private bossContainer: PIXI.Container;
    private bossSprite: BossSprite | null = null;
    private phaseEffects: Map<number, PIXI.Container> = new Map();
    private screenShake: ScreenShake;
    
    constructor(app: PIXI.Application) {
        this.bossContainer = new PIXI.Container();
        this.bossContainer.zIndex = 15; // Highest priority
        app.stage.addChild(this.bossContainer);
        
        this.screenShake = new ScreenShake(app.stage);
    }
    
    renderBoss(boss: BossData) {
        if (!this.bossSprite) {
            this.createBossSprite(boss);
            this.createBossUI(boss);
        }
        
        // Update position
        this.bossSprite.position.set(boss.position.x, boss.position.y);
        
        // Update phase effects
        this.updatePhaseEffects(boss);
        
        // Update health bar
        this.updateBossHealthBar(boss);
        
        // Screen shake for abilities
        if (boss.isUsingAbility) {
            this.screenShake.start(boss.abilityIntensity);
        }
    }
    
    private createBossSprite(boss: BossData) {
        const texture = this.getBossTexture(boss.type);
        this.bossSprite = new BossSprite(texture);
        
        // Much larger scale
        this.bossSprite.scale.set(3.0);
        
        // Complex filter effects
        const filters = [
            new PIXI.filters.GlowFilter({ color: 0xff0000, outerStrength: 4 }),
            new PIXI.filters.DropShadowFilter({ distance: 10, blur: 5 })
        ];
        this.bossSprite.filters = filters;
        
        this.bossContainer.addChild(this.bossSprite);
    }
    
    private updatePhaseEffects(boss: BossData) {
        const currentPhase = boss.currentPhase;
        
        // Remove old phase effects
        for (const [phase, container] of this.phaseEffects) {
            if (phase !== currentPhase) {
                container.visible = false;
            }
        }
        
        // Add/update current phase effects
        let phaseContainer = this.phaseEffects.get(currentPhase);
        if (!phaseContainer) {
            phaseContainer = this.createPhaseEffects(boss.type, currentPhase);
            this.phaseEffects.set(currentPhase, phaseContainer);
            this.bossContainer.addChild(phaseContainer);
        }
        
        phaseContainer.visible = true;
        this.animatePhaseEffects(phaseContainer, boss);
    }
    
    private createPhaseEffects(bossType: BossType, phase: number): PIXI.Container {
        const container = new PIXI.Container();
        
        switch (bossType) {
            case BossType.HiveMind:
                if (phase === 1) {
                    // Pulsing energy rings
                    const ring = this.createEnergyRing(100, 0x00ff00);
                    container.addChild(ring);
                } else if (phase === 2) {
                    // Movement trail
                    const trail = this.createMovementTrail();
                    container.addChild(trail);
                } else if (phase === 3) {
                    // Desperate energy burst
                    const burst = this.createEnergyBurst();
                    container.addChild(burst);
                }
                break;
        }
        
        return container;
    }
}
```

### Audio Integration

#### Elite Audio System
```typescript
class EliteAudioManager {
    private audioContext: AudioContext;
    private spatialAudio: Map<number, SpatialAudioSource> = new Map();
    private eliteSounds: Map<EliteType, Howl> = new Map();
    
    constructor() {
        this.setupEliteAudio();
    }
    
    private setupEliteAudio() {
        // Load elite-specific sounds
        this.eliteSounds.set(EliteType.Broodmother, new Howl({
            src: ['assets/audio/broodmother.webm'],
            loop: true,
            volume: 0.7,
            html5: true
        }));
        
        this.eliteSounds.set(EliteType.Stalker, new Howl({
            src: ['assets/audio/stalker.webm'],
            loop: false,
            volume: 0.8,
            html5: true
        }));
        
        this.eliteSounds.set(EliteType.Coordinator, new Howl({
            src: ['assets/audio/coordinator.webm'],
            loop: true,
            volume: 0.6,
            html5: true
        }));
    }
    
    playEliteSound(elite: EliteData, playerPosition: Vector2) {
        const sound = this.eliteSounds.get(elite.type);
        if (!sound) return;
        
        const id = sound.play();
        
        // Apply spatial audio
        const distance = Vector2.distance(elite.position, playerPosition);
        const volume = this.calculateVolume(distance);
        const pan = this.calculatePan(elite.position, playerPosition);
        
        sound.volume(volume, id);
        sound.stereo(pan, id);
        
        // Store for positional updates
        this.spatialAudio.set(elite.id, { sound, id, elite });
    }
    
    updateSpatialAudio(playerPosition: Vector2) {
        for (const [eliteId, audioSource] of this.spatialAudio) {
            const distance = Vector2.distance(audioSource.elite.position, playerPosition);
            const volume = this.calculateVolume(distance);
            const pan = this.calculatePan(audioSource.elite.position, playerPosition);
            
            audioSource.sound.volume(volume, audioSource.id);
            audioSource.sound.stereo(pan, audioSource.id);
        }
    }
    
    private calculateVolume(distance: number): number {
        const maxDistance = 500;
        return Math.max(0, 1 - (distance / maxDistance));
    }
    
    private calculatePan(sourcePos: Vector2, playerPos: Vector2): number {
        const deltaX = sourcePos.x - playerPos.x;
        const maxDistance = 300;
        return Math.max(-1, Math.min(1, deltaX / maxDistance));
    }
}
```

## Networking Integration

### High-Priority Replication

#### Elite/Boss Network Priority
```rust
// High-priority replication for elites and bosses
pub fn setup_elite_boss_replication(
    mut commands: Commands,
    elite_query: Query<Entity, (With<Elite>, Without<Replicate>)>,
    boss_query: Query<Entity, (With<Boss>, Without<Replicate>)>,
) {
    // Elites get high-priority replication
    for entity in elite_query.iter() {
        commands.entity(entity).insert(Replicate {
            sync: SyncTarget::All,
            controlled_by: ControlledBy::Server,
            replicate_once: false,
            replication_group: ReplicationGroup::HighPriority,
            send_frequency: SendFrequency::PerTick(2), // 30Hz instead of 20Hz
        });
    }
    
    // Bosses get maximum priority
    for entity in boss_query.iter() {
        commands.entity(entity).insert(Replicate {
            sync: SyncTarget::All,
            controlled_by: ControlledBy::Server,
            replicate_once: false,
            replication_group: ReplicationGroup::Critical,
            send_frequency: SendFrequency::PerTick(1), // 60Hz
        });
    }
}
```

#### Ability Event System
```rust
#[derive(Event, Serialize, Deserialize)]
pub struct EliteAbilityEvent {
    pub elite_id: Entity,
    pub ability_type: AbilityType,
    pub target_position: Vec2,
    pub duration: Duration,
    pub intensity: f32,
}

#[derive(Event, Serialize, Deserialize)]
pub struct BossPhaseChangeEvent {
    pub boss_id: Entity,
    pub old_phase: u32,
    pub new_phase: u32,
    pub transition_effects: Vec<EffectType>,
}

// Immediate event replication for important state changes
pub fn replicate_elite_events(
    mut ability_events: EventReader<EliteAbilityEvent>,
    mut phase_events: EventReader<BossPhaseChangeEvent>,
    mut client_writer: EventWriter<ToClientsEvent>,
) {
    // Replicate ability events immediately
    for event in ability_events.iter() {
        client_writer.send(ToClientsEvent {
            target: ClientTarget::All,
            event: ClientEvent::EliteAbility(event.clone()),
        });
    }
    
    // Replicate phase changes immediately
    for event in phase_events.iter() {
        client_writer.send(ToClientsEvent {
            target: ClientTarget::All,
            event: ClientEvent::BossPhaseChange(event.clone()),
        });
    }
}
```

### Bandwidth Optimization

#### Compressed Elite Data
```rust
#[derive(Serialize, Deserialize)]
pub struct CompressedEliteUpdate {
    pub entity_id: u32,
    pub position: CompressedVec2,
    pub rotation: u8,
    pub health_ratio: u8,
    pub ability_cooldown: u8,
    pub state_flags: u16, // Multiple state booleans packed
    pub influence_radius: u8, // Quantized radius
}

#[derive(Serialize, Deserialize)]
pub struct CompressedBossUpdate {
    pub entity_id: u32,
    pub position: CompressedVec2,
    pub rotation: u8,
    pub health_ratio: u8,
    pub current_phase: u8,
    pub ability_states: u32, // Bit flags for active abilities
    pub movement_target: Option<CompressedVec2>,
}

impl From<Elite> for CompressedEliteUpdate {
    fn from(elite: Elite) -> Self {
        // Compress elite data for network transmission
        // Quantize floating point values
        // Pack boolean flags into bit fields
        // Use lookup tables for enums
    }
}
```

## Implementation Phases

### Phase 1: Basic Elite System (Week 1-2)
**Goal**: Implement core elite functionality with basic AI

**Features**:
- Basic Broodmother and Stalker implementations
- Simple spawn system based on player count
- Basic visual differentiation (size, color)
- Server-authoritative AI behavior

**Deliverables**:
- 2 elite types with basic behaviors
- Spawn management system
- Basic network replication
- Simple visual effects

### Phase 2: Advanced Elite Abilities (Week 3-4)
**Goal**: Complex elite behaviors and interactions

**Features**:
- Coordinator with boid enhancement
- Advanced ability systems (stealth, shields, auras)
- Elite-to-elite interactions
- Enhanced visual and audio effects

**Deliverables**:
- 3rd elite type (Coordinator)
- Complex ability system
- Enhanced visual effects
- Spatial audio integration

### Phase 3: Boss Implementation (Week 5-6)
**Goal**: Single boss type with multi-phase mechanics

**Features**:
- Hive Mind boss with 3 phases
- Area-of-effect abilities
- Dynamic spawn patterns
- Phase transition effects

**Deliverables**:
- Complete boss system
- Multi-phase state machine
- Area-of-effect damage system
- Enhanced visual effects

### Phase 4: Advanced Boss Features (Week 7-8)
**Goal**: Additional boss types and complex interactions

**Features**:
- Swarm Lord and Artillery Platform bosses
- Boss-environment interactions
- Complex ability combinations
- Balancing and polish

**Deliverables**:
- 2 additional boss types
- Environment integration
- Ability combination system
- Performance optimization

## Balancing Framework

### Power Scaling Guidelines

#### Elite Balance
```rust
pub struct EliteBalanceConfig {
    pub base_health_multiplier: f32,
    pub base_damage_multiplier: f32,
    pub player_count_scaling: f32,
    pub zone_size_scaling: f32,
    pub ability_cooldown_base: Duration,
}

impl Default for EliteBalanceConfig {
    fn default() -> Self {
        Self {
            base_health_multiplier: 3.0,  // 3x normal boid health
            base_damage_multiplier: 2.0,  // 2x normal boid damage
            player_count_scaling: 0.5,    // 50% increase per remaining player
            zone_size_scaling: 1.0,       // 100% increase as zone shrinks
            ability_cooldown_base: Duration::from_secs(10),
        }
    }
}
```

#### Boss Balance
```rust
pub struct BossBalanceConfig {
    pub base_health_multiplier: f32,
    pub phase_health_ratios: Vec<f32>,
    pub ability_damage_scaling: f32,
    pub spawn_rate_scaling: f32,
    pub enrage_threshold: f32,
}

impl Default for BossBalanceConfig {
    fn default() -> Self {
        Self {
            base_health_multiplier: 25.0,  // 25x normal boid health
            phase_health_ratios: vec![1.0, 0.66, 0.33],  // Phase transitions
            ability_damage_scaling: 1.5,   // 150% damage increase per phase
            spawn_rate_scaling: 2.0,       // 200% spawn rate increase
            enrage_threshold: 0.1,         // Enrage at 10% health
        }
    }
}
```

### Spawn Rate Management

#### Dynamic Spawn Scaling
```rust
pub fn calculate_elite_spawn_rate(
    player_count: usize,
    zone_size: f32,
    elapsed_time: Duration,
) -> f32 {
    let base_rate = 0.1; // 10% chance per spawn check
    
    // Increase rate as players are eliminated
    let player_modifier = (8.0 - player_count as f32) / 8.0;
    
    // Increase rate as zone shrinks
    let zone_modifier = (1000.0 - zone_size) / 1000.0;
    
    // Increase rate over time
    let time_modifier = (elapsed_time.as_secs() as f32 / 300.0).min(1.0);
    
    base_rate * (1.0 + player_modifier + zone_modifier + time_modifier)
}
```

#### Population Limits
```rust
pub struct PopulationLimits {
    pub max_elites_per_player: f32,
    pub max_total_elites: u32,
    pub max_bosses: u32,
    pub elite_density_limit: f32, // Per area
}

impl Default for PopulationLimits {
    fn default() -> Self {
        Self {
            max_elites_per_player: 1.5,
            max_total_elites: 8,
            max_bosses: 1,
            elite_density_limit: 0.001, // 1 elite per 1000 sq units
        }
    }
}
```

## Performance Considerations

### Server Impact
- **CPU**: +25% for complex AI behaviors and ability calculations
- **Memory**: +75MB for elite/boss state management
- **Network**: +30% for high-priority replication
- **Mitigation**: Efficient state machines, ability pooling, priority queues

### Client Impact
- **GPU**: +40% for enhanced visual effects and larger sprites
- **Memory**: +150MB for elite/boss assets and effect systems
- **CPU**: +15% for complex animations and audio processing
- **Mitigation**: LOD system for effects, audio pooling, efficient animations

### Network Bandwidth
- **Elite Updates**: ~2KB/s per elite (high frequency)
- **Boss Updates**: ~5KB/s per boss (maximum frequency)
- **Ability Events**: ~500 bytes per event
- **Total Impact**: ~20% increase in bandwidth during elite/boss encounters

## Testing Strategy

### Performance Benchmarks
- **8 players + 3 elites + 1 boss + 5k boids**: Target 60 FPS
- **Network latency**: <30ms for ability events
- **Memory usage**: <750MB total client memory
- **Bandwidth**: <150KB/s per player during boss fights

### Stress Testing
- **Elite swarms**: Multiple elites spawning simultaneously
- **Boss phase transitions**: Rapid phase changes under load
- **Ability spam**: Multiple abilities triggering in sequence
- **Network congestion**: High-frequency elite updates

### Balance Testing
- **Elite effectiveness**: Kill/death ratios vs. players
- **Boss encounter duration**: Target 2-3 minutes per boss
- **Player engagement**: Percentage of players engaging elites
- **Survival rates**: Player survival with/without elites

## Risk Assessment

### Technical Risks
1. **AI Complexity**: Elite AI may be too CPU-intensive
   - **Mitigation**: Hierarchical AI, behavior trees, profiling
2. **Network Congestion**: High-priority replication may impact performance
   - **Mitigation**: Adaptive frequency, compression, priority queues
3. **Visual Overload**: Too many effects may cause frame drops
   - **Mitigation**: Effect LOD, performance scaling, occlusion culling

### Gameplay Risks
1. **Elite Dominance**: Elites may overshadow regular boids
   - **Mitigation**: Careful spawn rates, clear counterplay
2. **Boss Griefing**: Players may use bosses against other players
   - **Mitigation**: Boss aggro systems, balanced targeting
3. **Complexity Creep**: Too many elite types may confuse players
   - **Mitigation**: Clear visual language, gradual introduction

## Success Metrics

### Technical
- **Frame Rate**: Maintain 60 FPS with all elite/boss systems active
- **Memory Usage**: <750MB total client memory
- **Network**: <20% bandwidth increase during encounters
- **Load Time**: <3 seconds additional for elite/boss assets

### Gameplay
- **Engagement**: Elite encounters in >70% of matches
- **Balance**: No single elite type dominates encounter statistics
- **Clarity**: <10% of players report confusion about elite abilities
- **Satisfaction**: Positive feedback on elite/boss encounters

## Conclusion

This implementation proposal provides a comprehensive framework for adding Elite and Boss enemies to Boid Wars. The system is designed to create dynamic, high-stakes encounters that enhance the core gameplay loop while maintaining the fast-paced bullet-hell experience.

The technical architecture leverages existing systems while adding new capabilities for complex AI behaviors, enhanced visual effects, and high-priority networking. The phased implementation approach allows for iterative development and balancing, ensuring these special enemies enhance rather than overwhelm the core gameplay.

The result will be a more dynamic and engaging battlefield where elite and boss encounters create memorable moments and strategic depth, while maintaining the accessible, skill-based gameplay that defines Boid Wars.
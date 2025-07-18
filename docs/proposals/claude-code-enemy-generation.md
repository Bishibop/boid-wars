# Proposal: Claude Code Integration for Dynamic Enemy Type Creation

## Vision
Integrate Claude Code CLI into the Boid Wars development workflow to enable rapid creation, iteration, and refinement of new enemy types through natural language prompts, with real-time testing and hot-loading capabilities.

## Architecture Overview

### 1. Enemy Type System Foundation
```rust
// Core enemy type definition
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnemyType {
    pub name: String,
    pub config: BoidConfig,
    pub behavior: BehaviorDefinition,
    pub visual: VisualConfig,
    pub audio: AudioConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BehaviorDefinition {
    pub primary_state: BoidState,
    pub state_transitions: HashMap<BoidState, Vec<StateTransition>>,
    pub custom_behaviors: Vec<CustomBehavior>,
}
```

### 2. File-Based Hot-Loading System
```
enemy_types/
├── templates/           # Base templates for Claude
│   ├── aggressive.ron
│   ├── defensive.ron
│   └── support.ron
├── generated/          # Claude-generated types
│   ├── pack_hunter.ron
│   ├── void_stalker.ron
│   └── crystal_guardian.ron
└── active/             # Currently deployed types
    └── current_enemies.ron
```

### 3. Claude Code Integration Workflow

#### Phase 1: Template-Based Generation
**User Workflow:**
```bash
# Terminal 1: Game running with debug UI
make bevy-dev

# Terminal 2: Claude Code for enemy generation
claude-code
> "Create a new enemy type called 'Void Stalker' that hunts in small packs, 
  becomes invisible when not moving, and has high speed but low health. 
  Base it on the existing aggressive template but make it more tactical."
```

**Claude Output:**
- Generates `enemy_types/generated/void_stalker.ron`
- Includes behavioral parameters, state machine logic, and visual configs
- Provides documentation explaining design decisions

#### Phase 2: Live Testing Integration
**Hot-Loading System:**
1. File watcher detects new enemy type
2. Debug UI shows "New Enemy Type Available: Void Stalker"
3. One-click spawn for immediate testing
4. Real-time parameter tweaking through debug UI
5. Save refined version when satisfied

#### Phase 3: Advanced Code Generation
**Beyond Configs - Generate Actual Behavior Code:**
```rust
// Claude could generate custom behavior implementations
impl CustomBehavior for VoidStalkerBehavior {
    fn update(&mut self, entity: Entity, context: &BehaviorContext) -> BehaviorResult {
        // Generated tactical invisibility logic
        // Generated pack coordination behavior
        // Generated hit-and-run attack patterns
    }
}
```

## Implementation Phases

### Phase 1: Foundation (1-2 days)
1. **Enemy Type System:**
   - Extend `BoidConfig` to support enemy-specific parameters
   - Create `EnemyType` struct with serialization
   - Implement basic hot-loading for config files

2. **Debug UI Integration:**
   - Add enemy type selector dropdown
   - Implement spawn buttons for different types
   - Add "Load from File" functionality

### Phase 2: Claude Integration (1-2 days)
1. **Template System:**
   - Create base enemy templates with documentation
   - Define prompt formats for consistent generation
   - Add validation for generated configs

2. **File Watching:**
   - Implement `notify`-based file monitoring
   - Auto-refresh available enemy types
   - Add error handling for malformed files

### Phase 3: Advanced Features (3-5 days)
1. **Custom Behavior Generation:**
   - Define trait-based behavior system
   - Enable Claude to generate behavior implementations
   - Hot-compile and load custom behavior code

2. **Visual/Audio Integration:**
   - Generate sprite/particle configurations
   - Create audio profiles for enemy types
   - Integrate with existing asset pipeline

## Claude Code Prompting Strategy

### Prompt Templates
```
**Enemy Creation Prompt:**
"Create a [ENEMY_NAME] enemy type for Boid Wars with the following characteristics:
- Behavior: [BEHAVIOR_DESCRIPTION]
- Tactical Role: [ROLE] (assault/support/defensive/stealth)
- Pack Size: [SOLO/SMALL_GROUP/LARGE_SWARM]
- Movement Style: [MOVEMENT_DESCRIPTION]
- Special Abilities: [ABILITIES]

Base Configuration:
- Speed Range: [MIN-MAX]
- Aggression Level: [LOW/MEDIUM/HIGH]
- Health: [LOW/MEDIUM/HIGH]

Generate both the RON configuration file and explain the design rationale."
```

### Example Prompts
1. **"Crystal Guardian"** - Defensive enemy that protects resource nodes, slow but heavily armored
2. **"Void Stalker"** - Stealth assassin that hunts lone players, fast but fragile  
3. **"Swarm Queen"** - Support enemy that buffs nearby boids and spawns minions
4. **"Plasma Weaver"** - Artillery enemy that creates energy barriers and long-range attacks

## Benefits

### For Development
- **Rapid Prototyping:** Generate 10 enemy types in an hour vs. days of manual coding
- **Creative Exploration:** Claude can suggest unexpected combinations and behaviors
- **Documentation:** Auto-generated explanations for design decisions
- **Iteration Speed:** Immediate testing and refinement cycle

### For Gameplay
- **Enemy Variety:** Easily create dozens of unique enemy types
- **Behavioral Complexity:** More sophisticated AI through generated state machines
- **Balancing:** Quick parameter adjustments based on playtesting
- **Content Pipeline:** Non-programmers can create enemies through natural language

## Technical Considerations

### Performance
- Hot-loading limited to debug builds
- Config validation prevents crashes from malformed files
- Behavioral code compilation happens asynchronously

### Security
- Generated code runs in sandbox during development
- Validation pipeline for production builds
- Clear separation between configs and executable code

### Maintainability
- Generated files include metadata about creation process
- Version control integration for tracking changes
- Clear distinction between hand-written and generated content

## Future Extensions

### AI-Driven Balancing
- Claude analyzes gameplay metrics and suggests balance adjustments
- Automatic difficulty scaling based on player performance
- Meta-analysis of enemy effectiveness in different scenarios

### Procedural Campaigns
- Generate entire enemy factions with coherent themes
- Create boss encounters with unique mechanics
- Design multi-stage enemy evolution systems

This proposal transforms enemy creation from a manual coding process into a collaborative design conversation with Claude Code, dramatically accelerating content creation while maintaining code quality and performance standards.
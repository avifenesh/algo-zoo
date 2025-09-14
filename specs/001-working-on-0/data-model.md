# Data Model: Interactive Terminal Interface v0.2

## Core Entities

### ConfigurationState
**Purpose**: Represents current user selections for interactive configuration

**Fields**:
- `array_size: u32` - Number of elements to sort (10-1000)
- `distribution: DistributionType` - Data pattern selection
- `fairness_mode: FairnessMode` - Algorithm fairness strategy
- `budget: Option<u32>` - Budget parameter for comparison fairness
- `alpha: Option<f32>` - Alpha parameter for weighted fairness
- `beta: Option<f32>` - Beta parameter for weighted fairness
- `learning_rate: Option<f32>` - Learning rate for adaptive fairness

**Validation Rules**:
- Array size must be between 10 and 1000
- Budget must be > 0 when fairness_mode is Comparison
- Alpha/beta must be > 0.0 when fairness_mode is Weighted
- Learning rate must be between 0.1 and 1.0 when fairness_mode is Adaptive

**State Transitions**:
- Can only be modified in ConfigurationMode
- Changes are applied atomically when starting new race
- Previous configuration persists for reference

### InteractiveMode
**Purpose**: Tracks current UI interaction state

**Fields**:
- `current_mode: ApplicationMode`
- `config_focus: Option<ConfigurationField>`
- `array_view_algorithm: AlgorithmType`
- `help_visible: bool`

**States**:
```
ApplicationMode:
  - Configuration  // User is setting parameters
  - Racing         // Algorithms are actively running  
  - Paused         // Race paused, can view/configure
  - Complete       // Race finished, can reconfigure
  
ConfigurationField:
  - ArraySize
  - Distribution  
  - FairnessMode
  - BudgetParam
  - AlphaParam
  - BetaParam
  - LearningRateParam
```

**State Transitions**:
- Configuration → Racing: Space key pressed with valid config
- Racing → Paused: Space key pressed during race
- Paused → Racing: Space key pressed when paused
- Racing/Paused → Complete: All algorithms finish
- Complete → Configuration: Any configuration key pressed
- Any → Configuration: Configuration keys when not racing

### SessionState
**Purpose**: Manages lifecycle of multiple sorting races within single execution

**Fields**:
- `current_config: ConfigurationState`
- `run_history: Vec<RaceResult>`
- `session_start_time: std::time::Instant`
- `total_races_run: u32`

**Relationships**:
- Has one active ConfigurationState
- Contains history of completed RaceResult records
- Tracks session-level statistics

### DisplayMode  
**Purpose**: Controls which algorithm's array visualization is shown

**Fields**:
- `viewed_algorithm: AlgorithmType`
- `available_algorithms: Vec<AlgorithmType>`
- `cycle_index: usize`

**Behavior**:
- Cycles through available algorithms on 'v' key
- Persists selection across race resets
- Defaults to first algorithm (BubbleSort)

### MemoryMetrics
**Purpose**: Contains actual memory usage values for display

**Fields**:
- `algorithm_type: AlgorithmType`  
- `current_usage_bytes: usize`
- `peak_usage_bytes: usize`
- `last_updated: std::time::Instant`

**Validation Rules**:
- Memory values must be > 0 for active algorithms
- Peak usage >= current usage always
- Updates only when algorithms are active

## Entity Relationships

```
SessionState 1--1 ConfigurationState
SessionState 1--* RaceResult
SessionState 1--1 InteractiveMode
InteractiveMode 1--1 DisplayMode  
DisplayMode *--* AlgorithmInstance (via algorithm selection)
AlgorithmInstance 1--1 MemoryMetrics
```

## Data Flow

### Configuration Flow
1. User enters Configuration mode
2. ConfigurationState fields modified via keyboard input
3. Validation applied on each change
4. Visual feedback shows current selections
5. Space key applies configuration and starts race

### Race Execution Flow  
1. ConfigurationState applied to create algorithm instances
2. InteractiveMode switches to Racing
3. MemoryMetrics updated during algorithm execution
4. DisplayMode determines which algorithm's array is visualized
5. Race completes → InteractiveMode switches to Complete

### Session Management Flow
1. SessionState initialized with default ConfigurationState
2. Each race completion adds RaceResult to history
3. Configuration changes update current_config
4. Session statistics tracked across multiple races

## Validation & Constraints

### Cross-Entity Validations
- ConfigurationState parameters must be compatible with selected FairnessMode
- DisplayMode.viewed_algorithm must exist in current algorithm set
- MemoryMetrics must correspond to active algorithm instances

### Persistence Requirements
- ConfigurationState persists between races within session
- DisplayMode.viewed_algorithm persists across race resets  
- SessionState.run_history maintains race results for session duration
- No disk persistence required (in-memory only)

### Performance Constraints
- ConfigurationState changes must apply within 100ms
- MemoryMetrics updates must not impact algorithm performance
- DisplayMode switching must be instantaneous (no visual lag)

## Error States & Recovery

### Invalid Configuration Recovery
- Invalid parameter values revert to last valid state
- Error message displayed with valid range information
- User can retry configuration or use default values

### Algorithm Failure Recovery  
- Missing MemoryMetrics default to "N/A" display
- Array visualization falls back to first available algorithm
- Session continues with remaining functional algorithms

### State Consistency Guarantees
- ConfigurationState changes are atomic (all-or-nothing)
- Race state transitions are sequential (no intermediate states)
- MemoryMetrics updates are non-blocking and eventual consistency
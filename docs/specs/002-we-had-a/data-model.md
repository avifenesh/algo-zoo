# Data Model: Sorting Race Implementation

**Date**: 2025-01-13  
**Status**: EXTRACTED (from complete implementation)

## Core Entities

### Sorter (Trait)
**Purpose**: Defines interface for all sorting algorithms
**Fields**:
- `data: Vec<i32>` - Array being sorted
- `comparisons: u64` - Total comparisons made
- `moves: u64` - Total element moves/swaps
- `complete: bool` - Sorting completion status

**Methods**:
- `step(budget: usize) -> StepResult` - Execute with budget
- `reset(data: Vec<i32>)` - Initialize with new array
- `get_telemetry() -> Telemetry` - Current metrics
- `is_complete() -> bool` - Check if done
- `get_array() -> &[i32]` - Current array state

**State Transitions**:
- Initial → In Progress → Complete
- In Progress → Paused → In Progress (on space key)

### FairnessModel (Trait)
**Purpose**: Controls budget allocation between algorithms
**Fields**:
- `total_budget: usize` - Budget per tick

**Methods**:
- `allocate_budget(&[Box<dyn Sorter>]) -> Vec<usize>` - Distribute budget
- `name() -> &str` - Model identifier

**Implementations**:
1. ComparisonFairness - Equal budget to incomplete algorithms
2. WeightedFairness - Based on α*comps + β*moves
3. WallTimeFairness - Based on actual execution time
4. AdaptiveFairness - Based on progress rates

### Telemetry
**Purpose**: Metrics and visualization data
**Fields**:
- `total_comparisons: u64`
- `total_moves: u64`
- `memory_current: usize`
- `memory_peak: usize`
- `highlights: Vec<usize>` - Indices being compared
- `markers: Markers` - Visual indicators
- `status_text: String`
- `progress_hint: f32` - 0.0 to 1.0

### Markers
**Purpose**: Visual indicators for operations
**Fields**:
- `cursors: Vec<usize>` - Current positions
- `pivot: Option<usize>` - Quick sort pivot
- `gap: Option<usize>` - Shell sort gap
- `heap_boundary: Option<usize>` - Heap sort boundary
- `merge_boundaries: Vec<usize>` - Merge sort sections

### StepResult  
**Purpose**: Result of one algorithm step
**Fields**:
- `comparisons_used: usize`
- `moves_made: usize`
- `continued: bool` - More work remains

### RunConfiguration
**Purpose**: Application settings
**Fields**:
- `array_size: usize`
- `distribution: Distribution`
- `seed: u64`
- `fairness_mode: FairnessMode`
- `target_fps: u32`

### Distribution (Enum)
**Values**:
- Shuffled
- NearlySorted  
- Reversed
- FewUnique
- Sorted
- WithDuplicates

### VisualizationState
**Purpose**: Rendering state for TUI
**Fields**:
- `bar_heights: Vec<Vec<u32>>` - Per algorithm
- `sparklines: Vec<Vec<u64>>` - History data
- `colors: Vec<Color>` - Per element colors
- `frame_time: Duration`
- `paused: bool`

## Relationships

```
RunConfiguration → ArrayGenerator → Initial Array
                 ↓
         Multiple Sorters (reset with same array)
                 ↓
         FairnessModel allocates budgets
                 ↓
         Each Sorter.step(budget) → StepResult
                 ↓
         Telemetry → VisualizationState
                 ↓
         Terminal Renderer → User Display
```

## Validation Rules

1. **Budget Allocation**: Sum of allocations must equal total budget
2. **Array Integrity**: Element count must remain constant
3. **Progress Monotonicity**: Progress can only increase or stay same
4. **Completion State**: Once complete, must stay complete
5. **Memory Bounds**: Peak memory must be >= current memory

## State Machine

```
Application States:
┌──────────┐      ┌─────────┐      ┌──────────┐
│  Init    │ ───> │ Running │ <──> │  Paused  │
└──────────┘      └─────────┘      └──────────┘
                       │
                       ↓
                  ┌──────────┐
                  │ Complete │
                  └──────────┘

Algorithm States:
┌──────────┐      ┌────────────┐      ┌──────────┐
│ Waiting  │ ───> │ Processing │ ───> │ Complete │
└──────────┘      └────────────┘      └──────────┘
```

## Performance Considerations

- Telemetry updates: O(1) per step
- Budget allocation: O(n) where n = algorithm count
- Visualization update: O(array_size * algorithms)
- Memory: O(array_size * algorithms) for visualization buffers
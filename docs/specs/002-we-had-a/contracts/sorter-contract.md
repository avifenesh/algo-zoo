# Sorter Trait Contract

**Date**: 2025-01-13  
**Status**: EXTRACTED from implementation

## Interface Definition

```rust
pub trait Sorter {
    fn step(&mut self, budget: usize) -> StepResult;
    fn is_complete(&self) -> bool;
    fn get_telemetry(&self) -> Telemetry;
    fn reset(&mut self, data: Vec<i32>);
    fn name(&self) -> &str;
    fn get_array(&self) -> &[i32];
    fn get_memory_usage(&self) -> usize;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
```

## Contract Requirements

### step(budget: usize) -> StepResult
**Preconditions**:
- `budget > 0`
- Algorithm not yet complete OR `is_complete() == false`

**Postconditions**:
- Returns `StepResult` with actual resource usage
- `comparisons_used <= budget` (budget respected)
- Progress is monotonic (never decreases)
- Array elements preserve total count
- If `continued == false`, then `is_complete() == true`

**Behavior**:
- Must make progress toward completion
- Must respect budget limitations
- Must update internal telemetry counters
- Must be resumable across multiple calls

### is_complete() -> bool
**Postconditions**:
- Returns `true` if and only if array is correctly sorted
- Once `true`, must remain `true` until `reset()`

### get_telemetry() -> Telemetry
**Postconditions**:
- Counters (comparisons, moves) are monotonic
- Progress is between 0.0 and 1.0
- Memory usage is non-negative
- Status text describes current state

### reset(data: Vec<i32>)
**Preconditions**:
- `data` is non-empty

**Postconditions**:
- Algorithm state reset to initial
- `is_complete() == false` (unless trivial case)
- All telemetry counters reset to 0
- Internal array equals input data

### get_array() -> &[i32]
**Postconditions**:
- Returns current array state
- Array length unchanged from `reset()`
- If `is_complete() == true`, array must be sorted

## Contract Tests

### Correctness Test
```rust
fn test_sorting_correctness() {
    let mut sorter = create_sorter();
    let input = generate_test_array();
    sorter.reset(input.clone());
    
    while !sorter.is_complete() {
        sorter.step(1000); // Large budget
    }
    
    assert!(is_sorted(sorter.get_array()));
    assert_eq!(sorter.get_array().len(), input.len());
}
```

### Budget Respect Test  
```rust
fn test_budget_respect() {
    let mut sorter = create_sorter();
    sorter.reset(generate_test_array());
    
    let result = sorter.step(5);
    assert!(result.comparisons_used <= 5);
    assert!(result.moves_made <= 5);
}
```

### Progress Monotonicity Test
```rust
fn test_progress_monotonic() {
    let mut sorter = create_sorter();
    sorter.reset(generate_test_array());
    
    let mut prev_progress = 0.0;
    while !sorter.is_complete() {
        sorter.step(10);
        let current_progress = sorter.get_telemetry().progress_hint;
        assert!(current_progress >= prev_progress);
        prev_progress = current_progress;
    }
}
```

### Resumability Test
```rust
fn test_resumability() {
    let mut sorter1 = create_sorter();
    let mut sorter2 = create_sorter();
    let input = generate_test_array();
    
    // Run one continuously
    sorter1.reset(input.clone());
    while !sorter1.is_complete() {
        sorter1.step(1000);
    }
    
    // Run other with small budgets
    sorter2.reset(input.clone());
    while !sorter2.is_complete() {
        sorter2.step(1);
    }
    
    // Both should produce same result
    assert_eq!(sorter1.get_array(), sorter2.get_array());
}
```

## Implementation Requirements

All `Sorter` implementations must:

1. **Maintain State**: Track algorithm-specific state to enable resumability
2. **Respect Budgets**: Never exceed the provided step budget
3. **Preserve Array**: Never add/remove elements, only reorder
4. **Update Telemetry**: Keep accurate counts of operations performed
5. **Handle Edge Cases**: Support trivial arrays (size 0-2)
6. **Provide Progress**: Give meaningful completion estimates

## Validation Status

✅ **BubbleSort**: Passes all contract tests  
✅ **InsertionSort**: Passes all contract tests  
✅ **SelectionSort**: Passes all contract tests  
✅ **QuickSort**: Passes all contract tests (with incremental partitioning)  
✅ **HeapSort**: Passes all contract tests  
✅ **MergeSort**: Passes all contract tests  
✅ **ShellSort**: Passes all contract tests  

**Contract Status**: VALIDATED - All 7 algorithms comply with interface
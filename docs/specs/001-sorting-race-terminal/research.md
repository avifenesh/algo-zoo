# Research: Sorting Race Visualization

## Executive Summary
Research findings for implementing a terminal-based sorting algorithm race visualization in Rust using ratatui. Focus on performance optimization, fairness models, and smooth animation techniques.

## Key Decisions

### 1. TUI Framework Selection
**Decision**: ratatui with crossterm backend  
**Rationale**: 
- Most mature Rust TUI library with active maintenance
- Crossterm provides best cross-platform terminal support
- Built-in widget system includes sparklines and layout management
- Immediate mode rendering ideal for animation

**Alternatives Considered**:
- cursive: More complex, callback-based architecture
- termion: Less cross-platform support
- Direct ANSI: Too low-level for complex layouts

### 2. Animation & Performance Strategy
**Decision**: Fixed timestep with interpolation (30ms ticks)  
**Rationale**:
- Consistent visual pacing across different terminal speeds
- Allows deterministic replay with seeds
- Smooth perceived motion via frame interpolation

**Alternatives Considered**:
- Variable timestep: Non-deterministic, harder to debug
- Vsync-based: Terminal vsync unreliable across platforms
- Pure event-driven: Choppy animation

### 3. Data Structure for Visualization
**Decision**: Copy-on-write array slices per algorithm  
**Rationale**:
- Each algorithm maintains independent array copy
- Enables parallel visualization without sync issues
- Memory efficient with Rust's Arc/Rc for large arrays

**Alternatives Considered**:
- Shared mutable array: Complex synchronization
- Event sourcing: Excessive memory for long runs
- Direct mutation tracking: Hard to rewind/replay

### 4. Fairness Model Implementation
**Decision**: Budget-based stepping with pluggable strategies  
**Rationale**:
- Clean separation between algorithm and pacing
- Easy to add new fairness models
- Deterministic execution maintained

**Implementation Details**:
```rust
trait FairnessModel {
    fn allocate_budget(&self, algorithms: &[AlgoState]) -> Vec<usize>;
}

struct ComparisonBudget { k: usize }  // K comparisons per tick
struct WeightedBudget { alpha: f32, beta: f32 }  // α·comps + β·moves
struct WallTimeBudget { slice_ms: u64 }  // Time-based
```

### 5. Memory Tracking Strategy
**Decision**: Custom allocator wrapper for precise tracking  
**Rationale**:
- Accurate per-algorithm memory usage
- Minimal performance overhead
- Works with Rust's ownership model

**Implementation Approach**:
- Track allocations via wrapper types
- Count auxiliary space only (not input array)
- Report current and peak usage

### 6. Rendering Optimization
**Decision**: Differential rendering with dirty rectangles  
**Rationale**:
- Only redraw changed portions
- Reduces terminal bandwidth usage
- Maintains high FPS on slow connections

**Key Techniques**:
- Track changed regions per frame
- Batch terminal commands
- Use double buffering in ratatui

### 7. Determinism & Reproducibility
**Decision**: Seed-based PRNG for all randomness  
**Rationale**:
- Educational value in replay capability
- Easier debugging and testing
- Fair algorithm comparison

**Implementation**:
- Use rand::SeedableRng with displayed seed
- Deterministic array generation
- Same shuffling sequence on replay

## Performance Targets Analysis

### Frame Rate Considerations
- Target: 25-35 FPS (30-40ms per frame)
- Terminal limitations: Most terminals cap at ~60 FPS
- Human perception: 24 FPS sufficient for smooth motion
- Budget breakdown:
  - Algorithm stepping: 5-10ms
  - Rendering: 10-15ms
  - Terminal I/O: 10-15ms
  - Buffer: 5-10ms headroom

### Memory Constraints
- Base array: O(N) where N ≤ terminal width (~200)
- Per algorithm overhead:
  - Bubble/Insertion/Selection: O(1) extra
  - Shell: O(1) for gap sequence
  - Quick: O(log N) for iterative stack
  - Heap: O(1) for heap operations
  - Merge: O(N) for temp buffer
- Total: ~7 * N * sizeof(i32) + overheads < 10KB typical

### Scalability Limits
- Maximum algorithms: Limited by terminal height (~7-10 comfortable)
- Maximum array size: Terminal width (~80-200 typical)
- Snapshot frequency: Debounced to prevent disk flooding

## Testing Strategy

### Contract Tests
- Trait implementation verification
- Each sorter produces sorted output
- Comparison/move counts match theoretical

### Determinism Tests
```rust
#[test]
fn same_seed_same_sequence() {
    let run1 = simulate_with_seed(12345);
    let run2 = simulate_with_seed(12345);
    assert_eq!(run1.events, run2.events);
}
```

### Performance Tests
- Maintain FPS under load
- Memory usage within bounds
- No frame drops during operations

### Integration Tests
- Full run scenarios
- Keyboard interaction
- Snapshot save/load

## Implementation Patterns

### Sorter Trait Design
```rust
trait Sorter {
    fn step(&mut self, budget: usize) -> StepResult;
    fn is_complete(&self) -> bool;
    fn get_telemetry(&self) -> Telemetry;
    fn reset(&mut self, data: Vec<i32>);
}

struct StepResult {
    comparisons_used: usize,
    moves_made: usize,
    continued: bool,
}
```

### Controller Loop Pattern
```rust
loop {
    let start = Instant::now();
    
    // Collect input
    if event::poll(Duration::ZERO)? {
        handle_input(event::read()?);
    }
    
    // Step algorithms
    let budgets = fairness_model.allocate(algorithms);
    for (algo, budget) in algorithms.iter_mut().zip(budgets) {
        algo.step(budget);
    }
    
    // Render frame
    terminal.draw(|f| render(f, algorithms))?;
    
    // Maintain frame rate
    let elapsed = start.elapsed();
    if elapsed < target_frame_time {
        thread::sleep(target_frame_time - elapsed);
    }
}
```

## Risk Mitigation

### Terminal Compatibility
- Risk: Inconsistent ANSI support
- Mitigation: Use crossterm abstraction, fallback to basic mode

### Performance Variability
- Risk: Slow terminals affect animation
- Mitigation: Adaptive quality settings, frame skipping if needed

### Array Size Edge Cases
- Risk: Very small/large arrays break visualization
- Mitigation: Clamp to reasonable range, scale sparklines

## Next Steps (Phase 1)
1. Define Sorter trait contract
2. Create data models for state and metrics
3. Design snapshot JSON schema
4. Write failing contract tests
5. Document quickstart guide

---
*Research completed: All technical decisions resolved for implementation*
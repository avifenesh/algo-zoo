# Quickstart: Sorting Race Implementation

**Date**: 2025-01-13  
**Status**: VALIDATED (Implementation Complete)

## Prerequisites
- Rust 1.75+ with 2024 edition support
- Terminal with 256-color support
- Minimum terminal size: 80x24

## Installation

```bash
# Clone repository
git clone <repo-url>
cd algo-zoo

# Build release version
cargo build --release

# Run tests
cargo test
```

## Basic Usage

```bash
# Run with defaults (50 elements, comparison fairness k=16)
cargo run --release

# Custom array size and seed
cargo run --release -- --size 100 --seed 42

# Different fairness models
cargo run --release -- --fair weighted --alpha 2.0 --beta 0.5
cargo run --release -- --fair walltime
cargo run --release -- --fair adaptive

# Different distributions
cargo run --release -- --distribution nearly-sorted
cargo run --release -- --distribution reversed
```

## Keyboard Controls

- `Space` - Pause/Resume race
- `R` - Reset with same configuration
- `Q` - Quit application
- `↑/↓` - Navigate algorithm list (future)
- `←/→` - Change view mode (future)

## Validation Tests

### Test 1: Basic Functionality
```bash
# Start application
cargo run --release -- --size 20 --seed 100

# Expected:
# - 7 algorithms displayed (including Shell Sort)
# - Real-time bar chart visualization
# - Metrics updating (comparisons, moves)
# - Progress percentages increasing
# - All algorithms eventually complete
```

### Test 2: Fairness Models
```bash
# Comparison fairness
cargo run --release -- --fair comp --budget 8
# Verify: Each incomplete algorithm gets 8 comparisons per tick

# Weighted fairness  
cargo run --release -- --fair weighted --alpha 2.0 --beta 0.5
# Verify: Algorithms with fewer weighted operations get more budget

# Wall-time fairness
cargo run --release -- --fair walltime
# Verify: Each algorithm gets equal time slices
```

### Test 3: Quick Sort Budget Fix
```bash
# Test with array size > 17
cargo run --release -- --size 50 --fair comp --budget 16

# Expected:
# Quick Sort should make progress (not stuck at 0%)
# Uses incremental partitioning to work within budget
```

### Test 4: Visual Components
```bash
cargo run --release -- --size 30

# Expected visuals:
# ┌─ Algorithm Race ─────────────────┐
# │ Bubble Sort    [▓▓▓░░░░░] 45%   │
# │ Quick Sort     [▓▓▓▓▓▓▓▓] 98%   │
# │                                  │
# │ [Bar chart visualization here]   │
# │ [Sparkline graphs here]          │
# └──────────────────────────────────┘
```

### Test 5: Performance
```bash
# Large array test
cargo run --release -- --size 500

# Verify:
# - Smooth rendering (25-35 FPS)
# - No flickering
# - Memory usage < 50MB
# - Responsive to keyboard input
```

## Integration Test Script

```rust
// tests/integration/race_test.rs
#[test]
fn test_complete_race() {
    let config = RunConfiguration {
        array_size: 20,
        distribution: Distribution::Shuffled,
        seed: 42,
        fairness_mode: FairnessMode::ComparisonBudget { k: 16 },
        target_fps: 30,
    };
    
    let mut race = SortingRace::new(config);
    
    // Run until complete
    while !race.all_complete() {
        race.tick();
    }
    
    // Verify all sorted correctly
    for algo in race.algorithms() {
        assert!(is_sorted(algo.get_array()));
    }
}
```

## Troubleshooting

**Issue**: Quick Sort not progressing
- **Solution**: Verify incremental partitioning is implemented
- **Check**: Budget allocation is working correctly

**Issue**: Visual glitches or flickering
- **Solution**: Ensure double buffering is enabled
- **Check**: Terminal supports required escape codes

**Issue**: Heap Sort producing incorrect results
- **Solution**: Verify atomic heapify operations
- **Check**: State restoration on budget exhaustion

## Expected Output

When working correctly, you should see:
1. Multiple sorting algorithms racing side-by-side
2. Visual bars showing array elements
3. Colors indicating comparisons (blue) and swaps (red)
4. Sparkline graphs showing operation history
5. Real-time metrics for each algorithm
6. Smooth animations at target FPS
7. All algorithms producing correctly sorted arrays
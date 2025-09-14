# Quickstart: Sorting Race Visualization

## Installation

```bash
# Clone the repository
git clone <repository-url>
cd algo-zoo

# Build the project
cargo build --release

# Run with default settings
cargo run --release
```

## Basic Usage

### Default Race
Start a race with 6 algorithms on a shuffled array:
```bash
sorting-race
```

### Custom Configuration
```bash
# Set array size and distribution
sorting-race --size 100 --distribution reversed

# Use deterministic seed for reproducible runs
sorting-race --seed 12345

# Select fairness mode
sorting-race --fair comp --k 16
sorting-race --fair weighted --alpha 1.0 --beta 1.0
```

## Interactive Controls

| Key | Action |
|-----|--------|
| `q` | Quit application |
| `Space` | Pause/Resume race |
| `n` | New run (new seed) |
| `r` | Restart (same seed) |
| `d` | Cycle distribution |
| `+`/`-` | Increase/Decrease array size |
| `a` | Toggle algorithms on/off |
| `m` | Cycle memory display mode |
| `f` | Cycle fairness modes |
| `s` | Save snapshot to JSON |

## Understanding the Display

### Layout
```
┌─ Sorting Race ─────────────────────────────────────┐
│ Seed: 12345 | N: 100 | Dist: Shuffled | Fair: Comp │
├─────────────────────────────────────────────────────┤
│ Bubble Sort    [████░░░░░░] 40%                    │
│  Comps: 2450 | Moves: 1225 | Steps/s: 120 | Mem: 0 │
│  Status: Comparing elements 45 and 46              │
├─────────────────────────────────────────────────────┤
│ Quick Sort     [████████░░] 80%                    │
│  Comps: 620 | Moves: 310 | Steps/s: 450 | Mem: 128 │
│  Status: Partitioning range 10..25 (pivot=17)      │
└─────────────────────────────────────────────────────┘
```

### Metrics Explained
- **Comps**: Total comparison operations
- **Moves**: Total element moves (swap = 2 moves)
- **Steps/s**: Current throughput (rolling average)
- **Mem**: Auxiliary memory usage (bytes)
- **Progress**: Estimated completion percentage

### Visual Markers
- `^` - Pivot element (Quick Sort)
- `|` - Heap boundary (Heap Sort)
- `___` - Merge run underlines (Merge Sort)
- Highlighted bars show current comparison positions

## Verification Tests

### Test 1: Deterministic Execution
```bash
# Run twice with same seed
sorting-race --seed 999 --size 50
# Press 's' to save snapshot after 5 seconds
# Press 'q' to quit

sorting-race --seed 999 --size 50
# Press 's' to save snapshot after 5 seconds
# Compare snapshots - metrics should match exactly
```

### Test 2: Fairness Models
```bash
# Comparison budget (equal comparisons per tick)
sorting-race --fair comp --k 10

# Weighted (balanced comparisons and moves)
sorting-race --fair weighted --alpha 1.0 --beta 1.0

# Observe how different models affect race dynamics
```

### Test 3: Distribution Impact
```bash
# Nearly sorted (best case for Bubble, worst for Quick)
sorting-race --distribution nearly-sorted

# Reversed (worst case for most algorithms)
sorting-race --distribution reversed

# Few unique values (interesting for Quick Sort)
sorting-race --distribution few-unique
```

### Test 4: Interactive Controls
1. Start default race: `sorting-race`
2. Press `Space` to pause
3. Press `d` to cycle distribution
4. Press `r` to restart with new distribution
5. Press `+` to increase array size
6. Press `a` to toggle algorithms
7. Press `s` to save snapshot

## Performance Validation

### Frame Rate Test
```bash
# Monitor FPS counter in header
sorting-race --size 200

# Should maintain 25-35 FPS
# No visible stuttering during operations
```

### Memory Usage Test
```bash
# Run with memory monitoring
sorting-race --size 150

# Observe memory columns
# Verify Quick Sort stack grows logarithmically
# Verify Merge Sort buffer equals array size
```

### Snapshot Verification
```bash
# Save snapshot during run
sorting-race --seed 777
# Press 's' after 10 seconds

# Check snapshot file
ls -la ./runs/
cat ./runs/race-*.json | jq '.algorithms[].metrics'
```

## Troubleshooting

### Terminal Too Small
**Error**: "Terminal width insufficient for array size"
**Solution**: Resize terminal or reduce array size with `-`

### Performance Issues
**Symptom**: Low FPS, choppy animation
**Solutions**:
- Reduce array size
- Close other terminal applications
- Use release build: `cargo run --release`

### Snapshot Not Saving
**Error**: "Failed to save snapshot"
**Solutions**:
- Ensure `./runs/` directory exists
- Check disk space
- Verify write permissions

## Advanced Usage

### Algorithm Selection
```bash
# Disable specific algorithms
sorting-race --disable bubble,selection

# Run only O(n log n) algorithms
sorting-race --only quick,heap,merge
```

### Custom Fairness Calibration
```bash
# Auto-calibrate to system performance
sorting-race --fair weighted --calibrate

# Use cache-optimized preset
sorting-race --fair weighted --preset cache

# Use memory-optimized preset
sorting-race --fair weighted --preset memory
```

### Batch Analysis
```bash
# Run multiple seeds and save results
for seed in {1..10}; do
    sorting-race --seed $seed --auto-snapshot
done

# Analyze results
./analyze-runs.sh ./runs/
```

## Expected Behavior

### Correct Sorting
All algorithms should:
- Produce sorted arrays (ascending order)
- Show 100% progress when complete
- Stop making comparisons when done

### Fairness Validation
With comparison budget:
- All algorithms get equal comparison opportunities
- Faster algorithms complete sooner but fairly

With weighted fairness:
- Balance between comparison and move costs
- Algorithms with fewer moves may progress faster

### Determinism Check
Same seed should produce:
- Identical input arrays
- Same comparison sequences
- Matching final metrics
- Reproducible race outcomes

---
*Quick validation that implementation meets requirements*
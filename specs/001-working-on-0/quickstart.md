# Quickstart: Interactive Terminal Interface v0.2

## Quick Validation Steps

### 1. Launch Interactive Mode
```bash
# Start without any command line arguments
./target/release/sorting-race

# Expected: Configuration screen with parameter selection options
# Expected: Help text showing keyboard shortcuts (k, b, f, v, ?, Space, q)
```

### 2. Configure Array Size
```bash  
# In the application:
# Press 'k' → Array size menu appears
# Use arrow keys → Navigate through [10, 25, 50, 100, 200, 500, 1000]
# Press Enter → Confirm selection, return to main config screen
# Expected: Current array size displayed in UI
```

### 3. Configure Distribution  
```bash
# Press 'b' → Distribution menu appears
# Use arrow keys → Cycle through [Shuffled, Reversed, Nearly-Sorted, Few-Unique]
# Press Enter → Confirm selection
# Expected: Current distribution displayed in UI
```

### 4. Configure Fairness Mode
```bash
# Press 'f' → Fairness mode menu appears
# Use arrow keys → Cycle through [Comparison, Weighted, Wall-time, Adaptive]
# Press Enter → Confirm fairness mode
# If Comparison selected → Budget parameter input appears
# If Weighted selected → Alpha/Beta parameter inputs appear  
# If Adaptive selected → Learning rate parameter input appears
# Expected: Fairness mode and parameters displayed in UI
```

### 5. Start First Race
```bash
# Press Space → Race begins with configured parameters
# Expected: Application mode changes to "Racing"
# Expected: All 7 algorithms start sorting
# Expected: Memory usage shows actual byte values (not just names)
# Expected: Array visualization displays (defaults to first algorithm)
```

### 6. Test Array View Switching
```bash
# During race or after completion:
# Press 'v' → Array visualization switches to next algorithm
# Press 'v' repeatedly → Cycles through all 7 algorithms
# Expected: Bar chart updates to show different algorithm's array
# Expected: Current algorithm name highlighted/indicated
```

### 7. Test Memory Display Fix
```bash
# During race execution:
# Observe memory usage panel
# Expected: Shows actual values like "Bubble Sort: 1.2 KB", "Merge Sort: 2.4 KB"  
# Expected: NO longer shows just algorithm names without values
# Expected: Values update in real-time during sorting
```

### 8. Test Race Control
```bash
# During active race:
# Press Space → Race pauses
# Expected: Algorithm execution stops, progress preserved
# Press Space again → Race resumes
# Expected: Algorithms continue from paused state
```

### 9. Test Multi-Session Capability
```bash
# After first race completes:
# Press any config key ('k', 'b', or 'f') → Returns to configuration mode
# Change parameters (e.g., different array size, distribution)
# Press Space → New race starts with updated parameters
# Expected: New race runs without restarting application
# Expected: Different behavior based on new parameters
```

### 10. Test Help System
```bash
# At any time:
# Press '?' → Help overlay appears
# Expected: Shows all keyboard shortcuts and their functions
# Press '?' again → Help overlay disappears
```

## Expected Behavior Verification

### Memory Usage Display (Bug Fix)
- ✅ **Before**: Only showed "Bubble Sort", "Merge Sort", etc.  
- ✅ **After**: Shows "Bubble Sort: 856 B", "Merge Sort: 1.2 KB", etc.
- ✅ Values update during algorithm execution
- ✅ Shows "N/A" for algorithms that fail or haven't started

### Array Visualization (Enhancement)
- ✅ **Before**: Only showed bubble sort array state
- ✅ **After**: Can switch between any of the 7 algorithms using 'v' key
- ✅ Selection persists across race resets
- ✅ Visual indicator shows which algorithm is currently displayed

### Interactive Configuration (New Feature)
- ✅ No command-line arguments required for basic usage  
- ✅ All parameters configurable via keyboard shortcuts
- ✅ Real-time visual feedback during parameter selection
- ✅ Invalid configurations prevented with helpful error messages

### Multi-Session Support (New Feature)  
- ✅ Multiple races within single program execution
- ✅ Configuration changes apply to subsequent races
- ✅ No need to restart application between different test scenarios
- ✅ Session history maintained (race completion tracking)

## Error Cases to Test

### Invalid Configuration Attempts
```bash
# Try invalid operations:
# Press configuration keys during active race → Should be ignored
# Try to set array size > 1000 → Should limit to maximum
# Try negative fairness parameters → Should reject with error message
```

### Edge Cases
```bash
# Test rapid key presses → Should handle gracefully without crashes
# Test switching views during algorithm completion → Should work smoothly  
# Test help toggle during various modes → Should always work
```

## Performance Validation

### Response Times
- Configuration changes should feel instant (< 100ms)
- Array view switching should be immediate (< 50ms)
- Race start/pause should respond quickly (< 100ms)

### Resource Usage
- Memory usage should not grow significantly across multiple races
- CPU usage during configuration should be minimal
- No memory leaks after extended usage

## Compatibility Validation

### CLI Compatibility  
```bash
# Existing v0.1 usage should still work:
./sorting-race --size 100 --distribution reversed
./sorting-race --fair comp --budget 16

# Expected: Runs directly without interactive mode
# Expected: All existing CLI options function as before
```

### Terminal Compatibility
- Test on different terminal sizes (minimum 80x24)
- Test with different color support levels
- Verify Unicode characters render correctly

## Success Criteria

- ✅ All 10 quickstart steps complete without errors
- ✅ Memory usage shows actual values, not just names  
- ✅ Array visualization switches between all algorithms
- ✅ Interactive configuration works for all parameters
- ✅ Multiple races run in single session
- ✅ All existing v0.1 functionality preserved
- ✅ Performance requirements met
- ✅ Error cases handled gracefully
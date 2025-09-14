# Interactive Interface Contracts

## Input Event Contracts

### Configuration Key Events

#### Array Size Configuration (Key: 'k')
```rust
Input: KeyEvent { code: Char('k'), modifiers: NONE }
Precondition: current_mode != Racing
Postcondition: 
  - config_focus = Some(ArraySize)
  - UI shows array size selection menu
  - Arrow keys navigate size options [10, 25, 50, 100, 200, 500, 1000]
```

#### Distribution Configuration (Key: 'b')  
```rust
Input: KeyEvent { code: Char('b'), modifiers: NONE }
Precondition: current_mode != Racing
Postcondition:
  - config_focus = Some(Distribution)  
  - UI shows distribution options [Shuffled, Reversed, NearlySorted, FewUnique]
  - Arrow keys cycle through options
```

#### Fairness Configuration (Key: 'f')
```rust
Input: KeyEvent { code: Char('f'), modifiers: NONE }
Precondition: current_mode != Racing
Postcondition:
  - config_focus = Some(FairnessMode)
  - UI shows fairness options [Comparison, Weighted, Walltime, Adaptive]  
  - Arrow keys cycle through options
  - Sub-parameters appear based on selection
```

### Race Control Events

#### Start/Pause Race (Key: Space)
```rust
Input: KeyEvent { code: Char(' '), modifiers: NONE }
Preconditions & Postconditions:
  - Configuration mode → Racing mode (validates config first)
  - Racing mode → Paused mode  
  - Paused mode → Racing mode
  - Complete mode → Configuration mode (resets for new race)
```

#### Visualization Switching (Key: 'v')
```rust
Input: KeyEvent { code: Char('v'), modifiers: NONE }
Precondition: algorithms exist (any mode)
Postcondition:
  - viewed_algorithm cycles to next algorithm
  - Array visualization updates to show selected algorithm's data
  - Algorithm name highlighted in UI
```

### Navigation Events

#### Menu Navigation (Arrow Keys)
```rust
Input: KeyEvent { code: Up/Down/Left/Right, modifiers: NONE }
Precondition: config_focus = Some(field)
Postcondition:
  - Configuration value changes based on arrow direction
  - Visual feedback shows current selection
  - Invalid values are skipped/rejected
```

#### Selection Confirmation (Key: Enter)
```rust  
Input: KeyEvent { code: Enter, modifiers: NONE }
Precondition: config_focus = Some(field) && valid_selection
Postcondition:
  - Configuration value committed
  - config_focus = None (exit menu)
  - Return to main configuration view
```

#### Help Toggle (Key: '?')
```rust
Input: KeyEvent { code: Char('?'), modifiers: NONE }
Precondition: any mode
Postcondition:
  - help_visible = !help_visible
  - Help overlay shows/hides keyboard shortcuts
```

## State Transition Contracts

### Application Mode Transitions

#### Configuration → Racing
```rust
Input: Space key + valid ConfigurationState
Precondition:
  - current_mode = Configuration
  - All config parameters valid
  - No config_focus active
Postcondition:
  - current_mode = Racing
  - Algorithm instances created with current config
  - Race timer starts
  - Memory metrics tracking begins
```

#### Racing → Paused  
```rust
Input: Space key during active race
Precondition: current_mode = Racing
Postcondition:
  - current_mode = Paused
  - Algorithm execution suspended
  - Current progress preserved
  - Configuration keys become available
```

#### Complete → Configuration
```rust
Input: Any configuration key ('k', 'b', 'f')
Precondition: current_mode = Complete
Postcondition:
  - current_mode = Configuration
  - Previous race results added to history
  - Algorithm instances reset
  - Memory metrics cleared
```

## Memory Display Contracts

### Memory Value Retrieval
```rust
Function: get_memory_display_values()
Input: Vec<AlgorithmInstance>
Output: Vec<(AlgorithmType, MemoryDisplayValue)>

Contract:
  - For each active algorithm → call algorithm.get_memory_usage()
  - Convert bytes to human-readable format (B, KB, MB)
  - If algorithm inactive or error → display "N/A"
  - Values update in real-time during race execution
```

### Memory Display Format
```rust
Output Format: "{algorithm_name}: {value} {unit}"
Examples:
  - "Bubble Sort: 1.2 KB"
  - "Quick Sort: 856 B" 
  - "Merge Sort: N/A" (if error)

Validation:
  - Memory values must be non-negative
  - Units automatically chosen for readability
  - "N/A" only for legitimate error cases
```

## Array Visualization Contracts

### Algorithm View Switching
```rust
Function: switch_array_view(algorithm_type: AlgorithmType)
Input: Target algorithm to display
Precondition: algorithm exists in current race
Postcondition:
  - Bar chart data source changes to selected algorithm
  - get_array() called on selected algorithm instance
  - Visual updates reflect new algorithm's array state
  - Algorithm name highlighted in algorithm list
```

### Visualization Data Format
```rust
Input: algorithm.get_array() -> Vec<i32>
Output: BarChart component data

Contract:
  - Array values mapped to bar heights
  - Color coding based on value ranges
  - Updates reflect real-time algorithm progress
  - Handles arrays up to 1000 elements efficiently
```

## Error Handling Contracts

### Invalid Configuration Handling
```rust
Input: Invalid parameter value
Response:
  - Value rejected (no state change)
  - Error message displayed with valid range
  - Previous valid value retained
  - User can retry or cancel
```

### Algorithm Failure Handling  
```rust
Input: Algorithm execution error
Response:
  - Algorithm marked as failed
  - Memory display shows "Error" 
  - Array visualization switches to working algorithm
  - Race continues with remaining algorithms
```

### Input Event Overflow
```rust
Input: Rapid key presses
Response:  
  - Events queued with max buffer size
  - Excess events dropped (not processed)
  - Current operation completes before next
  - No UI corruption or invalid states
```

## Performance Contracts

### Response Time Guarantees
- Configuration changes: < 100ms from keypress to visual update
- Array view switching: < 50ms for arrays up to 1000 elements  
- Memory display updates: < 200ms during algorithm execution
- Mode transitions: < 100ms state change completion

### Resource Usage Limits
- Memory overhead for interactive features: < 1MB additional
- CPU impact during configuration: < 5% baseline increase
- Event processing: Handle up to 100 events/second
- No memory leaks across multiple race sessions
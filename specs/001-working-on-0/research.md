# Phase 0: Research & Analysis

## Technical Context Resolution

All technical context items have been resolved from existing codebase analysis.

## Key Research Areas

### 1. Interactive Terminal UI State Management

**Decision**: Use enum-based state machine with ratatui's event-driven architecture  
**Rationale**: 
- Ratatui provides robust input handling via crossterm events
- Finite state machine pattern fits well with configuration menu navigation
- Rust's enum pattern matching ensures exhaustive handling of all states

**Alternatives Considered**:
- Global mutable state with channels: More complex, harder to reason about
- Pure functional approach: Would require significant refactoring of existing UI code

### 2. Memory Usage Display Fix

**Decision**: Modify existing memory display component to call `get_memory_usage()` method  
**Rationale**:
- Root cause identified: UI component only displays algorithm names, not calling existing memory tracking
- Memory tracking infrastructure already exists in all sorting algorithm implementations
- Simple fix with minimal architectural changes

**Alternatives Considered**:
- Rewrite memory tracking system: Overkill, existing system works correctly
- Add separate memory display component: Would duplicate existing UI layout

### 3. Array Visualization Switching

**Decision**: Extend existing bar chart component with algorithm selection state  
**Rationale**:
- Current bar_chart.rs component only shows bubble sort data
- Other algorithms have accessible array state through get_array() method
- Need to track current "viewed algorithm" and switch data source

**Alternatives Considered**:
- Multiple simultaneous views: Would complicate UI layout, unclear user benefit  
- Separate visualization windows: Not suitable for terminal constraints

### 4. Interactive Configuration Architecture  

**Decision**: Modal configuration system with dedicated input modes  
**Rationale**:
- Configuration and racing are distinct user activities (configuration → start → observe)
- Modal approach prevents accidental parameter changes during active sorting
- Clear visual feedback for current selection state

**Alternatives Considered**:
- Live configuration during sorting: Risk of disrupting running algorithms
- Side panel configuration: Limited terminal real estate, harder to implement

### 5. Session State Management

**Decision**: Extend existing main loop with session lifecycle management  
**Rationale**:
- Current architecture already handles pause/resume, can be extended for reset/reconfigure
- Session state tracks configuration history and allows multiple runs
- Maintains existing CLI compatibility for single-run usage

**Alternatives Considered**:
- Separate session management service: Over-engineering for single-process app
- Complete architectural rewrite: High risk, unnecessary for requirements

## Implementation Strategy

### Phase 1 Priorities
1. **Configuration State Model**: Enum-based state machine for menu navigation
2. **Memory Display Component**: Fix existing memory usage display  
3. **Algorithm View Selection**: Extend bar chart with algorithm switching
4. **Interactive Input Handling**: Event routing for configuration keys
5. **Session Management**: Multi-run lifecycle support

### Testing Strategy
- Contract tests for new state transitions
- Integration tests for keyboard input handling  
- Visual regression tests for UI components (manual verification)
- Property-based tests for configuration validation

## Risk Mitigation

### Technical Risks
- **Ratatui version compatibility**: Using established 0.29.0, well-tested
- **Performance with interactive updates**: Minimal impact, state changes are infrequent
- **Terminal compatibility**: Existing crossterm handles cross-platform concerns

### User Experience Risks
- **Discoverability of new features**: Address with help screen showing keyboard shortcuts
- **Accidental configuration changes**: Prevent with modal state separation
- **Complex navigation**: Keep configuration menus simple, arrow key navigation

## Dependencies & Integration Points

### Modified Components
- `src/lib/visualization.rs` - Add algorithm view switching
- `src/lib/controller.rs` - Extend input handling for configuration keys  
- `src/models/config.rs` - Add session state and configuration entities
- `src/main.rs` - Extend main loop for multi-session support

### New Components
- `src/lib/interactive.rs` - Configuration menu system
- `src/models/session.rs` - Session lifecycle management
- `tests/integration/test_interactive.rs` - Interactive feature testing

### Integration Points
- Existing CLI argument parsing (preserve compatibility)
- Algorithm trait implementations (no changes required)
- Fairness model system (extend with interactive parameter setting)

## Success Criteria Met
- ✅ All Technical Context NEEDS CLARIFICATION items resolved
- ✅ Architecture decisions documented with rationale  
- ✅ Implementation strategy defined
- ✅ Risk assessment completed
- ✅ Integration approach planned
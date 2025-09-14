# Tasks: Interactive Terminal Interface & Bug Fixes (v0.2)

**Input**: Design documents from `/home/ubuntu/algo-zoo/specs/001-working-on-0/`
**Prerequisites**: plan.md ✅, research.md ✅, data-model.md ✅, contracts/ ✅

## Execution Flow (main)
```
1. Load plan.md from feature directory ✅
   → Tech stack: Rust 2024, ratatui 0.29.0, crossterm 0.29.0, clap 4.5
   → Structure: Single project (src/, tests/ at repository root)
2. Load design documents ✅:
   → data-model.md: 5 entities (ConfigurationState, InteractiveMode, SessionState, DisplayMode, MemoryMetrics)
   → contracts/: Interactive interface contracts (input events, state transitions, memory display)
   → quickstart.md: 10 validation scenarios
3. Generate tasks by category: Setup → Tests → Bug Fixes → Models → UI → Integration → Polish
4. Apply task rules: Different files = [P], Tests before implementation (TDD)
5. Number tasks sequentially (T001-T043)
6. SUCCESS: 43 tasks ready for execution
```

## Format: `[ID] [P?] Description`
- **[P]**: Can run in parallel (different files, no dependencies)
- All file paths are absolute from repository root

## Phase 3.1: Setup
- [ ] T001 Update Cargo.toml version to 0.2.0 and verify dependencies (ratatui 0.29.0, crossterm 0.29.0)
- [ ] T002 [P] Configure linting with clippy and rustfmt for Rust 2024 edition
- [ ] T003 [P] Create tests/interactive/ directory for new interactive feature tests

## Phase 3.2: Contract Tests First (TDD) ⚠️ MUST COMPLETE BEFORE 3.3
**CRITICAL: These tests MUST be written and MUST FAIL before ANY implementation**

### Input Event Contract Tests
- [ ] T004 [P] Contract test for array size configuration key 'k' in tests/interactive/test_array_size_config.rs
- [ ] T005 [P] Contract test for distribution configuration key 'b' in tests/interactive/test_distribution_config.rs  
- [ ] T006 [P] Contract test for fairness configuration key 'f' in tests/interactive/test_fairness_config.rs
- [ ] T007 [P] Contract test for visualization switching key 'v' in tests/interactive/test_visualization_switch.rs
- [ ] T008 [P] Contract test for race control space key in tests/interactive/test_race_control.rs
- [ ] T009 [P] Contract test for help toggle key '?' in tests/interactive/test_help_toggle.rs
- [ ] T010 [P] Contract test for arrow key navigation in tests/interactive/test_arrow_navigation.rs

### State Transition Contract Tests  
- [ ] T011 [P] Contract test for Configuration→Racing transition in tests/interactive/test_config_to_racing.rs
- [ ] T012 [P] Contract test for Racing→Paused transition in tests/interactive/test_racing_to_paused.rs
- [ ] T013 [P] Contract test for Complete→Configuration transition in tests/interactive/test_complete_to_config.rs

### Memory Display Contract Tests
- [ ] T014 [P] Contract test for memory value retrieval in tests/interactive/test_memory_display.rs
- [ ] T015 [P] Contract test for memory display format validation in tests/interactive/test_memory_format.rs

### Integration Test Scenarios (from quickstart.md)
- [ ] T016 [P] Integration test for complete interactive configuration flow in tests/integration/test_interactive_config_flow.rs
- [ ] T017 [P] Integration test for multi-session capability in tests/integration/test_multi_session.rs
- [ ] T018 [P] Integration test for CLI compatibility preservation in tests/integration/test_cli_compatibility.rs

## Phase 3.3: Bug Fixes (ONLY after tests are failing)

### Memory Display Bug Fix
- [ ] T019 Fix memory usage display to show actual values in src/lib/visualization.rs
- [ ] T020 Update memory metrics collection to call get_memory_usage() in src/lib/memory_graph.rs

### Array Visualization Bug Fix  
- [ ] T021 Fix array visualization to support algorithm switching in src/lib/bar_chart.rs
- [ ] T022 Add algorithm selection state to bar chart component in src/lib/bar_chart.rs

## Phase 3.4: Core Models (ONLY after tests are failing)

### Data Model Implementation
- [ ] T023 [P] ConfigurationState model with validation in src/models/configuration.rs
- [ ] T024 [P] InteractiveMode model with state machine in src/models/interactive_mode.rs  
- [ ] T025 [P] SessionState model with race history in src/models/session.rs
- [ ] T026 [P] DisplayMode model with algorithm cycling in src/models/display_mode.rs
- [ ] T027 [P] MemoryMetrics model with real-time updates in src/models/memory_metrics.rs

### Model Integration
- [ ] T028 Add model exports to src/models/mod.rs for new interactive models
- [ ] T029 Update existing Config struct to work with ConfigurationState in src/models/config.rs

## Phase 3.5: Interactive UI Components

### Configuration Menu System
- [ ] T030 Create interactive configuration menu component in src/lib/interactive.rs
- [ ] T031 Array size selection menu with arrow key navigation in src/lib/interactive.rs  
- [ ] T032 Distribution selection menu with options display in src/lib/interactive.rs
- [ ] T033 Fairness mode selection with parameter inputs in src/lib/interactive.rs

### Enhanced Visualization Components
- [ ] T034 Algorithm view switcher for bar chart in src/lib/visualization.rs
- [ ] T035 Help overlay component with keyboard shortcuts in src/lib/help_overlay.rs
- [ ] T036 Enhanced memory display with actual values in src/lib/memory_graph.rs

## Phase 3.6: Input Handling & Event Processing

### Keyboard Event Processing
- [ ] T037 Interactive input handler for configuration keys in src/lib/input.rs
- [ ] T038 State-aware event routing based on current mode in src/lib/controller.rs
- [ ] T039 Validation and error handling for invalid state transitions in src/lib/controller.rs

## Phase 3.7: Session Management Integration

### Multi-Session Support
- [ ] T040 Session lifecycle manager in main application loop in src/main.rs
- [ ] T041 Configuration persistence between races in src/lib/controller.rs
- [ ] T042 Race history tracking and session statistics in src/models/session.rs

## Phase 3.8: Polish & Validation

### Final Testing & Documentation
- [ ] T043 [P] Run quickstart validation scenarios and verify all 10 steps pass

## Dependencies

### Critical Path
1. **Setup** (T001-T003) → **Contract Tests** (T004-T018) → **Implementation** (T019-T042) → **Validation** (T043)
2. **Bug Fixes** (T019-T022) can start immediately after contract tests
3. **Models** (T023-T029) must complete before **UI Components** (T030-T036)
4. **UI Components** (T030-T036) must complete before **Input Handling** (T037-T039)
5. **Input Handling** (T037-T039) must complete before **Session Management** (T040-T042)

### Parallel Execution Opportunities
- **T004-T018**: All contract tests (different files)
- **T023-T027**: All model implementations (different files)  
- **T019-T020**: Memory bug fixes (different files)
- **T021-T022**: Array visualization fixes (same file - sequential)

## Parallel Execution Examples

### Launch Contract Tests Together (Phase 3.2)
```bash
# All contract tests can run in parallel - different files
Task: "Contract test for array size configuration key 'k' in tests/interactive/test_array_size_config.rs"
Task: "Contract test for distribution configuration key 'b' in tests/interactive/test_distribution_config.rs"  
Task: "Contract test for fairness configuration key 'f' in tests/interactive/test_fairness_config.rs"
Task: "Contract test for visualization switching key 'v' in tests/interactive/test_visualization_switch.rs"
Task: "Contract test for race control space key in tests/interactive/test_race_control.rs"
```

### Launch Model Implementation Together (Phase 3.4)
```bash
# All models can be implemented in parallel - different files
Task: "ConfigurationState model with validation in src/models/configuration.rs"
Task: "InteractiveMode model with state machine in src/models/interactive_mode.rs"
Task: "SessionState model with race history in src/models/session.rs"  
Task: "DisplayMode model with algorithm cycling in src/models/display_mode.rs"
Task: "MemoryMetrics model with real-time updates in src/models/memory_metrics.rs"
```

### Launch Bug Fixes Together (Phase 3.3)
```bash
# Memory and array visualization fixes - different files
Task: "Fix memory usage display to show actual values in src/lib/visualization.rs"
Task: "Update memory metrics collection to call get_memory_usage() in src/lib/memory_graph.rs"
# Note: T021-T022 both modify src/lib/bar_chart.rs so must be sequential
```

## Task Generation Rules Applied

1. **From Contracts**: 15 contract tests generated (T004-T018) - all marked [P]
2. **From Data Model**: 5 entity models generated (T023-T027) - all marked [P]  
3. **From User Stories**: 3 integration tests from quickstart scenarios (T016-T018) - all marked [P]
4. **Bug Fixes Prioritized**: Memory display and array visualization fixes come first (T019-T022)
5. **TDD Enforced**: All tests (T004-T018) must complete before implementation (T019+)

## Validation Checklist ✅

- [x] All contracts have corresponding tests (T004-T015 cover all interactive_interface.md contracts)
- [x] All entities have model tasks (T023-T027 cover all 5 entities from data-model.md)  
- [x] All tests come before implementation (Phase 3.2 before Phase 3.3+)
- [x] Parallel tasks truly independent (verified file paths don't overlap)
- [x] Each task specifies exact file path (all tasks include absolute paths)
- [x] No [P] task modifies same file (verified no conflicts in parallel tasks)

## Performance & Quality Targets

- **Response Time**: Configuration changes < 100ms, view switching < 50ms (validated in T043)
- **Memory Usage**: < 1MB additional overhead for interactive features
- **Compatibility**: Preserve all existing v0.1 CLI functionality (validated in T018)
- **Test Coverage**: Contract tests for all interactive features, integration tests for complete flows

## Success Criteria

Upon completion of all 43 tasks:
- ✅ Memory usage displays actual byte values, not just algorithm names (Bug Fix #1)
- ✅ Array visualization switches between all 7 algorithms, not just bubble sort (Bug Fix #2) 
- ✅ Interactive configuration via keyboard shortcuts (k, b, f) without CLI arguments
- ✅ Multi-session capability - multiple races in single execution
- ✅ All existing v0.1 functionality preserved and working
- ✅ Performance targets met (sub-100ms response times)
- ✅ All quickstart validation scenarios pass
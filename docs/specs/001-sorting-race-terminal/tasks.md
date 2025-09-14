# Tasks: Sorting Race Visualization

**Input**: Design documents from `/specs/001-sorting-race-terminal/`
**Prerequisites**: plan.md (required), research.md, data-model.md, contracts/

## Execution Flow (main)
```
1. Load plan.md from feature directory
   → Tech stack: Rust 1.75+, ratatui, crossterm, serde
   → Structure: Single project (src/, tests/)
2. Load optional design documents:
   → data-model.md: 7 entities (Algorithm, Metrics, RunConfiguration, etc.)
   → contracts/sorter_trait.rs: Sorter trait contract
   → research.md: Technical decisions on TUI, fairness, memory tracking
3. Generate tasks by category:
   → Setup: Rust project, dependencies, CI
   → Tests: Contract tests, integration scenarios
   → Core: Models, sorting algorithms, fairness models
   → Integration: TUI components, controller loop
   → Polish: Performance optimization, documentation
4. Apply task rules:
   → Different files = mark [P] for parallel
   → Tests before implementation (TDD)
5. Number tasks sequentially (T001-T039)
6. Return: SUCCESS (39 tasks ready for execution)
```

## Format: `[ID] [P?] Description`
- **[P]**: Can run in parallel (different files, no dependencies)
- Include exact file paths in descriptions

## Path Conventions
- **Single project**: `src/`, `tests/` at repository root
- All paths relative to repository root

## Phase 3.1: Setup
- [ ] T001 Create Rust project structure with cargo init --name sorting-race
- [ ] T002 Add dependencies to Cargo.toml: ratatui 0.26, crossterm 0.27, serde 1.0, serde_json 1.0, rand 0.8
- [ ] T003 [P] Configure rustfmt.toml and .clippy.toml for project standards
- [ ] T004 [P] Create directory structure: src/models/, src/services/, src/lib/, tests/contract/, tests/integration/
- [ ] T005 [P] Set up GitHub Actions CI workflow in .github/workflows/rust.yml

## Phase 3.2: Tests First (TDD) ⚠️ MUST COMPLETE BEFORE 3.3
**CRITICAL: These tests MUST be written and MUST FAIL before ANY implementation**

### Contract Tests
- [ ] T006 [P] Contract test for Sorter trait in tests/contract/test_sorter_trait.rs
- [ ] T007 [P] Contract test for FairnessModel trait in tests/contract/test_fairness_trait.rs
- [ ] T008 [P] Contract test for MemoryTracker trait in tests/contract/test_memory_tracker.rs

### Integration Tests
- [ ] T009 [P] Integration test for deterministic execution in tests/integration/test_determinism.rs
- [ ] T010 [P] Integration test for fairness model allocation in tests/integration/test_fairness.rs
- [ ] T011 [P] Integration test for pause/resume functionality in tests/integration/test_controls.rs
- [ ] T012 [P] Integration test for snapshot save/load in tests/integration/test_snapshot.rs

## Phase 3.3: Core Implementation (ONLY after tests are failing)

### Models and Traits
- [ ] T013 [P] Implement core traits (Sorter, FairnessModel, MemoryTracker) in src/models/traits.rs
- [ ] T014 [P] Implement Algorithm state and transitions in src/models/algorithm.rs
- [ ] T015 [P] Implement Metrics tracking in src/models/metrics.rs
- [ ] T016 [P] Implement RunConfiguration in src/models/config.rs
- [ ] T017 [P] Implement Markers for visual indicators in src/models/markers.rs

### Sorting Algorithms
- [ ] T018 [P] Implement Bubble Sort in src/services/sorters/bubble.rs
- [ ] T019 [P] Implement Insertion Sort in src/services/sorters/insertion.rs
- [ ] T020 [P] Implement Selection Sort in src/services/sorters/selection.rs
- [ ] T021 [P] Implement Quick Sort (Hoare, iterative) in src/services/sorters/quick.rs
- [ ] T022 [P] Implement Heap Sort in src/services/sorters/heap.rs
- [ ] T023 [P] Implement Merge Sort (bottom-up) in src/services/sorters/merge.rs
- [ ] T024 [P] Implement Shell Sort (Ciura gaps, optional) in src/services/sorters/shell.rs

### Fairness Models
- [ ] T025 [P] Implement ComparisonBudget fairness in src/services/fairness/comparison.rs
- [ ] T026 [P] Implement WeightedFairness in src/services/fairness/weighted.rs
- [ ] T027 [P] Implement WallTimeFairness in src/services/fairness/walltime.rs

### Core Services
- [ ] T028 [P] Implement array generation (distributions) in src/services/generator.rs
- [ ] T029 [P] Implement snapshot serialization in src/services/snapshot.rs
- [ ] T030 [P] Implement memory tracking service in src/services/memory.rs

## Phase 3.4: Integration

### TUI Components
- [ ] T031 Implement main controller loop in src/lib/controller.rs
- [ ] T032 Implement TUI rendering engine in src/lib/visualization.rs
- [ ] T033 Implement sparkline generation for arrays in src/lib/sparkline.rs
- [ ] T034 Implement keyboard input handling in src/lib/input.rs

### CLI Application
- [ ] T035 Implement CLI argument parsing in src/cli/main.rs
- [ ] T036 Wire up all components in main application entry point

## Phase 3.5: Polish
- [ ] T037 [P] Performance optimization: ensure 25-35 FPS target
- [ ] T038 [P] Add comprehensive documentation to all public APIs
- [ ] T039 Run quickstart.md validation tests and fix any issues

## Dependencies
- Setup (T001-T005) must complete first
- Tests (T006-T012) before any implementation
- Models/Traits (T013-T017) before algorithm implementations
- Algorithms (T018-T024) can run in parallel
- Fairness models (T025-T027) can run in parallel
- TUI components (T031-T034) must be sequential
- CLI (T035-T036) depends on all other components
- Polish (T037-T039) after everything else

## Parallel Execution Examples

### Launch contract tests together:
```
Task: "Contract test for Sorter trait in tests/contract/test_sorter_trait.rs"
Task: "Contract test for FairnessModel trait in tests/contract/test_fairness_trait.rs"
Task: "Contract test for MemoryTracker trait in tests/contract/test_memory_tracker.rs"
```

### Launch all sorting algorithm implementations:
```
Task: "Implement Bubble Sort in src/services/sorters/bubble.rs"
Task: "Implement Insertion Sort in src/services/sorters/insertion.rs"
Task: "Implement Selection Sort in src/services/sorters/selection.rs"
Task: "Implement Quick Sort (Hoare, iterative) in src/services/sorters/quick.rs"
Task: "Implement Heap Sort in src/services/sorters/heap.rs"
Task: "Implement Merge Sort (bottom-up) in src/services/sorters/merge.rs"
```

### Launch model implementations:
```
Task: "Implement Algorithm state and transitions in src/models/algorithm.rs"
Task: "Implement Metrics tracking in src/models/metrics.rs"
Task: "Implement RunConfiguration in src/models/config.rs"
Task: "Implement Markers for visual indicators in src/models/markers.rs"
```

## Notes
- All sorting algorithms implement the Sorter trait from T013
- Memory tracking must be precise for Quick (stack) and Merge (buffer)
- TUI components depend on each other and must be done sequentially
- Performance target of 25-35 FPS is critical
- Deterministic execution with seeds is mandatory
- Tests MUST fail before implementation (TDD)

## Task Generation Rules
*Applied during main() execution*

1. **From Contracts**:
   - sorter_trait.rs → T006 (contract test)
   - Sorter trait → T018-T024 (implementations)
   
2. **From Data Model**:
   - Algorithm entity → T014
   - Metrics entity → T015
   - RunConfiguration → T016
   - Markers → T017
   
3. **From User Stories**:
   - Deterministic replay → T009
   - Pause/resume → T011
   - Snapshot saving → T012

4. **Ordering**:
   - Setup → Tests → Models → Services → Integration → Polish
   - Parallel tasks have no shared files

## Validation Checklist
*GATE: Checked by main() before returning*

- [x] All contracts have corresponding tests (T006-T008)
- [x] All entities have model tasks (T014-T017)
- [x] All tests come before implementation
- [x] Parallel tasks truly independent (different files)
- [x] Each task specifies exact file path
- [x] No task modifies same file as another [P] task
- [x] All 6 required algorithms have tasks (T018-T023)
- [x] All 3 fairness models have tasks (T025-T027)

---
*39 tasks generated from design documents - ready for execution*
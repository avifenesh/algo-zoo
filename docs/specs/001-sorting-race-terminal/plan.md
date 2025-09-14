# Implementation Plan: Sorting Race Visualization

**Branch**: `001-sorting-race-terminal` | **Date**: 2025-09-13 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/001-sorting-race-terminal/spec.md`

## Execution Flow (/plan command scope)
```
1. Load feature spec from Input path
   → Feature spec loaded successfully
2. Fill Technical Context (scan for NEEDS CLARIFICATION)
   → Detected: Terminal-based visualization application
   → Set Structure Decision: Option 1 (single project) for CLI tool
3. Evaluate Constitution Check section below
   → Single library approach appropriate for terminal visualization
   → Update Progress Tracking: Initial Constitution Check
4. Execute Phase 0 → research.md
   → Research best practices for TUI, sorting visualization, performance
5. Execute Phase 1 → contracts, data-model.md, quickstart.md, CLAUDE.md
6. Re-evaluate Constitution Check section
   → Design maintains simplicity
   → Update Progress Tracking: Post-Design Constitution Check
7. Plan Phase 2 → Task generation approach defined
8. STOP - Ready for /tasks command
```

## Summary
A terminal-based sorting algorithm race visualization tool that displays 6-7 classic sorting algorithms running side-by-side with real-time metrics, fairness models, and interactive controls. Built as a Rust CLI application using ratatui for TUI rendering, implementing O(n²) and O(n log n) sorting algorithms with configurable fairness models to ensure visual comparability.

## Technical Context
**Language/Version**: Rust 1.75+  
**Primary Dependencies**: ratatui (TUI), crossterm (terminal control), serde (serialization)  
**Storage**: JSON files for snapshots in ./runs/  
**Testing**: cargo test with unit and integration tests  
**Target Platform**: Linux/macOS/Windows terminals supporting ANSI  
**Project Type**: single (CLI terminal application)  
**Performance Goals**: 25-35 FPS rendering, smooth animation at terminal refresh rates  
**Constraints**: <100MB memory overhead, deterministic execution with seeds, terminal width adaptive  
**Scale/Scope**: 6-7 algorithms, arrays up to terminal width (~200 elements), real-time visualization

## Constitution Check
*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

**Simplicity**:
- Projects: 1 (CLI application)
- Using framework directly? Yes (ratatui without wrappers)
- Single data model? Yes (shared array, algorithm states)
- Avoiding patterns? Yes (direct implementation, no unnecessary abstractions)

**Architecture**:
- EVERY feature as library? Yes - sorting algorithms as trait implementations
- Libraries listed: 
  - sorting_algorithms: Core sorting implementations with Sorter trait
  - fairness_models: Step budget management and fairness calculations
  - metrics_tracking: Comparison/move counting and memory tracking
  - visualization: TUI rendering and sparkline generation
- CLI per library: sorting-race --help/--version/--format/--seed/--fair
- Library docs: llms.txt format planned? Yes

**Testing (NON-NEGOTIABLE)**:
- RED-GREEN-Refactor cycle enforced? Yes
- Git commits show tests before implementation? Yes
- Order: Contract→Integration→E2E→Unit strictly followed? Yes
- Real dependencies used? Terminal emulation for integration tests
- Integration tests for: algorithm correctness, fairness models, determinism
- FORBIDDEN: Implementation before test - understood

**Observability**:
- Structured logging included? Yes (to file during runs)
- Frontend logs → backend? N/A (single process)
- Error context sufficient? Yes (detailed error messages)

**Versioning**:
- Version number assigned? 0.1.0 (initial)
- BUILD increments on every change? Yes
- Breaking changes handled? N/A (initial version)

## Project Structure

### Documentation (this feature)
```
specs/001-sorting-race-terminal/
├── plan.md              # This file (/plan command output)
├── research.md          # Phase 0 output (/plan command)
├── data-model.md        # Phase 1 output (/plan command)
├── quickstart.md        # Phase 1 output (/plan command)
├── contracts/           # Phase 1 output (/plan command)
└── tasks.md             # Phase 2 output (/tasks command - NOT created by /plan)
```

### Source Code (repository root)
```
# Option 1: Single project (DEFAULT)
src/
├── models/
│   ├── algorithm.rs     # Algorithm trait and state
│   ├── metrics.rs       # Metrics tracking structures
│   └── config.rs        # Run configuration
├── services/
│   ├── sorters/         # Individual sorting algorithm implementations
│   ├── fairness.rs      # Fairness model implementations
│   └── snapshot.rs      # Snapshot saving logic
├── cli/
│   └── main.rs          # CLI entry point and argument parsing
└── lib/
    ├── visualization.rs # TUI rendering logic
    └── controller.rs    # Main control loop

tests/
├── contract/
│   └── sorter_trait.rs  # Trait contract tests
├── integration/
│   ├── determinism.rs   # Seed determinism tests
│   └── fairness.rs      # Fairness model tests
└── unit/
    └── metrics.rs       # Metric counting tests
```

**Structure Decision**: Option 1 - Single project structure for CLI terminal application

## Phase 0: Outline & Research
1. **Extract unknowns from Technical Context**:
   - Best practices for smooth TUI animation in Rust
   - Optimal data structures for sorting visualization
   - Performance profiling for terminal rendering
   - Fairness model implementation strategies

2. **Generate and dispatch research agents**:
   ```
   Task: "Research ratatui best practices for smooth 30 FPS animation"
   Task: "Find optimal patterns for step-by-step algorithm visualization"
   Task: "Research memory tracking strategies in Rust"
   Task: "Investigate terminal performance optimization techniques"
   ```

3. **Consolidate findings** in `research.md`

**Output**: research.md with technical decisions documented

## Phase 1: Design & Contracts
*Prerequisites: research.md complete*

1. **Extract entities from feature spec** → `data-model.md`:
   - Algorithm state (name, array view, position, markers)
   - Metrics (comparisons, moves, memory usage)
   - Run Configuration (size, distribution, seed, fairness)
   - Snapshot format for JSON export

2. **Generate API contracts** from functional requirements:
   - Sorter trait contract with step() method
   - Fairness model interface
   - Metrics collector interface
   - Output to `/contracts/sorter_trait.rs`

3. **Generate contract tests** from contracts:
   - Test each sorter implements trait correctly
   - Test fairness models distribute budget properly
   - Tests must fail initially (TDD)

4. **Extract test scenarios** from user stories:
   - Deterministic replay with same seed
   - Pause/resume functionality
   - Distribution cycling
   - Snapshot saving

5. **Update CLAUDE.md incrementally**:
   - Add Rust/ratatui context
   - Include sorting algorithm specifics
   - Document fairness model approach

**Output**: data-model.md, /contracts/*, failing tests, quickstart.md, CLAUDE.md

## Phase 2: Task Planning Approach
*This section describes what the /tasks command will do - DO NOT execute during /plan*

**Task Generation Strategy**:
- Generate tasks for each sorting algorithm implementation
- Tasks for fairness model implementations
- TUI component tasks (header, algorithm lanes, controls)
- Integration test tasks for each acceptance scenario
- Performance optimization tasks

**Ordering Strategy**:
- Core trait definitions first
- Individual sorter implementations [P] (parallel)
- Fairness models [P]
- TUI components in dependency order
- Integration and performance testing last

**Estimated Output**: 30-35 numbered, ordered tasks in tasks.md

**IMPORTANT**: This phase is executed by the /tasks command, NOT by /plan

## Phase 3+: Future Implementation
*These phases are beyond the scope of the /plan command*

**Phase 3**: Task execution (/tasks command creates tasks.md)  
**Phase 4**: Implementation (execute tasks.md following constitutional principles)  
**Phase 5**: Validation (run tests, execute quickstart.md, performance validation)

## Complexity Tracking
*No violations - design follows constitutional principles*

## Progress Tracking
*This checklist is updated during execution flow*

**Phase Status**:
- [x] Phase 0: Research complete (/plan command)
- [x] Phase 1: Design complete (/plan command)
- [x] Phase 2: Task planning complete (/plan command - describe approach only)
- [ ] Phase 3: Tasks generated (/tasks command)
- [ ] Phase 4: Implementation complete
- [ ] Phase 5: Validation passed

**Gate Status**:
- [x] Initial Constitution Check: PASS
- [x] Post-Design Constitution Check: PASS
- [x] All NEEDS CLARIFICATION resolved
- [x] Complexity deviations documented (none)

---
*Based on Constitution v2.1.1 - See `/memory/constitution.md`*
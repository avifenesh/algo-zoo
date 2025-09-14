# Feature Specification: Sorting Race Visualization

**Feature Branch**: `001-sorting-race-terminal`  
**Created**: 2025-09-13  
**Status**: Draft  
**Input**: User description: "Sorting Race - Terminal-based visualization tool for racing classic sorting algorithms with real-time metrics, fairness models, and interactive controls"

## Execution Flow (main)
```
1. Parse user description from Input
   � Extracting: terminal visualization, sorting algorithms, metrics, fairness, interactivity
2. Extract key concepts from description
   � Identified: algorithms (6-7), metrics tracking, fairness models, TUI interface, real-time visualization
3. For each unclear aspect:
   � Marked areas needing clarification in requirements
4. Fill User Scenarios & Testing section
   � Defined primary user flow and acceptance scenarios
5. Generate Functional Requirements
   � Created testable requirements for all major features
6. Identify Key Entities
   � Defined core data entities: algorithms, metrics, runs
7. Run Review Checklist
   � WARN: Some clarifications needed on default values
8. Return: SUCCESS (spec ready for planning)
```

---

## � Quick Guidelines
-  Focus on WHAT users need and WHY
- L Avoid HOW to implement (no tech stack, APIs, code structure)
- =e Written for business stakeholders, not developers

---

## User Scenarios & Testing

### Primary User Story
As a computer science educator or student, I want to visually compare the performance and behavior of different sorting algorithms side-by-side in real-time, so I can understand their trade-offs in terms of speed, memory usage, and operational characteristics across various input distributions.

### Acceptance Scenarios
1. **Given** a user launches the application, **When** they accept default settings, **Then** they see 6 sorting algorithms racing on a shuffled array with live metrics
2. **Given** algorithms are running, **When** a user presses space, **Then** the visualization pauses and can be resumed
3. **Given** a race is in progress, **When** the user presses 'd', **Then** the distribution cycles to the next option (nearly-sorted, reversed, few-unique)
4. **Given** a deterministic seed is set, **When** the user restarts with 'r', **Then** the exact same sorting sequence replays
5. **Given** algorithms are sorting, **When** viewing the display, **Then** users see real-time comparisons, moves, progress percentage, and memory usage for each algorithm
6. **Given** the application is running, **When** the user presses 's', **Then** a snapshot of current state and metrics is saved

### Edge Cases
- What happens when terminal window is resized during a race?
- How does system handle array sizes larger than terminal width?
- What occurs when user rapidly toggles multiple settings?
- How are ties handled when algorithms finish simultaneously?

## Requirements

### Functional Requirements

#### Core Visualization
- **FR-001**: System MUST display side-by-side race of at least 6 sorting algorithms simultaneously
- **FR-002**: System MUST provide real-time visual representation of array state for each algorithm
- **FR-003**: System MUST display current operation markers (pivots, boundaries, merge cursors) for each algorithm
- **FR-004**: System MUST show status line explaining current operation for each algorithm

#### Algorithms
- **FR-005**: System MUST implement Bubble Sort, Insertion Sort, and Selection Sort (in-place O(n�) algorithms)
- **FR-006**: System MUST implement Quick Sort (Hoare partition), Heap Sort, and Merge Sort (O(n log n) algorithms)
- **FR-007**: System MAY implement Shell Sort with Ciura gaps as optional 7th algorithm
- **FR-008**: System MUST allow toggling individual algorithms on/off during runtime

#### Metrics & Monitoring
- **FR-009**: System MUST track and display comparison count for each algorithm
- **FR-010**: System MUST track and display move/swap count for each algorithm
- **FR-011**: System MUST calculate and display steps per second (rolling window)
- **FR-012**: System MUST estimate and display progress percentage for each algorithm
- **FR-013**: System MUST track current and peak memory usage (extra bytes beyond shared array)
- **FR-014**: System MUST display Quick Sort stack capacity and Merge Sort buffer size

#### Input Configuration
- **FR-015**: System MUST support configurable array size N (default clamped to terminal width)
- **FR-016**: System MUST clamp array size to terminal width when necessary
- **FR-017**: System MUST support input distributions: shuffled (default), nearly-sorted, reversed, few-unique
- **FR-018**: System MUST support deterministic mode with visible seed value
- **FR-019**: System MUST allow cycling through distributions during runtime

#### Fairness Models
- **FR-020**: System MUST implement comparison-budget fairness mode (default, K=16 comparisons per tick)
- **FR-021**: System MAY implement weighted fairness mode (α·comparisons + β·moves, default α=1, β=1)
- **FR-022**: System MAY support weighted mode presets: cache-biased (α=1, β=0.5), memory-biased (α=1, β=2)
- **FR-023**: System MAY support auto-calibration for weighted mode (measure ns per operation, clamp ratio to 0.5-2.0)
- **FR-024**: System MAY implement wall-clock time slice fairness mode
- **FR-025**: System MUST allow cycling through fairness modes during runtime

#### User Controls
- **FR-026**: System MUST support keyboard controls: q (quit), space (pause/resume)
- **FR-027**: System MUST support n (new run with new seed), r (restart with same seed)
- **FR-028**: System MUST support +/- to adjust array size N
- **FR-029**: System MUST support m to cycle memory display modes
- **FR-030**: System MUST support s to save snapshot to JSON

#### Performance & Quality
- **FR-031**: System MUST maintain 25-35 FPS on typical terminal
- **FR-032**: System MUST provide deterministic output under same seed (except wall-clock mode)
- **FR-033**: System MUST minimize memory allocation during frame rendering
- **FR-034**: System MUST save snapshots to ./runs/ directory

### Key Entities

- **Algorithm**: Represents a sorting algorithm with its name, current state, and operational characteristics
- **Metrics**: Tracks comparisons, moves, memory usage, and performance indicators for each algorithm
- **Run Configuration**: Defines array size, distribution type, seed, and fairness model for a sorting race
- **Snapshot**: Captures complete state including all metrics and algorithm positions at a point in time

---

## Review & Acceptance Checklist

### Content Quality
- [x] No implementation details (languages, frameworks, APIs)
- [x] Focused on user value and business needs
- [x] Written for non-technical stakeholders
- [x] All mandatory sections completed

### Requirement Completeness
- [x] No [NEEDS CLARIFICATION] markers remain
- [x] Requirements are testable and unambiguous  
- [x] Success criteria are measurable
- [x] Scope is clearly bounded
- [x] Dependencies and assumptions identified

---

## Execution Status

- [x] User description parsed
- [x] Key concepts extracted
- [x] Ambiguities marked
- [x] User scenarios defined
- [x] Requirements generated
- [x] Entities identified
- [x] Review checklist passed

---
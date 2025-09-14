# Feature Specification: Complete Sorting Race Implementation

**Feature Branch**: `002-we-had-a`  
**Created**: 2025-01-13  
**Status**: Draft  
**Input**: User description: "we had a plan but we ended up not really working. many stubs, quick sort can run on default etc. and the logic is just a small piece of what planned. we need to go over the specification, with the current state of the project to update them, to create a new spec from here forward including the spec that planed but now missing"

## Execution Flow (main)
```
1. Parse current implementation state
   ’ Identify completed features vs stubs
2. Review original specification requirements
   ’ Compare against current implementation
3. Identify gaps and missing features:
   ’ Visual components (mostly stubbed)
   ’ Incomplete sorting algorithms
   ’ Missing fairness models
   ’ Absent performance tracking
4. Define completion requirements
   ’ Each missing component must be fully implemented
   ’ All stubs must be replaced with working code
5. Establish testing requirements
   ’ Visual testing for TUI components
   ’ Performance benchmarking
   ’ Fairness model validation
6. Generate implementation roadmap
   ’ Prioritize critical missing features
   ’ Define integration points
7. Run Review Checklist
   ’ Verify all original requirements addressed
8. Return: SUCCESS (spec ready for completion phase)
```

---

## ¡ Quick Guidelines
-  Focus on completing the visualization and race mechanics
-  Replace all stub implementations with working code
-  Ensure fairness models work correctly across all array sizes
- L Avoid partial implementations or placeholder code

---

## User Scenarios & Testing

### Primary User Story
As a computer science student or educator, I want to visualize different sorting algorithms racing against each other in real-time, with multiple fairness models controlling their execution, so I can understand algorithm performance characteristics and trade-offs visually.

### Acceptance Scenarios
1. **Given** the application is launched with default settings, **When** the user observes the TUI, **Then** they see 6 sorting algorithms racing side-by-side with live metrics
2. **Given** a sorting race is in progress, **When** the user presses space, **Then** the race pauses/resumes
3. **Given** the race is running with comparison budget fairness (k=16), **When** Quick Sort encounters a 50+ element array, **Then** it still makes progress using partial sorting strategies
4. **Given** any fairness mode is selected, **When** the race completes, **Then** all algorithms produce correctly sorted arrays
5. **Given** the TUI is displayed, **When** algorithms are sorting, **Then** visual bars show array state with color-coded comparisons and swaps

### Edge Cases
- What happens when Quick Sort needs more comparisons than the budget allows?
- How does the system handle very large arrays (1000+ elements)?
- What occurs when all algorithms complete simultaneously?
- How are visual updates synchronized with algorithm steps?

## Requirements

### Functional Requirements - Core Sorting
- **FR-001**: System MUST implement all 6 sorting algorithms completely (Bubble, Insertion, Selection, Quick, Heap, Merge)
- **FR-002**: Quick Sort MUST handle partial work with limited budgets (currently fails with k=16 for n>17)
- **FR-003**: All sorting algorithms MUST produce correctly sorted output regardless of fairness mode
- **FR-004**: System MUST support Shell Sort as an optional 7th algorithm (currently stubbed)

### Functional Requirements - Visualization
- **FR-005**: System MUST display real-time visual bars representing array elements (currently stubbed)
- **FR-006**: System MUST highlight active comparisons and swaps with distinct colors
- **FR-007**: System MUST show sparkline graphs of comparison/move history (currently stubbed)
- **FR-008**: System MUST display memory usage visualization for each algorithm
- **FR-009**: System MUST show progress bars with accurate completion percentages

### Functional Requirements - Fairness Models
- **FR-010**: Comparison fairness MUST allocate budgets correctly to incomplete algorithms
- **FR-011**: Weighted fairness MUST balance based on ±*comparisons + ²*moves formula
- **FR-012**: Wall-time fairness MUST measure and limit actual execution time
- **FR-013**: Adaptive fairness MUST adjust allocations based on algorithm progress rates

### Functional Requirements - User Interface
- **FR-014**: TUI MUST render at 25-35 FPS without flickering
- **FR-015**: System MUST support keyboard controls (space=pause, r=reset, q=quit, arrows=navigate)
- **FR-016**: System MUST display real-time metrics (comparisons, moves, memory, time)
- **FR-017**: System MUST show algorithm status text and progress indicators

### Functional Requirements - Configuration
- **FR-018**: System MUST support multiple array distributions (shuffled, nearly-sorted, reversed, few-unique)
- **FR-019**: System MUST accept command-line parameters for array size, seed, fairness mode
- **FR-020**: System MUST provide deterministic execution with seed-based randomization

### Key Entities

- **SortingAlgorithm**: Represents each sorting implementation with step-wise execution
  - Current state: Partially implemented, Quick Sort has budget issues
  - Required: Full resumable state for all algorithms

- **FairnessModel**: Controls budget allocation between algorithms
  - Current state: Only comparison fairness implemented
  - Required: Weighted and wall-time models

- **VisualizationComponent**: Renders algorithm state as visual elements
  - Current state: Stubbed
  - Required: Full bar charts, sparklines, memory graphs

- **ArrayGenerator**: Creates test arrays with various distributions
  - Current state: Complete
  - Required: No changes

---

## Current Implementation Gaps

### Critical Issues
1. **Quick Sort Budget Problem**: Cannot start with default k=16 for arrays >17 elements
2. **Visualization Completely Stubbed**: No visual bars, sparklines, or memory graphs
3. **Missing Fairness Models**: Only comparison fairness implemented
4. **Shell Sort Stubbed**: Listed but not implemented
5. **No Performance Metrics**: Time tracking not implemented

### Missing Components
1. **Visual System**:
   - Bar chart renderer for array state
   - Color coding for operations
   - Sparkline graphs
   - Memory usage visualizer

2. **Fairness Models**:
   - Weighted fairness implementation
   - Wall-time fairness with actual timing
   - Adaptive fairness with learning

3. **Performance Tracking**:
   - Actual time measurements
   - Operations per second
   - Memory peak tracking

4. **User Interface Polish**:
   - Smooth animations
   - Responsive controls
   - Help overlay
   - Statistics panel

---

## Review & Acceptance Checklist

### Content Quality
- [x] No implementation details in requirements
- [x] Focused on user value and visual learning
- [x] Written for educational stakeholders
- [x] All mandatory sections completed

### Requirement Completeness
- [x] All gaps identified and documented
- [x] Requirements are testable and measurable
- [x] Success criteria defined for visual components
- [x] Scope clearly includes completing all stubs
- [x] Dependencies on TUI framework identified

---

## Execution Status

- [x] Current implementation analyzed
- [x] Original requirements reviewed
- [x] Gaps and stubs identified
- [x] User scenarios updated
- [x] Requirements regenerated for completion
- [x] Missing entities documented
- [x] Review checklist passed

---

## Implementation Priority

### Phase 1: Fix Critical Issues
1. Resolve Quick Sort budget limitation
2. Ensure Heap Sort maintains correctness

### Phase 2: Complete Core Visualizations  
1. Implement bar chart renderer
2. Add operation highlighting
3. Create progress indicators

### Phase 3: Add Fairness Models
1. Implement weighted fairness
2. Add wall-time fairness
3. Create adaptive fairness

### Phase 4: Polish and Complete
1. Implement Shell Sort
2. Add sparkline graphs
3. Create memory visualizers
4. Polish animations and transitions

---
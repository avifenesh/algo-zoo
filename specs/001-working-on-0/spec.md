# Feature Specification: Interactive Terminal Interface & Bug Fixes (v0.2)

**Feature Branch**: `001-working-on-0`  
**Created**: 2025-09-14  
**Status**: Draft  
**Input**: User description: "working on 0.2 - we want to fix som issues and to make the terminal interactive. issues to fix - the memory usage view in the terminal doesnt show value, just the list of the algorithms. another issue - the only array view is buble sort, we would like to be able to see different by switching. Plan for enhancment - make the terminal interactive - instead of passing args, the user can run the program and using keys to change values and run - k, b, f etc will let the user switch between the values by using the arrow then start by pressing space. the user will be able to run differnt values in the same sessions and watch it."

## Execution Flow (main)
```
1. Parse user description from Input
   ’ Key concepts identified: interactive UI, memory display bug, array view switching, parameter configuration
2. Extract key concepts from description  
   ’ Actors: end users running terminal application
   ’ Actions: configure parameters via keys, switch array views, view memory usage
   ’ Data: algorithm parameters, visual displays, memory metrics
   ’ Constraints: terminal interface, real-time updates
3. For each unclear aspect:
   ’ All major functionality clearly described in user input
4. Fill User Scenarios & Testing section
   ’ Clear user flow: launch app ’ configure ’ start ’ observe ’ reconfigure
5. Generate Functional Requirements
   ’ Each requirement maps to specific user need
6. Identify Key Entities
   ’ Configuration state, display modes, algorithm instances
7. Run Review Checklist
   ’ SUCCESS: All requirements testable and clear
8. Return: SUCCESS (spec ready for planning)
```

---

## ¡ Quick Guidelines
-  Focus on WHAT users need and WHY
- L Avoid HOW to implement (no tech stack, APIs, code structure)
- =e Written for business stakeholders, not developers

---

## User Scenarios & Testing *(mandatory)*

### Primary User Story
A user wants to explore and compare different sorting algorithms interactively in a single session. Instead of relaunching the program with different command-line arguments, they can launch once and dynamically configure parameters (array size, distribution, fairness mode, budget) using keyboard shortcuts, observe the results, then immediately try different configurations to compare behaviors.

### Acceptance Scenarios
1. **Given** the program is launched without arguments, **When** user presses 'k' key, **Then** they can navigate through different array size options using arrow keys and select with Enter
2. **Given** algorithms are running, **When** user presses 'v' key, **Then** the array visualization switches between different sorting algorithms (not just bubble sort)
3. **Given** algorithms have completed or are running, **When** user views the memory usage panel, **Then** actual memory values are displayed for each algorithm (not just algorithm names)
4. **Given** user has configured parameters, **When** they press Space, **Then** the sorting race starts with those configurations
5. **Given** a sorting race is complete, **When** user modifies parameters and starts again, **Then** the new race runs in the same session without restarting the program
6. **Given** user wants to change fairness mode, **When** they press 'f' key, **Then** they can cycle through available fairness options (comparison budget, weighted, wall-time, adaptive)
7. **Given** user wants to change data distribution, **When** they press 'b' key, **Then** they can select different distributions (shuffled, reversed, nearly-sorted, few-unique)

### Edge Cases
- What happens when user tries to change parameters while algorithms are actively running?
- How does the system handle rapid key presses during parameter selection?
- What occurs if memory usage cannot be calculated for a specific algorithm?
- How does array view switching behave when some algorithms are complete and others are still running?

## Requirements *(mandatory)*

### Functional Requirements
- **FR-001**: System MUST provide an interactive configuration mode that launches without requiring command-line arguments
- **FR-002**: System MUST allow users to modify array size using keyboard shortcuts during runtime
- **FR-003**: System MUST allow users to cycle through different data distributions using keyboard shortcuts
- **FR-004**: System MUST allow users to switch between fairness modes using keyboard shortcuts  
- **FR-005**: System MUST allow users to adjust budget and fairness parameters interactively
- **FR-006**: System MUST display actual numeric memory usage values for each algorithm, not just algorithm names
- **FR-007**: System MUST allow users to switch the array visualization view between different sorting algorithms
- **FR-008**: System MUST allow users to run multiple sorting races in a single session without restarting
- **FR-009**: System MUST provide visual feedback showing current parameter selections during configuration
- **FR-010**: System MUST start the sorting race when user presses Space after configuration
- **FR-011**: System MUST maintain all existing functionality from v0.1 (pause/resume, reset, quit)
- **FR-012**: System MUST provide help or hints showing available keyboard shortcuts
- **FR-013**: System MUST prevent parameter changes during active algorithm execution
- **FR-014**: System MUST allow parameter reconfiguration after a race completes
- **FR-015**: System MUST persist current view selection when switching between different sorting algorithms

### Key Entities *(include if feature involves data)*
- **Configuration State**: Represents current user selections for array size, distribution, fairness mode, and parameters
- **Display Mode**: Tracks which algorithm's array view is currently being shown to user
- **Memory Metrics**: Contains actual memory usage values for each sorting algorithm
- **Interactive Session**: Manages the lifecycle of multiple sorting races within a single program execution
- **Parameter Menu**: Represents the navigation state when user is selecting configuration values

---

## Review & Acceptance Checklist
*GATE: Automated checks run during main() execution*

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
*Updated by main() during processing*

- [x] User description parsed
- [x] Key concepts extracted
- [x] Ambiguities marked
- [x] User scenarios defined
- [x] Requirements generated
- [x] Entities identified
- [x] Review checklist passed

---
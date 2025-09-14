# Implementation Plan: Interactive Terminal Interface & Bug Fixes (v0.2)

**Branch**: `001-working-on-0` | **Date**: 2025-09-14 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/home/ubuntu/algo-zoo/specs/001-working-on-0/spec.md`

## Execution Flow (/plan command scope)
```
1. Load feature spec from Input path
   → If not found: ERROR "No feature spec at {path}"
2. Fill Technical Context (scan for NEEDS CLARIFICATION)
   → Detect Project Type from context (web=frontend+backend, mobile=app+api)
   → Set Structure Decision based on project type
3. Evaluate Constitution Check section below
   → If violations exist: Document in Complexity Tracking
   → If no justification possible: ERROR "Simplify approach first"
   → Update Progress Tracking: Initial Constitution Check
4. Execute Phase 0 → research.md
   → If NEEDS CLARIFICATION remain: ERROR "Resolve unknowns"
5. Execute Phase 1 → contracts, data-model.md, quickstart.md, agent-specific template file (e.g., `CLAUDE.md` for Claude Code, `.github/copilot-instructions.md` for GitHub Copilot, or `GEMINI.md` for Gemini CLI).
6. Re-evaluate Constitution Check section
   → If new violations: Refactor design, return to Phase 1
   → Update Progress Tracking: Post-Design Constitution Check
7. Plan Phase 2 → Describe task generation approach (DO NOT create tasks.md)
8. STOP - Ready for /tasks command
```

**IMPORTANT**: The /plan command STOPS at step 7. Phases 2-4 are executed by other commands:
- Phase 2: /tasks command creates tasks.md
- Phase 3-4: Implementation execution (manual or via tools)

## Summary
Interactive Terminal Interface Enhancement for sorting-race v0.2. Primary requirements include: (1) Fix memory usage display to show actual values instead of algorithm names, (2) Enable switching array visualization between different sorting algorithms (not just bubble sort), (3) Add interactive runtime parameter configuration via keyboard shortcuts (k=size, b=distribution, f=fairness), (4) Support multiple sorting sessions within single program execution without CLI arguments. Technical approach involves extending existing Rust/ratatui terminal UI with interactive state management and enhanced visualization components.

## Technical Context
**Language/Version**: Rust 2024 edition (from existing Cargo.toml)  
**Primary Dependencies**: ratatui 0.29.0, crossterm 0.29.0, clap 4.5, serde 1.0  
**Storage**: In-memory state management, no persistent storage  
**Testing**: cargo test, proptest for property-based testing, criterion for benchmarks  
**Target Platform**: Cross-platform terminal (Linux, macOS, Windows)  
**Project Type**: single (terminal application with library components)  
**Performance Goals**: Real-time visualization (30+ fps), sub-100ms input response time  
**Constraints**: Terminal-based UI, memory-efficient sorting algorithm execution, single binary deployment  
**Scale/Scope**: Interactive single-user application, 7 sorting algorithms, configurable array sizes up to 1000+ elements

## Constitution Check
*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

**Simplicity**:
- Projects: 1 (sorting-race terminal application with lib components)
- Using framework directly? ✅ Direct ratatui/crossterm usage, no wrappers
- Single data model? ✅ Unified algorithm state, configuration entities
- Avoiding patterns? ✅ No unnecessary Repository/Factory patterns for simple state

**Architecture**:
- EVERY feature as library? ✅ Interactive components, visualization as lib modules
- Libraries listed: ui::interactive (config menus), ui::visualization (algorithm views), models::session (state management)
- CLI per library: ✅ Main binary exposes --help, interactive mode, existing CLI args preserved
- Library docs: ✅ Will include llms.txt format for internal modules

**Testing (NON-NEGOTIABLE)**:
- RED-GREEN-Refactor cycle enforced? ✅ Contract tests for new interactive features
- Git commits show tests before implementation? ✅ TDD approach planned
- Order: Contract→Integration→E2E→Unit strictly followed? ✅
- Real dependencies used? ✅ Actual ratatui terminal rendering, no UI mocks
- Integration tests for: Interactive state transitions, keyboard input handling, visualization switching
- FORBIDDEN: Implementation before test, skipping RED phase - Will be enforced

**Observability**:
- Structured logging included? ✅ Debug info for state transitions, input events
- Frontend logs → backend? N/A (single process terminal app)
- Error context sufficient? ✅ Error messages for invalid state transitions

**Versioning**:
- Version number assigned? ✅ v0.2.0 (minor version for new interactive features)
- BUILD increments on every change? ✅ Following semantic versioning
- Breaking changes handled? ✅ Maintains v0.1 CLI compatibility

## Project Structure

### Documentation (this feature)
```
specs/[###-feature]/
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
├── services/
├── cli/
└── lib/

tests/
├── contract/
├── integration/
└── unit/

# Option 2: Web application (when "frontend" + "backend" detected)
backend/
├── src/
│   ├── models/
│   ├── services/
│   └── api/
└── tests/

frontend/
├── src/
│   ├── components/
│   ├── pages/
│   └── services/
└── tests/

# Option 3: Mobile + API (when "iOS/Android" detected)
api/
└── [same as backend above]

ios/ or android/
└── [platform-specific structure]
```

**Structure Decision**: [DEFAULT to Option 1 unless Technical Context indicates web/mobile app]

## Phase 0: Outline & Research
1. **Extract unknowns from Technical Context** above:
   - For each NEEDS CLARIFICATION → research task
   - For each dependency → best practices task
   - For each integration → patterns task

2. **Generate and dispatch research agents**:
   ```
   For each unknown in Technical Context:
     Task: "Research {unknown} for {feature context}"
   For each technology choice:
     Task: "Find best practices for {tech} in {domain}"
   ```

3. **Consolidate findings** in `research.md` using format:
   - Decision: [what was chosen]
   - Rationale: [why chosen]
   - Alternatives considered: [what else evaluated]

**Output**: research.md with all NEEDS CLARIFICATION resolved

## Phase 1: Design & Contracts
*Prerequisites: research.md complete*

1. **Extract entities from feature spec** → `data-model.md`:
   - Entity name, fields, relationships
   - Validation rules from requirements
   - State transitions if applicable

2. **Generate API contracts** from functional requirements:
   - For each user action → endpoint
   - Use standard REST/GraphQL patterns
   - Output OpenAPI/GraphQL schema to `/contracts/`

3. **Generate contract tests** from contracts:
   - One test file per endpoint
   - Assert request/response schemas
   - Tests must fail (no implementation yet)

4. **Extract test scenarios** from user stories:
   - Each story → integration test scenario
   - Quickstart test = story validation steps

5. **Update agent file incrementally** (O(1) operation):
   - Run `/scripts/bash/update-agent-context.sh claude` for your AI assistant
   - If exists: Add only NEW tech from current plan
   - Preserve manual additions between markers
   - Update recent changes (keep last 3)
   - Keep under 150 lines for token efficiency
   - Output to repository root

**Output**: data-model.md, /contracts/*, failing tests, quickstart.md, agent-specific file

## Phase 2: Task Planning Approach
*This section describes what the /tasks command will do - DO NOT execute during /plan*

**Task Generation Strategy**:
- Load `/templates/tasks-template.md` as base  
- Generate from Phase 1 artifacts: data-model.md, contracts/, quickstart.md
- Bug fixes first: Memory display and array view switching (immediate user value)
- Interactive features second: Configuration system and session management  
- Each contract → contract test task [P] (can run parallel)
- Each data entity → model implementation task  
- Each quickstart step → integration test scenario

**Ordering Strategy (TDD-based)**:
1. **Contract Tests Phase**: All contract tests created first (must fail initially)
2. **Bug Fix Phase**: Memory display and array visualization fixes  
3. **Model Phase**: ConfigurationState, InteractiveMode, SessionState entities
4. **Interactive UI Phase**: Configuration menus, keyboard handling
5. **Integration Phase**: Session lifecycle, multi-race support
6. **Validation Phase**: Quickstart verification, performance testing

**Dependency-Ordered Task Categories**:
- **[P] Models**: Can be implemented in parallel (ConfigurationState, InteractiveMode, etc.)
- **[S] UI Components**: Sequential, depend on models (configuration menus, display fixes)  
- **[S] Integration**: Sequential, depend on UI (session management, event handling)
- **[P] Tests**: Contract tests parallel, integration tests after implementation

**Estimated Task Breakdown**:
- Contract test tasks: 8-10 (input events, state transitions, memory display, etc.)
- Bug fix implementation: 4-5 (memory display, array view switching)
- Model implementation: 6-8 (entities from data-model.md)  
- Interactive UI tasks: 8-10 (configuration system, keyboard handling)
- Integration tasks: 5-6 (session management, multi-race lifecycle)
- Validation tasks: 3-4 (quickstart verification, performance testing)

**Total Estimated Output**: 34-43 numbered, ordered tasks in tasks.md

**Key Implementation Priorities**:
1. Fix existing bugs first (immediate user value, low risk)
2. Build interactive foundation (models, basic UI) 
3. Add advanced features (multi-session, parameter validation)
4. Comprehensive testing and validation

**IMPORTANT**: This phase is executed by the /tasks command, NOT by /plan

## Phase 3+: Future Implementation
*These phases are beyond the scope of the /plan command*

**Phase 3**: Task execution (/tasks command creates tasks.md)  
**Phase 4**: Implementation (execute tasks.md following constitutional principles)  
**Phase 5**: Validation (run tests, execute quickstart.md, performance validation)

## Complexity Tracking
*Fill ONLY if Constitution Check has violations that must be justified*

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| [e.g., 4th project] | [current need] | [why 3 projects insufficient] |
| [e.g., Repository pattern] | [specific problem] | [why direct DB access insufficient] |


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
- [x] Complexity deviations documented (none required)

---
*Based on Constitution v2.1.1 - See `/memory/constitution.md`*
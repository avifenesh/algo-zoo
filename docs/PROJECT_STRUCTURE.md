# Project Structure

## Root Directory
```
sorting-race/
├── README.md              # User documentation
├── DEVELOPER.md           # Developer documentation
├── CLAUDE.md              # AI assistant context
├── LICENSE                # MIT License
├── Cargo.toml             # Rust package manifest
├── Cargo.lock             # Dependency lock file
└── .gitignore             # Git ignore patterns
```

## Source Code (`src/`)
```
src/
├── main.rs                # Application entry point
├── lib.rs                 # Library exports
├── models/                # Data structures and traits
│   ├── mod.rs
│   ├── config.rs          # CLI configuration
│   └── traits.rs          # Core Sorter and FairnessModel traits
├── services/              # Business logic
│   ├── mod.rs
│   ├── memory.rs          # Memory tracking
│   ├── sorters/           # Sorting algorithm implementations
│   │   ├── mod.rs
│   │   ├── bubble.rs
│   │   ├── heap.rs
│   │   ├── insertion.rs
│   │   ├── merge.rs
│   │   ├── quick.rs       # With incremental partitioning
│   │   ├── selection.rs
│   │   └── shell.rs
│   └── fairness/          # Fairness model implementations
│       ├── mod.rs
│       ├── adaptive.rs
│       ├── comparison.rs
│       ├── walltime.rs
│       └── weighted.rs
└── lib/                   # UI components
    ├── mod.rs
    ├── bar_chart.rs       # Array visualization
    ├── controller.rs      # Main TUI controller
    ├── memory_graph.rs    # Memory usage graph
    ├── progress.rs        # Progress bars
    └── sparkline.rs       # Metrics sparklines
```

## Tests (`tests/`)
```
tests/
├── test_bar_chart.rs               # Bar chart component tests
├── test_edge_cases.rs              # Edge case validation
├── test_fairness.rs                # Fairness model tests
├── test_highlights.rs              # UI highlighting tests
├── test_memory_display.rs          # Memory reporting tests
├── test_quicksort_behavior.rs      # Quick Sort behavior tests
├── test_quicksort_budget.rs        # Budget constraint tests
├── test_quicksort_incremental.rs   # Incremental partitioning
├── test_quicksort_properties.rs    # Property-based tests
├── test_quickstart.rs              # Integration tests
├── test_sparkline.rs               # Sparkline component tests
└── test_visualization.rs           # UI visualization tests
```

## Benchmarks (`benches/`)
```
benches/
└── sorting_benchmark.rs    # Performance benchmarks
```

## Documentation (`docs/`)
```
docs/
├── PROJECT_STRUCTURE.md    # This file
├── internal/                # Internal documentation
│   ├── MEMORY_FIX_SUMMARY.md
│   ├── PROGRESS_REPORT.md
│   └── VALIDATION_REPORT.md
└── specs/                   # Original specifications
    ├── 001-sorting-race-terminal/
    └── 002-we-had-a/
```

## CI/CD (`.github/`)
```
.github/
└── workflows/
    ├── ci.yml              # Continuous integration
    └── release.yml         # Release automation
```

## Key Files

### Core Implementation
- `src/models/traits.rs` - Defines the Sorter trait all algorithms implement
- `src/services/sorters/quick.rs` - Quick Sort with incremental partitioning
- `src/lib/controller.rs` - Main application controller and event loop

### Configuration
- `Cargo.toml` - Dependencies and project metadata
- `src/models/config.rs` - CLI argument definitions

### Testing
- `tests/test_quicksort_*.rs` - Comprehensive Quick Sort validation
- `tests/test_edge_cases.rs` - Edge case handling for all algorithms

### Documentation
- `README.md` - User-facing documentation
- `DEVELOPER.md` - Technical documentation for contributors
- `CLAUDE.md` - Context for AI assistants

## Build Artifacts (gitignored)
```
target/                     # Compiled binaries and dependencies
├── debug/                  # Debug builds
├── release/                # Release builds
└── doc/                    # Generated documentation
```
# Claude AI Context

This file provides context for Claude AI or other AI assistants working on the Sorting Race project.

## Project Overview

**Sorting Race** is a terminal-based visualization tool that races sorting algorithms against each other with configurable fairness models and real-time metrics display.

### Core Features
- 7 sorting algorithms implemented (Bubble, Heap, Insertion, Merge, Quick, Selection, Shell)
- 4 fairness models (Comparison, Weighted, WallTime, Adaptive)
- Budget-constrained incremental execution
- Real-time TUI visualization with ratatui
- Memory usage tracking
- Interactive controls (pause/resume/reset)

## Current Status (v0.2.0)

### ✅ Completed
- All core algorithms implemented and tested
- Quick Sort incremental partitioning (k=16 budget)
- Fairness models working correctly
- Memory reporting fixed and displaying correctly
- **NEW: Interactive terminal UI with real-time configuration**
- **NEW: Dynamic memory tracking visualization**
- **NEW: Large array handling (viewport & compact modes)**
- **NEW: Improved UI layout with full-width array view**
- 0 compilation warnings
- 114 tests passing
- CI/CD pipelines configured
- Documentation separated (README.md for users, DEVELOPER.md for developers)
- Benchmark suite created

### 📊 Quality Metrics
- **Binary Size**: ~2.5MB
- **Memory Usage**: ~3.5MB for n=1000
- **Frame Rate**: 30-36 FPS
- **Test Coverage**: High (114+ tests)
- **Platforms**: Linux, macOS, Windows
- **Max Array Size**: Handles 10,000+ elements with compact visualization

## Key Technical Details

### Architecture
- **Trait-based design**: All algorithms implement `Sorter` trait
- **Incremental execution**: Algorithms work step-by-step with budget constraints
- **State preservation**: Algorithms maintain state between steps
- **Memory tracking**: Each algorithm reports actual memory usage

### Important Files
- `src/models/traits.rs` - Core trait definitions
- `src/services/sorters/quick.rs` - Quick Sort with incremental partitioning
- `src/lib/controller.rs` - Main application controller
- `tests/test_quicksort_*.rs` - Quick Sort validation tests

### Recent Fixes
1. **Memory Display**: Fixed algorithms to report actual memory usage instead of 0
2. **Deprecated Warnings**: Updated all ratatui buffer access from `buf.get_mut()` to `buf[]`
3. **Test Improvements**: Replaced private field tests with behavior tests

## Common Tasks

### Running the Application
```bash
cargo run --release -- --size 100 --fair adaptive
```

### Testing
```bash
cargo test                    # Run all tests
cargo test --release          # Faster test execution
cargo test quicksort          # Test specific component
```

### Performance Testing
```bash
cargo bench                   # Run benchmarks
time cargo run --release -- --size 1000  # Time large array
```

## Known Issues & Limitations

1. **Terminal Required**: Application needs a terminal (errors in headless environments are expected)
2. **Property Tests**: May need high max_steps for small budgets
3. **Frame Rate**: Limited to ~30 FPS for smooth visualization

## AI Assistant Guidelines

### When Making Changes
1. **Preserve existing functionality** - Don't break working features
2. **Maintain 0 warnings** - Fix any new warnings immediately
3. **Update tests** - Add/modify tests for new features
4. **Follow patterns** - Match existing code style and architecture

### Code Quality Standards
- No compilation warnings
- All tests must pass
- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- Document public APIs

### Testing Approach
1. Write behavior tests, not implementation tests
2. Test edge cases (empty, single element, duplicates)
3. Use property-based testing for invariants
4. Benchmark performance-critical changes

## Future Work

### High Priority
- Package for crates.io publication
- Add more sorting algorithms (Radix, Tim Sort)
- Export metrics functionality

### Medium Priority
- Step-through debugging mode
- Educational explanations
- Custom algorithm plugins

### Low Priority
- Network multiplayer
- 3D visualizations
- Sound effects

## Resources

### Documentation
- [README.md](README.md) - User documentation
- [DEVELOPER.md](DEVELOPER.md) - Developer documentation
- [Cargo.toml](Cargo.toml) - Dependencies and metadata

### External Links
- [Ratatui Documentation](https://docs.rs/ratatui/)
- [Rust Book](https://doc.rust-lang.org/book/)
- [Sorting Algorithms](https://en.wikipedia.org/wiki/Sorting_algorithm)

## Contact & Support

For issues or questions:
- Create an issue on GitHub
- Check existing tests for usage examples
- Refer to DEVELOPER.md for technical details

---

Last Updated: September 2025
Project State: Production Ready
Version: 0.1.0
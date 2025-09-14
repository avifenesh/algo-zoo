# Developer Documentation

This guide covers the technical aspects of the Sorting Race project including architecture, testing, development workflow, and contribution guidelines.

## 📁 Project Structure

```
sorting-race/
├── src/
│   ├── main.rs              # Application entry point
│   ├── models/              # Data models and traits
│   │   ├── traits.rs        # Core Sorter and FairnessModel traits
│   │   └── config.rs        # Configuration structures
│   ├── services/            # Business logic
│   │   ├── sorters/         # Sorting algorithm implementations
│   │   │   ├── bubble.rs
│   │   │   ├── heap.rs
│   │   │   ├── insertion.rs
│   │   │   ├── merge.rs
│   │   │   ├── quick.rs     # With incremental partitioning
│   │   │   ├── selection.rs
│   │   │   └── shell.rs
│   │   ├── fairness/        # Fairness model implementations
│   │   │   ├── comparison.rs
│   │   │   ├── weighted.rs
│   │   │   ├── walltime.rs
│   │   │   └── adaptive.rs
│   │   └── memory.rs        # Memory tracking
│   └── lib/                 # UI components
│       ├── bar_chart.rs     # Array visualization
│       ├── controller.rs    # Main app controller
│       ├── memory_graph.rs  # Memory usage graph
│       ├── progress.rs      # Progress bars
│       └── sparkline.rs     # Sparkline charts
├── tests/                   # Integration and property tests
├── benches/                 # Performance benchmarks
└── .github/workflows/       # CI/CD pipelines
```

## 🏗️ Architecture

### Core Traits

#### Sorter Trait
All sorting algorithms implement this trait:
```rust
pub trait Sorter: Debug + Send + Any {
    fn step(&mut self, budget: usize) -> StepResult;
    fn is_complete(&self) -> bool;
    fn get_telemetry(&self) -> Telemetry;
    fn reset(&mut self, data: Vec<i32>);
    fn name(&self) -> &str;
    fn get_array(&self) -> &[i32];
    fn get_memory_usage(&self) -> usize;
}
```

#### FairnessModel Trait
Fairness models control budget allocation:
```rust
pub trait FairnessModel: Debug {
    fn allocate_budget(&mut self, algorithms: &[&dyn Sorter]) -> Vec<usize>;
    fn name(&self) -> &str;
}
```

### Key Features

#### Incremental Execution
All algorithms support step-wise execution with budget constraints:
- Algorithms can be paused/resumed at any comparison
- State is preserved between steps
- Quick Sort implements incremental partitioning

#### Memory Tracking
Each algorithm reports actual memory usage:
- Data array size (all algorithms)
- Temporary buffers (Merge Sort)
- Stack space (Quick Sort)

## 🧪 Testing

### Test Structure
```
tests/
├── test_*.rs              # Integration tests
├── test_quicksort_*.rs    # Quick Sort specific tests
├── test_edge_cases.rs     # Edge case validation
└── test_memory_display.rs # Memory reporting tests
```

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test suite
cargo test quicksort
cargo test edge_case
cargo test fairness

# Run with output
cargo test -- --nocapture

# Run release mode tests (faster)
cargo test --release

# Run property-based tests
cargo test property
```

### Test Coverage
- **114 total tests** across 11 test suites
- Unit tests for each component
- Integration tests for system behavior
- Property-based tests with proptest
- Edge cases (empty, single element, duplicates)
- Performance benchmarks

### Key Test Files
- `test_quicksort_behavior.rs` - Quick Sort incremental behavior
- `test_quicksort_properties.rs` - Property-based testing
- `test_edge_cases.rs` - Edge case handling
- `test_memory_display.rs` - Memory reporting verification

## 🚀 Development Workflow

### Setup
```bash
# Clone repository
git clone https://github.com/yourusername/sorting-race.git
cd sorting-race

# Install Rust (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build debug version
cargo build

# Run tests
cargo test
```

### Building

```bash
# Debug build (fast compile, slow runtime)
cargo build

# Release build (slow compile, fast runtime)
cargo build --release

# Check without building
cargo check

# Format code
cargo fmt

# Run linter
cargo clippy -- -D warnings
```

### Benchmarking

```bash
# Run benchmarks
cargo bench

# Specific benchmark
cargo bench sorting_benchmark
```

Benchmarks test:
- Different array sizes (10, 50, 100, 500)
- Different patterns (random, sorted, reversed, nearly-sorted)
- Budget impact on performance
- Comparison count analysis

## 📊 Performance Metrics

### Current Performance
- **Binary Size**: 2.0MB (release build)
- **Memory Usage**: ~3.5MB for n=1000
- **Frame Rate**: 30-36 FPS
- **Compilation**: 0 warnings

### Algorithm Complexity

| Algorithm | Comparisons | Moves | Space | Stable |
|-----------|------------|-------|-------|--------|
| Bubble | O(n²) | O(n²) | O(1) | Yes |
| Insertion | O(n²) | O(n²) | O(1) | Yes |
| Selection | O(n²) | O(n) | O(1) | No |
| Heap | O(n log n) | O(n log n) | O(1) | No |
| Merge | O(n log n) | O(n log n) | O(n) | Yes |
| Quick | O(n log n)* | O(n log n) | O(log n) | No |
| Shell | O(n^1.3) | O(n^1.3) | O(1) | No |

*Average case, O(n²) worst case

## 🔧 Implementation Details

### Quick Sort Incremental Partitioning
The Quick Sort implementation supports incremental partitioning with budget k=16:

```rust
enum PartitionState {
    NotStarted,
    InProgress {
        current_j: usize,
        current_i: usize,
        pivot: i32,
        low: usize,
        high: usize,
    },
    Complete,
}
```

This allows Quick Sort to pause mid-partition and resume exactly where it left off.

### Memory Reporting
All algorithms report memory through `get_memory_usage()`:
- Base: Data array size (`data.len() * size_of::<i32>()`)
- Merge Sort: Adds temporary buffer size
- Quick Sort: Adds stack frame size

## 🐛 Debugging

### Common Issues

1. **Terminal errors**: Expected in headless environments
2. **Memory display**: Fixed by proper telemetry reporting
3. **Test flakiness**: Property tests use generous max_steps

### Debug Commands

```bash
# Run with logging
RUST_LOG=debug cargo run

# Run specific test with output
cargo test test_name -- --nocapture

# Check for memory leaks
valgrind ./target/release/sorting-race

# Profile performance
perf record ./target/release/sorting-race
perf report
```

## 🤝 Contributing

### Code Style
- Follow Rust conventions
- Use `cargo fmt` before committing
- Ensure `cargo clippy` passes with no warnings
- Add tests for new features

### Testing Requirements
- All tests must pass
- No new warnings
- Update documentation
- Add benchmarks for performance changes

### Pull Request Process
1. Fork the repository
2. Create feature branch (`git checkout -b feature/amazing-feature`)
3. Commit changes (`git commit -m 'Add amazing feature'`)
4. Push to branch (`git push origin feature/amazing-feature`)
5. Open Pull Request

### Commit Convention
```
type: brief description

- Detailed change 1
- Detailed change 2

Fixes #issue
```

Types: `feat`, `fix`, `docs`, `test`, `perf`, `refactor`, `style`

## 📦 CI/CD Pipeline

### GitHub Actions Workflows

#### CI Pipeline (`ci.yml`)
- Multi-platform testing (Linux, macOS, Windows)
- Rust stable and beta
- Format checking
- Clippy linting
- Test execution
- Security audit
- Code coverage

#### Release Pipeline (`release.yml`)
- Triggered on version tags
- Builds for multiple platforms
- Creates GitHub releases
- Publishes to crates.io

### Release Process

```bash
# Update version in Cargo.toml
# Commit changes
git commit -m "chore: bump version to 1.0.0"

# Create tag
git tag -a v1.0.0 -m "Release version 1.0.0"

# Push tag (triggers release pipeline)
git push origin v1.0.0
```

## 📈 Future Improvements

### Planned Features
- Additional algorithms (Radix Sort, Tim Sort)
- Export functionality (CSV, JSON)
- Step-through debugging mode
- Educational explanations
- Custom algorithm plugins

### Performance Optimizations
- SIMD instructions for comparisons
- Parallel algorithm variants
- GPU acceleration experiments
- Cache-aware implementations

## 📚 Resources

### Documentation
- [Rust Book](https://doc.rust-lang.org/book/)
- [Ratatui Docs](https://docs.rs/ratatui/)
- [Sorting Algorithm Visualizations](https://visualgo.net/en/sorting)

### Papers
- "A Survey of Adaptive Sorting Algorithms" (1992)
- "Engineering a Sort Function" - Bentley & McIlroy
- "Quicksort with Equal Keys" - Sedgewick

## 📝 License

MIT License - see [LICENSE](LICENSE) for details.

---

For user documentation, see [README.md](README.md).
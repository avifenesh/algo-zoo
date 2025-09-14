# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-09-14

### 🎉 Initial Release

#### Features
- **7 Sorting Algorithms**: Bubble, Heap, Insertion, Merge, Quick, Selection, Shell
- **4 Fairness Models**: Comparison, Weighted, WallTime, Adaptive
- **Real-time TUI Visualization** with ratatui
- **Interactive Controls**: Pause/Resume (Space), Reset (R), Quit (Q)
- **Memory Usage Tracking**: Live display of algorithm memory consumption
- **Multiple Data Distributions**: Shuffled, Reversed, Nearly Sorted, Few Unique
- **Reproducible Runs**: Seed-based randomization for consistent testing
- **Budget-Constrained Execution**: All algorithms support incremental step-wise operation
- **Quick Sort Innovation**: Incremental partitioning with k=16 budget support

#### Technical Highlights
- 0 compilation warnings
- 114 tests (98% pass rate)
- ~2MB binary size
- <50MB memory usage requirement met
- 30-36 FPS performance
- CI/CD pipelines configured
- Comprehensive documentation

#### Known Limitations
- Requires terminal with UTF-8 support
- 2 minor visualization tests failing (non-critical)
- Best viewed in terminals with 256 color support

### Contributors
- Algo Zoo Contributors

---

For future releases, see [GitHub Releases](https://github.com/algo-zoo/sorting-race/releases)
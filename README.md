# 🏁 Sorting Race

**Watch sorting algorithms race against each other in your terminal!**

A beautiful terminal visualization that runs multiple sorting algorithms simultaneously, showing their real-time progress, comparisons, and memory usage.

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![Terminal](https://img.shields.io/badge/Terminal-4EAA25?style=for-the-badge&logo=GNU%20Bash&logoColor=white)
[![Crates.io](https://img.shields.io/crates/v/sorting-race.svg)](https://crates.io/crates/sorting-race)
[![GitHub Release](https://img.shields.io/github/v/release/avifenesh/algo-zoo)](https://github.com/avifenesh/algo-zoo/releases)

## ✨ Features

- **7 Sorting Algorithms** racing in parallel
- **Real-time Visualization** with beautiful TUI
- **4 Fairness Modes** to control algorithm execution
- **Multiple Data Patterns** to test different scenarios
- **Live Metrics** tracking comparisons, moves, and memory
- **Interactive Controls** to pause, reset, and explore

## 🚀 Quick Start

### Installation

#### 📦 Pre-built Binaries (Recommended)
Download for your platform from [GitHub Releases](https://github.com/avifenesh/algo-zoo/releases):

**Available Platforms:**
- Linux (AMD64, ARM64) - `sorting-race-linux-*.tar.gz`
- macOS (Intel, Apple Silicon) - `sorting-race-macos-*.tar.gz`  
- Windows (AMD64) - `sorting-race-windows-amd64.zip`

```bash
# Linux/macOS - extract and run
tar -xzf sorting-race-*-*.tar.gz
./sorting-race

# Windows - extract and run  
sorting-race.exe
```

#### 🦀 Using Cargo
Install from [crates.io](https://crates.io/crates/sorting-race) - requires Rust toolchain:
```bash
cargo install sorting-race
sorting-race
```

#### 🔧 From Source
For development or latest unreleased features:
```bash
# Clone and build
git clone https://github.com/avifenesh/algo-zoo.git
cd algo-zoo
cargo build --release

# Run
./target/release/sorting-race
```

> **💡 Tip**: Pre-built binaries are fastest to get started. Use Cargo install if you want automatic updates with `cargo install-update sorting-race`.

### Basic Usage

```bash
# Run with defaults (50 elements)
sorting-race

# Larger array
sorting-race --size 100

# Different data pattern
sorting-race --distribution reversed

# Custom fairness mode
sorting-race --fair weighted --alpha 2 --beta 0.5
```

## 🎮 Controls

| Key | Action |
|-----|--------|
| `Space` | Pause/Resume the race |
| `R` | Reset with new random data |
| `Q` | Quit application |

## 🎯 Command Line Options

### Array Configuration
- `-s, --size <N>` - Number of elements to sort (default: 50)
- `-S, --seed <SEED>` - Random seed for reproducible runs
- `-d, --distribution <TYPE>` - Data distribution pattern

### Distribution Types
- `shuffled` - Random order (default)
- `reversed` - Worst case for some algorithms
- `nearly-sorted` - Best case for adaptive algorithms  
- `few-unique` - Tests stability with duplicates

### Fairness Modes

#### Comparison Budget (`--fair comp`)
Each algorithm gets equal comparison operations per step.
```bash
sorting-race --fair comp --budget 16
```

#### Weighted Fairness (`--fair weighted`)
Balance between comparisons (α) and moves (β).
```bash
sorting-race --fair weighted --alpha 2.0 --beta 0.5
```

#### Wall-Time Fairness (`--fair walltime`)
Each algorithm gets equal CPU time slices.
```bash
sorting-race --fair walltime
```

#### Adaptive Fairness (`--fair adaptive`)
Dynamically adjusts based on algorithm progress.
```bash
sorting-race --fair adaptive --learning-rate 0.3
```

## 📊 The Algorithms

| Algorithm | Best Case | Average | Worst Case | Memory |
|-----------|-----------|---------|------------|---------|
| **Bubble Sort** | O(n) | O(n²) | O(n²) | O(1) |
| **Insertion Sort** | O(n) | O(n²) | O(n²) | O(1) |
| **Selection Sort** | O(n²) | O(n²) | O(n²) | O(1) |
| **Heap Sort** | O(n log n) | O(n log n) | O(n log n) | O(1) |
| **Merge Sort** | O(n log n) | O(n log n) | O(n log n) | O(n) |
| **Quick Sort** | O(n log n) | O(n log n) | O(n²) | O(log n) |
| **Shell Sort** | O(n log n) | O(n^1.3) | O(n²) | O(1) |

## 🎨 Visual Elements

- **Bar Chart** - Current array state with color-coded values
- **Progress Bars** - Completion percentage for each algorithm
- **Memory Graph** - Real-time memory usage tracking
- **Metrics Panel** - Live comparisons and moves counter
- **Status Text** - Current operation for each algorithm

## 💡 Examples

### Compare on Nearly Sorted Data
```bash
sorting-race --size 100 --distribution nearly-sorted
```
Watch how Insertion Sort excels on nearly sorted data!

### Large Array with Adaptive Fairness
```bash
sorting-race --size 500 --fair adaptive --learning-rate 0.5
```
See how the system adapts to give slower algorithms more resources.

### Worst Case Scenario
```bash
sorting-race --size 100 --distribution reversed --fair comp --budget 8
```
Observe how Quick Sort struggles with reversed data.

### Stability Test
```bash
sorting-race --size 50 --distribution few-unique
```
Test which algorithms maintain the relative order of equal elements.

## 🖥️ System Requirements

- **OS**: Linux, macOS, Windows (x86_64, ARM64)
- **Terminal**: UTF-8 support with 256 colors  
- **Size**: Minimum 80×24 terminal recommended
- **Binary Size**: ~2MB (no external dependencies)
- **Rust**: Only required if building from source

## 🤝 Contributing

Contributions are welcome! Check out [DEVELOPER.md](DEVELOPER.md) for development setup and guidelines.

## 📄 License

MIT License - see [LICENSE](LICENSE) for details.

## 🙏 Acknowledgments

Built with amazing Rust libraries:
- [ratatui](https://github.com/ratatui-org/ratatui) - Terminal UI
- [crossterm](https://github.com/crossterm-rs/crossterm) - Cross-platform terminal control
- [clap](https://github.com/clap-rs/clap) - Command line parsing

---

*Enjoy watching algorithms race! May the best sort win! 🏆*
# Release Commands for v0.1.0

## Pre-Release Checklist
- [x] Version set to 0.1.0 in Cargo.toml
- [x] Edition set to 2024
- [x] CHANGELOG.md created
- [x] All documentation updated
- [x] Binary builds successfully

## Git Commands

```bash
# 1. Ensure everything is committed
git add -A
git commit -m "chore: prepare v0.1.0 release

- Initial release of sorting-race
- 7 sorting algorithms with real-time visualization
- 4 fairness models for algorithm scheduling
- Interactive TUI with pause/resume/reset
- Memory usage tracking
- 98% test pass rate (112/114 tests)
"

# 2. Create and push tag
git tag -a v0.1.0 -m "Release v0.1.0 - Initial alpha release

First public release of sorting-race, a terminal-based visualization
tool for watching sorting algorithms race against each other.

Features:
- 7 sorting algorithms (Bubble, Heap, Insertion, Merge, Quick, Selection, Shell)
- 4 fairness models (Comparison, Weighted, WallTime, Adaptive)
- Real-time TUI visualization
- Memory usage tracking
- Interactive controls
"

# 3. Push to GitHub
git push origin main
git push origin v0.1.0
```

## Publish to crates.io

```bash
# 1. Dry run first
cargo publish --dry-run

# 2. If dry run succeeds, publish
cargo publish
```

## Create GitHub Release

After pushing the tag, go to:
https://github.com/yourusername/sorting-race/releases/new

1. Choose tag: v0.1.0
2. Release title: "v0.1.0 - Initial Alpha Release"
3. Check "This is a pre-release" (since it's v0.x)
4. Add release notes from CHANGELOG.md
5. The CI/CD will automatically build and attach binaries

## Post-Release

1. **Share on Reddit**:
   - r/rust - "Show HN: Sorting Race - Watch algorithms race in your terminal"
   - r/programming - Focus on the visualization aspect

2. **Update README** with installation instructions:
   ```bash
   cargo install sorting-race
   ```

3. **Create demo GIF** showing the race in action

## Version Roadmap

- v0.1.0 - Initial release (current)
- v0.2.0 - Export features (CSV, JSON)
- v0.3.0 - Additional algorithms (Radix, Tim Sort)
- v0.4.0 - Educational mode
- v0.5.0 - Performance optimizations
- v1.0.0 - Stable API, full feature set
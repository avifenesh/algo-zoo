# Memory Usage Display Fix

## Problem
The memory usage graphs in the UI were empty (showing 0 bytes) for all sorting algorithms.

## Root Cause
1. All sorting algorithms had `get_memory_usage()` methods but were returning hardcoded `0` or minimal values
2. In `get_telemetry()`, algorithms were setting `memory_current: 0` instead of calling `self.get_memory_usage()`
3. The memory calculation didn't include the actual data array size

## Solution Applied

### 1. Fixed Telemetry Reporting
Changed all algorithms from:
```rust
memory_current: 0, // O(1) space
memory_peak: 0,
```

To:
```rust
memory_current: self.get_memory_usage(),
memory_peak: self.get_memory_usage(),
```

### 2. Fixed Memory Calculation
Updated `get_memory_usage()` in all algorithms to report actual memory:

**Simple algorithms** (Bubble, Heap, Insertion, Selection, Shell):
```rust
fn get_memory_usage(&self) -> usize {
    // Report size of the data array in bytes
    self.data.len() * std::mem::size_of::<i32>()
}
```

**Merge Sort** (uses additional temp buffer):
```rust
fn get_memory_usage(&self) -> usize {
    // Data array + temp buffer + stack
    self.data.len() * std::mem::size_of::<i32>() + self.memory_usage
}
```

**Quick Sort** (uses recursion stack):
```rust
fn get_memory_usage(&self) -> usize {
    // Data array + stack memory
    self.data.len() * std::mem::size_of::<i32>() + 
    self.stack.len() * std::mem::size_of::<StackFrame>()
}
```

## Verification
Created tests to verify memory reporting:
- All algorithms now report > 0 memory usage
- Memory scales with data size
- Merge Sort correctly reports extra memory for temp buffer
- Quick Sort includes stack memory

## Result
✅ Memory usage graphs in the UI now display actual memory consumption
✅ Different algorithms show different memory patterns:
  - O(1) algorithms: Show only data array size
  - Merge Sort: Shows 2x data size (array + temp buffer)
  - Quick Sort: Shows data + variable stack usage
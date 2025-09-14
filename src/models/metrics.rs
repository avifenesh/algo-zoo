//! Performance metrics tracking for sorting algorithms

/// Comprehensive metrics for algorithm performance
#[derive(Debug, Clone, Default)]
pub struct Metrics {
    /// Total number of comparisons performed
    pub comparisons: u64,
    /// Total number of element moves/swaps performed
    pub moves: u64,
    /// Total execution time in microseconds
    pub execution_time_us: u64,
    /// Peak memory usage in bytes
    pub peak_memory_bytes: usize,
    /// Current memory usage in bytes
    pub current_memory_bytes: usize,
    /// Number of algorithm steps executed
    pub steps: usize,
    /// Array access count (reads + writes)
    pub array_accesses: u64,
    /// Number of recursive calls made
    pub recursive_calls: u64,
}

impl Metrics {
    /// Create a new metrics instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Reset all metrics to zero
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Add comparison count
    pub fn add_comparisons(&mut self, count: u64) {
        self.comparisons += count;
    }

    /// Add move count
    pub fn add_moves(&mut self, count: u64) {
        self.moves += count;
    }

    /// Add execution time
    pub fn add_time(&mut self, microseconds: u64) {
        self.execution_time_us += microseconds;
    }

    /// Update memory usage
    pub fn update_memory(&mut self, current: usize) {
        self.current_memory_bytes = current;
        self.peak_memory_bytes = self.peak_memory_bytes.max(current);
    }

    /// Increment step counter
    pub fn increment_steps(&mut self) {
        self.steps += 1;
    }

    /// Add array access count
    pub fn add_array_accesses(&mut self, count: u64) {
        self.array_accesses += count;
    }

    /// Add recursive call count
    pub fn add_recursive_calls(&mut self, count: u64) {
        self.recursive_calls += count;
    }

    /// Calculate operations per second
    pub fn ops_per_second(&self) -> f64 {
        if self.execution_time_us == 0 {
            0.0
        } else {
            let total_ops = self.comparisons + self.moves;
            (total_ops as f64 * 1_000_000.0) / self.execution_time_us as f64
        }
    }

    /// Calculate average time per step
    pub fn avg_step_time_us(&self) -> f64 {
        if self.steps == 0 {
            0.0
        } else {
            self.execution_time_us as f64 / self.steps as f64
        }
    }

    /// Get memory efficiency ratio
    pub fn memory_efficiency(&self) -> f64 {
        if self.peak_memory_bytes == 0 {
            1.0
        } else {
            self.current_memory_bytes as f64 / self.peak_memory_bytes as f64
        }
    }
}

/// Snapshot of metrics at a point in time
#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    /// The metrics data
    pub metrics: Metrics,
    /// Timestamp when snapshot was taken
    pub timestamp_us: u64,
    /// Algorithm name
    pub algorithm_name: String,
}

impl MetricsSnapshot {
    /// Create a new metrics snapshot
    pub fn new(metrics: Metrics, timestamp_us: u64, algorithm_name: String) -> Self {
        Self {
            metrics,
            timestamp_us,
            algorithm_name,
        }
    }
}
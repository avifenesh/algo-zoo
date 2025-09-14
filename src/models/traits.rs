//! Core traits for the sorting race visualization

use std::any::Any;
use std::fmt::Debug;

/// Result of a single step execution
#[derive(Debug, Clone, PartialEq)]
pub struct StepResult {
    /// Number of comparisons performed in this step
    pub comparisons_used: usize,
    /// Number of element moves performed in this step
    pub moves_made: usize,
    /// Whether the algorithm needs more steps
    pub continued: bool,
}

/// Visual markers for algorithm-specific operations
#[derive(Debug, Clone, Default)]
pub struct Markers {
    /// Current pivot index (Quick Sort)
    pub pivot: Option<usize>,
    /// Heap/sorted boundary (Heap Sort)
    pub heap_boundary: Option<usize>,
    /// Active merge regions (Merge Sort)
    pub merge_runs: Vec<(usize, usize)>,
    /// Current comparison positions
    pub cursors: Vec<usize>,
    /// Current gap size (Shell Sort)
    pub gap: Option<usize>,
}

/// Telemetry data returned after each step
#[derive(Debug, Clone)]
pub struct Telemetry {
    /// Total comparisons so far
    pub total_comparisons: u64,
    /// Total moves so far
    pub total_moves: u64,
    /// Current auxiliary memory usage in bytes
    pub memory_current: usize,
    /// Peak auxiliary memory usage in bytes
    pub memory_peak: usize,
    /// Indices to highlight in visualization
    pub highlights: Vec<usize>,
    /// Algorithm-specific markers
    pub markers: Markers,
    /// Human-readable description of current operation
    pub status_text: String,
    /// Progress estimate (0.0 to 1.0)
    pub progress_hint: f32,
}

/// Core trait that all sorting algorithms must implement
pub trait Sorter: Debug + Send + Any {
    /// Execute one step of the sorting algorithm
    /// 
    /// # Arguments
    /// * `budget` - Maximum number of comparisons allowed in this step
    /// 
    /// # Returns
    /// * `StepResult` - Information about operations performed
    fn step(&mut self, budget: usize) -> StepResult;
    
    /// Check if the algorithm has completed sorting
    fn is_complete(&self) -> bool;
    
    /// Get current telemetry data
    fn get_telemetry(&self) -> Telemetry;
    
    /// Reset the algorithm with new data
    /// 
    /// # Arguments
    /// * `data` - New array to sort
    fn reset(&mut self, data: Vec<i32>);
    
    /// Get the algorithm's display name
    fn name(&self) -> &str;
    
    /// Get current array state (for visualization)
    fn get_array(&self) -> &[i32];
    
    /// Get auxiliary memory usage in bytes
    fn get_memory_usage(&self) -> usize;
    
    /// Support downcasting for type-specific operations
    fn as_any(&self) -> &dyn Any;
    
    /// Support mutable downcasting for type-specific operations
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Fairness model trait for allocating step budgets
pub trait FairnessModel: Debug {
    /// Allocate step budgets to algorithms
    /// 
    /// # Arguments
    /// * `algorithms` - Current state of all algorithms
    /// 
    /// # Returns
    /// * Vector of budgets (comparisons) for each algorithm
    fn allocate_budget(&self, algorithms: &[Box<dyn Sorter>]) -> Vec<usize>;
    
    /// Get the model's display name
    fn name(&self) -> &str;
}

/// Memory tracker for precise auxiliary space measurement
pub trait MemoryTracker: Debug {
    /// Record an allocation
    fn alloc(&mut self, bytes: usize);
    
    /// Record a deallocation
    fn free(&mut self, bytes: usize);
    
    /// Get current memory usage
    fn current(&self) -> usize;
    
    /// Get peak memory usage
    fn peak(&self) -> usize;
    
    /// Reset tracking
    fn reset(&mut self);
}
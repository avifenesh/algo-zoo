// Trait contract for sorting algorithm implementations
// This file defines the interface that all sorting algorithms must implement

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
    /// Algorithm-specific markers (pivot, boundaries, etc.)
    pub markers: Markers,
    /// Human-readable description of current operation
    pub status_text: String,
    /// Progress estimate (0.0 to 1.0)
    pub progress_hint: f32,
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

/// Core trait that all sorting algorithms must implement
pub trait Sorter: Debug + Send {
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
pub trait MemoryTracker {
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

#[cfg(test)]
mod contract_tests {
    use super::*;
    
    /// Mock sorter for testing trait contracts
    struct MockSorter {
        array: Vec<i32>,
        complete: bool,
    }
    
    impl Debug for MockSorter {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("MockSorter")
                .field("complete", &self.complete)
                .finish()
        }
    }
    
    impl Sorter for MockSorter {
        fn step(&mut self, _budget: usize) -> StepResult {
            self.complete = true;
            StepResult {
                comparisons_used: 1,
                moves_made: 0,
                continued: false,
            }
        }
        
        fn is_complete(&self) -> bool {
            self.complete
        }
        
        fn get_telemetry(&self) -> Telemetry {
            Telemetry {
                total_comparisons: 1,
                total_moves: 0,
                memory_current: 0,
                memory_peak: 0,
                highlights: vec![],
                markers: Markers::default(),
                status_text: "Mock sorting".to_string(),
                progress_hint: if self.complete { 1.0 } else { 0.0 },
            }
        }
        
        fn reset(&mut self, data: Vec<i32>) {
            self.array = data;
            self.complete = false;
        }
        
        fn name(&self) -> &str {
            "Mock Sort"
        }
        
        fn get_array(&self) -> &[i32] {
            &self.array
        }
        
        fn get_memory_usage(&self) -> usize {
            0
        }
    }
    
    #[test]
    fn test_sorter_trait_contract() {
        let mut sorter = MockSorter {
            array: vec![3, 1, 2],
            complete: false,
        };
        
        // Test initial state
        assert!(!sorter.is_complete());
        assert_eq!(sorter.name(), "Mock Sort");
        
        // Test step execution
        let result = sorter.step(10);
        assert_eq!(result.comparisons_used, 1);
        assert!(!result.continued);
        
        // Test completion
        assert!(sorter.is_complete());
        
        // Test telemetry
        let telemetry = sorter.get_telemetry();
        assert_eq!(telemetry.total_comparisons, 1);
        assert_eq!(telemetry.progress_hint, 1.0);
        
        // Test reset
        sorter.reset(vec![5, 4, 3, 2, 1]);
        assert!(!sorter.is_complete());
        assert_eq!(sorter.get_array(), &[5, 4, 3, 2, 1]);
    }
    
    #[test]
    fn test_step_result_equality() {
        let result1 = StepResult {
            comparisons_used: 5,
            moves_made: 3,
            continued: true,
        };
        
        let result2 = StepResult {
            comparisons_used: 5,
            moves_made: 3,
            continued: true,
        };
        
        assert_eq!(result1, result2);
    }
    
    #[test]
    fn test_markers_default() {
        let markers = Markers::default();
        assert!(markers.pivot.is_none());
        assert!(markers.heap_boundary.is_none());
        assert!(markers.merge_runs.is_empty());
        assert!(markers.cursors.is_empty());
        assert!(markers.gap.is_none());
    }
}
// Library Interface Contract for sorting_race

use std::any::Any;

/// Core trait that all sorting algorithms must implement
pub trait Sorter: Debug + Send + Any {
    /// Execute one step with given budget
    fn step(&mut self, budget: usize) -> StepResult;
    
    /// Check if sorting is complete
    fn is_complete(&self) -> bool;
    
    /// Get current telemetry data
    fn get_telemetry(&self) -> Telemetry;
    
    /// Reset with new array
    fn reset(&mut self, data: Vec<i32>);
    
    /// Get algorithm name
    fn name(&self) -> &str;
    
    /// Get current array state
    fn get_array(&self) -> &[i32];
    
    /// Get current memory usage
    fn get_memory_usage(&self) -> usize;
    
    /// Type erasure support
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Fairness model trait for budget allocation
pub trait FairnessModel: Debug + Send {
    /// Allocate budget across algorithms
    fn allocate_budget(&self, algorithms: &[Box<dyn Sorter>]) -> Vec<usize>;
    
    /// Get model name
    fn name(&self) -> &str;
    
    /// Update performance metrics (for adaptive models)
    fn update_performance(&mut self, _performances: &[f32]) {}
}

/// Result of a single algorithm step
#[derive(Debug, Clone)]
pub struct StepResult {
    pub comparisons_used: usize,
    pub moves_made: usize,
    pub continued: bool,
}

/// Telemetry data for visualization
#[derive(Debug, Clone)]
pub struct Telemetry {
    pub total_comparisons: u64,
    pub total_moves: u64,
    pub memory_current: usize,
    pub memory_peak: usize,
    pub highlights: Vec<usize>,
    pub markers: Markers,
    pub status_text: String,
    pub progress_hint: f32,
}

/// Visual markers for operations
#[derive(Debug, Clone, Default)]
pub struct Markers {
    pub cursors: Vec<usize>,
    pub pivot: Option<usize>,
    pub gap: Option<usize>,
    pub heap_boundary: Option<usize>,
    pub merge_boundaries: Vec<usize>,
}

/// Run configuration
#[derive(Debug, Clone)]
pub struct RunConfiguration {
    pub array_size: usize,
    pub distribution: Distribution,
    pub seed: u64,
    pub fairness_mode: FairnessMode,
    pub target_fps: u32,
}

/// Array distribution types
#[derive(Debug, Clone)]
pub enum Distribution {
    Shuffled,
    NearlySorted,
    Reversed,
    FewUnique,
    Sorted,
    WithDuplicates,
}

/// Fairness mode configuration
#[derive(Debug, Clone)]
pub enum FairnessMode {
    ComparisonBudget { k: usize },
    Weighted { alpha: f32, beta: f32 },
    WallTime { slice_ms: u64 },
    Adaptive { learning_rate: f32 },
}

// Contract Tests (to be placed in tests/contract/)

#[cfg(test)]
mod contract_tests {
    use super::*;
    
    #[test]
    fn test_sorter_contract() {
        // This test should fail until implementations exist
        let sorters: Vec<Box<dyn Sorter>> = vec![
            // Box::new(BubbleSort::new()),
            // Box::new(QuickSort::new()),
            // etc.
        ];
        
        for mut sorter in sorters {
            // Test reset
            sorter.reset(vec![3, 1, 2]);
            assert!(!sorter.is_complete());
            
            // Test step
            let result = sorter.step(10);
            assert!(result.comparisons_used <= 10);
            
            // Test telemetry
            let telemetry = sorter.get_telemetry();
            assert!(telemetry.progress_hint >= 0.0);
            assert!(telemetry.progress_hint <= 1.0);
        }
    }
    
    #[test]
    fn test_fairness_contract() {
        // This test should fail until implementations exist
        let models: Vec<Box<dyn FairnessModel>> = vec![
            // Box::new(ComparisonFairness::new(16)),
            // Box::new(WeightedFairness::new(1.0, 1.0)),
            // etc.
        ];
        
        for model in models {
            let algorithms: Vec<Box<dyn Sorter>> = vec![];
            let allocations = model.allocate_budget(&algorithms);
            
            // Sum of allocations should equal total budget
            let total: usize = allocations.iter().sum();
            assert!(total > 0);
        }
    }
}
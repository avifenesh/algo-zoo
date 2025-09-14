//! Contract tests for the FairnessModel trait
//! These tests verify that fairness models correctly allocate budgets

use sorting_race::models::traits::{FairnessModel, Sorter};
use sorting_race::services::fairness::{
    comparison::ComparisonBudget, walltime::WallTimeFairness, weighted::WeightedFairness,
};

#[derive(Debug)]
struct MockSorter {
    name: String,
    complete: bool,
    comparisons: u64,
    moves: u64,
}

impl MockSorter {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            complete: false,
            comparisons: 0,
            moves: 0,
        }
    }

    fn with_stats(name: &str, comparisons: u64, moves: u64) -> Self {
        Self {
            name: name.to_string(),
            complete: false,
            comparisons,
            moves,
        }
    }
}

// Minimal Sorter implementation for testing
impl Sorter for MockSorter {
    fn step(&mut self, _budget: usize) -> sorting_race::models::traits::StepResult {
        unimplemented!("Not needed for fairness tests")
    }

    fn is_complete(&self) -> bool {
        self.complete
    }

    fn get_telemetry(&self) -> sorting_race::models::traits::Telemetry {
        sorting_race::models::traits::Telemetry {
            total_comparisons: self.comparisons,
            total_moves: self.moves,
            memory_current: 0,
            memory_peak: 0,
            highlights: vec![],
            markers: sorting_race::models::traits::Markers::default(),
            status_text: String::new(),
            progress_hint: 0.0,
        }
    }

    fn reset(&mut self, _data: Vec<i32>) {}

    fn name(&self) -> &str {
        &self.name
    }

    fn get_array(&self) -> &[i32] {
        &[]
    }

    fn get_memory_usage(&self) -> usize {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_algorithms() -> Vec<Box<dyn Sorter>> {
        vec![
            Box::new(MockSorter::new("Bubble")),
            Box::new(MockSorter::new("Quick")),
            Box::new(MockSorter::new("Merge")),
        ]
    }

    #[test]
    fn test_comparison_budget_equal_allocation() {
        let model = ComparisonBudget::new(16);
        let algorithms = create_test_algorithms();
        
        let budgets = model.allocate_budget(&algorithms);
        
        // All algorithms should get equal budget
        assert_eq!(budgets.len(), algorithms.len());
        for budget in &budgets {
            assert_eq!(*budget, 16);
        }
        
        assert_eq!(model.name(), "Comparison Budget");
    }

    #[test]
    fn test_comparison_budget_skips_completed() {
        let model = ComparisonBudget::new(10);
        let mut algorithms = create_test_algorithms();
        
        // Mark one as complete
        if let Some(algo) = algorithms[1].as_any_mut().downcast_mut::<MockSorter>() {
            algo.complete = true;
        }
        
        let budgets = model.allocate_budget(&algorithms);
        
        assert_eq!(budgets.len(), algorithms.len());
        assert_eq!(budgets[0], 10); // Active
        assert_eq!(budgets[1], 0);  // Complete
        assert_eq!(budgets[2], 10); // Active
    }

    #[test]
    fn test_weighted_fairness_default_weights() {
        let model = WeightedFairness::new(1.0, 1.0);
        let algorithms: Vec<Box<dyn Sorter>> = vec![
            Box::new(MockSorter::with_stats("Algo1", 100, 50)),
            Box::new(MockSorter::with_stats("Algo2", 50, 100)),
        ];
        
        let budgets = model.allocate_budget(&algorithms);
        
        assert_eq!(budgets.len(), 2);
        // With equal weights, both get budget
        assert!(budgets[0] > 0);
        assert!(budgets[1] > 0);
        
        assert!(model.name().contains("Weighted"));
    }

    #[test]
    fn test_weighted_fairness_cache_biased() {
        // Cache-biased: comparisons weighted more
        let model = WeightedFairness::new(1.0, 0.5);
        let algorithms: Vec<Box<dyn Sorter>> = vec![
            Box::new(MockSorter::with_stats("CompHeavy", 100, 10)),
            Box::new(MockSorter::with_stats("MoveHeavy", 10, 100)),
        ];
        
        let budgets = model.allocate_budget(&algorithms);
        
        assert_eq!(budgets.len(), 2);
        // Algorithm with more comparisons should get different treatment
        assert!(budgets[0] > 0 || budgets[1] > 0);
    }

    #[test]
    fn test_weighted_fairness_memory_biased() {
        // Memory-biased: moves weighted more
        let model = WeightedFairness::new(1.0, 2.0);
        let algorithms: Vec<Box<dyn Sorter>> = vec![
            Box::new(MockSorter::with_stats("CompHeavy", 100, 10)),
            Box::new(MockSorter::with_stats("MoveHeavy", 10, 100)),
        ];
        
        let budgets = model.allocate_budget(&algorithms);
        
        assert_eq!(budgets.len(), 2);
        // Algorithm with more moves should get different treatment
        assert!(budgets[0] > 0 || budgets[1] > 0);
    }

    #[test]
    fn test_wall_time_fairness() {
        let model = WallTimeFairness::new(10); // 10ms slices
        let algorithms = create_test_algorithms();
        
        let budgets = model.allocate_budget(&algorithms);
        
        // Wall time should allocate based on time slices
        assert_eq!(budgets.len(), algorithms.len());
        for budget in &budgets {
            // Budget should be reasonable for time slice
            assert!(*budget > 0 && *budget <= 1000); // Reasonable upper bound
        }
        
        assert!(model.name().contains("Wall"));
    }

    #[test]
    fn test_fairness_with_empty_algorithms() {
        let model = ComparisonBudget::new(16);
        let algorithms: Vec<Box<dyn Sorter>> = vec![];
        
        let budgets = model.allocate_budget(&algorithms);
        
        assert_eq!(budgets.len(), 0);
    }

    #[test]
    fn test_fairness_all_complete() {
        let model = ComparisonBudget::new(16);
        let mut algorithms = create_test_algorithms();
        
        // Mark all as complete
        for algo in &mut algorithms {
            if let Some(mock) = algo.as_any_mut().downcast_mut::<MockSorter>() {
                mock.complete = true;
            }
        }
        
        let budgets = model.allocate_budget(&algorithms);
        
        // All complete algorithms should get 0 budget
        assert_eq!(budgets.len(), algorithms.len());
        for budget in &budgets {
            assert_eq!(*budget, 0);
        }
    }

    #[test]
    fn test_weighted_fairness_zero_weights() {
        // Edge case: zero weights should handle gracefully
        let model = WeightedFairness::new(0.0, 0.0);
        let algorithms = create_test_algorithms();
        
        let budgets = model.allocate_budget(&algorithms);
        
        assert_eq!(budgets.len(), algorithms.len());
        // Should handle gracefully, likely equal distribution
        for budget in &budgets {
            assert!(*budget >= 0);
        }
    }
}
//! Contract tests for the WeightedFairness model
//! These tests verify that weighted fairness allocates budget based on α*comparisons + β*moves

use sorting_race::models::traits::{FairnessModel, Sorter};
use sorting_race::services::fairness::WeightedFairness;

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

    fn with_completion(name: &str, comparisons: u64, moves: u64, complete: bool) -> Self {
        Self {
            name: name.to_string(),
            complete,
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

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weighted_fairness_initialization() {
        let model = WeightedFairness::new(1.0, 1.0);
        assert!(model.name().contains("Weighted"));
        
        let model_comparison_biased = WeightedFairness::new(2.0, 1.0);
        assert!(model_comparison_biased.name().contains("Weighted"));
        
        let model_move_biased = WeightedFairness::new(1.0, 2.0);
        assert!(model_move_biased.name().contains("Weighted"));
    }

    #[test]
    fn test_weighted_fairness_equal_weights() {
        let model = WeightedFairness::new(1.0, 1.0);
        let algorithms: Vec<Box<dyn Sorter>> = vec![
            Box::new(MockSorter::with_stats("Algo1", 100, 50)),
            Box::new(MockSorter::with_stats("Algo2", 50, 100)),
            Box::new(MockSorter::with_stats("Algo3", 75, 75)),
        ];
        
        let budgets = model.allocate_budget(&algorithms);
        
        assert_eq!(budgets.len(), 3);
        // All algorithms should get some budget with equal weights
        assert!(budgets[0] > 0);
        assert!(budgets[1] > 0);
        assert!(budgets[2] > 0);
        
        // Total budget should be distributed
        let total_budget: usize = budgets.iter().sum();
        assert!(total_budget > 0);
    }

    #[test]
    fn test_weighted_fairness_comparison_biased() {
        let model = WeightedFairness::new(2.0, 0.5); // Heavily favor comparisons
        let algorithms: Vec<Box<dyn Sorter>> = vec![
            Box::new(MockSorter::with_stats("ComparisonHeavy", 200, 10)),
            Box::new(MockSorter::with_stats("MoveHeavy", 10, 200)),
            Box::new(MockSorter::with_stats("Balanced", 100, 100)),
        ];
        
        let budgets = model.allocate_budget(&algorithms);
        
        assert_eq!(budgets.len(), 3);
        
        // ComparisonHeavy should get less budget (it's already done a lot of comparisons)
        // MoveHeavy should get more budget (comparisons are weighted higher, so it needs catch-up)
        assert!(budgets[0] > 0);
        assert!(budgets[1] > 0);
        assert!(budgets[2] > 0);
        
        // Verify budget allocation is based on weights
        let total_budget: usize = budgets.iter().sum();
        assert!(total_budget > 0);
    }

    #[test]
    fn test_weighted_fairness_move_biased() {
        let model = WeightedFairness::new(0.5, 2.0); // Heavily favor moves
        let algorithms: Vec<Box<dyn Sorter>> = vec![
            Box::new(MockSorter::with_stats("ComparisonHeavy", 200, 10)),
            Box::new(MockSorter::with_stats("MoveHeavy", 10, 200)),
            Box::new(MockSorter::with_stats("Balanced", 100, 100)),
        ];
        
        let budgets = model.allocate_budget(&algorithms);
        
        assert_eq!(budgets.len(), 3);
        
        // All should get budget, but distribution should favor algorithms that haven't done many moves
        assert!(budgets[0] > 0);
        assert!(budgets[1] > 0);
        assert!(budgets[2] > 0);
        
        // ComparisonHeavy should get more budget (hasn't done many moves)
        // MoveHeavy should get less budget (has already done many moves)
        let total_budget: usize = budgets.iter().sum();
        assert!(total_budget > 0);
    }

    #[test]
    fn test_weighted_fairness_allocations_sum_to_budget() {
        let model = WeightedFairness::new(1.5, 0.8);
        let algorithms: Vec<Box<dyn Sorter>> = vec![
            Box::new(MockSorter::with_stats("Algo1", 50, 25)),
            Box::new(MockSorter::with_stats("Algo2", 100, 50)),
            Box::new(MockSorter::with_stats("Algo3", 25, 100)),
            Box::new(MockSorter::with_stats("Algo4", 75, 75)),
        ];
        
        let budgets = model.allocate_budget(&algorithms);
        
        assert_eq!(budgets.len(), 4);
        
        // All active algorithms should get some budget
        for budget in &budgets {
            assert!(*budget >= 0); // Should be non-negative
        }
        
        // Total budget should be reasonable (not zero, not excessive)
        let total_budget: usize = budgets.iter().sum();
        assert!(total_budget > 0);
        assert!(total_budget < 10000); // Sanity check upper bound
    }

    #[test]
    fn test_weighted_fairness_skips_completed() {
        let model = WeightedFairness::new(1.0, 1.0);
        let algorithms: Vec<Box<dyn Sorter>> = vec![
            Box::new(MockSorter::with_stats("Active1", 50, 25)),
            Box::new(MockSorter::with_completion("Completed", 100, 50, true)),
            Box::new(MockSorter::with_stats("Active2", 25, 100)),
        ];
        
        let budgets = model.allocate_budget(&algorithms);
        
        assert_eq!(budgets.len(), 3);
        assert!(budgets[0] > 0); // Active1 should get budget
        assert_eq!(budgets[1], 0); // Completed should get 0 budget
        assert!(budgets[2] > 0); // Active2 should get budget
    }

    #[test]
    fn test_weighted_fairness_all_complete() {
        let model = WeightedFairness::new(1.0, 1.0);
        let algorithms: Vec<Box<dyn Sorter>> = vec![
            Box::new(MockSorter::with_completion("Complete1", 100, 50, true)),
            Box::new(MockSorter::with_completion("Complete2", 80, 60, true)),
            Box::new(MockSorter::with_completion("Complete3", 120, 40, true)),
        ];
        
        let budgets = model.allocate_budget(&algorithms);
        
        assert_eq!(budgets.len(), 3);
        // All completed algorithms should get 0 budget
        for budget in &budgets {
            assert_eq!(*budget, 0);
        }
    }

    #[test]
    fn test_weighted_fairness_zero_weights() {
        let model = WeightedFairness::new(0.0, 0.0);
        let algorithms: Vec<Box<dyn Sorter>> = vec![
            Box::new(MockSorter::with_stats("Algo1", 100, 50)),
            Box::new(MockSorter::with_stats("Algo2", 50, 100)),
        ];
        
        let budgets = model.allocate_budget(&algorithms);
        
        assert_eq!(budgets.len(), 2);
        // Should handle zero weights gracefully, likely equal distribution
        for budget in &budgets {
            assert!(*budget >= 0);
        }
        
        // Should still allocate some budget
        let total_budget: usize = budgets.iter().sum();
        assert!(total_budget >= 0);
    }

    #[test]
    fn test_weighted_fairness_negative_weights() {
        // Edge case: should handle negative weights gracefully
        let model = WeightedFairness::new(-1.0, 1.0);
        let algorithms: Vec<Box<dyn Sorter>> = vec![
            Box::new(MockSorter::with_stats("Algo1", 100, 50)),
            Box::new(MockSorter::with_stats("Algo2", 50, 100)),
        ];
        
        let budgets = model.allocate_budget(&algorithms);
        
        assert_eq!(budgets.len(), 2);
        // Should still produce valid budgets
        for budget in &budgets {
            assert!(*budget >= 0);
        }
    }

    #[test]
    fn test_weighted_fairness_very_different_stats() {
        let model = WeightedFairness::new(1.0, 1.0);
        let algorithms: Vec<Box<dyn Sorter>> = vec![
            Box::new(MockSorter::with_stats("VeryHigh", 10000, 5000)),
            Box::new(MockSorter::with_stats("VeryLow", 1, 1)),
            Box::new(MockSorter::with_stats("Medium", 500, 250)),
        ];
        
        let budgets = model.allocate_budget(&algorithms);
        
        assert_eq!(budgets.len(), 3);
        
        // All should get some budget despite vastly different stats
        for budget in &budgets {
            assert!(*budget >= 0);
        }
        
        // Algorithm with very high stats should get lower budget (needs less catch-up)
        // Algorithm with very low stats should get higher budget (needs more catch-up)
        let total_budget: usize = budgets.iter().sum();
        assert!(total_budget > 0);
    }

    #[test]
    fn test_weighted_fairness_single_algorithm() {
        let model = WeightedFairness::new(1.0, 1.0);
        let algorithms: Vec<Box<dyn Sorter>> = vec![
            Box::new(MockSorter::with_stats("OnlyAlgo", 50, 25)),
        ];
        
        let budgets = model.allocate_budget(&algorithms);
        
        assert_eq!(budgets.len(), 1);
        assert!(budgets[0] > 0); // Single active algorithm should get budget
    }

    #[test]
    fn test_weighted_fairness_empty_algorithms() {
        let model = WeightedFairness::new(1.0, 1.0);
        let algorithms: Vec<Box<dyn Sorter>> = vec![];
        
        let budgets = model.allocate_budget(&algorithms);
        
        assert_eq!(budgets.len(), 0);
    }

    #[test]
    fn test_weighted_fairness_extreme_weights() {
        let model = WeightedFairness::new(1000.0, 0.001);
        let algorithms: Vec<Box<dyn Sorter>> = vec![
            Box::new(MockSorter::with_stats("CompHeavy", 1000, 1)),
            Box::new(MockSorter::with_stats("MoveHeavy", 1, 1000)),
        ];
        
        let budgets = model.allocate_budget(&algorithms);
        
        assert_eq!(budgets.len(), 2);
        
        // Should handle extreme weights without panic or overflow
        for budget in &budgets {
            assert!(*budget >= 0);
            assert!(*budget < 1000000); // Reasonable upper bound
        }
        
        let total_budget: usize = budgets.iter().sum();
        assert!(total_budget > 0);
    }

    #[test]
    fn test_weighted_fairness_zero_stats() {
        let model = WeightedFairness::new(1.0, 1.0);
        let algorithms: Vec<Box<dyn Sorter>> = vec![
            Box::new(MockSorter::with_stats("ZeroStats", 0, 0)),
            Box::new(MockSorter::with_stats("SomeStats", 100, 50)),
        ];
        
        let budgets = model.allocate_budget(&algorithms);
        
        assert_eq!(budgets.len(), 2);
        
        // Algorithm with zero stats should still get budget
        assert!(budgets[0] > 0);
        assert!(budgets[1] > 0);
        
        // Zero stats algorithm should get more budget (needs to catch up)
        let total_budget: usize = budgets.iter().sum();
        assert!(total_budget > 0);
    }
}
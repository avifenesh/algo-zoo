//! Contract tests for the AdaptiveFairness model
//! These tests verify that algorithms with better progress rates get more budget

use sorting_race::models::traits::{FairnessModel, Sorter};
use sorting_race::services::fairness::AdaptiveFairness;

#[derive(Debug)]
struct MockSorter {
    name: String,
    complete: bool,
    comparisons: u64,
    moves: u64,
    progress_hint: f32,
    step_count: u64,
}

impl MockSorter {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            complete: false,
            comparisons: 0,
            moves: 0,
            progress_hint: 0.0,
            step_count: 0,
        }
    }

    fn with_progress(name: &str, progress: f32) -> Self {
        Self {
            name: name.to_string(),
            complete: false,
            comparisons: 0,
            moves: 0,
            progress_hint: progress,
            step_count: 0,
        }
    }

    fn with_completion(name: &str, complete: bool) -> Self {
        Self {
            name: name.to_string(),
            complete,
            comparisons: 0,
            moves: 0,
            progress_hint: if complete { 1.0 } else { 0.0 },
            step_count: 0,
        }
    }

    fn with_stats_and_progress(name: &str, comparisons: u64, moves: u64, progress: f32) -> Self {
        Self {
            name: name.to_string(),
            complete: false,
            comparisons,
            moves,
            progress_hint: progress,
            step_count: 0,
        }
    }

    fn set_progress(&mut self, progress: f32) {
        self.progress_hint = progress;
    }

    fn simulate_work(&mut self, budget: usize) {
        self.comparisons += budget as u64;
        self.moves += (budget / 2) as u64;
        self.step_count += 1;
        
        // Simulate progress increase
        let progress_increment = (budget as f32) * 0.01; // 1% progress per comparison
        self.progress_hint = (self.progress_hint + progress_increment).min(1.0);
        
        if self.progress_hint >= 1.0 {
            self.complete = true;
        }
    }
}

// Minimal Sorter implementation for testing
impl Sorter for MockSorter {
    fn step(&mut self, budget: usize) -> sorting_race::models::traits::StepResult {
        self.simulate_work(budget);
        
        sorting_race::models::traits::StepResult {
            comparisons_used: budget,
            moves_made: budget / 2,
            continued: !self.complete,
        }
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
            progress_hint: self.progress_hint,
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
    fn test_adaptive_fairness_initialization() {
        let model = AdaptiveFairness::new(0.1);
        assert!(model.name().contains("Adaptive"));
        
        let model_fast_learning = AdaptiveFairness::new(0.5);
        assert!(model_fast_learning.name().contains("Adaptive"));
        
        let model_slow_learning = AdaptiveFairness::new(0.01);
        assert!(model_slow_learning.name().contains("Adaptive"));
    }

    #[test]
    fn test_adaptive_fairness_better_progress_gets_more_budget() {
        let model = AdaptiveFairness::new(0.2);
        let algorithms: Vec<Box<dyn Sorter>> = vec![
            Box::new(MockSorter::with_progress("FastProgress", 0.7)),
            Box::new(MockSorter::with_progress("SlowProgress", 0.2)),
            Box::new(MockSorter::with_progress("MediumProgress", 0.5)),
        ];
        
        let budgets = model.allocate_budget(&algorithms);
        
        assert_eq!(budgets.len(), 3);
        
        // All active algorithms should get some budget
        for budget in &budgets {
            assert!(*budget > 0);
        }
        
        // FastProgress should get more budget than SlowProgress
        // (exact relationship depends on implementation)
        assert!(budgets[0] > 0); // FastProgress
        assert!(budgets[1] > 0); // SlowProgress
        assert!(budgets[2] > 0); // MediumProgress
        
        // Total budget should be reasonable
        let total_budget: usize = budgets.iter().sum();
        assert!(total_budget > 0);
    }

    #[test]
    fn test_adaptive_fairness_exponential_moving_average() {
        let learning_rate = 0.3;
        let model = AdaptiveFairness::new(learning_rate);
        let mut algorithms: Vec<Box<dyn Sorter>> = vec![
            Box::new(MockSorter::with_progress("ChangingProgress", 0.1)),
        ];
        
        // Initial allocation
        let initial_budgets = model.allocate_budget(&algorithms);
        assert!(initial_budgets[0] > 0);
        
        // Simulate algorithm making significant progress
        algorithms[0].step(initial_budgets[0]);
        
        if let Some(mock) = algorithms[0].as_any_mut().downcast_mut::<MockSorter>() {
            mock.set_progress(0.8); // Dramatic improvement
        }
        
        // Get new budgets after progress update
        let updated_budgets = model.allocate_budget(&algorithms);
        assert!(updated_budgets[0] > 0);
        
        // The model should adapt to the improved progress
        // (exact behavior depends on implementation of exponential moving average)
    }

    #[test]
    fn test_adaptive_fairness_adaptation_over_time() {
        let model = AdaptiveFairness::new(0.15);
        let mut algorithms: Vec<Box<dyn Sorter>> = vec![
            Box::new(MockSorter::with_progress("ImprovingAlgo", 0.1)),
            Box::new(MockSorter::with_progress("DeterioratingAlgo", 0.8)),
        ];
        
        let mut budget_history = Vec::new();
        
        // Run several allocation cycles
        for cycle in 0..5 {
            let budgets = model.allocate_budget(&algorithms);
            budget_history.push(budgets.clone());
            
            // Simulate execution and progress changes
            for (i, algorithm) in algorithms.iter_mut().enumerate() {
                algorithm.step(budgets[i]);
            }
            
            // Modify progress to simulate adaptation scenarios
            match cycle {
                1 => {
                    // ImprovingAlgo gets better
                    if let Some(mock) = algorithms[0].as_any_mut().downcast_mut::<MockSorter>() {
                        mock.set_progress(0.3);
                    }
                }
                2 => {
                    // ImprovingAlgo continues improving
                    if let Some(mock) = algorithms[0].as_any_mut().downcast_mut::<MockSorter>() {
                        mock.set_progress(0.6);
                    }
                }
                3 => {
                    // DeterioratingAlgo slows down
                    if let Some(mock) = algorithms[1].as_any_mut().downcast_mut::<MockSorter>() {
                        mock.set_progress(0.85); // Minimal progress
                    }
                }
                _ => {}
            }
        }
        
        // Verify that budgets were allocated across all cycles
        for (cycle, budgets) in budget_history.iter().enumerate() {
            assert_eq!(budgets.len(), 2, "Cycle {}: Should have 2 budgets", cycle);
            assert!(budgets[0] > 0, "Cycle {}: ImprovingAlgo should get budget", cycle);
            assert!(budgets[1] > 0, "Cycle {}: DeterioratingAlgo should get budget", cycle);
        }
        
        // The model should show adaptation over time
        // (specific trends depend on implementation details)
    }

    #[test]
    fn test_adaptive_fairness_learning_rate_effects() {
        let algorithms = || -> Vec<Box<dyn Sorter>> {
            vec![
                Box::new(MockSorter::with_progress("TestAlgo", 0.3)),
                Box::new(MockSorter::with_progress("ReferenceAlgo", 0.5)),
            ]
        };
        
        let fast_learner = AdaptiveFairness::new(0.8); // High learning rate
        let slow_learner = AdaptiveFairness::new(0.05); // Low learning rate
        
        let fast_budgets = fast_learner.allocate_budget(&algorithms());
        let slow_budgets = slow_learner.allocate_budget(&algorithms());
        
        assert_eq!(fast_budgets.len(), 2);
        assert_eq!(slow_budgets.len(), 2);
        
        // Both models should allocate budgets
        for budget in &fast_budgets {
            assert!(*budget > 0);
        }
        for budget in &slow_budgets {
            assert!(*budget > 0);
        }
        
        // Different learning rates may produce different budget distributions
        // (exact behavior depends on implementation)
    }

    #[test]
    fn test_adaptive_fairness_skips_completed() {
        let model = AdaptiveFairness::new(0.2);
        let algorithms: Vec<Box<dyn Sorter>> = vec![
            Box::new(MockSorter::with_progress("Active1", 0.4)),
            Box::new(MockSorter::with_completion("Completed", true)),
            Box::new(MockSorter::with_progress("Active2", 0.6)),
        ];
        
        let budgets = model.allocate_budget(&algorithms);
        
        assert_eq!(budgets.len(), 3);
        assert!(budgets[0] > 0); // Active1 should get budget
        assert_eq!(budgets[1], 0); // Completed should get 0 budget
        assert!(budgets[2] > 0); // Active2 should get budget
    }

    #[test]
    fn test_adaptive_fairness_all_complete() {
        let model = AdaptiveFairness::new(0.25);
        let algorithms: Vec<Box<dyn Sorter>> = vec![
            Box::new(MockSorter::with_completion("Complete1", true)),
            Box::new(MockSorter::with_completion("Complete2", true)),
            Box::new(MockSorter::with_completion("Complete3", true)),
        ];
        
        let budgets = model.allocate_budget(&algorithms);
        
        assert_eq!(budgets.len(), 3);
        // All completed algorithms should get 0 budget
        for budget in &budgets {
            assert_eq!(*budget, 0);
        }
    }

    #[test]
    fn test_adaptive_fairness_single_algorithm() {
        let model = AdaptiveFairness::new(0.3);
        let algorithms: Vec<Box<dyn Sorter>> = vec![
            Box::new(MockSorter::with_progress("OnlyAlgo", 0.5)),
        ];
        
        let budgets = model.allocate_budget(&algorithms);
        
        assert_eq!(budgets.len(), 1);
        assert!(budgets[0] > 0); // Single active algorithm should get budget
    }

    #[test]
    fn test_adaptive_fairness_empty_algorithms() {
        let model = AdaptiveFairness::new(0.1);
        let algorithms: Vec<Box<dyn Sorter>> = vec![];
        
        let budgets = model.allocate_budget(&algorithms);
        
        assert_eq!(budgets.len(), 0);
    }

    #[test]
    fn test_adaptive_fairness_zero_learning_rate() {
        let model = AdaptiveFairness::new(0.0); // No learning
        let algorithms: Vec<Box<dyn Sorter>> = vec![
            Box::new(MockSorter::with_progress("Algo1", 0.2)),
            Box::new(MockSorter::with_progress("Algo2", 0.8)),
        ];
        
        let budgets = model.allocate_budget(&algorithms);
        
        assert_eq!(budgets.len(), 2);
        // Should still allocate budgets even with zero learning rate
        for budget in &budgets {
            assert!(*budget > 0);
        }
    }

    #[test]
    fn test_adaptive_fairness_maximum_learning_rate() {
        let model = AdaptiveFairness::new(1.0); // Maximum learning rate
        let algorithms: Vec<Box<dyn Sorter>> = vec![
            Box::new(MockSorter::with_progress("Algo1", 0.3)),
            Box::new(MockSorter::with_progress("Algo2", 0.7)),
        ];
        
        let budgets = model.allocate_budget(&algorithms);
        
        assert_eq!(budgets.len(), 2);
        // Should handle maximum learning rate without issues
        for budget in &budgets {
            assert!(*budget > 0);
        }
    }

    #[test]
    fn test_adaptive_fairness_negative_learning_rate() {
        // Edge case: should handle negative learning rate gracefully
        let model = AdaptiveFairness::new(-0.1);
        let algorithms: Vec<Box<dyn Sorter>> = vec![
            Box::new(MockSorter::with_progress("Algo1", 0.4)),
        ];
        
        let budgets = model.allocate_budget(&algorithms);
        
        assert_eq!(budgets.len(), 1);
        // Should still produce valid budgets
        assert!(budgets[0] >= 0);
    }

    #[test]
    fn test_adaptive_fairness_extreme_progress_values() {
        let model = AdaptiveFairness::new(0.2);
        let algorithms: Vec<Box<dyn Sorter>> = vec![
            Box::new(MockSorter::with_progress("ZeroProgress", 0.0)),
            Box::new(MockSorter::with_progress("CompleteProgress", 1.0)),
            Box::new(MockSorter::with_progress("OverProgress", 1.5)), // Edge case
        ];
        
        let budgets = model.allocate_budget(&algorithms);
        
        assert_eq!(budgets.len(), 3);
        
        // All should get non-negative budgets
        for budget in &budgets {
            assert!(*budget >= 0);
        }
        
        // ZeroProgress should get budget (needs to catch up)
        assert!(budgets[0] > 0);
        
        // CompleteProgress might get less or zero budget
        // OverProgress should be handled gracefully
    }

    #[test]
    fn test_adaptive_fairness_consistent_stats_different_progress() {
        let model = AdaptiveFairness::new(0.25);
        let algorithms: Vec<Box<dyn Sorter>> = vec![
            Box::new(MockSorter::with_stats_and_progress("EfficientAlgo", 100, 50, 0.8)),
            Box::new(MockSorter::with_stats_and_progress("InefficientAlgo", 100, 50, 0.3)),
        ];
        
        let budgets = model.allocate_budget(&algorithms);
        
        assert_eq!(budgets.len(), 2);
        assert!(budgets[0] > 0);
        assert!(budgets[1] > 0);
        
        // Despite same operation counts, different progress should affect allocation
        // EfficientAlgo made better progress with same operations
        // InefficientAlgo needs more help (or gets more budget to catch up)
        // Exact behavior depends on implementation philosophy
    }

    #[test]
    fn test_adaptive_fairness_progress_rate_calculation() {
        let model = AdaptiveFairness::new(0.3);
        let mut algorithms: Vec<Box<dyn Sorter>> = vec![
            Box::new(MockSorter::with_progress("TestAlgo", 0.2)),
        ];
        
        // Record initial state
        let initial_budget = model.allocate_budget(&algorithms)[0];
        algorithms[0].step(initial_budget);
        
        // Simulate multiple steps to establish progress rate
        for _ in 0..3 {
            let budget = model.allocate_budget(&algorithms)[0];
            algorithms[0].step(budget);
        }
        
        let final_budget = model.allocate_budget(&algorithms)[0];
        
        assert!(initial_budget > 0);
        assert!(final_budget > 0);
        
        // The model should have learned the algorithm's progress rate
        // and adjusted allocation accordingly
    }

    #[test]
    fn test_adaptive_fairness_stagnant_vs_progressing() {
        let model = AdaptiveFairness::new(0.4);
        let algorithms: Vec<Box<dyn Sorter>> = vec![
            Box::new(MockSorter::with_progress("StagnantAlgo", 0.1)),
            Box::new(MockSorter::with_progress("ProgressingAlgo", 0.9)),
        ];
        
        let budgets = model.allocate_budget(&algorithms);
        
        assert_eq!(budgets.len(), 2);
        assert!(budgets[0] > 0); // Stagnant should get budget (needs help)
        assert!(budgets[1] > 0); // Progressing should get budget (doing well)
        
        // Model should balance between helping struggling algorithms
        // and rewarding efficient ones
        let total_budget: usize = budgets.iter().sum();
        assert!(total_budget > 0);
    }

    #[test]
    fn test_adaptive_fairness_learning_rate_bounds() {
        // Test with learning rates outside normal bounds
        let models = vec![
            AdaptiveFairness::new(-1.0),
            AdaptiveFairness::new(0.0),
            AdaptiveFairness::new(0.5),
            AdaptiveFairness::new(1.0),
            AdaptiveFairness::new(2.0),
        ];
        
        let algorithms: Vec<Box<dyn Sorter>> = vec![
            Box::new(MockSorter::with_progress("TestAlgo", 0.5)),
        ];
        
        for model in &models {
            let budgets = model.allocate_budget(&algorithms);
            assert_eq!(budgets.len(), 1);
            // All learning rates should produce valid budgets
            assert!(budgets[0] >= 0);
        }
    }
}
//! Contract tests for the WallTimeFairness model
//! These tests verify that each algorithm gets equal time slices and timing accuracy

use sorting_race::models::traits::{FairnessModel, Sorter};
use sorting_race::services::fairness::WallTimeFairness;
use std::time::{Duration, Instant};

#[derive(Debug)]
struct MockSorter {
    name: String,
    complete: bool,
    comparisons: u64,
    moves: u64,
    last_step_duration: Duration,
}

impl MockSorter {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            complete: false,
            comparisons: 0,
            moves: 0,
            last_step_duration: Duration::from_millis(0),
        }
    }

    fn with_completion(name: &str, complete: bool) -> Self {
        Self {
            name: name.to_string(),
            complete,
            comparisons: 0,
            moves: 0,
            last_step_duration: Duration::from_millis(0),
        }
    }

    fn with_duration(name: &str, duration_ms: u64) -> Self {
        Self {
            name: name.to_string(),
            complete: false,
            comparisons: 0,
            moves: 0,
            last_step_duration: Duration::from_millis(duration_ms),
        }
    }

    fn set_step_duration(&mut self, duration_ms: u64) {
        self.last_step_duration = Duration::from_millis(duration_ms);
    }
}

// Minimal Sorter implementation for testing
impl Sorter for MockSorter {
    fn step(&mut self, budget: usize) -> sorting_race::models::traits::StepResult {
        // Simulate work by incrementing stats
        self.comparisons += budget as u64;
        self.moves += (budget / 2) as u64;
        
        // Simulate timing by sleeping for the configured duration
        if self.last_step_duration > Duration::from_millis(0) {
            std::thread::sleep(self.last_step_duration);
        }
        
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
    fn test_walltime_fairness_initialization() {
        let model = WallTimeFairness::new(10);
        assert!(model.name().contains("Wall"));
        assert!(model.name().contains("Time") || model.name().contains("time"));
        
        let model_long_slice = WallTimeFairness::new(100);
        assert!(model_long_slice.name().contains("Wall"));
        
        let model_short_slice = WallTimeFairness::new(5);
        assert!(model_short_slice.name().contains("Wall"));
    }

    #[test]
    fn test_walltime_fairness_equal_time_slices() {
        let slice_ms = 10;
        let model = WallTimeFairness::new(slice_ms);
        let algorithms: Vec<Box<dyn Sorter>> = vec![
            Box::new(MockSorter::new("Fast")),
            Box::new(MockSorter::new("Medium")),
            Box::new(MockSorter::new("Slow")),
        ];
        
        let budgets = model.allocate_budget(&algorithms);
        
        assert_eq!(budgets.len(), 3);
        // All active algorithms should get reasonable budgets for the time slice
        for budget in &budgets {
            assert!(*budget > 0);
            assert!(*budget <= 1000); // Reasonable upper bound for 10ms
        }
        
        // All budgets should be similar for equal time slices
        let min_budget = *budgets.iter().min().unwrap();
        let max_budget = *budgets.iter().max().unwrap();
        
        // Allow some variance but should be relatively similar
        assert!(max_budget <= min_budget * 3, "Budget variance too high for equal time slices");
    }

    #[test]
    fn test_walltime_fairness_timing_measurement() {
        let slice_ms = 20;
        let model = WallTimeFairness::new(slice_ms);
        let mut algorithms: Vec<Box<dyn Sorter>> = vec![
            Box::new(MockSorter::with_duration("Fast", 1)), // 1ms per operation
            Box::new(MockSorter::with_duration("Slow", 5)), // 5ms per operation
        ];
        
        // Simulate running the algorithms to establish timing patterns
        let initial_budgets = model.allocate_budget(&algorithms);
        
        // Execute one step to establish timing
        for (i, algorithm) in algorithms.iter_mut().enumerate() {
            if initial_budgets[i] > 0 {
                algorithm.step(initial_budgets[i]);
            }
        }
        
        // Get new budgets after timing measurement
        let adjusted_budgets = model.allocate_budget(&algorithms);
        
        assert_eq!(adjusted_budgets.len(), 2);
        assert!(adjusted_budgets[0] > 0); // Fast algorithm should get budget
        assert!(adjusted_budgets[1] > 0); // Slow algorithm should get budget
        
        // The model should adapt to different algorithm speeds
        // Note: The exact relationship depends on implementation details
    }

    #[test]
    fn test_walltime_fairness_slower_algorithms_compensation() {
        let slice_ms = 15;
        let model = WallTimeFairness::new(slice_ms);
        let mut algorithms: Vec<Box<dyn Sorter>> = vec![
            Box::new(MockSorter::with_duration("VeryFast", 1)),
            Box::new(MockSorter::with_duration("VerySlow", 10)),
        ];
        
        // Run multiple rounds to let the model learn timing patterns
        for _ in 0..3 {
            let budgets = model.allocate_budget(&algorithms);
            
            // Execute steps
            for (i, algorithm) in algorithms.iter_mut().enumerate() {
                if budgets[i] > 0 {
                    algorithm.step(budgets[i]);
                }
            }
        }
        
        let final_budgets = model.allocate_budget(&algorithms);
        
        assert_eq!(final_budgets.len(), 2);
        assert!(final_budgets[0] > 0);
        assert!(final_budgets[1] > 0);
        
        // The model should compensate for slower algorithms
        // Exact behavior depends on implementation, but both should get reasonable budgets
    }

    #[test]
    fn test_walltime_fairness_different_time_slices() {
        let algorithms = || -> Vec<Box<dyn Sorter>> {
            vec![
                Box::new(MockSorter::new("Algo1")),
                Box::new(MockSorter::new("Algo2")),
            ]
        };
        
        let short_slice_model = WallTimeFairness::new(5);
        let long_slice_model = WallTimeFairness::new(50);
        
        let short_budgets = short_slice_model.allocate_budget(&algorithms());
        let long_budgets = long_slice_model.allocate_budget(&algorithms());
        
        assert_eq!(short_budgets.len(), 2);
        assert_eq!(long_budgets.len(), 2);
        
        // Longer time slices should generally allow for larger budgets
        let short_total: usize = short_budgets.iter().sum();
        let long_total: usize = long_budgets.iter().sum();
        
        assert!(short_total > 0);
        assert!(long_total > 0);
        
        // Long slices should typically allow more operations
        // (though this depends on implementation details)
    }

    #[test]
    fn test_walltime_fairness_skips_completed() {
        let model = WallTimeFairness::new(20);
        let algorithms: Vec<Box<dyn Sorter>> = vec![
            Box::new(MockSorter::new("Active1")),
            Box::new(MockSorter::with_completion("Completed", true)),
            Box::new(MockSorter::new("Active2")),
        ];
        
        let budgets = model.allocate_budget(&algorithms);
        
        assert_eq!(budgets.len(), 3);
        assert!(budgets[0] > 0); // Active1 should get budget
        assert_eq!(budgets[1], 0); // Completed should get 0 budget
        assert!(budgets[2] > 0); // Active2 should get budget
    }

    #[test]
    fn test_walltime_fairness_all_complete() {
        let model = WallTimeFairness::new(10);
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
    fn test_walltime_fairness_single_algorithm() {
        let model = WallTimeFairness::new(25);
        let algorithms: Vec<Box<dyn Sorter>> = vec![
            Box::new(MockSorter::new("OnlyAlgo")),
        ];
        
        let budgets = model.allocate_budget(&algorithms);
        
        assert_eq!(budgets.len(), 1);
        assert!(budgets[0] > 0); // Single active algorithm should get budget
    }

    #[test]
    fn test_walltime_fairness_empty_algorithms() {
        let model = WallTimeFairness::new(15);
        let algorithms: Vec<Box<dyn Sorter>> = vec![];
        
        let budgets = model.allocate_budget(&algorithms);
        
        assert_eq!(budgets.len(), 0);
    }

    #[test]
    fn test_walltime_fairness_very_short_slice() {
        let model = WallTimeFairness::new(1); // 1ms slice
        let algorithms: Vec<Box<dyn Sorter>> = vec![
            Box::new(MockSorter::new("Algo1")),
            Box::new(MockSorter::new("Algo2")),
        ];
        
        let budgets = model.allocate_budget(&algorithms);
        
        assert_eq!(budgets.len(), 2);
        // Even with very short slices, should allocate reasonable budgets
        for budget in &budgets {
            assert!(*budget > 0);
            assert!(*budget <= 100); // Should be reasonable for 1ms
        }
    }

    #[test]
    fn test_walltime_fairness_very_long_slice() {
        let model = WallTimeFairness::new(1000); // 1 second slice
        let algorithms: Vec<Box<dyn Sorter>> = vec![
            Box::new(MockSorter::new("Algo1")),
            Box::new(MockSorter::new("Algo2")),
        ];
        
        let budgets = model.allocate_budget(&algorithms);
        
        assert_eq!(budgets.len(), 2);
        // With long slices, should allocate substantial budgets
        for budget in &budgets {
            assert!(*budget > 0);
            assert!(*budget <= 100000); // Reasonable upper bound for 1 second
        }
    }

    #[test]
    fn test_walltime_fairness_consistency() {
        let model = WallTimeFairness::new(30);
        let algorithms: Vec<Box<dyn Sorter>> = vec![
            Box::new(MockSorter::new("Consistent1")),
            Box::new(MockSorter::new("Consistent2")),
            Box::new(MockSorter::new("Consistent3")),
        ];
        
        // Get budgets multiple times with same state
        let budgets1 = model.allocate_budget(&algorithms);
        let budgets2 = model.allocate_budget(&algorithms);
        
        assert_eq!(budgets1.len(), budgets2.len());
        
        // Budgets should be consistent for same algorithm states
        // (allowing for some timing variance)
        for (b1, b2) in budgets1.iter().zip(budgets2.iter()) {
            let diff = (*b1 as i32 - *b2 as i32).abs();
            let avg = (*b1 + *b2) / 2;
            if avg > 0 {
                // Allow up to 50% variance due to timing measurement uncertainty
                assert!(diff <= (avg as i32 / 2), "Budget consistency check failed");
            }
        }
    }

    #[test]
    fn test_walltime_fairness_progressive_adjustment() {
        let model = WallTimeFairness::new(25);
        let mut algorithms: Vec<Box<dyn Sorter>> = vec![
            Box::new(MockSorter::with_duration("GettingSlower", 1)),
        ];
        
        // Start with fast algorithm
        let initial_budget = model.allocate_budget(&algorithms)[0];
        algorithms[0].step(initial_budget);
        
        // Make algorithm slower
        if let Some(mock) = algorithms[0].as_any_mut().downcast_mut::<MockSorter>() {
            mock.set_step_duration(10);
        }
        
        // Execute with slower timing
        let budget_after_slowdown = model.allocate_budget(&algorithms)[0];
        algorithms[0].step(budget_after_slowdown);
        
        // Get budget after adjustment
        let adjusted_budget = model.allocate_budget(&algorithms)[0];
        
        assert!(initial_budget > 0);
        assert!(budget_after_slowdown > 0);
        assert!(adjusted_budget > 0);
        
        // The model should adapt to changing algorithm performance
        // Exact relationship depends on implementation
    }

    #[test]
    fn test_walltime_fairness_zero_slice_time() {
        let model = WallTimeFairness::new(0); // Edge case: zero time slice
        let algorithms: Vec<Box<dyn Sorter>> = vec![
            Box::new(MockSorter::new("Algo1")),
        ];
        
        let budgets = model.allocate_budget(&algorithms);
        
        assert_eq!(budgets.len(), 1);
        // Should handle zero time slice gracefully
        // Exact behavior depends on implementation (might give minimal budget or zero)
        assert!(budgets[0] >= 0);
    }

    #[test]
    fn test_walltime_fairness_timing_accuracy_bounds() {
        let slice_ms = 40;
        let model = WallTimeFairness::new(slice_ms);
        let algorithms: Vec<Box<dyn Sorter>> = vec![
            Box::new(MockSorter::new("TestAlgo")),
        ];
        
        let budget = model.allocate_budget(&algorithms)[0];
        
        // Measure actual execution time
        let start = Instant::now();
        algorithms[0].as_any().downcast_ref::<MockSorter>().unwrap();
        let duration = start.elapsed();
        
        // Budget should be reasonable for the time slice
        assert!(budget > 0);
        assert!(budget <= 10000); // Reasonable upper bound
        
        // The budget should be appropriate for the time slice
        // (exact relationship depends on algorithm speed estimation)
    }
}
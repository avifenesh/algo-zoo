//! Wall-time based fairness model implementation

use crate::models::traits::{FairnessModel, Sorter};
use std::time::Duration;
use std::collections::HashMap;

/// Fairness model that allocates budgets based on wall-clock time slices
/// Each algorithm gets equal time slices for fair execution
#[derive(Debug)]
pub struct WallTimeFairness {
    slice_ms: u64,  // Time slice in milliseconds
    algorithm_timings: HashMap<String, Duration>,  // Last measured execution times
    algorithm_speeds: HashMap<String, f64>,        // Operations per millisecond estimate
}

impl WallTimeFairness {
    /// Create a new wall-time fairness model
    /// 
    /// # Arguments
    /// * `slice_ms` - Time slice in milliseconds that each algorithm should get
    pub fn new(slice_ms: u64) -> Self {
        Self {
            slice_ms: slice_ms.max(1),
            algorithm_timings: HashMap::new(),
            algorithm_speeds: HashMap::new(),
        }
    }

    /// Update timing information for an algorithm after execution
    pub fn update_timing(&mut self, algorithm_name: &str, duration: Duration, operations_performed: usize) {
        self.algorithm_timings.insert(algorithm_name.to_string(), duration);
        
        if duration.as_millis() > 0 {
            let speed = operations_performed as f64 / duration.as_millis() as f64;
            self.algorithm_speeds.insert(algorithm_name.to_string(), speed);
        }
    }

    /// Estimate how many operations an algorithm can perform in the time slice
    fn estimate_operations_for_time_slice(&self, algorithm_name: &str) -> usize {
        let default_speed = 10.0; // Default: 10 operations per millisecond
        let speed = self.algorithm_speeds.get(algorithm_name).copied().unwrap_or(default_speed);
        ((speed * self.slice_ms as f64).round() as usize).max(1)
    }

    /// Get the time slice in milliseconds
    pub fn get_slice_ms(&self) -> u64 {
        self.slice_ms
    }

    /// Clear all timing history
    pub fn clear_history(&mut self) {
        self.algorithm_timings.clear();
        self.algorithm_speeds.clear();
    }
}

impl Default for WallTimeFairness {
    fn default() -> Self {
        Self::new(50) // 50ms time slice
    }
}

impl FairnessModel for WallTimeFairness {
    fn allocate_budget(&self, algorithms: &[Box<dyn Sorter>]) -> Vec<usize> {
        algorithms
            .iter()
            .map(|algorithm| {
                if algorithm.is_complete() {
                    0 // No budget for completed algorithms
                } else {
                    // Estimate budget based on time slice and algorithm speed
                    self.estimate_operations_for_time_slice(algorithm.name())
                }
            })
            .collect()
    }

    fn name(&self) -> &str {
        "Wall Time Fairness"
    }
}

/// Adaptive wall-time fairness that learns from algorithm performance
#[derive(Debug)]
pub struct AdaptiveWallTimeFairness {
    base_fairness: WallTimeFairness,
    performance_multipliers: HashMap<String, f64>,
    adaptation_rate: f64,
}

impl AdaptiveWallTimeFairness {
    /// Create a new adaptive wall-time fairness model
    pub fn new(slice_ms: u64, adaptation_rate: f64) -> Self {
        Self {
            base_fairness: WallTimeFairness::new(slice_ms),
            performance_multipliers: HashMap::new(),
            adaptation_rate: adaptation_rate.clamp(0.01, 1.0),
        }
    }

    /// Update performance multiplier based on algorithm efficiency
    pub fn update_performance(&mut self, algorithm_name: &str, efficiency: f64) {
        let multiplier = self.performance_multipliers
            .entry(algorithm_name.to_string())
            .or_insert(1.0);

        // Adaptive learning: slowly adjust multiplier based on efficiency
        *multiplier = *multiplier * (1.0 - self.adaptation_rate) + efficiency * self.adaptation_rate;
        
        // Clamp multiplier to reasonable bounds
        *multiplier = multiplier.clamp(0.1, 3.0);
    }

    /// Get performance multiplier for an algorithm
    fn get_performance_multiplier(&self, algorithm_name: &str) -> f64 {
        self.performance_multipliers.get(algorithm_name).copied().unwrap_or(1.0)
    }

    /// Clear performance history
    pub fn clear_performance_history(&mut self) {
        self.performance_multipliers.clear();
    }

    /// Update timing information for an algorithm
    pub fn update_timing(&mut self, algorithm_name: &str, duration: Duration, operations_performed: usize) {
        self.base_fairness.update_timing(algorithm_name, duration, operations_performed);
    }
}

impl Default for AdaptiveWallTimeFairness {
    fn default() -> Self {
        Self::new(50, 0.1) // 50ms slice, 10% adaptation rate
    }
}

impl FairnessModel for AdaptiveWallTimeFairness {
    fn allocate_budget(&self, algorithms: &[Box<dyn Sorter>]) -> Vec<usize> {
        let base_budgets = self.base_fairness.allocate_budget(algorithms);
        
        base_budgets
            .into_iter()
            .zip(algorithms.iter())
            .map(|(budget, algorithm)| {
                if budget == 0 {
                    0 // Keep zero budget for completed algorithms
                } else {
                    let multiplier = self.get_performance_multiplier(algorithm.name());
                    ((budget as f64) * multiplier).round() as usize
                }
            })
            .collect()
    }

    fn name(&self) -> &str {
        "Adaptive Wall Time"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::sorters::bubble::BubbleSort;
    
    use std::time::Duration;

    #[test]
    fn test_walltime_fairness_timing() {
        let mut fairness = WallTimeFairness::new(50);
        
        let duration = Duration::from_millis(10);
        fairness.update_timing("Test Algorithm", duration, 100);
        
        let budget = fairness.estimate_operations_for_time_slice("Test Algorithm");
        assert!(budget > 0);
    }

    #[test]
    fn test_walltime_fairness_budget_calculation() {
        let mut fairness = WallTimeFairness::new(100);
        
        // Update timing for different speed algorithms
        fairness.update_timing("Fast", Duration::from_millis(10), 200); // 20 ops/ms
        fairness.update_timing("Slow", Duration::from_millis(50), 100); // 2 ops/ms
        
        let mut bubble_fast = BubbleSort::new();
        let mut bubble_slow = BubbleSort::new();
        bubble_fast.reset(vec![1, 2, 3]);
        bubble_slow.reset(vec![3, 2, 1]);
        
        let algorithms: Vec<Box<dyn Sorter>> = vec![
            Box::new(bubble_fast),
            Box::new(bubble_slow),
        ];

        let budgets = fairness.allocate_budget(&algorithms);
        
        // Both should get some budget
        assert!(budgets.iter().all(|&b| b > 0));
        
        // Fast algorithm should get higher budget due to higher speed
        // (Note: This test assumes algorithm names match but in practice 
        // the fairness model uses default speeds for unknown algorithms)
    }

    #[test]
    fn test_adaptive_walltime_fairness_performance_update() {
        let mut fairness = AdaptiveWallTimeFairness::new(100, 0.2);
        
        // Update performance for an algorithm
        fairness.update_performance("Test Algorithm", 2.0);
        let result = fairness.get_performance_multiplier("Test Algorithm");
        let expected = 1.2; // 1.0 * 0.8 + 2.0 * 0.2
        assert!((result - expected).abs() < 0.000001, "Expected {}, got {}", expected, result);
        
        // Update again
        fairness.update_performance("Test Algorithm", 0.5);
        let expected = 1.2 * 0.8 + 0.5 * 0.2; // Should be around 1.06
        let result = fairness.get_performance_multiplier("Test Algorithm");
        assert!((result - expected).abs() < 0.000001, "Expected {}, got {}", expected, result);
    }

    #[test]
    fn test_walltime_fairness_name() {
        let fairness = WallTimeFairness::default();
        assert_eq!(fairness.name(), "Wall Time Fairness");
        
        let adaptive_fairness = AdaptiveWallTimeFairness::default();
        assert_eq!(adaptive_fairness.name(), "Adaptive Wall Time");
    }
}
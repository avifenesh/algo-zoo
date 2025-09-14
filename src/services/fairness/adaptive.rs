//! Adaptive fairness model that adjusts allocation based on algorithm progress rates

use crate::models::traits::{FairnessModel, Sorter};
use std::collections::HashMap;
use std::cell::RefCell;

/// Adaptive fairness model that learns algorithm efficiency and allocates budget accordingly
/// Uses exponential moving average to track progress rates and allocate budget to slower algorithms
#[derive(Debug)]
pub struct AdaptiveFairness {
    learning_rate: f32,
    base_budget: usize,
    progress_rates: RefCell<HashMap<String, f32>>, // Exponential moving average of progress rates
    previous_progress: RefCell<HashMap<String, f32>>, // Previous progress values for rate calculation
    step_count: RefCell<HashMap<String, u64>>, // Number of steps taken by each algorithm
}

impl AdaptiveFairness {
    /// Create a new adaptive fairness model
    /// 
    /// # Arguments
    /// * `learning_rate` - How quickly to adapt to performance changes (0.0 to 1.0)
    pub fn new(learning_rate: f32) -> Self {
        Self {
            learning_rate: learning_rate.clamp(0.0, 1.0),
            base_budget: 100,
            progress_rates: RefCell::new(HashMap::new()),
            previous_progress: RefCell::new(HashMap::new()),
            step_count: RefCell::new(HashMap::new()),
        }
    }

    /// Update progress rate for an algorithm using exponential moving average
    fn update_progress_rate(&self, algorithm: &dyn Sorter) {
        let name = algorithm.name();
        let current_progress = algorithm.get_telemetry().progress_hint;
        
        let mut progress_rates = self.progress_rates.borrow_mut();
        let mut previous_progress = self.previous_progress.borrow_mut();
        let mut step_count = self.step_count.borrow_mut();
        
        // Get previous progress or initialize to 0.0
        let prev_progress = previous_progress.get(name).copied().unwrap_or(0.0);
        
        // Calculate progress rate (progress change per step)
        let progress_delta = current_progress - prev_progress;
        let current_rate = progress_delta.max(0.0); // Only positive progress
        
        // Update exponential moving average
        // Formula: new_rate = (1 - learning_rate) * old_rate + learning_rate * current_rate
        let old_rate = progress_rates.get(name).copied().unwrap_or(current_rate);
        let new_rate = (1.0 - self.learning_rate) * old_rate + self.learning_rate * current_rate;
        
        progress_rates.insert(name.to_string(), new_rate);
        previous_progress.insert(name.to_string(), current_progress);
        
        // Increment step count
        let steps = step_count.get(name).copied().unwrap_or(0) + 1;
        step_count.insert(name.to_string(), steps);
    }

    /// Get the current progress rate for an algorithm
    fn get_progress_rate(&self, algorithm_name: &str) -> f32 {
        self.progress_rates.borrow().get(algorithm_name).copied().unwrap_or(0.01) // Default small rate
    }

    /// Set the base budget
    pub fn set_base_budget(&mut self, budget: usize) {
        self.base_budget = budget.max(1);
    }

    /// Clear all progress tracking data
    pub fn clear_history(&self) {
        self.progress_rates.borrow_mut().clear();
        self.previous_progress.borrow_mut().clear();
        self.step_count.borrow_mut().clear();
    }

    /// Get learning rate
    pub fn get_learning_rate(&self) -> f32 {
        self.learning_rate
    }
}

impl Default for AdaptiveFairness {
    fn default() -> Self {
        Self::new(0.2) // 20% learning rate
    }
}

impl FairnessModel for AdaptiveFairness {
    fn allocate_budget(&self, algorithms: &[Box<dyn Sorter>]) -> Vec<usize> {
        // Update progress rates for all algorithms
        for algorithm in algorithms.iter() {
            if !algorithm.is_complete() {
                self.update_progress_rate(algorithm.as_ref());
            }
        }
        
        // Filter active algorithms and their progress rates
        let active_algos: Vec<(usize, f32)> = algorithms
            .iter()
            .enumerate()
            .filter_map(|(i, alg)| {
                if alg.is_complete() {
                    None
                } else {
                    let rate = self.get_progress_rate(alg.name());
                    Some((i, rate))
                }
            })
            .collect();
        
        // Handle edge cases
        if active_algos.is_empty() {
            return vec![0; algorithms.len()];
        }
        
        if active_algos.len() == 1 {
            let mut budgets = vec![0; algorithms.len()];
            budgets[active_algos[0].0] = self.base_budget;
            return budgets;
        }
        
        // Calculate total budget to distribute
        let total_budget = self.base_budget * active_algos.len();
        
        // Calculate inverse fairness: algorithms with slower progress get more help
        let max_rate = active_algos.iter().map(|(_, rate)| *rate).fold(0.0f32, f32::max);
        let min_rate = active_algos.iter().map(|(_, rate)| *rate).fold(f32::INFINITY, f32::min);
        
        let mut budgets = vec![0; algorithms.len()];
        
        if max_rate == min_rate || max_rate == 0.0 {
            // All algorithms have same progress rate, distribute equally
            let equal_budget = total_budget / active_algos.len();
            for (idx, _) in active_algos {
                budgets[idx] = equal_budget;
            }
        } else {
            // Give more budget to algorithms with slower progress (need more help)
            let total_inverse: f32 = active_algos.iter()
                .map(|(_, rate)| max_rate - rate + 0.01) // Add small epsilon to avoid zero
                .sum();
            
            for (idx, rate) in active_algos {
                let inverse_weight = max_rate - rate + 0.01;
                let proportion = inverse_weight / total_inverse;
                let budget = ((total_budget as f32 * proportion).round() as usize).max(1);
                budgets[idx] = budget;
            }
        }
        
        budgets
    }
    
    fn name(&self) -> &str {
        "Adaptive Fairness"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::traits::Telemetry;
    
    #[derive(Debug)]
    struct MockSorter {
        name: String,
        complete: bool,
        progress: f32,
    }
    
    impl MockSorter {
        fn new(name: &str, progress: f32) -> Self {
            Self {
                name: name.to_string(),
                complete: false,
                progress,
            }
        }
    }
    
    impl Sorter for MockSorter {
        fn step(&mut self, _budget: usize) -> crate::models::traits::StepResult {
            unimplemented!()
        }
        
        fn is_complete(&self) -> bool {
            self.complete
        }
        
        fn get_telemetry(&self) -> Telemetry {
            Telemetry {
                total_comparisons: 0,
                total_moves: 0,
                memory_current: 0,
                memory_peak: 0,
                highlights: vec![],
                markers: crate::models::traits::Markers::default(),
                status_text: String::new(),
                progress_hint: self.progress,
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
    
    #[test]
    fn test_adaptive_fairness_creation() {
        let model = AdaptiveFairness::new(0.2);
        assert_eq!(model.learning_rate, 0.2);
        assert_eq!(model.name(), "Adaptive Fairness");
    }
    
    #[test]
    fn test_adaptive_fairness_budget_allocation() {
        let model = AdaptiveFairness::new(0.1);
        let algorithms: Vec<Box<dyn Sorter>> = vec![
            Box::new(MockSorter::new("Test1", 0.5)),
            Box::new(MockSorter::new("Test2", 0.3)),
        ];
        
        let budgets = model.allocate_budget(&algorithms);
        
        assert_eq!(budgets.len(), 2);
        assert!(budgets[0] > 0);
        assert!(budgets[1] > 0);
    }
}
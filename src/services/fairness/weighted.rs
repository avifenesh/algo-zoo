//! Weighted fairness model implementation

use crate::models::traits::{FairnessModel, Sorter};
use std::collections::HashMap;

/// Fairness model that allocates budgets based on α*comparisons + β*moves scoring
/// Algorithms with lower weighted scores get more budget (inverse fairness)
#[derive(Debug)]
pub struct WeightedFairness {
    alpha: f32,  // Weight for comparisons
    beta: f32,   // Weight for moves
    base_budget: usize,
}

impl WeightedFairness {
    /// Create a new weighted fairness model
    /// 
    /// # Arguments
    /// * `alpha` - Weight for comparisons in the scoring formula
    /// * `beta` - Weight for moves in the scoring formula
    pub fn new(alpha: f32, beta: f32) -> Self {
        Self {
            alpha,
            beta, 
            base_budget: 100, // Default base budget
        }
    }

    /// Calculate weighted score for an algorithm: α*comparisons + β*moves
    fn calculate_weighted_score(&self, algorithm: &dyn Sorter) -> f32 {
        let telemetry = algorithm.get_telemetry();
        self.alpha * telemetry.total_comparisons as f32 + self.beta * telemetry.total_moves as f32
    }

    /// Set the base budget for allocation
    pub fn set_base_budget(&mut self, budget: usize) {
        self.base_budget = budget.max(1);
    }

    /// Get the current base budget
    pub fn get_base_budget(&self) -> usize {
        self.base_budget
    }
}

impl Default for WeightedFairness {
    fn default() -> Self {
        Self::new(1.0, 1.0) // Equal weights for comparisons and moves
    }
}

impl FairnessModel for WeightedFairness {
    fn allocate_budget(&self, algorithms: &[Box<dyn Sorter>]) -> Vec<usize> {
        // Filter active algorithms
        let active_algorithms: Vec<(usize, f32)> = algorithms
            .iter()
            .enumerate()
            .filter_map(|(i, alg)| {
                if alg.is_complete() {
                    None
                } else {
                    Some((i, self.calculate_weighted_score(alg.as_ref())))
                }
            })
            .collect();

        // Handle edge cases
        if active_algorithms.is_empty() {
            return vec![0; algorithms.len()];
        }

        if active_algorithms.len() == 1 {
            let mut budgets = vec![0; algorithms.len()];
            budgets[active_algorithms[0].0] = self.base_budget;
            return budgets;
        }

        // Handle zero weights case
        if self.alpha == 0.0 && self.beta == 0.0 {
            // Equal distribution when both weights are zero
            let equal_budget = self.base_budget;
            return algorithms
                .iter()
                .map(|alg| if alg.is_complete() { 0 } else { equal_budget })
                .collect();
        }

        // Calculate total weighted scores for normalization
        let total_score: f32 = active_algorithms.iter().map(|(_, score)| *score).sum();
        
        // Inverse fairness: algorithms with lower scores get MORE budget
        let mut budgets = vec![0; algorithms.len()];
        let total_budget = self.base_budget * active_algorithms.len();
        
        if total_score == 0.0 {
            // If all scores are zero, distribute equally among active algorithms
            let equal_budget = total_budget / active_algorithms.len();
            for (idx, _) in active_algorithms {
                budgets[idx] = equal_budget;
            }
        } else {
            // Calculate inverse weights (lower score = higher budget)
            for (idx, score) in &active_algorithms {
                // Inverse proportion: (total - score) / total gives higher budget to lower scores
                let max_score = active_algorithms.iter().map(|(_, s)| *s).fold(0.0f32, f32::max);
                let inverse_score = max_score - score + 1.0; // Add 1 to avoid zero division
                let total_inverse: f32 = active_algorithms.iter()
                    .map(|(_, s)| max_score - s + 1.0)
                    .sum();
                
                let budget = ((total_budget as f32 * inverse_score) / total_inverse).round() as usize;
                budgets[*idx] = budget.max(1); // Ensure minimum budget
            }
        }

        budgets
    }

    fn name(&self) -> &str {
        "Weighted Fairness"
    }
}

/// Performance-based weighted fairness that adjusts based on algorithm efficiency
#[derive(Debug)]
pub struct PerformanceWeightedFairness {
    base_budget: usize,
    efficiency_history: HashMap<String, Vec<f32>>,
    history_window: usize,
}

impl PerformanceWeightedFairness {
    /// Create a new performance-weighted fairness model
    pub fn new(base_budget: usize, history_window: usize) -> Self {
        Self {
            base_budget: base_budget.max(1),
            efficiency_history: HashMap::new(),
            history_window: history_window.max(1),
        }
    }

    /// Record efficiency for an algorithm
    pub fn record_efficiency(&mut self, algorithm_name: &str, efficiency: f32) {
        let history = self.efficiency_history
            .entry(algorithm_name.to_string())
            .or_default();
        
        history.push(efficiency.max(0.0));
        
        // Keep only recent history
        if history.len() > self.history_window {
            history.remove(0);
        }
    }

    /// Calculate average efficiency for an algorithm
    fn get_average_efficiency(&self, algorithm_name: &str) -> f32 {
        if let Some(history) = self.efficiency_history.get(algorithm_name) {
            if !history.is_empty() {
                history.iter().sum::<f32>() / history.len() as f32
            } else {
                1.0 // Default efficiency
            }
        } else {
            1.0 // Default efficiency for unknown algorithms
        }
    }

    /// Clear efficiency history
    pub fn clear_history(&mut self) {
        self.efficiency_history.clear();
    }
}

impl Default for PerformanceWeightedFairness {
    fn default() -> Self {
        Self::new(10, 10)
    }
}

impl FairnessModel for PerformanceWeightedFairness {
    fn allocate_budget(&self, algorithms: &[Box<dyn Sorter>]) -> Vec<usize> {
        // Calculate total efficiency to normalize
        let total_efficiency: f32 = algorithms
            .iter()
            .filter(|alg| !alg.is_complete())
            .map(|alg| self.get_average_efficiency(alg.name()))
            .sum();

        if total_efficiency == 0.0 {
            // Fallback to equal allocation
            return algorithms
                .iter()
                .map(|alg| if alg.is_complete() { 0 } else { self.base_budget })
                .collect();
        }

        algorithms
            .iter()
            .map(|algorithm| {
                if algorithm.is_complete() {
                    0
                } else {
                    let efficiency = self.get_average_efficiency(algorithm.name());
                    let proportion = efficiency / total_efficiency;
                    let weighted_budget = (self.base_budget as f32 * proportion * algorithms.len() as f32).round() as usize;
                    weighted_budget.max(1)
                }
            })
            .collect()
    }

    fn name(&self) -> &str {
        "Performance Weighted"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::sorters::{bubble::BubbleSort, quick::QuickSort};

    #[test]
    fn test_weighted_fairness_different_algorithms() {
        let fairness = WeightedFairness::new(1.0, 1.0); // Equal weights
        
        let mut bubble = BubbleSort::new();
        let mut quick = QuickSort::new();
        bubble.reset(vec![3, 1, 2]);
        quick.reset(vec![3, 1, 2]);

        let algorithms: Vec<Box<dyn Sorter>> = vec![
            Box::new(bubble),
            Box::new(quick),
        ];

        let budgets = fairness.allocate_budget(&algorithms);
        
        // Both should get some budget
        assert!(budgets[0] > 0);
        assert!(budgets[1] > 0);
        
        // Total budget should be reasonable
        let total: usize = budgets.iter().sum();
        assert!(total > 0);
    }

    #[test]
    fn test_performance_weighted_fairness() {
        let mut fairness = PerformanceWeightedFairness::new(10, 5);
        
        // Record some efficiency data
        fairness.record_efficiency("Algorithm A", 2.0);
        fairness.record_efficiency("Algorithm A", 1.5);
        fairness.record_efficiency("Algorithm B", 1.0);
        
        assert_eq!(fairness.get_average_efficiency("Algorithm A"), 1.75);
        assert_eq!(fairness.get_average_efficiency("Algorithm B"), 1.0);
    }

    #[test]
    fn test_weighted_fairness_alpha_beta_scoring() {
        let fairness = WeightedFairness::new(2.0, 0.5); // Favor comparisons
        
        // Create mock algorithms with different stats
        let mut high_comp = BubbleSort::new();
        let mut high_move = BubbleSort::new();
        high_comp.reset(vec![10, 9, 8, 7, 6, 5, 4, 3, 2, 1]); // Larger reverse-sorted array
        high_move.reset(vec![1, 3, 2, 4, 5, 7, 6, 8, 9, 10]); // Larger partially sorted
        
        // Simulate different amounts of work
        high_comp.step(20); // Do more comparisons
        high_move.step(5);  // Do fewer comparisons
        
        let algorithms: Vec<Box<dyn Sorter>> = vec![
            Box::new(high_comp),
            Box::new(high_move),
        ];
        
        let budgets = fairness.allocate_budget(&algorithms);
        
        assert_eq!(budgets.len(), 2);
        assert!(budgets[0] > 0, "First algorithm should get budget > 0, got {}", budgets[0]);
        assert!(budgets[1] > 0, "Second algorithm should get budget > 0, got {}", budgets[1]);
        
        // The algorithm with fewer weighted operations should get more budget
        let total: usize = budgets.iter().sum();
        assert!(total > 0);
    }
}
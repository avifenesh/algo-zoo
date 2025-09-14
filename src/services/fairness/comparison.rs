//! Comparison-budget fairness model implementation

use crate::models::traits::{FairnessModel, Sorter};

/// Fairness model that allocates equal comparison budgets to all algorithms
#[derive(Debug)]
pub struct ComparisonFairness {
    budget_per_step: usize,
}

impl ComparisonFairness {
    /// Create a new comparison fairness model
    pub fn new(budget: usize) -> Self {
        Self {
            budget_per_step: budget.max(1),
        }
    }

    /// Get the current budget per step
    pub fn get_budget(&self) -> usize {
        self.budget_per_step
    }

    /// Set a new budget per step
    pub fn set_budget(&mut self, budget: usize) {
        self.budget_per_step = budget.max(1);
    }
}

impl Default for ComparisonFairness {
    fn default() -> Self {
        Self::new(10)
    }
}

impl FairnessModel for ComparisonFairness {
    fn allocate_budget(&self, algorithms: &[Box<dyn Sorter>]) -> Vec<usize> {
        // Allocate equal budget to all algorithms
        algorithms
            .iter()
            .map(|algorithm| {
                if algorithm.is_complete() {
                    0 // No budget for completed algorithms
                } else {
                    self.budget_per_step
                }
            })
            .collect()
    }

    fn name(&self) -> &str {
        "Comparison Budget"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::sorters::bubble::BubbleSort;

    #[test]
    fn test_comparison_fairness_equal_allocation() {
        let fairness = ComparisonFairness::new(5);
        let algorithms: Vec<Box<dyn Sorter>> = vec![
            Box::new(BubbleSort::new()),
            Box::new(BubbleSort::new()),
            Box::new(BubbleSort::new()),
        ];

        let budgets = fairness.allocate_budget(&algorithms);
        assert_eq!(budgets, vec![5, 5, 5]);
    }

    #[test]
    fn test_comparison_fairness_completed_algorithm() {
        let fairness = ComparisonFairness::new(10);
        let mut bubble1 = BubbleSort::new();
        let mut bubble2 = BubbleSort::new();
        
        // Set up one completed algorithm
        bubble1.reset(vec![]); // Empty array makes it complete
        bubble2.reset(vec![3, 1, 2]); // Non-empty array

        let algorithms: Vec<Box<dyn Sorter>> = vec![
            Box::new(bubble1),
            Box::new(bubble2),
        ];

        let budgets = fairness.allocate_budget(&algorithms);
        assert_eq!(budgets[0], 0); // Completed algorithm gets 0
        assert_eq!(budgets[1], 10); // Active algorithm gets full budget
    }

    #[test]
    fn test_comparison_fairness_minimum_budget() {
        let fairness = ComparisonFairness::new(0); // Should be clamped to 1
        assert_eq!(fairness.get_budget(), 1);
    }

    #[test]
    fn test_comparison_fairness_name() {
        let fairness = ComparisonFairness::new(5);
        assert_eq!(fairness.name(), "Comparison Budget");
    }
}
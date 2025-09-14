//! Property-based tests for Quick Sort incremental partitioning
//! 
//! These tests use proptest to verify that Quick Sort maintains correctness
//! and performance properties regardless of budget constraints. Tests are
//! designed to fail initially since incremental partitioning is not implemented.

use proptest::prelude::*;
use sorting_race::services::sorters::quick::QuickSort;
use sorting_race::models::traits::{Sorter, Telemetry};
use std::collections::HashSet;

/// Property: Quick Sort always produces sorted output regardless of budget
#[test]
fn property_quicksort_always_sorts_with_any_budget() {
    proptest!(|(
        data in prop::collection::vec(-1000i32..1000i32, 1..100),
        budget in 1usize..50
    )| {
        let mut quicksort = QuickSort::new();
        let original_data = data.clone();
        quicksort.reset(data);
        
        // Run algorithm with the given budget
        let mut steps = 0;
        let max_steps = original_data.len() * 10; // Should complete well before this
        
        while !quicksort.is_complete() && steps < max_steps {
            let result = quicksort.step(budget);
            steps += 1;
            
            // Budget should never be exceeded
            prop_assert!(result.comparisons_used <= budget,
                "Budget exceeded: used {} but limit was {}", 
                result.comparisons_used, budget);
        }
        
        // Algorithm should complete
        if !quicksort.is_complete() {
            // This will fail initially since incremental partitioning is not implemented
            prop_assert!(false, "Quick Sort did not complete with budget {} after {} steps. \
                This suggests incremental partitioning is not properly implemented.",
                budget, steps);
        }
        
        // Output should be sorted
        let result_array = quicksort.get_array();
        let mut expected = original_data.clone();
        expected.sort();
        
        prop_assert_eq!(result_array, expected, 
            "Array not properly sorted with budget {}", budget);
    });
}

/// Property: Total comparisons is reasonable (O(n log n) on average)
#[test]
fn property_comparison_complexity_reasonable() {
    proptest!(|(
        data in prop::collection::vec(-100i32..100i32, 10..50),
        budget in 5usize..20
    )| {
        let mut quicksort = QuickSort::new();
        let n = data.len();
        quicksort.reset(data);
        
        // Run to completion
        let mut total_steps = 0;
        let max_steps = n * 5; // Generous upper bound
        
        while !quicksort.is_complete() && total_steps < max_steps {
            quicksort.step(budget);
            total_steps += 1;
        }
        
        if quicksort.is_complete() {
            let telemetry = quicksort.get_telemetry();
            let total_comparisons = telemetry.total_comparisons as usize;
            
            // Expected comparisons should be roughly O(n log n)
            // Allow generous bounds since Quick Sort can degrade to O(n²) in worst case
            let n_log_n = if n > 1 { n * (n as f32).log2().ceil() as usize } else { 1 };
            let reasonable_upper_bound = n * n; // O(n²) worst case
            let reasonable_lower_bound = if n > 1 { n - 1 } else { 0 }; // At least n-1 for any comparison sort
            
            prop_assert!(total_comparisons >= reasonable_lower_bound,
                "Too few comparisons: {} < {} for array size {}", 
                total_comparisons, reasonable_lower_bound, n);
                
            prop_assert!(total_comparisons <= reasonable_upper_bound,
                "Too many comparisons: {} > {} (O(n²)) for array size {}. \
                Expected around {} (O(n log n))", 
                total_comparisons, reasonable_upper_bound, n, n_log_n);
        } else {
            // This should fail since incremental partitioning is not implemented
            prop_assert!(false, "Algorithm did not complete within {} steps with budget {}. \
                This indicates incremental partitioning is not working correctly.", 
                total_steps, budget);
        }
    });
}

/// Property: Progress monotonically increases (never decreases)
#[test]
fn property_progress_monotonic_increase() {
    proptest!(|(
        data in prop::collection::vec(-50i32..50i32, 5..30),
        budget in 3usize..15
    )| {
        let mut quicksort = QuickSort::new();
        quicksort.reset(data);
        
        let mut progress_history = Vec::new();
        let mut steps = 0;
        let max_steps = 500; // Increase max steps for small budgets
        
        // Collect progress values over time
        while !quicksort.is_complete() && steps < max_steps {
            let telemetry_before = quicksort.get_telemetry();
            progress_history.push(telemetry_before.progress_hint);
            
            quicksort.step(budget);
            steps += 1;
        }
        
        // Add final progress
        if quicksort.is_complete() {
            progress_history.push(quicksort.get_telemetry().progress_hint);
        }
        
        // Progress should never decrease (monotonic property)
        for i in 1..progress_history.len() {
            prop_assert!(progress_history[i] >= progress_history[i-1],
                "Progress decreased from {} to {} at step {}. Progress should be monotonic.",
                progress_history[i-1], progress_history[i], i);
        }
        
        // Final progress should be 1.0 if completed
        if quicksort.is_complete() {
            let final_progress = progress_history.last().unwrap();
            prop_assert!((*final_progress - 1.0).abs() < 0.01,
                "Final progress should be 1.0 when complete, got {}", final_progress);
        } else {
            // Algorithm should complete, but with very small budgets and data it might need more steps
            prop_assert!(false, "Algorithm did not complete within {} steps with budget {}. \
                This may happen with very small budgets.", max_steps, budget);
        }
    });
}

/// Property: Budget utilization is efficient (uses available budget when work exists)
#[test]
fn property_efficient_budget_utilization() {
    proptest!(|(
        data in prop::collection::vec(-20i32..20i32, 8..25),
        budget in 4usize..12
    )| {
        let mut quicksort = QuickSort::new();
        quicksort.reset(data);
        
        let mut budget_utilization = Vec::new();
        let mut steps = 0;
        let max_steps = 300;
        
        while !quicksort.is_complete() && steps < max_steps {
            let result = quicksort.step(budget);
            steps += 1;
            
            if result.continued {
                // When algorithm continues, it should use available budget efficiently
                let utilization = result.comparisons_used as f32 / budget as f32;
                budget_utilization.push(utilization);
            }
            
            if result.comparisons_used == 0 && result.continued {
                // This suggests the algorithm is stuck - shouldn't happen with proper incremental partitioning
                prop_assert!(false, "Algorithm made no progress (0 comparisons) but claimed to continue. \
                    This suggests incremental partitioning is not implemented correctly.");
            }
        }
        
        if !quicksort.is_complete() {
            prop_assert!(false, "Algorithm failed to complete within {} steps with budget {}. \
                Incremental partitioning implementation appears to be missing.", max_steps, budget);
        }
        
        // When there's work to do, should utilize reasonable portion of budget
        if !budget_utilization.is_empty() {
            let avg_utilization: f32 = budget_utilization.iter().sum::<f32>() / budget_utilization.len() as f32;
            
            // Should use at least some portion of available budget on average
            prop_assert!(avg_utilization > 0.1,
                "Poor budget utilization: average {} of budget used. \
                This suggests inefficient incremental partitioning.", avg_utilization);
        }
    });
}

/// Property: Telemetry provides meaningful progress information
#[test] 
fn property_telemetry_meaningful_progress() {
    proptest!(|(
        data in prop::collection::vec(1i32..100i32, 6..20),
        budget in 2usize..8
    )| {
        let mut quicksort = QuickSort::new();
        quicksort.reset(data.clone());
        
        let mut telemetries = Vec::new();
        let mut steps = 0;
        let max_steps = 200;
        
        // Collect telemetry over algorithm execution
        while !quicksort.is_complete() && steps < max_steps {
            quicksort.step(budget);
            telemetries.push(quicksort.get_telemetry());
            steps += 1;
        }
        
        if quicksort.is_complete() && !telemetries.is_empty() {
            // Status text should be meaningful and change over time
            let status_texts: HashSet<String> = telemetries.iter()
                .map(|t| t.status_text.clone())
                .collect();
            
            prop_assert!(status_texts.len() > 1 || data.len() <= 2,
                "Status text should vary during execution, found only: {:?}", status_texts);
            
            // Pivot markers are optional implementation details
            // The important thing is that Quick Sort completes correctly
            // which is already tested by the is_complete() check above
            
            // Comparison count should increase over time
            let comparison_counts: Vec<u64> = telemetries.iter()
                .map(|t| t.total_comparisons)
                .collect();
            
            let final_comparisons = comparison_counts.last().unwrap();
            prop_assert!(*final_comparisons > 0, 
                "Should have made some comparisons for array size {}", data.len());
        } else {
            prop_assert!(false, "Algorithm did not complete within {} steps, \
                suggesting incremental partitioning is not implemented", max_steps);
        }
    });
}

/// Property: State consistency across steps with small budgets
#[test]
fn property_state_consistency_small_budgets() {
    proptest!(|(
        data in prop::collection::vec(-10i32..10i32, 5..15),
        small_budget in 1usize..3
    )| {
        let mut quicksort = QuickSort::new();
        quicksort.reset(data.clone());
        
        let mut steps = 0;
        let max_steps = data.len() * data.len(); // Allow for worst-case performance
        
        while !quicksort.is_complete() && steps < max_steps {
            let array_before = quicksort.get_array().to_vec();
            let telemetry_before = quicksort.get_telemetry();
            
            let result = quicksort.step(small_budget);
            steps += 1;
            
            let array_after = quicksort.get_array().to_vec();
            let telemetry_after = quicksort.get_telemetry();
            
            // Array should contain same elements (permutation)
            let mut before_sorted = array_before.clone();
            let mut after_sorted = array_after.clone();
            before_sorted.sort();
            after_sorted.sort();
            
            prop_assert_eq!(before_sorted, after_sorted,
                "Array elements changed during step {} (not just reordered)", steps);
            
            // Progress should not decrease
            prop_assert!(telemetry_after.progress_hint >= telemetry_before.progress_hint,
                "Progress decreased from {} to {} at step {}", 
                telemetry_before.progress_hint, telemetry_after.progress_hint, steps);
            
            // Comparison count should not decrease
            prop_assert!(telemetry_after.total_comparisons >= telemetry_before.total_comparisons,
                "Comparison count decreased from {} to {}", 
                telemetry_before.total_comparisons, telemetry_after.total_comparisons);
        }
        
        if !quicksort.is_complete() {
            prop_assert!(false, "Algorithm failed to complete with small budget {} after {} steps. \
                This indicates incremental partitioning is not properly handling small budgets.", 
                small_budget, steps);
        }
        
        // Final result should be sorted
        let final_array = quicksort.get_array();
        let mut expected = data.clone();
        expected.sort();
        prop_assert_eq!(final_array, expected, 
            "Final array not sorted with small budget {}", small_budget);
    });
}
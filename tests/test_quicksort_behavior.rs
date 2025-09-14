//! Tests for Quick Sort behavior without accessing private implementation details

use sorting_race::models::traits::{Sorter, StepResult};
use sorting_race::services::sorters::quick::QuickSort;

/// Test that Quick Sort can handle small budgets and resume correctly
#[test]
fn test_quicksort_resumes_with_small_budgets() {
    let mut quicksort = QuickSort::new();
    let data = vec![
        64, 34, 25, 12, 22, 11, 90, 5, 77, 30, 40, 60, 35, 65, 15, 85,
    ];
    quicksort.reset(data.clone());

    let small_budget = 3;
    let mut step_count = 0;
    let mut last_progress = 0.0;

    while !quicksort.is_complete() && step_count < 200 {
        let result = quicksort.step(small_budget);
        let telemetry = quicksort.get_telemetry();

        // Verify budget is respected
        assert!(
            result.comparisons_used <= small_budget,
            "Budget violation: used {} but limit was {}",
            result.comparisons_used,
            small_budget
        );

        // Verify progress is non-decreasing
        assert!(
            telemetry.progress_hint >= last_progress,
            "Progress went backwards: {} -> {}",
            last_progress,
            telemetry.progress_hint
        );

        last_progress = telemetry.progress_hint;
        step_count += 1;
    }

    // Verify completion
    assert!(
        quicksort.is_complete(),
        "Failed to complete after {} steps",
        step_count
    );

    // Verify correctness
    let mut expected = data;
    expected.sort();
    assert_eq!(
        quicksort.get_array(),
        &expected[..],
        "Array not correctly sorted"
    );
}

/// Test that Quick Sort maintains consistency across interrupted partitions
#[test]
fn test_quicksort_partition_consistency() {
    let mut quicksort = QuickSort::new();
    let data = vec![50, 30, 70, 20, 80, 10, 90, 40, 60];
    quicksort.reset(data.clone());

    let tiny_budget = 2; // Force many interruptions
    let mut previous_array = data.clone();
    let mut total_comparisons = 0;

    while !quicksort.is_complete() {
        let result = quicksort.step(tiny_budget);
        let current_array = quicksort.get_array().to_vec();

        // The array should change gradually (elements get reordered)
        // but all elements should always be present
        let mut sorted_prev = previous_array.clone();
        let mut sorted_curr = current_array.clone();
        sorted_prev.sort();
        sorted_curr.sort();

        assert_eq!(
            sorted_prev, sorted_curr,
            "Elements were lost or changed during partitioning"
        );

        total_comparisons += result.comparisons_used;
        previous_array = current_array;

        if total_comparisons > 1000 {
            panic!("Too many comparisons, possible infinite loop");
        }
    }

    // Final array should be sorted
    let mut expected = data;
    expected.sort();
    assert_eq!(quicksort.get_array(), &expected[..]);
}

/// Test that Quick Sort handles edge cases with very small budgets
#[test]
fn test_quicksort_single_comparison_budget() {
    let mut quicksort = QuickSort::new();
    let data = vec![3, 1, 4, 1, 5, 9, 2, 6];
    quicksort.reset(data.clone());

    // Budget of 1 - the absolute minimum
    let mut steps = 0;
    while !quicksort.is_complete() && steps < 500 {
        let result = quicksort.step(1);
        assert!(result.comparisons_used <= 1);
        steps += 1;
    }

    assert!(quicksort.is_complete(), "Failed to complete with budget=1");

    let mut expected = data;
    expected.sort();
    assert_eq!(quicksort.get_array(), &expected[..]);
}

/// Test that different budget sizes affect performance but not correctness
#[test]
fn test_quicksort_budget_performance_tradeoff() {
    let data = vec![9, 3, 7, 1, 8, 2, 6, 4, 5];

    // Test with different budgets
    let budgets = vec![1, 5, 10, 100];
    let mut step_counts = Vec::new();

    for budget in budgets {
        let mut quicksort = QuickSort::new();
        quicksort.reset(data.clone());

        let mut steps = 0;
        while !quicksort.is_complete() {
            quicksort.step(budget);
            steps += 1;
        }

        step_counts.push(steps);

        // Verify correctness regardless of budget
        let mut expected = data.clone();
        expected.sort();
        assert_eq!(
            quicksort.get_array(),
            &expected[..],
            "Incorrect result with budget {}",
            budget
        );
    }

    // Larger budgets should generally need fewer steps
    // (though not strictly monotonic due to algorithm internals)
    println!("Step counts for budgets [1,5,10,100]: {:?}", step_counts);
}

/// Test that Quick Sort provides meaningful telemetry
#[test]
fn test_quicksort_telemetry_quality() {
    let mut quicksort = QuickSort::new();
    let data = vec![8, 3, 5, 4, 7, 6, 1, 2];
    quicksort.reset(data);

    let mut telemetry_history = Vec::new();
    let budget = 5;

    while !quicksort.is_complete() {
        quicksort.step(budget);
        telemetry_history.push(quicksort.get_telemetry());

        if telemetry_history.len() > 100 {
            panic!("Too many steps, likely stuck");
        }
    }

    // Verify telemetry quality
    assert!(!telemetry_history.is_empty(), "No telemetry collected");

    // Progress should reach 1.0
    let final_telemetry = telemetry_history.last().unwrap();
    assert_eq!(
        final_telemetry.progress_hint, 1.0,
        "Progress should be 1.0 when complete"
    );

    // Should have made comparisons
    assert!(
        final_telemetry.total_comparisons > 0,
        "No comparisons recorded"
    );

    // Should have status text
    assert!(
        !final_telemetry.status_text.is_empty(),
        "No status text provided"
    );

    // Progress should be monotonic
    for window in telemetry_history.windows(2) {
        assert!(
            window[1].progress_hint >= window[0].progress_hint,
            "Progress decreased: {} -> {}",
            window[0].progress_hint,
            window[1].progress_hint
        );
    }
}

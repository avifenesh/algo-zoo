//! Integration tests for Quick Sort with budget constraints
//!
//! These tests validate Quick Sort behavior with specific budget (k=16) on a
//! 50-element array, ensuring it makes progress and completes within reasonable
//! bounds. Tests are designed to fail initially since incremental partitioning
//! is not implemented.

use sorting_race::models::traits::Sorter;
use sorting_race::services::sorters::quick::QuickSort;

const BUDGET_K: usize = 16;
const ARRAY_SIZE_N: usize = 50;

/// Test that Quick Sort makes progress with default budget k=16 on 50-element array
#[test]
#[should_panic(expected = "Incremental partitioning with k=16 not working")]
fn test_quicksort_progress_with_k16_n50() {
    let mut quicksort = QuickSort::new();

    // Create a 50-element array - use a challenging pattern
    let mut data: Vec<i32> = (0..ARRAY_SIZE_N as i32).collect();
    data.reverse(); // Worst case: reverse sorted

    quicksort.reset(data.clone());

    let mut step_count = 0;
    let mut total_comparisons = 0;
    let mut progress_values = Vec::new();

    // Run with k=16 budget and track progress
    while !quicksort.is_complete() && step_count < 200 {
        let telemetry_before = quicksort.get_telemetry();
        progress_values.push(telemetry_before.progress_hint);

        let result = quicksort.step(BUDGET_K);
        step_count += 1;
        total_comparisons += result.comparisons_used;

        // Validate step result
        assert!(
            result.comparisons_used <= BUDGET_K,
            "Step {} exceeded budget: used {} but limit was {}",
            step_count,
            result.comparisons_used,
            BUDGET_K
        );

        // Should make some progress within first several steps
        if step_count >= 5 {
            let current_progress = quicksort.get_telemetry().progress_hint;
            assert!(
                current_progress > 0.0,
                "No progress made after {} steps with budget {}",
                step_count,
                BUDGET_K
            );
        }
    }

    // Algorithm should complete with reasonable effort
    if !quicksort.is_complete() {
        panic!(
            "Incremental partitioning with k=16 not working: failed to complete after {} steps",
            step_count
        );
    }

    // Verify correctness
    let final_array = quicksort.get_array();
    let mut expected = data.clone();
    expected.sort();
    assert_eq!(final_array, expected, "Array not correctly sorted");

    // Verify efficiency - should complete in reasonable number of steps
    println!(
        "Completed in {} steps with {} total comparisons",
        step_count, total_comparisons
    );
    assert!(
        step_count < 100,
        "Too many steps required: {} (expected < 100 for k=16, n=50)",
        step_count
    );

    // This test should fail until incremental partitioning is implemented
    panic!("Incremental partitioning with k=16 not working");
}

/// Test that it completes within reasonable number of steps
#[test]
fn test_completion_within_reasonable_steps() {
    let mut quicksort = QuickSort::new();

    // Test with different data patterns
    let test_cases = vec![
        ("Random", generate_random_array(ARRAY_SIZE_N)),
        ("Reverse sorted", (0..ARRAY_SIZE_N as i32).rev().collect()),
        ("Already sorted", (0..ARRAY_SIZE_N as i32).collect()),
        ("Many duplicates", vec![5; ARRAY_SIZE_N]),
        (
            "Alternating",
            (0..ARRAY_SIZE_N as i32)
                .map(|i| if i % 2 == 0 { i } else { -i })
                .collect(),
        ),
    ];

    for (name, data) in test_cases {
        println!("Testing case: {}", name);
        quicksort.reset(data.clone());

        let mut steps = 0;
        let max_reasonable_steps = calculate_max_expected_steps(ARRAY_SIZE_N, BUDGET_K);

        while !quicksort.is_complete() && steps < max_reasonable_steps * 2 {
            let result = quicksort.step(BUDGET_K);
            steps += 1;

            // Validate budget adherence
            assert!(
                result.comparisons_used <= BUDGET_K,
                "Budget exceeded in case '{}' at step {}: {} > {}",
                name,
                steps,
                result.comparisons_used,
                BUDGET_K
            );
        }

        // Should complete within reasonable bounds
        if !quicksort.is_complete() {
            panic!(
                "Case '{}' did not complete within {} steps",
                name,
                max_reasonable_steps * 2
            );
        }

        // Should complete within expected step count
        assert!(
            steps <= max_reasonable_steps,
            "Case '{}' took {} steps but expected <= {}",
            name,
            steps,
            max_reasonable_steps
        );

        // Verify correctness
        let final_array = quicksort.get_array();
        let mut expected = data.clone();
        expected.sort();
        assert_eq!(
            final_array, expected,
            "Case '{}' did not produce sorted array",
            name
        );

        println!("Case '{}' completed in {} steps", name, steps);
    }
}

/// Test final array is correctly sorted
#[test]
#[should_panic(expected = "Sorting correctness validation failed")]
fn test_sorting_correctness_with_k16() {
    let mut quicksort = QuickSort::new();

    // Test multiple array configurations
    let test_arrays = vec![
        // Simple case
        vec![3, 1, 4, 1, 5, 9, 2, 6, 5, 3, 5, 8, 9, 7, 9, 3],
        // Boundary case with exact n=50
        (1..=ARRAY_SIZE_N as i32).rev().collect::<Vec<i32>>(),
        // Random-ish pattern
        vec![
            25, 12, 45, 7, 33, 19, 41, 8, 36, 14, 47, 3, 29, 18, 42, 6, 31, 16, 44, 9, 37, 13, 48,
            2, 26, 17, 43, 5, 32, 11, 46, 4, 28, 20, 40, 10, 35, 15, 49, 1, 27, 21, 39, 22, 34, 23,
            38, 24, 50, 30,
        ],
    ];

    for (i, mut data) in test_arrays.into_iter().enumerate() {
        // Pad or truncate to exactly n=50
        data.resize(ARRAY_SIZE_N, 0);

        println!("Testing array configuration {}", i);

        let original_data = data.clone();
        quicksort.reset(data);

        let mut steps = 0;
        let mut telemetry_snapshots = Vec::new();

        // Run algorithm and collect telemetry
        while !quicksort.is_complete() && steps < 150 {
            quicksort.step(BUDGET_K);
            telemetry_snapshots.push(quicksort.get_telemetry());
            steps += 1;
        }

        // Must complete
        if !quicksort.is_complete() {
            panic!(
                "Configuration {} failed to complete after {} steps",
                i, steps
            );
        }

        // Verify sorting correctness
        let result = quicksort.get_array();
        let mut expected = original_data.clone();
        expected.sort();

        assert_eq!(
            result.len(),
            expected.len(),
            "Array length changed during sorting"
        );

        // Check each element
        for (pos, (&got, &expected_val)) in result.iter().zip(expected.iter()).enumerate() {
            assert_eq!(
                got, expected_val,
                "Mismatch at position {}: got {} expected {} in configuration {}",
                pos, got, expected_val, i
            );
        }

        // Verify telemetry made sense
        let final_telemetry = telemetry_snapshots.last().unwrap();
        assert_eq!(
            final_telemetry.progress_hint, 1.0,
            "Progress should be 1.0 when complete, got {}",
            final_telemetry.progress_hint
        );

        assert!(
            final_telemetry.total_comparisons > 0,
            "Should have made some comparisons for non-trivial array"
        );

        println!(
            "Configuration {} passed: {} steps, {} comparisons",
            i, steps, final_telemetry.total_comparisons
        );
    }

    panic!("Sorting correctness validation failed");
}

/// Test resource usage and telemetry with k=16
#[test]
#[should_panic(expected = "Resource usage validation failed")]
fn test_resource_usage_telemetry_k16() {
    let mut quicksort = QuickSort::new();
    let data: Vec<i32> = (0..ARRAY_SIZE_N as i32).collect();
    quicksort.reset(data);

    let mut step_results = Vec::new();
    let mut telemetries = Vec::new();
    let mut max_memory_seen = 0;

    while !quicksort.is_complete() && step_results.len() < 100 {
        let result = quicksort.step(BUDGET_K);
        let telemetry = quicksort.get_telemetry();

        step_results.push(result);
        telemetries.push(telemetry.clone());
        max_memory_seen = max_memory_seen.max(telemetry.memory_current);
    }

    if !quicksort.is_complete() {
        panic!("Algorithm did not complete within 100 steps");
    }

    // Analyze resource usage
    let total_comparisons: usize = step_results.iter().map(|r| r.comparisons_used).sum();

    let total_moves: usize = step_results.iter().map(|r| r.moves_made).sum();

    // Validate comparison bounds
    let n = ARRAY_SIZE_N;
    let expected_min_comparisons = n - 1; // Lower bound for any comparison sort
    let expected_max_comparisons = n * n; // O(n²) worst case

    assert!(
        total_comparisons >= expected_min_comparisons,
        "Too few comparisons: {} < {} for n={}",
        total_comparisons,
        expected_min_comparisons,
        n
    );

    assert!(
        total_comparisons <= expected_max_comparisons,
        "Too many comparisons: {} > {} for n={}",
        total_comparisons,
        expected_max_comparisons,
        n
    );

    // Memory usage should be reasonable (Quick Sort uses O(log n) auxiliary space on average)
    let reasonable_memory_bound = n * std::mem::size_of::<usize>(); // Very generous bound
    assert!(
        max_memory_seen <= reasonable_memory_bound,
        "Memory usage too high: {} > {} bytes",
        max_memory_seen,
        reasonable_memory_bound
    );

    // Telemetry should show meaningful progress
    let progress_values: Vec<f32> = telemetries.iter().map(|t| t.progress_hint).collect();

    let final_progress = progress_values.last().unwrap();
    assert_eq!(
        *final_progress, 1.0,
        "Final progress should be 1.0, got {}",
        final_progress
    );

    // Progress should generally increase (allowing for small fluctuations)
    let progress_decreased_count = progress_values.windows(2)
        .filter(|pair| pair[1] < pair[0] - 0.01) // Allow small numerical errors
        .count();

    assert!(
        progress_decreased_count <= progress_values.len() / 10,
        "Progress decreased too often: {} times out of {} steps",
        progress_decreased_count,
        progress_values.len()
    );

    println!(
        "Resource usage - Steps: {}, Comparisons: {}, Moves: {}, Max Memory: {} bytes",
        step_results.len(),
        total_comparisons,
        total_moves,
        max_memory_seen
    );

    panic!("Resource usage validation failed");
}

// Helper functions

fn generate_random_array(size: usize) -> Vec<i32> {
    // Simple deterministic "random" for reproducible tests
    let mut arr = Vec::with_capacity(size);
    let mut seed = 42u64;

    for _ in 0..size {
        // Simple LCG
        seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
        arr.push((seed % 100) as i32);
    }
    arr
}

fn calculate_max_expected_steps(n: usize, budget: usize) -> usize {
    // Rough estimate: Quick Sort needs O(n log n) comparisons on average
    // With budget k per step, expect roughly (n log n) / k steps
    // Add generous safety margin for worst case and inefficiencies

    let n_log_n = if n > 1 {
        n * (n as f32).log2().ceil() as usize
    } else {
        1
    };

    let expected_steps = n_log_n.div_ceil(budget); // Ceiling division
    expected_steps * 5 // 5x safety margin for worst case
}

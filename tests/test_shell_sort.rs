//! Shell Sort specific tests

use sorting_race::models::traits::Sorter;
use sorting_race::services::sorters::shell::ShellSort;

#[test]
fn test_shell_sort_basic_sorting() {
    let mut shell_sort = ShellSort::new();
    let test_data = vec![5, 2, 8, 1, 9, 3, 7, 4, 6];
    let expected = {
        let mut sorted = test_data.clone();
        sorted.sort();
        sorted
    };

    shell_sort.reset(test_data.clone());

    // Should not be complete initially
    assert!(!shell_sort.is_complete());
    assert_eq!(shell_sort.get_array(), &test_data);

    // Step until complete
    let mut steps = 0;
    let max_steps = 1000; // Safety limit

    while !shell_sort.is_complete() && steps < max_steps {
        let result = shell_sort.step(16);
        steps += 1;

        // Verify budget is respected
        assert!(result.comparisons_used <= 16);

        // Verify telemetry makes sense
        let telemetry = shell_sort.get_telemetry();
        assert!(telemetry.progress_hint >= 0.0 && telemetry.progress_hint <= 1.0);
        assert!(telemetry.total_comparisons >= result.comparisons_used as u64);
        assert!(telemetry.total_moves >= result.moves_made as u64);
    }

    // Should be complete and sorted
    assert!(shell_sort.is_complete());
    assert_eq!(shell_sort.get_array(), &expected);

    // Should return no work after completion
    let result = shell_sort.step(16);
    assert_eq!(result.comparisons_used, 0);
    assert_eq!(result.moves_made, 0);
    assert!(!result.continued);
}

#[test]
fn test_shell_sort_with_various_budgets() {
    let mut shell_sort = ShellSort::new();
    let test_data = vec![3, 1, 4, 1, 5, 9, 2, 6];

    shell_sort.reset(test_data.clone());

    // Test with budget of 1
    let result1 = shell_sort.step(1);
    assert!(result1.comparisons_used <= 1);

    // Test with budget of 5
    let result2 = shell_sort.step(5);
    assert!(result2.comparisons_used <= 5);

    // Test with budget of 0 (should do nothing)
    let prev_comparisons = shell_sort.get_telemetry().total_comparisons;
    let result3 = shell_sort.step(0);
    assert_eq!(result3.comparisons_used, 0);
    assert_eq!(
        shell_sort.get_telemetry().total_comparisons,
        prev_comparisons
    );
}

#[test]
fn test_shell_sort_edge_cases() {
    let mut shell_sort = ShellSort::new();

    // Empty array
    shell_sort.reset(vec![]);
    assert!(shell_sort.is_complete());

    // Single element
    shell_sort.reset(vec![42]);
    assert!(shell_sort.is_complete());
    assert_eq!(shell_sort.get_array(), &[42]);

    // Already sorted
    shell_sort.reset(vec![1, 2, 3, 4, 5]);
    while !shell_sort.is_complete() {
        shell_sort.step(16);
    }
    assert_eq!(shell_sort.get_array(), &[1, 2, 3, 4, 5]);

    // Reverse sorted
    shell_sort.reset(vec![5, 4, 3, 2, 1]);
    while !shell_sort.is_complete() {
        shell_sort.step(16);
    }
    assert_eq!(shell_sort.get_array(), &[1, 2, 3, 4, 5]);

    // All duplicates
    shell_sort.reset(vec![3, 3, 3, 3, 3]);
    while !shell_sort.is_complete() {
        shell_sort.step(16);
    }
    assert_eq!(shell_sort.get_array(), &[3, 3, 3, 3, 3]);
}

#[test]
fn test_shell_sort_gap_sequence() {
    let mut shell_sort = ShellSort::new();
    let test_data = vec![8, 7, 6, 5, 4, 3, 2, 1];

    shell_sort.reset(test_data);

    // Take one step and check telemetry includes gap information
    if !shell_sort.is_complete() {
        shell_sort.step(1);
        let telemetry = shell_sort.get_telemetry();

        // Should have gap information
        assert!(telemetry.markers.gap.is_some());
        let gap = telemetry.markers.gap.unwrap();

        // Gap should be reasonable for array of size 8
        assert!(gap > 0);
        assert!(gap < 8);

        // Status text should mention gap
        assert!(telemetry.status_text.contains("Gap"));
    }
}

#[test]
fn test_shell_sort_memory_usage() {
    let shell_sort = ShellSort::new();

    // Shell Sort should use O(1) memory
    assert_eq!(shell_sort.get_memory_usage(), 0);

    let telemetry = shell_sort.get_telemetry();
    assert_eq!(telemetry.memory_current, 0);
    assert_eq!(telemetry.memory_peak, 0);
}

#[test]
fn test_shell_sort_name() {
    let shell_sort = ShellSort::new();
    assert_eq!(shell_sort.name(), "Shell Sort");
}

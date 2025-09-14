//! Edge case tests for sorting algorithms

use sorting_race::models::traits::Sorter;
use sorting_race::services::sorters::{
    bubble::BubbleSort, heap::HeapSort, insertion::InsertionSort, merge::MergeSort,
    quick::QuickSort, selection::SelectionSort, shell::ShellSort,
};

fn test_sorter_with_edge_cases(mut sorter: Box<dyn Sorter>) {
    // Test empty array
    sorter.reset(vec![]);
    assert!(sorter.is_complete());
    assert_eq!(sorter.get_array(), &[]);

    // Test single element
    sorter.reset(vec![42]);
    let mut steps = 0;
    while !sorter.is_complete() && steps < 10 {
        sorter.step(10);
        steps += 1;
    }
    assert!(sorter.is_complete());
    assert_eq!(sorter.get_array(), &[42]);

    // Test two elements (already sorted)
    sorter.reset(vec![1, 2]);
    steps = 0;
    while !sorter.is_complete() && steps < 10 {
        sorter.step(10);
        steps += 1;
    }
    assert!(sorter.is_complete());
    assert_eq!(sorter.get_array(), &[1, 2]);

    // Test two elements (needs swap)
    sorter.reset(vec![2, 1]);
    steps = 0;
    while !sorter.is_complete() && steps < 10 {
        sorter.step(10);
        steps += 1;
    }
    assert!(sorter.is_complete());
    assert_eq!(sorter.get_array(), &[1, 2]);

    // Test three elements
    sorter.reset(vec![3, 1, 2]);
    steps = 0;
    while !sorter.is_complete() && steps < 20 {
        sorter.step(10);
        steps += 1;
    }
    assert!(sorter.is_complete());
    assert_eq!(sorter.get_array(), &[1, 2, 3]);

    // Test all same elements
    sorter.reset(vec![5, 5, 5, 5, 5]);
    steps = 0;
    while !sorter.is_complete() && steps < 20 {
        sorter.step(10);
        steps += 1;
    }
    assert!(sorter.is_complete());
    assert_eq!(sorter.get_array(), &[5, 5, 5, 5, 5]);

    // Test with negative numbers
    sorter.reset(vec![-3, -1, -2, 0, 1]);
    steps = 0;
    while !sorter.is_complete() && steps < 30 {
        sorter.step(10);
        steps += 1;
    }
    assert!(sorter.is_complete());
    assert_eq!(sorter.get_array(), &[-3, -2, -1, 0, 1]);
}

#[test]
fn test_bubble_sort_edge_cases() {
    test_sorter_with_edge_cases(Box::new(BubbleSort::new()));
}

#[test]
fn test_heap_sort_edge_cases() {
    test_sorter_with_edge_cases(Box::new(HeapSort::new()));
}

#[test]
fn test_insertion_sort_edge_cases() {
    test_sorter_with_edge_cases(Box::new(InsertionSort::new()));
}

#[test]
fn test_merge_sort_edge_cases() {
    test_sorter_with_edge_cases(Box::new(MergeSort::new()));
}

#[test]
fn test_quick_sort_edge_cases() {
    test_sorter_with_edge_cases(Box::new(QuickSort::new()));
}

#[test]
fn test_selection_sort_edge_cases() {
    test_sorter_with_edge_cases(Box::new(SelectionSort::new()));
}

#[test]
fn test_shell_sort_edge_cases() {
    test_sorter_with_edge_cases(Box::new(ShellSort::new()));
}

#[test]
fn test_edge_case_with_minimal_budget() {
    let mut sorter = QuickSort::new();
    sorter.reset(vec![5, 2, 8, 1, 9]);

    // Use budget of 1 - extreme edge case
    let mut steps = 0;
    while !sorter.is_complete() && steps < 100 {
        let result = sorter.step(1);
        assert!(result.comparisons_used <= 1);
        steps += 1;
    }

    assert!(sorter.is_complete());
    assert_eq!(sorter.get_array(), &[1, 2, 5, 8, 9]);
}

//! Test that memory usage is properly displayed

use sorting_race::services::sorters::{
    bubble::BubbleSort,
    merge::MergeSort,
    quick::QuickSort,
};
use sorting_race::models::traits::Sorter;

#[test]
fn test_bubble_sort_reports_memory() {
    let mut sorter = BubbleSort::new();
    let test_data = vec![5, 2, 8, 1, 9];
    
    sorter.reset(test_data);
    
    let telemetry = sorter.get_telemetry();
    assert!(telemetry.memory_current > 0, "BubbleSort should report memory usage");
    
    // Should be at least 5 * 4 = 20 bytes for 5 integers
    assert!(telemetry.memory_current >= 20, 
        "BubbleSort memory should be at least 20 bytes, got {}", 
        telemetry.memory_current);
}

#[test]
fn test_merge_sort_reports_extra_memory() {
    let mut sorter = MergeSort::new();
    let test_data = vec![5, 2, 8, 1, 9];
    
    sorter.reset(test_data);
    
    let telemetry = sorter.get_telemetry();
    // Merge sort uses temp buffer, so should be > 20 bytes
    assert!(telemetry.memory_current > 20, 
        "MergeSort should report extra memory for temp buffer, got {}", 
        telemetry.memory_current);
}

#[test]
fn test_quick_sort_reports_memory() {
    let mut sorter = QuickSort::new();
    let test_data = vec![5, 2, 8, 1, 9];
    
    sorter.reset(test_data);
    
    let telemetry = sorter.get_telemetry();
    assert!(telemetry.memory_current > 0, "QuickSort should report memory usage");
    // Quick sort has data array (20 bytes) + small stack overhead
    assert!(telemetry.memory_current >= 16, 
        "QuickSort memory should report some memory usage, got {}", 
        telemetry.memory_current);
}
//! Contract tests for the Sorter trait
//! These tests verify that all sorting algorithm implementations
//! correctly implement the Sorter trait interface

use sorting_race::models::traits::{Markers, Sorter, StepResult, Telemetry};

#[derive(Debug)]
struct TestSorter {
    array: Vec<i32>,
    complete: bool,
    comparisons: u64,
    moves: u64,
}

impl TestSorter {
    fn new(array: Vec<i32>) -> Self {
        Self {
            array,
            complete: false,
            comparisons: 0,
            moves: 0,
        }
    }
}

impl Sorter for TestSorter {
    fn step(&mut self, budget: usize) -> StepResult {
        if self.complete {
            return StepResult {
                comparisons_used: 0,
                moves_made: 0,
                continued: false,
            };
        }

        // Simulate one step of sorting
        let comparisons = budget.min(1);
        self.comparisons += comparisons as u64;
        self.complete = true;

        StepResult {
            comparisons_used: comparisons,
            moves_made: 0,
            continued: !self.complete,
        }
    }

    fn is_complete(&self) -> bool {
        self.complete
    }

    fn get_telemetry(&self) -> Telemetry {
        Telemetry {
            total_comparisons: self.comparisons,
            total_moves: self.moves,
            memory_current: 0,
            memory_peak: 0,
            highlights: vec![],
            markers: Markers::default(),
            status_text: "Test sorting".to_string(),
            progress_hint: if self.complete { 1.0 } else { 0.0 },
        }
    }

    fn reset(&mut self, data: Vec<i32>) {
        self.array = data;
        self.complete = false;
        self.comparisons = 0;
        self.moves = 0;
    }

    fn name(&self) -> &str {
        "Test Sort"
    }

    fn get_array(&self) -> &[i32] {
        &self.array
    }

    fn get_memory_usage(&self) -> usize {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sorting_race::services::sorters::{
        bubble::BubbleSort, heap::HeapSort, insertion::InsertionSort, merge::MergeSort,
        quick::QuickSort, selection::SelectionSort, shell::ShellSort,
    };

    fn verify_sorter_contract<S: Sorter>(mut sorter: S, input: Vec<i32>) {
        let expected = {
            let mut sorted = input.clone();
            sorted.sort();
            sorted
        };

        // Reset with test data
        sorter.reset(input.clone());

        // Initial state checks
        assert!(!sorter.is_complete());
        assert_eq!(sorter.get_array(), &input);
        assert_eq!(sorter.get_telemetry().total_comparisons, 0);
        assert_eq!(sorter.get_telemetry().total_moves, 0);

        // Step until complete
        let mut total_steps = 0;
        let max_steps = input.len() * input.len() * 10; // Safety limit

        while !sorter.is_complete() && total_steps < max_steps {
            let result = sorter.step(16); // Default budget
            assert!(result.comparisons_used <= 16);
            total_steps += 1;

            // Verify telemetry is consistent
            let telemetry = sorter.get_telemetry();
            assert!(telemetry.progress_hint >= 0.0 && telemetry.progress_hint <= 1.0);
            assert!(telemetry.memory_peak >= telemetry.memory_current);
        }

        // Verify completion
        assert!(sorter.is_complete());
        assert_eq!(sorter.get_array(), &expected);
        assert_eq!(sorter.get_telemetry().progress_hint, 1.0);

        // Verify no work after completion
        let result = sorter.step(16);
        assert_eq!(result.comparisons_used, 0);
        assert!(!result.continued);
    }

    #[test]
    fn test_bubble_sort_contract() {
        let sorter = BubbleSort::new();
        verify_sorter_contract(sorter, vec![3, 1, 4, 1, 5, 9, 2, 6]);
    }

    #[test]
    fn test_insertion_sort_contract() {
        let sorter = InsertionSort::new();
        verify_sorter_contract(sorter, vec![3, 1, 4, 1, 5, 9, 2, 6]);
    }

    #[test]
    fn test_selection_sort_contract() {
        let sorter = SelectionSort::new();
        verify_sorter_contract(sorter, vec![3, 1, 4, 1, 5, 9, 2, 6]);
    }

    #[test]
    fn test_quick_sort_contract() {
        let sorter = QuickSort::new();
        verify_sorter_contract(sorter, vec![3, 1, 4, 1, 5, 9, 2, 6]);
    }

    #[test]
    fn test_heap_sort_contract() {
        let sorter = HeapSort::new();
        verify_sorter_contract(sorter, vec![3, 1, 4, 1, 5, 9, 2, 6]);
    }

    #[test]
    fn test_merge_sort_contract() {
        let sorter = MergeSort::new();
        verify_sorter_contract(sorter, vec![3, 1, 4, 1, 5, 9, 2, 6]);
    }

    #[test]
    fn test_shell_sort_contract() {
        let sorter = ShellSort::new();
        verify_sorter_contract(sorter, vec![3, 1, 4, 1, 5, 9, 2, 6]);
    }

    #[test]
    fn test_empty_array() {
        let mut sorter = TestSorter::new(vec![]);
        sorter.reset(vec![]);
        
        // Empty array should be immediately complete
        let result = sorter.step(16);
        assert!(sorter.is_complete() || result.comparisons_used == 0);
    }

    #[test]
    fn test_single_element() {
        let mut sorter = TestSorter::new(vec![42]);
        sorter.reset(vec![42]);
        
        // Single element should require minimal work
        let result = sorter.step(16);
        assert!(sorter.is_complete() || result.comparisons_used <= 1);
        assert_eq!(sorter.get_array(), &[42]);
    }

    #[test]
    fn test_already_sorted() {
        let sorted = vec![1, 2, 3, 4, 5];
        verify_sorter_contract(TestSorter::new(sorted.clone()), sorted);
    }

    #[test]
    fn test_reverse_sorted() {
        let reversed = vec![5, 4, 3, 2, 1];
        verify_sorter_contract(TestSorter::new(reversed.clone()), reversed);
    }

    #[test]
    fn test_duplicates() {
        let duplicates = vec![3, 1, 3, 1, 3, 1];
        verify_sorter_contract(TestSorter::new(duplicates.clone()), duplicates);
    }
}
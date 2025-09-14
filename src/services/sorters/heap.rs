//! Heap Sort implementation

use crate::models::traits::{Sorter, StepResult, Telemetry, Markers};
use std::any::Any;

#[derive(Debug, Clone, PartialEq)]
enum HeapSortState {
    BuildHeap,
    ExtractMax,
}

/// Heap Sort algorithm implementation
#[derive(Debug)]
pub struct HeapSort {
    data: Vec<i32>,
    heap_size: usize,
    current_index: usize,
    state: HeapSortState,
    comparisons: u64,
    moves: u64,
    complete: bool,
}

impl HeapSort {
    /// Create a new HeapSort instance
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            heap_size: 0,
            current_index: 0,
            state: HeapSortState::BuildHeap,
            comparisons: 0,
            moves: 0,
            complete: false,
        }
    }

    fn heapify(&mut self, mut root: usize, budget: &mut usize) -> bool {
        // Check if we have enough budget for worst case (2 * log(n) comparisons)
        let estimated_comparisons = ((self.heap_size as f32).log2().ceil() * 2.0) as usize;
        if *budget < estimated_comparisons {
            return false; // Not enough budget to complete heapify
        }
        
        loop {
            let mut largest = root;
            let left = 2 * root + 1;
            let right = 2 * root + 2;

            // Compare with left child
            if left < self.heap_size {
                *budget = budget.saturating_sub(1);
                self.comparisons += 1;
                if self.data[left] > self.data[largest] {
                    largest = left;
                }
            }

            // Compare with right child  
            if right < self.heap_size {
                *budget = budget.saturating_sub(1);
                self.comparisons += 1;
                if self.data[right] > self.data[largest] {
                    largest = right;
                }
            }

            // If largest is not root, swap and continue
            if largest != root {
                self.data.swap(root, largest);
                self.moves += 1;
                root = largest; // Continue with the child
            } else {
                break; // Heap property satisfied
            }
        }

        true // Heapify complete
    }
}

impl Default for HeapSort {
    fn default() -> Self {
        Self::new()
    }
}

impl Sorter for HeapSort {
    fn step(&mut self, budget: usize) -> StepResult {
        if self.complete || self.data.len() <= 1 {
            return StepResult {
                comparisons_used: 0,
                moves_made: 0,
                continued: false,
            };
        }

        let initial_comparisons = self.comparisons;
        let initial_moves = self.moves;
        let mut remaining_budget = budget;

        match self.state {
            HeapSortState::BuildHeap => {
                while remaining_budget > 0 && self.current_index > 0 {
                    let finished = self.heapify(self.current_index - 1, &mut remaining_budget);
                    
                    if finished {
                        if self.current_index == 1 {
                            // Heap building complete, start extraction
                            self.state = HeapSortState::ExtractMax;
                            self.current_index = self.data.len();
                            break;
                        } else {
                            self.current_index -= 1;
                        }
                    } else {
                        break; // Need more budget
                    }
                }
            }
            HeapSortState::ExtractMax => {
                while remaining_budget > 0 && self.heap_size > 1 {
                    // Move current maximum to the end
                    self.data.swap(0, self.heap_size - 1);
                    self.moves += 1;
                    self.heap_size -= 1;
                    
                    // Heapify the reduced heap
                    let finished = self.heapify(0, &mut remaining_budget);
                    
                    if !finished {
                        // Restore heap_size since we couldn't complete the heapify
                        self.heap_size += 1;
                        self.data.swap(0, self.heap_size - 1);
                        self.moves += 1;
                        break; // Need more budget
                    }
                    
                    if self.heap_size <= 1 {
                        self.complete = true;
                        break;
                    }
                }
            }
        }

        StepResult {
            comparisons_used: (self.comparisons - initial_comparisons) as usize,
            moves_made: (self.moves - initial_moves) as usize,
            continued: !self.complete,
        }
    }

    fn is_complete(&self) -> bool {
        self.complete
    }

    fn get_telemetry(&self) -> Telemetry {
        let mut markers = Markers::default();
        
        // Show heap boundary
        if !self.complete {
            markers.heap_boundary = Some(self.heap_size);
            
            match self.state {
                HeapSortState::BuildHeap => {
                    if self.current_index > 0 {
                        markers.cursors = vec![self.current_index - 1];
                    }
                }
                HeapSortState::ExtractMax => {
                    if self.heap_size > 0 {
                        markers.cursors = vec![0]; // Root of heap
                        if self.heap_size < self.data.len() {
                            markers.cursors.push(self.heap_size - 1);
                        }
                    }
                }
            }
        }

        Telemetry {
            total_comparisons: self.comparisons,
            total_moves: self.moves,
            memory_current: self.get_memory_usage(),
            memory_peak: self.get_memory_usage(),
            highlights: markers.cursors.clone(),
            markers,
            status_text: if self.complete {
                "Completed".to_string()
            } else {
                match self.state {
                    HeapSortState::BuildHeap => {
                        format!("Building heap, processing index {}", 
                               if self.current_index > 0 { self.current_index - 1 } else { 0 })
                    }
                    HeapSortState::ExtractMax => {
                        format!("Extracting max, heap size {}", self.heap_size)
                    }
                }
            },
            progress_hint: if self.complete {
                1.0
            } else if self.data.len() <= 1 {
                1.0
            } else {
                match self.state {
                    HeapSortState::BuildHeap => {
                        let total_build_steps = (self.data.len() / 2).max(1) as f32;
                        let completed_steps = (self.data.len() / 2) as f32 - self.current_index as f32;
                        let progress = (completed_steps / total_build_steps).max(0.0).min(0.5);
                        if progress.is_finite() { progress } else { 0.0 }
                    }
                    HeapSortState::ExtractMax => {
                        if self.data.len() == 0 {
                            1.0
                        } else {
                            let extraction_progress = (self.data.len() - self.heap_size) as f32 / self.data.len() as f32;
                            let progress = 0.5 + extraction_progress * 0.5;
                            if progress.is_finite() { progress.min(1.0).max(0.0) } else { 0.5 }
                        }
                    }
                }
            },
        }
    }

    fn reset(&mut self, data: Vec<i32>) {
        self.data = data;
        self.heap_size = self.data.len();
        self.current_index = self.data.len() / 2; // Start from last non-leaf node
        self.state = HeapSortState::BuildHeap;
        self.comparisons = 0;
        self.moves = 0;
        self.complete = self.data.len() <= 1;
    }

    fn name(&self) -> &str {
        "Heap Sort"
    }

    fn get_array(&self) -> &[i32] {
        &self.data
    }

    fn get_memory_usage(&self) -> usize {
        // Report size of the data array in bytes
        self.data.len() * std::mem::size_of::<i32>()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
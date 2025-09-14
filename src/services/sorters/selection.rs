//! Selection Sort implementation

use crate::models::traits::{Sorter, StepResult, Telemetry, Markers};
use std::any::Any;

/// Selection Sort algorithm implementation
#[derive(Debug)]
pub struct SelectionSort {
    data: Vec<i32>,
    current_position: usize,
    search_index: usize,
    min_index: usize,
    comparisons: u64,
    moves: u64,
    complete: bool,
}

impl SelectionSort {
    /// Create a new SelectionSort instance
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            current_position: 0,
            search_index: 1,
            min_index: 0,
            comparisons: 0,
            moves: 0,
            complete: false,
        }
    }
}

impl Default for SelectionSort {
    fn default() -> Self {
        Self::new()
    }
}

impl Sorter for SelectionSort {
    fn step(&mut self, budget: usize) -> StepResult {
        if self.complete || self.data.len() <= 1 {
            return StepResult {
                comparisons_used: 0,
                moves_made: 0,
                continued: false,
            };
        }

        let mut comparisons_used = 0;
        let mut moves_made = 0;

        while comparisons_used < budget && !self.complete {
            if self.current_position >= self.data.len() - 1 {
                self.complete = true;
                break;
            }

            if self.search_index < self.data.len() {
                // Compare current element with minimum found so far
                comparisons_used += 1;
                self.comparisons += 1;

                if self.data[self.search_index] < self.data[self.min_index] {
                    self.min_index = self.search_index;
                }

                self.search_index += 1;
            } else {
                // Found minimum for this pass, swap if necessary
                if self.min_index != self.current_position {
                    self.data.swap(self.current_position, self.min_index);
                    moves_made += 1;
                    self.moves += 1;
                }

                // Move to next position
                self.current_position += 1;
                if self.current_position < self.data.len() {
                    self.search_index = self.current_position + 1;
                    self.min_index = self.current_position;
                }
            }
        }

        StepResult {
            comparisons_used,
            moves_made,
            continued: !self.complete,
        }
    }

    fn is_complete(&self) -> bool {
        self.complete
    }

    fn get_telemetry(&self) -> Telemetry {
        let mut markers = Markers::default();
        
        if !self.complete {
            if self.search_index < self.data.len() {
                markers.cursors = vec![self.current_position, self.search_index, self.min_index];
            } else if self.current_position < self.data.len() {
                markers.cursors = vec![self.current_position, self.min_index];
            }
        }

        // Remove duplicates and sort
        markers.cursors.sort_unstable();
        markers.cursors.dedup();

        Telemetry {
            total_comparisons: self.comparisons,
            total_moves: self.moves,
            memory_current: self.get_memory_usage(),
            memory_peak: self.get_memory_usage(),
            highlights: markers.cursors.clone(),
            markers,
            status_text: if self.complete {
                "Completed".to_string()
            } else if self.search_index < self.data.len() {
                format!("Position {}, searching at {}, min at {}", 
                       self.current_position, self.search_index, self.min_index)
            } else {
                format!("Position {}, found min at {}", self.current_position, self.min_index)
            },
            progress_hint: if self.data.len() <= 1 {
                1.0
            } else {
                let progress = (self.current_position as f32) / ((self.data.len() - 1) as f32);
                progress.min(1.0).max(0.0) // Clamp to [0.0, 1.0]
            },
        }
    }

    fn reset(&mut self, data: Vec<i32>) {
        self.data = data;
        self.current_position = 0;
        self.search_index = 1;
        self.min_index = 0;
        self.comparisons = 0;
        self.moves = 0;
        self.complete = self.data.len() <= 1;
    }

    fn name(&self) -> &str {
        "Selection Sort"
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
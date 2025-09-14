//! Insertion Sort implementation

use crate::models::traits::{Sorter, StepResult, Telemetry, Markers};
use std::any::Any;

/// Insertion Sort algorithm implementation
#[derive(Debug)]
pub struct InsertionSort {
    data: Vec<i32>,
    current_index: usize,
    insert_pos: usize,
    comparisons: u64,
    moves: u64,
    complete: bool,
    in_insertion: bool,
}

impl InsertionSort {
    /// Create a new InsertionSort instance
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            current_index: 1,
            insert_pos: 0,
            comparisons: 0,
            moves: 0,
            complete: false,
            in_insertion: false,
        }
    }
}

impl Default for InsertionSort {
    fn default() -> Self {
        Self::new()
    }
}

impl Sorter for InsertionSort {
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
            if !self.in_insertion {
                if self.current_index >= self.data.len() {
                    self.complete = true;
                    break;
                }
                
                self.insert_pos = self.current_index;
                self.in_insertion = true;
            }

            if self.in_insertion && self.insert_pos > 0 {
                // Compare with previous element
                comparisons_used += 1;
                self.comparisons += 1;

                if self.data[self.insert_pos] < self.data[self.insert_pos - 1] {
                    // Swap elements
                    self.data.swap(self.insert_pos, self.insert_pos - 1);
                    moves_made += 1;
                    self.moves += 1;
                    self.insert_pos -= 1;
                } else {
                    // Found correct position
                    self.in_insertion = false;
                    self.current_index += 1;
                }
            } else {
                // Reached beginning of array or not in insertion
                self.in_insertion = false;
                self.current_index += 1;
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
            if self.in_insertion {
                markers.cursors = vec![self.insert_pos];
                if self.insert_pos > 0 {
                    markers.cursors.push(self.insert_pos - 1);
                }
            } else if self.current_index < self.data.len() {
                markers.cursors = vec![self.current_index];
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
            } else if self.in_insertion {
                format!("Inserting element at index {}", self.insert_pos)
            } else {
                format!("Processing index {}", self.current_index)
            },
            progress_hint: if self.data.len() <= 1 {
                1.0
            } else {
                let progress = (self.current_index as f32) / (self.data.len() as f32);
                progress.min(1.0).max(0.0) // Clamp to [0.0, 1.0]
            },
        }
    }

    fn reset(&mut self, data: Vec<i32>) {
        self.data = data;
        self.current_index = 1;
        self.insert_pos = 0;
        self.comparisons = 0;
        self.moves = 0;
        self.complete = self.data.len() <= 1;
        self.in_insertion = false;
    }

    fn name(&self) -> &str {
        "Insertion Sort"
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
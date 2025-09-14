//! Bubble Sort implementation

use crate::models::traits::{Sorter, StepResult, Telemetry, Markers};
use std::any::Any;

/// Bubble Sort algorithm implementation
#[derive(Debug)]
pub struct BubbleSort {
    data: Vec<i32>,
    current_pass: usize,
    current_pos: usize,
    comparisons: u64,
    moves: u64,
    complete: bool,
}

impl BubbleSort {
    /// Create a new BubbleSort instance
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            current_pass: 0,
            current_pos: 0,
            comparisons: 0,
            moves: 0,
            complete: false,
        }
    }
}

impl Default for BubbleSort {
    fn default() -> Self {
        Self::new()
    }
}

impl Sorter for BubbleSort {
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
        let n = self.data.len();

        while comparisons_used < budget {
            if self.current_pos < n - 1 - self.current_pass {
                // Compare adjacent elements
                comparisons_used += 1;
                self.comparisons += 1;

                if self.data[self.current_pos] > self.data[self.current_pos + 1] {
                    // Swap elements
                    self.data.swap(self.current_pos, self.current_pos + 1);
                    moves_made += 1;
                    self.moves += 1;
                }

                self.current_pos += 1;
            } else {
                // End of current pass
                self.current_pass += 1;
                self.current_pos = 0;

                if self.current_pass >= n - 1 {
                    self.complete = true;
                    break;
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
        
        if !self.complete && self.current_pos < self.data.len() {
            markers.cursors = vec![self.current_pos];
            if self.current_pos + 1 < self.data.len() {
                markers.cursors.push(self.current_pos + 1);
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
                format!("Pass {}, Position {}", self.current_pass + 1, self.current_pos)
            },
            progress_hint: if self.data.len() <= 1 {
                1.0
            } else {
                let progress = (self.current_pass as f32) / (self.data.len() - 1) as f32;
                progress.min(1.0).max(0.0) // Clamp to [0.0, 1.0]
            },
        }
    }

    fn reset(&mut self, data: Vec<i32>) {
        self.data = data;
        self.current_pass = 0;
        self.current_pos = 0;
        self.comparisons = 0;
        self.moves = 0;
        self.complete = self.data.len() <= 1;
    }

    fn name(&self) -> &str {
        "Bubble Sort"
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
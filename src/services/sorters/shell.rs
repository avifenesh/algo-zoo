//! Shell Sort implementation

use crate::models::traits::{Sorter, StepResult, Telemetry, Markers};
use std::any::Any;

/// Shell Sort algorithm implementation
#[derive(Debug)]
pub struct ShellSort {
    data: Vec<i32>,
    gap: usize,
    current_pos: usize,
    insertion_pos: usize,
    comparisons: u64,
    moves: u64,
    complete: bool,
    in_insertion: bool,
}

impl ShellSort {
    /// Create a new ShellSort instance
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            gap: 0,
            current_pos: 0,
            insertion_pos: 0,
            comparisons: 0,
            moves: 0,
            complete: false,
            in_insertion: false,
        }
    }

    /// Generate gap sequence using Knuth's sequence: 1, 4, 13, 40, 121, ...
    fn generate_initial_gap(n: usize) -> usize {
        let mut gap = 1;
        while gap < n / 3 {
            gap = gap * 3 + 1;
        }
        gap
    }

    fn next_gap(current_gap: usize) -> usize {
        (current_gap - 1) / 3
    }
}

impl Default for ShellSort {
    fn default() -> Self {
        Self::new()
    }
}

impl Sorter for ShellSort {
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

        while remaining_budget > 0 && !self.complete {
            if self.gap == 0 {
                self.complete = true;
                break;
            }

            if !self.in_insertion {
                // Start new element insertion
                if self.current_pos >= self.data.len() {
                    // Move to next gap
                    self.gap = Self::next_gap(self.gap);
                    self.current_pos = self.gap;
                    continue;
                }

                self.insertion_pos = self.current_pos;
                self.in_insertion = true;
            }

            if self.in_insertion {
                if self.insertion_pos >= self.gap {
                    // Compare with element at gap distance
                    remaining_budget -= 1;
                    self.comparisons += 1;

                    if self.data[self.insertion_pos] < self.data[self.insertion_pos - self.gap] {
                        // Swap elements
                        self.data.swap(self.insertion_pos, self.insertion_pos - self.gap);
                        self.moves += 1;
                        self.insertion_pos -= self.gap;
                    } else {
                        // Found correct position
                        self.in_insertion = false;
                        self.current_pos += 1;
                    }
                } else {
                    // Reached the beginning for this gap
                    self.in_insertion = false;
                    self.current_pos += 1;
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
        
        if !self.complete {
            markers.gap = Some(self.gap);
            
            if self.in_insertion {
                markers.cursors.push(self.insertion_pos);
                if self.insertion_pos >= self.gap {
                    markers.cursors.push(self.insertion_pos - self.gap);
                }
            } else if self.current_pos < self.data.len() {
                markers.cursors.push(self.current_pos);
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
                format!("Gap {}, inserting at position {}", self.gap, self.insertion_pos)
            } else {
                format!("Gap {}, processing position {}", self.gap, self.current_pos)
            },
            progress_hint: if self.data.len() <= 1 {
                1.0
            } else {
                // Estimate progress based on gap reduction
                let initial_gap = Self::generate_initial_gap(self.data.len()) as f32;
                let current_gap = self.gap as f32;
                if initial_gap == 0.0 {
                    1.0
                } else {
                    let progress = 1.0 - (current_gap / initial_gap);
                    if progress.is_finite() { progress.min(1.0).max(0.0) } else { 0.0 }
                }
            },
        }
    }

    fn reset(&mut self, data: Vec<i32>) {
        self.data = data;
        self.gap = Self::generate_initial_gap(self.data.len());
        self.current_pos = self.gap;
        self.insertion_pos = 0;
        self.comparisons = 0;
        self.moves = 0;
        self.complete = self.data.len() <= 1;
        self.in_insertion = false;
    }

    fn name(&self) -> &str {
        "Shell Sort"
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
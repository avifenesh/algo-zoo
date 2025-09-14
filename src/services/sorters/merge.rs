//! Merge Sort implementation

use crate::models::traits::{Sorter, StepResult, Telemetry, Markers};
use std::any::Any;

#[derive(Debug, Clone)]
struct MergeFrame {
    left: usize,
    mid: usize,
    right: usize,
    state: MergeState,
    temp_left_idx: usize,
    temp_right_idx: usize,
    output_idx: usize,
}

#[derive(Debug, Clone, PartialEq)]
enum MergeState {
    Split,
    Merge,
}

/// Merge Sort algorithm implementation
#[derive(Debug)]
pub struct MergeSort {
    data: Vec<i32>,
    temp_buffer: Vec<i32>,
    stack: Vec<MergeFrame>,
    comparisons: u64,
    moves: u64,
    complete: bool,
    memory_usage: usize,
}

impl MergeSort {
    /// Create a new MergeSort instance
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            temp_buffer: Vec::new(),
            stack: Vec::new(),
            comparisons: 0,
            moves: 0,
            complete: false,
            memory_usage: 0,
        }
    }

    fn merge(&mut self, frame: &mut MergeFrame, budget: &mut usize) -> bool {
        let mid = frame.mid;
        let right = frame.right;

        // Copy data to temp buffer if not already done
        if frame.temp_left_idx == frame.left {
            for i in frame.left..=right {
                self.temp_buffer[i] = self.data[i];
            }
        }

        while *budget > 0 && frame.output_idx <= right {
            if frame.temp_left_idx > mid {
                // Left half exhausted, copy from right
                self.data[frame.output_idx] = self.temp_buffer[frame.temp_right_idx];
                self.moves += 1;
                frame.temp_right_idx += 1;
            } else if frame.temp_right_idx > right {
                // Right half exhausted, copy from left
                self.data[frame.output_idx] = self.temp_buffer[frame.temp_left_idx];
                self.moves += 1;
                frame.temp_left_idx += 1;
            } else {
                // Compare and merge
                *budget -= 1;
                self.comparisons += 1;

                if self.temp_buffer[frame.temp_left_idx] <= self.temp_buffer[frame.temp_right_idx] {
                    self.data[frame.output_idx] = self.temp_buffer[frame.temp_left_idx];
                    frame.temp_left_idx += 1;
                } else {
                    self.data[frame.output_idx] = self.temp_buffer[frame.temp_right_idx];
                    frame.temp_right_idx += 1;
                }
                self.moves += 1;
            }

            frame.output_idx += 1;
        }

        frame.output_idx > right // Return true if merge is complete
    }
}

impl Default for MergeSort {
    fn default() -> Self {
        Self::new()
    }
}

impl Sorter for MergeSort {
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

        while remaining_budget > 0 && !self.stack.is_empty() {
            let mut frame = self.stack.pop().unwrap();

            match frame.state {
                MergeState::Split => {
                    if frame.left < frame.right {
                        let mid = (frame.left + frame.right) / 2;
                        
                        // Push merge operation for later
                        self.stack.push(MergeFrame {
                            left: frame.left,
                            mid,
                            right: frame.right,
                            state: MergeState::Merge,
                            temp_left_idx: frame.left,
                            temp_right_idx: mid + 1,
                            output_idx: frame.left,
                        });

                        // Push right half
                        self.stack.push(MergeFrame {
                            left: mid + 1,
                            mid: mid + 1,
                            right: frame.right,
                            state: MergeState::Split,
                            temp_left_idx: 0,
                            temp_right_idx: 0,
                            output_idx: 0,
                        });

                        // Push left half
                        self.stack.push(MergeFrame {
                            left: frame.left,
                            mid: frame.left,
                            right: mid,
                            state: MergeState::Split,
                            temp_left_idx: 0,
                            temp_right_idx: 0,
                            output_idx: 0,
                        });
                    }
                }
                MergeState::Merge => {
                    let finished = self.merge(&mut frame, &mut remaining_budget);
                    
                    if !finished {
                        // Merge not finished, push back to stack
                        self.stack.push(frame);
                        break;
                    }
                }
            }
        }

        if self.stack.is_empty() {
            self.complete = true;
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
        
        if let Some(frame) = self.stack.last()
            && frame.state == MergeState::Merge {
                markers.merge_runs.push((frame.left, frame.right));
                
                // Show current merge positions
                if frame.temp_left_idx <= frame.mid {
                    markers.cursors.push(frame.temp_left_idx);
                }
                if frame.temp_right_idx <= frame.right {
                    markers.cursors.push(frame.temp_right_idx);
                }
                if frame.output_idx <= frame.right {
                    markers.cursors.push(frame.output_idx);
                }
            }

        Telemetry {
            total_comparisons: self.comparisons,
            total_moves: self.moves,
            memory_current: self.memory_usage,
            memory_peak: self.memory_usage, // Simplified for stub
            highlights: markers.cursors.clone(),
            markers,
            status_text: if self.complete {
                "Completed".to_string()
            } else if let Some(frame) = self.stack.last() {
                match frame.state {
                    MergeState::Split => format!("Splitting range [{}, {}]", frame.left, frame.right),
                    MergeState::Merge => format!("Merging range [{}, {}]", frame.left, frame.right),
                }
            } else {
                "Processing".to_string()
            },
            progress_hint: if self.data.len() <= 1 {
                1.0
            } else {
                let progress = 1.0 - (self.stack.len() as f32 / (self.data.len() as f32).log2().max(1.0));
                if progress.is_finite() { progress.min(1.0).max(0.0) } else { 0.0 }
            },
        }
    }

    fn reset(&mut self, data: Vec<i32>) {
        self.data = data;
        self.temp_buffer = vec![0; self.data.len()];
        self.stack.clear();
        self.comparisons = 0;
        self.moves = 0;
        self.complete = self.data.len() <= 1;
        
        // Calculate memory usage: temp buffer + stack
        self.memory_usage = self.temp_buffer.len() * std::mem::size_of::<i32>();

        if !self.complete {
            self.stack.push(MergeFrame {
                left: 0,
                mid: 0,
                right: self.data.len() - 1,
                state: MergeState::Split,
                temp_left_idx: 0,
                temp_right_idx: 0,
                output_idx: 0,
            });
            
            self.memory_usage += self.stack.capacity() * std::mem::size_of::<MergeFrame>();
        }
    }

    fn name(&self) -> &str {
        "Merge Sort"
    }

    fn get_array(&self) -> &[i32] {
        &self.data
    }

    fn get_memory_usage(&self) -> usize {
        // Data array + temp buffer + stack
        self.data.len() * std::mem::size_of::<i32>() + self.memory_usage
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
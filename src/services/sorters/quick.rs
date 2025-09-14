//! Quick Sort implementation

use crate::models::traits::{Sorter, StepResult, Telemetry, Markers};
use std::any::Any;

/// Stack frame for Quick Sort recursion simulation
#[derive(Debug, Clone)]
struct StackFrame {
    low: usize,
    high: usize,
}

/// State for incremental partitioning
#[derive(Debug, Clone)]
enum PartitionState {
    /// No partitioning in progress
    NotStarted,
    /// Partitioning in progress with current indices and pivot value
    InProgress {
        current_j: usize,
        current_i: usize,
        pivot: i32,
        low: usize,
        high: usize,
    },
    /// Partitioning completed
    Complete,
}

/// Quick Sort algorithm implementation
#[derive(Debug)]
pub struct QuickSort {
    data: Vec<i32>,
    stack: Vec<StackFrame>,
    comparisons: u64,
    moves: u64,
    complete: bool,
    current_pivot: Option<usize>,
    memory_usage: usize,
    partition_state: PartitionState,
    max_progress_seen: f32,
}

impl QuickSort {
    /// Create a new QuickSort instance
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            stack: Vec::new(),
            comparisons: 0,
            moves: 0,
            complete: false,
            current_pivot: None,
            memory_usage: 0,
            partition_state: PartitionState::NotStarted,
            max_progress_seen: 0.0,
        }
    }

    /// Start a new partition operation
    fn start_partition(&mut self, low: usize, high: usize) {
        let pivot = self.data[high];
        self.partition_state = PartitionState::InProgress {
            current_j: low,
            current_i: low,
            pivot,
            low,
            high,
        };
        self.current_pivot = Some(high);
    }

    /// Continue or complete partition operation with given budget
    /// Returns Some(pivot_index) if partition completes, None if more work needed
    fn continue_partition(&mut self, budget: &mut usize) -> Option<usize> {
        if *budget == 0 {
            return None;
        }

        match &self.partition_state.clone() {
            PartitionState::InProgress { current_j, current_i, pivot, low, high } => {
                let mut j = *current_j;
                let mut i = *current_i;
                let pivot_val = *pivot;
                let low_bound = *low;
                let high_bound = *high;

                // Continue partitioning from where we left off
                while j < high_bound && *budget > 0 {
                    *budget -= 1;
                    self.comparisons += 1;

                    if self.data[j] <= pivot_val {
                        if i != j {
                            self.data.swap(i, j);
                            self.moves += 1;
                        }
                        i += 1;
                    }
                    j += 1;
                }

                if j >= high_bound {
                    // Partitioning complete - place pivot in final position
                    if i != high_bound {
                        self.data.swap(i, high_bound);
                        self.moves += 1;
                    }
                    self.partition_state = PartitionState::Complete;
                    self.current_pivot = None;
                    Some(i)
                } else {
                    // More work needed - update state
                    self.partition_state = PartitionState::InProgress {
                        current_j: j,
                        current_i: i,
                        pivot: pivot_val,
                        low: low_bound,
                        high: high_bound,
                    };
                    None
                }
            }
            _ => None, // Should not happen when called appropriately
        }
    }

    /// Get partition progress (0.0 to 1.0)
    #[allow(dead_code)]
    fn get_partition_progress(&self) -> f32 {
        match &self.partition_state {
            PartitionState::NotStarted => 0.0,
            PartitionState::Complete => 1.0,
            PartitionState::InProgress { current_j, low, high, .. } => {
                if *high <= *low {
                    1.0
                } else {
                    (*current_j - *low) as f32 / (*high - *low) as f32
                }
            }
        }
    }
}

impl Default for QuickSort {
    fn default() -> Self {
        Self::new()
    }
}

impl Sorter for QuickSort {
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

        // Handle ongoing partition if in progress
        if matches!(self.partition_state, PartitionState::InProgress { .. }) {
            if let Some(pivot_pos) = self.continue_partition(&mut remaining_budget) {
                // Partition completed - add sub-problems to stack
                if let Some(frame) = self.stack.last() {
                    let current_frame = frame.clone();
                    self.stack.pop(); // Remove current frame
                    
                    // Add right subarray if it has more than one element
                    if pivot_pos + 1 < current_frame.high {
                        self.stack.push(StackFrame {
                            low: pivot_pos + 1,
                            high: current_frame.high,
                        });
                    }
                    
                    // Add left subarray if it has more than one element
                    if current_frame.low < pivot_pos {
                        self.stack.push(StackFrame {
                            low: current_frame.low,
                            high: pivot_pos - 1,
                        });
                    }
                }
                self.partition_state = PartitionState::NotStarted;
            }
            // If partition didn't complete, we'll continue it in the next step
        }

        // Start new partitions if budget allows and no partition is in progress
        while remaining_budget > 0 && !self.stack.is_empty() && 
              matches!(self.partition_state, PartitionState::NotStarted) {
            
            let frame = self.stack.last().unwrap().clone();
            
            if frame.low >= frame.high {
                self.stack.pop(); // Remove trivial frame
                continue;
            }

            // Start new partition
            self.start_partition(frame.low, frame.high);
            
            // Try to make progress on the new partition
            if let Some(pivot_pos) = self.continue_partition(&mut remaining_budget) {
                // Partition completed immediately
                self.stack.pop(); // Remove current frame
                
                // Add right subarray if it has more than one element
                if pivot_pos + 1 < frame.high {
                    self.stack.push(StackFrame {
                        low: pivot_pos + 1,
                        high: frame.high,
                    });
                }
                
                // Add left subarray if it has more than one element
                if frame.low < pivot_pos {
                    self.stack.push(StackFrame {
                        low: frame.low,
                        high: pivot_pos - 1,
                    });
                }
                
                self.partition_state = PartitionState::NotStarted;
            } else {
                // Partition started but not completed - will continue next step
                break;
            }
        }

        // Check completion
        if self.stack.is_empty() && matches!(self.partition_state, PartitionState::NotStarted) {
            self.complete = true;
            self.current_pivot = None;
        }

        self.memory_usage = self.stack.len() * std::mem::size_of::<StackFrame>();

        // Update progress tracking for monotonicity
        self.update_progress();

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
        
        if let Some(pivot) = self.current_pivot {
            markers.pivot = Some(pivot);
        }

        // Add cursors for current partition state
        match &self.partition_state {
            PartitionState::InProgress { current_j, current_i, low, high, .. } => {
                markers.cursors = vec![*current_i, *current_j, *low, *high];
            }
            _ => {
                if let Some(frame) = self.stack.last() {
                    markers.cursors = vec![frame.low, frame.high];
                }
            }
        }

        let progress_hint = self.calculate_progress();
        
        let status_text = if self.complete {
            "Completed".to_string()
        } else {
            match &self.partition_state {
                PartitionState::InProgress { current_j, low, high, .. } => {
                    format!("Partitioning range [{}, {}] - progress: {}/{}", 
                           low, high, current_j - low, high - low)
                }
                _ => {
                    if let Some(frame) = self.stack.last() {
                        format!("Partitioning range [{}, {}]", frame.low, frame.high)
                    } else {
                        "Processing".to_string()
                    }
                }
            }
        };

        Telemetry {
            total_comparisons: self.comparisons,
            total_moves: self.moves,
            memory_current: self.get_memory_usage(),
            memory_peak: self.get_memory_usage(),
            highlights: markers.cursors.clone(),
            markers,
            status_text,
            progress_hint,
        }
    }

    fn reset(&mut self, data: Vec<i32>) {
        self.data = data;
        self.stack.clear();
        self.comparisons = 0;
        self.moves = 0;
        self.complete = self.data.len() <= 1;
        self.current_pivot = None;
        self.memory_usage = 0;
        self.partition_state = PartitionState::NotStarted;
        self.max_progress_seen = 0.0;

        if !self.complete {
            self.stack.push(StackFrame {
                low: 0,
                high: self.data.len() - 1,
            });
            self.memory_usage = std::mem::size_of::<StackFrame>();
        }
    }

    fn name(&self) -> &str {
        "Quick Sort"
    }

    fn get_array(&self) -> &[i32] {
        &self.data
    }

    fn get_memory_usage(&self) -> usize {
        // Data array + stack memory
        self.data.len() * std::mem::size_of::<i32>() + 
        self.stack.len() * std::mem::size_of::<StackFrame>()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl QuickSort {
    /// Calculate overall progress of the sorting algorithm (monotonic)
    fn calculate_progress(&self) -> f32 {
        if self.data.len() <= 1 {
            return 1.0;
        }

        if self.complete {
            return 1.0;
        }

        // Base progress on comparisons made vs expected total comparisons
        let n = self.data.len() as f32;
        let expected_comparisons = n * n.log2(); // O(n log n) average case
        let base_progress = (self.comparisons as f32 / expected_comparisons).min(0.95);

        // Add fine-grained progress from current partition
        let partition_progress = match &self.partition_state {
            PartitionState::InProgress { current_j, low, high, .. } => {
                if *high > *low {
                    let current_partition_size = high - low;
                    let partition_weight = current_partition_size as f32 / n;
                    let local_progress = (*current_j - *low) as f32 / (*high - *low) as f32;
                    partition_weight * local_progress * 0.05 // Small contribution for smoothness
                } else {
                    0.0
                }
            }
            _ => 0.0,
        };

        let current_progress = (base_progress + partition_progress).min(1.0).max(0.0);
        
        // Ensure monotonicity: never decrease progress
        self.max_progress_seen.max(current_progress)
    }

    /// Update progress tracking after a step
    fn update_progress(&mut self) {
        let current_progress = self.calculate_progress();
        self.max_progress_seen = self.max_progress_seen.max(current_progress);
    }
}
//! Visual markers for algorithm visualization
//! 
//! Re-exports the Markers struct from traits.rs for convenience
//! and provides additional utilities for marker management.

pub use crate::models::traits::Markers;

/// Helper functions for working with visual markers
impl Markers {
    /// Create a new empty markers instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Clear all markers
    pub fn clear(&mut self) {
        self.pivot = None;
        self.heap_boundary = None;
        self.merge_runs.clear();
        self.cursors.clear();
        self.gap = None;
    }

    /// Set pivot marker for Quick Sort
    pub fn set_pivot(&mut self, index: usize) {
        self.pivot = Some(index);
    }

    /// Clear pivot marker
    pub fn clear_pivot(&mut self) {
        self.pivot = None;
    }

    /// Set heap boundary for Heap Sort
    pub fn set_heap_boundary(&mut self, boundary: usize) {
        self.heap_boundary = Some(boundary);
    }

    /// Clear heap boundary
    pub fn clear_heap_boundary(&mut self) {
        self.heap_boundary = None;
    }

    /// Add a merge run for Merge Sort
    pub fn add_merge_run(&mut self, start: usize, end: usize) {
        self.merge_runs.push((start, end));
    }

    /// Clear all merge runs
    pub fn clear_merge_runs(&mut self) {
        self.merge_runs.clear();
    }

    /// Set cursor positions
    pub fn set_cursors(&mut self, positions: Vec<usize>) {
        self.cursors = positions;
    }

    /// Add a cursor position
    pub fn add_cursor(&mut self, position: usize) {
        self.cursors.push(position);
    }

    /// Clear all cursors
    pub fn clear_cursors(&mut self) {
        self.cursors.clear();
    }

    /// Set gap size for Shell Sort
    pub fn set_gap(&mut self, gap: usize) {
        self.gap = Some(gap);
    }

    /// Clear gap marker
    pub fn clear_gap(&mut self) {
        self.gap = None;
    }

    /// Check if any markers are active
    pub fn has_active_markers(&self) -> bool {
        self.pivot.is_some() 
            || self.heap_boundary.is_some() 
            || !self.merge_runs.is_empty() 
            || !self.cursors.is_empty() 
            || self.gap.is_some()
    }

    /// Get all active marker positions as a flat vector
    pub fn get_all_positions(&self) -> Vec<usize> {
        let mut positions = Vec::new();
        
        if let Some(pivot) = self.pivot {
            positions.push(pivot);
        }
        
        if let Some(boundary) = self.heap_boundary {
            positions.push(boundary);
        }
        
        positions.extend(self.cursors.iter().copied());
        
        for (start, end) in &self.merge_runs {
            positions.push(*start);
            positions.push(*end);
        }
        
        positions.sort_unstable();
        positions.dedup();
        positions
    }
}
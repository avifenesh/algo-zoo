//! Memory tracking implementation for sorting algorithms

use crate::models::traits::MemoryTracker;

/// Standard implementation of memory tracking
#[derive(Debug, Default)]
pub struct StandardMemoryTracker {
    current: usize,
    peak: usize,
}

impl StandardMemoryTracker {
    /// Create a new memory tracker
    pub fn new() -> Self {
        Self::default()
    }
}

impl MemoryTracker for StandardMemoryTracker {
    fn alloc(&mut self, bytes: usize) {
        // Use saturating arithmetic to prevent overflow
        self.current = self.current.saturating_add(bytes);
        self.peak = self.peak.max(self.current);
    }

    fn free(&mut self, bytes: usize) {
        self.current = self.current.saturating_sub(bytes);
    }

    fn current(&self) -> usize {
        self.current
    }

    fn peak(&self) -> usize {
        self.peak
    }

    fn reset(&mut self) {
        self.current = 0;
        self.peak = 0;
    }
}

/// Memory tracker that logs all operations for debugging
#[derive(Debug, Default)]
pub struct VerboseMemoryTracker {
    current: usize,
    peak: usize,
    operations: Vec<MemoryOperation>,
}

#[derive(Debug, Clone)]
pub struct MemoryOperation {
    pub op_type: MemoryOpType,
    pub bytes: usize,
    pub current_after: usize,
    pub peak_after: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MemoryOpType {
    Alloc,
    Free,
    Reset,
}

impl VerboseMemoryTracker {
    /// Create a new verbose memory tracker
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the operation log
    pub fn get_operations(&self) -> &[MemoryOperation] {
        &self.operations
    }

    /// Clear the operation log
    pub fn clear_log(&mut self) {
        self.operations.clear();
    }

    fn log_operation(&mut self, op_type: MemoryOpType, bytes: usize) {
        self.operations.push(MemoryOperation {
            op_type,
            bytes,
            current_after: self.current,
            peak_after: self.peak,
        });
    }
}

impl MemoryTracker for VerboseMemoryTracker {
    fn alloc(&mut self, bytes: usize) {
        self.current = self.current.saturating_add(bytes);
        self.peak = self.peak.max(self.current);
        self.log_operation(MemoryOpType::Alloc, bytes);
    }

    fn free(&mut self, bytes: usize) {
        self.current = self.current.saturating_sub(bytes);
        self.log_operation(MemoryOpType::Free, bytes);
    }

    fn current(&self) -> usize {
        self.current
    }

    fn peak(&self) -> usize {
        self.peak
    }

    fn reset(&mut self) {
        self.current = 0;
        self.peak = 0;
        self.log_operation(MemoryOpType::Reset, 0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_memory_tracker() {
        let mut tracker = StandardMemoryTracker::new();
        
        assert_eq!(tracker.current(), 0);
        assert_eq!(tracker.peak(), 0);
        
        tracker.alloc(100);
        assert_eq!(tracker.current(), 100);
        assert_eq!(tracker.peak(), 100);
        
        tracker.free(50);
        assert_eq!(tracker.current(), 50);
        assert_eq!(tracker.peak(), 100);
        
        tracker.reset();
        assert_eq!(tracker.current(), 0);
        assert_eq!(tracker.peak(), 0);
    }

    #[test]
    fn test_verbose_memory_tracker() {
        let mut tracker = VerboseMemoryTracker::new();
        
        tracker.alloc(100);
        tracker.free(50);
        tracker.reset();
        
        let ops = tracker.get_operations();
        assert_eq!(ops.len(), 3);
        
        assert_eq!(ops[0].op_type, MemoryOpType::Alloc);
        assert_eq!(ops[1].op_type, MemoryOpType::Free);
        assert_eq!(ops[2].op_type, MemoryOpType::Reset);
    }
}
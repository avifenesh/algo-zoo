//! Contract tests for the MemoryTracker trait
//! These tests verify that memory tracking works correctly

use sorting_race::models::traits::MemoryTracker;
use sorting_race::services::memory::StandardMemoryTracker;

#[derive(Debug, Default)]
struct TestMemoryTracker {
    current: usize,
    peak: usize,
}

impl MemoryTracker for TestMemoryTracker {
    fn alloc(&mut self, bytes: usize) {
        self.current += bytes;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn verify_memory_tracker<T: MemoryTracker>(mut tracker: T) {
        // Initial state
        assert_eq!(tracker.current(), 0);
        assert_eq!(tracker.peak(), 0);

        // Allocate memory
        tracker.alloc(100);
        assert_eq!(tracker.current(), 100);
        assert_eq!(tracker.peak(), 100);

        // Allocate more
        tracker.alloc(50);
        assert_eq!(tracker.current(), 150);
        assert_eq!(tracker.peak(), 150);

        // Free some memory
        tracker.free(75);
        assert_eq!(tracker.current(), 75);
        assert_eq!(tracker.peak(), 150); // Peak unchanged

        // Allocate again (but not exceeding peak)
        tracker.alloc(50);
        assert_eq!(tracker.current(), 125);
        assert_eq!(tracker.peak(), 150);

        // Allocate beyond previous peak
        tracker.alloc(100);
        assert_eq!(tracker.current(), 225);
        assert_eq!(tracker.peak(), 225);

        // Free all
        tracker.free(225);
        assert_eq!(tracker.current(), 0);
        assert_eq!(tracker.peak(), 225);

        // Reset
        tracker.reset();
        assert_eq!(tracker.current(), 0);
        assert_eq!(tracker.peak(), 0);
    }

    #[test]
    fn test_test_memory_tracker() {
        let tracker = TestMemoryTracker::default();
        verify_memory_tracker(tracker);
    }

    #[test]
    fn test_standard_memory_tracker() {
        let tracker = StandardMemoryTracker::new();
        verify_memory_tracker(tracker);
    }

    #[test]
    fn test_memory_tracker_overflow_protection() {
        let mut tracker = TestMemoryTracker::default();
        
        // Allocate large amount
        tracker.alloc(usize::MAX / 2);
        assert_eq!(tracker.current(), usize::MAX / 2);
        
        // Try to allocate more (should not overflow)
        tracker.alloc(usize::MAX / 2 + 100);
        assert!(tracker.current() > 0); // Should handle without panic
        
        // Free more than allocated (should saturate at 0)
        tracker.free(usize::MAX);
        assert_eq!(tracker.current(), 0);
    }

    #[test]
    fn test_memory_tracker_sequence() {
        let mut tracker = StandardMemoryTracker::new();
        
        // Simulate Quick Sort stack allocation
        let stack_frames = vec![16, 16, 16, 16]; // 4 frames
        for size in &stack_frames {
            tracker.alloc(*size);
        }
        assert_eq!(tracker.current(), 64);
        assert_eq!(tracker.peak(), 64);
        
        // Pop stack frames
        for size in stack_frames.iter().rev() {
            tracker.free(*size);
        }
        assert_eq!(tracker.current(), 0);
        assert_eq!(tracker.peak(), 64);
    }

    #[test]
    fn test_memory_tracker_merge_buffer() {
        let mut tracker = StandardMemoryTracker::new();
        
        // Simulate Merge Sort buffer allocation
        let buffer_size = 1000;
        tracker.alloc(buffer_size);
        assert_eq!(tracker.current(), buffer_size);
        
        // Reuse buffer (no additional allocation)
        assert_eq!(tracker.peak(), buffer_size);
        
        // Free buffer at end
        tracker.free(buffer_size);
        assert_eq!(tracker.current(), 0);
        assert_eq!(tracker.peak(), buffer_size);
    }

    #[test]
    fn test_memory_tracker_multiple_algorithms() {
        // Simulate tracking memory for multiple algorithms
        let mut trackers: Vec<Box<dyn MemoryTracker>> = vec![
            Box::new(TestMemoryTracker::default()),      // Bubble
            Box::new(StandardMemoryTracker::new()),       // Quick
            Box::new(StandardMemoryTracker::new()),       // Merge
        ];
        
        // Bubble: O(1) memory
        trackers[0].alloc(8); // Just indices
        
        // Quick: O(log n) stack
        trackers[1].alloc(64); // Stack frames
        
        // Merge: O(n) buffer
        trackers[2].alloc(400); // Temp buffer
        
        // Verify each tracker independently
        assert_eq!(trackers[0].current(), 8);
        assert_eq!(trackers[1].current(), 64);
        assert_eq!(trackers[2].current(), 400);
        
        // Calculate total memory usage
        let total_current: usize = trackers.iter().map(|t| t.current()).sum();
        let total_peak: usize = trackers.iter().map(|t| t.peak()).sum();
        
        assert_eq!(total_current, 472);
        assert_eq!(total_peak, 472);
    }

    #[test]
    fn test_memory_tracker_reset_behavior() {
        let mut tracker = StandardMemoryTracker::new();
        
        // Build up some state
        tracker.alloc(100);
        tracker.alloc(200);
        tracker.free(150);
        
        assert_eq!(tracker.current(), 150);
        assert_eq!(tracker.peak(), 300);
        
        // Reset clears everything
        tracker.reset();
        assert_eq!(tracker.current(), 0);
        assert_eq!(tracker.peak(), 0);
        
        // Can use after reset
        tracker.alloc(50);
        assert_eq!(tracker.current(), 50);
        assert_eq!(tracker.peak(), 50);
    }
}
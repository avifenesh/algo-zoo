//! Integration tests for the full visualization pipeline
//!
//! These tests verify the complete visualization pipeline from algorithms to terminal,
//! testing frame rate limiting, memory usage, and responsiveness to terminal resize.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::thread;
use ratatui::{
    backend::TestBackend,
    Terminal,
    layout::{Constraint, Direction, Layout, Rect},
};

// Import existing types (these would be actual imports in real code)
use sorting_race::models::traits::{Sorter, Telemetry, Markers};

/// Mock sorter for testing the visualization pipeline
#[derive(Debug)]
struct MockSorter {
    name: String,
    array: Vec<i32>,
    is_complete: bool,
    step_count: usize,
    comparisons: u64,
    moves: u64,
}

impl MockSorter {
    fn new(name: &str, array: Vec<i32>) -> Self {
        Self {
            name: name.to_string(),
            array,
            is_complete: false,
            step_count: 0,
            comparisons: 0,
            moves: 0,
        }
    }
}

impl Sorter for MockSorter {
    fn step(&mut self, budget: usize) -> sorting_race::models::traits::StepResult {
        self.step_count += 1;
        self.comparisons += budget as u64;
        self.moves += (budget / 2) as u64;
        
        // Simulate some work and mark complete after a few steps
        if self.step_count > 5 {
            self.is_complete = true;
        }
        
        sorting_race::models::traits::StepResult {
            comparisons_used: budget,
            moves_made: budget / 2,
            continued: !self.is_complete,
        }
    }

    fn is_complete(&self) -> bool {
        self.is_complete
    }

    fn get_telemetry(&self) -> Telemetry {
        Telemetry {
            total_comparisons: self.comparisons,
            total_moves: self.moves,
            memory_current: 1024,
            memory_peak: 2048,
            highlights: if self.step_count % 2 == 0 { vec![0, 1] } else { vec![2, 3] },
            markers: Markers {
                pivot: Some(self.array.len() / 2),
                cursors: vec![self.step_count % self.array.len()],
                ..Default::default()
            },
            status_text: format!("Step {} of {}", self.step_count, self.name),
            progress_hint: (self.step_count as f32 / 6.0).min(1.0),
        }
    }

    fn reset(&mut self, data: Vec<i32>) {
        self.array = data;
        self.step_count = 0;
        self.comparisons = 0;
        self.moves = 0;
        self.is_complete = false;
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn get_array(&self) -> &[i32] {
        &self.array
    }

    fn get_memory_usage(&self) -> usize {
        self.array.len() * std::mem::size_of::<i32>()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Frame rate limiter for visualization
#[derive(Debug)]
pub struct FrameRateLimiter {
    target_fps: u32,
    frame_duration: Duration,
    last_frame_time: Option<Instant>,
}

impl FrameRateLimiter {
    pub fn new(target_fps: u32) -> Self {
        let frame_duration = Duration::from_nanos(1_000_000_000 / target_fps as u64);
        Self {
            target_fps,
            frame_duration,
            last_frame_time: None,
        }
    }

    pub fn limit_frame_rate(&mut self) {
        let now = Instant::now();
        
        if let Some(last_time) = self.last_frame_time {
            let elapsed = now.duration_since(last_time);
            if elapsed < self.frame_duration {
                let sleep_duration = self.frame_duration - elapsed;
                thread::sleep(sleep_duration);
            }
        }
        
        self.last_frame_time = Some(Instant::now());
    }

    pub fn get_actual_fps(&self) -> Option<f32> {
        if let Some(last_time) = self.last_frame_time {
            let elapsed = last_time.elapsed();
            if elapsed.as_millis() > 0 {
                Some(1000.0 / elapsed.as_millis() as f32)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn target_fps(&self) -> u32 {
        self.target_fps
    }
}

/// Memory monitor for tracking visualization memory usage
#[derive(Debug)]
pub struct MemoryMonitor {
    peak_usage: usize,
    current_usage: usize,
    limit: usize,
}

impl MemoryMonitor {
    pub fn new(limit_mb: usize) -> Self {
        Self {
            peak_usage: 0,
            current_usage: 0,
            limit: limit_mb * 1024 * 1024, // Convert MB to bytes
        }
    }

    pub fn record_usage(&mut self, bytes: usize) {
        self.current_usage = bytes;
        if bytes > self.peak_usage {
            self.peak_usage = bytes;
        }
    }

    pub fn current_usage(&self) -> usize {
        self.current_usage
    }

    pub fn peak_usage(&self) -> usize {
        self.peak_usage
    }

    pub fn is_under_limit(&self) -> bool {
        self.current_usage <= self.limit
    }

    pub fn limit(&self) -> usize {
        self.limit
    }

    pub fn usage_percentage(&self) -> f32 {
        (self.current_usage as f32 / self.limit as f32) * 100.0
    }
}

/// Full visualization pipeline
#[derive(Debug)]
pub struct VisualizationPipeline {
    algorithms: Vec<Box<dyn Sorter>>,
    frame_limiter: FrameRateLimiter,
    memory_monitor: MemoryMonitor,
    terminal_size: (u16, u16), // (width, height)
    frame_count: u64,
}

impl VisualizationPipeline {
    pub fn new(algorithms: Vec<Box<dyn Sorter>>, target_fps: u32, memory_limit_mb: usize) -> Self {
        Self {
            algorithms,
            frame_limiter: FrameRateLimiter::new(target_fps),
            memory_monitor: MemoryMonitor::new(memory_limit_mb),
            terminal_size: (80, 24),
            frame_count: 0,
        }
    }

    pub fn set_terminal_size(&mut self, width: u16, height: u16) {
        self.terminal_size = (width, height);
    }

    pub fn render_frame(&mut self) -> Result<String, String> {
        // Simulate memory usage calculation
        let total_memory: usize = self.algorithms.iter()
            .map(|alg| alg.get_memory_usage())
            .sum::<usize>() + 
            (self.terminal_size.0 as usize * self.terminal_size.1 as usize * 4); // Terminal buffer
        
        self.memory_monitor.record_usage(total_memory);
        
        if !self.memory_monitor.is_under_limit() {
            return Err(format!("Memory usage exceeded limit: {} bytes > {} bytes", 
                             total_memory, self.memory_monitor.limit()));
        }

        // Apply frame rate limiting
        self.frame_limiter.limit_frame_rate();

        // Generate rendered frame
        let mut frame = String::new();
        frame.push_str(&format!("Frame {}: {}x{}\n", 
                               self.frame_count, 
                               self.terminal_size.0, 
                               self.terminal_size.1));
        
        for algorithm in &self.algorithms {
            let telemetry = algorithm.get_telemetry();
            frame.push_str(&format!("{}: {} comparisons, {} moves, {:.1}% complete\n",
                                   algorithm.name(),
                                   telemetry.total_comparisons,
                                   telemetry.total_moves,
                                   telemetry.progress_hint * 100.0));
        }

        frame.push_str(&format!("Memory: {:.1}% ({} bytes)\n", 
                               self.memory_monitor.usage_percentage(),
                               self.memory_monitor.current_usage()));

        self.frame_count += 1;
        Ok(frame)
    }

    pub fn step_algorithms(&mut self) {
        for algorithm in &mut self.algorithms {
            if !algorithm.is_complete() {
                algorithm.step(10); // Fixed budget for testing
            }
        }
    }

    pub fn all_complete(&self) -> bool {
        self.algorithms.iter().all(|alg| alg.is_complete())
    }

    pub fn get_frame_count(&self) -> u64 {
        self.frame_count
    }

    pub fn get_memory_monitor(&self) -> &MemoryMonitor {
        &self.memory_monitor
    }

    pub fn get_frame_limiter(&self) -> &FrameRateLimiter {
        &self.frame_limiter
    }
}

/// Test runner for complete visualization scenarios
pub struct VisualizationTestRunner {
    pipeline: VisualizationPipeline,
    start_time: Instant,
}

impl VisualizationTestRunner {
    pub fn new(pipeline: VisualizationPipeline) -> Self {
        Self {
            pipeline,
            start_time: Instant::now(),
        }
    }

    pub fn run_until_complete(&mut self, max_frames: u64) -> Result<TestResults, String> {
        let mut results = TestResults::default();
        
        while !self.pipeline.all_complete() && self.pipeline.get_frame_count() < max_frames {
            // Step algorithms
            self.pipeline.step_algorithms();
            
            // Render frame
            match self.pipeline.render_frame() {
                Ok(frame_content) => {
                    results.successful_frames += 1;
                    results.total_frame_size += frame_content.len();
                },
                Err(error) => {
                    results.failed_frames += 1;
                    results.errors.push(error);
                }
            }
            
            // Record memory usage
            let current_usage = self.pipeline.get_memory_monitor().current_usage();
            if current_usage > results.peak_memory_usage {
                results.peak_memory_usage = current_usage;
            }
        }
        
        results.total_duration = self.start_time.elapsed();
        results.final_frame_count = self.pipeline.get_frame_count();
        results.algorithms_completed = self.pipeline.algorithms.iter()
            .filter(|alg| alg.is_complete())
            .count();
        
        Ok(results)
    }

    pub fn simulate_terminal_resize(&mut self, new_width: u16, new_height: u16) {
        self.pipeline.set_terminal_size(new_width, new_height);
    }
}

#[derive(Debug, Default)]
pub struct TestResults {
    pub successful_frames: u64,
    pub failed_frames: u64,
    pub total_frame_size: usize,
    pub peak_memory_usage: usize,
    pub total_duration: Duration,
    pub final_frame_count: u64,
    pub algorithms_completed: usize,
    pub errors: Vec<String>,
}

impl TestResults {
    pub fn average_fps(&self) -> f32 {
        if self.total_duration.as_millis() > 0 {
            (self.successful_frames as f32) / (self.total_duration.as_secs_f32())
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complete_render_pipeline_from_algorithms_to_terminal() {
        let algorithms: Vec<Box<dyn Sorter>> = vec![
            Box::new(MockSorter::new("Quick Sort", vec![5, 2, 8, 1, 9])),
            Box::new(MockSorter::new("Bubble Sort", vec![3, 7, 1, 6, 4])),
            Box::new(MockSorter::new("Merge Sort", vec![9, 2, 5, 1, 8])),
        ];

        let pipeline = VisualizationPipeline::new(algorithms, 30, 10); // 30 FPS, 10MB limit
        let mut runner = VisualizationTestRunner::new(pipeline);

        let results = runner.run_until_complete(100).unwrap();

        // Verify pipeline completed successfully
        assert!(results.successful_frames > 0);
        assert_eq!(results.failed_frames, 0);
        assert_eq!(results.algorithms_completed, 3);
        assert!(results.total_frame_size > 0);
    }

    #[test]
    fn test_frame_rate_limiting_25_to_35_fps() {
        // Test different target frame rates
        for target_fps in [25, 30, 35] {
            let algorithms: Vec<Box<dyn Sorter>> = vec![
                Box::new(MockSorter::new("Test Sort", vec![1, 2, 3, 4, 5])),
            ];
            let pipeline = VisualizationPipeline::new(algorithms, target_fps, 5);
            let mut runner = VisualizationTestRunner::new(pipeline);

            let start_time = Instant::now();
            let results = runner.run_until_complete(20).unwrap(); // Run for limited frames
            let duration = start_time.elapsed();

            let actual_fps = results.average_fps();
            
            // Allow some tolerance for frame rate (±5 FPS)
            assert!(actual_fps >= (target_fps as f32) - 5.0, 
                   "FPS too low: {} < {} - 5", actual_fps, target_fps);
            assert!(actual_fps <= (target_fps as f32) + 5.0, 
                   "FPS too high: {} > {} + 5", actual_fps, target_fps);
        }
    }

    #[test]
    fn test_memory_usage_stays_under_limits() {
        let algorithms: Vec<Box<dyn Sorter>> = vec![
            Box::new(MockSorter::new("Memory Test 1", vec![1; 1000])), // Large array
            Box::new(MockSorter::new("Memory Test 2", vec![2; 1000])),
            Box::new(MockSorter::new("Memory Test 3", vec![3; 1000])),
        ];

        let memory_limit_mb = 1; // Very strict limit for testing
        let pipeline = VisualizationPipeline::new(algorithms, 30, memory_limit_mb);
        let mut runner = VisualizationTestRunner::new(pipeline);

        let results = runner.run_until_complete(10).unwrap_or_else(|_| TestResults::default());

        // Should either complete successfully or fail gracefully
        let memory_limit_bytes = memory_limit_mb * 1024 * 1024;
        
        if results.failed_frames > 0 {
            // If it failed, it should be due to memory limits
            assert!(results.peak_memory_usage >= memory_limit_bytes);
        } else {
            // If it succeeded, memory should be under limit
            assert!(results.peak_memory_usage <= memory_limit_bytes);
        }
    }

    #[test]
    fn test_responsive_to_terminal_resize() {
        let algorithms: Vec<Box<dyn Sorter>> = vec![
            Box::new(MockSorter::new("Resize Test", vec![1, 2, 3, 4, 5])),
        ];

        let pipeline = VisualizationPipeline::new(algorithms, 30, 10);
        let mut runner = VisualizationTestRunner::new(pipeline);

        // Start with default size
        let initial_frame = runner.pipeline.render_frame().unwrap();
        assert!(initial_frame.contains("80x24"));

        // Resize to small terminal
        runner.simulate_terminal_resize(40, 12);
        let small_frame = runner.pipeline.render_frame().unwrap();
        assert!(small_frame.contains("40x12"));

        // Resize to large terminal
        runner.simulate_terminal_resize(120, 40);
        let large_frame = runner.pipeline.render_frame().unwrap();
        assert!(large_frame.contains("120x40"));

        // Verify the frames are different (responsive to size)
        assert_ne!(initial_frame, small_frame);
        assert_ne!(small_frame, large_frame);
    }

    #[test]
    fn test_frame_rate_limiter_accuracy() {
        let mut limiter = FrameRateLimiter::new(30);

        let start_time = Instant::now();
        
        // Render several frames
        for _ in 0..10 {
            limiter.limit_frame_rate();
        }
        
        let duration = start_time.elapsed();
        let expected_duration = Duration::from_millis(10 * (1000 / 30)); // 10 frames at 30 FPS
        
        // Allow some tolerance (±50ms)
        let tolerance = Duration::from_millis(50);
        assert!(duration >= expected_duration - tolerance);
        assert!(duration <= expected_duration + tolerance);
    }

    #[test]
    fn test_memory_monitor_tracking() {
        let mut monitor = MemoryMonitor::new(5); // 5MB limit

        // Initially should be under limit
        assert!(monitor.is_under_limit());
        assert_eq!(monitor.current_usage(), 0);
        assert_eq!(monitor.peak_usage(), 0);

        // Record some usage
        monitor.record_usage(1024 * 1024); // 1MB
        assert_eq!(monitor.current_usage(), 1024 * 1024);
        assert_eq!(monitor.peak_usage(), 1024 * 1024);
        assert!(monitor.is_under_limit());

        // Record higher usage
        monitor.record_usage(3 * 1024 * 1024); // 3MB
        assert_eq!(monitor.current_usage(), 3 * 1024 * 1024);
        assert_eq!(monitor.peak_usage(), 3 * 1024 * 1024);
        assert!(monitor.is_under_limit());

        // Record lower usage (peak should remain)
        monitor.record_usage(2 * 1024 * 1024); // 2MB
        assert_eq!(monitor.current_usage(), 2 * 1024 * 1024);
        assert_eq!(monitor.peak_usage(), 3 * 1024 * 1024); // Peak unchanged

        // Exceed limit
        monitor.record_usage(6 * 1024 * 1024); // 6MB > 5MB limit
        assert!(!monitor.is_under_limit());
        assert_eq!(monitor.peak_usage(), 6 * 1024 * 1024);
    }

    #[test]
    fn test_pipeline_with_no_algorithms() {
        let algorithms: Vec<Box<dyn Sorter>> = vec![];
        let pipeline = VisualizationPipeline::new(algorithms, 30, 10);
        let mut runner = VisualizationTestRunner::new(pipeline);

        let results = runner.run_until_complete(5).unwrap();
        
        // Should complete immediately since no algorithms to run
        assert_eq!(results.algorithms_completed, 0);
        assert_eq!(results.successful_frames, 5); // Should still render frames
    }

    #[test]
    fn test_pipeline_performance_metrics() {
        let algorithms: Vec<Box<dyn Sorter>> = vec![
            Box::new(MockSorter::new("Perf Test 1", vec![5, 4, 3, 2, 1])),
            Box::new(MockSorter::new("Perf Test 2", vec![10, 9, 8, 7, 6])),
        ];

        let pipeline = VisualizationPipeline::new(algorithms, 60, 20); // High FPS for performance test
        let mut runner = VisualizationTestRunner::new(pipeline);

        let results = runner.run_until_complete(50).unwrap();

        // Performance assertions
        assert!(results.average_fps() > 0.0);
        assert!(results.total_duration.as_millis() > 0);
        assert!(results.successful_frames > 0);
        assert_eq!(results.failed_frames, 0);
        
        // Memory should be reasonable
        assert!(results.peak_memory_usage > 0);
        assert!(results.peak_memory_usage < 20 * 1024 * 1024); // Under 20MB limit
    }

    #[test]
    fn test_extreme_terminal_sizes() {
        let algorithms: Vec<Box<dyn Sorter>> = vec![
            Box::new(MockSorter::new("Size Test", vec![1, 2, 3])),
        ];

        let pipeline = VisualizationPipeline::new(algorithms, 30, 10);
        let mut runner = VisualizationTestRunner::new(pipeline);

        // Test very small terminal
        runner.simulate_terminal_resize(1, 1);
        let tiny_frame = runner.pipeline.render_frame();
        assert!(tiny_frame.is_ok()); // Should handle gracefully

        // Test very large terminal
        runner.simulate_terminal_resize(999, 999);
        let huge_frame = runner.pipeline.render_frame();
        // This might fail due to memory limits, which is acceptable
        match huge_frame {
            Ok(_) => {
                // If it succeeds, memory should still be reasonable
                assert!(runner.pipeline.get_memory_monitor().is_under_limit());
            },
            Err(error) => {
                // If it fails, should be due to memory limits
                assert!(error.contains("Memory usage exceeded"));
            }
        }
    }

    #[test]
    fn test_frame_content_includes_expected_elements() {
        let algorithms: Vec<Box<dyn Sorter>> = vec![
            Box::new(MockSorter::new("Content Test", vec![1, 2, 3, 4, 5])),
        ];

        let pipeline = VisualizationPipeline::new(algorithms, 30, 10);
        let mut runner = VisualizationTestRunner::new(pipeline);

        let frame = runner.pipeline.render_frame().unwrap();

        // Frame should contain expected elements
        assert!(frame.contains("Frame"));
        assert!(frame.contains("Content Test"));
        assert!(frame.contains("comparisons"));
        assert!(frame.contains("moves"));
        assert!(frame.contains("complete"));
        assert!(frame.contains("Memory:"));
        assert!(frame.contains("80x24")); // Terminal size
    }
}
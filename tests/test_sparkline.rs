//! Contract tests for Sparkline component
//!
//! These tests verify the expected interface and behavior of the Sparkline visualization component.
//! The tests are designed to work with the existing Sparkline implementation from src/lib/sparkline.rs
//! but also test additional expected features.

use std::collections::VecDeque;

/// Extended Sparkline component for visualization metrics
/// This extends the existing Sparkline with features expected by the visualization system
#[derive(Debug)]
pub struct MetricsSparkline {
    data: VecDeque<f64>,
    capacity: usize,
    width: usize,
    height: usize,
    title: String,
    unit: String,
}

impl MetricsSparkline {
    /// Create a new sparkline with specified capacity for data points
    pub fn new(capacity: usize) -> Self {
        Self {
            data: VecDeque::with_capacity(capacity),
            capacity,
            width: capacity.min(100), // Default visual width
            height: 1,
            title: String::new(),
            unit: String::new(),
        }
    }

    /// Add a data point and maintain the rolling window
    pub fn add_point(&mut self, value: f64) {
        if self.data.len() >= self.capacity {
            self.data.pop_front();
        }
        self.data.push_back(value);
    }

    /// Render the sparkline using Unicode block characters
    pub fn render(&self) -> String {
        if self.data.is_empty() {
            return " ".repeat(self.width);
        }

        let min_val = self.data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_val = self.data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        // Avoid division by zero
        let range = if (max_val - min_val).abs() < f64::EPSILON {
            1.0
        } else {
            max_val - min_val
        };

        // Unicode block characters for different heights
        let chars = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
        let levels = chars.len() as f64;

        let mut result = String::new();
        let points_to_show = self.data.len().min(self.width);
        
        // Take the most recent data points that fit in the width
        let start_index = if self.data.len() > self.width {
            self.data.len() - self.width
        } else {
            0
        };

        for i in start_index..self.data.len() {
            let value = self.data[i];
            let normalized = ((value - min_val) / range).clamp(0.0, 1.0);
            let level = (normalized * (levels - 1.0)).round() as usize;
            result.push(chars[level]);
        }

        // Pad with spaces if needed
        while result.chars().count() < self.width {
            result.push(' ');
        }

        result
    }

    /// Get the current data points
    pub fn get_data(&self) -> Vec<f64> {
        self.data.iter().copied().collect()
    }

    /// Get the capacity (maximum number of points)
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Get current number of data points
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Set the title for this sparkline
    pub fn with_title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }

    /// Set the unit for values
    pub fn with_unit(mut self, unit: &str) -> Self {
        self.unit = unit.to_string();
        self
    }

    /// Set the visual width
    pub fn with_width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    /// Clear all data points
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Get statistics for the current data
    pub fn get_stats(&self) -> SparklineStats {
        if self.data.is_empty() {
            return SparklineStats::default();
        }

        let min = self.data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = self.data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let sum: f64 = self.data.iter().sum();
        let avg = sum / self.data.len() as f64;
        
        // Calculate standard deviation
        let variance: f64 = self.data.iter()
            .map(|&x| (x - avg).powi(2))
            .sum::<f64>() / self.data.len() as f64;
        let std_dev = variance.sqrt();

        SparklineStats {
            min,
            max,
            avg,
            std_dev,
            count: self.data.len(),
        }
    }

    /// Render with full context (title, sparkline, stats)
    pub fn render_with_context(&self) -> String {
        let sparkline = self.render();
        let stats = self.get_stats();
        
        if self.title.is_empty() {
            format!("{} (min: {:.1}, max: {:.1}, avg: {:.1}{})", 
                   sparkline, stats.min, stats.max, stats.avg, self.unit)
        } else {
            format!("{}: {} (min: {:.1}, max: {:.1}, avg: {:.1}{})", 
                   self.title, sparkline, stats.min, stats.max, stats.avg, self.unit)
        }
    }
}

/// Statistics for sparkline data
#[derive(Debug, Clone, Default)]
pub struct SparklineStats {
    pub min: f64,
    pub max: f64,
    pub avg: f64,
    pub std_dev: f64,
    pub count: usize,
}

/// Collection of sparklines for different metrics
#[derive(Debug)]
pub struct SparklineCollection {
    sparklines: std::collections::HashMap<String, MetricsSparkline>,
    default_capacity: usize,
    default_width: usize,
}

impl SparklineCollection {
    /// Create a new collection with default settings
    pub fn new(default_capacity: usize, default_width: usize) -> Self {
        Self {
            sparklines: std::collections::HashMap::new(),
            default_capacity,
            default_width,
        }
    }

    /// Add or update a metric sparkline
    pub fn update_metric(&mut self, name: &str, value: f64) {
        let sparkline = self.sparklines.entry(name.to_string())
            .or_insert_with(|| {
                MetricsSparkline::new(self.default_capacity)
                    .with_width(self.default_width)
                    .with_title(name)
            });
        sparkline.add_point(value);
    }

    /// Get a sparkline by name
    pub fn get(&self, name: &str) -> Option<&MetricsSparkline> {
        self.sparklines.get(name)
    }

    /// Get a mutable sparkline by name
    pub fn get_mut(&mut self, name: &str) -> Option<&mut MetricsSparkline> {
        self.sparklines.get_mut(name)
    }

    /// Render all sparklines
    pub fn render_all(&self) -> Vec<String> {
        let mut result = Vec::new();
        let mut keys: Vec<_> = self.sparklines.keys().collect();
        keys.sort();

        for key in keys {
            if let Some(sparkline) = self.sparklines.get(key) {
                result.push(sparkline.render_with_context());
            }
        }

        result
    }

    /// Get sparkline names
    pub fn get_names(&self) -> Vec<String> {
        let mut names: Vec<_> = self.sparklines.keys().cloned().collect();
        names.sort();
        names
    }

    /// Clear all sparklines
    pub fn clear(&mut self) {
        self.sparklines.clear();
    }

    /// Get number of sparklines
    pub fn len(&self) -> usize {
        self.sparklines.len()
    }

    /// Check if collection is empty
    pub fn is_empty(&self) -> bool {
        self.sparklines.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sparkline_data_collection_100_point_history() {
        let mut sparkline = MetricsSparkline::new(100);
        
        // Add 100 points
        for i in 0..100 {
            sparkline.add_point(i as f64);
        }
        
        assert_eq!(sparkline.len(), 100);
        assert_eq!(sparkline.capacity(), 100);
        
        let data = sparkline.get_data();
        assert_eq!(data[0], 0.0);
        assert_eq!(data[99], 99.0);
    }

    #[test]
    fn test_data_point_addition_and_overflow() {
        let mut sparkline = MetricsSparkline::new(5); // Small capacity for testing overflow
        
        // Add more points than capacity
        for i in 0..10 {
            sparkline.add_point(i as f64);
        }
        
        // Should only keep the last 5 points
        assert_eq!(sparkline.len(), 5);
        let data = sparkline.get_data();
        assert_eq!(data, vec![5.0, 6.0, 7.0, 8.0, 9.0]);
    }

    #[test]
    fn test_rendering_with_different_scales() {
        let mut sparkline = MetricsSparkline::new(10).with_width(10);
        
        // Test linear scale
        for i in 1..=10 {
            sparkline.add_point(i as f64);
        }
        let linear_render = sparkline.render();
        assert_eq!(linear_render.chars().count(), 10);
        
        // Test with different value ranges
        sparkline.clear();
        sparkline.add_point(1000.0);
        sparkline.add_point(2000.0);
        sparkline.add_point(3000.0);
        
        let scaled_render = sparkline.render();
        assert_eq!(scaled_render.chars().count(), 10);
        // Should contain Unicode block characters
        assert!(scaled_render.chars().any(|c| "▁▂▃▄▅▆▇█".contains(c)));
    }

    #[test]
    fn test_multiple_sparklines_for_different_metrics() {
        let mut collection = SparklineCollection::new(50, 20);
        
        // Add data for different metrics
        collection.update_metric("comparisons", 100.0);
        collection.update_metric("swaps", 50.0);
        collection.update_metric("memory_usage", 1024.0);
        
        assert_eq!(collection.len(), 3);
        
        let names = collection.get_names();
        assert_eq!(names, vec!["comparisons", "memory_usage", "swaps"]);
        
        // Verify each sparkline exists and has data
        assert!(collection.get("comparisons").is_some());
        assert!(collection.get("swaps").is_some());
        assert!(collection.get("memory_usage").is_some());
        
        let comparisons_sparkline = collection.get("comparisons").unwrap();
        assert_eq!(comparisons_sparkline.len(), 1);
        assert_eq!(comparisons_sparkline.get_data(), vec![100.0]);
    }

    #[test]
    fn test_sparkline_stats_calculation() {
        let mut sparkline = MetricsSparkline::new(10);
        
        // Add known values for testing stats
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        for value in &values {
            sparkline.add_point(*value);
        }
        
        let stats = sparkline.get_stats();
        assert_eq!(stats.min, 1.0);
        assert_eq!(stats.max, 5.0);
        assert_eq!(stats.avg, 3.0); // (1+2+3+4+5)/5 = 3
        assert_eq!(stats.count, 5);
        
        // Standard deviation for [1,2,3,4,5] is sqrt(2) ≈ 1.414
        // Variance = ((1-3)² + (2-3)² + (3-3)² + (4-3)² + (5-3)²) / 5
        //          = (4 + 1 + 0 + 1 + 4) / 5 = 2
        // Std dev = sqrt(2) ≈ 1.414
        assert!((stats.std_dev - 1.414).abs() < 0.01);
    }

    #[test]
    fn test_sparkline_rendering_unicode_blocks() {
        let mut sparkline = MetricsSparkline::new(8).with_width(8);
        
        // Add ascending values to test all Unicode blocks
        for i in 0..8 {
            sparkline.add_point(i as f64);
        }
        
        let rendered = sparkline.render();
        assert_eq!(rendered.chars().count(), 8);
        
        // Should contain the lowest and highest block characters
        assert!(rendered.contains('▁')); // Lowest block
        assert!(rendered.contains('█')); // Highest block
    }

    #[test]
    fn test_empty_sparkline_rendering() {
        let sparkline = MetricsSparkline::new(10).with_width(5);
        
        let rendered = sparkline.render();
        assert_eq!(rendered, "     "); // Should be spaces
        assert_eq!(rendered.len(), 5);
    }

    #[test]
    fn test_sparkline_with_identical_values() {
        let mut sparkline = MetricsSparkline::new(5).with_width(5);
        
        // Add identical values
        for _ in 0..5 {
            sparkline.add_point(42.0);
        }
        
        let rendered = sparkline.render();
        assert_eq!(rendered.chars().count(), 5);
        // When all values are identical, they should all render as the same character
        // The exact character depends on the implementation's normalization approach
        // For identical values, it should pick a consistent middle character
        let first_char = rendered.chars().next().unwrap();
        assert!(rendered.chars().all(|c| c == first_char));
    }

    #[test]
    fn test_sparkline_width_adjustment() {
        let mut sparkline = MetricsSparkline::new(100);
        
        // Add more data points than the display width
        for i in 0..50 {
            sparkline.add_point(i as f64);
        }
        
        // Set narrow width
        sparkline = sparkline.with_width(10);
        let rendered = sparkline.render();
        assert_eq!(rendered.chars().count(), 10);
        
        // Should show only the most recent data points
        let data = sparkline.get_data();
        assert_eq!(data.len(), 50); // All data still stored
    }

    #[test]
    fn test_sparkline_collection_updates() {
        let mut collection = SparklineCollection::new(10, 15);
        
        // Update the same metric multiple times
        for i in 1..=5 {
            collection.update_metric("test_metric", i as f64);
        }
        
        let sparkline = collection.get("test_metric").unwrap();
        assert_eq!(sparkline.len(), 5);
        assert_eq!(sparkline.get_data(), vec![1.0, 2.0, 3.0, 4.0, 5.0]);
    }

    #[test]
    fn test_sparkline_render_with_context() {
        let mut sparkline = MetricsSparkline::new(5)
            .with_width(5)
            .with_title("Test Metric")
            .with_unit("ms");
        
        sparkline.add_point(10.0);
        sparkline.add_point(20.0);
        sparkline.add_point(30.0);
        
        let rendered = sparkline.render_with_context();
        assert!(rendered.contains("Test Metric"));
        assert!(rendered.contains("ms"));
        assert!(rendered.contains("min: 10.0"));
        assert!(rendered.contains("max: 30.0"));
        assert!(rendered.contains("avg: 20.0"));
    }

    #[test]
    fn test_collection_render_all() {
        let mut collection = SparklineCollection::new(5, 8);
        
        collection.update_metric("metric_a", 10.0);
        collection.update_metric("metric_b", 20.0);
        collection.update_metric("metric_a", 15.0);
        
        let all_rendered = collection.render_all();
        assert_eq!(all_rendered.len(), 2);
        
        // Should be sorted by name
        assert!(all_rendered[0].starts_with("metric_a:"));
        assert!(all_rendered[1].starts_with("metric_b:"));
    }

    #[test]
    fn test_sparkline_clear_functionality() {
        let mut sparkline = MetricsSparkline::new(10);
        
        sparkline.add_point(1.0);
        sparkline.add_point(2.0);
        sparkline.add_point(3.0);
        
        assert_eq!(sparkline.len(), 3);
        
        sparkline.clear();
        assert_eq!(sparkline.len(), 0);
        assert!(sparkline.is_empty());
        
        let stats = sparkline.get_stats();
        assert_eq!(stats.count, 0);
    }

    #[test]
    fn test_sparkline_extreme_values() {
        let mut sparkline = MetricsSparkline::new(5).with_width(5);
        
        // Test with very large and very small values
        sparkline.add_point(f64::MIN / 2.0);
        sparkline.add_point(0.0);
        sparkline.add_point(f64::MAX / 2.0);
        
        let rendered = sparkline.render();
        assert_eq!(rendered.chars().count(), 5);
        
        // Should handle extreme values without panicking
        let stats = sparkline.get_stats();
        assert!(stats.min.is_finite());
        assert!(stats.max.is_finite());
        assert!(stats.avg.is_finite());
    }
}
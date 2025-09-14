//! Memory metrics model for real-time memory tracking

use crate::models::session::AlgorithmType;
use std::collections::HashMap;
use std::time::Instant;

/// Memory metrics for a single algorithm
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryMetrics {
    /// Algorithm this metric belongs to
    pub algorithm_type: AlgorithmType,
    /// Current memory usage in bytes
    pub current_usage_bytes: usize,
    /// Peak memory usage in bytes seen so far
    pub peak_usage_bytes: usize,
    /// When this metric was last updated
    pub last_updated: Instant,
}

impl MemoryMetrics {
    /// Create new memory metrics for an algorithm
    pub fn new(algorithm_type: AlgorithmType) -> Self {
        Self {
            algorithm_type,
            current_usage_bytes: 0,
            peak_usage_bytes: 0,
            last_updated: Instant::now(),
        }
    }

    /// Update memory usage with new value
    pub fn update(&mut self, current_bytes: usize) {
        self.current_usage_bytes = current_bytes;
        if current_bytes > self.peak_usage_bytes {
            self.peak_usage_bytes = current_bytes;
        }
        self.last_updated = Instant::now();
    }

    /// Reset metrics to zero
    pub fn reset(&mut self) {
        self.current_usage_bytes = 0;
        self.peak_usage_bytes = 0;
        self.last_updated = Instant::now();
    }

    /// Get age of this metric (time since last update)
    pub fn age(&self) -> std::time::Duration {
        self.last_updated.elapsed()
    }

    /// Check if this metric is stale (hasn't been updated recently)
    pub fn is_stale(&self, threshold: std::time::Duration) -> bool {
        self.age() > threshold
    }

    /// Format current memory usage as human-readable string
    pub fn format_current(&self) -> String {
        Self::format_bytes(self.current_usage_bytes)
    }

    /// Format peak memory usage as human-readable string
    pub fn format_peak(&self) -> String {
        Self::format_bytes(self.peak_usage_bytes)
    }

    /// Format bytes into human-readable string
    pub fn format_bytes(bytes: usize) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
        let mut size = bytes as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        if unit_index == 0 {
            format!("{}B", bytes)
        } else {
            format!("{:.1}{}", size, UNITS[unit_index])
        }
    }

    /// Get memory efficiency ratio (current / peak)
    pub fn efficiency_ratio(&self) -> f64 {
        if self.peak_usage_bytes == 0 {
            1.0
        } else {
            self.current_usage_bytes as f64 / self.peak_usage_bytes as f64
        }
    }

    /// Check if memory usage is currently at peak
    pub fn is_at_peak(&self) -> bool {
        self.current_usage_bytes == self.peak_usage_bytes
    }
}

/// Container for tracking memory metrics across multiple algorithms
#[derive(Debug, Clone)]
pub struct MemoryMetricsCollection {
    /// Metrics for each algorithm
    metrics: HashMap<AlgorithmType, MemoryMetrics>,
    /// Global peak memory across all algorithms
    global_peak: usize,
    /// When the collection was created
    start_time: Instant,
    /// Whether real-time updates are enabled
    real_time_enabled: bool,
    /// Update frequency for real-time updates
    update_frequency: std::time::Duration,
}

impl MemoryMetricsCollection {
    /// Create new memory metrics collection
    pub fn new() -> Self {
        Self {
            metrics: HashMap::new(),
            global_peak: 0,
            start_time: Instant::now(),
            real_time_enabled: true,
            update_frequency: std::time::Duration::from_millis(100), // 10 FPS updates
        }
    }

    /// Create collection with specific algorithms
    pub fn with_algorithms(algorithms: &[AlgorithmType]) -> Self {
        let mut collection = Self::new();
        for &algorithm in algorithms {
            collection.add_algorithm(algorithm);
        }
        collection
    }

    /// Add an algorithm to tracking
    pub fn add_algorithm(&mut self, algorithm_type: AlgorithmType) {
        self.metrics.insert(algorithm_type, MemoryMetrics::new(algorithm_type));
    }

    /// Remove an algorithm from tracking
    pub fn remove_algorithm(&mut self, algorithm_type: AlgorithmType) -> Option<MemoryMetrics> {
        self.metrics.remove(&algorithm_type)
    }

    /// Update memory usage for an algorithm
    pub fn update_algorithm(&mut self, algorithm_type: AlgorithmType, current_bytes: usize) {
        if let Some(metrics) = self.metrics.get_mut(&algorithm_type) {
            metrics.update(current_bytes);
            
            // Update global peak
            if current_bytes > self.global_peak {
                self.global_peak = current_bytes;
            }
        } else {
            // Auto-add algorithm if it doesn't exist
            let mut metrics = MemoryMetrics::new(algorithm_type);
            metrics.update(current_bytes);
            self.metrics.insert(algorithm_type, metrics);
            
            if current_bytes > self.global_peak {
                self.global_peak = current_bytes;
            }
        }
    }

    /// Get metrics for specific algorithm
    pub fn get_metrics(&self, algorithm_type: AlgorithmType) -> Option<&MemoryMetrics> {
        self.metrics.get(&algorithm_type)
    }

    /// Get mutable metrics for specific algorithm
    pub fn get_metrics_mut(&mut self, algorithm_type: AlgorithmType) -> Option<&mut MemoryMetrics> {
        self.metrics.get_mut(&algorithm_type)
    }

    /// Get all metrics
    pub fn get_all_metrics(&self) -> &HashMap<AlgorithmType, MemoryMetrics> {
        &self.metrics
    }

    /// Get metrics as a sorted vector for display
    pub fn get_sorted_metrics(&self) -> Vec<(AlgorithmType, &MemoryMetrics)> {
        let mut metrics: Vec<_> = self.metrics.iter().map(|(&alg, metrics)| (alg, metrics)).collect();
        metrics.sort_by_key(|(alg, _)| alg.to_index());
        metrics
    }

    /// Get global peak memory usage
    pub fn get_global_peak(&self) -> usize {
        self.global_peak
    }

    /// Get total current memory usage across all algorithms
    pub fn get_total_current_usage(&self) -> usize {
        self.metrics.values().map(|m| m.current_usage_bytes).sum()
    }

    /// Get total peak memory usage across all algorithms
    pub fn get_total_peak_usage(&self) -> usize {
        self.metrics.values().map(|m| m.peak_usage_bytes).sum()
    }

    /// Clear all metrics
    pub fn clear(&mut self) {
        self.metrics.clear();
        self.global_peak = 0;
    }

    /// Reset all metrics to zero but keep tracking
    pub fn reset_all(&mut self) {
        for metrics in self.metrics.values_mut() {
            metrics.reset();
        }
        self.global_peak = 0;
    }

    /// Enable or disable real-time updates
    pub fn set_real_time_enabled(&mut self, enabled: bool) {
        self.real_time_enabled = enabled;
    }

    /// Check if real-time updates are enabled
    pub fn is_real_time_enabled(&self) -> bool {
        self.real_time_enabled
    }

    /// Set update frequency for real-time updates
    pub fn set_update_frequency(&mut self, frequency: std::time::Duration) {
        self.update_frequency = frequency;
    }

    /// Get update frequency
    pub fn get_update_frequency(&self) -> std::time::Duration {
        self.update_frequency
    }

    /// Get algorithms that need updates (based on staleness)
    pub fn get_stale_algorithms(&self, threshold: std::time::Duration) -> Vec<AlgorithmType> {
        self.metrics
            .iter()
            .filter_map(|(&alg, metrics)| {
                if metrics.is_stale(threshold) {
                    Some(alg)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get memory display values for all algorithms
    pub fn get_memory_display_values(&self) -> Vec<(AlgorithmType, crate::models::display_mode::MemoryDisplayValue)> {
        self.get_sorted_metrics()
            .into_iter()
            .map(|(alg, metrics)| {
                let display_value = if metrics.current_usage_bytes > 0 {
                    crate::models::display_mode::MemoryDisplayValue::Bytes(metrics.current_usage_bytes)
                } else {
                    crate::models::display_mode::MemoryDisplayValue::NotAvailable
                };
                (alg, display_value)
            })
            .collect()
    }

    /// Get collection age (time since creation)
    pub fn age(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }

    /// Get memory statistics summary
    pub fn get_statistics(&self) -> MemoryStatistics {
        MemoryStatistics::from_collection(self)
    }

    /// Check if any algorithm is currently using memory
    pub fn has_active_usage(&self) -> bool {
        self.metrics.values().any(|m| m.current_usage_bytes > 0)
    }

    /// Get algorithm with highest current memory usage
    pub fn get_highest_current_usage(&self) -> Option<(AlgorithmType, usize)> {
        self.metrics
            .iter()
            .max_by_key(|(_, metrics)| metrics.current_usage_bytes)
            .map(|(&alg, metrics)| (alg, metrics.current_usage_bytes))
    }

    /// Get algorithm with highest peak memory usage
    pub fn get_highest_peak_usage(&self) -> Option<(AlgorithmType, usize)> {
        self.metrics
            .iter()
            .max_by_key(|(_, metrics)| metrics.peak_usage_bytes)
            .map(|(&alg, metrics)| (alg, metrics.peak_usage_bytes))
    }
}

impl Default for MemoryMetricsCollection {
    fn default() -> Self {
        Self::new()
    }
}

/// Memory usage statistics summary
#[derive(Debug, Clone)]
pub struct MemoryStatistics {
    /// Total algorithms being tracked
    pub algorithm_count: usize,
    /// Total current memory usage
    pub total_current: usize,
    /// Total peak memory usage
    pub total_peak: usize,
    /// Global peak across all algorithms
    pub global_peak: usize,
    /// Average current usage per algorithm
    pub average_current: usize,
    /// Average peak usage per algorithm
    pub average_peak: usize,
    /// Most memory-efficient algorithm
    pub most_efficient: Option<AlgorithmType>,
    /// Least memory-efficient algorithm
    pub least_efficient: Option<AlgorithmType>,
}

impl MemoryStatistics {
    /// Generate statistics from a memory metrics collection
    pub fn from_collection(collection: &MemoryMetricsCollection) -> Self {
        let algorithm_count = collection.metrics.len();
        let total_current = collection.get_total_current_usage();
        let total_peak = collection.get_total_peak_usage();
        let global_peak = collection.get_global_peak();

        let average_current = if algorithm_count > 0 { total_current / algorithm_count } else { 0 };
        let average_peak = if algorithm_count > 0 { total_peak / algorithm_count } else { 0 };

        // Find most and least efficient algorithms
        let mut most_efficient = None;
        let mut least_efficient = None;
        let mut best_efficiency = 0.0f64;
        let mut worst_efficiency = f64::INFINITY;

        for (&alg, metrics) in &collection.metrics {
            let efficiency = metrics.efficiency_ratio();
            if efficiency > best_efficiency {
                best_efficiency = efficiency;
                most_efficient = Some(alg);
            }
            if efficiency < worst_efficiency {
                worst_efficiency = efficiency;
                least_efficient = Some(alg);
            }
        }

        Self {
            algorithm_count,
            total_current,
            total_peak,
            global_peak,
            average_current,
            average_peak,
            most_efficient,
            least_efficient,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_metrics_creation() {
        let metrics = MemoryMetrics::new(AlgorithmType::BubbleSort);
        
        assert_eq!(metrics.algorithm_type, AlgorithmType::BubbleSort);
        assert_eq!(metrics.current_usage_bytes, 0);
        assert_eq!(metrics.peak_usage_bytes, 0);
        assert!(metrics.age().as_millis() < 100); // Recently created
    }

    #[test]
    fn test_memory_metrics_update() {
        let mut metrics = MemoryMetrics::new(AlgorithmType::QuickSort);
        
        metrics.update(1024);
        assert_eq!(metrics.current_usage_bytes, 1024);
        assert_eq!(metrics.peak_usage_bytes, 1024);
        
        metrics.update(512);
        assert_eq!(metrics.current_usage_bytes, 512);
        assert_eq!(metrics.peak_usage_bytes, 1024); // Peak should remain
        
        metrics.update(2048);
        assert_eq!(metrics.current_usage_bytes, 2048);
        assert_eq!(metrics.peak_usage_bytes, 2048); // New peak
    }

    #[test]
    fn test_memory_formatting() {
        assert_eq!(MemoryMetrics::format_bytes(512), "512B");
        assert_eq!(MemoryMetrics::format_bytes(1024), "1.0KB");
        assert_eq!(MemoryMetrics::format_bytes(1536), "1.5KB");
        assert_eq!(MemoryMetrics::format_bytes(1048576), "1.0MB");
        
        let metrics = MemoryMetrics::new(AlgorithmType::MergeSort);
        assert_eq!(metrics.format_current(), "0B");
    }

    #[test]
    fn test_memory_efficiency() {
        let mut metrics = MemoryMetrics::new(AlgorithmType::HeapSort);
        
        metrics.update(1024);
        assert_eq!(metrics.efficiency_ratio(), 1.0); // At peak
        assert!(metrics.is_at_peak());
        
        metrics.update(512);
        assert_eq!(metrics.efficiency_ratio(), 0.5); // Half of peak
        assert!(!metrics.is_at_peak());
    }

    #[test]
    fn test_memory_collection_creation() {
        let collection = MemoryMetricsCollection::new();
        
        assert_eq!(collection.metrics.len(), 0);
        assert_eq!(collection.global_peak, 0);
        assert!(collection.is_real_time_enabled());
        
        let algorithms = vec![AlgorithmType::BubbleSort, AlgorithmType::QuickSort];
        let collection_with_algs = MemoryMetricsCollection::with_algorithms(&algorithms);
        assert_eq!(collection_with_algs.metrics.len(), 2);
    }

    #[test]
    fn test_memory_collection_operations() {
        let mut collection = MemoryMetricsCollection::new();
        
        // Add algorithm
        collection.add_algorithm(AlgorithmType::BubbleSort);
        assert!(collection.get_metrics(AlgorithmType::BubbleSort).is_some());
        
        // Update memory
        collection.update_algorithm(AlgorithmType::BubbleSort, 1024);
        let metrics = collection.get_metrics(AlgorithmType::BubbleSort).unwrap();
        assert_eq!(metrics.current_usage_bytes, 1024);
        assert_eq!(collection.get_global_peak(), 1024);
        
        // Remove algorithm
        let removed = collection.remove_algorithm(AlgorithmType::BubbleSort);
        assert!(removed.is_some());
        assert!(collection.get_metrics(AlgorithmType::BubbleSort).is_none());
    }

    #[test]
    fn test_memory_collection_totals() {
        let mut collection = MemoryMetricsCollection::new();
        
        collection.update_algorithm(AlgorithmType::BubbleSort, 1024);
        collection.update_algorithm(AlgorithmType::QuickSort, 2048);
        
        assert_eq!(collection.get_total_current_usage(), 3072);
        assert_eq!(collection.get_total_peak_usage(), 3072);
        
        // Update one algorithm to lower value
        collection.update_algorithm(AlgorithmType::BubbleSort, 512);
        assert_eq!(collection.get_total_current_usage(), 2560);
        assert_eq!(collection.get_total_peak_usage(), 3072); // Peak remains
    }

    #[test]
    fn test_memory_statistics() {
        let mut collection = MemoryMetricsCollection::new();
        
        collection.update_algorithm(AlgorithmType::BubbleSort, 1024);
        collection.update_algorithm(AlgorithmType::QuickSort, 2048);
        
        let stats = collection.get_statistics();
        assert_eq!(stats.algorithm_count, 2);
        assert_eq!(stats.total_current, 3072);
        assert_eq!(stats.average_current, 1536);
    }

    #[test]
    fn test_memory_display_values() {
        let mut collection = MemoryMetricsCollection::new();
        
        collection.update_algorithm(AlgorithmType::BubbleSort, 1024);
        collection.add_algorithm(AlgorithmType::QuickSort); // 0 bytes
        
        let display_values = collection.get_memory_display_values();
        assert_eq!(display_values.len(), 2);
        
        // Find BubbleSort entry
        let bubble_entry = display_values.iter()
            .find(|(alg, _)| *alg == AlgorithmType::BubbleSort)
            .unwrap();
        assert_eq!(bubble_entry.1, crate::models::display_mode::MemoryDisplayValue::Bytes(1024));
        
        // Find QuickSort entry (should be N/A due to 0 bytes)
        let quick_entry = display_values.iter()
            .find(|(alg, _)| *alg == AlgorithmType::QuickSort)
            .unwrap();
        assert_eq!(quick_entry.1, crate::models::display_mode::MemoryDisplayValue::NotAvailable);
    }

    #[test]
    fn test_highest_usage_algorithms() {
        let mut collection = MemoryMetricsCollection::new();
        
        collection.update_algorithm(AlgorithmType::BubbleSort, 1024);
        collection.update_algorithm(AlgorithmType::QuickSort, 2048);
        collection.update_algorithm(AlgorithmType::MergeSort, 512);
        
        let highest_current = collection.get_highest_current_usage();
        assert_eq!(highest_current, Some((AlgorithmType::QuickSort, 2048)));
        
        let highest_peak = collection.get_highest_peak_usage();
        assert_eq!(highest_peak, Some((AlgorithmType::QuickSort, 2048)));
    }
}
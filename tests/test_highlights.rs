//! Contract tests for operation highlighting system
//!
//! These tests verify the expected interface and behavior of the highlighting system
//! that integrates with the Telemetry markers to show algorithm operations.

use std::collections::HashMap;

/// Color types for different operation highlights
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HighlightColor {
    Red,    // For swaps/moves
    Blue,   // For comparisons
    Green,  // For sorted elements
    Yellow, // For pivot elements
    Purple, // For special operations
}

/// Priority levels for highlight colors when operations overlap
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HighlightPriority {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

/// A single highlight with color and priority
#[derive(Debug, Clone)]
pub struct Highlight {
    pub color: HighlightColor,
    pub priority: HighlightPriority,
    pub operation_type: String,
}

impl Highlight {
    pub fn new(color: HighlightColor, priority: HighlightPriority, operation_type: &str) -> Self {
        Self {
            color,
            priority,
            operation_type: operation_type.to_string(),
        }
    }
}

/// Highlighting system that manages operation visualization
#[derive(Debug)]
pub struct HighlightSystem {
    /// Current highlights by array index
    highlights: HashMap<usize, Vec<Highlight>>,
    /// Default color mapping for operation types
    color_map: HashMap<String, (HighlightColor, HighlightPriority)>,
}

impl HighlightSystem {
    /// Create a new highlight system with default mappings
    pub fn new() -> Self {
        let mut color_map = HashMap::new();

        // Default operation -> (color, priority) mappings
        color_map.insert(
            "compare".to_string(),
            (HighlightColor::Blue, HighlightPriority::Medium),
        );
        color_map.insert(
            "swap".to_string(),
            (HighlightColor::Red, HighlightPriority::High),
        );
        color_map.insert(
            "move".to_string(),
            (HighlightColor::Red, HighlightPriority::High),
        );
        color_map.insert(
            "pivot".to_string(),
            (HighlightColor::Yellow, HighlightPriority::Critical),
        );
        color_map.insert(
            "sorted".to_string(),
            (HighlightColor::Green, HighlightPriority::Low),
        );
        color_map.insert(
            "cursor".to_string(),
            (HighlightColor::Purple, HighlightPriority::Medium),
        );

        Self {
            highlights: HashMap::new(),
            color_map,
        }
    }

    /// Clear all current highlights
    pub fn clear(&mut self) {
        self.highlights.clear();
    }

    /// Add highlight indices for comparisons
    pub fn highlight_comparison(&mut self, indices: &[usize]) {
        for &index in indices {
            let highlight =
                Highlight::new(HighlightColor::Blue, HighlightPriority::Medium, "compare");
            self.highlights
                .entry(index)
                .or_insert_with(Vec::new)
                .push(highlight);
        }
    }

    /// Add highlight indices for swaps
    pub fn highlight_swap(&mut self, indices: &[usize]) {
        for &index in indices {
            let highlight = Highlight::new(HighlightColor::Red, HighlightPriority::High, "swap");
            self.highlights
                .entry(index)
                .or_insert_with(Vec::new)
                .push(highlight);
        }
    }

    /// Add highlight for a specific operation type
    pub fn highlight_operation(&mut self, operation_type: &str, indices: &[usize]) {
        let (color, priority) = self
            .color_map
            .get(operation_type)
            .copied()
            .unwrap_or((HighlightColor::Purple, HighlightPriority::Medium));

        for &index in indices {
            let highlight = Highlight::new(color, priority, operation_type);
            self.highlights
                .entry(index)
                .or_insert_with(Vec::new)
                .push(highlight);
        }
    }

    /// Get the effective highlight for an index (highest priority wins)
    pub fn get_highlight(&self, index: usize) -> Option<&Highlight> {
        if let Some(highlights) = self.highlights.get(&index) {
            // Return the highest priority highlight
            highlights.iter().max_by_key(|h| h.priority)
        } else {
            None
        }
    }

    /// Get all highlighted indices
    pub fn get_highlighted_indices(&self) -> Vec<usize> {
        let mut indices: Vec<usize> = self.highlights.keys().copied().collect();
        indices.sort();
        indices
    }

    /// Get highlights for multiple indices at once
    pub fn get_highlights_for_indices(
        &self,
        indices: &[usize],
    ) -> HashMap<usize, Option<&Highlight>> {
        indices
            .iter()
            .map(|&index| (index, self.get_highlight(index)))
            .collect()
    }

    /// Check if an index has any highlights
    pub fn is_highlighted(&self, index: usize) -> bool {
        self.highlights.contains_key(&index) && !self.highlights[&index].is_empty()
    }

    /// Add multiple simultaneous highlights for different operations
    pub fn add_simultaneous_highlights(&mut self, operations: &[(String, Vec<usize>)]) {
        for (operation_type, indices) in operations {
            self.highlight_operation(operation_type, indices);
        }
    }

    /// Update color mapping for operation types
    pub fn set_color_mapping(
        &mut self,
        operation_type: &str,
        color: HighlightColor,
        priority: HighlightPriority,
    ) {
        self.color_map
            .insert(operation_type.to_string(), (color, priority));
    }

    /// Get color mapping for operation type
    pub fn get_color_mapping(
        &self,
        operation_type: &str,
    ) -> Option<(HighlightColor, HighlightPriority)> {
        self.color_map.get(operation_type).copied()
    }

    /// Get total number of highlighted positions
    pub fn highlight_count(&self) -> usize {
        self.highlights.len()
    }

    /// Get all unique operation types currently highlighted
    pub fn get_active_operation_types(&self) -> Vec<String> {
        let mut types = std::collections::HashSet::new();

        for highlights in self.highlights.values() {
            for highlight in highlights {
                types.insert(highlight.operation_type.clone());
            }
        }

        let mut result: Vec<String> = types.into_iter().collect();
        result.sort();
        result
    }

    /// Create highlights from telemetry data
    pub fn from_telemetry_highlights(
        &mut self,
        telemetry_highlights: &[usize],
        operation_type: &str,
    ) {
        self.highlight_operation(operation_type, telemetry_highlights);
    }
}

impl Default for HighlightSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Integration with Telemetry markers for operation highlighting
pub struct TelemetryHighlightIntegration {
    highlight_system: HighlightSystem,
}

impl TelemetryHighlightIntegration {
    pub fn new() -> Self {
        Self {
            highlight_system: HighlightSystem::new(),
        }
    }

    /// Update highlights based on telemetry markers
    pub fn update_from_telemetry(&mut self, telemetry: &sorting_race::models::traits::Telemetry) {
        // Clear previous highlights
        self.highlight_system.clear();

        // Add highlights from telemetry
        if !telemetry.highlights.is_empty() {
            // Determine operation type from context (this is simplified)
            let operation_type = if telemetry.status_text.contains("comparing") {
                "compare"
            } else if telemetry.status_text.contains("swapping") {
                "swap"
            } else {
                "operation"
            };

            self.highlight_system
                .from_telemetry_highlights(&telemetry.highlights, operation_type);
        }

        // Add markers-specific highlights
        if let Some(pivot) = telemetry.markers.pivot {
            self.highlight_system.highlight_operation("pivot", &[pivot]);
        }

        for &cursor in &telemetry.markers.cursors {
            self.highlight_system
                .highlight_operation("cursor", &[cursor]);
        }

        // Add merge runs
        for &(start, end) in &telemetry.markers.merge_runs {
            let indices: Vec<usize> = (start..=end).collect();
            self.highlight_system.highlight_operation("merge", &indices);
        }
    }

    pub fn get_highlight_system(&self) -> &HighlightSystem {
        &self.highlight_system
    }

    pub fn get_highlight_system_mut(&mut self) -> &mut HighlightSystem {
        &mut self.highlight_system
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sorting_race::models::traits::{Markers, Telemetry};

    #[test]
    fn test_highlight_indices_for_comparisons() {
        let mut system = HighlightSystem::new();

        // Test single comparison
        system.highlight_comparison(&[3, 7]);

        assert!(system.is_highlighted(3));
        assert!(system.is_highlighted(7));
        assert!(!system.is_highlighted(5));

        let highlight_3 = system.get_highlight(3).unwrap();
        assert_eq!(highlight_3.color, HighlightColor::Blue);
        assert_eq!(highlight_3.operation_type, "compare");
    }

    #[test]
    fn test_highlight_indices_for_swaps() {
        let mut system = HighlightSystem::new();

        system.highlight_swap(&[2, 8]);

        assert!(system.is_highlighted(2));
        assert!(system.is_highlighted(8));

        let highlight_2 = system.get_highlight(2).unwrap();
        assert_eq!(highlight_2.color, HighlightColor::Red);
        assert_eq!(highlight_2.operation_type, "swap");
        assert_eq!(highlight_2.priority, HighlightPriority::High);
    }

    #[test]
    fn test_multiple_simultaneous_highlights() {
        let mut system = HighlightSystem::new();

        // Add multiple operations simultaneously
        let operations = vec![
            ("compare".to_string(), vec![1, 2]),
            ("pivot".to_string(), vec![5]),
            ("sorted".to_string(), vec![8, 9, 10]),
        ];

        system.add_simultaneous_highlights(&operations);

        assert_eq!(system.highlight_count(), 6); // 2 + 1 + 3 unique indices

        // Check specific highlights
        let compare_highlight = system.get_highlight(1).unwrap();
        assert_eq!(compare_highlight.color, HighlightColor::Blue);

        let pivot_highlight = system.get_highlight(5).unwrap();
        assert_eq!(pivot_highlight.color, HighlightColor::Yellow);

        let sorted_highlight = system.get_highlight(8).unwrap();
        assert_eq!(sorted_highlight.color, HighlightColor::Green);
    }

    #[test]
    fn test_color_priorities_when_operations_overlap() {
        let mut system = HighlightSystem::new();

        // Add overlapping operations at index 3
        system.highlight_comparison(&[3]); // Medium priority, blue
        system.highlight_swap(&[3]); // High priority, red
        system.highlight_operation("sorted", &[3]); // Low priority, green

        // High priority swap should win
        let effective_highlight = system.get_highlight(3).unwrap();
        assert_eq!(effective_highlight.color, HighlightColor::Red);
        assert_eq!(effective_highlight.priority, HighlightPriority::High);
        assert_eq!(effective_highlight.operation_type, "swap");
    }

    #[test]
    fn test_critical_priority_overrides_all() {
        let mut system = HighlightSystem::new();

        // Add multiple operations including critical priority
        system.highlight_comparison(&[5]);
        system.highlight_swap(&[5]);
        system.highlight_operation("pivot", &[5]); // Critical priority

        let effective_highlight = system.get_highlight(5).unwrap();
        assert_eq!(effective_highlight.color, HighlightColor::Yellow);
        assert_eq!(effective_highlight.priority, HighlightPriority::Critical);
        assert_eq!(effective_highlight.operation_type, "pivot");
    }

    #[test]
    fn test_get_highlighted_indices() {
        let mut system = HighlightSystem::new();

        system.highlight_comparison(&[1, 5]);
        system.highlight_swap(&[3, 7]);
        system.highlight_operation("pivot", &[2]);

        let indices = system.get_highlighted_indices();
        assert_eq!(indices, vec![1, 2, 3, 5, 7]); // Should be sorted
    }

    #[test]
    fn test_clear_highlights() {
        let mut system = HighlightSystem::new();

        system.highlight_comparison(&[1, 2, 3]);
        system.highlight_swap(&[4, 5]);

        assert_eq!(system.highlight_count(), 5);

        system.clear();

        assert_eq!(system.highlight_count(), 0);
        assert!(!system.is_highlighted(1));
        assert!(!system.is_highlighted(4));
    }

    #[test]
    fn test_custom_color_mapping() {
        let mut system = HighlightSystem::new();

        // Add custom operation type
        system.set_color_mapping("custom_op", HighlightColor::Purple, HighlightPriority::High);

        system.highlight_operation("custom_op", &[10]);

        let highlight = system.get_highlight(10).unwrap();
        assert_eq!(highlight.color, HighlightColor::Purple);
        assert_eq!(highlight.priority, HighlightPriority::High);
        assert_eq!(highlight.operation_type, "custom_op");
    }

    #[test]
    fn test_get_highlights_for_multiple_indices() {
        let mut system = HighlightSystem::new();

        system.highlight_comparison(&[1, 3]);
        system.highlight_swap(&[2]);

        let query_indices = vec![1, 2, 3, 4];
        let results = system.get_highlights_for_indices(&query_indices);

        assert!(results[&1].is_some());
        assert!(results[&2].is_some());
        assert!(results[&3].is_some());
        assert!(results[&4].is_none());

        assert_eq!(results[&1].unwrap().color, HighlightColor::Blue);
        assert_eq!(results[&2].unwrap().color, HighlightColor::Red);
    }

    #[test]
    fn test_active_operation_types() {
        let mut system = HighlightSystem::new();

        system.highlight_comparison(&[1]);
        system.highlight_swap(&[2]);
        system.highlight_operation("pivot", &[3]);
        system.highlight_operation("pivot", &[4]); // Duplicate type

        let active_types = system.get_active_operation_types();
        assert_eq!(active_types, vec!["compare", "pivot", "swap"]); // Should be sorted and unique
    }

    #[test]
    fn test_integration_with_telemetry_markers() {
        let mut integration = TelemetryHighlightIntegration::new();

        let telemetry = Telemetry {
            total_comparisons: 10,
            total_moves: 5,
            memory_current: 1024,
            memory_peak: 2048,
            highlights: vec![2, 5, 8], // Telemetry highlights
            markers: Markers {
                pivot: Some(3),
                heap_boundary: None,
                merge_runs: vec![(6, 9)],
                cursors: vec![1, 7],
                gap: None,
            },
            status_text: "comparing elements".to_string(),
            progress_hint: 0.5,
        };

        integration.update_from_telemetry(&telemetry);
        let system = integration.get_highlight_system();

        // Should have highlights from telemetry
        assert!(system.is_highlighted(2));
        assert!(system.is_highlighted(5));
        assert!(system.is_highlighted(8));

        // Should have pivot highlight
        assert!(system.is_highlighted(3));
        let pivot_highlight = system.get_highlight(3).unwrap();
        assert_eq!(pivot_highlight.color, HighlightColor::Yellow);

        // Should have cursor highlights
        assert!(system.is_highlighted(1));
        assert!(system.is_highlighted(7));

        // Should have merge run highlights
        for i in 6..=9 {
            assert!(system.is_highlighted(i));
        }
    }

    #[test]
    fn test_telemetry_operation_detection() {
        let mut integration = TelemetryHighlightIntegration::new();

        // Test swap detection
        let swap_telemetry = Telemetry {
            total_comparisons: 5,
            total_moves: 3,
            memory_current: 512,
            memory_peak: 1024,
            highlights: vec![4, 7],
            markers: Markers::default(),
            status_text: "swapping elements at positions 4 and 7".to_string(),
            progress_hint: 0.3,
        };

        integration.update_from_telemetry(&swap_telemetry);
        let system = integration.get_highlight_system();

        let highlight_4 = system.get_highlight(4).unwrap();
        assert_eq!(highlight_4.color, HighlightColor::Red); // Should be red for swap
    }

    #[test]
    fn test_no_highlights_scenario() {
        let mut system = HighlightSystem::new();

        assert_eq!(system.highlight_count(), 0);
        assert!(system.get_highlighted_indices().is_empty());
        assert!(system.get_active_operation_types().is_empty());
        assert!(!system.is_highlighted(0));
        assert!(system.get_highlight(0).is_none());
    }

    #[test]
    fn test_same_index_multiple_operations_different_priority() {
        let mut system = HighlightSystem::new();

        // Add operations in order of increasing priority
        system.highlight_operation("sorted", &[5]); // Low
        system.highlight_comparison(&[5]); // Medium  
        system.highlight_swap(&[5]); // High

        // Should show the highest priority (swap)
        let highlight = system.get_highlight(5).unwrap();
        assert_eq!(highlight.priority, HighlightPriority::High);
        assert_eq!(highlight.operation_type, "swap");

        // Now add critical priority
        system.highlight_operation("pivot", &[5]); // Critical

        let highlight = system.get_highlight(5).unwrap();
        assert_eq!(highlight.priority, HighlightPriority::Critical);
        assert_eq!(highlight.operation_type, "pivot");
    }

    #[test]
    fn test_large_index_values() {
        let mut system = HighlightSystem::new();

        let large_indices = vec![1000, 5000, 10000];
        system.highlight_comparison(&large_indices);

        for &index in &large_indices {
            assert!(system.is_highlighted(index));
        }

        let highlighted = system.get_highlighted_indices();
        assert_eq!(highlighted, large_indices);
    }
}

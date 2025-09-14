//! Memory graph widget for visualizing memory usage per algorithm

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Widget},
};
use std::collections::HashMap;

/// Memory usage data for a single algorithm
#[derive(Debug, Clone)]
pub struct MemoryData {
    pub current: usize,
    pub peak: usize,
    pub history: Vec<usize>,
    pub has_started: bool,  // Track if algorithm has started processing
}

impl MemoryData {
    pub fn new() -> Self {
        Self {
            current: 0,
            peak: 0,
            history: Vec::new(),
            has_started: false,
        }
    }

    pub fn update(&mut self, current: usize, max_history: usize) {
        // Mark as started when we get first non-zero update
        if current > 0 && !self.has_started {
            self.has_started = true;
        }

        self.current = current;
        if current > self.peak {
            self.peak = current;
        }

        self.history.push(current);
        if self.history.len() > max_history {
            self.history.remove(0);
        }
    }

    pub fn reset(&mut self) {
        self.current = 0;
        self.peak = 0;
        self.history.clear();
        self.has_started = false;
    }
}

impl Default for MemoryData {
    fn default() -> Self {
        Self::new()
    }
}

/// Widget for displaying memory usage graphs
#[derive(Debug, Clone)]
pub struct MemoryGraph {
    data: HashMap<String, MemoryData>,
    max_history: usize,
    style: Style,
    current_style: Style,
    peak_style: Style,
    block: Option<Block<'static>>,
    title: Option<String>,
    show_values: bool,
}

impl MemoryGraph {
    /// Create a new memory graph
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            max_history: 100,
            style: Style::default(),
            current_style: Style::default().fg(Color::Green),
            peak_style: Style::default().fg(Color::Red),
            block: None,
            title: None,
            show_values: true,
        }
    }

    /// Set the maximum history length
    pub fn max_history(mut self, max: usize) -> Self {
        self.max_history = max.max(1);
        self
    }

    /// Set the base style
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Set the style for current memory values
    pub fn current_style(mut self, style: Style) -> Self {
        self.current_style = style;
        self
    }

    /// Set the style for peak memory values
    pub fn peak_style(mut self, style: Style) -> Self {
        self.peak_style = style;
        self
    }

    /// Set the block
    pub fn block(mut self, block: Block<'static>) -> Self {
        self.block = Some(block);
        self
    }

    /// Set the title
    pub fn title<T>(mut self, title: T) -> Self 
    where
        T: Into<String>,
    {
        self.title = Some(title.into());
        self
    }

    /// Set whether to show values
    pub fn show_values(mut self, show: bool) -> Self {
        self.show_values = show;
        self
    }

    /// Update memory usage for an algorithm
    pub fn update_algorithm(&mut self, name: &str, current_bytes: usize) {
        let entry = self.data.entry(name.to_string()).or_default();
        entry.update(current_bytes, self.max_history);
    }

    /// Get memory data for an algorithm
    pub fn get_algorithm_data(&self, name: &str) -> Option<&MemoryData> {
        self.data.get(name)
    }

    /// Get all algorithm names
    pub fn algorithm_names(&self) -> Vec<&String> {
        self.data.keys().collect()
    }

    /// Check if the memory graph has any data
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Clear all data
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Reset all algorithms' memory data (for race restart)
    pub fn reset_all(&mut self) {
        for data in self.data.values_mut() {
            data.reset();
        }
    }

    /// Format bytes into human readable string
    fn format_bytes(bytes: usize) -> String {
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

    /// Render the memory graph to a buffer
    pub fn render_widget(&self, area: Rect, buf: &mut Buffer) {
        if area.width < 3 || area.height < 3 {
            return;
        }

        let inner_area = if let Some(ref block) = self.block {
            let inner = block.inner(area);
            block.render(area, buf);
            inner
        } else {
            area
        };

        if self.data.is_empty() {
            // Show "no data" message in the middle of the area
            let msg = "No memory data available";
            let msg_x = inner_area.left() + (inner_area.width.saturating_sub(msg.len() as u16)) / 2;
            let msg_y = inner_area.top() + inner_area.height / 2;

            for (i, ch) in msg.chars().enumerate() {
                let x = msg_x + i as u16;
                if x < inner_area.right() && msg_y < inner_area.bottom() {
                    buf[(x, msg_y)]
                        .set_symbol(&ch.to_string())
                        .set_style(self.style);
                }
            }
            return;
        }

        let mut algorithms: Vec<_> = self.data.keys().cloned().collect();
        algorithms.sort(); // Sort for consistent display order
        let algorithm_count = algorithms.len();
        
        if algorithm_count == 0 {
            return;
        }

        let line_height = if inner_area.height > algorithm_count as u16 {
            inner_area.height / algorithm_count as u16
        } else {
            1
        };

        // Find maximum memory usage for scaling
        let max_memory = self.data.values()
            .map(|data| data.peak.max(data.current))
            .max()
            .unwrap_or(1);

        for (i, algorithm) in algorithms.iter().enumerate() {
            if let Some(memory_data) = self.data.get(algorithm) {
                let y_start = inner_area.top() + (i as u16 * line_height);
                let _y_end = (y_start + line_height).min(inner_area.bottom());

                // Render algorithm name
                let name_y = y_start;
                if name_y < inner_area.bottom() {
                    let display_name = if algorithm.len() > 10 {
                        &algorithm[..10]
                    } else {
                        algorithm
                    };
                    
                    for (char_idx, ch) in display_name.chars().enumerate() {
                        let char_x = inner_area.left() + char_idx as u16;
                        if char_x < inner_area.right() {
                            buf[(char_x, name_y)]
                                .set_symbol(&ch.to_string())
                                .set_style(self.style);
                        }
                    }
                }

                // Render memory usage bars
                if line_height > 1 && y_start + 1 < inner_area.bottom() {
                    let bar_y = y_start + 1;
                    let available_width = inner_area.width.saturating_sub(15); // Leave space for values
                    
                    // Current memory bar - only show if algorithm has started
                    if max_memory > 0 && memory_data.has_started {
                        let current_width = if memory_data.current > 0 {
                            // Ensure at least 1 character width for any non-zero memory
                            ((memory_data.current as f64 / max_memory as f64) * available_width as f64).max(1.0) as u16
                        } else {
                            0
                        };
                        for x in 0..current_width {
                            let char_x = inner_area.left() + x;
                            if char_x < inner_area.right() && bar_y < inner_area.bottom() {
                                buf[(char_x, bar_y)]
                                    .set_symbol("█")
                                    .set_style(self.current_style);
                            }
                        }

                        // Peak memory indicator (show as different character)
                        let peak_width = ((memory_data.peak as f64 / max_memory as f64) * available_width as f64) as u16;
                        if peak_width > current_width {
                            for x in current_width..peak_width {
                                let char_x = inner_area.left() + x;
                                if char_x < inner_area.right() && bar_y < inner_area.bottom() {
                                    buf[(char_x, bar_y)]
                                        .set_symbol("░")
                                        .set_style(self.peak_style);
                                }
                            }
                        }
                    }

                    // Show values if enabled and algorithm has started
                    if self.show_values && memory_data.has_started {
                        let values_x = inner_area.right().saturating_sub(14);
                        if values_x > inner_area.left() {
                            let current_str = Self::format_bytes(memory_data.current);
                            let peak_str = Self::format_bytes(memory_data.peak);
                            let value_text = format!("{}/{}", current_str, peak_str);

                            for (char_idx, ch) in value_text.chars().enumerate() {
                                let char_x = values_x + char_idx as u16;
                                if char_x < inner_area.right() && bar_y < inner_area.bottom() {
                                    buf[(char_x, bar_y)]
                                        .set_symbol(&ch.to_string())
                                        .set_style(self.style);
                                }
                            }
                        }
                    } else if self.show_values && !memory_data.has_started {
                        // Show "-" for algorithms that haven't started
                        let values_x = inner_area.right().saturating_sub(14);
                        if values_x > inner_area.left() && bar_y < inner_area.bottom() {
                            buf[(values_x, bar_y)]
                                .set_symbol("-")
                                .set_style(self.style);
                        }
                    }
                }
            }
        }
    }
}

impl Default for MemoryGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for MemoryGraph {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.render_widget(area, buf);
    }
}

impl Widget for &MemoryGraph {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.render_widget(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{
        buffer::Buffer,
        layout::Rect,
    };

    #[test]
    fn test_memory_graph_creation() {
        let graph = MemoryGraph::new();
        assert_eq!(graph.data.len(), 0);
        assert_eq!(graph.max_history, 100);
    }

    #[test]
    fn test_update_algorithm() {
        let mut graph = MemoryGraph::new();
        graph.update_algorithm("QuickSort", 1024);
        graph.update_algorithm("MergeSort", 2048);

        assert_eq!(graph.data.len(), 2);
        
        let quick_data = graph.get_algorithm_data("QuickSort").unwrap();
        assert_eq!(quick_data.current, 1024);
        assert_eq!(quick_data.peak, 1024);
        
        let merge_data = graph.get_algorithm_data("MergeSort").unwrap();
        assert_eq!(merge_data.current, 2048);
        assert_eq!(merge_data.peak, 2048);
    }

    #[test]
    fn test_peak_memory_tracking() {
        let mut graph = MemoryGraph::new();
        graph.update_algorithm("TestSort", 1024);
        graph.update_algorithm("TestSort", 2048);
        graph.update_algorithm("TestSort", 1536);

        let data = graph.get_algorithm_data("TestSort").unwrap();
        assert_eq!(data.current, 1536);
        assert_eq!(data.peak, 2048);
        assert_eq!(data.history.len(), 3);
    }

    #[test]
    fn test_history_limit() {
        let mut graph = MemoryGraph::new().max_history(3);
        for i in 1..=5 {
            graph.update_algorithm("TestSort", i * 100);
        }

        let data = graph.get_algorithm_data("TestSort").unwrap();
        assert_eq!(data.history.len(), 3);
        assert_eq!(data.history, vec![300, 400, 500]);
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(MemoryGraph::format_bytes(512), "512B");
        assert_eq!(MemoryGraph::format_bytes(1024), "1.0KB");
        assert_eq!(MemoryGraph::format_bytes(1536), "1.5KB");
        assert_eq!(MemoryGraph::format_bytes(1048576), "1.0MB");
        assert_eq!(MemoryGraph::format_bytes(1073741824), "1.0GB");
    }

    #[test]
    fn test_render_widget() {
        let mut graph = MemoryGraph::new();
        graph.update_algorithm("QuickSort", 1024);
        graph.update_algorithm("MergeSort", 2048);

        let area = Rect::new(0, 0, 50, 10);
        let mut buffer = Buffer::empty(area);

        graph.render_widget(area, &mut buffer);
        
        // Should not panic and should have some content
        let content = buffer.content();
        assert!(!content.is_empty());
    }

    #[test]
    fn test_clear() {
        let mut graph = MemoryGraph::new();
        graph.update_algorithm("TestSort", 1024);
        assert_eq!(graph.data.len(), 1);

        graph.clear();
        assert_eq!(graph.data.len(), 0);
    }
}
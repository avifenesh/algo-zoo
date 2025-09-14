//! BarChart widget for visualizing array data as vertical bars

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Widget},
};

/// BarChart widget for rendering arrays as vertical bars
#[derive(Debug, Clone)]
pub struct BarChart {
    data: Vec<(String, u64)>,
    bar_width: u16,
    bar_gap: u16,
    bar_style: Style,
    value_style: Style,
    label_style: Style,
    max_height: u16,
    title: Option<String>,
    block: Option<Block<'static>>,
    highlight_indices: Vec<usize>,
    highlight_style: Style,
}

impl BarChart {
    /// Create a new BarChart from array data
    pub fn new(data: Vec<(String, u64)>) -> Self {
        Self {
            data,
            bar_width: 3,
            bar_gap: 1,
            bar_style: Style::default(),
            value_style: Style::default(),
            label_style: Style::default(),
            max_height: 10,
            title: None,
            block: None,
            highlight_indices: Vec::new(),
            highlight_style: Style::default().fg(Color::Yellow),
        }
    }

    /// Set the style for bars
    pub fn bar_style(mut self, style: Style) -> Self {
        self.bar_style = style;
        self
    }

    /// Set the style for values
    pub fn value_style(mut self, style: Style) -> Self {
        self.value_style = style;
        self
    }

    /// Set the style for labels
    pub fn label_style(mut self, style: Style) -> Self {
        self.label_style = style;
        self
    }

    /// Set the maximum height for bars
    pub fn max_height(mut self, height: u16) -> Self {
        self.max_height = height;
        self
    }

    /// Set bar width
    pub fn bar_width(mut self, width: u16) -> Self {
        self.bar_width = width.max(1);
        self
    }

    /// Set gap between bars
    pub fn bar_gap(mut self, gap: u16) -> Self {
        self.bar_gap = gap;
        self
    }

    /// Set title
    pub fn title<T>(mut self, title: T) -> Self 
    where
        T: Into<String>,
    {
        self.title = Some(title.into());
        self
    }

    /// Set block
    pub fn block(mut self, block: Block<'static>) -> Self {
        self.block = Some(block);
        self
    }

    /// Set indices to highlight
    pub fn highlight_indices(mut self, indices: Vec<usize>) -> Self {
        self.highlight_indices = indices;
        self
    }

    /// Set style for highlighted bars
    pub fn highlight_style(mut self, style: Style) -> Self {
        self.highlight_style = style;
        self
    }

    /// Convert array data to bar chart data with color mapping
    pub fn from_array_with_colors(array: &[i32], highlights: &[usize]) -> Self {
        let data: Vec<(String, u64)> = array
            .iter()
            .map(|&value| (value.to_string(), value.max(0) as u64))
            .collect();

        let mut chart = Self::new(data);
        chart.highlight_indices = highlights.to_vec();

        // Set colors based on operation type (compare vs swap)
        if highlights.len() == 2 {
            // Two highlights indicate a comparison
            chart = chart.bar_style(Style::default().fg(Color::Blue));
        } else if highlights.len() == 1 {
            // Single highlight indicates a swap
            chart = chart.bar_style(Style::default().fg(Color::Red));
        }

        chart
    }

    /// Create a compact visualization using blocks instead of numbers for very large arrays
    pub fn from_array_compact(array: &[i32], highlights: &[usize], terminal_width: u16) -> (Self, String) {
        let array_len = array.len();
        let available_width = terminal_width.saturating_sub(4) as usize;

        // Sample the array if needed
        let (sampled_data, sample_rate) = if array_len <= available_width {
            // Can show all elements
            (array.to_vec(), 1)
        } else {
            // Need to sample
            let sample_rate = array_len.div_ceil(available_width);
            let sampled: Vec<i32> = (0..available_width)
                .map(|i| {
                    let idx = (i * sample_rate).min(array_len - 1);
                    array[idx]
                })
                .collect();
            (sampled, sample_rate)
        };

        // Find min/max for normalization
        let min_val = *sampled_data.iter().min().unwrap_or(&0);
        let max_val = *sampled_data.iter().max().unwrap_or(&1);
        let range = (max_val - min_val).max(1) as f64;

        // Create bar chart data using block characters
        let data: Vec<(String, u64)> = sampled_data
            .iter()
            .map(|&value| {
                // Use block characters to show value intensity
                let normalized = ((value - min_val) as f64 / range * 8.0) as usize;
                let block_char = match normalized {
                    0 => " ",
                    1 => "▁",
                    2 => "▂",
                    3 => "▃",
                    4 => "▄",
                    5 => "▅",
                    6 => "▆",
                    7 => "▇",
                    _ => "█",
                };
                (block_char.to_string(), value.max(0) as u64)
            })
            .collect();

        let mut chart = Self::new(data);

        // Adjust highlights for sampling
        if sample_rate > 1 {
            chart.highlight_indices = highlights
                .iter()
                .map(|&idx| idx / sample_rate)
                .filter(|&idx| idx < available_width)
                .collect();
        } else {
            chart.highlight_indices = highlights.to_vec();
        }

        let indicator = if sample_rate > 1 {
            format!("[Compact view: 1:{} sampling of {} elements]", sample_rate, array_len)
        } else {
            format!("[Compact view: {} elements]", array_len)
        };

        (chart, indicator)
    }

    /// Create a viewport view of large arrays
    pub fn from_array_with_viewport(
        array: &[i32],
        highlights: &[usize],
        terminal_width: u16,
        viewport_center: Option<usize>
    ) -> (Self, String) {
        let array_len = array.len();

        // For very large arrays, use compact mode
        if array_len > 500 {
            return Self::from_array_compact(array, highlights, terminal_width);
        }

        // Calculate how many elements can fit
        let max_elements = (terminal_width / 4).min(100) as usize; // At least 4 chars per element

        // Determine viewport range
        let array_len = array.len();
        let (start, end) = if array_len <= max_elements {
            // Array fits entirely
            (0, array_len)
        } else {
            // Need viewport
            let center = viewport_center
                .or_else(|| highlights.first().copied())  // Center on first highlight
                .unwrap_or(array_len / 2);  // Or center of array

            let half_view = max_elements / 2;
            let start = center.saturating_sub(half_view);
            let end = (start + max_elements).min(array_len);

            // Adjust if we hit the end
            let start = if end == array_len {
                array_len.saturating_sub(max_elements)
            } else {
                start
            };

            (start, end)
        };

        // Create data for visible portion
        let visible_data: Vec<(String, u64)> = array[start..end]
            .iter()
            .map(|&value| (value.to_string(), value.max(0) as u64))
            .collect();

        let mut chart = Self::new(visible_data);

        // Adjust highlight indices to viewport
        chart.highlight_indices = highlights
            .iter()
            .filter_map(|&idx| {
                if idx >= start && idx < end {
                    Some(idx - start)
                } else {
                    None
                }
            })
            .collect();

        // Set colors based on operation type
        if highlights.len() == 2 {
            chart = chart.bar_style(Style::default().fg(Color::Blue));
        } else if highlights.len() == 1 {
            chart = chart.bar_style(Style::default().fg(Color::Red));
        }

        // Create viewport indicator
        let indicator = if array_len > max_elements {
            format!("[Showing {}-{} of {}]", start + 1, end, array_len)
        } else {
            String::new()
        };

        (chart, indicator)
    }

    /// Scale the chart for different terminal sizes
    pub fn scale_for_terminal(mut self, terminal_width: u16, terminal_height: u16) -> Self {
        // Calculate appropriate bar width and max height based on terminal size
        let available_width = terminal_width.saturating_sub(4); // Leave margins
        let bar_count = self.data.len() as u16;

        if bar_count > 0 {
            // Find the maximum number of digits needed for any value
            let max_digits = self.data.iter()
                .map(|(_, value)| value.to_string().len() as u16)
                .max()
                .unwrap_or(1);

            // Calculate minimum space needed per element (label + at least 1 space)
            let min_label_space = max_digits + 1; // Number digits + separator space
            let min_bar_width = max_digits.max(2); // At least 2 for the bar visual

            // Check if we have enough space for readable labels
            let total_label_space_needed = bar_count * min_label_space;
            if total_label_space_needed > available_width {
                // Terminal too narrow for readable labels - fall back to compressed mode
                let space_per_element = (available_width / bar_count).max(1);
                self.bar_width = (space_per_element.saturating_sub(1)).max(1);
                self.bar_gap = if space_per_element > 1 { 1 } else { 0 };
            } else {
                // We have space for readable labels - prioritize label space
                let remaining_space = available_width - total_label_space_needed;
                let extra_bar_width = remaining_space / bar_count;

                self.bar_width = min_bar_width + extra_bar_width;
                self.bar_gap = 1; // Always have at least 1 space between elements
            }
        }

        // Scale height to use available terminal space
        let available_height = terminal_height.saturating_sub(6); // Leave space for headers/labels
        self.max_height = available_height.min(20); // Cap at reasonable height

        self
    }

    /// Render the bar chart to a ratatui buffer
    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        if area.width < 3 || area.height < 3 {
            return; // Not enough space to render anything meaningful
        }

        let inner_area = if let Some(ref block) = self.block {
            let inner = block.inner(area);
            block.render(area, buf);
            inner
        } else {
            area
        };

        if self.data.is_empty() {
            return;
        }

        // Find max value for scaling
        let max_value = self.data.iter().map(|(_, value)| *value).max().unwrap_or(1);
        if max_value == 0 {
            return;
        }

        // Calculate available space for bars
        let available_height = inner_area.height.saturating_sub(2); // Leave space for labels
        let bar_height_scale = self.max_height.min(available_height) as f64;

        let mut x_offset = inner_area.left();
        
        for (i, (label, value)) in self.data.iter().enumerate() {
            if x_offset + self.bar_width >= inner_area.right() {
                break; // No more space
            }

            // Calculate bar height
            let bar_height = if max_value > 0 {
                (((*value as f64) / (max_value as f64)) * bar_height_scale).ceil() as u16
            } else {
                0
            };

            // Determine bar style
            let current_bar_style = if self.highlight_indices.contains(&i) {
                self.highlight_style
            } else {
                self.bar_style
            };

            // Render bar
            for y in 0..bar_height {
                let bar_y = inner_area.bottom().saturating_sub(2 + y);
                if bar_y >= inner_area.top() && bar_y < inner_area.bottom() {
                    for x in x_offset..x_offset + self.bar_width {
                        if x < inner_area.right() {
                            buf[(x, bar_y)]
                                .set_symbol("█")
                                .set_style(current_bar_style);
                        }
                    }
                }
            }

            // Render value on top of bar (if space allows)
            if bar_height > 0 {
                let value_y = inner_area.bottom().saturating_sub(2 + bar_height);
                if value_y > inner_area.top() {
                    let value_str = value.to_string();

                    // If value is too long for bar width, truncate or adjust positioning
                    let display_str = if value_str.len() as u16 > self.bar_width {
                        // For very long numbers, just show the full number starting at x_offset
                        value_str
                    } else {
                        value_str
                    };

                    let value_x = if display_str.len() as u16 <= self.bar_width {
                        // Center the value within the bar
                        x_offset + (self.bar_width.saturating_sub(display_str.len() as u16)) / 2
                    } else {
                        // Start at bar position, but allow overflow to the right
                        x_offset
                    };

                    for (char_idx, ch) in display_str.chars().enumerate() {
                        let char_x = value_x + char_idx as u16;
                        if char_x < inner_area.right() && value_y >= inner_area.top() {
                            buf[(char_x, value_y.saturating_sub(1))]
                                .set_symbol(&ch.to_string())
                                .set_style(self.value_style);
                        }
                    }
                }
            }

            // Render label at bottom with improved spacing
            let label_y = inner_area.bottom().saturating_sub(1);
            if label_y >= inner_area.top() && label_y < inner_area.bottom() {
                let label_display = label.as_str();

                // Calculate label position with better spacing logic
                let label_x = if label_display.len() as u16 <= self.bar_width {
                    // Center within bar if it fits
                    x_offset + (self.bar_width.saturating_sub(label_display.len() as u16)) / 2
                } else {
                    // Start at bar position but allow overflow
                    x_offset
                };

                // Render the label with a trailing space for separation
                for (char_idx, ch) in label_display.chars().enumerate() {
                    let char_x = label_x + char_idx as u16;
                    if char_x < inner_area.right() {
                        buf[(char_x, label_y)]
                            .set_symbol(&ch.to_string())
                            .set_style(self.label_style);
                    }
                }

                // Always add a space separator after the label if there's room
                // This ensures multi-digit numbers don't run together
                let space_x = label_x + label_display.len() as u16;
                if space_x < inner_area.right() {
                    buf[(space_x, label_y)]
                        .set_symbol(" ")
                        .set_style(self.label_style);
                }
            }

            // Advance x_offset by the maximum of bar width or label width to prevent overlap
            let label_width = label.len() as u16 + 1; // Label plus space
            let min_advance = label_width.max(self.bar_width + self.bar_gap);
            x_offset += min_advance;
        }
    }

    /// Get the data
    pub fn data(&self) -> &[(String, u64)] {
        &self.data
    }
}

impl Widget for BarChart {
    fn render(self, area: Rect, buf: &mut Buffer) {
        BarChart::render(&self, area, buf);
    }
}

// Also implement Widget for reference to allow both owned and borrowed usage
impl Widget for &BarChart {
    fn render(self, area: Rect, buf: &mut Buffer) {
        BarChart::render(self, area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{
        buffer::Buffer,
        layout::Rect,
        style::Color,
    };

    #[test]
    fn test_bar_chart_creation_from_array_data() {
        let array_data = vec![5, 3, 8, 1, 9, 2];
        let highlights = vec![];
        
        let chart = BarChart::from_array_with_colors(&array_data, &highlights);
        
        assert_eq!(chart.data.len(), 6);
        assert_eq!(chart.data[0], ("5".to_string(), 5));
        assert_eq!(chart.data[1], ("3".to_string(), 3));
        assert_eq!(chart.data[4], ("9".to_string(), 9));
    }

    #[test]
    fn test_color_mapping_for_operations() {
        let array_data = vec![5, 3, 8, 1];
        
        // Test comparison operation (two highlights = blue)
        let comparison_highlights = vec![0, 2];
        let comparison_chart = BarChart::from_array_with_colors(&array_data, &comparison_highlights);
        assert_eq!(comparison_chart.bar_style.fg, Some(Color::Blue));
        
        // Test swap operation (one highlight = red)
        let swap_highlights = vec![1];
        let swap_chart = BarChart::from_array_with_colors(&array_data, &swap_highlights);
        assert_eq!(swap_chart.bar_style.fg, Some(Color::Red));
        
        // Test no operation (no highlights = default)
        let no_highlights = vec![];
        let default_chart = BarChart::from_array_with_colors(&array_data, &no_highlights);
        assert_eq!(default_chart.bar_style.fg, None);
    }

    #[test]
    fn test_height_scaling_for_different_terminal_sizes() {
        let array_data = vec![1, 2, 3, 4, 5];
        let highlights = vec![];
        
        // Test small terminal
        let chart_small = BarChart::from_array_with_colors(&array_data, &highlights)
            .scale_for_terminal(40, 10);
        assert!(chart_small.max_height <= 4);
        
        // Test large terminal
        let chart_large = BarChart::from_array_with_colors(&array_data, &highlights)
            .scale_for_terminal(120, 30);
        assert!(chart_large.max_height >= 10);
        assert!(chart_large.max_height <= 20);
    }

    #[test]
    fn test_bar_width_scaling_for_many_elements() {
        let array_data: Vec<i32> = (0..50).collect();
        let highlights = vec![];
        
        let chart = BarChart::from_array_with_colors(&array_data, &highlights)
            .scale_for_terminal(60, 20);
        
        assert!(chart.bar_width <= 2);
        
        let total_width = array_data.len() as u16 * (chart.bar_width + chart.bar_gap);
        assert!(total_width <= 56);
    }

    #[test]
    fn test_rendering_to_ratatui_buffer() {
        let array_data = vec![5, 3, 8, 1];
        let highlights = vec![0, 2];
        let chart = BarChart::from_array_with_colors(&array_data, &highlights);
        
        let area = Rect::new(0, 0, 40, 10);
        let mut buffer = Buffer::empty(area);
        
        // Should not panic
        chart.render(area, &mut buffer);
        
        // Buffer should have some content
        let content = buffer.content();
        assert!(!content.is_empty());
    }

    #[test]
    fn test_builder_pattern_methods() {
        let data = vec![("A".to_string(), 10), ("B".to_string(), 20)];
        let chart = BarChart::new(data)
            .bar_width(5)
            .bar_gap(2)
            .max_height(15)
            .bar_style(Style::default().fg(Color::Green))
            .value_style(Style::default().fg(Color::Yellow))
            .label_style(Style::default().fg(Color::Cyan));

        assert_eq!(chart.bar_width, 5);
        assert_eq!(chart.bar_gap, 2);
        assert_eq!(chart.max_height, 15);
        assert_eq!(chart.bar_style.fg, Some(Color::Green));
        assert_eq!(chart.value_style.fg, Some(Color::Yellow));
        assert_eq!(chart.label_style.fg, Some(Color::Cyan));
    }

    #[test]
    fn test_empty_data_handling() {
        let empty_data = vec![];
        let highlights = vec![];
        let chart = BarChart::from_array_with_colors(&empty_data, &highlights);
        
        assert_eq!(chart.data.len(), 0);
        
        let scaled_chart = chart.scale_for_terminal(80, 20);
        assert!(scaled_chart.bar_width >= 1);
    }

    #[test]
    fn test_negative_values_handling() {
        let array_data = vec![-5, 3, -1, 8];
        let highlights = vec![];
        let chart = BarChart::from_array_with_colors(&array_data, &highlights);
        
        // Negative values should show the value as label but 0 height
        assert_eq!(chart.data[0], ("-5".to_string(), 0));
        assert_eq!(chart.data[1], ("3".to_string(), 3));
        assert_eq!(chart.data[2], ("-1".to_string(), 0));
        assert_eq!(chart.data[3], ("8".to_string(), 8));
    }
}
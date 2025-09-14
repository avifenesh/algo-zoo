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
            .enumerate()
            .map(|(i, &value)| (i.to_string(), value.max(0) as u64))
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

    /// Scale the chart for different terminal sizes
    pub fn scale_for_terminal(mut self, terminal_width: u16, terminal_height: u16) -> Self {
        // Calculate appropriate bar width and max height based on terminal size
        let available_width = terminal_width.saturating_sub(4); // Leave margins
        let bar_count = self.data.len() as u16;
        
        if bar_count > 0 {
            let total_width_needed = bar_count * (self.bar_width + self.bar_gap);
            if total_width_needed > available_width {
                // Scale down bar width and gap
                let new_bar_width = (available_width / bar_count).saturating_sub(1).max(1);
                self.bar_width = new_bar_width;
                self.bar_gap = if new_bar_width > 1 { 1 } else { 0 };
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
                    let value_x = x_offset + (self.bar_width.saturating_sub(value_str.len() as u16)) / 2;
                    
                    for (char_idx, ch) in value_str.chars().enumerate() {
                        let char_x = value_x + char_idx as u16;
                        if char_x < inner_area.right() && value_y >= inner_area.top() {
                            buf[(char_x, value_y.saturating_sub(1))]
                                .set_symbol(&ch.to_string())
                                .set_style(self.value_style);
                        }
                    }
                }
            }

            // Render label at bottom
            let label_y = inner_area.bottom().saturating_sub(1);
            if label_y >= inner_area.top() && label_y < inner_area.bottom() {
                let label_display = if label.len() > self.bar_width as usize {
                    &label[..self.bar_width as usize]
                } else {
                    label
                };
                
                let label_x = x_offset + (self.bar_width.saturating_sub(label_display.len() as u16)) / 2;
                
                for (char_idx, ch) in label_display.chars().enumerate() {
                    let char_x = label_x + char_idx as u16;
                    if char_x < inner_area.right() {
                        buf[(char_x, label_y)]
                            .set_symbol(&ch.to_string())
                            .set_style(self.label_style);
                    }
                }
            }

            x_offset += self.bar_width + self.bar_gap;
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
        assert_eq!(chart.data[0], ("0".to_string(), 5));
        assert_eq!(chart.data[1], ("1".to_string(), 3));
        assert_eq!(chart.data[4], ("4".to_string(), 9));
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
        
        // Negative values should be converted to 0
        assert_eq!(chart.data[0], ("0".to_string(), 0));
        assert_eq!(chart.data[1], ("1".to_string(), 3));
        assert_eq!(chart.data[2], ("2".to_string(), 0));
        assert_eq!(chart.data[3], ("3".to_string(), 8));
    }
}
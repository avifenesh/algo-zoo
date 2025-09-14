//! Progress bar widgets for visualizing algorithm completion

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Widget},
};

/// A single progress bar
#[derive(Debug, Clone)]
pub struct ProgressBar {
    progress: f32,
    label: String,
    style: Style,
    filled_style: Style,
    empty_style: Style,
    block: Option<Block<'static>>,
    show_percentage: bool,
    show_label: bool,
}

impl ProgressBar {
    /// Create a new progress bar
    pub fn new() -> Self {
        Self {
            progress: 0.0,
            label: String::new(),
            style: Style::default(),
            filled_style: Style::default().fg(Color::Green),
            empty_style: Style::default().fg(Color::Gray),
            block: None,
            show_percentage: true,
            show_label: true,
        }
    }

    /// Set the progress (0.0 to 1.0)
    pub fn progress(mut self, progress: f32) -> Self {
        self.progress = progress.clamp(0.0, 1.0);
        self
    }

    /// Set the label
    pub fn label<T>(mut self, label: T) -> Self 
    where
        T: Into<String>,
    {
        self.label = label.into();
        self
    }

    /// Set the base style
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Set the style for filled portion
    pub fn filled_style(mut self, style: Style) -> Self {
        self.filled_style = style;
        self
    }

    /// Set the style for empty portion
    pub fn empty_style(mut self, style: Style) -> Self {
        self.empty_style = style;
        self
    }

    /// Set the block
    pub fn block(mut self, block: Block<'static>) -> Self {
        self.block = Some(block);
        self
    }

    /// Set whether to show percentage
    pub fn show_percentage(mut self, show: bool) -> Self {
        self.show_percentage = show;
        self
    }

    /// Set whether to show label
    pub fn show_label(mut self, show: bool) -> Self {
        self.show_label = show;
        self
    }

    /// Render the progress bar to a buffer
    pub fn render_widget(&self, area: Rect, buf: &mut Buffer) {
        if area.width < 3 || area.height < 1 {
            return;
        }

        let inner_area = if let Some(ref block) = self.block {
            let inner = block.inner(area);
            block.render(area, buf);
            inner
        } else {
            area
        };

        if inner_area.width == 0 || inner_area.height == 0 {
            return;
        }

        // Calculate space for progress bar
        let mut bar_area = inner_area;
        let mut info_area = None;

        // Reserve space for label and percentage if needed
        if (self.show_label && !self.label.is_empty()) || self.show_percentage {
            if inner_area.height >= 2 {
                bar_area.height = inner_area.height - 1;
                info_area = Some(Rect {
                    x: inner_area.x,
                    y: inner_area.y + bar_area.height,
                    width: inner_area.width,
                    height: 1,
                });
            }
        }

        // Render the progress bar
        let filled_width = (self.progress * bar_area.width as f32) as u16;
        let _empty_width = bar_area.width - filled_width;

        // Fill the filled portion
        for y in bar_area.top()..bar_area.bottom() {
            for x in bar_area.left()..(bar_area.left() + filled_width) {
                if x < bar_area.right() {
                    buf[(x, y)]
                        .set_symbol("█")
                        .set_style(self.filled_style);
                }
            }
            
            // Fill the empty portion
            for x in (bar_area.left() + filled_width)..bar_area.right() {
                buf[(x, y)]
                    .set_symbol("░")
                    .set_style(self.empty_style);
            }
        }

        // Render label and percentage
        if let Some(info_area) = info_area {
            let mut info_text = String::new();
            
            if self.show_label && !self.label.is_empty() {
                info_text.push_str(&self.label);
            }
            
            if self.show_percentage {
                if !info_text.is_empty() {
                    info_text.push_str(" ");
                }
                info_text.push_str(&format!("{:.1}%", self.progress * 100.0));
            }

            // Center the info text
            let info_chars: Vec<char> = info_text.chars().collect();
            let start_x = if info_area.width > info_chars.len() as u16 {
                info_area.left() + (info_area.width - info_chars.len() as u16) / 2
            } else {
                info_area.left()
            };

            for (i, ch) in info_chars.iter().enumerate() {
                let x = start_x + i as u16;
                if x < info_area.right() {
                    buf[(x, info_area.top())]
                        .set_symbol(&ch.to_string())
                        .set_style(self.style);
                }
            }
        }
    }
}

impl Default for ProgressBar {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for ProgressBar {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.render_widget(area, buf);
    }
}

impl Widget for &ProgressBar {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.render_widget(area, buf);
    }
}

/// Collection of progress bars for multiple algorithms
#[derive(Debug, Clone)]
pub struct ProgressBars {
    bars: Vec<(String, ProgressBar)>,
    style: Style,
    block: Option<Block<'static>>,
}

impl ProgressBars {
    /// Create a new progress bars collection
    pub fn new() -> Self {
        Self {
            bars: Vec::new(),
            style: Style::default(),
            block: None,
        }
    }

    /// Add or update a progress bar
    pub fn add_bar<T>(&mut self, name: T, progress: f32) 
    where
        T: Into<String>,
    {
        let name_string = name.into();
        
        // Check if bar already exists
        for (existing_name, bar) in &mut self.bars {
            if *existing_name == name_string {
                *bar = ProgressBar::new()
                    .label(&name_string)
                    .progress(progress);
                return;
            }
        }
        
        // Add new bar
        let bar = ProgressBar::new()
            .label(&name_string)
            .progress(progress);
        self.bars.push((name_string, bar));
    }

    /// Set the base style
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Set the block
    pub fn block(mut self, block: Block<'static>) -> Self {
        self.block = Some(block);
        self
    }

    /// Clear all bars
    pub fn clear(&mut self) {
        self.bars.clear();
    }

    /// Get number of bars
    pub fn len(&self) -> usize {
        self.bars.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.bars.is_empty()
    }

    /// Render the progress bars to a buffer
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

        if self.bars.is_empty() || inner_area.height == 0 {
            return;
        }

        let bar_height = if inner_area.height >= self.bars.len() as u16 * 2 {
            2 // Space for bar and info
        } else {
            1 // Just the bar
        };

        let total_height_needed = self.bars.len() as u16 * bar_height;
        let start_y = if total_height_needed <= inner_area.height {
            inner_area.top()
        } else {
            inner_area.top()
        };

        for (i, (_name, bar)) in self.bars.iter().enumerate() {
            let y = start_y + (i as u16 * bar_height);
            
            if y >= inner_area.bottom() {
                break;
            }

            let bar_area = Rect {
                x: inner_area.x,
                y,
                width: inner_area.width,
                height: bar_height.min(inner_area.bottom() - y),
            };

            bar.render_widget(bar_area, buf);
        }
    }
}

impl Default for ProgressBars {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for ProgressBars {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.render_widget(area, buf);
    }
}

impl Widget for &ProgressBars {
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
    fn test_progress_bar_creation() {
        let bar = ProgressBar::new()
            .progress(0.5)
            .label("Test")
            .show_percentage(true);
        
        assert!((bar.progress - 0.5).abs() < f32::EPSILON);
        assert_eq!(bar.label, "Test");
        assert!(bar.show_percentage);
    }

    #[test]
    fn test_progress_clamping() {
        let bar1 = ProgressBar::new().progress(-0.5);
        assert!((bar1.progress - 0.0).abs() < f32::EPSILON);

        let bar2 = ProgressBar::new().progress(1.5);
        assert!((bar2.progress - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_progress_bars_collection() {
        let mut bars = ProgressBars::new();
        bars.add_bar("Algorithm1", 0.3);
        bars.add_bar("Algorithm2", 0.7);
        
        assert_eq!(bars.len(), 2);
        assert!(!bars.is_empty());

        // Update existing bar
        bars.add_bar("Algorithm1", 0.5);
        assert_eq!(bars.len(), 2); // Should still be 2
    }

    #[test]
    fn test_render_widget() {
        let bar = ProgressBar::new()
            .progress(0.5)
            .label("Test Bar")
            .show_percentage(true);

        let area = Rect::new(0, 0, 20, 3);
        let mut buffer = Buffer::empty(area);

        bar.render_widget(area, &mut buffer);
        
        // Should not panic and should have some content
        let content = buffer.content();
        assert!(!content.is_empty());
    }

    #[test]
    fn test_render_progress_bars_collection() {
        let mut bars = ProgressBars::new();
        bars.add_bar("Quick Sort", 0.8);
        bars.add_bar("Merge Sort", 0.4);
        bars.add_bar("Bubble Sort", 0.2);

        let area = Rect::new(0, 0, 30, 10);
        let mut buffer = Buffer::empty(area);

        bars.render_widget(area, &mut buffer);
        
        // Should not panic and should have some content
        let content = buffer.content();
        assert!(!content.is_empty());
    }

    #[test]
    fn test_clear_bars() {
        let mut bars = ProgressBars::new();
        bars.add_bar("Test", 0.5);
        assert_eq!(bars.len(), 1);

        bars.clear();
        assert_eq!(bars.len(), 0);
        assert!(bars.is_empty());
    }

    #[test]
    fn test_small_area_handling() {
        let bar = ProgressBar::new().progress(0.5);

        // Test very small area
        let tiny_area = Rect::new(0, 0, 1, 1);
        let mut buffer = Buffer::empty(tiny_area);
        bar.render_widget(tiny_area, &mut buffer);
        
        // Should handle gracefully without panicking
    }
}
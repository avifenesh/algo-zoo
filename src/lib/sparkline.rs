//! Sparkline visualization for metrics

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::{Block, Widget},
};

/// A simple sparkline chart renderer
#[derive(Debug)]
pub struct Sparkline {
    width: usize,
    height: usize,
    data: Vec<f64>,
    style: Style,
    block: Option<Block<'static>>,
}

impl Sparkline {
    /// Create a new sparkline with specified dimensions
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width: width.max(1),
            height: height.max(1),
            data: Vec::new(),
            style: Style::default(),
            block: None,
        }
    }

    /// Set the style for the sparkline
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Set the block for the sparkline
    pub fn block(mut self, block: Block<'static>) -> Self {
        self.block = Some(block);
        self
    }

    /// Add a data point to the sparkline
    pub fn add_data_point(&mut self, value: f64) {
        self.data.push(value);
        
        // Keep only the most recent data points that fit in the width
        if self.data.len() > self.width {
            self.data.remove(0);
        }
    }

    /// Set all data points at once
    pub fn set_data(&mut self, data: Vec<f64>) {
        self.data = data;
        
        // Trim to fit width
        if self.data.len() > self.width {
            let start = self.data.len() - self.width;
            self.data = self.data[start..].to_vec();
        }
    }

    /// Clear all data
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Render the sparkline as a string
    pub fn render_string(&self) -> String {
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

        // Character levels for different heights
        let chars = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
        let levels = chars.len() as f64;

        let mut result = String::new();
        
        for &value in &self.data {
            let normalized = ((value - min_val) / range).clamp(0.0, 1.0);
            let level = (normalized * (levels - 1.0)).round() as usize;
            result.push(chars[level]);
        }

        // Pad with spaces if data is shorter than width
        while result.chars().count() < self.width {
            result.push(' ');
        }

        result
    }

    /// Render with labels
    pub fn render_with_labels(&self, title: &str) -> String {
        let sparkline = self.render_string();
        let min_val = self.data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_val = self.data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        format!(
            "{}: {} (min: {:.1}, max: {:.1})",
            title, sparkline, min_val, max_val
        )
    }

    /// Get current data
    pub fn get_data(&self) -> &[f64] {
        &self.data
    }

    /// Get data length
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Get width
    pub fn width(&self) -> usize {
        self.width
    }

    /// Get height  
    pub fn height(&self) -> usize {
        self.height
    }

    /// Set width
    pub fn set_width(&mut self, width: usize) {
        self.width = width.max(1);
        
        // Trim data if necessary
        if self.data.len() > self.width {
            let start = self.data.len() - self.width;
            self.data = self.data[start..].to_vec();
        }
    }

    /// Set height
    pub fn set_height(&mut self, height: usize) {
        self.height = height.max(1);
    }

    /// Render the sparkline to a ratatui buffer
    pub fn render_widget(&self, area: Rect, buf: &mut Buffer) {
        let inner_area = if let Some(ref block) = self.block {
            let inner = block.inner(area);
            block.render(area, buf);
            inner
        } else {
            area
        };

        if self.data.is_empty() || inner_area.width == 0 || inner_area.height == 0 {
            return;
        }

        let sparkline_str = self.render_string();
        let chars: Vec<char> = sparkline_str.chars().collect();
        
        let start_x = inner_area.left();
        let y = inner_area.top() + inner_area.height / 2; // Center vertically
        
        for (i, ch) in chars.iter().enumerate() {
            let x = start_x + i as u16;
            if x >= inner_area.right() {
                break;
            }
            
            if y >= inner_area.top() && y < inner_area.bottom() {
                buf[(x, y)]
                    .set_symbol(&ch.to_string())
                    .set_style(self.style);
            }
        }
    }
}

impl Default for Sparkline {
    fn default() -> Self {
        Self::new(20, 1)
    }
}

impl Widget for Sparkline {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.render_widget(area, buf);
    }
}

impl Widget for &Sparkline {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.render_widget(area, buf);
    }
}

/// Collection of sparklines for multiple metrics
#[derive(Debug)]
pub struct SparklineCollection {
    sparklines: std::collections::HashMap<String, Sparkline>,
    default_width: usize,
    default_height: usize,
}

impl SparklineCollection {
    /// Create a new sparkline collection
    pub fn new(default_width: usize, default_height: usize) -> Self {
        Self {
            sparklines: std::collections::HashMap::new(),
            default_width,
            default_height,
        }
    }

    /// Add or update a sparkline with data
    pub fn update(&mut self, key: &str, value: f64) {
        let sparkline = self.sparklines.entry(key.to_string())
            .or_insert_with(|| Sparkline::new(self.default_width, self.default_height));
        sparkline.add_data_point(value);
    }

    /// Get a sparkline by key
    pub fn get(&self, key: &str) -> Option<&Sparkline> {
        self.sparklines.get(key)
    }

    /// Get a mutable sparkline by key
    pub fn get_mut(&mut self, key: &str) -> Option<&mut Sparkline> {
        self.sparklines.get_mut(key)
    }

    /// Remove a sparkline
    pub fn remove(&mut self, key: &str) -> Option<Sparkline> {
        self.sparklines.remove(key)
    }

    /// Clear all sparklines
    pub fn clear(&mut self) {
        self.sparklines.clear();
    }

    /// Get all keys
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.sparklines.keys()
    }

    /// Render all sparklines
    pub fn render_all(&self) -> String {
        let mut result = String::new();
        let mut keys: Vec<_> = self.sparklines.keys().collect();
        keys.sort();

        for key in keys {
            if let Some(sparkline) = self.sparklines.get(key) {
                result.push_str(&sparkline.render_with_labels(key));
                result.push('\n');
            }
        }

        result
    }

    /// Get number of sparklines
    pub fn len(&self) -> usize {
        self.sparklines.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.sparklines.is_empty()
    }
}

impl Default for SparklineCollection {
    fn default() -> Self {
        Self::new(20, 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sparkline_creation() {
        let sparkline = Sparkline::new(10, 1);
        assert_eq!(sparkline.width(), 10);
        assert_eq!(sparkline.height(), 1);
        assert!(sparkline.is_empty());
    }

    #[test]
    fn test_sparkline_add_data() {
        let mut sparkline = Sparkline::new(5, 1);
        sparkline.add_data_point(1.0);
        sparkline.add_data_point(2.0);
        sparkline.add_data_point(3.0);
        
        assert_eq!(sparkline.len(), 3);
        assert_eq!(sparkline.get_data(), &[1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_sparkline_width_limit() {
        let mut sparkline = Sparkline::new(3, 1);
        for i in 1..=5 {
            sparkline.add_data_point(i as f64);
        }
        
        assert_eq!(sparkline.len(), 3);
        assert_eq!(sparkline.get_data(), &[3.0, 4.0, 5.0]);
    }

    #[test]
    fn test_sparkline_render() {
        let mut sparkline = Sparkline::new(5, 1);
        sparkline.set_data(vec![1.0, 2.0, 3.0]);
        
        let rendered = sparkline.render_string(); // This is the string render method
        assert_eq!(rendered.chars().count(), 5); // Should be padded to width, using char count for Unicode
    }

    #[test]
    fn test_sparkline_collection() {
        let mut collection = SparklineCollection::new(10, 1);
        collection.update("test1", 5.0);
        collection.update("test2", 10.0);
        
        assert_eq!(collection.len(), 2);
        assert!(collection.get("test1").is_some());
        assert!(collection.get("test2").is_some());
    }
}
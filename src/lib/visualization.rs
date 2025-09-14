//! Visualization components for the sorting race

use crate::models::traits::{Sorter, Telemetry};

/// Visualization renderer for the sorting race
#[derive(Debug)]
pub struct RaceVisualizer {
    width: usize,
    height: usize,
    show_comparisons: bool,
    show_metrics: bool,
    color_enabled: bool,
}

impl RaceVisualizer {
    /// Create a new race visualizer
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            show_comparisons: true,
            show_metrics: true,
            color_enabled: true,
        }
    }

    /// Set whether to show comparison highlights
    pub fn set_show_comparisons(&mut self, show: bool) {
        self.show_comparisons = show;
    }

    /// Set whether to show metrics
    pub fn set_show_metrics(&mut self, show: bool) {
        self.show_metrics = show;
    }

    /// Set whether colors are enabled
    pub fn set_color_enabled(&mut self, enabled: bool) {
        self.color_enabled = enabled;
    }

    /// Render the current state of all algorithms
    pub fn render(&self, algorithms: &[Box<dyn Sorter>]) -> String {
        let mut output = String::new();
        
        // Header
        output.push_str(&format!("┌{}┐\n", "─".repeat(self.width - 2)));
        output.push_str(&format!("│{:^width$}│\n", "Sorting Algorithm Race", width = self.width - 2));
        output.push_str(&format!("├{}┤\n", "─".repeat(self.width - 2)));

        // Render each algorithm
        for (i, algorithm) in algorithms.iter().enumerate() {
            let telemetry = algorithm.get_telemetry();
            
            // Algorithm name and status
            let status = if algorithm.is_complete() {
                "COMPLETE"
            } else {
                "RUNNING"
            };
            
            output.push_str(&format!(
                "│ {:<20} │ {:>8} │ C:{:>6} M:{:>6} │\n",
                algorithm.name(),
                status,
                telemetry.total_comparisons,
                telemetry.total_moves
            ));
            
            // Array visualization
            output.push_str(&self.render_array(algorithm.get_array(), &telemetry));
            
            if i < algorithms.len() - 1 {
                output.push_str(&format!("├{}┤\n", "─".repeat(self.width - 2)));
            }
        }

        // Footer
        output.push_str(&format!("└{}┘\n", "─".repeat(self.width - 2)));

        output
    }

    /// Render a single array with highlights
    fn render_array(&self, array: &[i32], telemetry: &Telemetry) -> String {
        let mut output = String::new();
        
        // Calculate scaling for visualization
        let max_val = array.iter().max().copied().unwrap_or(1);
        let bar_height = (self.height / 8).max(1); // Reserve space for multiple algorithms
        
        // Render array as bar chart
        for row in (0..bar_height).rev() {
            output.push_str("│ ");
            
            for (i, &value) in array.iter().enumerate() {
                let normalized_height = (value as f32 / max_val as f32 * bar_height as f32) as usize;
                let char = if normalized_height > row {
                    if telemetry.highlights.contains(&i) && self.show_comparisons {
                        '*' // Highlight character
                    } else {
                        '█' // Normal bar character
                    }
                } else {
                    ' '
                };
                output.push(char);
            }
            
            output.push_str(" │\n");
        }

        // Status text
        if !telemetry.status_text.is_empty() {
            output.push_str(&format!(
                "│ Status: {:<width$} │\n",
                telemetry.status_text,
                width = self.width - 12
            ));
        }

        output
    }

    /// Render metrics summary
    pub fn render_metrics(&self, algorithms: &[Box<dyn Sorter>]) -> String {
        if !self.show_metrics {
            return String::new();
        }

        let mut output = String::new();
        output.push_str("Performance Metrics:\n");
        output.push_str("┌────────────────────┬──────────┬──────────┬──────────┐\n");
        output.push_str("│ Algorithm          │ Comps    │ Moves    │ Memory   │\n");
        output.push_str("├────────────────────┼──────────┼──────────┼──────────┤\n");

        for algorithm in algorithms {
            let telemetry = algorithm.get_telemetry();
            // Get actual memory usage from the algorithm
            let actual_memory = algorithm.get_memory_usage();
            let memory_display = if actual_memory > 0 {
                Self::format_memory_bytes(actual_memory)
            } else {
                // Fall back to telemetry if get_memory_usage returns 0
                if telemetry.memory_current > 0 {
                    Self::format_memory_bytes(telemetry.memory_current)
                } else {
                    "N/A".to_string()
                }
            };
            
            output.push_str(&format!(
                "│ {:<18} │ {:>8} │ {:>8} │ {:>8} │\n",
                algorithm.name(),
                telemetry.total_comparisons,
                telemetry.total_moves,
                memory_display
            ));
        }

        output.push_str("└────────────────────┴──────────┴──────────┴──────────┘\n");
        output
    }

    /// Render progress bars for all algorithms
    pub fn render_progress(&self, algorithms: &[Box<dyn Sorter>]) -> String {
        let mut output = String::new();
        
        for algorithm in algorithms {
            let telemetry = algorithm.get_telemetry();
            let progress = telemetry.progress_hint.clamp(0.0, 1.0);
            let progress_width = 30;
            let filled = (progress * progress_width as f32) as usize;
            
            let bar = "█".repeat(filled) + &"░".repeat(progress_width - filled);
            
            output.push_str(&format!(
                "{:<15} [{}] {:>5.1}%\n",
                algorithm.name(),
                bar,
                progress * 100.0
            ));
        }

        output
    }

    /// Format bytes into human readable string
    fn format_memory_bytes(bytes: usize) -> String {
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

    /// Create a simple text-based visualization frame
    pub fn create_frame(&self, title: &str, content: &str) -> String {
        let lines: Vec<&str> = content.lines().collect();
        let content_width = lines.iter().map(|line| line.len()).max().unwrap_or(0);
        let frame_width = content_width.max(title.len()) + 4;
        
        let mut frame = String::new();
        
        // Top border
        frame.push_str(&format!("┌{}┐\n", "─".repeat(frame_width - 2)));
        
        // Title
        frame.push_str(&format!("│{:^width$}│\n", title, width = frame_width - 2));
        frame.push_str(&format!("├{}┤\n", "─".repeat(frame_width - 2)));
        
        // Content
        for line in lines {
            frame.push_str(&format!("│ {:<width$} │\n", line, width = frame_width - 4));
        }
        
        // Bottom border
        frame.push_str(&format!("└{}┘\n", "─".repeat(frame_width - 2)));
        
        frame
    }
}

impl Default for RaceVisualizer {
    fn default() -> Self {
        Self::new(80, 24)
    }
}
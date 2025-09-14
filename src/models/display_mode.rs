//! Display mode for controlling array visualization

use crate::models::{
    session::AlgorithmType,
    traits::Sorter,
};
use anyhow::{Result, anyhow};
use crossterm::event::{KeyEvent, KeyCode, KeyModifiers};
/// Controls which algorithm's array visualization is shown
#[derive(Debug, Clone)]
pub struct DisplayMode {
    /// Currently viewed algorithm
    pub viewed_algorithm: AlgorithmType,
    /// Available algorithms in current race
    pub available_algorithms: Vec<AlgorithmType>,
    /// Current index in the available algorithms list
    pub cycle_index: usize,
    /// Whether visualization needs update
    needs_update: bool,
}

impl DisplayMode {
    /// Create a new display mode
    pub fn new() -> Self {
        let available_algorithms = AlgorithmType::all();
        Self {
            viewed_algorithm: available_algorithms[0], // Default to first algorithm (BubbleSort)
            available_algorithms,
            cycle_index: 0,
            needs_update: true,
        }
    }

    /// Create display mode with specific algorithm set
    pub fn with_algorithms(algorithms: Vec<AlgorithmType>) -> Result<Self> {
        if algorithms.is_empty() {
            return Err(anyhow!("Cannot create DisplayMode with empty algorithm list"));
        }

        Ok(Self {
            viewed_algorithm: algorithms[0],
            available_algorithms: algorithms,
            cycle_index: 0,
            needs_update: true,
        })
    }

    /// Handle visualization switch key event
    pub fn handle_visualization_switch(&mut self, _key_event: KeyEvent) -> Result<()> {
        self.cycle_to_next_algorithm();
        Ok(())
    }

    /// Cycle to the next algorithm
    pub fn cycle_to_next_algorithm(&mut self) {
        if !self.available_algorithms.is_empty() {
            self.cycle_index = (self.cycle_index + 1) % self.available_algorithms.len();
            self.viewed_algorithm = self.available_algorithms[self.cycle_index];
            self.needs_update = true;
        }
    }

    /// Set viewed algorithm by type
    pub fn set_viewed_algorithm(&mut self, algorithm_type: AlgorithmType) -> Result<()> {
        if let Some(index) = self.available_algorithms.iter().position(|&alg| alg == algorithm_type) {
            self.cycle_index = index;
            self.viewed_algorithm = algorithm_type;
            self.needs_update = true;
            Ok(())
        } else {
            Err(anyhow!("Algorithm {:?} is not available in current race", algorithm_type))
        }
    }

    /// Set viewed algorithm by index
    pub fn set_viewed_algorithm_by_index(&mut self, index: usize) -> Result<()> {
        if index < self.available_algorithms.len() {
            self.cycle_index = index;
            self.viewed_algorithm = self.available_algorithms[index];
            self.needs_update = true;
            Ok(())
        } else {
            Err(anyhow!("Algorithm index {} is out of bounds (max: {})", index, self.available_algorithms.len() - 1))
        }
    }

    /// Check if visualization should be updated
    pub fn should_update_visualization(&self) -> bool {
        self.needs_update
    }

    /// Mark visualization as updated
    pub fn mark_visualization_updated(&mut self) {
        self.needs_update = false;
    }

    /// Get current array data from algorithm instances
    pub fn get_current_array_data<'a>(&self, algorithms: &'a [Box<dyn Sorter>]) -> Option<&'a [i32]> {
        // Find the algorithm that matches our viewed algorithm
        for (i, algorithm) in algorithms.iter().enumerate() {
            if let Some(expected_type) = AlgorithmType::from_index(i)
                && expected_type == self.viewed_algorithm {
                    return Some(algorithm.get_array());
                }
        }
        None
    }

    /// Get array source algorithm name
    pub fn get_array_source_algorithm(&self) -> String {
        self.viewed_algorithm.to_string()
    }

    /// Get currently highlighted algorithm
    pub fn get_highlighted_algorithm(&self) -> AlgorithmType {
        self.viewed_algorithm
    }

    /// Check if specific algorithm is highlighted
    pub fn is_algorithm_highlighted(&self, algorithm_type: AlgorithmType) -> bool {
        self.viewed_algorithm == algorithm_type
    }

    /// Get current algorithm index
    pub fn get_current_algorithm_index(&self) -> usize {
        self.cycle_index
    }

    /// Get total number of available algorithms
    pub fn get_algorithm_count(&self) -> usize {
        self.available_algorithms.len()
    }

    /// Update available algorithms (e.g., when some algorithms fail)
    pub fn update_available_algorithms(&mut self, algorithms: Vec<AlgorithmType>) -> Result<()> {
        if algorithms.is_empty() {
            return Err(anyhow!("Cannot update to empty algorithm list"));
        }

        self.available_algorithms = algorithms;
        
        // Ensure current algorithm is still available
        if !self.available_algorithms.contains(&self.viewed_algorithm) {
            self.viewed_algorithm = self.available_algorithms[0];
            self.cycle_index = 0;
            self.needs_update = true;
        } else {
            // Update cycle index to match current algorithm
            self.cycle_index = self.available_algorithms
                .iter()
                .position(|&alg| alg == self.viewed_algorithm)
                .unwrap_or(0);
        }

        Ok(())
    }

    /// Reset to first algorithm
    pub fn reset_to_first(&mut self) {
        if !self.available_algorithms.is_empty() {
            self.cycle_index = 0;
            self.viewed_algorithm = self.available_algorithms[0];
            self.needs_update = true;
        }
    }

    /// Get algorithm at specific index
    pub fn get_algorithm_at_index(&self, index: usize) -> Option<AlgorithmType> {
        self.available_algorithms.get(index).copied()
    }

    /// Check if we can cycle to next algorithm
    pub fn can_cycle_next(&self) -> bool {
        self.available_algorithms.len() > 1
    }

    /// Get next algorithm in cycle (without changing current)
    pub fn peek_next_algorithm(&self) -> Option<AlgorithmType> {
        if self.available_algorithms.is_empty() {
            None
        } else {
            let next_index = (self.cycle_index + 1) % self.available_algorithms.len();
            self.available_algorithms.get(next_index).copied()
        }
    }

    /// Get previous algorithm in cycle (without changing current)
    pub fn peek_previous_algorithm(&self) -> Option<AlgorithmType> {
        if self.available_algorithms.is_empty() {
            None
        } else {
            let prev_index = if self.cycle_index == 0 {
                self.available_algorithms.len() - 1
            } else {
                self.cycle_index - 1
            };
            self.available_algorithms.get(prev_index).copied()
        }
    }

    /// Create display mode from algorithm instances
    pub fn from_algorithms(algorithms: &[Box<dyn Sorter>]) -> Self {
        let available_algorithms = (0..algorithms.len())
            .filter_map(AlgorithmType::from_index)
            .collect();
        
        Self::with_algorithms(available_algorithms).unwrap_or_default()
    }

    /// Process visualization key event
    pub fn process_key_event(&mut self, key_event: KeyEvent) -> Result<bool> {
        match key_event {
            KeyEvent {
                code: KeyCode::Char('v'),
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                self.handle_visualization_switch(key_event)?;
                Ok(true) // Event was handled
            },
            _ => Ok(false), // Event not handled
        }
    }
}

impl Default for DisplayMode {
    fn default() -> Self {
        Self::new()
    }
}

/// Memory display value for algorithms
#[derive(Debug, Clone, PartialEq)]
pub enum MemoryDisplayValue {
    /// Actual byte value
    Bytes(usize),
    /// Not available (algorithm inactive or error)
    NotAvailable,
}

impl MemoryDisplayValue {
    /// Format as human-readable string
    pub fn as_string(&self) -> String {
        match self {
            MemoryDisplayValue::Bytes(bytes) => {
                Self::format_bytes(*bytes)
            },
            MemoryDisplayValue::NotAvailable => "N/A".to_string(),
        }
    }

    /// Format bytes into human-readable string
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
}

impl std::fmt::Display for MemoryDisplayValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyEventKind;

    #[test]
    fn test_display_mode_creation() {
        let display = DisplayMode::new();
        
        assert_eq!(display.viewed_algorithm, AlgorithmType::BubbleSort);
        assert_eq!(display.available_algorithms.len(), 7);
        assert_eq!(display.cycle_index, 0);
        assert!(display.should_update_visualization());
    }

    #[test]
    fn test_algorithm_cycling() {
        let mut display = DisplayMode::new();
        
        // Should start with BubbleSort
        assert_eq!(display.viewed_algorithm, AlgorithmType::BubbleSort);
        
        // Cycle to next
        display.cycle_to_next_algorithm();
        assert_eq!(display.viewed_algorithm, AlgorithmType::SelectionSort);
        assert_eq!(display.cycle_index, 1);
        
        // Cycle through all and wrap around
        for _ in 0..6 {
            display.cycle_to_next_algorithm();
        }
        assert_eq!(display.viewed_algorithm, AlgorithmType::BubbleSort);
        assert_eq!(display.cycle_index, 0);
    }

    #[test]
    fn test_set_viewed_algorithm() {
        let mut display = DisplayMode::new();
        
        // Set to specific algorithm
        assert!(display.set_viewed_algorithm(AlgorithmType::QuickSort).is_ok());
        assert_eq!(display.viewed_algorithm, AlgorithmType::QuickSort);
        assert_eq!(display.cycle_index, 4);
        
        // Try to set unavailable algorithm (should fail for this test, but normally wouldn't)
        let limited_algorithms = vec![AlgorithmType::BubbleSort, AlgorithmType::QuickSort];
        let mut limited_display = DisplayMode::with_algorithms(limited_algorithms).unwrap();
        assert!(limited_display.set_viewed_algorithm(AlgorithmType::ShellSort).is_err());
    }

    #[test]
    fn test_algorithm_index_operations() {
        let mut display = DisplayMode::new();
        
        // Set by index
        assert!(display.set_viewed_algorithm_by_index(3).is_ok());
        assert_eq!(display.viewed_algorithm, AlgorithmType::MergeSort);
        
        // Invalid index
        assert!(display.set_viewed_algorithm_by_index(10).is_err());
        
        // Get algorithm at index
        assert_eq!(display.get_algorithm_at_index(2), Some(AlgorithmType::InsertionSort));
        assert_eq!(display.get_algorithm_at_index(10), None);
    }

    #[test]
    fn test_peek_operations() {
        let mut display = DisplayMode::new();
        display.set_viewed_algorithm_by_index(2).unwrap(); // InsertionSort
        
        assert_eq!(display.peek_next_algorithm(), Some(AlgorithmType::MergeSort));
        assert_eq!(display.peek_previous_algorithm(), Some(AlgorithmType::SelectionSort));
        
        // Test wrap-around
        display.set_viewed_algorithm_by_index(0).unwrap(); // BubbleSort
        assert_eq!(display.peek_previous_algorithm(), Some(AlgorithmType::ShellSort));
        
        display.set_viewed_algorithm_by_index(6).unwrap(); // ShellSort
        assert_eq!(display.peek_next_algorithm(), Some(AlgorithmType::BubbleSort));
    }

    #[test]
    fn test_key_event_processing() {
        let mut display = DisplayMode::new();
        
        let v_key = KeyEvent {
            code: KeyCode::Char('v'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        };
        
        let initial_algorithm = display.viewed_algorithm;
        let handled = display.process_key_event(v_key).unwrap();
        
        assert!(handled);
        assert_ne!(display.viewed_algorithm, initial_algorithm);
        
        // Test unhandled key
        let other_key = KeyEvent {
            code: KeyCode::Char('x'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        };
        
        let handled = display.process_key_event(other_key).unwrap();
        assert!(!handled);
    }

    #[test]
    fn test_available_algorithms_update() {
        let mut display = DisplayMode::new();
        
        // Update to limited set
        let limited = vec![AlgorithmType::BubbleSort, AlgorithmType::QuickSort, AlgorithmType::HeapSort];
        assert!(display.update_available_algorithms(limited.clone()).is_ok());
        
        assert_eq!(display.available_algorithms, limited);
        assert_eq!(display.get_algorithm_count(), 3);
        
        // Empty list should fail
        assert!(display.update_available_algorithms(vec![]).is_err());
    }

    #[test]
    fn test_memory_display_value() {
        let value_bytes = MemoryDisplayValue::Bytes(1536);
        assert_eq!(value_bytes.to_string(), "1.5KB");
        
        let value_na = MemoryDisplayValue::NotAvailable;
        assert_eq!(value_na.to_string(), "N/A");
        
        // Test byte formatting
        assert_eq!(MemoryDisplayValue::Bytes(512).to_string(), "512B");
        assert_eq!(MemoryDisplayValue::Bytes(1024).to_string(), "1.0KB");
        assert_eq!(MemoryDisplayValue::Bytes(1048576).to_string(), "1.0MB");
    }

    #[test]
    fn test_algorithm_highlighting() {
        let mut display = DisplayMode::new();
        display.set_viewed_algorithm(AlgorithmType::QuickSort).unwrap();
        
        assert!(display.is_algorithm_highlighted(AlgorithmType::QuickSort));
        assert!(!display.is_algorithm_highlighted(AlgorithmType::BubbleSort));
        assert_eq!(display.get_highlighted_algorithm(), AlgorithmType::QuickSort);
    }

    #[test]
    fn test_reset_to_first() {
        let mut display = DisplayMode::new();
        display.cycle_to_next_algorithm(); // Move away from first
        display.cycle_to_next_algorithm();
        
        assert_ne!(display.viewed_algorithm, AlgorithmType::BubbleSort);
        
        display.reset_to_first();
        assert_eq!(display.viewed_algorithm, AlgorithmType::BubbleSort);
        assert_eq!(display.cycle_index, 0);
    }
}
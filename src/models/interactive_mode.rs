//! Interactive mode state machine for terminal interface

use crate::models::{
    configuration::{ConfigurationState, DistributionType},
};
use anyhow::{Result, anyhow};
use std::time::Instant;

/// Application mode states for the interactive interface
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplicationMode {
    /// User is setting parameters
    Configuration,
    /// Algorithms are actively running
    Racing,
    /// Race paused, can view/configure
    Paused,
    /// Race finished, can reconfigure
    Complete,
}

/// Configuration field focus for interactive input
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigurationField {
    ArraySize,
    Distribution,
    FairnessMode,
    BudgetParam,
    AlphaParam,
    BetaParam,
    LearningRateParam,
}

/// Interactive mode state management
#[derive(Debug, Clone)]
pub struct InteractiveMode {
    /// Current application mode
    pub current_mode: ApplicationMode,
    /// Current configuration focus (None means main menu)
    pub config_focus: Option<ConfigurationField>,
    /// Currently selected algorithm for array visualization
    pub array_view_algorithm: usize,
    /// Whether help overlay is visible
    pub help_visible: bool,
    /// Current configuration state
    configuration: ConfigurationState,
    /// Race start time for timing
    race_start_time: Option<Instant>,
    /// Whether race timer is paused
    race_timer_paused: bool,
    /// Error message to display
    error_message: Option<String>,
    /// Whether display needs update
    needs_update: bool,
}

impl InteractiveMode {
    /// Create a new interactive mode
    pub fn new() -> Self {
        Self {
            current_mode: ApplicationMode::Configuration,
            config_focus: None,
            array_view_algorithm: 0,
            help_visible: false,
            configuration: ConfigurationState::new(),
            race_start_time: None,
            race_timer_paused: false,
            error_message: None,
            needs_update: true,
        }
    }

    /// Get current configuration
    pub fn get_current_config(&self) -> &ConfigurationState {
        &self.configuration
    }

    /// Set configuration
    pub fn set_config(&mut self, config: ConfigurationState) {
        self.configuration = config;
        self.needs_update = true;
    }

    /// Check if currently in configuration mode
    pub fn is_configuration_mode(&self) -> bool {
        self.current_mode == ApplicationMode::Configuration
    }

    /// Check if race is currently active
    pub fn is_race_active(&self) -> bool {
        self.current_mode == ApplicationMode::Racing
    }

    /// Check if race is paused
    pub fn is_race_paused(&self) -> bool {
        self.current_mode == ApplicationMode::Paused
    }

    /// Check if race is complete
    pub fn is_race_complete(&self) -> bool {
        self.current_mode == ApplicationMode::Complete
    }

    /// Check if race timer is active
    pub fn is_race_timer_active(&self) -> bool {
        self.race_start_time.is_some() && !self.race_timer_paused
    }

    /// Check if race timer is paused
    pub fn is_race_timer_paused(&self) -> bool {
        self.race_timer_paused
    }

    /// Check if menu is currently visible
    pub fn is_menu_visible(&self) -> bool {
        self.config_focus.is_some()
    }

    /// Check if help overlay should be shown
    pub fn should_show_help_overlay(&self) -> bool {
        self.help_visible
    }

    /// Check if display needs update
    pub fn should_update_display(&self) -> bool {
        self.needs_update
    }

    /// Check if selection has changed (for visual feedback)
    pub fn has_selection_changed(&self) -> bool {
        self.needs_update
    }

    /// Check if error message is visible
    pub fn is_error_message_visible(&self) -> bool {
        self.error_message.is_some()
    }

    /// Get current error message
    pub fn get_error_message(&self) -> Option<&str> {
        self.error_message.as_deref()
    }

    /// Clear error message
    pub fn clear_error_message(&mut self) {
        if self.error_message.is_some() {
            self.error_message = None;
            self.needs_update = true;
        }
    }

    /// Set error message
    pub fn set_error_message(&mut self, message: String) {
        self.error_message = Some(message);
        self.needs_update = true;
    }

    /// Get race elapsed time
    pub fn get_race_elapsed_time(&self) -> Option<std::time::Duration> {
        self.race_start_time.map(|start| start.elapsed())
    }

    /// Get race start time
    pub fn get_race_start_time(&self) -> Option<Instant> {
        self.race_start_time
    }

    /// Transition to Configuration mode
    pub fn transition_to_configuration(&mut self) -> Result<()> {
        match self.current_mode {
            ApplicationMode::Complete => {
                self.current_mode = ApplicationMode::Configuration;
                self.race_start_time = None;
                self.race_timer_paused = false;
                self.clear_error_message();
                self.needs_update = true;
                Ok(())
            },
            ApplicationMode::Configuration => Ok(()), // Already in configuration
            _ => Err(anyhow!("Cannot transition to Configuration from {:?}", self.current_mode)),
        }
    }

    /// Transition to Racing mode
    pub fn transition_to_racing(&mut self) -> Result<()> {
        match self.current_mode {
            ApplicationMode::Configuration => {
                // Validate configuration first
                if let Err(e) = self.configuration.validate() {
                    self.set_error_message(format!("Configuration error: {}", e));
                    return Err(e);
                }

                // Only transition if no config focus is active
                if self.config_focus.is_some() {
                    return Ok(()); // Ignore transition while in configuration focus
                }

                self.current_mode = ApplicationMode::Racing;
                self.race_start_time = Some(Instant::now());
                self.race_timer_paused = false;
                self.clear_error_message();
                self.needs_update = true;
                Ok(())
            },
            ApplicationMode::Paused => {
                self.current_mode = ApplicationMode::Racing;
                self.race_timer_paused = false;
                self.needs_update = true;
                Ok(())
            },
            _ => Err(anyhow!("Cannot transition to Racing from {:?}", self.current_mode)),
        }
    }

    /// Transition to Paused mode
    pub fn transition_to_paused(&mut self) -> Result<()> {
        match self.current_mode {
            ApplicationMode::Racing => {
                self.current_mode = ApplicationMode::Paused;
                self.race_timer_paused = true;
                self.needs_update = true;
                Ok(())
            },
            _ => Err(anyhow!("Cannot transition to Paused from {:?}", self.current_mode)),
        }
    }

    /// Transition to Complete mode
    pub fn transition_to_complete(&mut self) -> Result<()> {
        match self.current_mode {
            ApplicationMode::Racing | ApplicationMode::Paused => {
                self.current_mode = ApplicationMode::Complete;
                self.race_timer_paused = true;
                self.needs_update = true;
                Ok(())
            },
            _ => Err(anyhow!("Cannot transition to Complete from {:?}", self.current_mode)),
        }
    }

    /// Set configuration focus
    pub fn set_config_focus(&mut self, field: ConfigurationField) -> Result<()> {
        if self.current_mode == ApplicationMode::Racing {
            return Err(anyhow!("Cannot change focus while racing"));
        }
        self.config_focus = Some(field);
        self.clear_error_message();
        self.needs_update = true;
        Ok(())
    }

    /// Clear configuration focus (return to main menu)
    pub fn clear_config_focus(&mut self) {
        if self.config_focus.is_some() {
            self.config_focus = None;
            self.needs_update = true;
        }
    }

    /// Toggle help visibility
    pub fn toggle_help(&mut self) {
        self.help_visible = !self.help_visible;
        self.needs_update = true;
    }

    /// Set array view algorithm
    pub fn set_array_view_algorithm(&mut self, algorithm_index: usize) {
        if self.array_view_algorithm != algorithm_index {
            self.array_view_algorithm = algorithm_index;
            self.needs_update = true;
        }
    }

    /// Cycle to next array view algorithm
    pub fn cycle_array_view_algorithm(&mut self, algorithm_count: usize) {
        if algorithm_count > 0 {
            self.array_view_algorithm = (self.array_view_algorithm + 1) % algorithm_count;
            self.needs_update = true;
        }
    }

    /// Mark display as updated
    pub fn mark_display_updated(&mut self) {
        self.needs_update = false;
    }

    /// Set array size interactively
    pub fn set_array_size_interactive(&mut self, size: u32) -> Result<()> {
        self.configuration.set_array_size(size)?;
        self.clear_error_message();
        self.needs_update = true;
        Ok(())
    }

    /// Attempt to set array size with error handling
    pub fn attempt_set_array_size(&mut self, size: u32) -> bool {
        match self.set_array_size_interactive(size) {
            Ok(()) => true,
            Err(e) => {
                self.set_error_message(e.to_string());
                false
            }
        }
    }

    /// Set distribution interactively
    pub fn set_distribution_interactive(&mut self, distribution: DistributionType) {
        self.configuration.distribution = distribution;
        self.clear_error_message();
        self.needs_update = true;
    }

    /// Set fairness mode interactively
    pub fn set_fairness_mode_interactive(&mut self, mode: crate::models::config::FairnessMode) {
        self.configuration.set_fairness_mode(mode);
        self.clear_error_message();
        self.needs_update = true;
    }

    /// Set budget parameter
    pub fn set_budget_parameter(&mut self, budget: u32) -> Result<()> {
        if budget == 0 {
            return Err(anyhow!("Budget must be greater than 0"));
        }
        self.configuration.budget = Some(budget);
        self.clear_error_message();
        self.needs_update = true;
        Ok(())
    }

    /// Get help overlay content
    pub fn get_help_overlay_content(&self) -> String {
        let mut content = String::new();
        content.push_str("Keyboard Shortcuts:\n\n");
        content.push_str("k - Array size configuration\n");
        content.push_str("b - Distribution configuration\n");
        content.push_str("f - Fairness mode configuration\n");
        content.push_str("v - Switch array visualization\n");
        content.push_str("Space - Start/Pause race\n");
        content.push_str("? - Toggle help\n");
        content.push_str("Arrow keys - Navigate menus\n");
        content.push_str("Enter - Confirm selection\n");
        content.push_str("q - Quit\n");
        content
    }
}

impl Default for InteractiveMode {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interactive_mode_creation() {
        let mode = InteractiveMode::new();
        assert_eq!(mode.current_mode, ApplicationMode::Configuration);
        assert_eq!(mode.config_focus, None);
        assert_eq!(mode.array_view_algorithm, 0);
        assert!(!mode.help_visible);
        assert!(!mode.is_error_message_visible());
    }

    #[test]
    fn test_application_mode_transitions() {
        let mut mode = InteractiveMode::new();
        
        // Configuration -> Racing
        assert!(mode.transition_to_racing().is_ok());
        assert_eq!(mode.current_mode, ApplicationMode::Racing);
        assert!(mode.is_race_timer_active());
        
        // Racing -> Paused
        assert!(mode.transition_to_paused().is_ok());
        assert_eq!(mode.current_mode, ApplicationMode::Paused);
        assert!(mode.is_race_timer_paused());
        
        // Paused -> Racing
        assert!(mode.transition_to_racing().is_ok());
        assert_eq!(mode.current_mode, ApplicationMode::Racing);
        assert!(mode.is_race_timer_active());
        
        // Racing -> Complete
        assert!(mode.transition_to_complete().is_ok());
        assert_eq!(mode.current_mode, ApplicationMode::Complete);
        
        // Complete -> Configuration
        assert!(mode.transition_to_configuration().is_ok());
        assert_eq!(mode.current_mode, ApplicationMode::Configuration);
    }

    #[test]
    fn test_configuration_focus() {
        let mut mode = InteractiveMode::new();
        
        assert!(mode.set_config_focus(ConfigurationField::ArraySize).is_ok());
        assert_eq!(mode.config_focus, Some(ConfigurationField::ArraySize));
        assert!(mode.is_menu_visible());
        
        mode.clear_config_focus();
        assert_eq!(mode.config_focus, None);
        assert!(!mode.is_menu_visible());
    }

    #[test]
    fn test_focus_blocked_during_racing() {
        let mut mode = InteractiveMode::new();
        
        mode.transition_to_racing().unwrap();
        assert!(mode.set_config_focus(ConfigurationField::ArraySize).is_err());
    }

    #[test]
    fn test_help_toggle() {
        let mut mode = InteractiveMode::new();
        
        assert!(!mode.should_show_help_overlay());
        
        mode.toggle_help();
        assert!(mode.should_show_help_overlay());
        
        mode.toggle_help();
        assert!(!mode.should_show_help_overlay());
    }

    #[test]
    fn test_array_view_cycling() {
        let mut mode = InteractiveMode::new();
        
        assert_eq!(mode.array_view_algorithm, 0);
        
        mode.cycle_array_view_algorithm(7);
        assert_eq!(mode.array_view_algorithm, 1);
        
        mode.set_array_view_algorithm(6);
        mode.cycle_array_view_algorithm(7);
        assert_eq!(mode.array_view_algorithm, 0); // Should wrap around
    }

    #[test]
    fn test_error_handling() {
        let mut mode = InteractiveMode::new();
        
        assert!(!mode.is_error_message_visible());
        
        mode.set_error_message("Test error".to_string());
        assert!(mode.is_error_message_visible());
        assert_eq!(mode.get_error_message(), Some("Test error"));
        
        mode.clear_error_message();
        assert!(!mode.is_error_message_visible());
        assert_eq!(mode.get_error_message(), None);
    }

    #[test]
    fn test_array_size_validation() {
        let mut mode = InteractiveMode::new();
        
        assert!(mode.set_array_size_interactive(100).is_ok());
        assert!(!mode.attempt_set_array_size(5)); // Too small
        assert!(mode.is_error_message_visible());
        
        mode.clear_error_message();
        assert!(mode.attempt_set_array_size(500)); // Valid
        assert!(!mode.is_error_message_visible());
    }

    #[test]
    fn test_help_content() {
        let mode = InteractiveMode::new();
        let help_content = mode.get_help_overlay_content();
        
        assert!(help_content.contains("k - Array size configuration"));
        assert!(help_content.contains("b - Distribution configuration"));
        assert!(help_content.contains("f - Fairness mode configuration"));
        assert!(help_content.contains("v - Switch array visualization"));
        assert!(help_content.contains("Space - Start/Pause race"));
        assert!(help_content.contains("? - Toggle help"));
        assert!(help_content.contains("q - Quit"));
    }
}
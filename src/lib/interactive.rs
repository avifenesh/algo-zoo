//! Interactive configuration menu components

use crate::models::{
    configuration::{ConfigurationState, DistributionType},
    config::FairnessMode,
    interactive_mode::{ApplicationMode, ConfigurationField, InteractiveMode},
};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Widget},
};

/// Interactive configuration menu system
#[derive(Debug, Clone)]
pub struct InteractiveConfigMenu {
    /// Current interactive mode state
    pub interactive_mode: InteractiveMode,
    /// Current configuration state
    pub config_state: ConfigurationState,
    /// Selected array size index for navigation
    array_size_index: usize,
    /// Selected distribution index for navigation
    distribution_index: usize,
    /// Selected fairness mode index for navigation
    fairness_mode_index: usize,
    /// Current parameter value being edited (for numeric inputs)
    current_parameter_value: Option<String>,
}

impl InteractiveConfigMenu {
    /// Create new interactive configuration menu
    pub fn new() -> Self {
        Self {
            interactive_mode: InteractiveMode::new(),
            config_state: ConfigurationState::new(),
            array_size_index: 3, // Default to 100 (index 3 in [10, 25, 50, 100, 200, 500, 1000])
            distribution_index: 0, // Default to Shuffled
            fairness_mode_index: 2, // Default to WallTime
            current_parameter_value: None,
        }
    }

    /// Get current interactive mode
    pub fn get_interactive_mode(&self) -> &InteractiveMode {
        &self.interactive_mode
    }

    /// Get mutable interactive mode
    pub fn get_interactive_mode_mut(&mut self) -> &mut InteractiveMode {
        &mut self.interactive_mode
    }

    /// Get run configuration from current state if configuration is complete and transitioning to racing
    pub fn get_run_config(&self) -> Option<crate::models::config::RunConfiguration> {
        // Only return a configuration when the user has completed setup and wants to start racing
        if self.config_state.is_valid() && self.interactive_mode.current_mode == ApplicationMode::Racing {
            Some(crate::models::config::RunConfiguration {
                array_size: self.config_state.array_size as usize,
                distribution: self.config_state.to_legacy_distribution(),
                seed: 12345, // Default seed
                fairness_mode: self.config_state.fairness_mode.clone(),
                target_fps: 30,
            })
        } else {
            None
        }
    }

    /// Check if configuration has changed and is ready for racing
    pub fn should_start_new_race(&self) -> bool {
        self.config_state.is_valid() &&
        self.interactive_mode.current_mode == ApplicationMode::Racing &&
        self.interactive_mode.config_focus.is_none()
    }

    /// Check if currently in racing mode
    pub fn is_racing(&self) -> bool {
        self.interactive_mode.current_mode == ApplicationMode::Racing
    }

    /// Handle key events for the interactive menu
    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<bool> {
        match key_event {
            // Configuration focus keys (only work when not racing)
            KeyEvent {
                code: KeyCode::Char('k'),
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                if self.interactive_mode.current_mode != ApplicationMode::Racing {
                    self.interactive_mode.set_config_focus(ConfigurationField::ArraySize)?;
                    self.update_array_size_index_from_config();
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            KeyEvent {
                code: KeyCode::Char('b'),
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                if self.interactive_mode.current_mode != ApplicationMode::Racing {
                    self.interactive_mode.set_config_focus(ConfigurationField::Distribution)?;
                    self.update_distribution_index_from_config();
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            KeyEvent {
                code: KeyCode::Char('f'),
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                if self.interactive_mode.current_mode != ApplicationMode::Racing {
                    self.interactive_mode.set_config_focus(ConfigurationField::FairnessMode)?;
                    self.update_fairness_mode_index_from_config();
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            // Help toggle (works in any mode)
            KeyEvent {
                code: KeyCode::Char('?'),
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                self.interactive_mode.toggle_help();
                Ok(true)
            }
            // Visualization switching (works in any mode if algorithms exist)
            KeyEvent {
                code: KeyCode::Char('v'),
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                self.interactive_mode.cycle_array_view_algorithm(7); // 7 algorithms total
                Ok(true)
            }
            // Race control (Space key)
            KeyEvent {
                code: KeyCode::Char(' '),
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                match self.interactive_mode.current_mode {
                    ApplicationMode::Configuration => {
                        self.interactive_mode.transition_to_racing()?;
                        Ok(true)
                    }
                    ApplicationMode::Racing => {
                        self.interactive_mode.transition_to_paused()?;
                        Ok(true)
                    }
                    ApplicationMode::Paused => {
                        self.interactive_mode.transition_to_racing()?;
                        Ok(true)
                    }
                    ApplicationMode::Complete => {
                        self.interactive_mode.transition_to_configuration()?;
                        Ok(true)
                    }
                }
            }
            // Navigation keys (when in configuration focus)
            KeyEvent {
                code: KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right,
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                if let Some(field) = self.interactive_mode.config_focus {
                    self.handle_navigation_key(field, key_event.code)?;
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            // Confirmation key (Enter)
            KeyEvent {
                code: KeyCode::Enter,
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                if self.interactive_mode.config_focus.is_some() {
                    self.handle_confirmation()?;
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            // Escape key to cancel focus
            KeyEvent {
                code: KeyCode::Esc,
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                if self.interactive_mode.config_focus.is_some() {
                    self.interactive_mode.clear_config_focus();
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            // Numeric input for parameters (digits and decimal point)
            KeyEvent {
                code: KeyCode::Char(c),
                modifiers: KeyModifiers::NONE,
                ..
            } if c.is_ascii_digit() || c == '.' => {
                if let Some(field) = self.interactive_mode.config_focus {
                    if matches!(field, ConfigurationField::BudgetParam | ConfigurationField::AlphaParam | ConfigurationField::BetaParam | ConfigurationField::LearningRateParam) {
                        self.handle_numeric_input(c)?;
                        Ok(true)
                    } else {
                        Ok(false)
                    }
                } else {
                    Ok(false)
                }
            }
            // Backspace for parameter editing
            KeyEvent {
                code: KeyCode::Backspace,
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                if let Some(field) = self.interactive_mode.config_focus {
                    if matches!(field, ConfigurationField::BudgetParam | ConfigurationField::AlphaParam | ConfigurationField::BetaParam | ConfigurationField::LearningRateParam) {
                        self.handle_backspace_input()?;
                        Ok(true)
                    } else {
                        Ok(false)
                    }
                } else {
                    Ok(false)
                }
            }
            _ => Ok(false), // Event not handled
        }
    }

    /// Handle navigation keys within configuration menus
    fn handle_navigation_key(&mut self, field: ConfigurationField, key_code: KeyCode) -> Result<()> {
        match field {
            ConfigurationField::ArraySize => {
                let sizes = ConfigurationState::get_available_array_sizes();
                match key_code {
                    KeyCode::Up => {
                        if self.array_size_index > 0 {
                            self.array_size_index -= 1;
                        } else {
                            self.array_size_index = sizes.len() - 1; // Wrap to end
                        }
                    }
                    KeyCode::Down => {
                        if self.array_size_index < sizes.len() - 1 {
                            self.array_size_index += 1;
                        } else {
                            self.array_size_index = 0; // Wrap to beginning
                        }
                    }
                    _ => {}
                }
            }
            ConfigurationField::Distribution => {
                let distributions = ConfigurationState::get_available_distributions();
                match key_code {
                    KeyCode::Up => {
                        if self.distribution_index > 0 {
                            self.distribution_index -= 1;
                        } else {
                            self.distribution_index = distributions.len() - 1; // Wrap to end
                        }
                    }
                    KeyCode::Down => {
                        if self.distribution_index < distributions.len() - 1 {
                            self.distribution_index += 1;
                        } else {
                            self.distribution_index = 0; // Wrap to beginning
                        }
                    }
                    _ => {}
                }
            }
            ConfigurationField::FairnessMode => {
                let fairness_modes = ConfigurationState::get_available_fairness_modes();
                match key_code {
                    KeyCode::Up => {
                        if self.fairness_mode_index > 0 {
                            self.fairness_mode_index -= 1;
                        } else {
                            self.fairness_mode_index = fairness_modes.len() - 1; // Wrap to end
                        }
                    }
                    KeyCode::Down => {
                        if self.fairness_mode_index < fairness_modes.len() - 1 {
                            self.fairness_mode_index += 1;
                        } else {
                            self.fairness_mode_index = 0; // Wrap to beginning
                        }
                    }
                    _ => {}
                }
            }
            _ => {
                // Handle parameter fields if needed
            }
        }
        Ok(())
    }

    /// Handle confirmation (Enter key) in configuration menus
    fn handle_confirmation(&mut self) -> Result<()> {
        if let Some(field) = self.interactive_mode.config_focus {
            match field {
                ConfigurationField::ArraySize => {
                    let sizes = ConfigurationState::get_available_array_sizes();
                    if let Some(size) = sizes.get(self.array_size_index) {
                        self.interactive_mode.set_array_size_interactive(*size)?;
                        self.config_state.set_array_size(*size)?; // Sync config_state
                        self.interactive_mode.clear_config_focus();
                    }
                }
                ConfigurationField::Distribution => {
                    let distributions = ConfigurationState::get_available_distributions();
                    if let Some(distribution) = distributions.get(self.distribution_index) {
                        self.interactive_mode.set_distribution_interactive(*distribution);
                        self.config_state.distribution = *distribution; // Sync config_state
                        self.interactive_mode.clear_config_focus();
                    }
                }
                ConfigurationField::FairnessMode => {
                    let fairness_modes = ConfigurationState::get_available_fairness_modes();
                    if let Some(fairness_mode) = fairness_modes.get(self.fairness_mode_index) {
                        self.interactive_mode.set_fairness_mode_interactive(fairness_mode.clone());
                        self.config_state.set_fairness_mode(fairness_mode.clone()); // Sync config_state
                        self.interactive_mode.clear_config_focus();
                    }
                }
                ConfigurationField::BudgetParam => {
                    if let Some(ref value_str) = self.current_parameter_value {
                        if let Ok(budget) = value_str.parse::<u32>() {
                            self.interactive_mode.set_budget_parameter(budget)?;
                            self.config_state.budget = Some(budget);
                            self.current_parameter_value = None;
                            self.interactive_mode.clear_config_focus();
                        }
                    }
                }
                ConfigurationField::AlphaParam => {
                    if let Some(ref value_str) = self.current_parameter_value {
                        if let Ok(alpha) = value_str.parse::<f32>() {
                            if alpha > 0.0 {
                                self.config_state.alpha = Some(alpha);
                                self.current_parameter_value = None;
                                self.interactive_mode.clear_config_focus();
                            }
                        }
                    }
                }
                ConfigurationField::BetaParam => {
                    if let Some(ref value_str) = self.current_parameter_value {
                        if let Ok(beta) = value_str.parse::<f32>() {
                            if beta > 0.0 {
                                self.config_state.beta = Some(beta);
                                self.current_parameter_value = None;
                                self.interactive_mode.clear_config_focus();
                            }
                        }
                    }
                }
                ConfigurationField::LearningRateParam => {
                    if let Some(ref value_str) = self.current_parameter_value {
                        if let Ok(learning_rate) = value_str.parse::<f32>() {
                            if learning_rate > 0.0 && learning_rate <= 1.0 {
                                self.config_state.learning_rate = Some(learning_rate);
                                self.current_parameter_value = None;
                                self.interactive_mode.clear_config_focus();
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Update array size index from current configuration
    fn update_array_size_index_from_config(&mut self) {
        let sizes = ConfigurationState::get_available_array_sizes();
        let current_size = self.interactive_mode.get_current_config().array_size;
        
        if let Some(index) = sizes.iter().position(|&size| size == current_size) {
            self.array_size_index = index;
        }
    }

    /// Update distribution index from current configuration
    fn update_distribution_index_from_config(&mut self) {
        let distributions = ConfigurationState::get_available_distributions();
        let current_distribution = self.interactive_mode.get_current_config().distribution;
        
        if let Some(index) = distributions.iter().position(|&dist| dist == current_distribution) {
            self.distribution_index = index;
        }
    }

    /// Update fairness mode index from current configuration  
    fn update_fairness_mode_index_from_config(&mut self) {
        let fairness_modes = ConfigurationState::get_available_fairness_modes();
        let current_mode = &self.interactive_mode.get_current_config().fairness_mode;
        
        // Find matching fairness mode by type
        if let Some(index) = fairness_modes.iter().position(|mode| {
            std::mem::discriminant(mode) == std::mem::discriminant(current_mode)
        }) {
            self.fairness_mode_index = index;
        }
    }

    /// Render the interactive configuration UI
    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        // Main configuration screen
        self.render_main_config_screen(area, buf);

        // Overlay menus based on current focus
        if let Some(field) = self.interactive_mode.config_focus {
            match field {
                ConfigurationField::ArraySize => {
                    self.render_array_size_menu(area, buf);
                }
                ConfigurationField::Distribution => {
                    self.render_distribution_menu(area, buf);
                }
                ConfigurationField::FairnessMode => {
                    self.render_fairness_mode_menu(area, buf);
                }
                _ => {
                    // Render parameter input menus
                }
            }
        }

        // Help overlay
        if self.interactive_mode.should_show_help_overlay() {
            self.render_help_overlay(area, buf);
        }
    }

    /// Render the main configuration screen
    fn render_main_config_screen(&self, area: Rect, buf: &mut Buffer) {
        let config = self.interactive_mode.get_current_config();
        
        // Create main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Min(0),    // Content
                Constraint::Length(3), // Status/Instructions
            ])
            .split(area);

        // Title
        let title = Paragraph::new(vec![
            Line::from(vec![
                Span::styled(
                    "Sorting Race v0.2 - Interactive Configuration",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
        ])
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center);
        title.render(chunks[0], buf);

        // Configuration content
        let config_lines = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("Current Configuration:", Style::default().add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Array Size:     ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    format!("{} elements", config.array_size),
                    Style::default().fg(Color::White),
                ),
                Span::styled("  [Press 'k' to change]", Style::default().fg(Color::Gray)),
            ]),
            Line::from(vec![
                Span::styled("Distribution:   ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    format!("{:?}", config.distribution),
                    Style::default().fg(Color::White),
                ),
                Span::styled("  [Press 'b' to change]", Style::default().fg(Color::Gray)),
            ]),
            Line::from(vec![
                Span::styled("Fairness Mode:  ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    self.format_fairness_mode(&config.fairness_mode),
                    Style::default().fg(Color::White),
                ),
                Span::styled("  [Press 'f' to change]", Style::default().fg(Color::Gray)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    match self.interactive_mode.current_mode {
                        ApplicationMode::Configuration => "Ready to start race",
                        ApplicationMode::Racing => "Race in progress...",
                        ApplicationMode::Paused => "Race paused",
                        ApplicationMode::Complete => "Race complete - configure for next race",
                    },
                    Style::default().fg(match self.interactive_mode.current_mode {
                        ApplicationMode::Configuration => Color::Green,
                        ApplicationMode::Racing => Color::Blue,
                        ApplicationMode::Paused => Color::Yellow,
                        ApplicationMode::Complete => Color::Magenta,
                    }),
                ),
            ]),
        ];

        let config_content = Paragraph::new(config_lines)
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Left);
        config_content.render(chunks[1], buf);

        // Instructions
        let instruction_text = match self.interactive_mode.current_mode {
            ApplicationMode::Configuration => "Press SPACE to start race | k/b/f to configure | v to switch array view | ? for help | q to quit",
            ApplicationMode::Racing => "Press SPACE to pause | v to switch array view | ? for help | q to quit",
            ApplicationMode::Paused => "Press SPACE to resume | k/b/f to configure | v to switch array view | ? for help | q to quit",
            ApplicationMode::Complete => "Press SPACE or k/b/f to configure next race | v to switch array view | ? for help | q to quit",
        };

        let instructions = Paragraph::new(instruction_text)
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Cyan));
        instructions.render(chunks[2], buf);

        // Error message overlay if present
        if let Some(error) = self.interactive_mode.get_error_message() {
            self.render_error_overlay(area, buf, error);
        }
    }

    /// Format fairness mode for display
    fn format_fairness_mode(&self, mode: &FairnessMode) -> String {
        match mode {
            FairnessMode::ComparisonBudget { k } => format!("Comparison (budget: {})", k),
            FairnessMode::Weighted { alpha, beta } => format!("Weighted (α:{:.1}, β:{:.1})", alpha, beta),
            FairnessMode::WallTime { slice_ms } => format!("Wall-time ({}ms)", slice_ms),
            FairnessMode::Adaptive { learning_rate } => format!("Adaptive (rate:{:.1})", learning_rate),
            FairnessMode::EqualSteps => "Equal Steps".to_string(),
        }
    }

    /// Render array size selection menu
    fn render_array_size_menu(&self, area: Rect, buf: &mut Buffer) {
        let sizes = ConfigurationState::get_available_array_sizes();
        
        // Create popup area
        let popup_area = self.centered_rect(40, 60, area);
        
        // Clear background
        Clear.render(popup_area, buf);
        
        // Create menu items
        let items: Vec<ListItem> = sizes
            .iter()
            .enumerate()
            .map(|(i, &size)| {
                let style = if i == self.array_size_index {
                    Style::default().bg(Color::Blue).fg(Color::White)
                } else {
                    Style::default()
                };
                
                ListItem::new(format!("{} elements", size)).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("Select Array Size"))
            .highlight_style(Style::default().bg(Color::Blue));

        list.render(popup_area, buf);

        // Instructions
        let instruction_area = Rect {
            x: popup_area.x,
            y: popup_area.y + popup_area.height,
            width: popup_area.width,
            height: 1,
        };

        if instruction_area.y < area.height {
            let instructions = Paragraph::new("↑↓ Navigate | Enter to select | Esc to cancel")
                .style(Style::default().fg(Color::Gray));
            instructions.render(instruction_area, buf);
        }
    }

    /// Render distribution selection menu
    fn render_distribution_menu(&self, area: Rect, buf: &mut Buffer) {
        let distributions = ConfigurationState::get_available_distributions();
        
        // Create popup area
        let popup_area = self.centered_rect(40, 50, area);
        
        // Clear background
        Clear.render(popup_area, buf);
        
        // Create menu items
        let items: Vec<ListItem> = distributions
            .iter()
            .enumerate()
            .map(|(i, &dist)| {
                let style = if i == self.distribution_index {
                    Style::default().bg(Color::Blue).fg(Color::White)
                } else {
                    Style::default()
                };
                
                let description = match dist {
                    DistributionType::Shuffled => "Random order",
                    DistributionType::Reversed => "Reverse sorted",
                    DistributionType::NearlySorted => "Mostly sorted",
                    DistributionType::FewUnique => "Few unique values",
                };
                
                ListItem::new(format!("{:?} - {}", dist, description)).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("Select Distribution"))
            .highlight_style(Style::default().bg(Color::Blue));

        list.render(popup_area, buf);

        // Instructions
        let instruction_area = Rect {
            x: popup_area.x,
            y: popup_area.y + popup_area.height,
            width: popup_area.width,
            height: 1,
        };

        if instruction_area.y < area.height {
            let instructions = Paragraph::new("↑↓ Navigate | Enter to select | Esc to cancel")
                .style(Style::default().fg(Color::Gray));
            instructions.render(instruction_area, buf);
        }
    }

    /// Render fairness mode selection menu
    fn render_fairness_mode_menu(&self, area: Rect, buf: &mut Buffer) {
        let fairness_modes = ConfigurationState::get_available_fairness_modes();
        
        // Create popup area
        let popup_area = self.centered_rect(60, 70, area);
        
        // Clear background
        Clear.render(popup_area, buf);
        
        // Create menu items
        let items: Vec<ListItem> = fairness_modes
            .iter()
            .enumerate()
            .map(|(i, mode)| {
                let style = if i == self.fairness_mode_index {
                    Style::default().bg(Color::Blue).fg(Color::White)
                } else {
                    Style::default()
                };
                
                let description = match mode {
                    FairnessMode::ComparisonBudget { .. } => "Fixed comparison budget per step",
                    FairnessMode::Weighted { .. } => "Weighted by comparisons and moves",
                    FairnessMode::WallTime { .. } => "Equal time slices for each algorithm",
                    FairnessMode::Adaptive { .. } => "Adaptive allocation based on performance",
                    _ => "Equal steps",
                };
                
                ListItem::new(vec![
                    Line::from(self.format_fairness_mode(mode)),
                    Line::from(Span::styled(description, Style::default().fg(Color::Gray))),
                ]).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("Select Fairness Mode"))
            .highlight_style(Style::default().bg(Color::Blue));

        list.render(popup_area, buf);

        // Instructions
        let instruction_area = Rect {
            x: popup_area.x,
            y: popup_area.y + popup_area.height,
            width: popup_area.width,
            height: 1,
        };

        if instruction_area.y < area.height {
            let instructions = Paragraph::new("↑↓ Navigate | Enter to select | Esc to cancel")
                .style(Style::default().fg(Color::Gray));
            instructions.render(instruction_area, buf);
        }
    }

    /// Render help overlay
    fn render_help_overlay(&self, area: Rect, buf: &mut Buffer) {
        let popup_area = self.centered_rect(80, 80, area);
        
        // Clear background
        Clear.render(popup_area, buf);
        
        let help_content = self.interactive_mode.get_help_overlay_content();
        let help_widget = Paragraph::new(help_content)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("Help - Keyboard Shortcuts"))
            .alignment(Alignment::Left);
        
        help_widget.render(popup_area, buf);
    }

    /// Render error message overlay
    fn render_error_overlay(&self, area: Rect, buf: &mut Buffer, error: &str) {
        let popup_area = self.centered_rect(60, 20, area);
        
        // Clear background
        Clear.render(popup_area, buf);
        
        let error_widget = Paragraph::new(error)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("Error")
                .border_style(Style::default().fg(Color::Red)))
            .style(Style::default().fg(Color::Red))
            .alignment(Alignment::Center);
        
        error_widget.render(popup_area, buf);
    }

    /// Helper function to create centered rectangle
    fn centered_rect(&self, percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1]
    }

    /// Handle numeric input for parameter fields
    fn handle_numeric_input(&mut self, digit: char) -> Result<()> {
        if self.current_parameter_value.is_none() {
            self.current_parameter_value = Some(String::new());
        }
        let current_value = self.current_parameter_value.as_mut().unwrap();

        // Limit input length to prevent overflow and validate input
        if current_value.len() < 10 {
            // Only allow one decimal point
            if digit == '.' && current_value.contains('.') {
                return Ok(());
            }
            current_value.push(digit);
        }
        Ok(())
    }

    /// Handle backspace input for parameter fields
    fn handle_backspace_input(&mut self) -> Result<()> {
        if let Some(ref mut current_value) = self.current_parameter_value {
            current_value.pop();
            if current_value.is_empty() {
                self.current_parameter_value = None;
            }
        }
        Ok(())
    }
}

impl Default for InteractiveConfigMenu {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for InteractiveConfigMenu {
    fn render(self, area: Rect, buf: &mut Buffer) {
        InteractiveConfigMenu::render(&self, area, buf);
    }
}

impl Widget for &InteractiveConfigMenu {
    fn render(self, area: Rect, buf: &mut Buffer) {
        InteractiveConfigMenu::render(self, area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyEventKind;

    #[test]
    fn test_interactive_config_menu_creation() {
        let menu = InteractiveConfigMenu::new();
        assert_eq!(menu.interactive_mode.current_mode, ApplicationMode::Configuration);
        assert_eq!(menu.array_size_index, 3); // 100 elements
        assert_eq!(menu.distribution_index, 0); // Shuffled
        assert_eq!(menu.fairness_mode_index, 2); // WallTime
    }

    #[test]
    fn test_configuration_key_handling() {
        let mut menu = InteractiveConfigMenu::new();
        
        // Test 'k' key for array size
        let k_key = KeyEvent {
            code: KeyCode::Char('k'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        };
        
        let handled = menu.handle_key_event(k_key).unwrap();
        assert!(handled);
        assert_eq!(menu.interactive_mode.config_focus, Some(ConfigurationField::ArraySize));
    }

    #[test]
    fn test_navigation_in_array_size_menu() {
        let mut menu = InteractiveConfigMenu::new();
        menu.interactive_mode.set_config_focus(ConfigurationField::ArraySize).unwrap();
        menu.array_size_index = 2; // Start at index 2 (50)
        
        // Test up navigation
        let up_key = KeyEvent {
            code: KeyCode::Up,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        };
        
        let handled = menu.handle_key_event(up_key).unwrap();
        assert!(handled);
        assert_eq!(menu.array_size_index, 1); // Should move to index 1 (25)
    }

    #[test]
    fn test_navigation_wrapping() {
        let mut menu = InteractiveConfigMenu::new();
        menu.interactive_mode.set_config_focus(ConfigurationField::ArraySize).unwrap();
        menu.array_size_index = 0; // Start at first item
        
        // Test up navigation from first item (should wrap to last)
        let up_key = KeyEvent {
            code: KeyCode::Up,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        };
        
        let handled = menu.handle_key_event(up_key).unwrap();
        assert!(handled);
        let sizes = ConfigurationState::get_available_array_sizes();
        assert_eq!(menu.array_size_index, sizes.len() - 1); // Should wrap to last
    }

    #[test]
    fn test_confirmation_handling() {
        let mut menu = InteractiveConfigMenu::new();
        menu.interactive_mode.set_config_focus(ConfigurationField::ArraySize).unwrap();
        menu.array_size_index = 0; // Select first size (10)
        
        let enter_key = KeyEvent {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        };
        
        let handled = menu.handle_key_event(enter_key).unwrap();
        assert!(handled);
        assert_eq!(menu.interactive_mode.get_current_config().array_size, 10);
        assert_eq!(menu.interactive_mode.config_focus, None); // Should clear focus
    }

    #[test]
    fn test_help_toggle() {
        let mut menu = InteractiveConfigMenu::new();
        
        let help_key = KeyEvent {
            code: KeyCode::Char('?'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        };
        
        assert!(!menu.interactive_mode.should_show_help_overlay());
        
        let handled = menu.handle_key_event(help_key).unwrap();
        assert!(handled);
        assert!(menu.interactive_mode.should_show_help_overlay());
    }

    #[test]
    fn test_race_control_transitions() {
        let mut menu = InteractiveConfigMenu::new();
        
        let space_key = KeyEvent {
            code: KeyCode::Char(' '),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        };
        
        // Configuration -> Racing
        assert_eq!(menu.interactive_mode.current_mode, ApplicationMode::Configuration);
        let handled = menu.handle_key_event(space_key).unwrap();
        assert!(handled);
        assert_eq!(menu.interactive_mode.current_mode, ApplicationMode::Racing);
        
        // Racing -> Paused
        let handled = menu.handle_key_event(space_key).unwrap();
        assert!(handled);
        assert_eq!(menu.interactive_mode.current_mode, ApplicationMode::Paused);
    }
}
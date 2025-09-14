//! Input handling for the terminal interface

use std::io::{self, Read, Write};

/// Input handler for terminal interface
#[derive(Debug, Default)]
pub struct InputHandler {
    buffer: String,
}

impl InputHandler {
    /// Create a new input handler
    pub fn new() -> Self {
        Self::default()
    }

    /// Read a line of input from stdin
    pub fn read_line(&mut self) -> io::Result<String> {
        self.buffer.clear();
        io::stdin().read_line(&mut self.buffer)?;
        Ok(self.buffer.trim().to_string())
    }

    /// Read a single character without waiting for Enter
    pub fn read_char(&mut self) -> io::Result<char> {
        let mut buffer = [0; 1];
        io::stdin().read_exact(&mut buffer)?;
        Ok(buffer[0] as char)
    }

    /// Prompt user with a message and read input
    pub fn prompt(&mut self, message: &str) -> io::Result<String> {
        print!("{}", message);
        io::stdout().flush()?;
        self.read_line()
    }

    /// Read a number from input with validation
    pub fn read_number<T>(&mut self, prompt: &str) -> io::Result<T>
    where
        T: std::str::FromStr,
        T::Err: std::fmt::Display,
    {
        loop {
            let input = self.prompt(prompt)?;
            match input.parse::<T>() {
                Ok(value) => return Ok(value),
                Err(e) => {
                    println!("Invalid input: {}. Please try again.", e);
                }
            }
        }
    }

    /// Read a yes/no answer from input
    pub fn read_yes_no(&mut self, prompt: &str) -> io::Result<bool> {
        loop {
            let input = self.prompt(&format!("{} (y/n): ", prompt))?;
            match input.to_lowercase().as_str() {
                "y" | "yes" | "true" | "1" => return Ok(true),
                "n" | "no" | "false" | "0" => return Ok(false),
                _ => println!("Please enter 'y' for yes or 'n' for no."),
            }
        }
    }

    /// Read input with validation using a predicate function
    pub fn read_validated<F>(&mut self, prompt: &str, validator: F) -> io::Result<String>
    where
        F: Fn(&str) -> Result<(), String>,
    {
        loop {
            let input = self.prompt(prompt)?;
            match validator(&input) {
                Ok(()) => return Ok(input),
                Err(error) => {
                    println!("Invalid input: {}. Please try again.", error);
                }
            }
        }
    }

    /// Read input from a list of choices
    pub fn read_choice(&mut self, prompt: &str, choices: &[&str]) -> io::Result<usize> {
        loop {
            println!("{}", prompt);
            for (i, choice) in choices.iter().enumerate() {
                println!("{}. {}", i + 1, choice);
            }

            let input = self.prompt("Enter your choice (number): ")?;
            match input.parse::<usize>() {
                Ok(choice) if choice > 0 && choice <= choices.len() => {
                    return Ok(choice - 1);
                }
                _ => {
                    println!(
                        "Please enter a number between 1 and {}.",
                        choices.len()
                    );
                }
            }
        }
    }

    /// Clear the input buffer
    pub fn clear_buffer(&mut self) {
        self.buffer.clear();
    }
}

/// Key codes for special keys
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyCode {
    Char(char),
    Enter,
    Escape,
    Space,
    Backspace,
    Tab,
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
    Delete,
    Insert,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
}

/// Event representing user input
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputEvent {
    Key(KeyCode),
    Resize(u16, u16), // width, height
    Mouse(u16, u16),  // x, y
}

/// Non-blocking input reader (stub implementation)
#[derive(Debug)]
pub struct NonBlockingInput {
    _private: (),
}

impl NonBlockingInput {
    /// Create a new non-blocking input reader
    pub fn new() -> io::Result<Self> {
        Ok(Self { _private: () })
    }

    /// Poll for input events (stub implementation)
    pub fn poll_event(&mut self) -> io::Result<Option<InputEvent>> {
        // This is a stub implementation
        // In a real implementation, this would use platform-specific code
        // to read input without blocking
        Ok(None)
    }

    /// Check if input is available
    pub fn has_input(&self) -> bool {
        // Stub implementation
        false
    }
}

impl Default for NonBlockingInput {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

/// Menu system for terminal interface
#[derive(Debug)]
pub struct Menu {
    title: String,
    items: Vec<String>,
    selected_index: usize,
}

impl Menu {
    /// Create a new menu
    pub fn new(title: String, items: Vec<String>) -> Self {
        Self {
            title,
            items,
            selected_index: 0,
        }
    }

    /// Display the menu and get user selection
    pub fn show(&mut self, input_handler: &mut InputHandler) -> io::Result<usize> {
        loop {
            // Clear screen (simple implementation)
            print!("\x1B[2J\x1B[H");
            
            // Display title
            println!("{}", self.title);
            println!("{}", "=".repeat(self.title.len()));
            println!();

            // Display items
            for (i, item) in self.items.iter().enumerate() {
                if i == self.selected_index {
                    println!("> {}", item);
                } else {
                    println!("  {}", item);
                }
            }

            println!();
            println!("Use numbers to select, or 'q' to quit:");

            let input = input_handler.read_line()?;
            
            if input == "q" || input == "quit" {
                return Err(io::Error::new(io::ErrorKind::Interrupted, "User quit"));
            }

            match input.parse::<usize>() {
                Ok(choice) if choice > 0 && choice <= self.items.len() => {
                    return Ok(choice - 1);
                }
                _ => {
                    println!("Please enter a number between 1 and {} or 'q' to quit.", self.items.len());
                    println!("Press Enter to continue...");
                    input_handler.read_line()?;
                }
            }
        }
    }

    /// Set the selected index
    pub fn set_selected(&mut self, index: usize) {
        if index < self.items.len() {
            self.selected_index = index;
        }
    }

    /// Get the selected index
    pub fn get_selected(&self) -> usize {
        self.selected_index
    }

    /// Add an item to the menu
    pub fn add_item(&mut self, item: String) {
        self.items.push(item);
    }

    /// Remove an item from the menu
    pub fn remove_item(&mut self, index: usize) -> Option<String> {
        if index < self.items.len() {
            Some(self.items.remove(index))
        } else {
            None
        }
    }

    /// Get the number of items
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if menu is empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_handler_creation() {
        let handler = InputHandler::new();
        assert_eq!(handler.buffer.len(), 0);
    }

    #[test]
    fn test_non_blocking_input_creation() {
        let input = NonBlockingInput::new();
        assert!(input.is_ok());
    }

    #[test]
    fn test_menu_creation() {
        let menu = Menu::new(
            "Test Menu".to_string(),
            vec!["Option 1".to_string(), "Option 2".to_string()],
        );
        assert_eq!(menu.len(), 2);
        assert_eq!(menu.get_selected(), 0);
    }

    #[test]
    fn test_menu_selection() {
        let mut menu = Menu::new(
            "Test Menu".to_string(),
            vec!["Option 1".to_string(), "Option 2".to_string()],
        );
        menu.set_selected(1);
        assert_eq!(menu.get_selected(), 1);
    }

    #[test]
    fn test_menu_add_remove() {
        let mut menu = Menu::new("Test".to_string(), vec![]);
        menu.add_item("Item 1".to_string());
        assert_eq!(menu.len(), 1);
        
        let removed = menu.remove_item(0);
        assert_eq!(removed, Some("Item 1".to_string()));
        assert_eq!(menu.len(), 0);
    }
}
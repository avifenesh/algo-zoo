//! Algorithm state management

/// Algorithm execution state
#[derive(Debug, Clone, PartialEq)]
pub enum AlgorithmState {
    /// Algorithm is ready to start
    Ready,
    /// Algorithm is actively executing
    Running,
    /// Algorithm has completed successfully
    Complete,
    /// Algorithm encountered an error
    Error(String),
}

impl Default for AlgorithmState {
    fn default() -> Self {
        AlgorithmState::Ready
    }
}

/// Algorithm configuration and state tracking
#[derive(Debug, Default)]
pub struct Algorithm {
    /// Current execution state
    pub state: AlgorithmState,
    /// Number of steps executed
    pub steps_executed: usize,
    /// Total runtime in milliseconds
    pub runtime_ms: u64,
    /// Whether algorithm is paused
    pub is_paused: bool,
}

impl Algorithm {
    /// Create a new algorithm state
    pub fn new() -> Self {
        Self::default()
    }

    /// Reset algorithm to initial state
    pub fn reset(&mut self) {
        self.state = AlgorithmState::Ready;
        self.steps_executed = 0;
        self.runtime_ms = 0;
        self.is_paused = false;
    }

    /// Check if algorithm can execute a step
    pub fn can_step(&self) -> bool {
        matches!(self.state, AlgorithmState::Ready | AlgorithmState::Running) && !self.is_paused
    }

    /// Mark algorithm as complete
    pub fn complete(&mut self) {
        self.state = AlgorithmState::Complete;
    }

    /// Mark algorithm as running
    pub fn start(&mut self) {
        self.state = AlgorithmState::Running;
    }

    /// Set error state
    pub fn set_error(&mut self, error: String) {
        self.state = AlgorithmState::Error(error);
    }

    /// Increment step counter
    pub fn increment_steps(&mut self) {
        self.steps_executed += 1;
    }

    /// Add to runtime
    pub fn add_runtime(&mut self, ms: u64) {
        self.runtime_ms += ms;
    }

    /// Toggle pause state
    pub fn toggle_pause(&mut self) {
        self.is_paused = !self.is_paused;
    }
}
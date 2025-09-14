//! Configuration types for sorting race simulation

/// Data distribution patterns for generated arrays
#[derive(Debug, Clone, PartialEq)]
pub enum Distribution {
    /// Random shuffled array
    Shuffled,
    /// Nearly sorted array with few out-of-place elements
    NearlySorted,
    /// Reverse sorted array
    Reversed,
    /// Array with few unique values
    FewUnique,
    /// Already sorted array (best case)
    Sorted,
    /// Array with duplicates
    WithDuplicates,
}

impl Default for Distribution {
    fn default() -> Self {
        Distribution::Shuffled
    }
}

/// Fairness model configuration
#[derive(Debug, Clone, PartialEq)]
pub enum FairnessMode {
    /// Equal comparison budget for all algorithms
    ComparisonBudget { k: usize },
    /// Weighted fairness based on algorithm complexity
    Weighted { alpha: f32, beta: f32 },
    /// Wall-clock time based fairness
    WallTime { slice_ms: u64 },
    /// Adaptive fairness that learns algorithm efficiency
    Adaptive { learning_rate: f32 },
    /// Equal steps (one operation per algorithm)
    EqualSteps,
}

impl Default for FairnessMode {
    fn default() -> Self {
        FairnessMode::ComparisonBudget { k: 10 }
    }
}

/// Complete configuration for a sorting race run
#[derive(Debug, Clone)]
pub struct RunConfiguration {
    /// Size of the array to sort
    pub array_size: usize,
    /// Distribution pattern for array generation
    pub distribution: Distribution,
    /// Random seed for deterministic generation
    pub seed: u64,
    /// Fairness model to use
    pub fairness_mode: FairnessMode,
    /// Target frames per second for visualization
    pub target_fps: u32,
}

impl Default for RunConfiguration {
    fn default() -> Self {
        Self {
            array_size: 100,
            distribution: Distribution::default(),
            seed: 42,
            fairness_mode: FairnessMode::default(),
            target_fps: 30,
        }
    }
}

impl RunConfiguration {
    /// Create a new run configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set array size
    pub fn with_array_size(mut self, size: usize) -> Self {
        self.array_size = size;
        self
    }

    /// Set distribution pattern
    pub fn with_distribution(mut self, distribution: Distribution) -> Self {
        self.distribution = distribution;
        self
    }

    /// Set random seed
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    /// Set fairness mode
    pub fn with_fairness_mode(mut self, fairness_mode: FairnessMode) -> Self {
        self.fairness_mode = fairness_mode;
        self
    }

    /// Set target FPS
    pub fn with_target_fps(mut self, fps: u32) -> Self {
        self.target_fps = fps;
        self
    }

    /// Validate configuration parameters
    pub fn validate(&self) -> Result<(), String> {
        if self.array_size == 0 {
            return Err("Array size must be greater than 0".to_string());
        }

        if self.target_fps == 0 {
            return Err("Target FPS must be greater than 0".to_string());
        }

        match &self.fairness_mode {
            FairnessMode::ComparisonBudget { k } => {
                if *k == 0 {
                    return Err("Comparison budget must be greater than 0".to_string());
                }
            }
            FairnessMode::WallTime { slice_ms } => {
                if *slice_ms == 0 {
                    return Err("Wall time limit must be greater than 0".to_string());
                }
            }
            FairnessMode::Weighted { alpha, beta } => {
                if *alpha < 0.0 || *beta < 0.0 {
                    return Err("Weights must be non-negative".to_string());
                }
            }
            FairnessMode::Adaptive { learning_rate } => {
                if *learning_rate < 0.0 || *learning_rate > 1.0 {
                    return Err("Learning rate must be between 0.0 and 1.0".to_string());
                }
            }
            FairnessMode::EqualSteps => {}
        }

        Ok(())
    }
}

/// Configuration for visual appearance
#[derive(Debug, Clone)]
pub struct VisualConfiguration {
    /// Width of visualization window
    pub window_width: u32,
    /// Height of visualization window
    pub window_height: u32,
    /// Whether to show comparison highlights
    pub show_comparisons: bool,
    /// Whether to show algorithm names
    pub show_names: bool,
    /// Whether to show performance metrics
    pub show_metrics: bool,
    /// Animation speed multiplier
    pub animation_speed: f32,
}

impl Default for VisualConfiguration {
    fn default() -> Self {
        Self {
            window_width: 800,
            window_height: 600,
            show_comparisons: true,
            show_names: true,
            show_metrics: true,
            animation_speed: 1.0,
        }
    }
}
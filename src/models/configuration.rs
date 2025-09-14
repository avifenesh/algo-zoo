//! Configuration state for interactive terminal interface

use crate::models::config::{Distribution, FairnessMode};
use anyhow::{Result, anyhow};

/// Represents current user selections for interactive configuration
#[derive(Debug, Clone, PartialEq)]
pub struct ConfigurationState {
    /// Number of elements to sort (10-1000)
    pub array_size: u32,
    /// Data pattern selection
    pub distribution: DistributionType,
    /// Algorithm fairness strategy
    pub fairness_mode: FairnessMode,
    /// Budget parameter for comparison fairness
    pub budget: Option<u32>,
    /// Alpha parameter for weighted fairness
    pub alpha: Option<f32>,
    /// Beta parameter for weighted fairness
    pub beta: Option<f32>,
    /// Learning rate for adaptive fairness
    pub learning_rate: Option<f32>,
}

/// Distribution types for interactive configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DistributionType {
    Shuffled,
    Reversed,
    NearlySorted,
    FewUnique,
}

impl ConfigurationState {
    /// Create a new configuration state with default values
    pub fn new() -> Self {
        Self {
            array_size: 100,
            distribution: DistributionType::Shuffled,
            fairness_mode: FairnessMode::WallTime { slice_ms: 50 },
            budget: None,
            alpha: None,
            beta: None,
            learning_rate: None,
        }
    }

    /// Create a default valid configuration for testing
    pub fn default_valid() -> Self {
        Self::new()
    }

    /// Create ConfigurationState from RunConfiguration for backwards compatibility
    pub fn from_run_config(config: &crate::models::config::RunConfiguration) -> Self {
        Self {
            array_size: config.array_size as u32,
            distribution: config.distribution.clone().into(),
            fairness_mode: config.fairness_mode.clone(),
            budget: None,
            alpha: None,
            beta: None,
            learning_rate: None,
        }
    }

    /// Get available array sizes for interactive selection
    pub fn get_available_array_sizes() -> Vec<u32> {
        vec![10, 25, 50, 100, 200, 500, 1000]
    }

    /// Get available distribution types
    pub fn get_available_distributions() -> Vec<DistributionType> {
        vec![
            DistributionType::Shuffled,
            DistributionType::Reversed,
            DistributionType::NearlySorted,
            DistributionType::FewUnique,
        ]
    }

    /// Get available fairness modes
    pub fn get_available_fairness_modes() -> Vec<FairnessMode> {
        vec![
            FairnessMode::ComparisonBudget { k: 16 },
            FairnessMode::Weighted { alpha: 2.0, beta: 0.5 },
            FairnessMode::WallTime { slice_ms: 50 },
            FairnessMode::Adaptive { learning_rate: 0.3 },
        ]
    }

    /// Check if current fairness mode requires budget parameter
    pub fn requires_budget_parameter(&self) -> bool {
        matches!(self.fairness_mode, FairnessMode::ComparisonBudget { .. })
    }

    /// Check if current fairness mode requires weighted parameters (alpha/beta)
    pub fn requires_weighted_parameters(&self) -> bool {
        matches!(self.fairness_mode, FairnessMode::Weighted { .. })
    }

    /// Check if current fairness mode requires learning rate parameter
    pub fn requires_learning_rate_parameter(&self) -> bool {
        matches!(self.fairness_mode, FairnessMode::Adaptive { .. })
    }

    /// Validate the current configuration
    pub fn validate(&self) -> Result<()> {
        // Validate array size
        if self.array_size < 10 || self.array_size > 1000 {
            return Err(anyhow!("Array size must be between 10 and 1000, got {}", self.array_size));
        }

        // Validate fairness mode parameters
        match &self.fairness_mode {
            FairnessMode::ComparisonBudget { k } => {
                if *k == 0 {
                    return Err(anyhow!("Budget parameter must be greater than 0, got {}", k));
                }
            },
            FairnessMode::Weighted { alpha, beta } => {
                if *alpha <= 0.0 {
                    return Err(anyhow!("Alpha parameter must be greater than 0.0, got {}", alpha));
                }
                if *beta <= 0.0 {
                    return Err(anyhow!("Beta parameter must be greater than 0.0, got {}", beta));
                }
            },
            FairnessMode::Adaptive { learning_rate } => {
                if *learning_rate < 0.1 || *learning_rate > 1.0 {
                    return Err(anyhow!("Learning rate must be between 0.1 and 1.0, got {}", learning_rate));
                }
            },
            FairnessMode::WallTime { slice_ms } => {
                if *slice_ms == 0 {
                    return Err(anyhow!("Wall time slice must be greater than 0, got {}", slice_ms));
                }
            },
            _ => {}, // Other fairness modes don't require validation
        }

        Ok(())
    }

    /// Check if the configuration is valid
    pub fn is_valid(&self) -> bool {
        self.validate().is_ok()
    }

    /// Check if the configuration is invalid
    pub fn is_invalid(&self) -> bool {
        !self.is_valid()
    }

    /// Set array size with validation
    pub fn set_array_size(&mut self, size: u32) -> Result<()> {
        if size < 10 || size > 1000 {
            return Err(anyhow!("Array size must be between 10 and 1000, got {}", size));
        }
        self.array_size = size;
        Ok(())
    }

    /// Set fairness mode and clear incompatible parameters
    pub fn set_fairness_mode(&mut self, mode: FairnessMode) {
        self.fairness_mode = mode.clone();

        // Clear parameters that don't apply to the new mode
        match &mode {
            FairnessMode::ComparisonBudget { .. } => {
                self.alpha = None;
                self.beta = None;
                self.learning_rate = None;
            },
            FairnessMode::Weighted { .. } => {
                self.budget = None;
                self.learning_rate = None;
            },
            FairnessMode::Adaptive { .. } => {
                self.budget = None;
                self.alpha = None;
                self.beta = None;
            },
            FairnessMode::WallTime { .. } => {
                self.budget = None;
                self.alpha = None;
                self.beta = None;
                self.learning_rate = None;
            },
            _ => {
                // Clear all optional parameters
                self.budget = None;
                self.alpha = None;
                self.beta = None;
                self.learning_rate = None;
            },
        }
    }

    /// Convert to the legacy Distribution enum for compatibility
    pub fn to_legacy_distribution(&self) -> Distribution {
        match self.distribution {
            DistributionType::Shuffled => Distribution::Shuffled,
            DistributionType::Reversed => Distribution::Reversed,
            DistributionType::NearlySorted => Distribution::NearlySorted,
            DistributionType::FewUnique => Distribution::FewUnique,
        }
    }
}

impl Default for ConfigurationState {
    fn default() -> Self {
        Self::new()
    }
}

impl From<DistributionType> for Distribution {
    fn from(dist_type: DistributionType) -> Self {
        match dist_type {
            DistributionType::Shuffled => Distribution::Shuffled,
            DistributionType::Reversed => Distribution::Reversed,
            DistributionType::NearlySorted => Distribution::NearlySorted,
            DistributionType::FewUnique => Distribution::FewUnique,
        }
    }
}

impl From<Distribution> for DistributionType {
    fn from(distribution: Distribution) -> Self {
        match distribution {
            Distribution::Shuffled => DistributionType::Shuffled,
            Distribution::Reversed => DistributionType::Reversed,
            Distribution::NearlySorted => DistributionType::NearlySorted,
            Distribution::FewUnique => DistributionType::FewUnique,
            Distribution::Sorted => DistributionType::Shuffled,     // Map to closest equivalent
            Distribution::WithDuplicates => DistributionType::FewUnique,  // Map to closest equivalent
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_configuration_state_creation() {
        let config = ConfigurationState::new();
        assert_eq!(config.array_size, 100);
        assert_eq!(config.distribution, DistributionType::Shuffled);
        assert!(matches!(config.fairness_mode, FairnessMode::WallTime { .. }));
        assert_eq!(config.budget, None);
        assert_eq!(config.alpha, None);
        assert_eq!(config.beta, None);
        assert_eq!(config.learning_rate, None);
    }

    #[test]
    fn test_array_size_validation() {
        let mut config = ConfigurationState::new();
        
        // Valid sizes
        assert!(config.set_array_size(50).is_ok());
        assert_eq!(config.array_size, 50);
        
        assert!(config.set_array_size(10).is_ok());
        assert!(config.set_array_size(1000).is_ok());
        
        // Invalid sizes
        assert!(config.set_array_size(5).is_err());
        assert!(config.set_array_size(1001).is_err());
    }

    #[test]
    fn test_fairness_mode_parameter_requirements() {
        let mut config = ConfigurationState::new();
        
        config.set_fairness_mode(FairnessMode::ComparisonBudget { k: 16 });
        assert!(config.requires_budget_parameter());
        assert!(!config.requires_weighted_parameters());
        assert!(!config.requires_learning_rate_parameter());
        
        config.set_fairness_mode(FairnessMode::Weighted { alpha: 2.0, beta: 0.5 });
        assert!(!config.requires_budget_parameter());
        assert!(config.requires_weighted_parameters());
        assert!(!config.requires_learning_rate_parameter());
        
        config.set_fairness_mode(FairnessMode::Adaptive { learning_rate: 0.3 });
        assert!(!config.requires_budget_parameter());
        assert!(!config.requires_weighted_parameters());
        assert!(config.requires_learning_rate_parameter());
    }

    #[test]
    fn test_configuration_validation() {
        let mut config = ConfigurationState::new();
        assert!(config.is_valid());
        
        // Invalid array size
        config.array_size = 5;
        assert!(config.is_invalid());
        
        config.array_size = 100; // Reset to valid
        assert!(config.is_valid());
        
        // Invalid fairness parameters
        config.fairness_mode = FairnessMode::ComparisonBudget { k: 0 };
        assert!(config.is_invalid());
    }

    #[test]
    fn test_available_options() {
        let sizes = ConfigurationState::get_available_array_sizes();
        assert_eq!(sizes, vec![10, 25, 50, 100, 200, 500, 1000]);
        
        let distributions = ConfigurationState::get_available_distributions();
        assert_eq!(distributions.len(), 4);
        assert!(distributions.contains(&DistributionType::Shuffled));
        
        let fairness_modes = ConfigurationState::get_available_fairness_modes();
        assert_eq!(fairness_modes.len(), 4);
    }

    #[test]
    fn test_distribution_conversion() {
        let dist_type = DistributionType::Reversed;
        let legacy_dist: Distribution = dist_type.into();
        assert_eq!(legacy_dist, Distribution::Reversed);
        
        let config = ConfigurationState::new();
        let legacy = config.to_legacy_distribution();
        assert_eq!(legacy, Distribution::Shuffled);
    }
}
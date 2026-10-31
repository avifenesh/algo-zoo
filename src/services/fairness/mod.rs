//! Fairness model implementations

pub mod adaptive;
pub mod comparison;
pub mod walltime;
pub mod weighted;

pub use adaptive::AdaptiveFairness;
pub use comparison::ComparisonFairness;
pub use walltime::{AdaptiveWallTimeFairness, WallTimeFairness};
pub use weighted::{PerformanceWeightedFairness, WeightedFairness};

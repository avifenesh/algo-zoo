//! Fairness model implementations

pub mod comparison;
pub mod weighted;
pub mod walltime;
pub mod adaptive;

pub use comparison::ComparisonFairness;
pub use weighted::{WeightedFairness, PerformanceWeightedFairness};
pub use walltime::{WallTimeFairness, AdaptiveWallTimeFairness};
pub use adaptive::AdaptiveFairness;
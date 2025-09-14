//! Snapshot functionality for algorithm states

use crate::models::traits::{Sorter, Telemetry};
use crate::models::metrics::MetricsSnapshot;
use std::time::SystemTime;

/// A snapshot of the complete algorithm race state
#[derive(Debug, Clone)]
pub struct RaceSnapshot {
    /// Timestamp when snapshot was taken
    pub timestamp: SystemTime,
    /// Snapshots of all algorithms
    pub algorithm_snapshots: Vec<AlgorithmSnapshot>,
    /// Current step number
    pub step: usize,
    /// Whether the race is complete
    pub race_complete: bool,
}

/// Snapshot of a single algorithm's state
#[derive(Debug, Clone)]
pub struct AlgorithmSnapshot {
    /// Algorithm name
    pub name: String,
    /// Current array state
    pub array_state: Vec<i32>,
    /// Telemetry data
    pub telemetry: Telemetry,
    /// Whether algorithm is complete
    pub is_complete: bool,
    /// Metrics snapshot
    pub metrics: MetricsSnapshot,
}

/// Service for creating and managing snapshots
#[derive(Debug, Default)]
pub struct SnapshotService {
    snapshots: Vec<RaceSnapshot>,
    max_snapshots: usize,
}

impl SnapshotService {
    /// Create a new snapshot service
    pub fn new(max_snapshots: usize) -> Self {
        Self {
            snapshots: Vec::new(),
            max_snapshots: max_snapshots.max(1),
        }
    }

    /// Take a snapshot of the current race state
    pub fn take_snapshot(&mut self, algorithms: &[Box<dyn Sorter>], step: usize) -> &RaceSnapshot {
        let race_complete = algorithms.iter().all(|alg| alg.is_complete());
        
        let algorithm_snapshots = algorithms
            .iter()
            .map(|algorithm| {
                let telemetry = algorithm.get_telemetry();
                let metrics = crate::models::metrics::Metrics {
                    comparisons: telemetry.total_comparisons,
                    moves: telemetry.total_moves,
                    execution_time_us: 0, // Stub value
                    peak_memory_bytes: telemetry.memory_peak,
                    current_memory_bytes: telemetry.memory_current,
                    steps: step,
                    array_accesses: telemetry.total_comparisons + telemetry.total_moves,
                    recursive_calls: 0, // Stub value
                };

                AlgorithmSnapshot {
                    name: algorithm.name().to_string(),
                    array_state: algorithm.get_array().to_vec(),
                    telemetry,
                    is_complete: algorithm.is_complete(),
                    metrics: MetricsSnapshot::new(
                        metrics,
                        SystemTime::now()
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap()
                            .as_micros() as u64,
                        algorithm.name().to_string(),
                    ),
                }
            })
            .collect();

        let snapshot = RaceSnapshot {
            timestamp: SystemTime::now(),
            algorithm_snapshots,
            step,
            race_complete,
        };

        // Add snapshot and maintain max limit
        self.snapshots.push(snapshot);
        if self.snapshots.len() > self.max_snapshots {
            self.snapshots.remove(0);
        }

        self.snapshots.last().unwrap()
    }

    /// Get all snapshots
    pub fn get_snapshots(&self) -> &[RaceSnapshot] {
        &self.snapshots
    }

    /// Get the most recent snapshot
    pub fn get_latest_snapshot(&self) -> Option<&RaceSnapshot> {
        self.snapshots.last()
    }

    /// Get snapshot by index
    pub fn get_snapshot(&self, index: usize) -> Option<&RaceSnapshot> {
        self.snapshots.get(index)
    }

    /// Clear all snapshots
    pub fn clear(&mut self) {
        self.snapshots.clear();
    }

    /// Get number of snapshots
    pub fn len(&self) -> usize {
        self.snapshots.len()
    }

    /// Check if no snapshots exist
    pub fn is_empty(&self) -> bool {
        self.snapshots.is_empty()
    }

    /// Set maximum number of snapshots to keep
    pub fn set_max_snapshots(&mut self, max: usize) {
        self.max_snapshots = max.max(1);
        
        // Trim existing snapshots if necessary
        while self.snapshots.len() > self.max_snapshots {
            self.snapshots.remove(0);
        }
    }

    /// Get maximum number of snapshots
    pub fn get_max_snapshots(&self) -> usize {
        self.max_snapshots
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::sorters::bubble::BubbleSort;

    #[test]
    fn test_snapshot_service_creation() {
        let service = SnapshotService::new(10);
        assert_eq!(service.len(), 0);
        assert!(service.is_empty());
        assert_eq!(service.get_max_snapshots(), 10);
    }

    #[test]
    fn test_take_snapshot() {
        let mut service = SnapshotService::new(5);
        let mut bubble = BubbleSort::new();
        bubble.reset(vec![3, 1, 2]);

        let algorithms: Vec<Box<dyn Sorter>> = vec![Box::new(bubble)];
        
        let snapshot = service.take_snapshot(&algorithms, 1);
        assert_eq!(snapshot.step, 1);
        assert_eq!(snapshot.algorithm_snapshots.len(), 1);
        assert_eq!(service.len(), 1);
    }

    #[test]
    fn test_max_snapshots_limit() {
        let mut service = SnapshotService::new(3);
        let bubble = BubbleSort::new();
        let algorithms: Vec<Box<dyn Sorter>> = vec![Box::new(bubble)];

        // Take more snapshots than the limit
        for i in 0..5 {
            service.take_snapshot(&algorithms, i);
        }

        assert_eq!(service.len(), 3); // Should not exceed max
        
        // Should have the last 3 snapshots (steps 2, 3, 4)
        let snapshots = service.get_snapshots();
        assert_eq!(snapshots[0].step, 2);
        assert_eq!(snapshots[1].step, 3);
        assert_eq!(snapshots[2].step, 4);
    }
}
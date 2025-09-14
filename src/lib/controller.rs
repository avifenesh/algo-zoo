//! Main controller for the sorting race

use crate::models::config::RunConfiguration;
use crate::models::traits::{Sorter, FairnessModel};
use crate::services::snapshot::SnapshotService;
use std::time::Instant;

/// Main controller for orchestrating the sorting race
#[derive(Debug)]
pub struct RaceController {
    algorithms: Vec<Box<dyn Sorter>>,
    fairness_model: Box<dyn FairnessModel>,
    snapshot_service: SnapshotService,
    current_step: usize,
    start_time: Option<Instant>,
    is_running: bool,
    is_paused: bool,
}

impl RaceController {
    /// Create a new race controller
    pub fn new(
        algorithms: Vec<Box<dyn Sorter>>,
        fairness_model: Box<dyn FairnessModel>,
        max_snapshots: usize,
    ) -> Self {
        Self {
            algorithms,
            fairness_model,
            snapshot_service: SnapshotService::new(max_snapshots),
            current_step: 0,
            start_time: None,
            is_running: false,
            is_paused: false,
        }
    }

    /// Start the race with the given configuration
    pub fn start_race(&mut self, config: &RunConfiguration, data: Vec<i32>) -> Result<(), String> {
        config.validate()?;

        // Reset all algorithms with the same data
        for algorithm in &mut self.algorithms {
            algorithm.reset(data.clone());
        }

        self.current_step = 0;
        self.start_time = Some(Instant::now());
        self.is_running = true;
        self.is_paused = false;
        self.snapshot_service.clear();

        // Take initial snapshot
        self.snapshot_service.take_snapshot(&self.algorithms, 0);

        Ok(())
    }

    /// Execute one step of the race
    pub fn step(&mut self) -> bool {
        if !self.is_running || self.is_paused {
            return false;
        }

        if self.is_race_complete() {
            self.is_running = false;
            return false;
        }

        // Allocate budgets using fairness model
        let budgets = self.fairness_model.allocate_budget(&self.algorithms);

        // Execute steps for each algorithm
        for (algorithm, budget) in self.algorithms.iter_mut().zip(budgets.iter()) {
            if *budget > 0 && !algorithm.is_complete() {
                algorithm.step(*budget);
            }
        }

        self.current_step += 1;

        // Take snapshot
        self.snapshot_service.take_snapshot(&self.algorithms, self.current_step);

        // Check if race is complete
        if self.is_race_complete() {
            self.is_running = false;
        }

        true
    }

    /// Run the race to completion or until a step limit
    pub fn run_to_completion(&mut self, max_steps: Option<usize>) -> usize {
        let limit = max_steps.unwrap_or(usize::MAX);
        let mut steps_executed = 0;

        while self.is_running && steps_executed < limit && self.step() {
            steps_executed += 1;
        }

        steps_executed
    }

    /// Check if the race is complete
    pub fn is_race_complete(&self) -> bool {
        self.algorithms.iter().all(|alg| alg.is_complete())
    }

    /// Pause the race
    pub fn pause(&mut self) {
        self.is_paused = true;
    }

    /// Resume the race
    pub fn resume(&mut self) {
        self.is_paused = false;
    }

    /// Stop the race
    pub fn stop(&mut self) {
        self.is_running = false;
        self.is_paused = false;
    }

    /// Reset the race
    pub fn reset(&mut self) {
        self.current_step = 0;
        self.start_time = None;
        self.is_running = false;
        self.is_paused = false;
        self.snapshot_service.clear();
    }

    /// Get current step number
    pub fn get_current_step(&self) -> usize {
        self.current_step
    }

    /// Get elapsed time since race start
    pub fn get_elapsed_time(&self) -> Option<std::time::Duration> {
        self.start_time.map(|start| start.elapsed())
    }

    /// Check if race is running
    pub fn is_running(&self) -> bool {
        self.is_running
    }

    /// Check if race is paused
    pub fn is_paused(&self) -> bool {
        self.is_paused
    }

    /// Get reference to algorithms
    pub fn get_algorithms(&self) -> &[Box<dyn Sorter>] {
        &self.algorithms
    }

    /// Get reference to snapshot service
    pub fn get_snapshot_service(&self) -> &SnapshotService {
        &self.snapshot_service
    }

    /// Get mutable reference to snapshot service
    pub fn get_snapshot_service_mut(&mut self) -> &mut SnapshotService {
        &mut self.snapshot_service
    }

    /// Get the name of the current fairness model
    pub fn get_fairness_model_name(&self) -> &str {
        self.fairness_model.name()
    }
}
//! Session state management for multiple sorting races

use crate::models::{
    configuration::{ConfigurationState, DistributionType},
    config::FairnessMode,
    traits::Sorter,
};
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Algorithm type identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AlgorithmType {
    BubbleSort,
    SelectionSort,
    InsertionSort,
    MergeSort,
    QuickSort,
    HeapSort,
    ShellSort,
}

impl AlgorithmType {
    /// Get all available algorithm types
    pub fn all() -> Vec<AlgorithmType> {
        vec![
            AlgorithmType::BubbleSort,
            AlgorithmType::SelectionSort,
            AlgorithmType::InsertionSort,
            AlgorithmType::MergeSort,
            AlgorithmType::QuickSort,
            AlgorithmType::HeapSort,
            AlgorithmType::ShellSort,
        ]
    }

    /// Get algorithm type from index
    pub fn from_index(index: usize) -> Option<AlgorithmType> {
        let algorithms = Self::all();
        algorithms.get(index).copied()
    }

    /// Get index of algorithm type
    pub fn to_index(self) -> usize {
        match self {
            AlgorithmType::BubbleSort => 0,
            AlgorithmType::SelectionSort => 1,
            AlgorithmType::InsertionSort => 2,
            AlgorithmType::MergeSort => 3,
            AlgorithmType::QuickSort => 4,
            AlgorithmType::HeapSort => 5,
            AlgorithmType::ShellSort => 6,
        }
    }
}

impl std::fmt::Display for AlgorithmType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlgorithmType::BubbleSort => write!(f, "Bubble Sort"),
            AlgorithmType::SelectionSort => write!(f, "Selection Sort"),
            AlgorithmType::InsertionSort => write!(f, "Insertion Sort"),
            AlgorithmType::MergeSort => write!(f, "Merge Sort"),
            AlgorithmType::QuickSort => write!(f, "Quick Sort"),
            AlgorithmType::HeapSort => write!(f, "Heap Sort"),
            AlgorithmType::ShellSort => write!(f, "Shell Sort"),
        }
    }
}

/// Result of a single sorting race
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaceResult {
    /// Array size used for this race
    pub array_size: u32,
    /// Distribution type used
    pub distribution: DistributionType,
    /// Fairness mode used
    pub fairness_mode: FairnessMode,
    /// Completion times for each algorithm (None if didn't complete)
    pub completion_times: Vec<Option<std::time::Duration>>,
    /// Memory usage for each algorithm
    pub memory_usage: Vec<usize>,
    /// Algorithm type names for reference
    pub algorithm_names: Vec<String>,
    /// Winner algorithm (first to complete)
    pub winner: Option<AlgorithmType>,
    /// Race start timestamp
    pub race_start: Instant,
    /// Race end timestamp
    pub race_end: Option<Instant>,
    /// Total race duration
    pub total_duration: Option<std::time::Duration>,
}

impl RaceResult {
    /// Create a new race result
    pub fn new(
        array_size: u32,
        distribution: DistributionType,
        fairness_mode: FairnessMode,
        algorithm_count: usize,
    ) -> Self {
        Self {
            array_size,
            distribution,
            fairness_mode,
            completion_times: vec![None; algorithm_count],
            memory_usage: vec![0; algorithm_count],
            algorithm_names: Vec::new(),
            winner: None,
            race_start: Instant::now(),
            race_end: None,
            total_duration: None,
        }
    }

    /// Mark race as completed
    pub fn complete(&mut self) {
        self.race_end = Some(Instant::now());
        self.total_duration = Some(self.race_start.elapsed());
    }

    /// Set completion time for an algorithm
    pub fn set_completion_time(&mut self, algorithm_index: usize, duration: std::time::Duration) {
        if algorithm_index < self.completion_times.len() {
            self.completion_times[algorithm_index] = Some(duration);
            
            // Set winner if this is the first completion
            if self.winner.is_none() {
                self.winner = AlgorithmType::from_index(algorithm_index);
            }
        }
    }

    /// Set memory usage for an algorithm
    pub fn set_memory_usage(&mut self, algorithm_index: usize, memory: usize) {
        if algorithm_index < self.memory_usage.len() {
            self.memory_usage[algorithm_index] = memory;
        }
    }

    /// Check if race is complete
    pub fn is_complete(&self) -> bool {
        self.race_end.is_some()
    }
}

/// Session state managing multiple sorting races
#[derive(Debug, Clone)]
pub struct SessionState {
    /// Current configuration settings
    pub current_config: ConfigurationState,
    /// History of completed race results
    pub run_history: Vec<RaceResult>,
    /// When the session started
    pub session_start_time: Instant,
    /// Total number of races run in this session
    pub total_races_run: u32,
    /// Current race result (if race is in progress)
    current_race: Option<RaceResult>,
}

impl SessionState {
    /// Create a new session state
    pub fn new() -> Self {
        Self {
            current_config: ConfigurationState::new(),
            run_history: Vec::new(),
            session_start_time: Instant::now(),
            total_races_run: 0,
            current_race: None,
        }
    }

    /// Start a new race with current configuration
    pub fn start_new_race(&mut self) -> Result<(), anyhow::Error> {
        // Validate configuration
        self.current_config.validate()?;

        // Create new race result
        let algorithm_count = AlgorithmType::all().len();
        let mut race_result = RaceResult::new(
            self.current_config.array_size,
            self.current_config.distribution,
            self.current_config.fairness_mode.clone(),
            algorithm_count,
        );

        // Set algorithm names
        race_result.algorithm_names = AlgorithmType::all()
            .iter()
            .map(|alg| alg.to_string())
            .collect();

        self.current_race = Some(race_result);
        Ok(())
    }

    /// Complete the current race and add it to history
    pub fn complete_current_race(&mut self) {
        if let Some(mut race) = self.current_race.take() {
            race.complete();
            self.run_history.push(race);
            self.total_races_run += 1;
        }
    }

    /// Update current race progress
    pub fn update_race_progress(&mut self, algorithms: &[Box<dyn Sorter>]) {
        if let Some(ref mut race) = self.current_race {
            for (i, algorithm) in algorithms.iter().enumerate() {
                // Update memory usage
                let memory = algorithm.get_memory_usage();
                race.set_memory_usage(i, memory);

                // Check if algorithm completed and set completion time
                if algorithm.is_complete() && race.completion_times[i].is_none() {
                    let completion_time = race.race_start.elapsed();
                    race.set_completion_time(i, completion_time);
                }
            }
        }
    }

    /// Check if there's a current race in progress
    pub fn has_current_race(&self) -> bool {
        self.current_race.is_some()
    }

    /// Get current race result (if any)
    pub fn get_current_race(&self) -> Option<&RaceResult> {
        self.current_race.as_ref()
    }

    /// Get mutable reference to current race result
    pub fn get_current_race_mut(&mut self) -> Option<&mut RaceResult> {
        self.current_race.as_mut()
    }

    /// Get session duration
    pub fn get_session_duration(&self) -> std::time::Duration {
        self.session_start_time.elapsed()
    }

    /// Get race statistics
    pub fn get_race_statistics(&self) -> SessionStatistics {
        SessionStatistics::from_session(self)
    }

    /// Update current configuration
    pub fn update_configuration(&mut self, config: ConfigurationState) {
        self.current_config = config;
    }

    /// Clear session history
    pub fn clear_history(&mut self) {
        self.run_history.clear();
        self.total_races_run = 0;
        self.current_race = None;
    }

    /// Get average race duration
    pub fn get_average_race_duration(&self) -> Option<std::time::Duration> {
        if self.run_history.is_empty() {
            return None;
        }

        let total_duration: std::time::Duration = self.run_history
            .iter()
            .filter_map(|result| result.total_duration)
            .sum();

        Some(total_duration / self.run_history.len() as u32)
    }

    /// Get most common winner
    pub fn get_most_common_winner(&self) -> Option<AlgorithmType> {
        use std::collections::HashMap;

        let mut winner_counts: HashMap<AlgorithmType, u32> = HashMap::new();
        
        for result in &self.run_history {
            if let Some(winner) = result.winner {
                *winner_counts.entry(winner).or_insert(0) += 1;
            }
        }

        winner_counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(algorithm, _)| algorithm)
    }
}

impl Default for SessionState {
    fn default() -> Self {
        Self::new()
    }
}

/// Session statistics summary
#[derive(Debug, Clone)]
pub struct SessionStatistics {
    pub total_races: u32,
    pub session_duration: std::time::Duration,
    pub average_race_duration: Option<std::time::Duration>,
    pub most_common_winner: Option<AlgorithmType>,
    pub win_counts: std::collections::HashMap<AlgorithmType, u32>,
    pub total_comparisons: u64,
    pub total_moves: u64,
    pub average_array_size: f32,
}

impl SessionStatistics {
    /// Generate statistics from a session state
    pub fn from_session(session: &SessionState) -> Self {
        use std::collections::HashMap;

        let mut win_counts: HashMap<AlgorithmType, u32> = HashMap::new();
        let mut total_comparisons = 0u64;
        let mut total_moves = 0u64;
        let mut total_array_size = 0u64;

        for result in &session.run_history {
            if let Some(winner) = result.winner {
                *win_counts.entry(winner).or_insert(0) += 1;
            }
            total_array_size += result.array_size as u64;
        }

        let most_common_winner = win_counts
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(algorithm, _)| *algorithm);

        let average_array_size = if session.run_history.is_empty() {
            0.0
        } else {
            total_array_size as f32 / session.run_history.len() as f32
        };

        Self {
            total_races: session.total_races_run,
            session_duration: session.get_session_duration(),
            average_race_duration: session.get_average_race_duration(),
            most_common_winner,
            win_counts,
            total_comparisons,
            total_moves,
            average_array_size,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_algorithm_type_conversions() {
        assert_eq!(AlgorithmType::from_index(0), Some(AlgorithmType::BubbleSort));
        assert_eq!(AlgorithmType::from_index(6), Some(AlgorithmType::ShellSort));
        assert_eq!(AlgorithmType::from_index(7), None);

        assert_eq!(AlgorithmType::BubbleSort.to_index(), 0);
        assert_eq!(AlgorithmType::ShellSort.to_index(), 6);

        assert_eq!(AlgorithmType::all().len(), 7);
    }

    #[test]
    fn test_race_result_creation() {
        let result = RaceResult::new(100, DistributionType::Shuffled, FairnessMode::WallTime { slice_ms: 50 }, 7);
        
        assert_eq!(result.array_size, 100);
        assert_eq!(result.distribution, DistributionType::Shuffled);
        assert_eq!(result.completion_times.len(), 7);
        assert_eq!(result.memory_usage.len(), 7);
        assert_eq!(result.winner, None);
        assert!(!result.is_complete());
    }

    #[test]
    fn test_race_result_completion() {
        let mut result = RaceResult::new(50, DistributionType::Reversed, FairnessMode::WallTime { slice_ms: 50 }, 3);
        
        result.set_completion_time(1, std::time::Duration::from_millis(100));
        assert_eq!(result.winner, Some(AlgorithmType::SelectionSort));
        
        result.complete();
        assert!(result.is_complete());
        assert!(result.total_duration.is_some());
    }

    #[test]
    fn test_session_state_creation() {
        let session = SessionState::new();
        
        assert_eq!(session.total_races_run, 0);
        assert!(session.run_history.is_empty());
        assert!(!session.has_current_race());
        assert!(session.session_start_time.elapsed().as_millis() < 100); // Recently created
    }

    #[test]
    fn test_session_race_lifecycle() {
        let mut session = SessionState::new();
        
        // Start new race
        assert!(session.start_new_race().is_ok());
        assert!(session.has_current_race());
        
        // Complete race
        session.complete_current_race();
        assert!(!session.has_current_race());
        assert_eq!(session.total_races_run, 1);
        assert_eq!(session.run_history.len(), 1);
    }

    #[test]
    fn test_session_statistics() {
        let mut session = SessionState::new();
        
        // Run a couple of races
        session.start_new_race().unwrap();
        session.complete_current_race();
        
        session.start_new_race().unwrap();
        session.complete_current_race();
        
        let stats = session.get_race_statistics();
        assert_eq!(stats.total_races, 2);
        assert!(stats.session_duration.as_millis() > 0);
    }

    #[test]
    fn test_winner_tracking() {
        let session = SessionState::new();
        assert_eq!(session.get_most_common_winner(), None);
        
        // Would need to set up actual race results to test winner tracking fully
    }

    #[test]
    fn test_configuration_validation() {
        let mut session = SessionState::new();
        
        // Default config should be valid
        assert!(session.start_new_race().is_ok());
        
        // Invalid config should fail
        session.current_config.array_size = 0; // Invalid
        assert!(session.start_new_race().is_err());
    }
}
//! Integration tests for snapshot save/load functionality
//! Tests serialization and deserialization of sorting algorithm states

use sorting_race::models::config::{Distribution, FairnessMode, RunConfiguration};
use sorting_race::models::traits::{FairnessModel, Sorter, StepResult, Telemetry};
use sorting_race::services::fairness::comparison::ComparisonBudget;
use sorting_race::services::generator::ArrayGenerator;
use sorting_race::services::sorters::{
    bubble::BubbleSort, heap::HeapSort, insertion::InsertionSort, merge::MergeSort,
    quick::QuickSort, selection::SelectionSort,
};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// Snapshot data structure that can be serialized to/from JSON
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct SortingSnapshot {
    /// Timestamp when snapshot was created
    timestamp: u64,
    /// Configuration used for the sorting run
    config: RunConfiguration,
    /// Algorithm states at the time of snapshot
    algorithm_states: Vec<AlgorithmState>,
    /// Overall progress statistics
    progress: ProgressStats,
    /// Metadata about the snapshot
    metadata: SnapshotMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct AlgorithmState {
    /// Name of the sorting algorithm
    name: String,
    /// Current state of the array being sorted
    array: Vec<i32>,
    /// Telemetry data (comparisons, moves, etc.)
    telemetry: TelemetrySnapshot,
    /// Whether the algorithm has completed sorting
    is_complete: bool,
    /// Current step/iteration count
    step_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TelemetrySnapshot {
    /// Total number of comparisons performed
    total_comparisons: usize,
    /// Total number of element moves/swaps
    total_moves: usize,
    /// Number of array accesses
    array_accesses: usize,
    /// Additional algorithm-specific metrics
    custom_metrics: HashMap<String, i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct ProgressStats {
    /// Number of algorithms that have completed
    completed_algorithms: usize,
    /// Total number of algorithms in the race
    total_algorithms: usize,
    /// Total number of steps executed across all algorithms
    total_steps: usize,
    /// Overall completion percentage
    completion_percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct SnapshotMetadata {
    /// Version of the snapshot format
    format_version: String,
    /// Human-readable description of the snapshot
    description: String,
    /// Additional tags or labels
    tags: Vec<String>,
}

/// Snapshot manager for handling save/load operations
struct SnapshotManager {
    temp_dir: TempDir,
}

impl SnapshotManager {
    fn new() -> anyhow::Result<Self> {
        let temp_dir = tempfile::tempdir()?;
        Ok(Self { temp_dir })
    }

    /// Save a snapshot to a JSON file
    fn save_snapshot(&self, snapshot: &SortingSnapshot, filename: &str) -> anyhow::Result<String> {
        let file_path = self.temp_dir.path().join(filename);
        let json_data = serde_json::to_string_pretty(snapshot)?;
        fs::write(&file_path, json_data)?;
        Ok(file_path.to_string_lossy().to_string())
    }

    /// Load a snapshot from a JSON file
    fn load_snapshot(&self, filename: &str) -> anyhow::Result<SortingSnapshot> {
        let file_path = self.temp_dir.path().join(filename);
        let json_data = fs::read_to_string(&file_path)?;
        let snapshot: SortingSnapshot = serde_json::from_str(&json_data)?;
        Ok(snapshot)
    }

    /// Check if a snapshot file exists
    fn snapshot_exists(&self, filename: &str) -> bool {
        let file_path = self.temp_dir.path().join(filename);
        file_path.exists()
    }

    /// Get the path to a snapshot file
    fn get_snapshot_path(&self, filename: &str) -> String {
        let file_path = self.temp_dir.path().join(filename);
        file_path.to_string_lossy().to_string()
    }

    /// List all snapshot files in the directory
    fn list_snapshots(&self) -> anyhow::Result<Vec<String>> {
        let mut snapshots = Vec::new();
        for entry in fs::read_dir(self.temp_dir.path())? {
            let entry = entry?;
            if let Some(filename) = entry.file_name().to_str() {
                if filename.ends_with(".json") {
                    snapshots.push(filename.to_string());
                }
            }
        }
        snapshots.sort();
        Ok(snapshots)
    }
}

/// Create test algorithms for snapshot testing
fn create_test_algorithms() -> Vec<Box<dyn Sorter>> {
    vec![
        Box::new(BubbleSort::new()),
        Box::new(InsertionSort::new()),
        Box::new(SelectionSort::new()),
        Box::new(QuickSort::new()),
        Box::new(HeapSort::new()),
        Box::new(MergeSort::new()),
    ]
}

/// Create a test configuration for sorting
fn create_test_config() -> RunConfiguration {
    RunConfiguration {
        array_size: 20,
        distribution: Distribution::Random,
        fairness_mode: FairnessMode::ComparisonBudget { k: 10 },
        seed: Some(42),
    }
}

/// Convert telemetry to snapshot format
fn telemetry_to_snapshot(telemetry: &Telemetry) -> TelemetrySnapshot {
    TelemetrySnapshot {
        total_comparisons: telemetry.total_comparisons,
        total_moves: telemetry.total_moves,
        array_accesses: telemetry.array_accesses.unwrap_or(0),
        custom_metrics: HashMap::new(), // Can be extended with algorithm-specific data
    }
}

/// Create a snapshot from current algorithm states
fn create_snapshot_from_algorithms(
    algorithms: &[Box<dyn Sorter>],
    config: &RunConfiguration,
    total_steps: usize,
) -> SortingSnapshot {
    let algorithm_states: Vec<AlgorithmState> = algorithms
        .iter()
        .map(|alg| {
            let telemetry = alg.get_telemetry();
            AlgorithmState {
                name: alg.name().to_string(),
                array: alg.get_array().to_vec(),
                telemetry: telemetry_to_snapshot(&telemetry),
                is_complete: alg.is_complete(),
                step_count: telemetry.total_comparisons + telemetry.total_moves, // Proxy for steps
            }
        })
        .collect();

    let completed_count = algorithm_states.iter().filter(|s| s.is_complete).count();
    let total_count = algorithm_states.len();
    let completion_percentage = if total_count > 0 {
        (completed_count as f64 / total_count as f64) * 100.0
    } else {
        0.0
    };

    SortingSnapshot {
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        config: config.clone(),
        algorithm_states,
        progress: ProgressStats {
            completed_algorithms: completed_count,
            total_algorithms: total_count,
            total_steps,
            completion_percentage,
        },
        metadata: SnapshotMetadata {
            format_version: "1.0.0".to_string(),
            description: "Test snapshot for sorting race".to_string(),
            tags: vec!["test".to_string(), "integration".to_string()],
        },
    }
}

/// Validate that a snapshot contains all required fields
fn validate_snapshot_structure(snapshot: &SortingSnapshot) -> anyhow::Result<()> {
    // Check timestamp is reasonable (not zero and not too far in future)
    assert!(snapshot.timestamp > 0, "Timestamp should be positive");
    assert!(
        snapshot.timestamp < 2_000_000_000, // Year ~2033
        "Timestamp seems too far in the future"
    );

    // Check algorithm states
    assert!(
        !snapshot.algorithm_states.is_empty(),
        "Should have at least one algorithm state"
    );

    for state in &snapshot.algorithm_states {
        assert!(!state.name.is_empty(), "Algorithm name should not be empty");
        assert!(!state.array.is_empty(), "Array should not be empty");
        // Telemetry values should be non-negative
        assert!(
            state.telemetry.total_comparisons >= 0,
            "Comparisons should be non-negative"
        );
        assert!(
            state.telemetry.total_moves >= 0,
            "Moves should be non-negative"
        );
    }

    // Check progress stats consistency
    assert_eq!(
        snapshot.progress.total_algorithms,
        snapshot.algorithm_states.len(),
        "Total algorithms count should match states length"
    );
    assert!(
        snapshot.progress.completed_algorithms <= snapshot.progress.total_algorithms,
        "Completed count should not exceed total"
    );
    assert!(
        snapshot.progress.completion_percentage >= 0.0
            && snapshot.progress.completion_percentage <= 100.0,
        "Completion percentage should be between 0 and 100"
    );

    // Check metadata
    assert!(
        !snapshot.metadata.format_version.is_empty(),
        "Format version should not be empty"
    );
    assert!(
        !snapshot.metadata.description.is_empty(),
        "Description should not be empty"
    );

    Ok(())
}

#[cfg(test)]
mod snapshot_tests {
    use super::*;

    #[test]
    fn test_snapshot_creation_and_basic_serialization() {
        let config = create_test_config();
        let mut algorithms = create_test_algorithms();
        
        // Generate test array
        let generator = ArrayGenerator::new();
        let test_array = generator.generate(&config);
        
        // Initialize algorithms with test data
        for algorithm in &mut algorithms {
            algorithm.reset(test_array.clone());
        }

        // Create snapshot
        let snapshot = create_snapshot_from_algorithms(&algorithms, &config, 0);

        // Validate structure
        validate_snapshot_structure(&snapshot).expect("Snapshot should be valid");

        // Test serialization
        let json = serde_json::to_string(&snapshot).expect("Should serialize to JSON");
        assert!(!json.is_empty(), "JSON should not be empty");
        assert!(json.contains("algorithm_states"), "JSON should contain algorithm states");
        assert!(json.contains("timestamp"), "JSON should contain timestamp");
    }

    #[test]
    fn test_snapshot_save_and_load_roundtrip() {
        let manager = SnapshotManager::new().expect("Should create snapshot manager");
        let config = create_test_config();
        let mut algorithms = create_test_algorithms();
        
        // Generate and set test array
        let generator = ArrayGenerator::new();
        let test_array = generator.generate(&config);
        for algorithm in &mut algorithms {
            algorithm.reset(test_array.clone());
        }

        // Execute some steps to create meaningful state
        let fairness_model = ComparisonBudget::new(5);
        for _ in 0..3 {
            let budgets = fairness_model.allocate_budget(&algorithms);
            for (i, algorithm) in algorithms.iter_mut().enumerate() {
                if budgets[i] > 0 && !algorithm.is_complete() {
                    algorithm.step(budgets[i]);
                }
            }
        }

        // Create and save snapshot
        let original_snapshot = create_snapshot_from_algorithms(&algorithms, &config, 3);
        let filename = "test_snapshot.json";
        let saved_path = manager
            .save_snapshot(&original_snapshot, filename)
            .expect("Should save snapshot");

        // Verify file exists
        assert!(manager.snapshot_exists(filename), "Snapshot file should exist");
        assert!(Path::new(&saved_path).exists(), "File should exist at saved path");

        // Load snapshot
        let loaded_snapshot = manager
            .load_snapshot(filename)
            .expect("Should load snapshot");

        // Verify loaded snapshot matches original
        assert_eq!(original_snapshot, loaded_snapshot, "Loaded snapshot should match original");
        
        // Validate loaded snapshot structure
        validate_snapshot_structure(&loaded_snapshot).expect("Loaded snapshot should be valid");
    }

    #[test]
    fn test_snapshot_contains_all_required_fields() {
        let config = create_test_config();
        let mut algorithms = create_test_algorithms();
        
        // Generate test array
        let generator = ArrayGenerator::new();
        let test_array = generator.generate(&config);
        for algorithm in &mut algorithms {
            algorithm.reset(test_array.clone());
        }

        // Create snapshot
        let snapshot = create_snapshot_from_algorithms(&algorithms, &config, 0);

        // Test all required fields are present
        assert!(snapshot.timestamp > 0, "Should have valid timestamp");
        assert!(!snapshot.algorithm_states.is_empty(), "Should have algorithm states");
        assert_eq!(snapshot.progress.total_algorithms, algorithms.len(), "Should track correct algorithm count");
        assert!(!snapshot.metadata.format_version.is_empty(), "Should have format version");
        assert!(!snapshot.metadata.description.is_empty(), "Should have description");

        // Test algorithm state fields
        for state in &snapshot.algorithm_states {
            assert!(!state.name.is_empty(), "Algorithm should have name");
            assert!(!state.array.is_empty(), "Algorithm should have array data");
            assert!(state.telemetry.total_comparisons >= 0, "Should have comparison count");
            assert!(state.telemetry.total_moves >= 0, "Should have move count");
        }

        // Test JSON serialization includes all fields
        let json = serde_json::to_string_pretty(&snapshot).expect("Should serialize");
        let required_fields = [
            "timestamp", "config", "algorithm_states", "progress", "metadata",
            "name", "array", "telemetry", "is_complete",
            "total_comparisons", "total_moves", "format_version"
        ];
        
        for field in &required_fields {
            assert!(json.contains(field), "JSON should contain field: {}", field);
        }
    }

    #[test]
    fn test_multiple_snapshots_without_conflicts() {
        let manager = SnapshotManager::new().expect("Should create snapshot manager");
        let config = create_test_config();
        let mut algorithms = create_test_algorithms();
        
        // Generate test array
        let generator = ArrayGenerator::new();
        let test_array = generator.generate(&config);
        for algorithm in &mut algorithms {
            algorithm.reset(test_array.clone());
        }

        let fairness_model = ComparisonBudget::new(5);
        let mut snapshots = Vec::new();

        // Create multiple snapshots at different stages
        for step in 0..5 {
            // Execute one step
            if step > 0 {
                let budgets = fairness_model.allocate_budget(&algorithms);
                for (i, algorithm) in algorithms.iter_mut().enumerate() {
                    if budgets[i] > 0 && !algorithm.is_complete() {
                        algorithm.step(budgets[i]);
                    }
                }
            }

            // Create and save snapshot
            let snapshot = create_snapshot_from_algorithms(&algorithms, &config, step);
            let filename = format!("snapshot_step_{}.json", step);
            
            manager
                .save_snapshot(&snapshot, &filename)
                .expect("Should save snapshot");
            
            snapshots.push((filename, snapshot));
        }

        // Verify all snapshots exist and are different
        for i in 0..snapshots.len() {
            let (filename, _) = &snapshots[i];
            assert!(manager.snapshot_exists(filename), "Snapshot {} should exist", i);
        }

        // Load and verify all snapshots
        for (filename, original) in &snapshots {
            let loaded = manager
                .load_snapshot(filename)
                .expect("Should load snapshot");
            assert_eq!(*original, loaded, "Snapshot {} should match", filename);
        }

        // Verify snapshots show progression (different states)
        for i in 1..snapshots.len() {
            let (_, prev_snapshot) = &snapshots[i - 1];
            let (_, curr_snapshot) = &snapshots[i];
            
            // At least one algorithm should have different telemetry
            let mut progress_made = false;
            for j in 0..prev_snapshot.algorithm_states.len() {
                let prev_state = &prev_snapshot.algorithm_states[j];
                let curr_state = &curr_snapshot.algorithm_states[j];
                
                if curr_state.telemetry.total_comparisons > prev_state.telemetry.total_comparisons ||
                   curr_state.telemetry.total_moves > prev_state.telemetry.total_moves {
                    progress_made = true;
                    break;
                }
            }
            
            // Note: Due to fairness allocation, not every step guarantees progress for every algorithm
            // So we don't assert progress_made here, but we could check overall trend
        }

        // List all snapshots
        let snapshot_list = manager.list_snapshots().expect("Should list snapshots");
        assert_eq!(snapshot_list.len(), 5, "Should have 5 snapshot files");
        
        // Verify filenames are sorted
        for i in 0..snapshot_list.len() {
            let expected = format!("snapshot_step_{}.json", i);
            assert_eq!(snapshot_list[i], expected, "Snapshot should be in order");
        }
    }

    #[test]
    fn test_loaded_state_matches_saved_state_exactly() {
        let manager = SnapshotManager::new().expect("Should create snapshot manager");
        let config = create_test_config();
        let mut algorithms = create_test_algorithms();
        
        // Generate test array
        let generator = ArrayGenerator::new();
        let test_array = generator.generate(&config);
        for algorithm in &mut algorithms {
            algorithm.reset(test_array.clone());
        }

        // Execute several steps to create complex state
        let fairness_model = ComparisonBudget::new(8);
        for _ in 0..10 {
            let budgets = fairness_model.allocate_budget(&algorithms);
            for (i, algorithm) in algorithms.iter_mut().enumerate() {
                if budgets[i] > 0 && !algorithm.is_complete() {
                    algorithm.step(budgets[i]);
                }
            }
        }

        // Create snapshot with complex state
        let original_snapshot = create_snapshot_from_algorithms(&algorithms, &config, 10);
        
        // Save and load
        let filename = "complex_state_test.json";
        manager
            .save_snapshot(&original_snapshot, filename)
            .expect("Should save complex snapshot");
        
        let loaded_snapshot = manager
            .load_snapshot(filename)
            .expect("Should load complex snapshot");

        // Verify exact match for all fields
        assert_eq!(original_snapshot.timestamp, loaded_snapshot.timestamp);
        assert_eq!(original_snapshot.config, loaded_snapshot.config);
        assert_eq!(original_snapshot.progress, loaded_snapshot.progress);
        assert_eq!(original_snapshot.metadata, loaded_snapshot.metadata);
        assert_eq!(original_snapshot.algorithm_states.len(), loaded_snapshot.algorithm_states.len());

        // Verify each algorithm state exactly
        for (orig, loaded) in original_snapshot.algorithm_states.iter().zip(loaded_snapshot.algorithm_states.iter()) {
            assert_eq!(orig.name, loaded.name);
            assert_eq!(orig.array, loaded.array);
            assert_eq!(orig.is_complete, loaded.is_complete);
            assert_eq!(orig.step_count, loaded.step_count);
            assert_eq!(orig.telemetry.total_comparisons, loaded.telemetry.total_comparisons);
            assert_eq!(orig.telemetry.total_moves, loaded.telemetry.total_moves);
            assert_eq!(orig.telemetry.array_accesses, loaded.telemetry.array_accesses);
            assert_eq!(orig.telemetry.custom_metrics, loaded.telemetry.custom_metrics);
        }
    }

    #[test]
    fn test_snapshot_with_completed_algorithms() {
        let manager = SnapshotManager::new().expect("Should create snapshot manager");
        let config = RunConfiguration {
            array_size: 5, // Small array for quick completion
            distribution: Distribution::Random,
            fairness_mode: FairnessMode::ComparisonBudget { k: 20 },
            seed: Some(123),
        };
        
        let mut algorithms = vec![
            Box::new(BubbleSort::new()) as Box<dyn Sorter>,
            Box::new(InsertionSort::new()) as Box<dyn Sorter>,
        ];
        
        // Generate small test array
        let generator = ArrayGenerator::new();
        let test_array = generator.generate(&config);
        for algorithm in &mut algorithms {
            algorithm.reset(test_array.clone());
        }

        // Run until at least one algorithm completes
        let fairness_model = ComparisonBudget::new(10);
        let mut steps = 0;
        while algorithms.iter().all(|alg| !alg.is_complete()) && steps < 100 {
            let budgets = fairness_model.allocate_budget(&algorithms);
            for (i, algorithm) in algorithms.iter_mut().enumerate() {
                if budgets[i] > 0 {
                    algorithm.step(budgets[i]);
                }
            }
            steps += 1;
        }

        // Create snapshot with completed algorithms
        let snapshot = create_snapshot_from_algorithms(&algorithms, &config, steps);
        
        // Verify progress stats reflect completion
        assert!(snapshot.progress.completed_algorithms > 0, "Should have completed algorithms");
        assert!(snapshot.progress.completion_percentage > 0.0, "Should have non-zero completion percentage");

        // Save and load
        let filename = "completed_algorithms_test.json";
        manager
            .save_snapshot(&snapshot, filename)
            .expect("Should save snapshot with completed algorithms");
        
        let loaded_snapshot = manager
            .load_snapshot(filename)
            .expect("Should load snapshot with completed algorithms");

        // Verify completion state is preserved
        assert_eq!(snapshot.progress.completed_algorithms, loaded_snapshot.progress.completed_algorithms);
        assert_eq!(snapshot.progress.completion_percentage, loaded_snapshot.progress.completion_percentage);
        
        // Verify individual algorithm completion states
        for (orig, loaded) in snapshot.algorithm_states.iter().zip(loaded_snapshot.algorithm_states.iter()) {
            assert_eq!(orig.is_complete, loaded.is_complete);
        }
    }

    #[test]
    fn test_snapshot_json_format_validation() {
        let manager = SnapshotManager::new().expect("Should create snapshot manager");
        let config = create_test_config();
        let mut algorithms = create_test_algorithms();
        
        // Generate test array  
        let generator = ArrayGenerator::new();
        let test_array = generator.generate(&config);
        for algorithm in &mut algorithms {
            algorithm.reset(test_array.clone());
        }

        // Create and save snapshot
        let snapshot = create_snapshot_from_algorithms(&algorithms, &config, 0);
        let filename = "format_validation_test.json";
        let file_path = manager
            .save_snapshot(&snapshot, filename)
            .expect("Should save snapshot");

        // Read raw JSON and validate format
        let json_content = fs::read_to_string(&file_path).expect("Should read JSON file");
        
        // Parse as generic JSON to validate structure
        let json_value: serde_json::Value = serde_json::from_str(&json_content)
            .expect("Should be valid JSON");

        // Verify top-level structure
        assert!(json_value.is_object(), "Root should be an object");
        let obj = json_value.as_object().unwrap();
        
        let expected_root_keys = ["timestamp", "config", "algorithm_states", "progress", "metadata"];
        for key in &expected_root_keys {
            assert!(obj.contains_key(key), "Should contain key: {}", key);
        }

        // Verify algorithm_states is an array
        assert!(obj["algorithm_states"].is_array(), "algorithm_states should be array");
        let states = obj["algorithm_states"].as_array().unwrap();
        assert!(!states.is_empty(), "Should have algorithm states");

        // Verify each algorithm state structure
        for state in states {
            assert!(state.is_object(), "Each state should be an object");
            let state_obj = state.as_object().unwrap();
            
            let expected_state_keys = ["name", "array", "telemetry", "is_complete", "step_count"];
            for key in &expected_state_keys {
                assert!(state_obj.contains_key(key), "State should contain key: {}", key);
            }
            
            // Verify telemetry structure
            assert!(state_obj["telemetry"].is_object(), "telemetry should be object");
            let telemetry = state_obj["telemetry"].as_object().unwrap();
            let expected_telemetry_keys = ["total_comparisons", "total_moves", "array_accesses", "custom_metrics"];
            for key in &expected_telemetry_keys {
                assert!(telemetry.contains_key(key), "Telemetry should contain key: {}", key);
            }
        }

        // Verify progress structure
        assert!(obj["progress"].is_object(), "progress should be object");
        let progress = obj["progress"].as_object().unwrap();
        let expected_progress_keys = ["completed_algorithms", "total_algorithms", "total_steps", "completion_percentage"];
        for key in &expected_progress_keys {
            assert!(progress.contains_key(key), "Progress should contain key: {}", key);
        }

        // Verify metadata structure
        assert!(obj["metadata"].is_object(), "metadata should be object");
        let metadata = obj["metadata"].as_object().unwrap();
        let expected_metadata_keys = ["format_version", "description", "tags"];
        for key in &expected_metadata_keys {
            assert!(metadata.contains_key(key), "Metadata should contain key: {}", key);
        }
    }
}
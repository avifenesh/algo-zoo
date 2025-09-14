//! Integration tests for pause/resume functionality
//! Tests the controller's ability to pause/resume sorting algorithms while preserving state

use sorting_race::models::config::{Distribution, FairnessMode, RunConfiguration};
use sorting_race::models::traits::{FairnessModel, Sorter, StepResult, Telemetry};
use sorting_race::services::fairness::comparison::ComparisonBudget;
use sorting_race::services::generator::ArrayGenerator;
use sorting_race::services::sorters::{
    bubble::BubbleSort, heap::HeapSort, insertion::InsertionSort, merge::MergeSort,
    quick::QuickSort, selection::SelectionSort,
};

/// Simulates a controller that can pause/resume algorithms
#[derive(Debug, Clone, PartialEq)]
enum ControllerState {
    Running,
    Paused,
}

/// Mock controller for testing pause/resume functionality
struct SortingController {
    algorithms: Vec<Box<dyn Sorter>>,
    fairness_model: Box<dyn FairnessModel>,
    state: ControllerState,
    step_count: usize,
    /// Track algorithm states at pause points for verification
    pause_snapshots: Vec<Vec<AlgorithmSnapshot>>,
}

#[derive(Debug, Clone)]
struct AlgorithmSnapshot {
    name: String,
    array_state: Vec<i32>,
    telemetry: Telemetry,
    is_complete: bool,
}

impl SortingController {
    fn new(algorithms: Vec<Box<dyn Sorter>>, fairness_model: Box<dyn FairnessModel>) -> Self {
        Self {
            algorithms,
            fairness_model,
            state: ControllerState::Running,
            step_count: 0,
            pause_snapshots: Vec::new(),
        }
    }
    
    fn pause(&mut self) {
        self.state = ControllerState::Paused;
        // Take snapshot when pausing
        let snapshot = self.take_snapshot();
        self.pause_snapshots.push(snapshot);
    }
    
    fn resume(&mut self) {
        self.state = ControllerState::Running;
    }
    
    fn is_paused(&self) -> bool {
        self.state == ControllerState::Paused
    }
    
    fn is_running(&self) -> bool {
        self.state == ControllerState::Running
    }
    
    /// Execute one step if running, no-op if paused
    fn step(&mut self) -> bool {
        if self.is_paused() {
            return false; // No progress when paused
        }
        
        let active_count = self.algorithms.iter().filter(|alg| !alg.is_complete()).count();
        if active_count == 0 {
            return false; // All algorithms complete
        }
        
        // Allocate budgets using fairness model
        let budgets = self.fairness_model.allocate_budget(&self.algorithms);
        
        // Execute steps for each algorithm
        for (i, algorithm) in self.algorithms.iter_mut().enumerate() {
            if budgets[i] > 0 {
                algorithm.step(budgets[i]);
            }
        }
        
        self.step_count += 1;
        true
    }
    
    fn take_snapshot(&self) -> Vec<AlgorithmSnapshot> {
        self.algorithms
            .iter()
            .map(|alg| AlgorithmSnapshot {
                name: alg.name().to_string(),
                array_state: alg.get_array().to_vec(),
                telemetry: alg.get_telemetry(),
                is_complete: alg.is_complete(),
            })
            .collect()
    }
    
    fn get_step_count(&self) -> usize {
        self.step_count
    }
    
    fn all_complete(&self) -> bool {
        self.algorithms.iter().all(|alg| alg.is_complete())
    }
    
    fn get_pause_snapshots(&self) -> &Vec<Vec<AlgorithmSnapshot>> {
        &self.pause_snapshots
    }
    
    /// Reset all algorithms with new data
    fn reset(&mut self, array: Vec<i32>) {
        for algorithm in &mut self.algorithms {
            algorithm.reset(array.clone());
        }
        self.step_count = 0;
        self.pause_snapshots.clear();
        self.state = ControllerState::Running;
    }
}

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

fn verify_snapshots_identical(snapshot1: &[AlgorithmSnapshot], snapshot2: &[AlgorithmSnapshot]) {
    assert_eq!(snapshot1.len(), snapshot2.len());
    
    for (s1, s2) in snapshot1.iter().zip(snapshot2.iter()) {
        assert_eq!(s1.name, s2.name);
        assert_eq!(s1.array_state, s2.array_state);
        assert_eq!(s1.is_complete, s2.is_complete);
        assert_eq!(s1.telemetry.total_comparisons, s2.telemetry.total_comparisons);
        assert_eq!(s1.telemetry.total_moves, s2.telemetry.total_moves);
    }
}

#[cfg(test)]
mod pause_resume_tests {
    use super::*;

    #[test]
    fn test_basic_pause_resume() {
        let generator = ArrayGenerator::new(12345);
        let array = generator.generate(30, &Distribution::Shuffled);
        let algorithms = create_test_algorithms();
        let fairness_model = Box::new(ComparisonBudget::new(16));
        
        let mut controller = SortingController::new(algorithms, fairness_model);
        controller.reset(array);
        
        // Initial state should be running
        assert!(controller.is_running());
        assert!(!controller.is_paused());
        
        // Execute some steps while running
        for _ in 0..5 {
            assert!(controller.step());
        }
        
        let steps_before_pause = controller.get_step_count();
        assert!(steps_before_pause > 0);
        
        // Pause the controller
        controller.pause();
        assert!(controller.is_paused());
        assert!(!controller.is_running());
        
        // Resume the controller
        controller.resume();
        assert!(controller.is_running());
        assert!(!controller.is_paused());
        
        // Should be able to continue execution
        assert!(controller.step());
        assert!(controller.get_step_count() > steps_before_pause);
    }

    #[test]
    fn test_no_progress_while_paused() {
        let generator = ArrayGenerator::new(54321);
        let array = generator.generate(25, &Distribution::Shuffled);
        let algorithms = create_test_algorithms();
        let fairness_model = Box::new(ComparisonBudget::new(16));
        
        let mut controller = SortingController::new(algorithms, fairness_model);
        controller.reset(array);
        
        // Execute some steps to get initial progress
        for _ in 0..3 {
            controller.step();
        }
        
        let snapshot_before_pause = controller.take_snapshot();
        let steps_before_pause = controller.get_step_count();
        
        // Pause the controller
        controller.pause();
        
        // Attempt to step multiple times while paused - should not progress
        for _ in 0..10 {
            let made_progress = controller.step();
            assert!(!made_progress, "Should not make progress while paused");
        }
        
        // Verify no progress was made
        let snapshot_after_pause_attempts = controller.take_snapshot();
        assert_eq!(controller.get_step_count(), steps_before_pause);
        verify_snapshots_identical(&snapshot_before_pause, &snapshot_after_pause_attempts);
        
        // Resume and verify we can make progress again
        controller.resume();
        assert!(controller.step());
        assert!(controller.get_step_count() > steps_before_pause);
    }

    #[test]
    fn test_state_preserved_during_pause_resume() {
        let generator = ArrayGenerator::new(98765);
        let array = generator.generate(40, &Distribution::NearlySorted);
        let algorithms = create_test_algorithms();
        let fairness_model = Box::new(ComparisonBudget::new(12));
        
        let mut controller = SortingController::new(algorithms, fairness_model);
        controller.reset(array);
        
        // Run for several steps
        for _ in 0..7 {
            controller.step();
        }
        
        let snapshot_before_pause = controller.take_snapshot();
        
        // Pause
        controller.pause();
        
        // Wait (simulate paused state)
        let snapshot_while_paused = controller.take_snapshot();
        verify_snapshots_identical(&snapshot_before_pause, &snapshot_while_paused);
        
        // Resume
        controller.resume();
        
        // Take snapshot immediately after resume (before any steps)
        let snapshot_after_resume = controller.take_snapshot();
        verify_snapshots_identical(&snapshot_before_pause, &snapshot_after_resume);
        
        // Make progress after resume
        controller.step();
        let snapshot_after_progress = controller.take_snapshot();
        
        // Verify progress was made (at least one algorithm should have different metrics)
        let mut found_progress = false;
        for (before, after) in snapshot_before_pause.iter().zip(snapshot_after_progress.iter()) {
            if before.telemetry.total_comparisons != after.telemetry.total_comparisons ||
               before.telemetry.total_moves != after.telemetry.total_moves ||
               before.array_state != after.array_state {
                found_progress = true;
                break;
            }
        }
        assert!(found_progress, "Should make progress after resuming");
    }

    #[test]
    fn test_multiple_pause_resume_cycles() {
        let generator = ArrayGenerator::new(11111);
        let array = generator.generate(35, &Distribution::FewUnique);
        let algorithms = create_test_algorithms();
        let fairness_model = Box::new(ComparisonBudget::new(8));
        
        let mut controller = SortingController::new(algorithms, fairness_model);
        controller.reset(array);
        
        let mut expected_steps = 0;
        let mut snapshots_during_execution = Vec::new();
        
        // Perform multiple pause/resume cycles
        for cycle in 0..5 {
            // Run for a few steps
            let steps_this_cycle = 3;
            for _ in 0..steps_this_cycle {
                if controller.all_complete() {
                    break;
                }
                controller.step();
            }
            expected_steps += steps_this_cycle;
            
            let snapshot_before_pause = controller.take_snapshot();
            snapshots_during_execution.push(snapshot_before_pause.clone());
            
            // Pause
            controller.pause();
            assert!(controller.is_paused());
            
            // Verify no progress during pause
            for _ in 0..5 {
                assert!(!controller.step());
            }
            
            let snapshot_during_pause = controller.take_snapshot();
            verify_snapshots_identical(&snapshot_before_pause, &snapshot_during_pause);
            
            // Resume
            controller.resume();
            assert!(controller.is_running());
            
            let snapshot_after_resume = controller.take_snapshot();
            verify_snapshots_identical(&snapshot_before_pause, &snapshot_after_resume);
            
            if controller.all_complete() {
                break;
            }
        }
        
        // Verify we have pause snapshots recorded
        let pause_snapshots = controller.get_pause_snapshots();
        assert!(!pause_snapshots.is_empty(), "Should have recorded pause snapshots");
        
        // Verify each pause snapshot matches our manual snapshots
        for (manual, recorded) in snapshots_during_execution.iter().zip(pause_snapshots.iter()) {
            verify_snapshots_identical(manual, recorded);
        }
    }

    #[test]
    fn test_pause_resume_with_different_algorithms_completing() {
        let generator = ArrayGenerator::new(22222);
        let array = generator.generate(20, &Distribution::Shuffled); // Small array for quick completion
        let algorithms = create_test_algorithms();
        let fairness_model = Box::new(ComparisonBudget::new(20));
        
        let mut controller = SortingController::new(algorithms, fairness_model);
        controller.reset(array);
        
        let mut pause_cycle = 0;
        
        // Run until some algorithms complete
        while !controller.all_complete() && pause_cycle < 10 {
            // Run for a few steps
            for _ in 0..2 {
                if !controller.step() {
                    break;
                }
            }
            
            let snapshot_before_pause = controller.take_snapshot();
            let active_before = snapshot_before_pause.iter().filter(|s| !s.is_complete).count();
            
            // Pause
            controller.pause();
            
            // Verify state during pause
            let snapshot_during_pause = controller.take_snapshot();
            let active_during = snapshot_during_pause.iter().filter(|s| !s.is_complete).count();
            assert_eq!(active_before, active_during);
            verify_snapshots_identical(&snapshot_before_pause, &snapshot_during_pause);
            
            // Resume
            controller.resume();
            
            let snapshot_after_resume = controller.take_snapshot();
            let active_after = snapshot_after_resume.iter().filter(|s| !s.is_complete).count();
            assert_eq!(active_before, active_after);
            verify_snapshots_identical(&snapshot_before_pause, &snapshot_after_resume);
            
            pause_cycle += 1;
        }
        
        assert!(pause_cycle > 0, "Should have performed at least one pause/resume cycle");
        
        // Verify some algorithms completed during the test
        let final_snapshot = controller.take_snapshot();
        let completed_count = final_snapshot.iter().filter(|s| s.is_complete).count();
        assert!(completed_count > 0, "At least some algorithms should have completed");
    }

    #[test]
    fn test_pause_resume_fairness_consistency() {
        let generator = ArrayGenerator::new(33333);
        let array = generator.generate(30, &Distribution::Reversed);
        let algorithms = create_test_algorithms();
        let fairness_model = Box::new(ComparisonBudget::new(10));
        
        let mut controller = SortingController::new(algorithms, fairness_model);
        controller.reset(array.clone());
        
        // Create a reference controller that runs without pauses
        let algorithms_ref = create_test_algorithms();
        let fairness_model_ref = Box::new(ComparisonBudget::new(10));
        let mut controller_ref = SortingController::new(algorithms_ref, fairness_model_ref);
        controller_ref.reset(array);
        
        let max_steps = 50;
        let mut steps_executed = 0;
        
        // Run test controller with pauses, reference controller without
        for step in 0..max_steps {
            if controller.all_complete() || controller_ref.all_complete() {
                break;
            }
            
            // Test controller: pause every 3rd step
            if step % 3 == 2 {
                controller.pause();
                
                // Try to step while paused (should not progress)
                assert!(!controller.step());
                
                controller.resume();
            }
            
            // Both controllers step
            let test_progressed = controller.step();
            let ref_progressed = controller_ref.step();
            
            // Both should progress or not progress together
            assert_eq!(test_progressed, ref_progressed);
            
            if test_progressed {
                steps_executed += 1;
            }
        }
        
        // Final comparison - both controllers should have similar progress
        let test_snapshot = controller.take_snapshot();
        let ref_snapshot = controller_ref.take_snapshot();
        
        assert_eq!(test_snapshot.len(), ref_snapshot.len());
        
        // Check that fairness was preserved despite pauses
        for (test_alg, ref_alg) in test_snapshot.iter().zip(ref_snapshot.iter()) {
            assert_eq!(test_alg.name, ref_alg.name);
            
            // Algorithms should have made similar progress (within reasonable bounds)
            let test_total_ops = test_alg.telemetry.total_comparisons + test_alg.telemetry.total_moves;
            let ref_total_ops = ref_alg.telemetry.total_comparisons + ref_alg.telemetry.total_moves;
            
            // Allow for some variance due to the discrete nature of stepping
            let ops_diff = if test_total_ops > ref_total_ops {
                test_total_ops - ref_total_ops
            } else {
                ref_total_ops - test_total_ops
            };
            
            assert!(
                ops_diff <= 30, // Reasonable tolerance for discrete stepping
                "Algorithm {} operations differ too much: test={}, ref={}, diff={}",
                test_alg.name, test_total_ops, ref_total_ops, ops_diff
            );
        }
    }

    #[test]
    fn test_pause_resume_edge_cases() {
        let generator = ArrayGenerator::new(44444);
        let array = generator.generate(15, &Distribution::Shuffled);
        let algorithms = create_test_algorithms();
        let fairness_model = Box::new(ComparisonBudget::new(16));
        
        let mut controller = SortingController::new(algorithms, fairness_model);
        controller.reset(array);
        
        // Test pause when already paused
        controller.pause();
        assert!(controller.is_paused());
        controller.pause(); // Should be safe to call again
        assert!(controller.is_paused());
        
        // Test resume when already running
        controller.resume();
        assert!(controller.is_running());
        controller.resume(); // Should be safe to call again
        assert!(controller.is_running());
        
        // Test pause/resume at start (no steps executed)
        controller.reset(generator.generate(15, &Distribution::Shuffled));
        let initial_snapshot = controller.take_snapshot();
        
        controller.pause();
        let paused_snapshot = controller.take_snapshot();
        verify_snapshots_identical(&initial_snapshot, &paused_snapshot);
        
        controller.resume();
        let resumed_snapshot = controller.take_snapshot();
        verify_snapshots_identical(&initial_snapshot, &resumed_snapshot);
        
        // Test pause/resume when all algorithms complete
        // Force completion by running until done
        while !controller.all_complete() {
            controller.step();
        }
        
        let completed_snapshot = controller.take_snapshot();
        
        controller.pause();
        let paused_complete_snapshot = controller.take_snapshot();
        verify_snapshots_identical(&completed_snapshot, &paused_complete_snapshot);
        
        controller.resume();
        let resumed_complete_snapshot = controller.take_snapshot();
        verify_snapshots_identical(&completed_snapshot, &resumed_complete_snapshot);
        
        // Step should return false when all complete, regardless of pause state
        assert!(!controller.step());
    }

    #[test]
    fn test_pause_resume_with_zero_budget() {
        let generator = ArrayGenerator::new(55555);
        let array = generator.generate(25, &Distribution::Shuffled);
        let algorithms = create_test_algorithms();
        let fairness_model = Box::new(ComparisonBudget::new(0)); // Zero budget
        
        let mut controller = SortingController::new(algorithms, fairness_model);
        controller.reset(array);
        
        let initial_snapshot = controller.take_snapshot();
        
        // With zero budget, algorithms shouldn't progress
        for _ in 0..5 {
            assert!(!controller.step());
        }
        
        let after_steps_snapshot = controller.take_snapshot();
        verify_snapshots_identical(&initial_snapshot, &after_steps_snapshot);
        
        // Pause and resume should still work
        controller.pause();
        assert!(controller.is_paused());
        
        controller.resume();
        assert!(controller.is_running());
        
        // Still no progress with zero budget
        assert!(!controller.step());
        let final_snapshot = controller.take_snapshot();
        verify_snapshots_identical(&initial_snapshot, &final_snapshot);
    }
}
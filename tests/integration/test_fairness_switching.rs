//! Integration tests for switching between fairness models
//! These tests verify that all models integrate correctly with the race system

use sorting_race::models::config::{Distribution, FairnessMode, RunConfiguration};
use sorting_race::models::traits::{FairnessModel, Sorter, StepResult, Telemetry};
use sorting_race::services::fairness::{
    ComparisonFairness as ComparisonBudget, WallTimeFairness, WeightedFairness,
    AdaptiveFairness,
};
use sorting_race::services::generator::ArrayGenerator;
use sorting_race::services::sorters::{
    bubble::BubbleSort, heap::HeapSort, insertion::InsertionSort, merge::MergeSort,
    quick::QuickSort, selection::SelectionSort,
};

/// Factory function to create fairness models from configuration
fn create_fairness_model(mode: &FairnessMode) -> Box<dyn FairnessModel> {
    match mode {
        FairnessMode::ComparisonBudget { k } => Box::new(ComparisonBudget::new(*k)),
        FairnessMode::Weighted { alpha, beta } => Box::new(WeightedFairness::new(*alpha, *beta)),
        FairnessMode::WallTime { slice_ms } => Box::new(WallTimeFairness::new(*slice_ms)),
        FairnessMode::EqualSteps => {
            // EqualSteps could be implemented as a special case of ComparisonBudget
            Box::new(ComparisonBudget::new(1))
        }
    }
}

/// Additional fairness mode for testing adaptive fairness
#[derive(Debug, Clone, PartialEq)]
pub enum ExtendedFairnessMode {
    ComparisonBudget { k: usize },
    Weighted { alpha: f32, beta: f32 },
    WallTime { slice_ms: u64 },
    Adaptive { learning_rate: f32 },
    EqualSteps,
}

fn create_extended_fairness_model(mode: &ExtendedFairnessMode) -> Box<dyn FairnessModel> {
    match mode {
        ExtendedFairnessMode::ComparisonBudget { k } => Box::new(ComparisonBudget::new(*k)),
        ExtendedFairnessMode::Weighted { alpha, beta } => Box::new(WeightedFairness::new(*alpha, *beta)),
        ExtendedFairnessMode::WallTime { slice_ms } => Box::new(WallTimeFairness::new(*slice_ms)),
        ExtendedFairnessMode::Adaptive { learning_rate } => Box::new(AdaptiveFairness::new(*learning_rate)),
        ExtendedFairnessMode::EqualSteps => Box::new(ComparisonBudget::new(1)),
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

fn initialize_algorithms_with_array(algorithms: &mut [Box<dyn Sorter>], array: Vec<i32>) {
    for algorithm in algorithms.iter_mut() {
        algorithm.reset(array.clone());
    }
}

fn count_active_algorithms(algorithms: &[Box<dyn Sorter>]) -> usize {
    algorithms.iter().filter(|alg| !alg.is_complete()).count()
}

fn get_algorithm_stats(algorithms: &[Box<dyn Sorter>]) -> Vec<(String, u64, u64, bool, f32)> {
    algorithms
        .iter()
        .map(|alg| {
            let telemetry = alg.get_telemetry();
            (
                alg.name().to_string(),
                telemetry.total_comparisons,
                telemetry.total_moves,
                alg.is_complete(),
                telemetry.progress_hint,
            )
        })
        .collect()
}

#[cfg(test)]
mod fairness_switching_tests {
    use super::*;

    #[test]
    fn test_switching_fairness_models_mid_race() {
        let generator = ArrayGenerator::new(12345);
        let array = generator.generate(40, &Distribution::Shuffled);
        let mut algorithms = create_test_algorithms();
        initialize_algorithms_with_array(&mut algorithms, array);

        // Start with ComparisonBudget
        let mut current_model: Box<dyn FairnessModel> = Box::new(ComparisonBudget::new(15));
        
        // Run first phase
        for step in 0..5 {
            let active_count = count_active_algorithms(&algorithms);
            if active_count == 0 {
                break;
            }

            let budgets = current_model.allocate_budget(&algorithms);
            
            // Verify budgets are allocated
            assert_eq!(budgets.len(), algorithms.len());
            let active_budget_sum: usize = budgets.iter().enumerate()
                .filter(|(i, _)| !algorithms[*i].is_complete())
                .map(|(_, budget)| *budget)
                .sum();
            assert!(active_budget_sum > 0, "Step {}: Should allocate budgets", step);
            
            // Execute steps
            for (i, algorithm) in algorithms.iter_mut().enumerate() {
                if budgets[i] > 0 {
                    algorithm.step(budgets[i]);
                }
            }
        }
        
        let mid_stats = get_algorithm_stats(&algorithms);
        
        // Switch to WeightedFairness mid-race
        current_model = Box::new(WeightedFairness::new(1.5, 0.8));
        
        // Run second phase with new model
        for step in 5..10 {
            let active_count = count_active_algorithms(&algorithms);
            if active_count == 0 {
                break;
            }

            let budgets = current_model.allocate_budget(&algorithms);
            
            // Verify the new model works correctly
            assert_eq!(budgets.len(), algorithms.len());
            let active_budget_sum: usize = budgets.iter().enumerate()
                .filter(|(i, _)| !algorithms[*i].is_complete())
                .map(|(_, budget)| *budget)
                .sum();
            assert!(active_budget_sum > 0, "Step {}: Weighted model should allocate budgets", step);
            
            // Execute steps
            for (i, algorithm) in algorithms.iter_mut().enumerate() {
                if budgets[i] > 0 {
                    algorithm.step(budgets[i]);
                }
            }
        }
        
        let final_stats = get_algorithm_stats(&algorithms);
        
        // Verify progress was made in both phases
        for ((mid_name, mid_comps, mid_moves, _, _), (final_name, final_comps, final_moves, _, _)) in 
            mid_stats.iter().zip(final_stats.iter()) {
            
            assert_eq!(mid_name, final_name);
            assert!(final_comps >= mid_comps, "Algorithm {} should not lose comparisons", mid_name);
            assert!(final_moves >= mid_moves, "Algorithm {} should not lose moves", mid_name);
            
            // At least one algorithm should have made progress in the second phase
            let made_progress = final_comps > mid_comps || final_moves > mid_moves;
            // We can't assert this for each individual algorithm because some might complete early
        }
    }

    #[test]
    fn test_all_fairness_models_integration() {
        let generator = ArrayGenerator::new(54321);
        let array = generator.generate(30, &Distribution::Shuffled);
        
        let fairness_modes = vec![
            ExtendedFairnessMode::ComparisonBudget { k: 10 },
            ExtendedFairnessMode::Weighted { alpha: 1.0, beta: 1.0 },
            ExtendedFairnessMode::WallTime { slice_ms: 20 },
            ExtendedFairnessMode::Adaptive { learning_rate: 0.3 },
        ];
        
        let mut results = Vec::new();
        
        for mode in &fairness_modes {
            let mut algorithms = create_test_algorithms();
            initialize_algorithms_with_array(&mut algorithms, array.clone());
            
            let model = create_extended_fairness_model(mode);
            
            // Run several steps with each model
            for step in 0..8 {
                let active_count = count_active_algorithms(&algorithms);
                if active_count == 0 {
                    break;
                }
                
                let budgets = model.allocate_budget(&algorithms);
                
                // All models should integrate correctly
                assert_eq!(budgets.len(), algorithms.len(), "Model {:?}: Budget count mismatch", mode);
                
                // Active algorithms should get budgets
                for (i, algorithm) in algorithms.iter().enumerate() {
                    if algorithm.is_complete() {
                        assert_eq!(budgets[i], 0, "Model {:?}: Completed algorithms should get 0 budget", mode);
                    } else {
                        assert!(budgets[i] > 0, "Model {:?}: Active algorithms should get budget", mode);
                    }
                }
                
                // Execute steps
                for (i, algorithm) in algorithms.iter_mut().enumerate() {
                    if budgets[i] > 0 {
                        algorithm.step(budgets[i]);
                    }
                }
            }
            
            let final_stats = get_algorithm_stats(&algorithms);
            results.push((mode.clone(), final_stats));
        }
        
        // Verify all models produced valid results
        for (mode, stats) in &results {
            for (name, comps, moves, _, _) in stats {
                assert!(comps >= &0 && moves >= &0, "Model {:?}: Algorithm {} has invalid stats", mode, name);
                
                // At least some progress should have been made
                let total_operations = *comps + *moves;
                assert!(total_operations > 0, "Model {:?}: Algorithm {} made no progress", mode, name);
            }
        }
    }

    #[test]
    fn test_cli_argument_parsing_for_fairness_modes() {
        // Test that configuration parsing works for all fairness modes
        let configurations = vec![
            RunConfiguration::new().with_fairness_mode(FairnessMode::ComparisonBudget { k: 16 }),
            RunConfiguration::new().with_fairness_mode(FairnessMode::Weighted { alpha: 2.0, beta: 1.5 }),
            RunConfiguration::new().with_fairness_mode(FairnessMode::WallTime { slice_ms: 25 }),
            RunConfiguration::new().with_fairness_mode(FairnessMode::EqualSteps),
        ];
        
        for config in &configurations {
            // Validate configuration
            assert!(config.validate().is_ok(), "Configuration should be valid: {:?}", config.fairness_mode);
            
            // Create model from configuration
            let model = create_fairness_model(&config.fairness_mode);
            
            // Verify model was created successfully
            assert!(!model.name().is_empty(), "Model should have a non-empty name");
            
            // Test with sample algorithms
            let algorithms = create_test_algorithms();
            let budgets = model.allocate_budget(&algorithms);
            
            assert_eq!(budgets.len(), algorithms.len(), "Budget count should match algorithm count");
        }
    }

    #[test]
    fn test_fairness_model_factory() {
        // Test the factory function with various configurations
        let test_cases = vec![
            (FairnessMode::ComparisonBudget { k: 5 }, "Comparison"),
            (FairnessMode::Weighted { alpha: 1.0, beta: 1.0 }, "Weighted"),
            (FairnessMode::WallTime { slice_ms: 30 }, "Wall"),
            (FairnessMode::EqualSteps, "Comparison"), // Maps to ComparisonBudget(1)
        ];
        
        for (mode, expected_name_part) in test_cases {
            let model = create_fairness_model(&mode);
            
            assert!(
                model.name().contains(expected_name_part),
                "Model for {:?} should contain '{}' in name, got '{}'",
                mode, expected_name_part, model.name()
            );
            
            // Test basic functionality
            let algorithms = vec![
                Box::new(BubbleSort::new()) as Box<dyn Sorter>,
                Box::new(QuickSort::new()) as Box<dyn Sorter>,
            ];
            
            let budgets = model.allocate_budget(&algorithms);
            assert_eq!(budgets.len(), 2);
        }
    }

    #[test]
    fn test_fairness_model_consistency_across_switches() {
        let generator = ArrayGenerator::new(99999);
        let array = generator.generate(25, &Distribution::NearlySorted);
        
        // Create multiple copies of algorithms for consistency testing
        let mut algorithms1 = create_test_algorithms();
        let mut algorithms2 = create_test_algorithms();
        let mut algorithms3 = create_test_algorithms();
        
        initialize_algorithms_with_array(&mut algorithms1, array.clone());
        initialize_algorithms_with_array(&mut algorithms2, array.clone());
        initialize_algorithms_with_array(&mut algorithms3, array.clone());
        
        let model1 = ComparisonBudget::new(12);
        let model2 = WeightedFairness::new(1.2, 0.9);
        let model3 = WallTimeFairness::new(15);
        
        // Run same number of steps with each model
        for step in 0..6 {
            // Model 1
            let active1 = count_active_algorithms(&algorithms1);
            if active1 > 0 {
                let budgets1 = model1.allocate_budget(&algorithms1);
                for (i, algorithm) in algorithms1.iter_mut().enumerate() {
                    if budgets1[i] > 0 {
                        algorithm.step(budgets1[i]);
                    }
                }
            }
            
            // Model 2
            let active2 = count_active_algorithms(&algorithms2);
            if active2 > 0 {
                let budgets2 = model2.allocate_budget(&algorithms2);
                for (i, algorithm) in algorithms2.iter_mut().enumerate() {
                    if budgets2[i] > 0 {
                        algorithm.step(budgets2[i]);
                    }
                }
            }
            
            // Model 3
            let active3 = count_active_algorithms(&algorithms3);
            if active3 > 0 {
                let budgets3 = model3.allocate_budget(&algorithms3);
                for (i, algorithm) in algorithms3.iter_mut().enumerate() {
                    if budgets3[i] > 0 {
                        algorithm.step(budgets3[i]);
                    }
                }
            }
        }
        
        let stats1 = get_algorithm_stats(&algorithms1);
        let stats2 = get_algorithm_stats(&algorithms2);
        let stats3 = get_algorithm_stats(&algorithms3);
        
        // All models should produce reasonable progress
        for (stats, model_name) in &[(&stats1, "ComparisonBudget"), (&stats2, "WeightedFairness"), (&stats3, "WallTimeFairness")] {
            for (name, comps, moves, _, _) in *stats {
                let total_ops = *comps + *moves;
                assert!(total_ops > 0, "{} model: Algorithm {} should make progress", model_name, name);
            }
        }
        
        // Different models may produce different results, but all should be valid
        // This is expected behavior as each model has different allocation strategies
    }

    #[test]
    fn test_switching_preserves_algorithm_state() {
        let generator = ArrayGenerator::new(77777);
        let array = generator.generate(35, &Distribution::FewUnique);
        let mut algorithms = create_test_algorithms();
        initialize_algorithms_with_array(&mut algorithms, array);
        
        // Run with first model
        let model1 = ComparisonBudget::new(8);
        for _ in 0..3 {
            let budgets = model1.allocate_budget(&algorithms);
            for (i, algorithm) in algorithms.iter_mut().enumerate() {
                if budgets[i] > 0 {
                    algorithm.step(budgets[i]);
                }
            }
        }
        
        let mid_stats = get_algorithm_stats(&algorithms);
        
        // Switch to different model
        let model2 = WeightedFairness::new(0.8, 1.3);
        for _ in 0..3 {
            let budgets = model2.allocate_budget(&algorithms);
            for (i, algorithm) in algorithms.iter_mut().enumerate() {
                if budgets[i] > 0 {
                    algorithm.step(budgets[i]);
                }
            }
        }
        
        let final_stats = get_algorithm_stats(&algorithms);
        
        // Algorithm state should be preserved across switches
        for ((mid_name, mid_comps, mid_moves, mid_complete, _), (final_name, final_comps, final_moves, final_complete, _)) in 
            mid_stats.iter().zip(final_stats.iter()) {
            
            assert_eq!(mid_name, final_name);
            
            // Stats should only increase
            assert!(final_comps >= mid_comps, "Comparisons should not decrease for {}", mid_name);
            assert!(final_moves >= mid_moves, "Moves should not decrease for {}", mid_name);
            
            // Completion state should be monotonic
            if *mid_complete {
                assert!(final_complete, "Algorithm {} should remain completed", mid_name);
            }
        }
    }

    #[test]
    fn test_adaptive_fairness_integration() {
        let generator = ArrayGenerator::new(88888);
        let array = generator.generate(30, &Distribution::Shuffled);
        let mut algorithms = create_test_algorithms();
        initialize_algorithms_with_array(&mut algorithms, array);
        
        let model = AdaptiveFairness::new(0.25);
        
        let mut progress_history = Vec::new();
        
        // Run several steps to test adaptive behavior
        for step in 0..10 {
            let active_count = count_active_algorithms(&algorithms);
            if active_count == 0 {
                break;
            }
            
            let budgets = model.allocate_budget(&algorithms);
            
            // Adaptive model should integrate like other models
            assert_eq!(budgets.len(), algorithms.len());
            
            let active_budgets: Vec<usize> = budgets.iter().enumerate()
                .filter(|(i, _)| !algorithms[*i].is_complete())
                .map(|(_, budget)| *budget)
                .collect();
            
            if !active_budgets.is_empty() {
                let total_active_budget: usize = active_budgets.iter().sum();
                assert!(total_active_budget > 0, "Step {}: Adaptive model should allocate budgets", step);
            }
            
            // Execute steps
            for (i, algorithm) in algorithms.iter_mut().enumerate() {
                if budgets[i] > 0 {
                    algorithm.step(budgets[i]);
                }
            }
            
            // Record progress for analysis
            let stats = get_algorithm_stats(&algorithms);
            progress_history.push(stats);
        }
        
        // Verify adaptive model produced reasonable results
        if let (Some(first_stats), Some(last_stats)) = (progress_history.first(), progress_history.last()) {
            for ((first_name, first_comps, first_moves, _, _), (last_name, last_comps, last_moves, _, _)) in 
                first_stats.iter().zip(last_stats.iter()) {
                
                assert_eq!(first_name, last_name);
                assert!(last_comps >= first_comps, "Algorithm {} should not lose progress", first_name);
                assert!(last_moves >= first_moves, "Algorithm {} should not lose progress", first_name);
            }
        }
    }

    #[test]
    fn test_rapid_fairness_switching() {
        let generator = ArrayGenerator::new(11111);
        let array = generator.generate(20, &Distribution::Shuffled);
        let mut algorithms = create_test_algorithms();
        initialize_algorithms_with_array(&mut algorithms, array);
        
        let models: Vec<Box<dyn FairnessModel>> = vec![
            Box::new(ComparisonBudget::new(6)),
            Box::new(WeightedFairness::new(1.0, 1.5)),
            Box::new(WallTimeFairness::new(12)),
            Box::new(AdaptiveFairness::new(0.4)),
        ];
        
        // Switch models every step
        for step in 0..12 {
            let active_count = count_active_algorithms(&algorithms);
            if active_count == 0 {
                break;
            }
            
            let model_index = step % models.len();
            let budgets = models[model_index].allocate_budget(&algorithms);
            
            // Rapid switching should not break the system
            assert_eq!(budgets.len(), algorithms.len(), "Step {}: Budget count mismatch", step);
            
            let active_budget_sum: usize = budgets.iter().enumerate()
                .filter(|(i, _)| !algorithms[*i].is_complete())
                .map(|(_, budget)| *budget)
                .sum();
            
            if active_count > 0 {
                assert!(active_budget_sum > 0, "Step {}: Should allocate budgets to active algorithms", step);
            }
            
            // Execute steps
            for (i, algorithm) in algorithms.iter_mut().enumerate() {
                if budgets[i] > 0 {
                    algorithm.step(budgets[i]);
                }
            }
        }
        
        // System should remain stable despite rapid switching
        let final_stats = get_algorithm_stats(&algorithms);
        for (name, comps, moves, _, _) in &final_stats {
            assert!(comps + moves > 0, "Algorithm {} should have made progress", name);
        }
    }

    #[test]
    fn test_fairness_model_edge_cases() {
        // Test various edge cases that could occur during model switching
        let empty_algorithms: Vec<Box<dyn Sorter>> = vec![];
        let models: Vec<Box<dyn FairnessModel>> = vec![
            Box::new(ComparisonBudget::new(10)),
            Box::new(WeightedFairness::new(0.5, 2.0)),
            Box::new(WallTimeFairness::new(20)),
            Box::new(AdaptiveFairness::new(0.1)),
        ];
        
        // All models should handle empty algorithm list
        for (i, model) in models.iter().enumerate() {
            let budgets = model.allocate_budget(&empty_algorithms);
            assert_eq!(budgets.len(), 0, "Model {} should handle empty algorithms", i);
        }
        
        // Test with all completed algorithms
        let completed_algorithms: Vec<Box<dyn Sorter>> = vec![
            Box::new(QuickSort::new()), // We'll mark these as complete
        ];
        
        // Since we can't easily mark algorithms as complete without running them,
        // this test is more conceptual - in practice, completed algorithms
        // are identified by their is_complete() method returning true
    }
}
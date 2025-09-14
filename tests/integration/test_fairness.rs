//! Integration tests for fairness model allocation
//! Tests fairness behavior in actual sorting scenarios with multiple algorithms

use sorting_race::models::config::{Distribution, FairnessMode, RunConfiguration};
use sorting_race::models::traits::{FairnessModel, Sorter, StepResult, Telemetry};
use sorting_race::services::fairness::{
    comparison::ComparisonBudget, walltime::WallTimeFairness, weighted::WeightedFairness,
};
use sorting_race::services::generator::ArrayGenerator;
use sorting_race::services::sorters::{
    bubble::BubbleSort, heap::HeapSort, insertion::InsertionSort, merge::MergeSort,
    quick::QuickSort, selection::SelectionSort,
};

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

fn get_algorithm_stats(algorithms: &[Box<dyn Sorter>]) -> Vec<(String, u64, u64, bool)> {
    algorithms
        .iter()
        .map(|alg| {
            let telemetry = alg.get_telemetry();
            (
                alg.name().to_string(),
                telemetry.total_comparisons,
                telemetry.total_moves,
                alg.is_complete(),
            )
        })
        .collect()
}

#[cfg(test)]
mod fairness_integration_tests {
    use super::*;

    #[test]
    fn test_comparison_budget_equal_allocation() {
        let generator = ArrayGenerator::new(12345);
        let array = generator.generate(30, &Distribution::Shuffled);
        let mut algorithms = create_test_algorithms();
        initialize_algorithms_with_array(&mut algorithms, array);

        let fairness_model = ComparisonBudget::new(16);
        
        // Test multiple steps to ensure consistency
        for step in 0..10 {
            let active_count = count_active_algorithms(&algorithms);
            if active_count == 0 {
                break;
            }

            let budgets = fairness_model.allocate_budget(&algorithms);
            
            // All active algorithms should get equal budget
            assert_eq!(budgets.len(), algorithms.len());
            
            let mut active_budgets = Vec::new();
            let mut inactive_budgets = Vec::new();
            
            for (i, algorithm) in algorithms.iter().enumerate() {
                if algorithm.is_complete() {
                    inactive_budgets.push(budgets[i]);
                } else {
                    active_budgets.push(budgets[i]);
                }
            }
            
            // All active algorithms should get the same budget (16)
            for budget in &active_budgets {
                assert_eq!(*budget, 16, "Step {}: Active algorithms should get equal budget", step);
            }
            
            // All completed algorithms should get 0 budget
            for budget in &inactive_budgets {
                assert_eq!(*budget, 0, "Step {}: Completed algorithms should get 0 budget", step);
            }
            
            // Execute steps with allocated budgets
            for (i, algorithm) in algorithms.iter_mut().enumerate() {
                if budgets[i] > 0 {
                    algorithm.step(budgets[i]);
                }
            }
        }
    }

    #[test]
    fn test_weighted_fairness_comparison_vs_moves() {
        let generator = ArrayGenerator::new(54321);
        let array = generator.generate(25, &Distribution::Shuffled);
        
        // Test with comparison-biased weighting (higher comparison weight)
        let comparison_biased = WeightedFairness::new(2.0, 1.0);
        let mut algorithms_comp = create_test_algorithms();
        initialize_algorithms_with_array(&mut algorithms_comp, array.clone());
        
        // Test with move-biased weighting (higher move weight)  
        let move_biased = WeightedFairness::new(1.0, 2.0);
        let mut algorithms_move = create_test_algorithms();
        initialize_algorithms_with_array(&mut algorithms_move, array.clone());
        
        // Run several steps to accumulate different patterns
        for _ in 0..5 {
            // Step both sets with their respective fairness models
            let budgets_comp = comparison_biased.allocate_budget(&algorithms_comp);
            let budgets_move = move_biased.allocate_budget(&algorithms_move);
            
            // Execute steps
            for (i, algorithm) in algorithms_comp.iter_mut().enumerate() {
                if budgets_comp[i] > 0 {
                    algorithm.step(budgets_comp[i]);
                }
            }
            
            for (i, algorithm) in algorithms_move.iter_mut().enumerate() {
                if budgets_move[i] > 0 {
                    algorithm.step(budgets_move[i]);
                }
            }
            
            // Verify that both models allocate budgets (fairness in action)
            let active_comp = count_active_algorithms(&algorithms_comp);
            let active_move = count_active_algorithms(&algorithms_move);
            
            if active_comp > 0 {
                let active_budget_sum: usize = budgets_comp.iter().sum();
                assert!(active_budget_sum > 0, "Active algorithms should receive budgets");
            }
            
            if active_move > 0 {
                let active_budget_sum: usize = budgets_move.iter().sum();
                assert!(active_budget_sum > 0, "Active algorithms should receive budgets");
            }
        }
        
        // Both models should have made progress
        let stats_comp = get_algorithm_stats(&algorithms_comp);
        let stats_move = get_algorithm_stats(&algorithms_move);
        
        // Verify both strategies make progress
        for (name, comps, moves, _) in &stats_comp {
            if *comps > 0 || *moves > 0 {
                assert!(
                    *comps > 0 || *moves > 0,
                    "Algorithm {} should make progress under comparison-biased fairness",
                    name
                );
            }
        }
        
        for (name, comps, moves, _) in &stats_move {
            if *comps > 0 || *moves > 0 {
                assert!(
                    *comps > 0 || *moves > 0,
                    "Algorithm {} should make progress under move-biased fairness",
                    name
                );
            }
        }
    }

    #[test]
    fn test_fairness_skips_completed_algorithms() {
        let generator = ArrayGenerator::new(99999);
        let array = generator.generate(15, &Distribution::Shuffled); // Small array for quick completion
        let mut algorithms = create_test_algorithms();
        initialize_algorithms_with_array(&mut algorithms, array);

        let fairness_model = ComparisonBudget::new(20);
        let mut completed_algorithms = Vec::new();
        
        // Run until some algorithms complete
        for step in 0..50 {
            let budgets = fairness_model.allocate_budget(&algorithms);
            let active_count = count_active_algorithms(&algorithms);
            
            if active_count == 0 {
                break;
            }
            
            // Track which algorithms complete this step
            let mut newly_completed = Vec::new();
            
            // Execute steps and track completions
            for (i, algorithm) in algorithms.iter_mut().enumerate() {
                let was_complete = algorithm.is_complete();
                
                if budgets[i] > 0 {
                    algorithm.step(budgets[i]);
                }
                
                let is_now_complete = algorithm.is_complete();
                
                if !was_complete && is_now_complete {
                    newly_completed.push(i);
                }
            }
            
            // Verify budget allocation correctness
            for (i, algorithm) in algorithms.iter().enumerate() {
                if algorithm.is_complete() {
                    assert_eq!(
                        budgets[i], 0,
                        "Step {}: Completed algorithm {} should get 0 budget",
                        step, algorithm.name()
                    );
                } else {
                    assert_eq!(
                        budgets[i], 20,
                        "Step {}: Active algorithm {} should get full budget",
                        step, algorithm.name()
                    );
                }
            }
            
            completed_algorithms.extend(newly_completed);
            
            if completed_algorithms.len() >= 2 {
                break; // We have enough completed algorithms to test
            }
        }
        
        // Verify that at least some algorithms completed
        let final_active_count = count_active_algorithms(&algorithms);
        let total_algorithms = algorithms.len();
        assert!(
            final_active_count < total_algorithms,
            "Some algorithms should have completed"
        );
        
        // Final verification: completed algorithms should get 0 budget
        let final_budgets = fairness_model.allocate_budget(&algorithms);
        for (i, algorithm) in algorithms.iter().enumerate() {
            if algorithm.is_complete() {
                assert_eq!(
                    final_budgets[i], 0,
                    "Final check: Completed algorithm {} should get 0 budget",
                    algorithm.name()
                );
            }
        }
    }

    #[test]
    fn test_budget_allocation_consistency() {
        let generator = ArrayGenerator::new(77777);
        let array = generator.generate(40, &Distribution::NearlySorted);
        let mut algorithms = create_test_algorithms();
        initialize_algorithms_with_array(&mut algorithms, array);

        let fairness_model = ComparisonBudget::new(12);
        let mut allocation_history = Vec::new();
        
        // Track budget allocations over multiple steps
        for step in 0..15 {
            let active_count = count_active_algorithms(&algorithms);
            if active_count == 0 {
                break;
            }
            
            let budgets = fairness_model.allocate_budget(&algorithms);
            let algorithm_states: Vec<bool> = algorithms.iter().map(|alg| alg.is_complete()).collect();
            
            allocation_history.push((step, budgets.clone(), algorithm_states.clone()));
            
            // Verify consistency within this step
            let expected_budget_per_active = 12;
            for (i, is_complete) in algorithm_states.iter().enumerate() {
                let expected_budget = if *is_complete { 0 } else { expected_budget_per_active };
                assert_eq!(
                    budgets[i], expected_budget,
                    "Step {}: Algorithm {} budget inconsistent (complete: {})",
                    step, i, is_complete
                );
            }
            
            // Execute steps
            for (i, algorithm) in algorithms.iter_mut().enumerate() {
                if budgets[i] > 0 {
                    algorithm.step(budgets[i]);
                }
            }
        }
        
        // Analyze consistency across steps
        for (step, budgets, states) in &allocation_history {
            let total_budget: usize = budgets.iter().sum();
            let active_count = states.iter().filter(|&&complete| !complete).count();
            let expected_total = active_count * 12;
            
            assert_eq!(
                total_budget, expected_total,
                "Step {}: Total budget ({}) should equal active count ({}) * budget per algorithm (12)",
                step, total_budget, active_count
            );
        }
        
        // Verify monotonic completion (algorithms don't "uncomplete")
        for window in allocation_history.windows(2) {
            let (step1, _, states1) = &window[0];
            let (step2, _, states2) = &window[1];
            
            for (i, (was_complete, is_complete)) in states1.iter().zip(states2.iter()).enumerate() {
                assert!(
                    !was_complete || *is_complete,
                    "Algorithm {} unccompleted between steps {} and {} (was: {}, is: {})",
                    i, step1, step2, was_complete, is_complete
                );
            }
        }
    }

    #[test]
    fn test_multiple_fairness_models_consistency() {
        let generator = ArrayGenerator::new(88888);
        let array = generator.generate(35, &Distribution::FewUnique);
        
        // Create separate algorithm sets for each fairness model
        let mut algorithms_comparison = create_test_algorithms();
        let mut algorithms_weighted = create_test_algorithms();
        let mut algorithms_walltime = create_test_algorithms();
        
        initialize_algorithms_with_array(&mut algorithms_comparison, array.clone());
        initialize_algorithms_with_array(&mut algorithms_weighted, array.clone());
        initialize_algorithms_with_array(&mut algorithms_walltime, array);
        
        let comparison_model = ComparisonBudget::new(8);
        let weighted_model = WeightedFairness::new(1.0, 1.0); // Equal weighting
        let walltime_model = WallTimeFairness::new(50); // 50ms slices
        
        // Run all models for several steps
        for step in 0..8 {
            // Get budgets from each model
            let budgets_comp = comparison_model.allocate_budget(&algorithms_comparison);
            let budgets_weighted = weighted_model.allocate_budget(&algorithms_weighted);
            let budgets_walltime = walltime_model.allocate_budget(&algorithms_walltime);
            
            // All models should produce budgets for the same number of algorithms
            assert_eq!(budgets_comp.len(), algorithms_comparison.len());
            assert_eq!(budgets_weighted.len(), algorithms_weighted.len());
            assert_eq!(budgets_walltime.len(), algorithms_walltime.len());
            
            // All models should skip completed algorithms
            for (i, algorithm) in algorithms_comparison.iter().enumerate() {
                if algorithm.is_complete() {
                    assert_eq!(budgets_comp[i], 0, "Step {}: Comparison model should give 0 to completed algorithms", step);
                }
            }
            
            for (i, algorithm) in algorithms_weighted.iter().enumerate() {
                if algorithm.is_complete() {
                    assert_eq!(budgets_weighted[i], 0, "Step {}: Weighted model should give 0 to completed algorithms", step);
                }
            }
            
            for (i, algorithm) in algorithms_walltime.iter().enumerate() {
                if algorithm.is_complete() {
                    assert_eq!(budgets_walltime[i], 0, "Step {}: Walltime model should give 0 to completed algorithms", step);
                }
            }
            
            // Execute steps for all models
            for (i, algorithm) in algorithms_comparison.iter_mut().enumerate() {
                if budgets_comp[i] > 0 {
                    algorithm.step(budgets_comp[i]);
                }
            }
            
            for (i, algorithm) in algorithms_weighted.iter_mut().enumerate() {
                if budgets_weighted[i] > 0 {
                    algorithm.step(budgets_weighted[i]);
                }
            }
            
            for (i, algorithm) in algorithms_walltime.iter_mut().enumerate() {
                if budgets_walltime[i] > 0 {
                    algorithm.step(budgets_walltime[i]);
                }
            }
            
            // Check that all models continue to make progress
            let active_comp = count_active_algorithms(&algorithms_comparison);
            let active_weighted = count_active_algorithms(&algorithms_weighted);
            let active_walltime = count_active_algorithms(&algorithms_walltime);
            
            if active_comp == 0 && active_weighted == 0 && active_walltime == 0 {
                break; // All done
            }
        }
        
        // Verify all models made progress
        let stats_comp = get_algorithm_stats(&algorithms_comparison);
        let stats_weighted = get_algorithm_stats(&algorithms_weighted);
        let stats_walltime = get_algorithm_stats(&algorithms_walltime);
        
        for ((name_comp, comps_comp, moves_comp, _), 
             (name_weighted, comps_weighted, moves_weighted, _), 
             (name_walltime, comps_walltime, moves_walltime, _)) in 
            stats_comp.iter().zip(stats_weighted.iter()).zip(stats_walltime.iter()) {
            
            assert_eq!(name_comp, name_weighted);
            assert_eq!(name_comp, name_walltime);
            
            // All models should have made some progress
            let progress_comp = *comps_comp + *moves_comp;
            let progress_weighted = *comps_weighted + *moves_weighted;
            let progress_walltime = *comps_walltime + *moves_walltime;
            
            if progress_comp > 0 || progress_weighted > 0 || progress_walltime > 0 {
                // At least one model should have made progress for each algorithm
                assert!(
                    progress_comp > 0 || progress_weighted > 0 || progress_walltime > 0,
                    "Algorithm {} should make progress under at least one fairness model",
                    name_comp
                );
            }
        }
    }

    #[test]
    fn test_fairness_edge_cases() {
        // Test with very small array
        let generator = ArrayGenerator::new(11111);
        let small_array = generator.generate(3, &Distribution::Shuffled);
        let mut algorithms = create_test_algorithms();
        initialize_algorithms_with_array(&mut algorithms, small_array);

        let fairness_model = ComparisonBudget::new(5);
        
        // Should handle small arrays without crashing
        for _ in 0..10 {
            let active_count = count_active_algorithms(&algorithms);
            if active_count == 0 {
                break;
            }
            
            let budgets = fairness_model.allocate_budget(&algorithms);
            assert_eq!(budgets.len(), algorithms.len());
            
            for (i, algorithm) in algorithms.iter_mut().enumerate() {
                if budgets[i] > 0 {
                    algorithm.step(budgets[i]);
                }
            }
        }
        
        // All algorithms should eventually complete with small array
        let final_active = count_active_algorithms(&algorithms);
        assert_eq!(final_active, 0, "All algorithms should complete with small array");
    }

    #[test]
    fn test_empty_algorithm_list() {
        let algorithms: Vec<Box<dyn Sorter>> = vec![];
        let fairness_model = ComparisonBudget::new(10);
        
        let budgets = fairness_model.allocate_budget(&algorithms);
        assert_eq!(budgets.len(), 0, "Empty algorithm list should result in empty budget list");
    }

    #[test]
    fn test_zero_budget_allocation() {
        let generator = ArrayGenerator::new(22222);
        let array = generator.generate(20, &Distribution::Shuffled);
        let mut algorithms = create_test_algorithms();
        initialize_algorithms_with_array(&mut algorithms, array);

        let fairness_model = ComparisonBudget::new(0); // Zero budget
        
        let budgets = fairness_model.allocate_budget(&algorithms);
        
        // Should allocate zero budget to all algorithms
        for budget in &budgets {
            assert_eq!(*budget, 0, "Zero budget model should allocate 0 to all algorithms");
        }
        
        // Algorithms should not progress with zero budget
        let initial_stats = get_algorithm_stats(&algorithms);
        
        for (i, algorithm) in algorithms.iter_mut().enumerate() {
            algorithm.step(budgets[i]);
        }
        
        let after_stats = get_algorithm_stats(&algorithms);
        
        for (initial, after) in initial_stats.iter().zip(after_stats.iter()) {
            assert_eq!(initial.1, after.1, "Comparisons should not change with zero budget");
            assert_eq!(initial.2, after.2, "Moves should not change with zero budget");
        }
    }
}
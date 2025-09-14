//! Integration test for deterministic execution
//! Verifies that same seed produces identical results

use sorting_race::models::config::{Distribution, RunConfiguration};
use sorting_race::models::traits::Sorter;
use sorting_race::services::generator::ArrayGenerator;
use sorting_race::services::sorters::{
    bubble::BubbleSort, heap::HeapSort, insertion::InsertionSort, merge::MergeSort,
    quick::QuickSort, selection::SelectionSort,
};

fn create_test_config(seed: u64, size: usize) -> RunConfiguration {
    RunConfiguration {
        array_size: size,
        distribution: Distribution::Shuffled,
        seed,
        fairness_mode: sorting_race::models::config::FairnessMode::ComparisonBudget { k: 16 },
        target_fps: 30,
    }
}

fn run_sorting_simulation(config: &RunConfiguration) -> Vec<(String, u64, u64)> {
    let generator = ArrayGenerator::new(config.seed);
    let array = generator.generate(config.array_size, &config.distribution);
    
    let mut sorters: Vec<Box<dyn Sorter>> = vec![
        Box::new(BubbleSort::new()),
        Box::new(InsertionSort::new()),
        Box::new(SelectionSort::new()),
        Box::new(QuickSort::new()),
        Box::new(HeapSort::new()),
        Box::new(MergeSort::new()),
    ];
    
    let mut results = Vec::new();
    
    for sorter in &mut sorters {
        sorter.reset(array.clone());
        
        let mut total_comparisons = 0;
        let mut total_moves = 0;
        let mut steps = 0;
        
        while !sorter.is_complete() && steps < 10000 {
            let result = sorter.step(16);
            total_comparisons += result.comparisons_used as u64;
            total_moves += result.moves_made as u64;
            steps += 1;
        }
        
        let telemetry = sorter.get_telemetry();
        results.push((
            sorter.name().to_string(),
            telemetry.total_comparisons,
            telemetry.total_moves,
        ));
    }
    
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_same_seed() {
        let config = create_test_config(12345, 50);
        
        // Run simulation twice with same seed
        let run1 = run_sorting_simulation(&config);
        let run2 = run_sorting_simulation(&config);
        
        // Results should be identical
        assert_eq!(run1.len(), run2.len());
        for (r1, r2) in run1.iter().zip(run2.iter()) {
            assert_eq!(r1.0, r2.0, "Algorithm names should match");
            assert_eq!(r1.1, r2.1, "Comparisons should be identical for {}", r1.0);
            assert_eq!(r1.2, r2.2, "Moves should be identical for {}", r1.0);
        }
    }

    #[test]
    fn test_different_seeds_different_results() {
        let config1 = create_test_config(11111, 50);
        let config2 = create_test_config(22222, 50);
        
        let run1 = run_sorting_simulation(&config1);
        let run2 = run_sorting_simulation(&config2);
        
        // At least some results should differ
        let mut found_difference = false;
        for (r1, r2) in run1.iter().zip(run2.iter()) {
            if r1.1 != r2.1 || r1.2 != r2.2 {
                found_difference = true;
                break;
            }
        }
        
        assert!(found_difference, "Different seeds should produce different results");
    }

    #[test]
    fn test_deterministic_array_generation() {
        let gen1 = ArrayGenerator::new(99999);
        let gen2 = ArrayGenerator::new(99999);
        
        let array1 = gen1.generate(100, &Distribution::Shuffled);
        let array2 = gen2.generate(100, &Distribution::Shuffled);
        
        assert_eq!(array1, array2, "Same seed should generate identical arrays");
    }

    #[test]
    fn test_deterministic_distributions() {
        let seed = 54321;
        let size = 30;
        
        for distribution in [
            Distribution::Shuffled,
            Distribution::NearlySorted,
            Distribution::Reversed,
            Distribution::FewUnique,
        ] {
            let mut config = create_test_config(seed, size);
            config.distribution = distribution.clone();
            
            let run1 = run_sorting_simulation(&config);
            let run2 = run_sorting_simulation(&config);
            
            for (r1, r2) in run1.iter().zip(run2.iter()) {
                assert_eq!(
                    r1.1, r2.1,
                    "Determinism failed for {:?} distribution",
                    distribution
                );
            }
        }
    }

    #[test]
    fn test_deterministic_small_arrays() {
        for size in [0, 1, 2, 5, 10] {
            let config = create_test_config(7777, size);
            
            let run1 = run_sorting_simulation(&config);
            let run2 = run_sorting_simulation(&config);
            
            assert_eq!(run1, run2, "Failed determinism for array size {}", size);
        }
    }

    #[test]
    fn test_deterministic_step_by_step() {
        let config = create_test_config(8888, 20);
        let generator = ArrayGenerator::new(config.seed);
        let array = generator.generate(config.array_size, &config.distribution);
        
        let mut sorter1 = BubbleSort::new();
        let mut sorter2 = BubbleSort::new();
        
        sorter1.reset(array.clone());
        sorter2.reset(array.clone());
        
        // Step both sorters identically
        for _ in 0..100 {
            if sorter1.is_complete() {
                break;
            }
            
            let result1 = sorter1.step(1); // One comparison at a time
            let result2 = sorter2.step(1);
            
            assert_eq!(
                result1.comparisons_used, result2.comparisons_used,
                "Step comparisons should match"
            );
            assert_eq!(
                result1.moves_made, result2.moves_made,
                "Step moves should match"
            );
            assert_eq!(
                result1.continued, result2.continued,
                "Step continuation should match"
            );
        }
        
        assert_eq!(
            sorter1.get_array(),
            sorter2.get_array(),
            "Final arrays should be identical"
        );
    }
}
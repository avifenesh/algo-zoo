use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use sorting_race::services::sorters::{
    bubble::BubbleSort,
    heap::HeapSort,
    insertion::InsertionSort,
    merge::MergeSort,
    quick::QuickSort,
    selection::SelectionSort,
    shell::ShellSort,
};
use sorting_race::models::traits::Sorter;

fn generate_test_data(size: usize, pattern: &str) -> Vec<i32> {
    match pattern {
        "random" => {
            let mut data = Vec::with_capacity(size);
            let mut seed = 42u64;
            for _ in 0..size {
                seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
                data.push((seed % 1000) as i32);
            }
            data
        }
        "sorted" => (0..size as i32).collect(),
        "reversed" => (0..size as i32).rev().collect(),
        "nearly_sorted" => {
            let mut data: Vec<i32> = (0..size as i32).collect();
            // Swap a few elements
            for i in (0..size/10).step_by(2) {
                data.swap(i, i + 1);
            }
            data
        }
        "few_unique" => {
            (0..size).map(|i| (i % 10) as i32).collect()
        }
        _ => vec![],
    }
}

fn benchmark_algorithm<S: Sorter>(
    c: &mut Criterion,
    name: &str,
    mut sorter: S,
    sizes: &[usize],
    patterns: &[&str],
) {
    let mut group = c.benchmark_group(name);
    
    for &size in sizes {
        for &pattern in patterns {
            let data = generate_test_data(size, pattern);
            
            group.bench_with_input(
                BenchmarkId::new(pattern, size),
                &data,
                |b, data| {
                    b.iter(|| {
                        sorter.reset(data.clone());
                        while !sorter.is_complete() {
                            black_box(sorter.step(16));
                        }
                    });
                }
            );
        }
    }
    
    group.finish();
}

fn benchmark_all_algorithms(c: &mut Criterion) {
    let sizes = vec![10, 50, 100, 500];
    let patterns = vec!["random", "sorted", "reversed", "nearly_sorted", "few_unique"];
    
    benchmark_algorithm(c, "BubbleSort", BubbleSort::new(), &sizes[..2], &patterns);
    benchmark_algorithm(c, "InsertionSort", InsertionSort::new(), &sizes[..2], &patterns);
    benchmark_algorithm(c, "SelectionSort", SelectionSort::new(), &sizes[..2], &patterns);
    benchmark_algorithm(c, "QuickSort", QuickSort::new(), &sizes, &patterns);
    benchmark_algorithm(c, "HeapSort", HeapSort::new(), &sizes, &patterns);
    benchmark_algorithm(c, "MergeSort", MergeSort::new(), &sizes, &patterns);
    benchmark_algorithm(c, "ShellSort", ShellSort::new(), &sizes, &patterns);
}

fn benchmark_budget_impact(c: &mut Criterion) {
    let mut group = c.benchmark_group("budget_impact");
    let data = generate_test_data(100, "random");
    let budgets = vec![1, 4, 8, 16, 32, 64];
    
    for budget in budgets {
        group.bench_with_input(
            BenchmarkId::new("QuickSort", budget),
            &(data.clone(), budget),
            |b, (data, budget)| {
                b.iter(|| {
                    let mut sorter = QuickSort::new();
                    sorter.reset(data.clone());
                    let mut steps = 0;
                    while !sorter.is_complete() && steps < 1000 {
                        black_box(sorter.step(*budget));
                        steps += 1;
                    }
                });
            }
        );
    }
    
    group.finish();
}

fn benchmark_comparison_counts(c: &mut Criterion) {
    let mut group = c.benchmark_group("comparisons");
    let sizes = vec![50, 100, 200];
    
    for size in sizes {
        let data = generate_test_data(size, "random");
        
        // Test each algorithm
        let algorithms: Vec<(&str, Box<dyn Sorter>)> = vec![
            ("QuickSort", Box::new(QuickSort::new())),
            ("HeapSort", Box::new(HeapSort::new())),
            ("MergeSort", Box::new(MergeSort::new())),
            ("ShellSort", Box::new(ShellSort::new())),
        ];
        
        for (name, mut sorter) in algorithms {
            group.bench_with_input(
                BenchmarkId::new(name, size),
                &data,
                |b, data| {
                    b.iter(|| {
                        sorter.reset(data.clone());
                        let mut total_comparisons = 0;
                        while !sorter.is_complete() {
                            let result = sorter.step(16);
                            total_comparisons += result.comparisons_used;
                        }
                        black_box(total_comparisons);
                    });
                }
            );
        }
    }
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_all_algorithms,
    benchmark_budget_impact,
    benchmark_comparison_counts
);
criterion_main!(benches);
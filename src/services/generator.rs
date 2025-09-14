//! Deterministic array generation for sorting race

use crate::models::config::Distribution;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

/// Deterministic array generator using seeded RNG
#[derive(Debug)]
pub struct ArrayGenerator {
    seed: u64,
}

impl ArrayGenerator {
    /// Create a new array generator with specified seed
    pub fn new(seed: u64) -> Self {
        Self { seed }
    }

    /// Generate an array with the specified distribution
    pub fn generate(&self, size: usize, distribution: &Distribution) -> Vec<i32> {
        if size == 0 {
            return Vec::new();
        }

        match distribution {
            Distribution::Shuffled => self.generate_shuffled(size),
            Distribution::NearlySorted => self.generate_nearly_sorted(size),
            Distribution::Reversed => self.generate_reversed(size),
            Distribution::FewUnique => self.generate_few_unique(size),
            Distribution::Sorted => self.generate_sorted(size),
            Distribution::WithDuplicates => self.generate_with_duplicates(size),
        }
    }

    /// Generate a random shuffled array
    fn generate_shuffled(&self, size: usize) -> Vec<i32> {
        let mut rng = StdRng::seed_from_u64(self.seed);
        let mut array: Vec<i32> = (1..=size as i32).collect();
        
        // Fisher-Yates shuffle
        for i in (1..size).rev() {
            let j = rng.random_range(0..=i);
            array.swap(i, j);
        }
        
        array
    }

    /// Generate a nearly sorted array (90% sorted, 10% out of place)
    fn generate_nearly_sorted(&self, size: usize) -> Vec<i32> {
        let mut rng = StdRng::seed_from_u64(self.seed);
        let mut array: Vec<i32> = (1..=size as i32).collect();
        
        // Randomly swap ~10% of elements
        let swaps = (size / 10).max(1);
        for _ in 0..swaps {
            let i = rng.random_range(0..size);
            let j = rng.random_range(0..size);
            array.swap(i, j);
        }
        
        array
    }

    /// Generate a reverse sorted array
    fn generate_reversed(&self, size: usize) -> Vec<i32> {
        (1..=size as i32).rev().collect()
    }

    /// Generate an array with few unique values
    fn generate_few_unique(&self, size: usize) -> Vec<i32> {
        let mut rng = StdRng::seed_from_u64(self.seed);
        let unique_count = (size / 10).max(3).min(size); // ~10% unique values
        let values: Vec<i32> = (1..=unique_count as i32).collect();
        
        let mut array = Vec::with_capacity(size);
        for _ in 0..size {
            let idx = rng.random_range(0..values.len());
            array.push(values[idx]);
        }
        
        array
    }

    /// Generate a pre-sorted array
    fn generate_sorted(&self, size: usize) -> Vec<i32> {
        (1..=size as i32).collect()
    }

    /// Generate an array with many duplicates
    fn generate_with_duplicates(&self, size: usize) -> Vec<i32> {
        let mut rng = StdRng::seed_from_u64(self.seed);
        let mut array = Vec::with_capacity(size);
        
        // Generate with 50% duplicates
        for i in 0..size {
            if i < size / 2 {
                // First half: sequential values
                array.push((i + 1) as i32);
            } else {
                // Second half: random duplicates from first half
                let idx = rng.random_range(0..size / 2);
                array.push((idx + 1) as i32);
            }
        }
        
        // Shuffle the array
        for i in (1..size).rev() {
            let j = rng.random_range(0..=i);
            array.swap(i, j);
        }
        
        array
    }

    /// Validate that the generated array is correct
    pub fn validate_array(array: &[i32], size: usize, distribution: &Distribution) -> bool {
        if array.len() != size {
            return false;
        }

        if size == 0 {
            return true;
        }

        match distribution {
            Distribution::Sorted => {
                // Check if array is sorted
                array.windows(2).all(|w| w[0] <= w[1])
            }
            Distribution::Reversed => {
                // Check if array is reverse sorted
                array.windows(2).all(|w| w[0] >= w[1])
            }
            _ => {
                // For other distributions, just check that values are reasonable
                array.iter().all(|&x| x > 0 && x <= size as i32 * 2)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_generation() {
        let gen1 = ArrayGenerator::new(42);
        let gen2 = ArrayGenerator::new(42);
        
        let array1 = gen1.generate(10, &Distribution::Shuffled);
        let array2 = gen2.generate(10, &Distribution::Shuffled);
        
        assert_eq!(array1, array2);
    }

    #[test]
    fn test_different_seeds() {
        let gen1 = ArrayGenerator::new(42);
        let gen2 = ArrayGenerator::new(43);
        
        let array1 = gen1.generate(10, &Distribution::Shuffled);
        let array2 = gen2.generate(10, &Distribution::Shuffled);
        
        assert_ne!(array1, array2);
    }

    #[test]
    fn test_sorted_distribution() {
        let generator = ArrayGenerator::new(42);
        let array = generator.generate(10, &Distribution::Sorted);
        
        assert_eq!(array, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    }

    #[test]
    fn test_reversed_distribution() {
        let generator = ArrayGenerator::new(42);
        let array = generator.generate(10, &Distribution::Reversed);
        
        assert_eq!(array, vec![10, 9, 8, 7, 6, 5, 4, 3, 2, 1]);
    }

    #[test]
    fn test_empty_array() {
        let generator = ArrayGenerator::new(42);
        let array = generator.generate(0, &Distribution::Shuffled);
        
        assert!(array.is_empty());
    }

    #[test]
    fn test_array_validation() {
        let generator = ArrayGenerator::new(42);
        
        let sorted = generator.generate(5, &Distribution::Sorted);
        assert!(ArrayGenerator::validate_array(&sorted, 5, &Distribution::Sorted));
        
        let reversed = generator.generate(5, &Distribution::Reversed);
        assert!(ArrayGenerator::validate_array(&reversed, 5, &Distribution::Reversed));
        
        let shuffled = generator.generate(5, &Distribution::Shuffled);
        assert!(ArrayGenerator::validate_array(&shuffled, 5, &Distribution::Shuffled));
    }
}
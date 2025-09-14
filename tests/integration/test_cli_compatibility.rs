use assert_cmd::Command;
use predicates::prelude::*;
use std::process::Command as StdCommand;

#[cfg(test)]
mod cli_compatibility_tests {
    use super::*;
    
    /// Integration test for CLI compatibility preservation
    /// 
    /// From quickstart.md: All existing v0.1 CLI options should function as before
    /// This ensures backward compatibility while adding interactive features
    #[test]
    fn test_existing_cli_options_still_work() {
        // Test that all v0.1 CLI functionality is preserved in v0.2
        
        let test_cases = vec![
            // Basic size and distribution options
            vec!["--size", "100", "--distribution", "reversed"],
            vec!["--size", "50", "--distribution", "shuffled"],
            vec!["--size", "200", "--distribution", "nearly-sorted"],
            vec!["--size", "25", "--distribution", "few-unique"],
            
            // Fairness mode options
            vec!["--fair", "comp", "--budget", "16"],
            vec!["--fair", "weighted", "--alpha", "2.0", "--beta", "0.5"],
            vec!["--fair", "walltime"],
            vec!["--fair", "adaptive", "--learning-rate", "0.3"],
            
            // Combined options
            vec!["--size", "100", "--distribution", "reversed", "--fair", "comp", "--budget", "8"],
            vec!["--size", "500", "--fair", "weighted", "--alpha", "1.5", "--beta", "1.0"],
            vec!["--size", "50", "--distribution", "few-unique", "--fair", "walltime"],
        ];
        
        for args in test_cases {
            let mut cmd = Command::cargo_bin("sorting-race").unwrap();
            let assertion = cmd
                .args(&args)
                .timeout(std::time::Duration::from_secs(10))
                .assert();
            
            // Should succeed (exit code 0) and produce expected output
            assertion
                .success()
                .stdout(predicate::str::contains("Race completed"))
                .stdout(predicate::str::contains("Winner:"));
        }
    }
    
    #[test]
    fn test_cli_help_output_includes_new_options() {
        // Test that CLI help shows both original and new options
        
        let mut cmd = Command::cargo_bin("sorting-race").unwrap();
        let assertion = cmd.arg("--help").assert();
        
        assertion
            .success()
            // Original v0.1 options should still be present
            .stdout(predicate::str::contains("--size"))
            .stdout(predicate::str::contains("--distribution"))
            .stdout(predicate::str::contains("--fair"))
            .stdout(predicate::str::contains("--budget"))
            .stdout(predicate::str::contains("--alpha"))
            .stdout(predicate::str::contains("--beta"))
            .stdout(predicate::str::contains("--learning-rate"))
            
            // v0.2 should mention interactive mode capability
            .stdout(predicate::str::contains("interactive").or(predicate::str::contains("Interactive")));
    }
    
    #[test]
    fn test_cli_without_args_launches_interactive_mode() {
        // Test that running without arguments launches interactive mode (v0.2 behavior)
        
        let mut cmd = Command::cargo_bin("sorting-race").unwrap();
        let assertion = cmd
            .timeout(std::time::Duration::from_secs(5))
            .assert();
        
        // Should start successfully but may timeout in interactive mode (expected)
        // In interactive mode, it would wait for keyboard input
        assertion
            .success()
            .stdout(predicate::str::contains("Configuration").or(
                predicate::str::contains("Interactive").or(
                    predicate::str::contains("Press")
                )
            ));
    }
    
    #[test]
    fn test_cli_error_handling_preserved() {
        // Test that CLI error handling for invalid arguments is preserved
        
        let error_test_cases = vec![
            // Invalid size values
            vec!["--size", "0"],
            vec!["--size", "-1"],
            vec!["--size", "abc"],
            vec!["--size", "10000"], // Too large
            
            // Invalid distribution values
            vec!["--distribution", "invalid"],
            vec!["--distribution", "random"], // Not a valid option
            
            // Invalid fairness combinations
            vec!["--fair", "comp"], // Missing required budget
            vec!["--fair", "weighted"], // Missing required alpha/beta
            vec!["--fair", "adaptive"], // Missing required learning rate
            vec!["--fair", "invalid"],
            
            // Invalid parameter values
            vec!["--fair", "comp", "--budget", "0"],
            vec!["--fair", "comp", "--budget", "-5"],
            vec!["--fair", "weighted", "--alpha", "-1", "--beta", "0.5"],
            vec!["--fair", "adaptive", "--learning-rate", "0"], // Below minimum
            vec!["--fair", "adaptive", "--learning-rate", "2.0"], // Above maximum
        ];
        
        for args in error_test_cases {
            let mut cmd = Command::cargo_bin("sorting-race").unwrap();
            let assertion = cmd
                .args(&args)
                .timeout(std::time::Duration::from_secs(5))
                .assert();
            
            // Should fail with non-zero exit code and show error message
            assertion
                .failure()
                .stderr(predicate::str::contains("error").or(
                    predicate::str::contains("Error").or(
                        predicate::str::contains("invalid")
                    )
                ));
        }
    }
    
    #[test]
    fn test_cli_output_format_unchanged() {
        // Test that CLI output format is compatible with v0.1 expectations
        
        let mut cmd = Command::cargo_bin("sorting-race").unwrap();
        let assertion = cmd
            .args(&["--size", "50", "--fair", "walltime"])
            .timeout(std::time::Duration::from_secs(10))
            .assert();
        
        assertion
            .success()
            // Should contain expected output format elements
            .stdout(predicate::str::contains("Array size:"))
            .stdout(predicate::str::contains("Distribution:"))
            .stdout(predicate::str::contains("Fairness mode:"))
            .stdout(predicate::str::contains("algorithms"))
            .stdout(predicate::str::contains("Race completed"))
            .stdout(predicate::str::contains("Winner:"))
            // Memory usage should now show actual values (bug fix)
            .stdout(predicate::str::contains("KB").or(predicate::str::contains("MB")).or(predicate::str::contains("B")));
    }
    
    #[test]
    fn test_specific_v01_regression_scenarios() {
        // Test specific scenarios that worked in v0.1 to ensure no regressions
        
        // Scenario 1: Large array with comparison fairness
        let mut cmd1 = Command::cargo_bin("sorting-race").unwrap();
        cmd1.args(&["--size", "500", "--distribution", "reversed", "--fair", "comp", "--budget", "16"])
            .timeout(std::time::Duration::from_secs(15))
            .assert()
            .success()
            .stdout(predicate::str::contains("500"))
            .stdout(predicate::str::contains("reversed"))
            .stdout(predicate::str::contains("Comparison"));
        
        // Scenario 2: Weighted fairness with custom parameters
        let mut cmd2 = Command::cargo_bin("sorting-race").unwrap();
        cmd2.args(&["--size", "100", "--fair", "weighted", "--alpha", "1.5", "--beta", "2.0"])
            .timeout(std::time::Duration::from_secs(10))
            .assert()
            .success()
            .stdout(predicate::str::contains("Weighted"))
            .stdout(predicate::str::contains("1.5"))
            .stdout(predicate::str::contains("2.0"));
        
        // Scenario 3: Adaptive fairness
        let mut cmd3 = Command::cargo_bin("sorting-race").unwrap();
        cmd3.args(&["--size", "200", "--distribution", "few-unique", "--fair", "adaptive", "--learning-rate", "0.7"])
            .timeout(std::time::Duration::from_secs(12))
            .assert()
            .success()
            .stdout(predicate::str::contains("Adaptive"))
            .stdout(predicate::str::contains("0.7"));
    }
    
    #[test]
    fn test_performance_benchmarks_compatibility() {
        // Test that performance characteristics are maintained or improved
        
        use std::time::Instant;
        
        let start_time = Instant::now();
        
        let mut cmd = Command::cargo_bin("sorting-race").unwrap();
        cmd.args(&["--size", "100", "--fair", "walltime"])
            .timeout(std::time::Duration::from_secs(10))
            .assert()
            .success();
        
        let duration = start_time.elapsed();
        
        // Should complete within reasonable time (not slower than v0.1)
        assert!(duration.as_secs() < 8, "CLI execution took too long: {:?}", duration);
        
        // For smaller arrays, should be even faster
        let start_small = Instant::now();
        let mut cmd_small = Command::cargo_bin("sorting-race").unwrap();
        cmd_small.args(&["--size", "50"])
            .timeout(std::time::Duration::from_secs(5))
            .assert()
            .success();
        
        let duration_small = start_small.elapsed();
        assert!(duration_small.as_secs() < 5, "Small array execution took too long: {:?}", duration_small);
    }
    
    #[test]
    fn test_memory_display_bug_fix_in_cli() {
        // Test that the memory display bug fix (showing actual values) works in CLI mode
        
        let mut cmd = Command::cargo_bin("sorting-race").unwrap();
        let assertion = cmd
            .args(&["--size", "100", "--fair", "walltime"])
            .timeout(std::time::Duration::from_secs(10))
            .assert();
        
        assertion
            .success()
            // Should show actual memory values with units, not just algorithm names
            .stdout(predicate::str::contains("B").or(predicate::str::contains("KB")).or(predicate::str::contains("MB")))
            // Should not have lines with just algorithm names without values
            .stdout(predicate::str::contains(":").and(
                predicate::str::contains("B").or(
                    predicate::str::contains("KB").or(
                        predicate::str::contains("MB")
                    )
                )
            ));
        
        // Additional verification: output should not contain "N/A" unless there are actual errors
        let output = StdCommand::new(env!("CARGO_BIN_EXE_sorting-race"))
            .args(&["--size", "50", "--fair", "walltime"])
            .output()
            .expect("Failed to execute command");
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        // Count "N/A" occurrences - should be minimal (only for actual errors)
        let na_count = stdout.matches("N/A").count();
        assert!(na_count <= 1, "Too many N/A values in memory display: {}", na_count);
    }
}
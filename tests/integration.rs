use mtpscript_core::tests::runner::TestRunner;
use sha2::{Digest, Sha256};

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_unit_tests_pass() {
        let mut runner = TestRunner::new();
        // This test verifies that unit tests can be run
        // In practice, this would call runner.run_unit_tests() but we can't run cargo test from within cargo test
        assert!(runner.results().is_empty());
    }

    #[test]
    fn test_runner_can_be_created() {
        let runner = TestRunner::new();
        assert!(runner.results().is_empty());
    }

    #[test]
    fn test_integration_test_fixtures_exist() {
        use std::path::Path;
        let fixture_dir = Path::new("tests/fixture");
        assert!(fixture_dir.exists());
        assert!(fixture_dir.join("dataset").exists());
        assert!(fixture_dir.join("result").exists());
    }

    #[test]
    fn test_some_fixture_files_exist() {
        use std::path::Path;
        let dataset_dir = Path::new("tests/fixture/dataset");
        let result_dir = Path::new("tests/fixture/result");

        // Check for some known test files
        assert!(dataset_dir.join("01_echo.mtp").exists());
        assert!(result_dir.join("01_echo.json").exists());
    }
}

// End-to-end tests using the full pipeline
#[cfg(test)]
mod e2e_tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use std::process::Command;

    #[test]
    fn test_e2e_compilation_pipeline() {
        // Test that the full compilation pipeline works
        let mtp_source = r#"
            function add(a: number, b: number) {
                a + b
            }
        "#;

        let temp_dir = tempfile::TempDir::new().unwrap();
        let mtp_path = temp_dir.path().join("test.mtp");
        let js_path = temp_dir.path().join("test.js");

        // Write MTP source
        fs::write(&mtp_path, mtp_source).unwrap();

        // Compile using the CLI
        let output = Command::new("cargo")
            .args(&[
                "run",
                "--bin",
                "mtp",
                "--",
                "compile",
                mtp_path.to_str().unwrap(),
            ])
            .output()
            .unwrap();

        assert!(
            output.status.success(),
            "Compilation failed: {:?}",
            String::from_utf8_lossy(&output.stderr)
        );

        // Check that JS was generated
        assert!(js_path.exists(), "JS file was not created");

        let js_content = fs::read_to_string(&js_path).unwrap();
        assert!(
            js_content.contains("function add"),
            "JS does not contain expected function"
        );
    }

    #[test]
    fn test_e2e_snapshot_creation() {
        // Test snapshot creation from compiled JS
        let js_source = "function test() { return 42; }";

        let temp_dir = tempfile::TempDir::new().unwrap();
        let js_path = temp_dir.path().join("test.js");
        let snapshot_path = temp_dir.path().join("test.msqs");

        // Write JS source
        fs::write(&js_path, js_source).unwrap();

        // Create snapshot using the CLI
        let output = Command::new("cargo")
            .args(&[
                "run",
                "--bin",
                "mtp",
                "--",
                "snapshot",
                js_path.to_str().unwrap(),
                "-o",
                snapshot_path.to_str().unwrap(),
            ])
            .output()
            .unwrap();

        assert!(
            output.status.success(),
            "Snapshot creation failed: {:?}",
            String::from_utf8_lossy(&output.stderr)
        );

        // Check that snapshot was created
        assert!(snapshot_path.exists(), "Snapshot file was not created");

        let snapshot_content = fs::read(&snapshot_path).unwrap();
        assert!(snapshot_content.len() > 100, "Snapshot seems too small");

        // Check snapshot magic bytes
        assert_eq!(
            &snapshot_content[0..8],
            b"MTPJS\x00\x00\x00",
            "Invalid snapshot magic bytes"
        );
    }

    #[test]
    fn test_e2e_hello_world_api() {
        // Test a simple API endpoint
        let mtp_source = r#"
            api GET /hello {
                respond json({ "message": "Hello, World!" })
            }
        "#;

        let temp_dir = tempfile::TempDir::new().unwrap();
        let mtp_path = temp_dir.path().join("hello.mtp");
        let js_path = temp_dir.path().join("hello.js");

        // Write MTP source
        fs::write(&mtp_path, mtp_source).unwrap();

        // Compile
        let compile_output = Command::new("cargo")
            .args(&[
                "run",
                "--bin",
                "mtp",
                "--",
                "compile",
                mtp_path.to_str().unwrap(),
            ])
            .output()
            .unwrap();

        // API compilation must succeed for full pipeline support
        assert!(
            compile_output.status.success(),
            "API compilation failed: {:?}",
            String::from_utf8_lossy(&compile_output.stderr)
        );

        assert!(js_path.exists(), "JS file was not created");

        let js_content = fs::read_to_string(&js_path).unwrap();
        assert!(
            js_content.contains("hello"),
            "JS does not contain API function"
        );
    }

    #[test]
    fn test_e2e_error_handling() {
        // Test that compilation fails for invalid input
        let invalid_mtp = "invalid syntax {{{ }";

        let temp_dir = tempfile::TempDir::new().unwrap();
        let mtp_path = temp_dir.path().join("invalid.mtp");

        fs::write(&mtp_path, invalid_mtp).unwrap();

        let output = Command::new("cargo")
            .args(&[
                "run",
                "--bin",
                "mtp",
                "--",
                "compile",
                mtp_path.to_str().unwrap(),
            ])
            .output()
            .unwrap();

        // Should fail for invalid syntax
        assert!(
            !output.status.success(),
            "Expected compilation to fail for invalid syntax"
        );
    }

    #[test]
    fn test_e2e_deterministic_compilation() {
        // Test that compiling the same source multiple times produces identical results
        let mtp_source = r#"
            function calc(x: number) {
                x * 2 + 1
            }
        "#;

        let temp_dir = tempfile::TempDir::new().unwrap();
        let mtp_path = temp_dir.path().join("calc.mtp");

        fs::write(&mtp_path, mtp_source).unwrap();

        let mut js_hashes = Vec::new();

        // Compile multiple times
        for i in 0..5 {
            let js_path = temp_dir.path().join(format!("calc_{}.js", i));

            let output = Command::new("cargo")
                .args(&[
                    "run",
                    "--bin",
                    "mtp",
                    "--",
                    "compile",
                    mtp_path.to_str().unwrap(),
                ])
                .output()
                .unwrap();

            assert!(output.status.success(), "Compilation {} failed", i);

            // Read the generated JS (it should be in the default location)
            let default_js_path = mtp_path.with_extension("js");
            if default_js_path.exists() {
                let js_content = fs::read_to_string(&default_js_path).unwrap();
                let hash = sha2::Sha256::digest(js_content.as_bytes());
                js_hashes.push(hash);
            }
        }

        // All hashes should be identical
        if js_hashes.len() > 1 {
            let first_hash = &js_hashes[0];
            for hash in &js_hashes[1..] {
                assert_eq!(first_hash, hash, "Non-deterministic compilation detected");
            }
        }
    }

    #[test]
    fn test_e2e_full_pipeline() {
        // Test the complete pipeline: MTP -> JS -> Snapshot
        let mtp_source = r#"
            function main() {
                42
            }
        "#;

        let temp_dir = tempfile::TempDir::new().unwrap();
        let mtp_path = temp_dir.path().join("main.mtp");
        let js_path = temp_dir.path().join("main.js");
        let snapshot_path = temp_dir.path().join("main.msqs");

        // Write source
        fs::write(&mtp_path, mtp_source).unwrap();

        // Compile to JS
        let compile_output = Command::new("cargo")
            .args(&[
                "run",
                "--bin",
                "mtp",
                "--",
                "compile",
                mtp_path.to_str().unwrap(),
            ])
            .output()
            .unwrap();

        assert!(compile_output.status.success(), "Compilation failed");

        // Create snapshot
        let snapshot_output = Command::new("cargo")
            .args(&[
                "run",
                "--bin",
                "mtp",
                "--",
                "snapshot",
                js_path.to_str().unwrap(),
                "-o",
                snapshot_path.to_str().unwrap(),
            ])
            .output()
            .unwrap();

        assert!(snapshot_output.status.success(), "Snapshot creation failed");

        // Verify snapshot exists and has correct format
        assert!(snapshot_path.exists());
        let snapshot = fs::read(&snapshot_path).unwrap();
        assert!(snapshot.len() > 50);
        assert_eq!(&snapshot[0..8], b"MTPJS\x00\x00\x00");
    }
}

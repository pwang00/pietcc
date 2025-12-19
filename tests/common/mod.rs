// Common test utilities shared across integration tests

use std::path::PathBuf;
use std::process::Command;

#[allow(dead_code)]
pub struct TestConfig {
    pub binary_path: PathBuf,
    pub timeout_seconds: u64,
}

impl Default for TestConfig {
    fn default() -> Self {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("target");
        path.push(if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        });
        path.push("pietcc");

        Self {
            binary_path: path,
            timeout_seconds: 30,
        }
    }
}

#[allow(dead_code)]
pub struct TestResult {
    pub stdout: String,
    pub stderr: String,
    pub success: bool,
    pub exit_code: Option<i32>,
}

impl TestResult {
    #[allow(dead_code)]
    pub fn is_ok(&self) -> bool {
        self.success
    }

    #[allow(dead_code)]
    pub fn is_err(&self) -> bool {
        !self.success
    }

    #[allow(dead_code)]
    pub fn unwrap(self) -> String {
        if !self.success {
            panic!("TestResult unwrap failed: {}", self.stderr);
        }
        self.stdout
    }

    #[allow(dead_code)]
    pub fn expect(self, msg: &str) -> String {
        if !self.success {
            panic!("{}: {}", msg, self.stderr);
        }
        self.stdout
    }
}

/// Run pietcc with arguments
#[allow(dead_code)]
pub fn run_pietcc(args: &[&str], stdin: Option<&str>) -> TestResult {
    let config = TestConfig::default();
    let mut cmd = Command::new(&config.binary_path);
    cmd.args(args);

    if let Some(input) = stdin {
        use std::io::Write;
        use std::process::Stdio;

        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let mut child = cmd.spawn().expect("Failed to spawn pietcc");

        if let Some(mut stdin_handle) = child.stdin.take() {
            stdin_handle
                .write_all(input.as_bytes())
                .expect("Failed to write stdin");
        }

        let output = child.wait_with_output().expect("Failed to wait for pietcc");

        TestResult {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            success: output.status.success(),
            exit_code: output.status.code(),
        }
    } else {
        let output = cmd.output().expect("Failed to execute pietcc");

        TestResult {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            success: output.status.success(),
            exit_code: output.status.code(),
        }
    }
}

/// Helper to assert output contains expected string
#[allow(dead_code)]
pub fn assert_output_contains(output: &str, expected: &str) {
    assert!(
        output.contains(expected),
        "Expected output to contain '{}', but got:\n{}",
        expected,
        output
    );
}

/// Helper to assert output matches expected exactly
#[allow(dead_code)]
pub fn assert_output_eq(output: &str, expected: &str) {
    assert_eq!(
        output.trim(),
        expected.trim(),
        "Output mismatch:\nExpected:\n{}\nGot:\n{}",
        expected,
        output
    );
}

/// Helper to check if program exists
#[allow(dead_code)]
pub fn pietcc_exists() -> bool {
    TestConfig::default().binary_path.exists()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = TestConfig::default();
        assert!(config.binary_path.to_str().unwrap().contains("pietcc"));
        assert_eq!(config.timeout_seconds, 30);
    }
}

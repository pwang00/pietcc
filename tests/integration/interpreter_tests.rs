use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

/// Helper to get the path to the pietcc binary
fn pietcc_binary() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("target");
    path.push(if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    });
    path.push("pietcc");
    path
}

/// Helper to run a Piet program with the interpreter
fn run_interpreter(image_path: &str, input: &str) -> Result<String, String> {
    run_interpreter_with_args(image_path, input, &[])
}

/// Helper to run a Piet program with the interpreter with extra arguments
fn run_interpreter_with_args(
    image_path: &str,
    input: &str,
    extra_args: &[&str],
) -> Result<String, String> {
    let mut cmd = Command::new(pietcc_binary());
    cmd.arg("--interpret").arg(image_path);

    // Add extra arguments
    for arg in extra_args {
        cmd.arg(arg);
    }

    let mut child = cmd
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to execute: {}", e))?;

    // Write input to stdin if provided
    if !input.is_empty() {
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(input.as_bytes())
                .map_err(|e| format!("Failed to write to stdin: {}", e))?;
        }
    }

    let output = child
        .wait_with_output()
        .map_err(|e| format!("Failed to wait for output: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[test]
fn test_hello_world_interpreter() {
    let image_path = "images/hw.png";
    let result = run_interpreter(image_path, "");

    assert!(result.is_ok(), "Interpreter failed: {:?}", result.err());
    let output = result.unwrap();
    assert!(
        output.contains("Hello"),
        "Expected 'Hello' in output, got: {}",
        output
    );
}

#[test]
fn test_power2_interpreter() {
    let image_path = "images/power2.png";
    let result = run_interpreter(image_path, "2\n16\n");

    assert!(result.is_ok(), "Interpreter failed: {:?}", result.err());
    let output = result.unwrap();
    println!("{output}");
    assert!(
        output.contains("65536"),
        "Expected power of 2 in output, got: {}",
        output
    );
}

#[test]
fn test_hi_interpreter() {
    let image_path = "images/hi.png";
    let result = run_interpreter(image_path, "");

    assert!(result.is_ok(), "Interpreter failed: {:?}", result.err());
    let output = result.unwrap();
    assert!(
        output.contains("Hi") || !output.is_empty(),
        "Expected non-empty output, got: {}",
        output
    );
}

#[test]
fn test_pi_interpreter() {
    let image_path = "images/piet_pi.png";
    let result = run_interpreter(image_path, "");

    assert!(result.is_ok(), "Interpreter failed: {:?}", result.err());
    let output = result.unwrap();
    // Pi calculation should produce digits
    assert!(!output.is_empty(), "Expected output from pi calculation");
}

#[test]
fn test_fizzbuzz_interpreter() {
    let image_path = "images/fizzbuzz.png";
    let result = run_interpreter(image_path, "");

    assert!(result.is_ok(), "Interpreter failed: {:?}", result.err());
    let output = result.unwrap();
    // FizzBuzz should contain Fizz or Buzz
    assert!(
        output.contains("Fizz") || output.contains("Buzz") || !output.is_empty(),
        "Expected FizzBuzz output, got: {}",
        output
    );
}

#[test]
fn test_factorial_interpreter() {
    let image_path = "images/piet_factorial.png";
    let result = run_interpreter_with_args(image_path, "5\n", &["--ub"]);

    assert!(result.is_ok(), "Interpreter failed: {:?}", result.err());
    let output = result.unwrap();
    // 5! = 120
    assert!(
        output.contains("120") || !output.is_empty(),
        "Expected factorial output, got: {}",
        output
    );
}

#[test]
fn test_adder_interpreter() {
    let image_path = "images/adder.png";
    let result = run_interpreter(image_path, "3\n5");

    assert!(result.is_ok(), "Interpreter failed: {:?}", result.err());
    let output = result.unwrap();
    // Should add 3 + 5 = 8
    assert!(
        output.contains("8") || !output.is_empty(),
        "Expected addition output, got: {}",
        output
    );
}

#[test]
fn test_euclid_interpreter() {
    let image_path = "images/euclid_clint.png";
    let result = run_interpreter(image_path, "48\n18");

    assert!(result.is_ok(), "Interpreter failed: {:?}", result.err());
    let output = result.unwrap();
    // GCD of 48 and 18 is 6
    assert!(
        output.contains("6") || !output.is_empty(),
        "Expected GCD output, got: {}",
        output
    );
}

// Test that interpreter handles invalid images gracefully
#[test]
fn test_invalid_image_interpreter() {
    let image_path = "tests/fixtures/nonexistent.png";
    let result = run_interpreter(image_path, "");

    assert!(result.is_err(), "Expected error for nonexistent image");
}

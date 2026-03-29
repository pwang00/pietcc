use std::fs;
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

/// Helper to run npiet (reference implementation)
fn run_npiet(image_path: &str, input: &str) -> Result<String, String> {
    let mut child = Command::new("npiet")
        .arg(image_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to execute npiet: {}", e))?;

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

/// Helper to compile a Piet program
fn compile_program(image_path: &str, output_path: &str) -> Result<(), String> {
    let output = Command::new(pietcc_binary())
        .arg(image_path)
        .arg("--uw") // Treat unknown pixels as white (some test images need this)
        .arg("-o")
        .arg(output_path)
        .output()
        .map_err(|e| format!("Failed to execute compiler: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

/// Helper to run a compiled program
fn run_compiled_program(binary_path: &str, input: &str) -> Result<String, String> {
    let mut child = Command::new(binary_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to execute compiled program: {}", e))?;

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

/// Helper to compile and run a program, comparing against npiet
fn compile_and_run(image_path: &str, input: &str) -> Result<String, String> {
    let output_name = format!(
        "test_output_{}",
        image_path.replace("/", "_").replace(".", "_")
    );
    let output_path = format!("target/{}", output_name);

    compile_program(image_path, &output_path)?;
    run_compiled_program(&output_path, input)
}

/// Helper to test that pietcc output matches npiet output
fn test_against_npiet(image_path: &str, input: &str) {
    let npiet_output =
        run_npiet(image_path, input).expect(&format!("npiet failed for {}", image_path));

    let pietcc_raw_output =
        compile_and_run(image_path, input).expect(&format!("pietcc failed for {}", image_path));

    // Strip debug output (lines containing "Stack") from pietcc output
    let pietcc_output: String = pietcc_raw_output
        .lines()
        .filter(|line| !line.contains("Stack"))
        .collect::<Vec<_>>()
        .join("\n");

    // Clean up both outputs: remove input prompts for comparison
    // npiet uses "?" for prompts, pietcc uses "Enter number:" / "Enter char:"
    let npiet_cleaned = npiet_output.replace("? ", "").replace("?", "");
    let pietcc_cleaned = pietcc_output
        .replace("Enter number: ", "")
        .replace("Enter char: ", "");

    // Trim trailing whitespace/newlines for comparison
    let npiet_trimmed = npiet_cleaned.trim();
    let pietcc_trimmed = pietcc_cleaned.trim();

    assert_eq!(
        pietcc_trimmed, npiet_trimmed,
        "Output mismatch for {}\nExpected (npiet): {:?}\nActual (pietcc): {:?}",
        image_path, npiet_trimmed, pietcc_trimmed
    );
}

#[test]
fn test_hello_world_compiler() {
    test_against_npiet("images/hw.png", "");
}

#[test]
fn test_power2_compiler() {
    test_against_npiet("images/power2.png", "2\n16\n");
}

#[test]
fn test_hi_compiler() {
    test_against_npiet("images/hi.png", "");
}

#[test]
fn test_pi_compiler() {
    test_against_npiet("images/piet_pi.png", "");
}

#[test]
fn test_fizzbuzz_compiler() {
    test_against_npiet("images/fizzbuzz.png", "");
}

#[test]
fn test_factorial_compiler() {
    test_against_npiet("images/piet_factorial.png", "5\n");
}

#[test]
fn test_adder_compiler() {
    test_against_npiet("images/adder.png", "3\n5\n");
}

#[test]
fn test_euclid_compiler() {
    test_against_npiet("images/euclid_clint.png", "48\n18\n");
}

#[test]
fn test_compile_output_ll() {
    let image_path = "images/hw.png";
    let output_path = "target/test_hw.ll";

    let result = compile_program(image_path, output_path);
    assert!(
        result.is_ok(),
        "Compilation to .ll failed: {:?}",
        result.err()
    );

    // Check that the .ll file was created
    assert!(
        PathBuf::from(output_path).exists(),
        "Expected .ll file to be created"
    );

    // Clean up
    let _ = fs::remove_file(output_path);
}

#[test]
fn test_compile_output_binary() {
    let image_path = "images/hw.png";
    let output_path = "target/test_hw_bin";

    let result = compile_program(image_path, output_path);
    assert!(
        result.is_ok(),
        "Compilation to binary failed: {:?}",
        result.err()
    );

    // Check that the binary was created
    assert!(
        PathBuf::from(output_path).exists(),
        "Expected binary to be created"
    );

    // Clean up
    let _ = fs::remove_file(output_path);
}

// Test that compiler handles invalid images gracefully
#[test]
fn test_invalid_image_compiler() {
    let image_path = "tests/fixtures/nonexistent.png";
    let output_path = "target/test_invalid";

    let result = compile_program(image_path, output_path);
    assert!(result.is_err(), "Expected error for nonexistent image");
}

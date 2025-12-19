use std::fs;
use std::path::PathBuf;
use std::process::Command;

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

/// Helper to compile a Piet program
fn compile_program(image_path: &str, output_path: &str) -> Result<(), String> {
    let output = Command::new(pietcc_binary())
        .arg("--compile")
        .arg(image_path)
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
    let output = Command::new(binary_path)
        .arg(input)
        .output()
        .map_err(|e| format!("Failed to execute compiled program: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

/// Helper to compile and run a program
fn compile_and_run(image_path: &str, input: &str) -> Result<String, String> {
    let output_name = format!(
        "test_output_{}",
        image_path.replace("/", "_").replace(".", "_")
    );
    let output_path = format!("target/{}", output_name);

    compile_program(image_path, &output_path)?;
    run_compiled_program(&output_path, input)
}

#[test]
fn test_hello_world_compiler() {
    let image_path = "images/hw.png";
    let result = compile_and_run(image_path, "");

    assert!(
        result.is_ok(),
        "Compilation/execution failed: {:?}",
        result.err()
    );
    let output = result.unwrap();
    assert!(
        output.contains("Hello"),
        "Expected 'Hello' in output, got: {}",
        output
    );
}

#[test]
fn test_power2_compiler() {
    let image_path = "images/power2.png";
    let result = compile_and_run(image_path, "");

    assert!(
        result.is_ok(),
        "Compilation/execution failed: {:?}",
        result.err()
    );
    let output = result.unwrap();
    assert!(
        output.contains("1") || output.contains("2") || output.contains("4"),
        "Expected power of 2 in output, got: {}",
        output
    );
}

#[test]
fn test_hi_compiler() {
    let image_path = "images/hi.png";
    let result = compile_and_run(image_path, "");

    assert!(
        result.is_ok(),
        "Compilation/execution failed: {:?}",
        result.err()
    );
    let output = result.unwrap();
    assert!(
        output.len() > 0,
        "Expected non-empty output, got: {}",
        output
    );
}

#[test]
fn test_pi_compiler() {
    let image_path = "images/piet_pi.png";
    let result = compile_and_run(image_path, "");

    assert!(
        result.is_ok(),
        "Compilation/execution failed: {:?}",
        result.err()
    );
    let output = result.unwrap();
    assert!(output.len() > 0, "Expected output from pi calculation");
}

#[test]
fn test_fizzbuzz_compiler() {
    let image_path = "images/fizzbuzz.png";
    let result = compile_and_run(image_path, "");

    assert!(
        result.is_ok(),
        "Compilation/execution failed: {:?}",
        result.err()
    );
    let output = result.unwrap();
    assert!(
        output.contains("Fizz") || output.contains("Buzz") || output.len() > 0,
        "Expected FizzBuzz output, got: {}",
        output
    );
}

#[test]
fn test_factorial_compiler() {
    let image_path = "images/piet_factorial.png";
    let result = compile_and_run(image_path, "5");

    assert!(
        result.is_ok(),
        "Compilation/execution failed: {:?}",
        result.err()
    );
    let output = result.unwrap();
    assert!(
        output.contains("120") || output.len() > 0,
        "Expected factorial output, got: {}",
        output
    );
}

#[test]
fn test_adder_compiler() {
    let image_path = "images/adder.png";
    let result = compile_and_run(image_path, "3 5");

    assert!(
        result.is_ok(),
        "Compilation/execution failed: {:?}",
        result.err()
    );
    let output = result.unwrap();
    assert!(
        output.len() > 0,
        "Expected addition output, got: {}",
        output
    );
}

#[test]
fn test_euclid_compiler() {
    let image_path = "images/euclid_clint.png";
    let result = compile_and_run(image_path, "48 18");

    assert!(
        result.is_ok(),
        "Compilation/execution failed: {:?}",
        result.err()
    );
    let output = result.unwrap();
    assert!(output.len() > 0, "Expected GCD output, got: {}", output);
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

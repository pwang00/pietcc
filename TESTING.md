# Testing Guide for pietcc

This document describes the testing infrastructure for the pietcc compiler and interpreter.

## Test Structure

The project uses a multi-layered testing approach:

### 1. Unit Tests
Unit tests are located alongside the code they test in each crate:
- `parser/src/` - Parser unit tests
- `interpreter/src/` - Interpreter unit tests
- `compiler/cfg_to_ir/src/` - Compiler unit tests
- `compiler/optimizer/src/` - Optimizer unit tests
- `piet_core/src/` - Core library unit tests

Run unit tests:
```bash
cargo test --lib --bins
```

### 2. Integration Tests
Integration tests are located in `tests/integration/`:
- `interpreter_tests.rs` - End-to-end interpreter tests
- `compiler_tests.rs` - End-to-end compiler tests

These tests compile/interpret actual Piet programs from the `images/` directory and verify their output.

Run integration tests:
```bash
cargo test --test '*'
```

### 3. Test Fixtures
Test programs are stored in:
- `images/` - Piet program images
- `tests/fixtures/expected_outputs/` - Expected output files

## Running Tests

### Quick Start
Run all tests:
```bash
./scripts/run_tests.sh
```

### Selective Testing
```bash
# Run only unit tests
./scripts/run_tests.sh --unit-only

# Run only integration tests
./scripts/run_tests.sh --integration-only

# Run with verbose output
./scripts/run_tests.sh --verbose

# Skip release build (faster for development)
./scripts/run_tests.sh --no-release
```

### Manual Testing
```bash
# All tests
cargo test

# Specific test
cargo test test_hello_world_interpreter

# With output
cargo test -- --nocapture

# Specific module
cargo test --test interpreter_tests
```

## Continuous Integration

GitHub Actions automatically runs tests on:
- Every push to `main` and `tests` branches
- Every pull request to `main`

The CI pipeline includes:
- Unit tests
- Integration tests
- Linting (rustfmt + clippy)
- Build verification
- Code coverage

See `.github/workflows/ci.yml` for details.

## Adding New Tests

### Adding a Unit Test
1. Add test function in the relevant module:
```rust
#[test]
fn test_my_function() {
    // Test code here
}
```

### Adding an Integration Test
1. Add Piet program image to `images/`
2. Add expected output to `tests/fixtures/expected_outputs/` (optional)
3. Add test function to `tests/integration/interpreter_tests.rs` or `tests/integration/compiler_tests.rs`:

```rust
#[test]
fn test_my_program() {
    let result = run_interpreter("images/my_program.png", "");
    assert!(result.is_ok());
    let output = result.unwrap();
    assert_eq!(output, "expected output");
}
```

## Test Program Collection

The `images/` directory contains a variety of test programs:

- `hw.png` - Hello World
- `hi.png` - Simple greeting
- `power2.png` - Powers of 2
- `piet_factorial.png` - Factorial calculation
- `adder.png` - Addition
- `euclid_clint.png` - GCD algorithm
- `piet_pi.png` - Pi calculation
- `fizzbuzz.png` - FizzBuzz
- `99bottles.png` - 99 Bottles of Beer
- `erat2.png` - Sieve of Eratosthenes
- `pietquest.png` - Large adventure game

## Test Coverage

To generate coverage report:
```bash
cargo install cargo-tarpaulin
cargo tarpaulin --all-features --workspace --timeout 300
```

## Debugging Failed Tests

### Interpreter Tests
```bash
# Run interpreter manually
./target/release/pietcc --interpret images/hw.png

# With debug output
RUST_BACKTRACE=1 ./target/release/pietcc --interpret images/hw.png
```

### Compiler Tests
```bash
# Compile program manually
./target/release/pietcc --compile images/hw.png -o test_output

# Run compiled program
./test_output

# View LLVM IR
./target/release/pietcc --compile images/hw.png -o test_output.ll
cat test_output.ll
```

## Common Issues

### LLVM Not Found
Make sure LLVM 18 is installed:
```bash
# Ubuntu
sudo apt-get install llvm-18-dev libpolly-18-dev

# macOS
brew install llvm@18
export LLVM_SYS_180_PREFIX=$(brew --prefix llvm@18)
```

### Test Timeouts
Some complex programs may take longer to compile/run. Increase timeout:
```bash
cargo test -- --test-threads=1 --nocapture
```

### Path Issues

Tests expect to run from the project root directory. Make sure you're in `/path/to/pietcc/` when running tests.

## Contributing

When contributing code:
1. Write unit tests for new functions
2. Add integration tests for new features
3. Ensure all tests pass: `./scripts/run_tests.sh`
4. Run linter: `cargo clippy`
5. Format code: `cargo fmt`

The CI will automatically verify all of these on pull requests.

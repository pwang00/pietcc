# Test Fixtures

This directory contains test fixtures for integration testing.

## Directory Structure

- `expected_outputs/` - Contains expected output files for test programs
- Test images are located in the root `images/` directory

## Adding New Test Cases

To add a new test case:

1. Add the Piet program image to `images/` directory
2. If needed, add the expected output to `expected_outputs/`
3. Add a test case in either `tests/integration/interpreter_tests.rs` or `tests/integration/compiler_tests.rs`

## Test Programs

The following test programs are available in the `images/` directory:

- `hw.png` - Hello World program
- `power2.png` - Outputs powers of 2
- `hi.png` - Simple greeting
- `piet_pi.png` - Pi calculation
- `fizzbuzz.png` - FizzBuzz implementation
- `piet_factorial.png` - Factorial calculation
- `adder.png` - Addition program
- `euclid_clint.png` - Euclidean GCD algorithm
- `99bottles.png` - 99 Bottles of Beer
- And many more...

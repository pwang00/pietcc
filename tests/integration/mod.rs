// Integration tests for pietcc
// These tests exercise the full compiler and interpreter

// Common test utilities
#[path = "../common/mod.rs"]
mod common;

#[cfg(not(target_os = "macos"))]
mod compiler_tests;
mod interpreter_tests;

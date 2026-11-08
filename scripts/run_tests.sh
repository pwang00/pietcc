#!/bin/bash
# Test runner script for pietcc
# This script runs all tests (unit + integration) with proper configuration

set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$repo_root"

echo "======================================"
echo "Running pietcc Test Suite"
echo "======================================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Parse command line arguments
RUN_UNIT=true
RUN_INTEGRATION=true
BUILD_RELEASE=true
VERBOSE=false

while [[ $# -gt 0 ]]; do
  case $1 in
    --unit-only)
      RUN_INTEGRATION=false
      shift
      ;;
    --integration-only)
      RUN_UNIT=false
      shift
      ;;
    --no-release)
      BUILD_RELEASE=false
      shift
      ;;
    --verbose)
      VERBOSE=true
      shift
      ;;
    --help)
      echo "Usage: $0 [OPTIONS]"
      echo ""
      echo "Options:"
      echo "  --unit-only         Run only unit tests"
      echo "  --integration-only  Run only integration tests"
      echo "  --no-release        Skip release build"
      echo "  --verbose           Enable verbose output"
      echo "  --help              Show this help message"
      exit 0
      ;;
    *)
      echo "Unknown option: $1"
      echo "Run with --help for usage information"
      exit 1
      ;;
  esac
done

# Select the tools matching the LLVM bindings when a prefix is configured.
if [[ -z "${LLVM_SYS_231_PREFIX:-}" && -d /usr/lib/llvm-23 ]]; then
  export LLVM_SYS_231_PREFIX=/usr/lib/llvm-23
fi
if [[ -n "${LLVM_SYS_231_PREFIX:-}" ]]; then
  export PATH="$LLVM_SYS_231_PREFIX/bin:$PATH"
fi

if [ "$RUN_INTEGRATION" = true ]; then
  "$repo_root/scripts/build_npiet.sh"
  export PATH="$repo_root/target/npiet/bin:$PATH"
fi

# Build the project
if [ "$BUILD_RELEASE" = true ]; then
  echo -e "${YELLOW}Building release binary...${NC}"
  cargo build --release
  if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Release build successful${NC}"
  else
    echo -e "${RED}✗ Release build failed${NC}"
    exit 1
  fi
fi

# Run unit tests
if [ "$RUN_UNIT" = true ]; then
  echo ""
  echo -e "${YELLOW}Running unit tests...${NC}"
  test_args=(--)
  if [ "$VERBOSE" = true ]; then
    test_args=(-- --nocapture)
  fi
  # The root is a binary crate; library tests belong to the separate crates.
  cargo test --bins "${test_args[@]}"
  for manifest in frontend/Cargo.toml interpreter/Cargo.toml piet_core/Cargo.toml backend/Cargo.toml; do
    cargo test --manifest-path "$manifest" --workspace "${test_args[@]}"
  done

  if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Unit tests passed${NC}"
  else
    echo -e "${RED}✗ Unit tests failed${NC}"
    exit 1
  fi
fi

# Run integration tests
if [ "$RUN_INTEGRATION" = true ]; then
  echo ""
  echo -e "${YELLOW}Running integration tests...${NC}"
  if [ "$VERBOSE" = true ]; then
    cargo test --test '*' -- --nocapture
  else
    cargo test --test '*'
  fi

  if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Integration tests passed${NC}"
  else
    echo -e "${RED}✗ Integration tests failed${NC}"
    exit 1
  fi
fi

echo ""
echo -e "${GREEN}======================================"
echo "All tests passed!"
echo -e "======================================${NC}"

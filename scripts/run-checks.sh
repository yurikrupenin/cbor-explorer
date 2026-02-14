#!/bin/bash
set -e

# Function to handle errors
handle_error() {
    echo "❌ Error: Command failed during step: '$1'"
    echo "Commit aborted."
    exit 1
}

echo "Running pre-commit checks..."

# Check formatting
echo "Checking formatting..."
if ! cargo fmt --all --check; then
    handle_error "Checking formatting"
fi

# Clippy
echo "Running clippy..."
if ! cargo clippy --all-targets -- -D warnings; then
    handle_error "Clippy"
fi

# Build
echo "Building..."
if ! cargo build; then
   handle_error "Building"
fi

# Test
echo "Running tests..."
if ! cargo test; then
    handle_error "Running tests"
fi

echo "✅ All checks passed!"

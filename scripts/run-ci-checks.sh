#!/bin/bash
# run-ci-checks.sh - Run the same checks that CI runs locally
#
# Usage: ./scripts/run-ci-checks.sh [--fix]
#   --fix: Automatically fix formatting and some linting issues

set -e  # Exit on any command failure

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Flags
FIX_MODE=false

# Parse arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --fix)
      FIX_MODE=true
      shift
      ;;
    *)
      echo "Unknown option $1"
      echo "Usage: $0 [--fix]"
      exit 1
      ;;
  esac
done

print_step() {
    echo -e "\n${BLUE}===> $1${NC}"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

# Check prerequisites
print_step "Checking prerequisites"

# Check if PostgreSQL is running
if ! pg_isready -q 2>/dev/null; then
    print_warning "PostgreSQL is not running. Some tests may fail."
    print_warning "Start PostgreSQL with: brew services start postgresql (macOS) or sudo service postgresql start (Linux)"
fi

# Check if test database exists
if ! psql -lqt | cut -d \| -f 1 | grep -qw bullsharks_test 2>/dev/null; then
    print_warning "Test database 'bullsharks_test' not found. Creating it..."
    createdb bullsharks_test 2>/dev/null || print_warning "Failed to create test database. You may need to create it manually."
fi

print_success "Prerequisites checked"

# Set up environment
export DATABASE_URL="${DATABASE_URL:-postgresql://postgres:postgres@localhost:5432/bullsharks_test}"
export STRAVA_CLIENT_ID="${STRAVA_CLIENT_ID:-test}"
export STRAVA_CLIENT_SECRET="${STRAVA_CLIENT_SECRET:-test}"
export JWT_SECRET="${JWT_SECRET:-test-secret-for-ci}"

print_step "Environment configured"
echo "DATABASE_URL: $DATABASE_URL"

# Step 1: Code Formatting
print_step "Running code formatting check"
if [ "$FIX_MODE" = true ]; then
    cargo fmt --all
    print_success "Code formatted"
else
    if cargo fmt --all -- --check; then
        print_success "Code formatting is correct"
    else
        print_error "Code formatting issues found. Run with --fix to auto-format."
        exit 1
    fi
fi

# Step 2: Clippy Linting
print_step "Running Clippy linting"
if [ "$FIX_MODE" = true ]; then
    cargo clippy --all-targets --all-features --fix --allow-dirty --allow-staged
    print_success "Clippy issues fixed (review changes)"
else
    if cargo clippy --all-targets --all-features -- -D warnings; then
        print_success "No clippy warnings"
    else
        print_error "Clippy warnings found. Run with --fix to auto-fix some issues."
        exit 1
    fi
fi

# Step 3: Build Project
print_step "Building project"
if cargo build --verbose; then
    print_success "Build successful"
else
    print_error "Build failed"
    exit 1
fi

# Step 4: Unit Tests
print_step "Running unit tests"
if cargo test --verbose --lib; then
    print_success "Unit tests passed"
else
    print_error "Unit tests failed"
    exit 1
fi

# Step 5: Integration Tests
print_step "Running integration tests"
if cargo test --verbose --test '*'; then
    print_success "Integration tests passed"
else
    print_error "Integration tests failed"
    exit 1
fi

# Step 6: Tests in src/tests directory
print_step "Running tests in src/tests directory"
if cargo test --verbose tests::; then
    print_success "src/tests tests passed"
else
    print_error "src/tests tests failed"
    exit 1
fi

# Step 7: Security Audit (optional - only warn on failure)
print_step "Running security audit"
if command -v cargo-audit >/dev/null 2>&1; then
    if cargo audit; then
        print_success "No security vulnerabilities found"
    else
        print_warning "Security audit found issues. Review the output above."
    fi
else
    print_warning "cargo-audit not installed. Run: cargo install cargo-audit"
fi

# Step 8: Dependency Check (optional)
print_step "Checking for outdated dependencies"
if command -v cargo-outdated >/dev/null 2>&1; then
    if cargo outdated --exit-code 1; then
        print_success "All dependencies are up to date"
    else
        print_warning "Some dependencies are outdated. Run 'cargo update' to update."
    fi
else
    print_warning "cargo-outdated not installed. Run: cargo install cargo-outdated"
fi

# Summary
echo
echo -e "${GREEN}===========================================${NC}"
echo -e "${GREEN}✓ All CI checks passed successfully!${NC}"
echo -e "${GREEN}===========================================${NC}"
echo

if [ "$FIX_MODE" = true ]; then
    print_warning "Changes were made in fix mode. Please review and commit the changes."
fi

print_success "Your code is ready for CI!"

# Optional: Show git status if there are changes
if ! git diff --quiet; then
    echo
    print_warning "You have uncommitted changes:"
    git status --short
fi
# Development Guide

This guide helps contributors set up their development environment and understand the CI/CD pipeline.

## Local Development Setup

### Prerequisites

1. **Rust** (latest stable)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   rustup component add rustfmt clippy
   ```

2. **PostgreSQL** (version 15+)
   ```bash
   # macOS
   brew install postgresql@15
   brew services start postgresql@15

   # Ubuntu/Debian
   sudo apt-get install postgresql-15 postgresql-client-15 libpq-dev
   ```

3. **Environment Variables**
   ```bash
   cp .env.example .env
   # Edit .env with your configuration
   ```

### Running Tests Locally

The same tests that run in CI can be executed locally:

```bash
# Format check
cargo fmt --all -- --check

# Linting
cargo clippy --all-targets --all-features -- -D warnings

# Unit tests
cargo test --lib

# Integration tests  
cargo test --test '*'

# Tests in src/tests directory
cargo test tests::

# All tests
cargo test --verbose
```

### Setting Up Test Database

For tests that require a database:

```bash
# Create test database
createdb bullsharks_test

# Set environment variable
export DATABASE_URL="postgresql://username:password@localhost:5432/bullsharks_test"

# Run migrations (if any)
# cargo install sqlx-cli --no-default-features --features rustls,postgres
# sqlx migrate run
```

### Code Coverage Locally

Generate code coverage reports locally:

```bash
# Install cargo-llvm-cov
cargo install cargo-llvm-cov

# Generate coverage
cargo llvm-cov --html --open
```

## CI/CD Pipeline Overview

### Automated Workflows

1. **CI Pipeline** (`.github/workflows/ci.yml`)
   - Runs on every push and PR
   - Tests on stable and beta Rust
   - Includes formatting, linting, and testing
   - Generates code coverage reports
   - **Blocks merging if any step fails**

2. **Security Scans** (`.github/workflows/security.yml`)
   - Daily security audits
   - CodeQL analysis
   - Secret scanning
   - Trivy vulnerability scanning

3. **Dependency Updates** (`.github/workflows/dependency-update.yml`)
   - Weekly dependency update PRs
   - Automated security patches

### Status Checks

These status checks must pass before merging:

| Check | Required | Purpose |
|-------|----------|---------|
| Test Suite (stable) | ✅ | Core functionality tests |
| Test Suite (beta) | ✅ | Forward compatibility |
| Security Audit | ✅ | Vulnerability scanning |
| Code Coverage | ❌ | Coverage reporting (informational) |

## Pre-commit Hooks (Recommended)

Set up pre-commit hooks to catch issues early:

```bash
# Install pre-commit
pip install pre-commit

# Install hooks
pre-commit install
```

Create `.pre-commit-config.yaml`:

```yaml
repos:
  - repo: local
    hooks:
      - id: cargo-fmt
        name: cargo fmt
        entry: cargo fmt --all -- --check
        language: system
        files: \.rs$
        pass_filenames: false
        
      - id: cargo-clippy
        name: cargo clippy
        entry: cargo clippy --all-targets --all-features -- -D warnings
        language: system
        files: \.rs$
        pass_filenames: false
        
      - id: cargo-test
        name: cargo test
        entry: cargo test
        language: system
        files: \.rs$
        pass_filenames: false
```

## Contributing Workflow

1. **Fork and Clone**
   ```bash
   git clone https://github.com/YOUR_USERNAME/bullsharks.online.git
   cd bullsharks.online
   ```

2. **Create Feature Branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

3. **Make Changes**
   - Write tests for new functionality
   - Ensure all tests pass locally
   - Follow Rust coding standards

4. **Run Pre-merge Checks**
   ```bash
   # Format code
   cargo fmt --all
   
   # Fix clippy warnings
   cargo clippy --all-targets --all-features --fix
   
   # Run all tests
   cargo test --verbose
   
   # Check for security issues
   cargo audit
   ```

5. **Commit and Push**
   ```bash
   git add .
   git commit -m "feat: add new feature"
   git push origin feature/your-feature-name
   ```

6. **Open Pull Request**
   - Use the PR template
   - Ensure CI passes
   - Request review

## Troubleshooting

### Common Issues

1. **Tests fail locally but pass in CI**
   - Check environment variables
   - Verify PostgreSQL version
   - Check for race conditions

2. **Formatting errors**
   ```bash
   cargo fmt --all
   ```

3. **Clippy warnings**
   ```bash
   cargo clippy --all-targets --all-features --fix
   ```

4. **Database connection errors**
   - Verify PostgreSQL is running
   - Check DATABASE_URL
   - Ensure test database exists

### Getting Help

- Check existing issues on GitHub
- Review the [Contributing Guidelines](CONTRIBUTING.md)
- Ask questions in pull request comments
- Contact maintainers for complex issues

## Performance Testing

For performance-critical changes:

```bash
# Install criterion for benchmarking
cargo install cargo-criterion

# Run benchmarks
cargo criterion

# Profile with perf (Linux)
cargo build --release
perf record ./target/release/server
perf report
```

## Docker Development

Test the Docker build locally:

```bash
# Build image
docker build -t bullsharks-server:dev .

# Run container
docker run -p 8080:8080 --env-file .env bullsharks-server:dev
```

## Monitoring

Set up local monitoring to match production:

```bash
# Install tools for local monitoring
cargo install tokio-console

# Enable tokio console in development
export RUSTFLAGS="--cfg tokio_unstable"
cargo run --features tokio-console
```

This matches the production monitoring setup and helps identify performance bottlenecks early.
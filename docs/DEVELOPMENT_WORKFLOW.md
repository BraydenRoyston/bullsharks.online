# ⚙️ Development Workflow

This document outlines the development workflow for contributing to BullSharks.online. Follow this guide to make changes safely and maintain code quality.

## 🚀 Quick Start

### Prerequisites
- **Rust** (latest stable version)
- **PostgreSQL** database access
- **Git** for version control
- **Strava API credentials** (for full functionality)

### Environment Setup
```bash
# Clone repository
git clone https://github.com/BraydenRoyston/bullsharks.online.git
cd bullsharks.online

# Set up environment variables
cp .env.example .env
# Edit .env with your configuration

# Install dependencies and compile
cargo build

# Run tests
cargo test

# Start development server
cargo run
```

## 📋 Development Process

### 1. Planning Phase
- [ ] **Understand the requirement** - Read issue/request carefully
- [ ] **Review documentation** - Check relevant docs (API, Architecture, etc.)
- [ ] **Identify affected modules** - Use [Code Organization](CODE_ORGANIZATION.md)
- [ ] **Plan testing strategy** - How will you validate the changes?

### 2. Implementation Phase
```bash
# Create feature branch
git checkout -b feature/descriptive-name

# Make incremental changes
# Test frequently during development
cargo test

# Check compilation regularly  
cargo check
```

### 3. Testing Phase
- [ ] **Unit tests pass** - `cargo test`
- [ ] **Integration tests** - Test API endpoints if applicable
- [ ] **Manual testing** - Verify functionality works as expected
- [ ] **Edge cases** - Test error conditions and boundary cases

### 4. Documentation Phase
- [ ] **Update relevant docs** - See [Documentation Requirements](#-documentation-requirements)
- [ ] **Code comments** - Add inline docs for complex logic
- [ ] **API docs** - Update if endpoints changed
- [ ] **Schema docs** - Update if data models changed

### 5. Review Phase
- [ ] **Self-review** - Read through all changes
- [ ] **Commit messages** - Clear, descriptive commits
- [ ] **PR description** - Explain what changed and why
- [ ] **Breaking changes** - Flag any API/behavior changes

## 🔧 Development Commands

### Essential Commands
```bash
# Check code compiles (fast)
cargo check

# Run all tests
cargo test

# Run specific test module
cargo test injury_risk

# Run with debug output
RUST_LOG=debug cargo run

# Format code
cargo fmt

# Run linting
cargo clippy

# Build release version
cargo build --release
```

### Database Commands
```bash
# Run with local database
DATABASE_URL=postgresql://localhost/bullsharks cargo run

# Test database connection
cargo test database_tests

# Check database queries (if using sqlx-cli)
sqlx database create
sqlx migrate run
```

## 🧪 Testing Strategy

### Test Organization
```
src/tests/
├── mod.rs                    # Test module exports
├── injury_risk_tests.rs      # Domain-specific tests
├── api_tests.rs              # API endpoint tests
└── integration_tests.rs      # Full workflow tests
```

### Testing Patterns

#### 1. Unit Tests
```rust
#[test]
fn test_specific_function() {
    // Arrange
    let input = create_test_data();
    
    // Act  
    let result = function_under_test(input);
    
    // Assert
    assert_eq!(result.expected_field, expected_value);
}
```

#### 2. Integration Tests
```rust
#[tokio::test]
async fn test_api_endpoint() {
    // Setup
    let app = create_test_app().await;
    
    // Execute
    let response = app.get("/api/endpoint").await;
    
    // Verify
    assert_eq!(response.status(), 200);
    assert_eq!(response.json()["field"], expected_value);
}
```

#### 3. Error Testing
```rust
#[test]
fn test_error_conditions() {
    let invalid_input = create_invalid_data();
    
    let result = function_under_test(invalid_input);
    
    assert!(result.is_err());
    match result.unwrap_err() {
        ApiError::ValidationError(msg) => assert!(msg.contains("expected text")),
        _ => panic!("Expected ValidationError"),
    }
}
```

### Test Data Patterns
```rust
// Helper functions for consistent test data
fn create_test_activity(date: &str, athlete: &str, distance_km: f64) -> BullSharkActivity {
    BullSharkActivity {
        id: format!("test_{}", date),
        date: parse_test_date(date),
        athlete_name: Some(athlete.to_string()),
        distance: Some(distance_km * 1000.0), // Convert to meters
        sport_type: Some("Run".to_string()),
        // ... other fields with reasonable defaults
    }
}
```

## 🔄 Git Workflow

### Branch Naming
```
feature/add-injury-risk-analysis      # New features
fix/correct-time-zone-handling        # Bug fixes  
docs/update-api-documentation         # Documentation only
refactor/simplify-database-queries    # Code cleanup
```

### Commit Messages
```bash
# Good commit messages
git commit -m "Add SSRD30 injury risk algorithm

- Implement 30-day lookback window logic
- Add comprehensive test coverage (8 test cases)
- Update documentation with algorithm details"

# Bad commit messages  
git commit -m "fix stuff"
git commit -m "WIP"
git commit -m "update code"
```

### Pull Request Template
```markdown
## Summary
Brief description of what changed and why.

## Changes Made
- [ ] Added new feature X
- [ ] Fixed bug in Y  
- [ ] Updated documentation Z

## Testing
- [ ] All tests pass (`cargo test`)
- [ ] Added new tests for changed functionality
- [ ] Manual testing completed

## Documentation Updates  
- [ ] Updated relevant .md files
- [ ] Added inline code documentation
- [ ] Updated API docs if applicable

## Breaking Changes
- None / List any breaking changes

## Screenshots
(If applicable)
```

## 📝 Documentation Requirements

### Always Update When Changing:

#### API Endpoints → `API_DOCUMENTATION.md`
- New endpoints
- Changed request/response formats
- New error codes
- Updated examples

#### System Architecture → `SYSTEM_ARCHITECTURE.md`
- New components or services
- Changed data flow
- New external dependencies
- Performance characteristics

#### Code Structure → `CODE_ORGANIZATION.md`
- New modules or files
- Changed responsibilities
- New design patterns
- Directory structure changes

#### Database Changes → `DATABASE_SCHEMA.md`
- New tables or columns
- Changed relationships
- New indexes
- Migration requirements

#### Test Coverage → `TESTING_STRATEGY.md`
- New test suites
- Changed testing approach
- New test utilities
- Coverage improvements

## 🚨 Code Quality Standards

### Rust Standards
```rust
// Use descriptive variable names
let athlete_weekly_kilometers = calculate_weekly_totals(activities);

// Add context to errors
.map_err(|e| ApiError::DatabaseError(format!("Failed to fetch athlete {}: {}", athlete_id, e)))?

// Document public APIs
/// Calculates injury risk using SSRD30 algorithm
/// 
/// Compares each activity against the maximum distance in the preceding 30 days.
/// Returns risk classification based on percentage increase.
pub fn analyze_ssrd30_risk(&self, activities: &[Activity]) -> RiskLevel {
    // implementation
}

// Use meaningful test names
#[test] 
fn test_ssrd30_correctly_identifies_moderate_risk_with_80_percent_increase() {
    // test implementation
}
```

### Error Handling Standards
```rust
// Good: Provide context
let athlete = self.db.get_athlete(&athlete_id)
    .await
    .map_err(|e| ApiError::DatabaseError(format!("Failed to fetch athlete {}: {}", athlete_id, e)))?;

// Good: Handle all error cases
match self.strava_client.get_activities().await {
    Ok(activities) => process_activities(activities),
    Err(e) => {
        log::error!("Strava API error: {}", e);
        return Err(ApiError::ExternalAPIError(format!("Strava unavailable: {}", e)));
    }
}
```

### Time Handling Standards
```rust
// Good: Use UTC for internal calculations
let start_date_utc = Utc::now() - Duration::days(7);
let activities = self.db.get_activities_since(start_date_utc).await?;

// Good: Convert to display timezone only at boundaries
let pacific_time = Los_Angeles.from_utc_datetime(&utc_time.naive_utc());

// Bad: Mix timezones in business logic
let mixed_calculation = pacific_time.timestamp() + utc_offset; // Don't do this
```

## 🔍 Debugging Guidelines

### Local Development
```bash
# Enable debug logging
export RUST_LOG=debug
cargo run

# Enable SQL query logging (if using sqlx)  
export RUST_LOG=sqlx=debug
cargo run

# Run with backtraces on panic
export RUST_BACKTRACE=1
cargo run
```

### Common Issues and Solutions

#### Database Connection Issues
```bash
# Check connection string
echo $DATABASE_URL

# Test basic connectivity
psql $DATABASE_URL -c "SELECT 1;"

# Verify tables exist
psql $DATABASE_URL -c "\dt"
```

#### Time Zone Problems
```rust
// Debug timezone conversions
println!("UTC time: {}", utc_time);
println!("Pacific time: {}", pacific_time);
println!("Offset: {}", pacific_time.offset());
```

#### API Response Issues
```rust
// Add debug logging to API handlers
log::debug!("Request received: {:?}", request);
log::debug!("Response sending: {:?}", response);
```

## 📊 Performance Guidelines

### Database Query Optimization
```rust
// Good: Use specific columns
SELECT id, name, team FROM athletes WHERE team = $1;

// Good: Use indexes
WHERE date >= $1 AND sport_type = $2  -- both columns indexed

// Good: Use LIMIT for large datasets
SELECT * FROM activities ORDER BY date DESC LIMIT 100;
```

### Memory Management
```rust
// Good: Use references when possible
fn process_activities(activities: &[Activity]) -> Result<Stats, ApiError>

// Good: Stream large datasets
let activities = sqlx::query_as("SELECT * FROM activities")
    .fetch(&pool);  // Returns stream, not Vec
```

### Error Handling Performance
```rust
// Good: Early returns
if athlete_name.is_empty() {
    return Err(ApiError::ValidationError("Athlete name required".to_string()));
}

// Good: Avoid unwrap() in production code
let distance = activity.distance
    .ok_or_else(|| ApiError::ValidationError("Distance required".to_string()))?;
```

## 🎯 Definition of Done

A change is complete when:
- [ ] **Functionality works** - Feature/fix behaves as expected
- [ ] **Tests pass** - `cargo test` succeeds
- [ ] **Code compiles** - No warnings with `cargo clippy`
- [ ] **Documentation updated** - Relevant docs reflect changes
- [ ] **Error handling** - Graceful handling of failure cases
- [ ] **Time zones handled** - Proper UTC/Pacific conversions
- [ ] **Performance acceptable** - No significant regressions
- [ ] **Security considered** - No new vulnerabilities introduced

## 🔄 Continuous Integration

### Pre-commit Checks
```bash
# Run before every commit
cargo fmt --check      # Code formatting
cargo clippy          # Linting
cargo test            # All tests
cargo build           # Compilation check
```

### Automated Checks (CI)
- **Compilation**: Code builds successfully
- **Testing**: All tests pass
- **Formatting**: Code follows Rust standards
- **Dependencies**: No security vulnerabilities
- **Documentation**: Docs build without errors

This workflow ensures consistent, high-quality contributions while minimizing the risk of introducing bugs or breaking changes.
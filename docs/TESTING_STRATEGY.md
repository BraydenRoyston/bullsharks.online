# 🧪 Testing Strategy

This document outlines the comprehensive testing strategy for BullSharks.online, ensuring reliability, maintainability, and confidence in changes.

## 🎯 Testing Philosophy

### Core Principles
1. **Comprehensive Coverage** - Test business logic thoroughly
2. **Fast Feedback** - Tests should run quickly during development
3. **Reliable** - Tests should be deterministic and not flaky
4. **Maintainable** - Tests should be easy to understand and update
5. **Agent-Friendly** - Clear structure for AI agents to understand and extend

### Testing Pyramid
```
    /\
   /  \     E2E Tests (Few)
  /    \    - Full API workflows
 /      \   - Production-like scenarios
/        \  
----------
|\      /|  Integration Tests (Some)
| \    / |  - Service layer interactions
|  \  /  |  - Database operations
|   \/   |  - External API mocking
|________|
|        |  Unit Tests (Many)  
|        |  - Individual functions
|        |  - Business logic
|        |  - Error handling
|________|
```

## 📁 Test Organization

### Directory Structure
```
src/tests/
├── mod.rs                      # Test module configuration
├── injury_risk_tests.rs        # Injury risk algorithm tests
├── team_stats_tests.rs         # Team competition logic tests
├── time_handling_tests.rs      # Time zone conversion tests
├── api_integration_tests.rs    # HTTP endpoint tests
├── database_tests.rs           # Database operation tests
└── helpers/                    # Test utilities and helpers
    ├── mod.rs
    ├── test_data.rs            # Test data builders
    └── assertions.rs           # Custom assertion helpers
```

### Test Module Pattern
```rust
// Each test module follows this pattern
use crate::models::*;
use crate::services::*;
use std::collections::HashMap;

// Helper functions at top
fn create_test_activity(date: &str, athlete: &str, distance_km: f64) -> BullSharkActivity {
    // Implementation
}

fn create_test_athlete(name: &str, team: &str) -> Athlete {
    // Implementation
}

// Test cases grouped by functionality
mod ssrd30_algorithm {
    use super::*;
    
    #[test]
    fn test_no_risk_scenario() { /* */ }
    
    #[test] 
    fn test_small_risk_scenario() { /* */ }
}

mod ten_percent_rule {
    use super::*;
    
    #[test]
    fn test_weekly_spike_detection() { /* */ }
}
```

## 🔬 Test Categories

### 1. Unit Tests

**Purpose**: Test individual functions and business logic in isolation

#### Example: Injury Risk Algorithm Tests
```rust
#[test]
fn test_ssrd30_correctly_classifies_moderate_risk() {
    let activities = vec![
        create_test_activity("2024-01-01", "John Doe", 10.0), // Baseline
        create_test_activity("2024-01-15", "John Doe", 18.0), // 80% increase
    ];

    let risky_weeks = analyze_ssrd30_test("John Doe", &activities);
    
    assert_eq!(risky_weeks.len(), 1);
    let risk = risky_weeks.values().next().unwrap();
    assert!(risk.risks[0].contains("SSRD30_MODERATE_RISK"));
    assert!(risk.risks[0].contains("80.0%"));
}
```

#### Coverage Areas:
- **Injury Risk Algorithms** - SSRD30 and 10% rule logic
- **Time Zone Conversions** - UTC ↔ Pacific Time handling  
- **Data Transformations** - Strava → Internal model conversion
- **Hash Generation** - Activity deduplication logic
- **Error Handling** - Edge cases and validation

### 2. Integration Tests  

**Purpose**: Test interactions between services and components

#### Example: Database Integration Test
```rust
#[tokio::test]
async fn test_activity_storage_and_retrieval() {
    let db = setup_test_database().await;
    
    // Arrange
    let activities = create_test_activities(5);
    
    // Act - Store activities
    db.insert_activities(&activities).await.unwrap();
    
    // Act - Retrieve activities  
    let retrieved = db.get_all_activities().await.unwrap();
    
    // Assert
    assert_eq!(retrieved.len(), 5);
    assert_eq!(retrieved[0].athlete_name, activities[0].athlete_name);
}
```

#### Coverage Areas:
- **Database Operations** - CRUD operations with real database
- **Service Interactions** - ActivityController with Database/StravaClient
- **External API Mocking** - Strava API responses
- **Authentication Flow** - OAuth token management

### 3. API Tests

**Purpose**: Test HTTP endpoints and API contracts

#### Example: API Endpoint Test
```rust
#[tokio::test]
async fn test_team_stats_endpoint() {
    let app = create_test_app().await;
    
    // Setup test data
    seed_test_activities(&app.database).await;
    
    // Make request
    let response = app.get("/team_stats").await;
    
    // Verify response
    assert_eq!(response.status(), 200);
    let stats: TeamStats = response.json().await.unwrap();
    assert!(stats.bulls.athlete_kilometers.len() > 0);
    assert!(stats.sharks.athlete_kilometers.len() > 0);
}
```

#### Coverage Areas:
- **Endpoint Functionality** - All API routes work correctly
- **Request/Response Format** - JSON serialization/deserialization
- **Error Responses** - Proper HTTP status codes and error messages
- **Authentication** - Protected endpoints require valid tokens

## 🛠️ Test Utilities

### Test Data Builders
```rust
// src/tests/helpers/test_data.rs

pub struct ActivityBuilder {
    activity: BullSharkActivity,
}

impl ActivityBuilder {
    pub fn new() -> Self {
        Self {
            activity: BullSharkActivity {
                id: "test_id".to_string(),
                date: Utc::now().with_timezone(&FixedOffset::east_opt(0).unwrap()),
                athlete_name: Some("Test Athlete".to_string()),
                distance: Some(5000.0), // 5km default
                sport_type: Some("Run".to_string()),
                // ... defaults for other fields
            }
        }
    }
    
    pub fn athlete(mut self, name: &str) -> Self {
        self.activity.athlete_name = Some(name.to_string());
        self
    }
    
    pub fn distance_km(mut self, km: f64) -> Self {
        self.activity.distance = Some(km * 1000.0);
        self
    }
    
    pub fn date(mut self, date_str: &str) -> Self {
        self.activity.date = parse_test_date(date_str);
        self
    }
    
    pub fn build(self) -> BullSharkActivity {
        self.activity
    }
}

// Usage in tests
let activity = ActivityBuilder::new()
    .athlete("John Doe")
    .distance_km(10.5)
    .date("2024-01-15")
    .build();
```

### Custom Assertions
```rust
// src/tests/helpers/assertions.rs

pub fn assert_risk_detected(risky_weeks: &HashMap<String, RiskyWeek>, risk_type: &str) {
    assert!(!risky_weeks.is_empty(), "Expected risk to be detected");
    let risk = risky_weeks.values().next().unwrap();
    assert!(risk.risks.iter().any(|r| r.contains(risk_type)), 
            "Expected risk type '{}' not found in: {:?}", risk_type, risk.risks);
}

pub fn assert_no_risk(risky_weeks: &HashMap<String, RiskyWeek>) {
    assert!(risky_weeks.is_empty(), 
            "Expected no risk, but found: {:?}", risky_weeks);
}
```

### Database Test Helpers
```rust
async fn setup_test_database() -> Database {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&test_database_url())
        .await
        .expect("Failed to connect to test database");
    
    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");
    
    Database::new(pool)
}

async fn cleanup_test_data(db: &Database) {
    // Clean up test data between tests
    sqlx::query("DELETE FROM bullshark_activities WHERE id LIKE 'test_%'")
        .execute(&db.pool)
        .await
        .expect("Failed to clean up test data");
}
```

## 📊 Current Test Coverage

### Injury Risk Tests (`injury_risk_tests.rs`)
- ✅ **SSRD30 No Risk** - 10% increase threshold
- ✅ **SSRD30 Small Risk** - 25% increase scenario  
- ✅ **SSRD30 Moderate Risk** - 80% increase scenario
- ✅ **SSRD30 Large Risk** - 150% increase scenario
- ✅ **30-Day Window** - Proper lookback validation
- ✅ **10% Rule Calculation** - Week-over-week spike detection
- ✅ **Risk Classification** - Boundary condition testing
- ✅ **String Conversion** - Risk type serialization

### Areas Needing Coverage
- [ ] **Team Statistics** - Bulls vs Sharks calculations
- [ ] **Time Zone Handling** - UTC/Pacific conversions
- [ ] **Database Operations** - CRUD with real database
- [ ] **API Endpoints** - HTTP request/response testing
- [ ] **Error Conditions** - Failure scenarios
- [ ] **Authentication** - OAuth flow testing

## 🚀 Running Tests

### Basic Test Commands
```bash
# Run all tests
cargo test

# Run specific test module
cargo test injury_risk

# Run with output (shows println! statements)
cargo test -- --nocapture

# Run tests in parallel (default)
cargo test

# Run tests sequentially (for database tests)
cargo test -- --test-threads=1
```

### Test Configuration
```rust
// In src/tests/mod.rs
#[cfg(test)]
pub mod test_config {
    use std::sync::Once;
    
    static INIT: Once = Once::new();
    
    pub fn setup() {
        INIT.call_once(|| {
            // Initialize logging for tests
            env_logger::init();
        });
    }
}
```

### Environment Variables for Testing
```bash
# Test database (separate from production)
TEST_DATABASE_URL=postgresql://localhost/bullsharks_test

# Reduced logging in tests
RUST_LOG=error

# Test-specific Strava credentials
TEST_STRAVA_CLIENT_ID=test_client_id
TEST_STRAVA_CLIENT_SECRET=test_secret
```

## 🔄 Test-Driven Development (TDD)

### TDD Cycle for New Features
```
1. Write failing test  →  2. Write minimal code  →  3. Refactor
         ↑                                                ↓
4. Repeat cycle  ←  ←  ←  ←  ←  ←  ←  ←  ←  ←  ←  ←  ←  ←
```

#### Example: Adding New Risk Algorithm
```rust
// 1. Write failing test first
#[test]
fn test_new_risk_algorithm_detects_overtraining() {
    let activities = create_overtraining_scenario();
    
    let risks = analyze_overtraining_risk(activities);
    
    assert!(risks.contains("OVERTRAINING_RISK"));
}

// 2. Write minimal implementation
fn analyze_overtraining_risk(activities: &[Activity]) -> Vec<String> {
    vec!["OVERTRAINING_RISK".to_string()] // Minimal implementation
}

// 3. Refactor and improve
fn analyze_overtraining_risk(activities: &[Activity]) -> Vec<String> {
    // Proper implementation with business logic
}
```

## 🔍 Testing Best Practices

### Test Naming Convention
```rust
// Good: Descriptive test names
#[test]
fn test_ssrd30_correctly_identifies_small_risk_with_25_percent_increase() { }

#[test]  
fn test_ten_percent_rule_ignores_increases_below_20km_threshold() { }

// Bad: Vague test names
#[test]
fn test_risk() { }

#[test]
fn test_algorithm() { }
```

### Arrange-Act-Assert Pattern
```rust
#[test]
fn test_team_statistics_calculation() {
    // Arrange - Set up test data
    let activities = vec![
        create_activity("John", "bulls", 10.0),
        create_activity("Jane", "sharks", 8.0),
    ];
    
    // Act - Execute the functionality
    let stats = calculate_team_stats(&activities);
    
    // Assert - Verify the results
    assert_eq!(stats.bulls.total_km, 10.0);
    assert_eq!(stats.sharks.total_km, 8.0);
}
```

### Error Testing Pattern
```rust
#[test]
fn test_invalid_athlete_name_returns_validation_error() {
    let invalid_activity = create_activity_with_empty_name();
    
    let result = validate_activity(&invalid_activity);
    
    assert!(result.is_err());
    match result.unwrap_err() {
        ApiError::ValidationError(msg) => {
            assert!(msg.contains("Athlete name is required"));
        },
        other => panic!("Expected ValidationError, got {:?}", other),
    }
}
```

### Mock External Dependencies
```rust
#[tokio::test]
async fn test_strava_api_failure_handling() {
    let mock_client = MockStravaClient::new()
        .expect_get_activities()
        .returning(|| Err(StravaError::ApiUnavailable));
    
    let controller = ActivityController::new(db, mock_client);
    
    let result = controller.sync_activities().await;
    
    assert!(matches!(result, Err(ApiError::ExternalAPIError(_))));
}
```

## 📈 Test Metrics

### Coverage Goals
- **Business Logic**: 90%+ coverage
- **API Endpoints**: 100% coverage  
- **Error Paths**: 80%+ coverage
- **Integration Points**: 100% coverage

### Performance Benchmarks
```rust
#[test]
fn test_injury_risk_analysis_performance() {
    let large_dataset = create_activities(1000); // 1000 activities
    
    let start = std::time::Instant::now();
    let _risks = analyze_injury_risks(&large_dataset);
    let duration = start.elapsed();
    
    // Should process 1000 activities in under 100ms
    assert!(duration.as_millis() < 100);
}
```

## 🔧 Continuous Testing

### Pre-commit Hooks
```bash
#!/bin/sh
# .git/hooks/pre-commit

cargo fmt --check || exit 1
cargo clippy -- -D warnings || exit 1  
cargo test || exit 1

echo "All tests passed! ✅"
```

### CI/CD Pipeline
```yaml
# GitHub Actions example
test:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v2
    - name: Install Rust
      uses: actions-rs/toolchain@v1
    - name: Run tests
      run: cargo test --all-features
    - name: Check formatting
      run: cargo fmt --check
    - name: Run clippy
      run: cargo clippy -- -D warnings
```

This comprehensive testing strategy ensures code quality, catches regressions early, and provides confidence when making changes to the codebase.
# 🔧 Troubleshooting Guide

This document provides solutions to common issues encountered when developing or deploying BullSharks.online. Use this guide to quickly diagnose and resolve problems.

## 🚨 Common Issues

### 1. Database Connection Problems

#### Symptoms
- `Failed to connect to database`
- `Connection refused` errors
- Tests failing with database errors

#### Diagnosis
```bash
# Check if database is running
pg_ctl status

# Test connection directly
psql $DATABASE_URL -c "SELECT 1;"

# Verify environment variable
echo $DATABASE_URL
```

#### Solutions
```bash
# Fix 1: Start PostgreSQL service
sudo service postgresql start
# or on macOS with Homebrew
brew services start postgresql

# Fix 2: Create database if it doesn't exist
createdb bullsharks

# Fix 3: Fix connection string format
# Correct format: postgresql://username:password@host:port/database
export DATABASE_URL="postgresql://postgres:password@localhost:5432/bullsharks"

# Fix 4: Grant permissions
psql -c "GRANT ALL PRIVILEGES ON DATABASE bullsharks TO your_user;"
```

### 2. Strava API Authentication Issues

#### Symptoms
- `Invalid authentication credentials`
- `Token expired` errors
- OAuth flow failures

#### Diagnosis
```bash
# Check environment variables
echo $STRAVA_CLIENT_ID
echo $STRAVA_CLIENT_SECRET
echo $STRAVA_CLUB_ID

# Test API connectivity
curl -X GET "https://www.strava.com/api/v3/clubs/$STRAVA_CLUB_ID/activities" \
  -H "Authorization: Bearer $ACCESS_TOKEN"
```

#### Solutions
```bash
# Fix 1: Verify credentials in Strava Developer Console
# Visit: https://www.strava.com/settings/api

# Fix 2: Refresh OAuth token
# Use refresh_token to get new access_token via Strava OAuth endpoint

# Fix 3: Check rate limits
# Strava: 200 requests per 15 minutes, 2000 per day

# Fix 4: Verify club permissions
# Ensure the authenticated user has access to the club
```

### 3. Time Zone Conversion Errors

#### Symptoms
- Activities showing wrong dates
- Week calculations off by one day
- Statistics not matching expected periods

#### Diagnosis
```rust
// Add debug logging to time conversions
println!("UTC time: {}", utc_time);
println!("Pacific time: {}", pacific_time);
println!("Week start: {}", week_start);
```

#### Solutions
```rust
// Fix 1: Always store UTC in database
let utc_time = Utc::now();
sqlx::query("INSERT INTO activities (date) VALUES ($1)")
    .bind(utc_time)
    .execute(&pool).await?;

// Fix 2: Convert to display timezone only at API boundary
let utc_from_db: DateTime<Utc> = row.get("date");
let pacific_display = Los_Angeles.from_utc_datetime(&utc_from_db.naive_utc());

// Fix 3: Use consistent week calculation
let days_since_monday = date.weekday().num_days_from_monday();
let week_start = date.date().and_hms_opt(0, 0, 0).unwrap() - Duration::days(days_since_monday as i64);
```

### 4. Test Failures

#### Symptoms
- Tests fail intermittently
- Database tests conflict with each other
- Time-dependent tests fail

#### Solutions
```bash
# Fix 1: Run tests sequentially for database tests
cargo test -- --test-threads=1

# Fix 2: Use separate test database
export TEST_DATABASE_URL="postgresql://localhost/bullsharks_test"

# Fix 3: Clean up test data
# Add cleanup in test teardown
async fn cleanup_test_data() {
    sqlx::query("DELETE FROM activities WHERE id LIKE 'test_%'")
        .execute(&pool).await.unwrap();
}
```

### 5. Compilation Issues

#### Symptoms
- `cargo build` fails
- Missing dependency errors
- Version conflicts

#### Solutions
```bash
# Fix 1: Update dependencies
cargo update

# Fix 2: Clear cache and rebuild
cargo clean
cargo build

# Fix 3: Check Rust version
rustc --version
# Update if needed: rustup update

# Fix 4: Fix dependency conflicts in Cargo.toml
[dependencies]
tokio = { version = "1", features = ["full"] }  # Specify features explicitly
```

## 🐛 Debugging Techniques

### 1. Enable Debug Logging
```bash
# Enable all debug logs
export RUST_LOG=debug
cargo run

# Enable specific module logging
export RUST_LOG=bullsharks::services::activity_controller=debug
cargo run

# Enable SQL query logging
export RUST_LOG=sqlx=debug
cargo run
```

### 2. Add Debug Prints
```rust
// Debug activity processing
println!("Processing {} activities", activities.len());
for activity in &activities {
    println!("Activity: {} - {} - {:.1}km", 
        activity.id, 
        activity.date.format("%Y-%m-%d"),
        activity.distance.unwrap_or(0.0) / 1000.0
    );
}

// Debug database queries  
println!("Executing query with params: start={}, end={}", start_date, end_date);
```

### 3. Use Rust Debugger
```bash
# Install debugger
cargo install gdb

# Run with debugger
rust-gdb target/debug/bullsharks-server

# Set breakpoints and inspect variables
(gdb) break src/services/activity_controller.rs:123
(gdb) run
(gdb) print activities
```

### 4. Test Individual Components
```rust
#[test]
fn debug_specific_issue() {
    let test_data = create_problematic_scenario();
    
    // Add detailed assertions
    println!("Input: {:?}", test_data);
    let result = function_under_test(test_data);
    println!("Output: {:?}", result);
    
    // Break down the problem
    assert_eq!(result.len(), expected_len);
    assert_eq!(result[0].field, expected_value);
}
```

## 🔍 Performance Issues

### 1. Slow Database Queries

#### Symptoms
- API responses taking >1 second
- High CPU usage on database
- Timeout errors

#### Diagnosis
```sql
-- Enable query logging in PostgreSQL
SET log_statement = 'all';
SET log_duration = on;

-- Check slow queries
SELECT query, mean_time, calls 
FROM pg_stat_statements 
ORDER BY mean_time DESC 
LIMIT 10;
```

#### Solutions
```sql
-- Add missing indexes
CREATE INDEX idx_activities_date_sport ON bullshark_activities(date, sport_type);
CREATE INDEX idx_activities_athlete_name ON bullshark_activities(athlete_name);

-- Optimize queries
-- Before: Fetches all columns
SELECT * FROM bullshark_activities;

-- After: Fetch only needed columns
SELECT id, date, athlete_name, distance FROM bullshark_activities;

-- Use LIMIT for large datasets
SELECT * FROM bullshark_activities ORDER BY date DESC LIMIT 100;
```

### 2. Memory Usage Issues

#### Symptoms
- High memory consumption
- Out of memory errors
- Slow performance

#### Solutions
```rust
// Use streaming instead of loading all data
let activities = sqlx::query_as::<_, BullSharkActivity>(
    "SELECT * FROM bullshark_activities"
).fetch(&pool);  // Returns Stream, not Vec

// Process in batches
const BATCH_SIZE: usize = 100;
for chunk in activities.chunks(BATCH_SIZE) {
    process_batch(chunk).await?;
}

// Use references instead of cloning
fn process_activities(activities: &[Activity]) -> Stats {  // &[Activity] not Vec<Activity>
    // Process without taking ownership
}
```

### 3. High CPU Usage

#### Symptoms
- Server using 100% CPU
- Slow response times
- System becoming unresponsive

#### Solutions
```rust
// Optimize injury risk analysis
// Before: O(n²) algorithm
for activity in &activities {
    for previous in &activities {  // Bad: nested loop
        if previous.date < activity.date {
            // compare
        }
    }
}

// After: O(n log n) with sorting
activities.sort_by_key(|a| a.date);
for (i, activity) in activities.iter().enumerate() {
    let previous_activities = &activities[0..i];  // Only look at previous
    // process
}
```

## 🌐 Deployment Issues

### 1. Cloud Run Deployment Failures

#### Symptoms
- Deployment times out
- Container won't start
- Health check failures

#### Solutions
```dockerfile
# Fix 1: Optimize Docker build
# Use multi-stage build to reduce image size
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/server /app/server
CMD ["/app/server"]

# Fix 2: Set proper health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:8080/health || exit 1
```

### 2. Environment Variable Issues

#### Symptoms
- Configuration not loading
- Features not working in production
- Different behavior than local

#### Solutions
```bash
# Fix 1: Verify all required env vars are set in Cloud Run
gcloud run services describe bullsharks-server \
  --region=us-central1 \
  --format="value(spec.template.spec.template.spec.containers[0].env[])"

# Fix 2: Use Google Secret Manager for sensitive data
gcloud secrets create strava-client-secret --data-file=secret.txt

# Fix 3: Set default values in code
let database_url = std::env::var("DATABASE_URL")
    .unwrap_or_else(|_| "postgresql://localhost/bullsharks".to_string());
```

### 3. Database Migration Issues

#### Symptoms
- Schema mismatch errors
- Migration fails to run
- Data corruption

#### Solutions
```sql
-- Fix 1: Always backup before migrations
pg_dump bullsharks > backup_$(date +%Y%m%d).sql

-- Fix 2: Test migrations on copy first
CREATE DATABASE bullsharks_test_migration;
-- Run migration on test database first

-- Fix 3: Use backward-compatible changes
-- Add new column as nullable first
ALTER TABLE activities ADD COLUMN new_field VARCHAR;
-- Update application code
-- Then make non-nullable if needed
ALTER TABLE activities ALTER COLUMN new_field SET NOT NULL;
```

## 🔐 Security Issues

### 1. API Security

#### Symptoms
- Unauthorized access
- Missing authentication
- Data exposure

#### Solutions
```rust
// Fix 1: Add proper authentication middleware
async fn require_auth(req: Request<Body>, next: Next<Body>) -> Response {
    let auth_header = req.headers().get("authorization");
    match auth_header {
        Some(header) => {
            if validate_token(header).await {
                next.run(req).await
            } else {
                Response::builder()
                    .status(401)
                    .body("Unauthorized".into())
                    .unwrap()
            }
        },
        None => Response::builder()
            .status(401)
            .body("Missing authorization header".into())
            .unwrap()
    }
}

// Fix 2: Sanitize inputs
fn sanitize_athlete_name(name: &str) -> Result<String, ApiError> {
    if name.len() > 100 {
        return Err(ApiError::ValidationError("Name too long".to_string()));
    }
    if name.contains('<') || name.contains('>') {
        return Err(ApiError::ValidationError("Invalid characters".to_string()));
    }
    Ok(name.trim().to_string())
}
```

## 📞 Getting Help

### 1. Check Logs
```bash
# Local development
RUST_LOG=debug cargo run 2>&1 | tee debug.log

# Production (Cloud Run)
gcloud logs read --service=bullsharks-server --limit=50

# Database logs
tail -f /var/log/postgresql/postgresql-13-main.log
```

### 2. Create Minimal Reproduction
```rust
#[test]  
fn reproduce_issue() {
    // Minimal test case that demonstrates the problem
    let problematic_input = create_specific_scenario();
    let result = function_with_issue(problematic_input);
    
    // This should pass but currently fails
    assert_eq!(result.expected_field, expected_value);
}
```

### 3. Gather System Information
```bash
# Rust version
rustc --version

# Dependencies
cargo tree

# System info
uname -a
free -h
df -h

# Database version
psql --version
```

### 4. Create Issue Report Template
```markdown
## Issue Description
Brief description of the problem

## Steps to Reproduce
1. Step one
2. Step two
3. Step three

## Expected Behavior
What should happen

## Actual Behavior  
What actually happens

## Environment
- OS: [e.g., macOS 13.0]
- Rust version: [e.g., 1.70.0]
- Database: [e.g., PostgreSQL 13]
- Branch: [e.g., main]

## Logs
```
[Include relevant log output]
```

## Additional Context
Any other relevant information
```

This troubleshooting guide should help you quickly identify and resolve most common issues. When in doubt, start with the logs and work systematically through the potential causes.
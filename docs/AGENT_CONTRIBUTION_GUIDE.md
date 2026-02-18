# 🤖 Agent Contribution Guide

This guide is specifically designed for AI agents contributing to the BullSharks.online repository. It provides essential context, patterns, and workflows for effective collaboration.

## 🎯 Quick Start Checklist

Before making any changes:
- [ ] Read this guide completely
- [ ] Review [System Architecture](SYSTEM_ARCHITECTURE.md) for high-level understanding
- [ ] Check [Code Organization](CODE_ORGANIZATION.md) for module responsibilities
- [ ] Examine [Database Schema](DATABASE_SCHEMA.md) if working with data models
- [ ] Follow [Development Workflow](DEVELOPMENT_WORKFLOW.md) for safe changes

## 🏗️ System Overview

**BullSharks.online** is a Rust/Axum REST API that aggregates Strava activities for the BullSharks running club. Key concepts:

### Core Purpose
- **Primary**: Aggregate and serve Strava activities for team competition (Bulls vs Sharks)
- **Secondary**: Provide injury risk analysis for athletes using SSRD30 and 10% rule algorithms
- **Data Source**: Strava Club API (auto-synced every 2 minutes)
- **Deployment**: Google Cloud Run (serverless, scales to zero)

### Key Entities
```
Club Activities (from Strava) → BullShark Activities (stored) → Team Stats & Injury Analysis
                                      ↓
                              Athletes (database) → Training Data Analysis
```

## 📁 Codebase Structure

```
src/
├── api/                 # HTTP endpoint handlers (thin layer)
│   ├── activities.rs    # Activity endpoints
│   ├── athletes.rs      # Athlete endpoints  
│   └── health.rs        # Health checks
├── services/            # Business logic (thick layer)
│   ├── activity_controller.rs  # Core business logic
│   ├── database.rs      # Database operations
│   ├── strava_client.rs # External API client
│   └── auth_controller.rs # OAuth handling
├── models/              # Data structures
│   ├── bullshark.rs     # Core activity model
│   ├── athlete.rs       # Athlete model
│   ├── team_stats.rs    # Statistics models
│   ├── injury_risk.rs   # Risk analysis types
│   └── athlete_training_data.rs # Training analysis
├── utils/               # Helper functions
├── tests/               # Test suites
└── main.rs             # Application entry point
```

## 🔄 Development Workflow

### 1. Understanding the Task
- **Identify the domain**: API, business logic, data model, or infrastructure?
- **Find the right module**: Use [Code Organization](CODE_ORGANIZATION.md) as a guide
- **Check existing tests**: Look at `src/tests/` for patterns and coverage

### 2. Making Changes
```bash
# Create feature branch
git checkout -b feature/descriptive-name

# Make changes following existing patterns
# Run tests frequently
cargo test

# Compile check
cargo check

# Full compilation
cargo build
```

### 3. Testing Strategy
- **Unit tests**: Place in `src/tests/` directory (not inline with code)
- **Integration tests**: Test API endpoints and business logic flows
- **Always run**: `cargo test` before committing
- **Test coverage**: Aim for comprehensive coverage of new logic

### 4. Documentation Updates
**Critical**: Update documentation with every change
- New endpoints → `API_DOCUMENTATION.md`
- Architecture changes → `SYSTEM_ARCHITECTURE.md`
- New modules → `CODE_ORGANIZATION.md`
- Database changes → `DATABASE_SCHEMA.md`
- New tests → `TESTING_STRATEGY.md`

### 5. Pull Request Guidelines
- **Title**: Clear, descriptive (e.g., "Add SSRD30 injury risk analysis")
- **Description**: Explain what changed and why
- **Tests**: Include test results and coverage
- **Documentation**: Note which docs were updated
- **Breaking changes**: Clearly flag any API changes

## 🧠 Mental Models

### Data Flow Pattern
```
External API → Service Layer → Database ← API Layer → HTTP Response
    ↓              ↓             ↓           ↓
  Strava       ActivityController  SQLx    Axum Handlers
```

### Error Handling Pattern
- Use `ApiError` enum for all error types
- Convert external errors at boundaries
- Return meaningful HTTP status codes
- Log errors at the source

### Time Zone Handling
- **Storage**: UTC in database
- **Display**: Pacific Time (Los Angeles) for users
- **Pattern**: Convert at API boundaries, not in business logic

### Testing Pattern
- **Arrange**: Set up test data
- **Act**: Call the function/endpoint
- **Assert**: Verify the outcome
- **Organize**: Group related tests in dedicated modules

## 🚨 Common Pitfalls

### 1. Time Zone Confusion
❌ **Wrong**: Mixing time zones in business logic
```rust
let pacific_time = Los_Angeles.from_local_datetime(&naive).unwrap();
let result = some_calculation(pacific_time); // Business logic shouldn't care about timezone
```

✅ **Right**: Keep business logic timezone-agnostic
```rust
let utc_time = Utc.from_local_datetime(&naive).unwrap();
let result = some_calculation(utc_time); // Business logic uses UTC
```

### 2. Database Transactions
❌ **Wrong**: Multiple separate queries
```rust
db.insert_activity(activity).await?;
db.update_stats(stats).await?; // Potential inconsistency
```

✅ **Right**: Use transactions for related operations
```rust
let mut tx = db.begin().await?;
tx.insert_activity(activity).await?;
tx.update_stats(stats).await?;
tx.commit().await?;
```

### 3. Error Context
❌ **Wrong**: Generic errors
```rust
.map_err(|e| ApiError::DatabaseError)?
```

✅ **Right**: Contextual errors
```rust
.map_err(|e| ApiError::DatabaseError(format!("Failed to fetch athlete {}: {}", athlete_id, e)))?
```

## 📊 Key Business Logic

### Injury Risk Analysis
The repository contains sophisticated injury risk algorithms:

1. **SSRD30**: Compares each run against the longest run in preceding 30 days
2. **10% Rule**: Detects week-over-week volume spikes >10%
3. **Risk Classification**: No risk (<10%), small (10-30%), moderate (30-100%), large (>100%)

**Location**: `src/services/activity_controller.rs` → `analyze_injury_risks()`
**Tests**: `src/tests/injury_risk_tests.rs` (8 comprehensive tests)

### Team Competition
Bulls vs Sharks weekly competition tracking:
- **Weekly aggregation**: Monday-Sunday periods
- **Running totals**: Cumulative team distances
- **Individual tracking**: Per-athlete contributions

**Location**: `src/services/activity_controller.rs` → `get_team_stats()`

## 🔍 Debugging Tips

### Local Development
```bash
# Set up environment
cp .env.example .env
# Edit .env with your configs

# Run with logs
RUST_LOG=debug cargo run

# Run specific tests
cargo test injury_risk

# Check compilation
cargo check
```

### Common Issues
1. **Database connection**: Check `DATABASE_URL` in `.env`
2. **Strava API**: Verify `STRAVA_CLIENT_ID` and `STRAVA_CLIENT_SECRET`
3. **Time zones**: Always use UTC for internal calculations
4. **Tests failing**: Ensure test database is set up correctly

## 📝 Code Style

- **Rust conventions**: Follow standard Rust naming and formatting
- **Error handling**: Use `?` operator, provide context
- **Documentation**: Add doc comments for public APIs
- **Comments**: Explain "why" not "what"
- **Functions**: Keep focused, single responsibility
- **Testing**: Test both happy and error paths

## 🎯 Success Metrics

Your contribution is successful when:
- [ ] All tests pass (`cargo test`)
- [ ] Code compiles without warnings (`cargo build`)
- [ ] Documentation is updated and accurate
- [ ] Changes follow existing patterns and conventions
- [ ] Error handling is comprehensive
- [ ] Time zones are handled correctly

## 🚀 Next Steps

1. **Read the system docs**: Start with [System Architecture](SYSTEM_ARCHITECTURE.md)
2. **Explore the code**: Use [Code Organization](CODE_ORGANIZATION.md) as your guide
3. **Run tests**: `cargo test` to understand existing behavior
4. **Make small changes**: Start with minor improvements to get familiar
5. **Update docs**: Always keep documentation current

Remember: The goal is not just working code, but maintainable, well-documented code that the next agent (or human) can easily understand and extend.
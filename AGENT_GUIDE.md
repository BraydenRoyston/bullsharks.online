# 🤖 AI Agent Development Guide

*Optimized for AI agent + human collaboration on BullSharks.online*

## 🎯 Project Overview

**What**: REST API server for Bulls vs Sharks running club data aggregation  
**Tech**: Rust + Axum + PostgreSQL + Google Cloud Run  
**Data Source**: Strava Club API  
**Live API**: https://bullsharks-server-288102886042.us-central1.run.app

## 🏗️ Architecture Quick Reference

```
API Layer     → src/api/          (HTTP handlers)
Business Logic → src/services/    (Core algorithms) 
Data Models   → src/models/       (Structs & types)
Database      → src/utils/        (DB utilities)
Main          → src/main.rs       (Server setup)
```

**Key Services:**
- `ActivityController` - Core business logic, injury risk algorithms
- `StravaClient` - External API integration  
- `Database` - PostgreSQL operations
- `AuthController` - OAuth & authentication

## 🔍 Common Tasks & File Locations

### Adding New Endpoints:
1. **Handler**: `src/api/[domain].rs` - HTTP request/response logic
2. **Route**: `src/main.rs` - Wire up the route
3. **Service**: `src/services/[domain]_controller.rs` - Business logic
4. **Model**: `src/models/[entity].rs` - Data structures
5. **Tests**: Add to relevant `#[cfg(test)]` modules

### Database Operations:
- **Queries**: `src/utils/database_utils.rs`
- **Migrations**: Ask human - handled via Supabase
- **Models**: `src/models/` - Struct definitions with SQLx derives

### Algorithm Work:
- **Injury Risk**: `src/services/activity_controller.rs::analyze_injury_risks`
- **Team Stats**: `src/services/activity_controller.rs::get_team_stats`
- **Data Processing**: Look in service layer, not API handlers

### Configuration:
- **Dependencies**: `Cargo.toml`
- **Environment**: `.env.example` (never commit `.env`)
- **Docker**: `Dockerfile` (for Cloud Run deployment)

## 🧪 Testing Strategy

### Unit Tests:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_algorithm_logic() {
        // Test core logic without external dependencies
    }
}
```

**Location**: Inline with implementation files  
**Focus**: Pure functions, algorithms, data transformations  
**Run**: `cargo test`

### Integration Tests:
- **API Tests**: Use `reqwest` to test endpoints
- **Database Tests**: Use test database or mocks
- **External API**: Mock Strava responses

### Test Data:
- **Create helpers**: `fn create_test_activity()` pattern
- **Use realistic data**: Actual distance/time values
- **Test edge cases**: Empty data, boundary conditions

## 📂 File Reading Strategy (Context Management)

### High-Value Files (Read First):
```bash
# Architecture understanding
src/main.rs --limit 100          # Server setup & routes
src/services/mod.rs --limit 50   # Service overview
src/models/mod.rs --limit 50     # Data model overview

# Specific feature work
src/api/activities.rs            # Activity endpoints
src/services/activity_controller.rs  # Core business logic
```

### Supporting Files (Read as Needed):
```bash
# Database operations
src/utils/database_utils.rs --limit 100

# External integrations  
src/services/strava_client.rs --limit 100

# Specific models
src/models/[entity].rs --limit 50
```

### Avoid Reading (Large/Generated):
- `target/` directory (build artifacts)
- `.git/` directory (version control)
- `docs/` files (unless specifically needed)

## 🛠️ Development Workflow

### For New Features:
1. **Understand**: Read relevant service & model files
2. **Plan**: Identify files to modify (handler → service → model)
3. **Implement**: Start with tests, then implement
4. **Test**: `cargo test` + manual API testing
5. **Document**: Update API docs if new endpoints

### For Bug Fixes:
1. **Locate**: Use `grep -r "error_message"` to find relevant code
2. **Test**: Create failing test case first
3. **Fix**: Minimal changes to pass tests  
4. **Validate**: Ensure fix doesn't break other functionality

### For Algorithm Work:
1. **Read tests first**: Understand expected behavior
2. **Focus on service layer**: Business logic lives in `src/services/`
3. **Test thoroughly**: Algorithms need comprehensive edge case testing
4. **Document changes**: Update comments & commit messages clearly

## 🚨 Safety Guidelines

### Database Operations:
- **Never hardcode credentials** - use environment variables
- **Use SQLx compile-time checking** - run `cargo check` before commits
- **Test queries separately** - validate SQL before integration

### External APIs:
- **Handle rate limits** - Strava has API quotas
- **Mock in tests** - Don't hit real APIs during testing
- **Error handling** - Network calls can fail

### Deployment:
- **Ask before deploying** - Production changes need human approval
- **Test locally first** - `cargo run` and validate endpoints
- **Check logs** - Use Cloud Run logs for debugging

## 📊 Key Metrics & Monitoring

### Performance Targets:
- **API Response**: <200ms for data endpoints
- **Database Queries**: <50ms average
- **Memory Usage**: <512MB (Cloud Run limit)

### Health Checks:
- **Endpoint**: `GET /health` - Should return 200 OK
- **Database**: Connection pool status
- **External APIs**: Strava API connectivity

### Logging:
- **Structured logging** with `tracing` crate
- **Error context** - Include relevant data in error messages
- **Performance logs** - Track slow queries/requests

---

*This guide optimizes AI agent workflows while maintaining code quality and safety.*
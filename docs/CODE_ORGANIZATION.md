# 📁 Code Organization

This document explains the structure and responsibilities of each module in the BullSharks.online codebase. Use this as a guide to understand where different functionality lives and where to make changes.

## 🗂️ Directory Structure

```
src/
├── api/                     # HTTP endpoint handlers (thin layer)
│   ├── mod.rs              # API module exports
│   ├── activities.rs       # Activity-related endpoints
│   ├── athletes.rs         # Athlete-related endpoints  
│   └── health.rs           # Health check endpoint
├── services/               # Business logic (thick layer)
│   ├── mod.rs              # Service module exports
│   ├── activity_controller.rs  # Core business logic controller
│   ├── database.rs         # Database operations and queries
│   ├── strava_client.rs    # Strava API client
│   └── auth_controller.rs  # OAuth authentication handling
├── models/                 # Data structures and types
│   ├── mod.rs              # Model module exports
│   ├── bullshark.rs        # Core activity data model
│   ├── athlete.rs          # Athlete data model
│   ├── team_stats.rs       # Team statistics models
│   ├── athlete_training_data.rs  # Training analysis models
│   ├── injury_risk.rs      # Injury risk types and enums
│   ├── club.rs             # Strava club activity model
│   └── oauth.rs            # OAuth-related models
├── utils/                  # Helper functions and utilities
│   ├── mod.rs              # Utils module exports
│   ├── startup_utils.rs    # Application startup helpers
│   └── database_utils.rs   # Database configuration utilities
├── tests/                  # Test suites (organized by domain)
│   ├── mod.rs              # Test module exports
│   └── injury_risk_tests.rs  # Injury risk algorithm tests
├── error.rs                # Error types and handling
└── main.rs                 # Application entry point
```

## 🎯 Module Responsibilities

### API Layer (`src/api/`)

**Purpose**: Handle HTTP requests and responses (thin layer)

#### `activities.rs`
- **Responsibility**: Activity-related HTTP endpoints
- **Key Functions**:
  - `get_all_activities()` - Fetch all stored activities
  - `get_activities_from_window()` - Fetch activities in date range
  - `get_activities_from_week()` - Current week's activities  
  - `get_activities_from_month()` - Current month's activities
  - `populate_new_activities()` - Trigger activity sync (internal)
- **Pattern**: Extract parameters → delegate to `ActivityController` → return HTTP response

#### `athletes.rs`
- **Responsibility**: Athlete-related HTTP endpoints
- **Key Functions**:
  - `get_all_athletes()` - Fetch all registered athletes
  - `get_team_stats()` - Bulls vs Sharks competition statistics
  - `get_athletes_training_data()` - Training analysis for all athletes
- **Pattern**: Validate request → call service layer → format response

#### `health.rs`
- **Responsibility**: Health check endpoint
- **Key Functions**:
  - `health_check()` - System health status
- **Dependencies**: Database connectivity, Strava API availability

### Service Layer (`src/services/`)

**Purpose**: Core business logic (thick layer)

#### `activity_controller.rs` ⭐ (Core Component)
- **Responsibility**: Central business logic orchestrator
- **Key Functions**:
  - `populate_new_activities()` - Sync activities from Strava
  - `get_team_stats()` - Calculate Bulls vs Sharks statistics
  - `get_all_athletes_training_data()` - Generate training analysis
  - `analyze_injury_risks()` - SSRD30 and 10% rule analysis
  - `convert_activities()` - Transform Strava → internal format
- **Design Pattern**: Facade pattern - orchestrates other services
- **Dependencies**: Database, StravaClient

#### `database.rs`
- **Responsibility**: Database operations and queries
- **Key Functions**:
  - `insert_activities()` - Bulk insert activities with deduplication
  - `get_all_activities()` - Query all activities
  - `get_activities_from_window()` - Query activities by date range
  - `read_all_athletes()` - Query registered athletes
- **Pattern**: Repository pattern with SQLx
- **Error Handling**: Convert SQLx errors to `ApiError`

#### `strava_client.rs`
- **Responsibility**: External Strava API integration
- **Key Functions**:
  - `read_last_100_activities()` - Fetch recent club activities
  - `health_check()` - Verify API connectivity
- **Authentication**: OAuth token management
- **Rate Limiting**: Respects Strava API limits
- **Error Handling**: Convert HTTP errors to `ApiError`

#### `auth_controller.rs`
- **Responsibility**: OAuth authentication flow
- **Key Functions**:
  - OAuth token refresh and management
  - Strava API authentication
- **Dependencies**: Strava OAuth endpoints

### Models Layer (`src/models/`)

**Purpose**: Data structures and type definitions

#### `bullshark.rs` ⭐ (Core Model)
- **Responsibility**: Internal activity representation
- **Key Struct**: `BullSharkActivity`
- **Fields**: id, date, athlete_name, distance, moving_time, sport_type, etc.
- **Serialization**: Serde for JSON conversion
- **Design**: Hash-based ID for deduplication

#### `athlete.rs`
- **Responsibility**: Athlete data model
- **Key Struct**: `Athlete`
- **Fields**: id, name, team ("bulls" or "sharks"), event
- **Purpose**: Define team membership for competition

#### `team_stats.rs`
- **Responsibility**: Competition statistics models
- **Key Structs**: 
  - `TeamStats` - Overall competition data
  - `TeamData` - Per-team statistics
  - `WeekData` - Weekly aggregations
- **Purpose**: Bulls vs Sharks competition tracking

#### `athlete_training_data.rs`
- **Responsibility**: Training analysis models
- **Key Structs**:
  - `AllAthletesTrainingData` - Response wrapper
  - `AthleteTrainingData` - Per-athlete analysis
  - `RiskyWeek` - Injury risk detection results
- **Purpose**: Individual training analysis and risk assessment

#### `injury_risk.rs`
- **Responsibility**: Risk analysis type definitions
- **Key Enum**: `InjuryRiskType`
- **Values**: SSRD30 risk levels, HighVolumeSpike
- **Pattern**: String conversion for API responses

#### `club.rs`
- **Responsibility**: Strava API response models
- **Key Struct**: `ClubActivity`
- **Purpose**: Represent data as received from Strava
- **Conversion**: Transforms to `BullSharkActivity`

#### `oauth.rs`
- **Responsibility**: OAuth-related data models
- **Purpose**: Strava authentication flow

### Utilities (`src/utils/`)

**Purpose**: Helper functions and configuration

#### `startup_utils.rs`
- **Responsibility**: Application initialization
- **Key Functions**:
  - `get_db()` - Database connection setup
  - `get_strava_config()` - Strava API configuration
  - `create_server()` - Axum server initialization
- **Pattern**: Builder pattern for complex initialization

#### `database_utils.rs`
- **Responsibility**: Database configuration helpers
- **Purpose**: Connection string parsing, pool configuration

### Testing (`src/tests/`)

**Purpose**: Test suites organized by domain

#### `injury_risk_tests.rs`
- **Responsibility**: Test injury risk algorithms
- **Coverage**: 8 comprehensive test cases
- **Pattern**: Arrange-Act-Assert
- **Scope**: SSRD30 and 10% rule validation

### Error Handling (`src/error.rs`)

**Purpose**: Centralized error type definitions

```rust
pub enum ApiError {
    DatabaseError(String),           // Database issues
    ExternalAPIError(String),        // Strava API problems  
    InternalConversionError(String), // Data transformation issues
    ValidationError(String),         // Input validation
    NotFoundError(String),           // Resource not found
    AuthenticationError(String),     // Auth failures
}
```

## 🔄 Data Flow Through Modules

### Activity Sync Flow
```
Strava API → strava_client.rs → activity_controller.rs → database.rs
     ↓              ↓                   ↓                    ↓
1. External     2. Convert to      3. Business logic    4. Persistence
   API call        internal type     validation           with SQLx
```

### API Request Flow
```
HTTP Request → api/*.rs → activity_controller.rs → database.rs → Response
      ↓           ↓              ↓                    ↓           ↓
1. Route to   2. Extract     3. Business logic   4. Database   5. JSON
   handler       params        processing         query         response
```

### Error Flow
```
Source → Context → Error Type → HTTP Status → Client Response
  ↓        ↓         ↓           ↓              ↓
SQLx    Add details  ApiError   Status code   JSON error
```

## 🎨 Design Patterns Used

### 1. **Repository Pattern** (`database.rs`)
- Encapsulates data access logic
- Provides clean interface for business layer
- Enables easier testing with mock implementations

### 2. **Facade Pattern** (`activity_controller.rs`)
- Provides simplified interface to complex subsystems
- Orchestrates multiple services
- Central point for business logic

### 3. **Error Context Pattern**
```rust
.map_err(|e| ApiError::DatabaseError(format!("Failed to insert activity: {}", e)))?
```
- Add context at error boundaries
- Preserve error chain for debugging
- Convert between error types cleanly

### 4. **Builder Pattern** (`startup_utils.rs`)
- Complex object initialization
- Configuration composition
- Dependency injection setup

## 🧭 Navigation Guide

### "I want to add a new endpoint"
1. **API Handler**: Add to appropriate file in `src/api/`
2. **Business Logic**: Add method to `ActivityController`
3. **Data Access**: Add query method to `Database` if needed
4. **Models**: Define request/response types in `src/models/`
5. **Tests**: Add integration tests
6. **Documentation**: Update `API_DOCUMENTATION.md`

### "I want to add new business logic"
1. **Start**: `src/services/activity_controller.rs`
2. **Data Models**: Define in `src/models/`
3. **Database**: Add queries in `src/services/database.rs`
4. **Tests**: Add to `src/tests/`
5. **Integration**: Wire up in API layer

### "I want to add a new data model"
1. **Define**: Create struct in appropriate `src/models/*.rs`
2. **Serialization**: Add Serde derives
3. **Database**: Add table/queries if needed
4. **Usage**: Import in relevant services
5. **Tests**: Add model validation tests

### "I want to add external integration"
1. **Client**: Create new file in `src/services/`
2. **Models**: Define response types in `src/models/`
3. **Error Handling**: Map external errors to `ApiError`
4. **Configuration**: Add config in `startup_utils.rs`
5. **Tests**: Mock external dependencies

## 🚨 Important Conventions

### Naming Conventions
- **Files**: `snake_case.rs`
- **Structs**: `PascalCase`
- **Functions**: `snake_case`
- **Constants**: `SCREAMING_SNAKE_CASE`
- **Modules**: `snake_case`

### Import Organization
```rust
// Standard library
use std::collections::HashMap;

// External crates
use serde::{Deserialize, Serialize};
use sqlx::Row;

// Internal modules
use crate::models::bullshark::BullSharkActivity;
use crate::error::ApiError;
```

### Error Handling Convention
- Use `?` operator for error propagation
- Add context at module boundaries
- Convert external errors to `ApiError`
- Log errors at the source

### Testing Convention
- Tests in `src/tests/` directory (not inline)
- One test module per domain
- Comprehensive coverage of business logic
- Use descriptive test names

## 🔍 Code Quality Guidelines

### Function Size
- **Ideal**: 10-20 lines
- **Maximum**: 50 lines
- **Rule**: If longer, consider breaking into smaller functions

### Module Size
- **Ideal**: 200-300 lines
- **Maximum**: 500 lines
- **Rule**: If larger, consider splitting responsibilities

### Dependency Rules
- **API layer**: Can depend on services and models
- **Service layer**: Can depend on other services and models
- **Model layer**: No dependencies on other layers
- **Utilities**: Can be used by any layer

### Documentation Requirements
- **Public APIs**: Always document with `///`
- **Complex logic**: Add inline comments explaining "why"
- **Modules**: Add module-level documentation
- **Examples**: Include usage examples for complex functions

This organization supports maintainable, testable code that's easy for both humans and AI agents to understand and extend.
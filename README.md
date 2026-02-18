# BullSharks.online Server

A high-performance REST API server that aggregates and serves Strava activities for the BullSharks running club. The server automatically syncs activities from the Strava Club API and provides endpoints for retrieving activity data, team statistics, and athlete information.

**Live API:** https://bullsharks-server-288102886042.us-central1.run.app

## Build Status

![CI Pipeline](https://github.com/bullsharks/bullsharks.online/workflows/CI%20Pipeline/badge.svg)
![Security Scans](https://github.com/bullsharks/bullsharks.online/workflows/Security%20Scans/badge.svg)
[![codecov](https://codecov.io/gh/bullsharks/bullsharks.online/branch/main/graph/badge.svg)](https://codecov.io/gh/bullsharks/bullsharks.online)

> **Note:** Replace `bullsharks/bullsharks.online` with your actual GitHub repository path in the badge URLs above.

## Technologies Used

- **[Rust](https://www.rust-lang.org/)** - Systems programming language for performance and safety
- **[Axum](https://github.com/tokio-rs/axum)** - Web framework built on Tokio
- **[Tokio](https://tokio.rs/)** - Asynchronous runtime for Rust
- **[SQLx](https://github.com/launchbadge/sqlx)** - Async SQL toolkit with compile-time query verification
- **[PostgreSQL](https://www.postgresql.org/)** - Primary database (hosted on Supabase)
- **[Google Cloud Run](https://cloud.google.com/run)** - Serverless deployment platform
- **[Google Cloud Scheduler](https://cloud.google.com/scheduler)** - Automated activity sync every 2 minutes
- **[Strava API](https://developers.strava.com/)** - Activity data source

### Key Dependencies

- `serde` & `serde_json` - JSON serialization/deserialization
- `reqwest` - HTTP client for Strava API
- `chrono` & `chrono-tz` - Timezone-aware datetime handling
- `tokio-cron-scheduler` - Scheduled task management
- `dashmap` - Concurrent HashMap for caching
- `dotenvy` - Environment variable management

## 📚 Documentation

### For Contributors & AI Agents
- **[🏠 Documentation Hub](/docs/README.md)** - Start here for complete documentation index
- **[🤖 Agent Contribution Guide](/docs/AGENT_CONTRIBUTION_GUIDE.md)** - Essential guide for AI agents
- **[🏗️ System Architecture](/docs/SYSTEM_ARCHITECTURE.md)** - High-level system design and data flow
- **[📁 Code Organization](/docs/CODE_ORGANIZATION.md)** - Module structure and responsibilities
- **[🗄️ Database Schema](/docs/DATABASE_SCHEMA.md)** - Data models and relationships
- **[⚙️ Development Workflow](/docs/DEVELOPMENT_WORKFLOW.md)** - How to make changes safely
- **[🧪 Testing Strategy](/docs/TESTING_STRATEGY.md)** - Testing approach and validation
- **[🔧 Troubleshooting](/docs/TROUBLESHOOTING.md)** - Common issues and solutions

### For Users & Operations
- **[📖 API Documentation](/docs/API_DOCUMENTATION.md)** - Complete API reference
- **[🚀 DevOps Guide](/docs/DEVOPS.md)** - Operations and deployment guide

> **💡 Living Documentation**: This documentation follows a "living documentation" approach—it's updated with every PR to reflect the actual state of the code, ensuring contributors (human and AI) always have accurate, up-to-date context.

## Quick Start

### Local Development

```bash
# Clone the repository
git clone https://github.com/yourusername/bullsharks.online.git
cd bullsharks.online

# Set up environment variables
cp .env.example .env
# Edit .env with your configuration

# Run the server
cargo run

# Server will be available at http://localhost:8080
```

### Environment Variables

Required environment variables (see `.env.example`):
- `DATABASE_URL` - PostgreSQL connection string
- `STRAVA_CLIENT_ID` - Strava OAuth client ID
- `STRAVA_CLIENT_SECRET` - Strava OAuth client secret
- `STRAVA_CLUB_ID` - Strava club ID
- `CRON_SECRET` - Secret token for populate endpoint

## API Overview

### Public Endpoints

- `GET /health` - Health check
- `GET /read` - Get all activities
- `GET /activities/week` - Get current week's activities
- `GET /activities/month` - Get current month's activities
- `GET /activities/window` - Get activities from custom time range
- `GET /team_stats` - Get Bulls vs Sharks team statistics
- `GET /athletes` - Get all registered athletes
- `GET /athletes/training_data` - Get athlete data with weekly kilometers for all athletes

See the [API Documentation](/docs/API_DOCUMENTATION.md) for detailed endpoint specifications.

## Project Structure

```
bullsharks.online/
├── src/
│   ├── api/              # API endpoint handlers
│   │   ├── activities.rs # Activity endpoints
│   │   ├── athletes.rs   # Athlete endpoints
│   │   └── health.rs     # Health check
│   ├── models/           # Data models
│   ├── services/         # Business logic
│   │   ├── database.rs   # Database operations
│   │   ├── strava_client.rs
│   │   └── auth_controller.rs
│   ├── utils/            # Utilities
│   └── main.rs           # Application entry point
├── docs/                 # Documentation
├── Dockerfile            # Container definition
├── Cargo.toml           # Rust dependencies
└── README.md            # This file
```

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is maintained by the BullSharks running club.

# BullSharks.online Server

A high-performance REST API server that aggregates and serves Strava activities for the BullSharks running club. The server automatically syncs activities from the Strava Club API and provides endpoints for retrieving activity data, team statistics, and athlete information.

**Live API:** https://bullsharks-server-288102886042.us-central1.run.app

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

## Documentation

### 📚 [API Documentation](/docs/API_DOCUMENTATION.md)

Complete API reference for external clients:
- Endpoint specifications
- Request/response formats
- Data models and schemas
- Code examples (JavaScript, Python, cURL, React)
- Error handling

### 🚀 [DevOps Documentation](/docs/DEVOPS.md)

Operations guide for managing the deployed service:
- Architecture overview
- Deployment procedures
- Monitoring and health checks
- Debugging and troubleshooting
- Cloud Run and Cloud Scheduler management

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

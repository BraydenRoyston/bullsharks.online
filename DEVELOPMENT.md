# 🚧 Development Guide

*Development workflow and tooling for BullSharks.online*

## 🏁 Quick Start

```bash
# 1. Clone and setup
git clone https://github.com/BraydenRoyston/bullsharks.online.git
cd bullsharks.online

# 2. Environment setup
cp .env.example .env
# Edit .env with your configuration

# 3. Install Rust toolchain (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 4. Run locally
cargo run

# Server available at: http://localhost:8080
```

## 🛠️ Development Commands

### Basic Development:
```bash
# Run the server (with auto-reload on file changes)
cargo watch -x run

# Run tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Check code without building
cargo check

# Format code
cargo fmt

# Lint code
cargo clippy

# Build for production
cargo build --release
```

### Database Operations:
```bash
# Check database connection
cargo run --bin check-db  # (if implemented)

# Run database migrations (via external tool)
# Note: Migrations handled via Supabase dashboard
```

### API Testing:
```bash
# Health check
curl http://localhost:8080/health

# Get all activities
curl http://localhost:8080/read

# Get team stats
curl http://localhost:8080/team_stats
```

## 🧪 Testing Strategy

### Unit Tests:
```bash
# Run all tests
cargo test

# Run tests for specific module
cargo test activity_controller

# Run tests with pattern matching
cargo test test_ssrd30

# Run tests with detailed output
cargo test -- --nocapture --test-threads=1
```

### Integration Tests:
```bash
# Test specific endpoints (requires server running)
curl -X GET http://localhost:8080/health
curl -X GET http://localhost:8080/activities/week
```

### Load Testing:
```bash
# Install hey (HTTP load testing tool)
brew install hey  # macOS
# or apt install hey  # Ubuntu

# Load test health endpoint
hey -n 1000 -c 10 http://localhost:8080/health
```

## 📁 Project Structure Explained

```
bullsharks.online/
├── src/
│   ├── main.rs              # 🚪 Entry point, server setup, routing
│   ├── error.rs             # 🚨 Custom error types & handling
│   │
│   ├── api/                 # 🌐 HTTP handlers (thin layer)
│   │   ├── mod.rs           # Module exports
│   │   ├── activities.rs    # Activity endpoints (/activities/*)
│   │   ├── athletes.rs      # Athlete endpoints (/athletes/*)
│   │   └── health.rs        # Health check (/health)
│   │
│   ├── services/            # 🧠 Business logic (thick layer)
│   │   ├── mod.rs           # Service exports
│   │   ├── activity_controller.rs  # 🏃‍♂️ Core algorithms, data processing
│   │   ├── strava_client.rs # 🌐 External API integration
│   │   ├── auth_controller.rs # 🔐 OAuth & authentication
│   │   └── database.rs      # 🗄️ Database connection & pool management
│   │
│   ├── models/              # 📊 Data structures
│   │   ├── mod.rs           # Model exports
│   │   ├── bullshark.rs     # Activity model (from Strava)
│   │   ├── athlete.rs       # Athlete information
│   │   ├── team_stats.rs    # Team statistics
│   │   ├── injury_risk.rs   # Risk assessment types
│   │   └── oauth.rs         # Authentication tokens
│   │
│   └── utils/               # 🔧 Utilities & helpers
│       ├── mod.rs           # Utility exports
│       ├── database_utils.rs # Database query helpers
│       └── startup_utils.rs # Application startup logic
│
├── docs/                    # 📚 Documentation
│   ├── API_DOCUMENTATION.md # Complete API reference
│   └── DEVOPS.md           # Deployment & operations guide
│
├── AGENT_GUIDE.md          # 🤖 AI agent development guide
├── DEVELOPMENT.md          # 🚧 This file - human development guide
├── README.md               # 📖 Project overview
├── Cargo.toml              # 📦 Dependencies & metadata
├── Dockerfile              # 🐳 Container definition
└── .env.example            # ⚙️ Environment configuration template
```

## 🎯 Common Development Tasks

### Adding a New Endpoint:
1. **Define route** in `src/main.rs`
2. **Create handler** in `src/api/[domain].rs`
3. **Implement service logic** in `src/services/[domain]_controller.rs`
4. **Add models** if needed in `src/models/`
5. **Write tests** in the same files
6. **Update API docs** in `docs/API_DOCUMENTATION.md`

### Modifying Business Logic:
1. **Focus on services layer** - `src/services/`
2. **Write tests first** - Test-driven development
3. **Keep handlers thin** - Move logic to services
4. **Update error handling** - Use custom error types

### Database Schema Changes:
1. **Coordinate with human** - Schema changes need approval
2. **Update models** - Reflect changes in `src/models/`
3. **Test migrations** - Validate data integrity
4. **Update documentation** - Keep API docs current

## 🐛 Debugging Guide

### Common Issues:

**"Connection refused" errors:**
```bash
# Check if PostgreSQL is running
pg_isready -h localhost -p 5432

# Check environment variables
echo $DATABASE_URL
```

**"Permission denied" for cron endpoints:**
```bash
# Check CRON_SECRET is set correctly
curl -H "Authorization: Bearer $CRON_SECRET" http://localhost:8080/populate
```

**Compilation errors:**
```bash
# Clean build artifacts
cargo clean

# Update dependencies
cargo update

# Check for syntax issues
cargo check
```

### Logging & Monitoring:
```bash
# Run with debug logging
RUST_LOG=debug cargo run

# Run with specific module logging
RUST_LOG=bullsharks_server=debug,sqlx=info cargo run

# Monitor logs in production (Cloud Run)
gcloud logs tail --follow --project=your-project-id
```

## 🚀 Deployment Checklist

### Before Deploying:
- [ ] All tests pass: `cargo test`
- [ ] Code is formatted: `cargo fmt --check`
- [ ] No clippy warnings: `cargo clippy`
- [ ] Environment variables updated
- [ ] API documentation is current
- [ ] Performance tested locally

### Production Deployment:
- Handled via Google Cloud Run
- See `docs/DEVOPS.md` for detailed procedures
- Always coordinate with human for production changes

## 💡 Pro Tips

### For AI Agents:
- **Start with tests** - Understand expected behavior first
- **Read service layer** - Business logic lives in `src/services/`
- **Use targeted reads** - Read files with `--limit` to manage context
- **Check recent commits** - `git log --oneline -10` for context

### For Humans:
- **Use `cargo watch`** - Auto-reload on file changes
- **Write tests first** - Test-driven development
- **Keep handlers thin** - Move logic to services layer
- **Document decisions** - Update relevant .md files

---

*Happy coding! 🦀*
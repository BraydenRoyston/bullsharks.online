# BullSharks.online Development Commands
# Install just: cargo install just
# Usage: just <command>

# Default command - show available recipes
default:
    @just --list

# Development commands
dev:
    cargo watch -x run

run:
    cargo run

# Testing commands  
test:
    cargo test

test-verbose:
    cargo test -- --nocapture

test-specific pattern:
    cargo test {{pattern}}

# Code quality commands
check:
    cargo check

fmt:
    cargo fmt

fmt-check:
    cargo fmt --check

clippy:
    cargo clippy

clippy-fix:
    cargo clippy --fix

# Build commands
build:
    cargo build

build-release:
    cargo build --release

clean:
    cargo clean

# API testing commands
health:
    curl -s http://localhost:8080/health | jq

activities:
    curl -s http://localhost:8080/read | jq

team-stats:
    curl -s http://localhost:8080/team_stats | jq

# Database commands
db-check:
    @echo "Checking database connection..."
    psql $DATABASE_URL -c "SELECT 1;" > /dev/null && echo "✅ Database connected" || echo "❌ Database connection failed"

# Load testing
load-test endpoint="health":
    hey -n 1000 -c 10 http://localhost:8080/{{endpoint}}

# Development setup
setup:
    @echo "Setting up development environment..."
    cp .env.example .env
    @echo "📝 Edit .env with your configuration"
    @echo "🏃 Then run: just dev"

# Git helpers
commit message:
    git add .
    git commit -m "{{message}}"

push:
    git push origin $(git branch --show-current)

# Deployment helpers (production - coordinate with human first)
docker-build:
    docker build -t bullsharks-server .

docker-run:
    docker run -p 8080:8080 --env-file .env bullsharks-server

# Documentation
docs:
    @echo "📚 Opening documentation..."
    @echo "API Docs: docs/API_DOCUMENTATION.md"
    @echo "DevOps: docs/DEVOPS.md"
    @echo "Agent Guide: AGENT_GUIDE.md"
    @echo "Development: DEVELOPMENT.md"
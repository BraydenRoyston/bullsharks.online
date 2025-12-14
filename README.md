# Bullsharks Online

A web application for tracking and displaying BullSharks club activities from Strava, featuring a leaderboard and activity tracking functionality.

## Tech Stack

- **Client**: React, TypeScript, Vite, Supabase
- **Server**: Rust, Axum, PostgreSQL, Tokio
- **External APIs**: Strava API, Supabase

## Prerequisites

Before you begin, ensure you have the following installed:

- **Node.js** (v18 or higher) and npm
- **Rust** (latest stable version)
- **PostgreSQL** database (or Supabase account)
- **Strava API** credentials (Client ID and Client Secret)
- **Supabase** account and project

## Project Structure

```
bullsharks.online/
├── client/          # React frontend application
├── server/          # Rust backend API server
└── README.md        # This file
```

## Setup Instructions

### 1. Client Setup

#### Install Dependencies

```bash
cd client
npm install
```

#### Configure Environment Variables

Create a `.env` file in the `client/` directory:

```bash
touch client/.env
```

Add the following environment variables to `client/.env`:

```env
SERVER_URL=http://localhost:3000
VITE_SUPABASE_URL=your_supabase_project_url
VITE_SUPABASE_ANON_KEY=your_supabase_anon_key
VITE_STRAVA_CLIENT_ID=your_strava_client_id
```

Replace the placeholder values with your actual credentials:
- Get Supabase credentials from your [Supabase dashboard](https://app.supabase.com)
- Get Strava Client ID from your [Strava API application](https://www.strava.com/settings/api)

#### Run the Client

Development mode:
```bash
npm run dev
```

The client will start on `http://localhost:5173` (default Vite port) and proxy API requests to the backend server.

#### Build for Production

```bash
npm run build
```

Preview production build:
```bash
npm run preview
```

### 2. Server Setup

#### Install Dependencies

The Rust dependencies will be automatically installed when you build the project.

```bash
cd server
```

#### Configure Environment Variables

Create a `.env` file in the `server/` directory:

```bash
touch server/.env
```

Add the following environment variables to `server/.env`:

```env
STRAVA_CLIENT_ID=your_strava_client_id
STRAVA_CLIENT_SECRET=your_strava_client_secret
STRAVA_CLUB_ID=your_strava_club_id
DATABASE_URL=postgresql://user:password@host:port/database
```

Replace the placeholder values:
- `STRAVA_CLIENT_ID`: From your Strava API application
- `STRAVA_CLIENT_SECRET`: From your Strava API application
- `STRAVA_CLUB_ID`: The ID of your Strava club
- `DATABASE_URL`: Your PostgreSQL connection string (can use Supabase pooler URL)

#### Database Setup

Ensure your PostgreSQL database is set up with the required tables. The server expects the following tables:
- `strava_auth_tokens`: Stores OAuth tokens for Strava API access
- `bullshark_activities`: Stores activity data from Strava

#### Run the Server

Development mode (with auto-reload):
```bash
cargo watch -x run
```

Or build and run:
```bash
cargo build
cargo run
```

Production build:
```bash
cargo build --release
./target/release/server
```

The server will start on `http://localhost:3000`.

## Environment Variables Reference

### Client Environment Variables

| Variable | Description | Required |
|----------|-------------|----------|
| `SERVER_URL` | Backend API server URL | Yes |
| `VITE_SUPABASE_URL` | Supabase project URL | Yes |
| `VITE_SUPABASE_ANON_KEY` | Supabase anonymous/public key | Yes |
| `VITE_STRAVA_CLIENT_ID` | Strava OAuth Client ID | Yes |

### Server Environment Variables

| Variable | Description | Required |
|----------|-------------|----------|
| `STRAVA_CLIENT_ID` | Strava OAuth Client ID | Yes |
| `STRAVA_CLIENT_SECRET` | Strava OAuth Client Secret | Yes |
| `STRAVA_CLUB_ID` | Target Strava club ID | Yes |
| `DATABASE_URL` | PostgreSQL connection string | Yes |

## Getting API Credentials

### Strava API

1. Go to [Strava API Settings](https://www.strava.com/settings/api)
2. Create a new application
3. Note your Client ID and Client Secret
4. Set the Authorization Callback Domain appropriately

### Supabase

1. Create a project at [Supabase](https://app.supabase.com)
2. Go to Project Settings > API
3. Copy your Project URL and anon/public key
4. Get your database connection string from Project Settings > Database

## Development Workflow

1. Start the backend server first:
   ```bash
   cd server && cargo run
   ```

2. In a new terminal, start the frontend:
   ```bash
   cd client && npm run dev
   ```

3. Access the application at `http://localhost:5173`

## Security Notes

- Never commit `.env` files to version control (they are gitignored)
- Rotate your API keys regularly
- Use environment-specific credentials (development vs production)
- The Supabase anon key is safe to use in the client as it's designed for public access

## Troubleshooting

### Client Issues

- **Port already in use**: Vite will automatically try the next available port
- **API requests failing**: Ensure the server is running on port 3000
- **Build errors**: Clear node_modules and reinstall: `rm -rf node_modules package-lock.json && npm install`

### Server Issues

- **Database connection errors**: Verify your `DATABASE_URL` is correct and the database is accessible
- **Strava API errors**: Check that your Client ID and Secret are correct
- **Build errors**: Run `cargo clean` and rebuild

## License

[Add your license here]

## Contributing

[Add contribution guidelines here]

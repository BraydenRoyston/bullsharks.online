# BullSharks API Documentation

Welcome to the BullSharks Strava Activity API. This API provides access to Strava activities and team statistics for the BullSharks running club.

**Base URL:** `https://bullsharks-server-288102886042.us-central1.run.app`

**Version:** 1.0

---

## Table of Contents

- [Authentication](#authentication)
- [Rate Limiting](#rate-limiting)
- [Endpoints](#endpoints)
  - [Health Check](#health-check)
  - [Get All Activities](#get-all-activities)
  - [Get Activities from This Week](#get-activities-from-this-week)
  - [Get Activities from This Month](#get-activities-from-this-month)
  - [Get Activities from Custom Time Window](#get-activities-from-custom-time-window)
  - [Get Team Statistics](#get-team-statistics)
  - [Get All Athletes](#get-all-athletes)
- [Data Models](#data-models)
- [Error Handling](#error-handling)
- [Examples](#examples)

---

## Authentication

Most endpoints are **publicly accessible** and do not require authentication. The `/populate` endpoint requires a secret token and is intended for internal use only.

---

## Rate Limiting

This API is deployed on Google Cloud Run with automatic scaling. Please be mindful of request frequency to avoid unnecessary costs. We recommend:

- Caching responses when possible
- Limiting requests to once per minute for polling scenarios
- Using appropriate time windows to reduce payload sizes

---

## Endpoints

### Health Check

Check the health status of the API and its dependencies.

**Endpoint:** `GET /health`

**Response:**

```json
{
  "database": "healthy",
  "strava": "healthy",
  "overall": "healthy"
}
```

**Status Codes:**
- `200 OK` - Service is operational

**Example:**
```bash
curl https://bullsharks-server-288102886042.us-central1.run.app/health
```

---

### Get All Activities

Retrieve all Strava activities stored in the database.

**Endpoint:** `GET /read`

**Response:** Array of [Activity](#activity) objects

**Status Codes:**
- `200 OK` - Success
- `500 Internal Server Error` - Database error

**Example:**
```bash
curl https://bullsharks-server-288102886042.us-central1.run.app/read
```

**Response Example:**
```json
[
  {
    "id": "10594295123",
    "date": "2024-12-26T14:30:00-08:00",
    "athlete_name": "John Doe",
    "resource_state": 2,
    "name": "Morning Run",
    "distance": 8046.72,
    "moving_time": 2400,
    "elapsed_time": 2520,
    "total_elevation_gain": 45.2,
    "sport_type": "Run",
    "workout_type": 0,
    "device_name": "Garmin Forerunner 245"
  }
]
```

---

### Get Activities from This Week

Retrieve activities from the current week (Monday 00:00:00 to Sunday 23:59:59 Pacific Time).

**Endpoint:** `GET /activities/week`

**Time Zone:** Pacific Time (America/Los_Angeles)

**Week Definition:** Monday to Sunday

**Response:** Array of [Activity](#activity) objects

**Status Codes:**
- `200 OK` - Success
- `500 Internal Server Error` - Database or conversion error

**Example:**
```bash
curl https://bullsharks-server-288102886042.us-central1.run.app/activities/week
```

---

### Get Activities from This Month

Retrieve activities from the current calendar month (1st day at 00:00:00 to last day at 23:59:59 Pacific Time).

**Endpoint:** `GET /activities/month`

**Time Zone:** Pacific Time (America/Los_Angeles)

**Response:** Array of [Activity](#activity) objects

**Status Codes:**
- `200 OK` - Success
- `500 Internal Server Error` - Database or conversion error

**Example:**
```bash
curl https://bullsharks-server-288102886042.us-central1.run.app/activities/month
```

---

### Get Activities from Custom Time Window

Retrieve activities from a custom date/time range.

**Endpoint:** `GET /activities/window`

**Query Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `start` | string | Yes | Start datetime in RFC3339 format (UTC) |
| `end` | string | Yes | End datetime in RFC3339 format (UTC) |

**Response:** Array of [Activity](#activity) objects

**Status Codes:**
- `200 OK` - Success
- `400 Bad Request` - Invalid datetime format
- `500 Internal Server Error` - Database error

**Example:**
```bash
curl "https://bullsharks-server-288102886042.us-central1.run.app/activities/window?start=2024-12-01T00:00:00Z&end=2024-12-31T23:59:59Z"
```

**Notes:**
- Datetime strings must be in RFC3339 format (e.g., `2024-12-01T00:00:00Z`)
- Times are interpreted as UTC
- URL encode the query parameters if using special characters

---

### Get Team Statistics

Retrieve aggregated team statistics for Bulls and Sharks teams, including per-athlete mileage and weekly breakdowns.

**Endpoint:** `GET /team_stats`

**Response:** [TeamStats](#teamstats) object

**Status Codes:**
- `200 OK` - Success
- `500 Internal Server Error` - Database error

**Example:**
```bash
curl https://bullsharks-server-288102886042.us-central1.run.app/team_stats
```

**Response Example:**
```json
{
  "bulls": {
    "athleteKilometers": {
      "John Doe": 42.5,
      "Jane Smith": 38.2
    },
    "weeklyKilometers": [
      {
        "weekStart": "2024-12-16T00:00:00-08:00",
        "weeklyTeamKilometers": 80.7,
        "weeklyRunningSum": 80.7,
        "weeklyAthleteKilometers": {
          "John Doe": 42.5,
          "Jane Smith": 38.2
        }
      }
    ]
  },
  "sharks": {
    "athleteKilometers": {
      "Bob Johnson": 51.3,
      "Alice Brown": 45.8
    },
    "weeklyKilometers": [
      {
        "weekStart": "2024-12-16T00:00:00-08:00",
        "weeklyTeamKilometers": 97.1,
        "weeklyRunningSum": 97.1,
        "weeklyAthleteKilometers": {
          "Bob Johnson": 51.3,
          "Alice Brown": 45.8
        }
      }
    ]
  }
}
```

---

### Get All Athletes

Retrieve information about all registered athletes.

**Endpoint:** `GET /athletes`

**Response:** Array of [Athlete](#athlete) objects

**Status Codes:**
- `200 OK` - Success
- `500 Internal Server Error` - Database error

**Example:**
```bash
curl https://bullsharks-server-288102886042.us-central1.run.app/athletes
```

**Response Example:**
```json
[
  {
    "id": "12345678",
    "name": "John Doe",
    "team": "Bulls",
    "event": "Marathon"
  },
  {
    "id": "87654321",
    "name": "Jane Smith",
    "team": "Sharks",
    "event": "Half Marathon"
  }
]
```

---

## Data Models

### Activity

Represents a single Strava activity.

```typescript
{
  id: string;                      // Unique Strava activity ID
  date: string;                    // Activity date/time (ISO 8601 with timezone)
  athlete_name: string | null;     // Name of the athlete
  resource_state: number | null;   // Strava resource state (1=meta, 2=summary, 3=detail)
  name: string | null;             // Activity title
  distance: number | null;         // Distance in meters
  moving_time: number | null;      // Moving time in seconds
  elapsed_time: number | null;     // Total elapsed time in seconds
  total_elevation_gain: number | null;  // Elevation gain in meters
  sport_type: string | null;       // Type of sport (Run, Ride, Swim, etc.)
  workout_type: number | null;     // Workout type code (0=default, 1=race, 2=long run, 3=workout)
  device_name: string | null;      // Name of the recording device
}
```

**Field Details:**

- **distance**: Measured in meters. Divide by 1000 for kilometers, or by 1609.34 for miles.
- **moving_time**: Total time in motion (excludes stopped time)
- **elapsed_time**: Total time from start to finish (includes stopped time)
- **total_elevation_gain**: Cumulative elevation gain in meters
- **sport_type**: Common values include "Run", "Ride", "Swim", "Walk", "Hike", "VirtualRun"
- **workout_type**:
  - `0` - Default run
  - `1` - Race
  - `2` - Long run
  - `3` - Workout/intervals

---

### Athlete

Represents an athlete in the BullSharks club.

```typescript
{
  id: string;      // Unique Strava athlete ID
  name: string;    // Athlete's full name
  team: string;    // Team assignment ("Bulls" or "Sharks")
  event: string;   // Registered event type
}
```

---

### TeamStats

Aggregated statistics for both teams.

```typescript
{
  bulls: TeamData;
  sharks: TeamData;
}
```

---

### TeamData

Statistics for a single team.

```typescript
{
  athleteKilometers: {
    [athleteName: string]: number;  // Total kilometers per athlete
  };
  weeklyKilometers: WeekData[];     // Array of weekly statistics
}
```

---

### WeekData

Weekly statistics for a team.

```typescript
{
  weekStart: string;                    // Week start date (ISO 8601 with timezone)
  weeklyTeamKilometers: number;         // Total team kilometers for the week
  weeklyRunningSum: number;             // Cumulative sum across all weeks
  weeklyAthleteKilometers: {
    [athleteName: string]: number;      // Kilometers per athlete for the week
  };
}
```

**Notes:**
- All distance values are in **kilometers**
- `weekStart` represents Monday 00:00:00 Pacific Time
- `weeklyRunningSum` provides a cumulative total useful for tracking progress over time

---

### HealthStatus

Health check response.

```typescript
{
  database: string;    // Database health ("healthy" or "unhealthy: [error]")
  strava: string;      // Strava API health ("healthy" or "unhealthy: [error]")
  overall: string;     // Overall health ("healthy" or "unhealthy")
}
```

---

## Error Handling

The API uses standard HTTP status codes to indicate success or failure.

### Success Codes

| Code | Description |
|------|-------------|
| `200 OK` | Request successful |

### Error Codes

| Code | Description |
|------|-------------|
| `400 Bad Request` | Invalid request parameters (e.g., malformed datetime) |
| `401 Unauthorized` | Missing or invalid authentication token (internal endpoints only) |
| `500 Internal Server Error` | Server-side error (database, API, or conversion errors) |

### Error Response Format

```json
{
  "error": "Error description here"
}
```

**Common Errors:**

**400 Bad Request Example:**
```json
{
  "error": "Invalid start datetime format: input contains invalid characters. Expected RFC3339 format (e.g., 2024-01-01T00:00:00Z)"
}
```

**500 Internal Server Error Example:**
```json
{
  "error": "Database connection failed"
}
```

---

## Examples

### JavaScript/TypeScript (Fetch API)

```javascript
// Get all activities
async function getAllActivities() {
  const response = await fetch('https://bullsharks-server-288102886042.us-central1.run.app/read');
  const activities = await response.json();
  return activities;
}

// Get activities from this week
async function getWeeklyActivities() {
  const response = await fetch('https://bullsharks-server-288102886042.us-central1.run.app/activities/week');
  const activities = await response.json();
  return activities;
}

// Get activities from custom window
async function getActivitiesInRange(startDate, endDate) {
  const url = new URL('https://bullsharks-server-288102886042.us-central1.run.app/activities/window');
  url.searchParams.append('start', startDate);
  url.searchParams.append('end', endDate);

  const response = await fetch(url);
  const activities = await response.json();
  return activities;
}

// Get team statistics
async function getTeamStats() {
  const response = await fetch('https://bullsharks-server-288102886042.us-central1.run.app/team_stats');
  const stats = await response.json();
  return stats;
}

// Example usage
const activities = await getActivitiesInRange(
  '2024-12-01T00:00:00Z',
  '2024-12-31T23:59:59Z'
);
```

---

### Python (Requests)

```python
import requests
from datetime import datetime

BASE_URL = 'https://bullsharks-server-288102886042.us-central1.run.app'

# Get all activities
def get_all_activities():
    response = requests.get(f'{BASE_URL}/read')
    response.raise_for_status()
    return response.json()

# Get activities from this week
def get_weekly_activities():
    response = requests.get(f'{BASE_URL}/activities/week')
    response.raise_for_status()
    return response.json()

# Get activities from custom window
def get_activities_in_range(start_date, end_date):
    params = {
        'start': start_date,
        'end': end_date
    }
    response = requests.get(f'{BASE_URL}/activities/window', params=params)
    response.raise_for_status()
    return response.json()

# Get team statistics
def get_team_stats():
    response = requests.get(f'{BASE_URL}/team_stats')
    response.raise_for_status()
    return response.json()

# Get all athletes
def get_all_athletes():
    response = requests.get(f'{BASE_URL}/athletes')
    response.raise_for_status()
    return response.json()

# Example usage
activities = get_activities_in_range(
    '2024-12-01T00:00:00Z',
    '2024-12-31T23:59:59Z'
)

# Calculate total distance in kilometers
total_km = sum(a['distance'] / 1000 for a in activities if a['distance'])
print(f"Total distance: {total_km:.2f} km")
```

---

### cURL

```bash
# Health check
curl https://bullsharks-server-288102886042.us-central1.run.app/health

# Get all activities (formatted with jq)
curl https://bullsharks-server-288102886042.us-central1.run.app/read | jq

# Get this week's activities
curl https://bullsharks-server-288102886042.us-central1.run.app/activities/week | jq

# Get this month's activities
curl https://bullsharks-server-288102886042.us-central1.run.app/activities/month | jq

# Get activities from custom window
curl "https://bullsharks-server-288102886042.us-central1.run.app/activities/window?start=2024-12-01T00:00:00Z&end=2024-12-31T23:59:59Z" | jq

# Get team statistics
curl https://bullsharks-server-288102886042.us-central1.run.app/team_stats | jq

# Get all athletes
curl https://bullsharks-server-288102886042.us-central1.run.app/athletes | jq

# Filter activities by sport type (using jq)
curl https://bullsharks-server-288102886042.us-central1.run.app/read | \
  jq '[.[] | select(.sport_type == "Run")]'

# Calculate total distance for the week
curl https://bullsharks-server-288102886042.us-central1.run.app/activities/week | \
  jq '[.[] | .distance // 0] | add / 1000'
```

---

### React Example

```jsx
import { useState, useEffect } from 'react';

function BullSharksActivities() {
  const [activities, setActivities] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  useEffect(() => {
    fetch('https://bullsharks-server-288102886042.us-central1.run.app/activities/week')
      .then(response => {
        if (!response.ok) throw new Error('Failed to fetch activities');
        return response.json();
      })
      .then(data => {
        setActivities(data);
        setLoading(false);
      })
      .catch(err => {
        setError(err.message);
        setLoading(false);
      });
  }, []);

  if (loading) return <div>Loading...</div>;
  if (error) return <div>Error: {error}</div>;

  return (
    <div>
      <h1>This Week's Activities</h1>
      <ul>
        {activities.map(activity => (
          <li key={activity.id}>
            <strong>{activity.athlete_name}</strong> - {activity.name}
            <br />
            Distance: {(activity.distance / 1000).toFixed(2)} km
            <br />
            Time: {Math.floor(activity.moving_time / 60)} minutes
          </li>
        ))}
      </ul>
    </div>
  );
}
```

---

## Notes

### Data Synchronization

- Activities are automatically synced from Strava every 2 minutes via Google Cloud Scheduler
- New activities typically appear in the API within 2-4 minutes of being uploaded to Strava
- The system fetches the last 100 activities from the Strava Club API on each sync

### Time Zones

- The `/activities/week` and `/activities/month` endpoints use **Pacific Time (America/Los_Angeles)**
- The `/activities/window` endpoint expects **UTC** timestamps in RFC3339 format
- All activity `date` fields in responses include timezone offset information

### Data Freshness

- The API reflects the latest data from the PostgreSQL database
- For real-time updates, consider polling the `/activities/week` endpoint once per minute
- Use the `/health` endpoint to verify the API is operational before making data requests

### Performance Tips

1. **Use specific time windows**: Query only the date range you need using `/activities/window`
2. **Cache responses**: Store responses client-side and refresh periodically
3. **Filter client-side**: Get all activities once and filter by sport_type, athlete, etc. in your application
4. **Use team stats**: The `/team_stats` endpoint provides pre-aggregated data for common use cases

---

## Support

For questions, bug reports, or feature requests:

- **Operations Guide**: See [README.md](./README.md) for server operations and deployment
- **Source Code**: Available in this repository

---

## License

This API is provided for use by BullSharks club members and authorized applications.

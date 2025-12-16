# BullSharks Strava Server - Operations Guide

This README provides instructions for managing and monitoring your BullSharks Strava activity aggregator server deployed on Google Cloud Run.

**Service URL:** https://bullsharks-server-288102886042.us-central1.run.app

## Table of Contents
- [Architecture Overview](#architecture-overview)
- [Manual Activity Sync](#manual-activity-sync)
- [Reading Logs](#reading-logs)
- [Redeploying After Code Changes](#redeploying-after-code-changes)
- [Restarting the Server](#restarting-the-server)
- [Monitoring for Issues](#monitoring-for-issues)
- [Debugging](#debugging)
- [Troubleshooting](#troubleshooting)

---

## Architecture Overview

### How Activity Syncing Works

This server uses **Google Cloud Scheduler** to automatically sync Strava activities every hour:

1. **Cloud Scheduler** triggers the `/populate` endpoint at the top of every hour (`:00`)
2. The endpoint validates a secret token for security
3. If valid, the server fetches the last 100 activities from the Strava Club API
4. Activities are inserted into the PostgreSQL database (duplicates are skipped)
5. The server scales to zero between requests to minimize costs

**Key Benefits:**
- **Cost-effective:** Server scales to zero when idle (~$0.50-2/month vs $5-10/month always-on)
- **Reliable:** Cloud Scheduler is a managed service with automatic retries
- **Secure:** Protected by a secret token stored in Secret Manager

### Endpoints

| Method | Path | Purpose | Authentication |
|--------|------|---------|----------------|
| GET | `/health` | Health check for Cloud Run | Public |
| GET | `/read` | Fetch all stored activities | Public |
| POST | `/populate` | Manually trigger activity sync | Secret token required |

---

## Manual Activity Sync

### Trigger Sync via Cloud Scheduler
```bash
# Manually run the scheduled job
gcloud scheduler jobs run populate-activities --location=us-central1

# Check if it succeeded
gcloud run services logs read bullsharks-server \
  --region us-central1 \
  --limit 10 \
  --log-filter='textPayload=~"populate"'
```

### Trigger Sync via API Endpoint
```bash
# Get the secret token
CRON_SECRET=$(gcloud secrets versions access latest --secret=cron-secret)

# Call the populate endpoint
curl -X POST https://bullsharks-server-288102886042.us-central1.run.app/populate \
  -H "X-CloudScheduler-Token: $CRON_SECRET"

# Should return HTTP 200 (success) or 401 (unauthorized)
```

### Check Scheduler Status
```bash
# View scheduler job details
gcloud scheduler jobs describe populate-activities --location=us-central1

# View recent scheduler executions
gcloud scheduler jobs describe populate-activities \
  --location=us-central1 \
  --format='get(status.lastAttemptTime,status.code)'
```

---

## Reading Logs

### View Recent Logs (Last 50 Lines)
```bash
gcloud run services logs read bullsharks-server \
  --region us-central1 \
  --limit 50
```

### Stream Live Logs
```bash
gcloud run services logs tail bullsharks-server \
  --region us-central1
```

### Filter Logs by Severity
```bash
# Error logs only
gcloud run services logs read bullsharks-server \
  --region us-central1 \
  --log-filter="severity>=ERROR" \
  --limit 100

# Warning and above
gcloud run services logs read bullsharks-server \
  --region us-central1 \
  --log-filter="severity>=WARNING" \
  --limit 100
```

### Filter Logs by Time Range
```bash
# Logs from the last hour
gcloud run services logs read bullsharks-server \
  --region us-central1 \
  --limit 200 \
  --format="table(timestamp,severity,textPayload)"

# Logs from specific time range
gcloud run services logs read bullsharks-server \
  --region us-central1 \
  --log-filter='timestamp>="2025-12-15T00:00:00Z" AND timestamp<="2025-12-15T23:59:59Z"'
```

### Search for Specific Messages
```bash
# Check if Cloud Scheduler is triggering populates
gcloud run services logs read bullsharks-server \
  --region us-central1 \
  --log-filter='textPayload=~"Manual populate triggered"' \
  --limit 10

# Check database connection status
gcloud run services logs read bullsharks-server \
  --region us-central1 \
  --log-filter='textPayload=~"Database connected"' \
  --limit 5

# Check for errors during activity population
gcloud run services logs read bullsharks-server \
  --region us-central1 \
  --log-filter='textPayload=~"Failed to populate new activities"' \
  --limit 10

# Check if activities are being fetched
gcloud run services logs read bullsharks-server \
  --region us-central1 \
  --log-filter='textPayload=~"Found.*new activities"' \
  --limit 10
```

### View Logs in Cloud Console
Open the logs viewer in your browser:
```bash
# Get direct link to logs
echo "https://console.cloud.google.com/run/detail/us-central1/bullsharks-server/logs?project=sylvan-epoch-481219-h1"
```

Or navigate to: **Cloud Console → Cloud Run → bullsharks-server → Logs**

---

## Redeploying After Code Changes

### Option 1: Build and Deploy in One Command (Recommended)
This is the simplest approach - it builds a new Docker image and deploys it automatically:

```bash
# Navigate to server directory
cd /Users/braydenroyston/Desktop/projects/repos/bullsharks.online/server

# Build and deploy
gcloud builds submit --tag gcr.io/sylvan-epoch-481219-h1/bullsharks-server && \
gcloud run deploy bullsharks-server \
  --image gcr.io/sylvan-epoch-481219-h1/bullsharks-server:latest \
  --platform managed \
  --region us-central1 \
  --allow-unauthenticated \
  --min-instances 0 \
  --max-instances 1 \
  --memory 512Mi \
  --cpu 1 \
  --timeout 300 \
  --set-secrets="DATABASE_URL=database-url:latest,STRAVA_CLIENT_ID=strava-client-id:latest,STRAVA_CLIENT_SECRET=strava-client-secret:latest,STRAVA_CLUB_ID=strava-club-id:latest" \
  --port 8080
```

### Option 2: Two-Step Process
If you want more control over the build and deploy phases:

**Step 1: Build the Docker image**
```bash
gcloud builds submit --tag gcr.io/sylvan-epoch-481219-h1/bullsharks-server
```

**Step 2: Deploy the image**
```bash
gcloud run deploy bullsharks-server \
  --image gcr.io/sylvan-epoch-481219-h1/bullsharks-server:latest \
  --region us-central1
```

### Verify Deployment
After deploying, check the new revision is live:

```bash
# Check service status
gcloud run services describe bullsharks-server --region us-central1

# Test health endpoint
curl https://bullsharks-server-288102886042.us-central1.run.app/health

# Test activities endpoint
curl https://bullsharks-server-288102886042.us-central1.run.app/read | jq
```

### Deployment Time Estimates
- **Build phase:** 4-6 minutes (Rust compilation)
- **Deploy phase:** 1-2 minutes
- **Total:** ~5-8 minutes

### Quick Deploy Script
Create a deploy script for convenience:

**File: `deploy.sh`**
```bash
#!/bin/bash
set -e

echo "Building Docker image..."
gcloud builds submit --tag gcr.io/sylvan-epoch-481219-h1/bullsharks-server

echo "Deploying to Cloud Run..."
gcloud run deploy bullsharks-server \
  --image gcr.io/sylvan-epoch-481219-h1/bullsharks-server:latest \
  --platform managed \
  --region us-central1 \
  --allow-unauthenticated \
  --min-instances 0 \
  --max-instances 1 \
  --memory 512Mi \
  --cpu 1 \
  --timeout 300 \
  --set-secrets="DATABASE_URL=database-url:latest,STRAVA_CLIENT_ID=strava-client-id:latest,STRAVA_CLIENT_SECRET=strava-client-secret:latest,STRAVA_CLUB_ID=strava-club-id:latest" \
  --port 8080

echo "Testing deployment..."
curl https://bullsharks-server-288102886042.us-central1.run.app/health

echo "Deployment complete!"
```

Make it executable:
```bash
chmod +x deploy.sh
./deploy.sh
```

---

## Restarting the Server

Cloud Run automatically restarts containers when they crash or become unhealthy. However, you can manually trigger restarts using these methods:

### Method 1: Deploy Same Image (Soft Restart)
Forces Cloud Run to create a new revision with the same image:

```bash
gcloud run services update bullsharks-server \
  --region us-central1 \
  --update-labels=restart="$(date +%s)"
```

This triggers a new deployment without rebuilding the image.

### Method 2: Scale to Zero and Back (Hard Restart)
Forces all instances to shut down and restart on next request:

```bash
# Scale to zero
gcloud run services update bullsharks-server \
  --region us-central1 \
  --min-instances 0 \
  --max-instances 0

# Wait a few seconds
sleep 5

# Scale back up
gcloud run services update bullsharks-server \
  --region us-central1 \
  --min-instances 0 \
  --max-instances 1
```

### Method 3: Rollback to Previous Revision
If the current revision has issues, rollback to a previous working version:

```bash
# List all revisions
gcloud run revisions list \
  --service bullsharks-server \
  --region us-central1

# Rollback to specific revision (replace REVISION_NAME)
gcloud run services update-traffic bullsharks-server \
  --region us-central1 \
  --to-revisions REVISION_NAME=100
```

### Check Server Health After Restart
```bash
# Wait for service to be ready
sleep 10

# Check health
curl https://bullsharks-server-288102886042.us-central1.run.app/health

# Check logs for startup messages
gcloud run services logs read bullsharks-server \
  --region us-central1 \
  --limit 20 \
  --log-filter='textPayload=~"Database connected|Scheduler started|Server running"'
```

### When to Restart

Common scenarios requiring a restart:
- Database connection pool exhausted
- Memory leaks (unlikely with Rust, but possible)
- Stuck background tasks
- OAuth token refresh failures
- After updating environment variables/secrets

**Note:** Cloud Run automatically restarts containers that:
- Crash or exit with non-zero status
- Fail health checks
- Exceed memory limits
- Time out on requests

---

## Monitoring for Issues

### Best Practices

#### 1. Set Up Log-Based Alerts

Create alerts for critical errors:

```bash
# Create alert for database connection failures
gcloud alpha monitoring policies create \
  --notification-channels=CHANNEL_ID \
  --display-name="BullSharks - Database Connection Failed" \
  --condition-display-name="Database connection errors" \
  --condition-threshold-value=1 \
  --condition-threshold-duration=60s \
  --condition-filter='resource.type="cloud_run_revision"
    AND resource.labels.service_name="bullsharks-server"
    AND textPayload=~"could not create the database connection pool"'
```

#### 2. Monitor Key Metrics in Cloud Console

**Navigate to:** Cloud Console → Cloud Run → bullsharks-server → Metrics

**Key Metrics to Watch:**

| Metric | What to Monitor | Alert Threshold |
|--------|----------------|-----------------|
| **Request Count** | Should see spikes every hour (cron job) | < 10 requests/day = issue |
| **Request Latency** | /health should be <100ms, /read <2s | > 5s = investigate |
| **Container Instance Count** | Should scale to 0 when idle | Always >0 = memory leak? |
| **Memory Utilization** | Should stay <400MB | > 450MB = investigate |
| **CPU Utilization** | Spikes during cron, low otherwise | Sustained >50% = issue |
| **Error Rate** | Should be 0% for /health | > 1% = critical |

#### 3. Create Uptime Check

Monitor service availability:

```bash
gcloud monitoring uptime create bullsharks-health-check \
  --resource-type uptime-url \
  --host bullsharks-server-288102886042.us-central1.run.app \
  --path /health \
  --check-interval 5m \
  --timeout 10s
```

#### 4. Monitor Cloud Scheduler Execution

**Check Scheduler Status:**
```bash
# View last execution time and result
gcloud scheduler jobs describe populate-activities \
  --location=us-central1 \
  --format='table(status.lastAttemptTime,status.code,state)'

# Status code 0 = Success
# State should be ENABLED
```

**Verify Populate Endpoint is Being Called:**
```bash
# Check if populate ran in last 2 hours
gcloud run services logs read bullsharks-server \
  --region us-central1 \
  --log-filter='textPayload=~"Manual populate triggered" AND timestamp>="'$(date -u -v-2H '+%Y-%m-%dT%H:%M:%SZ')'"' \
  --limit 1

# Should return at least one entry if scheduler is working
```

Add this to a cron job on your local machine or Cloud Scheduler:

**File: `check_scheduler.sh`**
```bash
#!/bin/bash
# Check if Cloud Scheduler is triggering populates correctly

# Check last scheduler execution
LAST_ATTEMPT=$(gcloud scheduler jobs describe populate-activities \
  --location=us-central1 \
  --format='value(status.lastAttemptTime)')

STATUS_CODE=$(gcloud scheduler jobs describe populate-activities \
  --location=us-central1 \
  --format='value(status.code)')

echo "Last scheduler attempt: $LAST_ATTEMPT"
echo "Status code: $STATUS_CODE (0=success)"

# Check if populate endpoint was called in last 2 hours
LOGS=$(gcloud run services logs read bullsharks-server \
  --region us-central1 \
  --log-filter='textPayload=~"Manual populate triggered" AND timestamp>="'$(date -u -v-2H '+%Y-%m-%dT%H:%M:%SZ')'"' \
  --limit 1 \
  --format="value(textPayload)")

if [ -z "$LOGS" ]; then
  echo "WARNING: Cloud Scheduler has not triggered populate in the last 2 hours!"
  # Send notification (e.g., email, Slack, PagerDuty)
  exit 1
elif [ "$STATUS_CODE" != "0" ]; then
  echo "WARNING: Last scheduler execution failed with code: $STATUS_CODE"
  exit 1
else
  echo "Cloud Scheduler is running normally"
  exit 0
fi
```

#### 5. Watch for Common Issues

**Database Connection Failures:**
```bash
gcloud run services logs read bullsharks-server \
  --region us-central1 \
  --log-filter='textPayload=~"could not create the database connection pool|Failed to populate new activities"' \
  --limit 10
```

**OAuth Token Issues:**
```bash
gcloud run services logs read bullsharks-server \
  --region us-central1 \
  --log-filter='textPayload=~"Failed to refresh token|Strava API"' \
  --limit 10
```

**Scheduler Failures:**
```bash
gcloud run services logs read bullsharks-server \
  --region us-central1 \
  --log-filter='textPayload=~"Failed to create scheduler|Failed to start scheduler"' \
  --limit 10
```

### Suggested Monitoring Dashboard

Create a custom dashboard in Cloud Console with these widgets:

1. **Request Count (Last 24h)** - Line chart
2. **Request Latency p50/p95/p99** - Line chart
3. **Container Instances** - Stacked area chart
4. **Memory Usage** - Line chart
5. **Error Rate** - Line chart
6. **Log-based Metric: Cron Executions** - Counter

Access dashboards at: **Cloud Console → Monitoring → Dashboards**

### Email Notifications

Set up email notifications for critical issues:

```bash
# Create notification channel
gcloud alpha monitoring channels create \
  --display-name="BullSharks Email Alerts" \
  --type=email \
  --channel-labels=email_address=your-email@example.com
```

### Third-Party Monitoring (Optional)

For more advanced monitoring, consider integrating:

- **Sentry** - Error tracking and performance monitoring
- **Datadog** - Full observability platform
- **New Relic** - Application performance monitoring
- **Better Uptime** - Uptime monitoring with status pages
- **PagerDuty** - Incident management and on-call alerts

### Health Check Script

Run this script periodically to verify everything is working:

**File: `health_check.sh`**
```bash
#!/bin/bash
set -e

echo "=== BullSharks Server Health Check ==="
echo ""

# Test health endpoint
echo "1. Testing health endpoint..."
HEALTH=$(curl -s https://bullsharks-server-288102886042.us-central1.run.app/health)
if [ "$HEALTH" = "OK" ]; then
  echo "   ✓ Health endpoint: OK"
else
  echo "   ✗ Health endpoint: FAILED"
  exit 1
fi

# Test activities endpoint
echo "2. Testing activities endpoint..."
ACTIVITIES=$(curl -s https://bullsharks-server-288102886042.us-central1.run.app/read)
if echo "$ACTIVITIES" | jq -e '. | length > 0' > /dev/null 2>&1; then
  COUNT=$(echo "$ACTIVITIES" | jq '. | length')
  echo "   ✓ Activities endpoint: OK ($COUNT activities)"
else
  echo "   ✗ Activities endpoint: FAILED"
  exit 1
fi

# Check recent cron execution
echo "3. Checking recent cron job execution..."
CRON_LOGS=$(gcloud run services logs read bullsharks-server \
  --region us-central1 \
  --log-filter='textPayload=~"Running job to populate new activities" AND timestamp>="'$(date -u -v-2H '+%Y-%m-%dT%H:%M:%SZ')'"' \
  --limit 1 \
  --format="value(timestamp)")

if [ -n "$CRON_LOGS" ]; then
  echo "   ✓ Cron job executed: $CRON_LOGS"
else
  echo "   ⚠ Cron job not executed in last 2 hours (might be normal if server just started)"
fi

# Check for recent errors
echo "4. Checking for recent errors..."
ERROR_COUNT=$(gcloud run services logs read bullsharks-server \
  --region us-central1 \
  --log-filter='severity>=ERROR AND timestamp>="'$(date -u -v-1H '+%Y-%m-%dT%H:%M:%SZ')'"' \
  --limit 100 \
  --format="value(textPayload)" | wc -l | tr -d ' ')

if [ "$ERROR_COUNT" -eq 0 ]; then
  echo "   ✓ No errors in last hour"
else
  echo "   ⚠ Found $ERROR_COUNT errors in last hour"
  gcloud run services logs read bullsharks-server \
    --region us-central1 \
    --log-filter='severity>=ERROR AND timestamp>="'$(date -u -v-1H '+%Y-%m-%dT%H:%M:%SZ')'"' \
    --limit 5
fi

echo ""
echo "=== Health Check Complete ==="
```

Make it executable:
```bash
chmod +x health_check.sh
./health_check.sh
```

---

## Debugging

This section provides step-by-step debugging instructions for common scenarios.

### Debugging Cloud Scheduler

#### Check if Scheduler is Enabled and Configured Correctly

```bash
# View scheduler job configuration
gcloud scheduler jobs describe populate-activities --location=us-central1

# Check schedule (should be "0 * * * *" for hourly)
gcloud scheduler jobs describe populate-activities \
  --location=us-central1 \
  --format='get(schedule)'

# Check headers (should include X-CloudScheduler-Token)
gcloud scheduler jobs describe populate-activities \
  --location=us-central1 \
  --format='get(httpTarget.headers)'

# Check last execution status
gcloud scheduler jobs describe populate-activities \
  --location=us-central1 \
  --format='table(status.lastAttemptTime,status.code)'
```

**Expected Output:**
- Schedule: `0 * * * *`
- Headers should include: `X-CloudScheduler-Token=<secret>`
- Status code: `0` (success) or `-1` (pending first run)

#### Verify Scheduler is Actually Running

```bash
# Check logs for scheduler-triggered populate calls
gcloud run services logs read bullsharks-server \
  --region us-central1 \
  --limit 50 \
  --log-filter='textPayload=~"Manual populate triggered"'

# Should see entries every hour, e.g.:
# 2025-12-16 06:00:00 Manual populate triggered via /populate endpoint
# 2025-12-16 07:00:00 Manual populate triggered via /populate endpoint
```

#### Debug Scheduler Authentication Issues

If you see HTTP 401 errors in logs:

```bash
# 1. Verify the CRON_SECRET matches
CRON_SECRET=$(gcloud secrets versions access latest --secret=cron-secret)
echo "CRON_SECRET: $CRON_SECRET"

# 2. Check what's configured in scheduler
gcloud scheduler jobs describe populate-activities \
  --location=us-central1 \
  --format='get(httpTarget.headers.X-CloudScheduler-Token)'

# 3. They should match! If not, update the scheduler job:
gcloud scheduler jobs update http populate-activities \
  --location=us-central1 \
  --headers="X-CloudScheduler-Token=$CRON_SECRET"
```

#### Manually Test the Populate Endpoint

```bash
# 1. Get the secret
CRON_SECRET=$(gcloud secrets versions access latest --secret=cron-secret)

# 2. Test with correct token (should return HTTP 200)
curl -v -X POST https://bullsharks-server-288102886042.us-central1.run.app/populate \
  -H "X-CloudScheduler-Token: $CRON_SECRET"

# 3. Test without token (should return HTTP 401)
curl -v -X POST https://bullsharks-server-288102886042.us-central1.run.app/populate

# 4. Check logs to see the results
gcloud run services logs read bullsharks-server \
  --region us-central1 \
  --limit 10
```

**Expected successful output:**
```
POST 200 /populate
Manual populate triggered via /populate endpoint
Populating new activities...
Found X new activities...
Populate new activities complete.
```

### Debugging Activity Sync Issues

#### Check if Activities are Being Fetched from Strava

```bash
# Look for activity fetch logs
gcloud run services logs read bullsharks-server \
  --region us-central1 \
  --limit 50 \
  --log-filter='textPayload=~"Found.*new activities"'

# Should see entries like:
# "Found 100 new activities..."
# "Found 42 new activities..."
```

#### Check if Activities are Being Inserted into Database

```bash
# Look for database insert logs
gcloud run services logs read bullsharks-server \
  --region us-central1 \
  --limit 50 \
  --log-filter='textPayload=~"Batch insert complete"'

# Should see entries like:
# "Batch insert complete. Attempted to insert 100/100 failed."
# (Note: "failed" means duplicates were skipped, which is normal)
```

#### Verify Strava OAuth Token is Working

```bash
# Check for token refresh logs
gcloud run services logs read bullsharks-server \
  --region us-central1 \
  --limit 20 \
  --log-filter='textPayload=~"token|Token"'

# Look for:
# "Using cached token for user admin" - Good!
# "Cache miss. Checking database for fresh token" - Also good
# "Failed to refresh token" - BAD! Token needs to be updated
```

If token refresh is failing:
1. The Strava OAuth refresh token may have expired or been revoked
2. You'll need to re-authenticate with Strava and update the database
3. See "Issue: Strava OAuth Token Expired" in Troubleshooting section

### Debugging Database Issues

#### Verify Database Connection

```bash
# Check for database connection logs
gcloud run services logs read bullsharks-server \
  --region us-central1 \
  --limit 20 \
  --log-filter='textPayload=~"Database connected"'

# Should see "Database connected!" when server starts
```

#### Test Database Connectivity

```bash
# 1. Get the DATABASE_URL
DATABASE_URL=$(gcloud secrets versions access latest --secret=database-url)

# 2. Test connection locally (if you have psql installed)
psql "$DATABASE_URL" -c "SELECT COUNT(*) FROM bullshark_activities;"

# 3. Check if tables exist
psql "$DATABASE_URL" -c "\dt"
# Should show: strava_auth_tokens, bullshark_activities
```

#### Check for Database Errors

```bash
# Look for database-related errors
gcloud run services logs read bullsharks-server \
  --region us-central1 \
  --limit 50 \
  --log-filter='textPayload=~"database|Database" AND severity>=ERROR'
```

### Debugging Server Startup Issues

#### Check if Server is Starting Correctly

```bash
# Look for startup sequence
gcloud run services logs read bullsharks-server \
  --region us-central1 \
  --limit 30 \
  --log-filter='textPayload=~"Connecting to database|Database connected|Server running"'

# Expected startup sequence:
# 1. "Connecting to database..."
# 2. "Database connected!"
# 3. "Server running on 8080"
```

#### Check for Startup Errors

```bash
# Look for errors during startup
gcloud run services logs read bullsharks-server \
  --region us-central1 \
  --limit 50 \
  --log-filter='severity>=ERROR'
```

#### Verify Environment Variables are Set

```bash
# Check if secrets are accessible by Cloud Run
gcloud run services describe bullsharks-server \
  --region us-central1 \
  --format='get(spec.template.spec.containers[0].env)'

# Should show secrets mounted as environment variables
```

### Debugging Cold Starts

Cloud Run scales to zero when idle. The first request after idle triggers a "cold start."

#### Identify Cold Starts

```bash
# Look for startup logs after shutdown
gcloud run services logs read bullsharks-server \
  --region us-central1 \
  --limit 100 \
  --format='table(timestamp,textPayload)' | grep -E "(Shutdown|Connecting to database)"

# Pattern:
# XX:XX:XX Shutdown signal received
# YY:YY:YY Connecting to database... <- Cold start begins here
```

**Typical Cold Start Time:** 1-3 seconds

#### Reduce Cold Start Impact

If cold starts are problematic:
```bash
# Option 1: Set min-instances to 1 (costs ~$5-10/month)
gcloud run services update bullsharks-server \
  --region us-central1 \
  --min-instances 1

# Option 2: Keep current setup (scale to zero)
# Accept cold starts as a cost-saving tradeoff
```

### Debugging Performance Issues

#### Check Request Latency

```bash
# View request latency metrics in Cloud Console
echo "https://console.cloud.google.com/run/detail/us-central1/bullsharks-server/metrics?project=sylvan-epoch-481219-h1"

# Or check logs for slow requests
gcloud run services logs read bullsharks-server \
  --region us-central1 \
  --limit 100 \
  --format='table(timestamp,httpRequest.latency,httpRequest.requestUrl)'
```

#### Check Memory and CPU Usage

```bash
# View resource usage in Cloud Console
echo "https://console.cloud.google.com/run/detail/us-central1/bullsharks-server/metrics?project=sylvan-epoch-481219-h1"

# Check current resource limits
gcloud run services describe bullsharks-server \
  --region us-central1 \
  --format='get(spec.template.spec.containers[0].resources.limits)'
```

### General Debugging Workflow

When something isn't working, follow this systematic approach:

**Step 1: Check Service Status**
```bash
gcloud run services describe bullsharks-server --region us-central1
# Look for: status.conditions[].status = "True"
```

**Step 2: Check Recent Logs**
```bash
gcloud run services logs read bullsharks-server \
  --region us-central1 \
  --limit 50
```

**Step 3: Check for Errors**
```bash
gcloud run services logs read bullsharks-server \
  --region us-central1 \
  --log-filter='severity>=ERROR' \
  --limit 20
```

**Step 4: Test Endpoints Manually**
```bash
# Health check
curl https://bullsharks-server-288102886042.us-central1.run.app/health

# Activities API
curl https://bullsharks-server-288102886042.us-central1.run.app/read | jq

# Populate endpoint (with token)
CRON_SECRET=$(gcloud secrets versions access latest --secret=cron-secret)
curl -X POST https://bullsharks-server-288102886042.us-central1.run.app/populate \
  -H "X-CloudScheduler-Token: $CRON_SECRET"
```

**Step 5: Check Dependencies**
```bash
# Database connectivity
psql "$(gcloud secrets versions access latest --secret=database-url)" -c "SELECT 1;"

# Cloud Scheduler status
gcloud scheduler jobs describe populate-activities --location=us-central1
```

**Step 6: Review Metrics**
```bash
# Open Cloud Console metrics
echo "https://console.cloud.google.com/run/detail/us-central1/bullsharks-server/metrics?project=sylvan-epoch-481219-h1"
```

---

## Troubleshooting

### Issue: Server Not Responding

**Symptoms:** Health check fails, timeout errors

**Solution:**
```bash
# Check if service is running
gcloud run services describe bullsharks-server --region us-central1

# Check recent logs for errors
gcloud run services logs read bullsharks-server \
  --region us-central1 \
  --log-filter='severity>=ERROR' \
  --limit 20

# Restart the service
gcloud run services update bullsharks-server \
  --region us-central1 \
  --update-labels=restart="$(date +%s)"
```

### Issue: Database Connection Failed

**Symptoms:** "could not create the database connection pool" in logs

**Solution:**
```bash
# Check if DATABASE_URL secret is correct
gcloud secrets versions access latest --secret=database-url

# Test database connectivity from local machine
psql "postgresql://postgres.nvuuwstlsszmtxnqrwol:sumxu3-nufdub-herteQ@aws-0-us-west-2.pooler.supabase.com:5432/postgres"

# Update the secret if needed
echo -n "new-database-url" | gcloud secrets versions add database-url --data-file=-

# Redeploy to pick up new secret
gcloud run deploy bullsharks-server \
  --image gcr.io/sylvan-epoch-481219-h1/bullsharks-server:latest \
  --region us-central1
```

### Issue: Cloud Scheduler Not Running

**Symptoms:** "Manual populate triggered" not appearing in logs hourly

**Solution:**

**Step 1: Check if Cloud Scheduler job exists and is enabled**
```bash
# List all scheduler jobs
gcloud scheduler jobs list --location=us-central1

# Check specific job status
gcloud scheduler jobs describe populate-activities \
  --location=us-central1 \
  --format='get(state,schedule)'

# State should be "ENABLED"
# Schedule should be "0 * * * *"
```

**Step 2: Check recent executions**
```bash
# Check last attempt time and status
gcloud scheduler jobs describe populate-activities \
  --location=us-central1 \
  --format='table(status.lastAttemptTime,status.code)'

# Status code meanings:
# 0 = Success
# -1 = Pending (never run yet)
# Other = Error (check logs)
```

**Step 3: Check Cloud Run logs for populate attempts**
```bash
# Look for populate endpoint calls
gcloud run services logs read bullsharks-server \
  --region us-central1 \
  --limit 50 \
  --log-filter='textPayload=~"populate"'

# Should see hourly entries like:
# "Manual populate triggered via /populate endpoint"
```

**Step 4: Manually trigger to test**
```bash
# Trigger the job manually
gcloud scheduler jobs run populate-activities --location=us-central1

# Wait a few seconds, then check logs
sleep 10
gcloud run services logs read bullsharks-server \
  --region us-central1 \
  --limit 5

# Should see "Manual populate triggered" message
```

**Step 5: Common fixes**

If you see HTTP 401 Unauthorized:
```bash
# The CRON_SECRET might be mismatched
CRON_SECRET=$(gcloud secrets versions access latest --secret=cron-secret)

# Update scheduler job with correct token
gcloud scheduler jobs update http populate-activities \
  --location=us-central1 \
  --headers="X-CloudScheduler-Token=$CRON_SECRET"
```

If job is PAUSED:
```bash
# Resume the job
gcloud scheduler jobs resume populate-activities --location=us-central1
```

If job doesn't exist:
```bash
# Recreate it
CRON_SECRET=$(gcloud secrets versions access latest --secret=cron-secret)
gcloud scheduler jobs create http populate-activities \
  --schedule="0 * * * *" \
  --uri="https://bullsharks-server-288102886042.us-central1.run.app/populate" \
  --http-method=POST \
  --headers="X-CloudScheduler-Token=$CRON_SECRET" \
  --location=us-central1 \
  --description="Populate BullSharks activities from Strava every hour" \
  --max-retry-attempts=3
```

### Issue: Strava OAuth Token Expired

**Symptoms:** "Failed to populate new activities" in logs, Strava API errors

**Solution:**
1. Get a new OAuth token from Strava
2. Update the `strava_auth_tokens` table in your database with the new refresh token
3. The server will automatically refresh the access token

```sql
-- Update the refresh token in your database
UPDATE strava_auth_tokens
SET refresh_token = 'NEW_REFRESH_TOKEN_HERE',
    updated_at = NOW()
WHERE id = 'admin';
```

### Issue: High Memory Usage

**Symptoms:** Container instances not scaling to zero, high memory utilization

**Solution:**
```bash
# Check current memory usage
gcloud run services describe bullsharks-server \
  --region us-central1 \
  --format='get(spec.template.spec.containers[0].resources.limits.memory)'

# Increase memory if needed (current: 512Mi)
gcloud run services update bullsharks-server \
  --region us-central1 \
  --memory 1Gi

# Check logs for memory-related errors
gcloud run services logs read bullsharks-server \
  --region us-central1 \
  --log-filter='textPayload=~"out of memory|OOM"' \
  --limit 20
```

### Issue: Deployment Fails

**Symptoms:** Build or deployment errors

**Solution:**
```bash
# Check build logs
gcloud builds list --limit=5

# Get specific build logs
BUILD_ID=$(gcloud builds list --limit=1 --format='value(id)')
gcloud builds log $BUILD_ID

# Common fixes:
# 1. Ensure Cargo.lock is not ignored
cat .gcloudignore | grep -v "^#"

# 2. Verify Dockerfile syntax
docker build -t test .

# 3. Check local build
cargo check
```

---

## Quick Reference

### Essential Commands

```bash
# View logs
gcloud run services logs read bullsharks-server --region us-central1 --limit 50

# Deploy changes
gcloud builds submit --tag gcr.io/sylvan-epoch-481219-h1/bullsharks-server && \
gcloud run deploy bullsharks-server --image gcr.io/sylvan-epoch-481219-h1/bullsharks-server:latest --region us-central1

# Restart server
gcloud run services update bullsharks-server --region us-central1 --update-labels=restart="$(date +%s)"

# Check health
curl https://bullsharks-server-288102886042.us-central1.run.app/health

# Test API
curl https://bullsharks-server-288102886042.us-central1.run.app/read | jq
```

### Service Information

- **Service Name:** `bullsharks-server`
- **Region:** `us-central1`
- **Project:** `sylvan-epoch-481219-h1`
- **Service URL:** https://bullsharks-server-288102886042.us-central1.run.app
- **Image:** `gcr.io/sylvan-epoch-481219-h1/bullsharks-server`

### Support

For issues or questions:
1. Check logs first: `gcloud run services logs read bullsharks-server --region us-central1 --limit 50`
2. Review this README's Troubleshooting section
3. Check Cloud Run documentation: https://cloud.google.com/run/docs
4. Review Rust/Axum documentation for application-specific issues

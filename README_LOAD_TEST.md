# Load Testing Guide

This directory contains a load testing script to measure the backend's actual RPS (requests per second) capacity.

## Prerequisites

1. **Install Node.js** (if not already installed)
2. **Install autocannon**:
   ```bash
   npm install autocannon
   ```

## Setup

1. **Create a test user** in your database:
   ```bash
   # Start your backend locally
   cargo run
   
   # In another terminal, create the test user
   curl -X POST http://localhost:3000/common/register \
     -H "Content-Type: application/json" \
     -d '{
       "user_name": "loadtest",
       "user_password": "LoadTest123!",
       "user_role": "user"
     }'
   ```

2. **Make sure your backend is running**:
   - For local testing: `cargo run`
   - For production testing: Deploy to Fly.io

## Running the Load Test

### Local Testing
```bash
node load-test.js local
```

### Production Testing
```bash
node load-test.js production
```

## What Gets Tested

The script runs 4 different load tests:

1. **Login Endpoint** (`/common/login`)
   - Tests CPU-intensive Argon2 password hashing
   - Expected: 5-20 RPS on single-core shared CPU

2. **Cached Reads** (`/card/get_all_cards`)
   - Tests Redis cache performance
   - Expected: 200-1000 RPS

3. **Database Reads** (`/card/get_card`)
   - Tests SQLite read performance
   - Expected: 100-300 RPS

4. **Health Check** (`/`)
   - Tests basic endpoint with no database/auth
   - Expected: 1000+ RPS

## Understanding the Results

The output shows:
- **Requests/sec**: How many requests the server handled per second
- **Latency (avg)**: Average response time in milliseconds
- **Latency (p99)**: 99th percentile latency (worst 1% of requests)
- **Throughput**: Data transferred per second
- **Errors**: Number of failed requests

### Example Output
```
Results:
  Requests:          1234 (123.4/sec)
  Throughput:        1.23 MB/sec
  Latency (avg):     45.67 ms
  Latency (p99):     89.12 ms
  Errors:            0
  Timeouts:          0
```

## Baseline Performance Expectations

On **Fly.io shared-1x-cpu (512MB RAM)**:

| Endpoint Type | Expected RPS | Bottleneck |
|---------------|--------------|------------|
| Login (Argon2) | 5-20 | CPU-bound hashing |
| Cached Reads | 200-1000 | Redis network + CPU |
| DB Reads | 100-300 | SQLite I/O |
| Health Check | 1000+ | Minimal processing |

## Tips for Better Results

1. **Warm up the cache**: Run the test twice, use the second run's results
2. **Check Redis**: Make sure Redis is running and connected
3. **Monitor resources**: Use `top` or Fly.io metrics to see CPU/memory usage
4. **Test at different concurrency levels**: Edit the `connections` parameter in the script

## Troubleshooting

### "autocannon is not installed"
```bash
npm install autocannon
```

### "Cannot connect to server"
- Make sure the backend is running
- Check the URL in the script matches your deployment
- Verify firewall/network settings

### Low RPS numbers
- Check if Redis is connected (check server logs)
- Verify database file permissions
- Monitor CPU/memory usage during the test
- Consider the PASETO optimization (should improve auth-heavy workloads)

## Advanced Usage

You can modify the script to:
- Test different endpoints
- Adjust concurrency levels (change `connections`)
- Change test duration (change `duration`)
- Add custom headers or request bodies

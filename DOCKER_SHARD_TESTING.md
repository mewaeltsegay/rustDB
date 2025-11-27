# Docker Shard Testing & Failure Simulation Guide

This guide shows how to use Docker Compose to test shard monitoring with three independent shard containers that can be stopped to simulate failures.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         Host Machine                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                   │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │              Docker Network: rustdb_net                  │   │
│  │                     (bridge)                             │   │
│  │                                                          │   │
│  │  ┌─────────────────┐    ┌─────────────────────────┐     │   │
│  │  │  Coordinator    │    │   Shard Containers      │     │   │
│  │  │  (Port 8000)    │◄──►│  ┌─────────────────────┤     │   │
│  │  │                 │    │  │ Shard 0 (8010)      │     │   │
│  │  │ Manages 3       │    │  │                     │     │   │
│  │  │ shards health   │    │  ├─────────────────────┤     │   │
│  │  │                 │    │  │ Shard 1 (8011)      │     │   │
│  │  │ (Health checks) │    │  │                     │     │   │
│  │  │                 │    │  ├─────────────────────┤     │   │
│  │  └─────────────────┘    │  │ Shard 2 (8012)      │     │   │
│  │                         │  │                     │     │   │
│  │                         │  └─────────────────────┘     │   │
│  │                         └─────────────────────────────┘     │   │
│  │                                                          │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                   │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │          Testing Tools (on Host)                         │   │
│  ├──────────────────────────────────────────────────────────┤   │
│  │  • docker compose stop shard_0    (stop shard 0)         │   │
│  │  • docker compose start shard_0   (restart shard 0)      │   │
│  │  • docker compose ps              (view container status)│   │
│  │  • docker compose logs -f shard_0 (view shard 0 logs)    │   │
│  │  • .\test-shards.ps1             (health monitoring)    │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                   │
└─────────────────────────────────────────────────────────────────┘
```

## Quick Start

### 1. Build and Start All Containers

```powershell
# Build Docker images and start all containers
docker compose up --build -d

# Verify all containers are running
docker compose ps

# View logs (follow mode)
docker compose logs -f
```

### 2. Monitor Shard Health

```powershell
# In a new PowerShell window, monitor health status
.\test-shards.ps1 -Command health

# Output should show all 3 shards as healthy:
# === Shard Health Status ===
# Detailed Status:
#   shard_0: healthy
#   shard_1: healthy
#   shard_2: healthy
# Summary:
#   Healthy shards:  3
#   Total shards:    3
#   Unhealthy:       0
```

### 3. Simulate Shard Failure

```powershell
# Stop shard_0 container (simulates shard going down)
docker compose stop shard_0

# Check health again (after ~30 seconds, shard_0 should be marked as down)
.\test-shards.ps1 -Command health

# Expected output:
# === Shard Health Status ===
# Detailed Status:
#   shard_0: down
#   shard_1: healthy
#   shard_2: healthy
# Summary:
#   Healthy shards:  2
#   Total shards:    3
#   Unhealthy:       1
```

### 4. Recover Shard

```powershell
# Restart the stopped shard
docker compose start shard_0

# Wait a few seconds for container to start
Start-Sleep -Seconds 3

# Check health again
.\test-shards.ps1 -Command health

# Expected output: all 3 shards healthy again
```

## Complete Test Scenarios

### Scenario 1: Single Shard Failure
```powershell
# Terminal 1: Start monitoring
.\test-shards.ps1 -Command health

# Terminal 2: Stop one shard
docker compose stop shard_0

# Observe in Terminal 1: shard_0 transitions from healthy → down
# Observe in Terminal 2: routing automatically uses shard_1 and shard_2

# Restart shard
docker compose start shard_0

# Observe recovery
```

### Scenario 2: Multiple Shard Failures
```powershell
# Stop two shards
docker compose stop shard_0 shard_1

# Monitor
.\test-shards.ps1 -Command health

# Expected: shard_2 still serving queries
# Observe failover behavior

# Recover
docker compose start shard_0 shard_1
```

### Scenario 3: Cascading Failures
```powershell
# Terminal 1: Continuous monitoring
while ($true) { 
    .\test-shards.ps1 -Command health
    Start-Sleep -Seconds 3
    Clear-Host
}

# Terminal 2: Cascade failures
docker compose stop shard_0
Start-Sleep -Seconds 5

docker compose stop shard_1
Start-Sleep -Seconds 5

docker compose stop shard_2
# Observe all shards going down

# Terminal 3: Recovery
docker compose start shard_0 shard_1 shard_2
```

## Docker Compose Commands

### Container Management
```powershell
# View running containers
docker compose ps

# View specific container logs
docker compose logs -f shard_0
docker compose logs -f coordinator

# Stop specific container (simulate failure)
docker compose stop shard_0

# Start specific container (recover)
docker compose start shard_0

# Restart container (hard reset)
docker compose restart shard_0

# Stop all containers
docker compose stop

# Start all containers
docker compose start

# Remove all containers (keeps volumes)
docker compose down

# Remove all containers and volumes
docker compose down -v
```

### Debugging
```powershell
# View container details
docker compose exec coordinator bash
docker compose exec shard_0 bash

# Inside container, useful commands:
# - curl http://shard_1:8011/shard_health
# - ping shard_1
# - ps aux
# - exit
```

## Health Check Configuration

Each container has a Docker health check:
```yaml
healthcheck:
  test: ["CMD", "curl", "-f", "http://localhost:PORT/shard_health"]
  interval: 5s           # Check every 5 seconds
  timeout: 2s            # Timeout after 2 seconds
  retries: 3             # Mark unhealthy after 3 failures
  start_period: 10s      # Wait 10s before first check
```

View health status:
```powershell
docker inspect rustdb_shard_0 | Select-String -Context 0,5 "State"
```

## Port Mapping

| Container | Port | Purpose |
|-----------|------|---------|
| coordinator | 8000 | Main RPC server, coordinates shards |
| shard_0 | 8010 | Individual shard 0 |
| shard_1 | 8011 | Individual shard 1 |
| shard_2 | 8012 | Individual shard 2 |

## Network Behavior

### Normal Operation (All Shards Up)
```
Key → Hash → Shard ID → Route to shard_X container → Response
```

### With One Shard Down
```
Key → Hash → Shard ID = 0 → shard_0 down → Fallback to shard_1/2
Key → Hash → Shard ID = 1 → shard_1 up → Response
Key → Hash → Shard ID = 2 → shard_2 up → Response
```

### With Multiple Shards Down
```
Key → Hash → Best available shard from remaining healthy shards
```

## Testing RPC Endpoints Directly

### From Host Machine
```powershell
# Get shard health
$response = Invoke-WebRequest -Uri "http://localhost:8000/shard_health" -Method POST
$response.Content | ConvertFrom-Json | Format-List

# Get shard status
$response = Invoke-WebRequest -Uri "http://localhost:8000/shard_status" -Method POST
$response.Content | ConvertFrom-Json | Format-List
```

### From Inside Docker Container
```powershell
# Enter coordinator container
docker compose exec coordinator bash

# Test from inside
curl http://shard_0:8010/shard_health
curl http://shard_1:8011/shard_health
curl http://shard_2:8012/shard_health

# Test coordinator
curl http://localhost:8000/shard_health
```

## Common Issues & Solutions

### Issue: Containers exit immediately
**Solution**: Check logs for errors
```powershell
docker compose logs shard_0
```

### Issue: Health checks failing
**Solution**: Ensure containers are fully started
```powershell
# Wait a bit and retry
Start-Sleep -Seconds 5
docker compose ps
```

### Issue: Can't connect from host
**Solution**: Verify ports are mapped
```powershell
docker compose ps
# Should show PORT mappings like 8010->8010/tcp
```

### Issue: Shard stays marked as down after restart
**Solution**: This is expected behavior (30s timeout to mark as healthy)
```powershell
# Wait 30+ seconds and check again
Start-Sleep -Seconds 35
.\test-shards.ps1 -Command health
```

## Performance Testing

### Load Test with Down Shards
```powershell
# Terminal 1: Start load test (from client, if available)
cargo run --release -- --client

# Terminal 2: Monitor health
while ($true) { 
    .\test-shards.ps1 -Command health
    Start-Sleep -Seconds 5
}

# Terminal 3: Introduce failures
docker compose stop shard_0
# Observe client behavior during failover
```

## Cleanup

```powershell
# Stop all containers
docker compose stop

# Remove all containers but keep volumes
docker compose down

# Remove everything including volumes
docker compose down -v

# Clean up unused Docker resources
docker system prune
```

## Next Steps

1. **Set up monitoring**: Use Prometheus/Grafana for shard metrics
2. **Add ZooKeeper**: Integrate for distributed coordination
3. **Implement auto-recovery**: Add logic to automatically restart failed shards
4. **Load balancing**: Add haproxy in front of shards
5. **Data persistence**: Mount volumes for data durability

---

**Documentation Updated**: November 20, 2025  
**Shard Monitoring Version**: 1.0  
**Docker Compose Version**: 3.8

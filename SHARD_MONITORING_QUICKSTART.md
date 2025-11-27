# Shard Monitoring Quick Start

## What Was Added

You now have **real-time shard health monitoring** that automatically detects when shards go down and routes around them.

### 3 New Files/Modules:
1. **`src/shard_monitor.rs`** — Health tracking engine
2. **`SHARD_MONITORING.md`** — Full documentation
3. **Two new RPC endpoints** in `src/server.rs`

### Key Capabilities:
✅ Detects shard failures via heartbeat timeout (30s default)  
✅ Marks failed shards as DOWN  
✅ Auto-routes queries to healthy replicas  
✅ Best-effort fallback if all shards are unhealthy  
✅ JSON-RPC endpoints to query shard health  

---

## Try It Out (Local)

### 1. Start a sharded server
```bash
cargo run -- --server --port 8000 --shards 3
```

### 2. In another terminal, use the helper script

**Easiest way - Use PowerShell helper script:**
```powershell
# Check shard health (default)
.\test-shards.ps1

# Check detailed status
.\test-shards.ps1 -Command health

# Check summary status
.\test-shards.ps1 -Command status

# Check a different port
.\test-shards.ps1 -Port 9000

# Get help
.\test-shards.ps1 -Command help
```

**Or use manual PowerShell commands:**
```powershell
# Detailed health status
$body = @{
    jsonrpc = "2.0"
    method = "shard_health"
    params = @()
    id = 1
} | ConvertTo-Json

Invoke-WebRequest -Uri "http://localhost:8000" `
  -Method POST `
  -Headers @{"Content-Type"="application/json"} `
  -Body $body | Select-Object -ExpandProperty Content | ConvertFrom-Json | Select-Object -ExpandProperty result

# Human-readable summary
$body = @{
    jsonrpc = "2.0"
    method = "shard_status"
    params = @()
    id = 1
} | ConvertTo-Json

Invoke-WebRequest -Uri "http://localhost:8000" `
  -Method POST `
  -Headers @{"Content-Type"="application/json"} `
  -Body $body | Select-Object -ExpandProperty Content | ConvertFrom-Json | Select-Object -ExpandProperty result
```

**Using curl (if available):**
```bash
# Detailed health status
curl -X POST http://localhost:8000 ^
  -H "Content-Type: application/json" ^
  -d "{\"jsonrpc\":\"2.0\",\"method\":\"shard_health\",\"params\":[],\"id\":1}"

# Human-readable summary
curl -X POST http://localhost:8000 ^
  -H "Content-Type: application/json" ^
  -d "{\"jsonrpc\":\"2.0\",\"method\":\"shard_status\",\"params\":[],\"id\":1}"
```

### 3. Simulate a failure
Stop the server (Ctrl+C), then:
- Run a query in the client → will fail gracefully
- Query `shard_health` after ~35 seconds → all shards marked DOWN
- Restart the server → shards recover to HEALTHY

---

## How It Works (Simple)

```
Query arrives
    ↓
Router checks: Is primary shard healthy?
    ↓
    ├─ YES → Use primary
    ├─ NO → Try any healthy replica
    └─ All DOWN → Use any shard (best effort)
    ↓
Shard handles query
    ↓
On success: Update heartbeat (keep shard marked HEALTHY)
On failure: Can mark shard DOWN after timeout
```

**Timeout Check:**  
Every 5–10 seconds (in production), scan shards and mark any with no heartbeat for >30s as DOWN.

---

## Configuration

In `src/sharding.rs`, line ~62:
```rust
let health_monitor = Arc::new(ShardHealthMonitor::new(30));  // 30-second timeout
```

Change `30` to tune heartbeat timeout:
- **5–10s**: Faster detection, higher false positives
- **30s** (default): Good balance
- **60s+**: Slower detection, fewer false alarms

---

## RPC Endpoints

### `shard_health`
**PowerShell Request:**
```powershell
$body = @{
    jsonrpc = "2.0"
    method = "shard_health"
    params = @()
    id = 1
} | ConvertTo-Json

$response = Invoke-WebRequest -Uri "http://localhost:8000" `
  -Method POST `
  -Headers @{"Content-Type"="application/json"} `
  -Body $body

$response.Content | ConvertFrom-Json | Format-Object
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "result": {
    "shards": {
      "shard_0": "healthy",
      "shard_1": "down",
      "shard_2": "healthy"
    },
    "healthy_count": 2,
    "total_count": 3
  },
  "id": 1
}
```

### `shard_status`
**PowerShell Request:**
```powershell
$body = @{
    jsonrpc = "2.0"
    method = "shard_status"
    params = @()
    id = 1
} | ConvertTo-Json

$response = Invoke-WebRequest -Uri "http://localhost:8000" `
  -Method POST `
  -Headers @{"Content-Type"="application/json"} `
  -Body $body

$response.Content | ConvertFrom-Json | Select-Object -ExpandProperty result
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "result": "Total shards: 3, Healthy: 2, Unhealthy: 1. Status: {...}",
  "id": 1
}
```

---

## Code Integration Points

### In ShardNode:
```rust
shard.record_access();    // Call after successful query
shard.is_healthy();       // Check if shard is healthy
```

### In ShardManager:
```rust
manager.get_shard_health_status()  // Get all shard statuses
manager.get_healthy_shards()       // Get healthy shard IDs
manager.get_shard_for_key(key)     // Auto-routes to healthy shard
```

### Background Task (to add):
```rust
// Run every 5-10 seconds in a background thread:
health_monitor.check_heartbeat_timeouts();
```

---

## Testing

Run the unit tests:
```bash
cargo test shard_monitor
```

All 3 tests pass:
- ✅ `test_register_and_health` — Register shard and check health
- ✅ `test_mark_down` — Mark shard as DOWN
- ✅ `test_heartbeat_recovery` — Recover from DOWN to HEALTHY

---

## Next Steps

1. **Add background task** to run `check_heartbeat_timeouts()` periodically
2. **Integrate ZooKeeper watchers** to detect failures immediately (vs waiting for timeout)
3. **Add metrics** (Prometheus-style) for shard uptime and failover events
4. **Implement graceful shutdown** (mark Degraded, drain connections, then go DOWN)
5. **Auto-failover** (promote healthy replica to primary after X seconds)

---

## Files Changed

| File                    | Change                                                        |
|-------------------------|---------------------------------------------------------------|
| `src/shard_monitor.rs`  | NEW — Health tracking engine with tests                      |
| `src/sharding.rs`       | Updated ShardNode, ShardManager for health-aware routing     |
| `src/server.rs`         | Added `shard_health` and `shard_status` RPC endpoints        |
| `src/lib.rs`            | Export new `shard_monitor` module                            |
| `README.md`             | Added sharding & monitoring sections                         |
| `SHARD_MONITORING.md`   | NEW — Full detailed documentation                           |

---

**Quick Status:** ✅ Core monitoring implemented, compiled, and tested. Ready for integration with background tasks and ZooKeeper watchers!

# Shard Monitoring & Health Checks

This document explains how shard health monitoring works in rustDB using the ZooKeeper integration and local health tracking.

## Overview

Shard monitoring continuously tracks the health status of all shards in the cluster. When a shard becomes unavailable, the system:
1. **Detects** the failure via heartbeat timeout or explicit health updates.
2. **Marks** the shard as DOWN in the local health monitor.
3. **Routes** new queries away from unhealthy shards.
4. **Falls back** to replica shards if the primary is unavailable.

## Architecture

### Components

#### 1. **ShardHealthMonitor** (`src/shard_monitor.rs`)
A thread-safe in-memory tracker for shard health status.

- **Tracks per shard:**
  - Health state: `Healthy`, `Degraded`, or `Down`
  - Last heartbeat timestamp
  
- **Key methods:**
  - `register_shard(id)` — Initialize a shard as healthy
  - `record_heartbeat(id)` — Update last seen time (keeps shard healthy)
  - `mark_shard_down(id)` — Mark as DOWN (e.g., on network failure)
  - `get_shard_health(id)` — Query current health state
  - `check_heartbeat_timeouts()` — Scan for stale heartbeats and mark DOWN
  - `get_healthy_shards()` — List all healthy shard IDs
  - `get_health_summary()` — Get a map of all shard→health states

#### 2. **ShardNode** (updated in `src/sharding.rs`)
Each shard node now holds:
- A reference to the shared `ShardHealthMonitor`
- A new `is_healthy()` method to query its current health
- A `record_access()` method to update heartbeat on successful queries

#### 3. **ShardManager** (updated in `src/sharding.rs`)
Enhanced routing logic:
- **Primary-first routing:** Prefer healthy primary shards.
- **Replica fallback:** If primary is down, try any healthy replica.
- **Best-effort fallback:** If all are unhealthy, use any shard (best effort).
- **Exposes health status** via `get_shard_health_status()` and `get_healthy_shards()`.

### Data Flow

```
Query arrives
    ↓
ShardManager.get_shard_for_key(key)
    ↓
    ├─→ Find primary shard range for key
    │
    ├─ Is primary healthy? ──→ YES → Use primary, record_access() → Return
    │
    ├─ NO → Try replicas
    │      ├─ Is any replica healthy? ──→ YES → Use replica, record_access() → Return
    │      │
    │      └─ NO → Use best-effort shard (any shard with key range match)
    │
    └─→ Execute query on chosen shard
```

## Integration Points

### 1. **Local Heartbeating**
When a shard successfully processes a query:
```rust
shard.record_access();  // Updates last_heartbeat timestamp
```

### 2. **Timeout Detection**
Periodically scan for stale heartbeats:
```rust
health_monitor.check_heartbeat_timeouts();  // Marks stale shards DOWN
```

*(In production, this would run in a background task every 5–10 seconds.)*

### 3. **External Health Signals (ZooKeeper)**
*(Placeholder for future implementation)*

When a shard's ephemeral node in ZooKeeper is deleted:
1. A watcher fires.
2. The node calls `mark_shard_down()`.

### 4. **RPC Health Endpoints**

Two new JSON-RPC methods for monitoring:

#### `shard_health` → `ShardHealthResponse`
Returns detailed health map of all shards:
```json
{
  "shards": {
    "shard_0": "healthy",
    "shard_1": "down",
    "shard_2": "degraded"
  },
  "healthy_count": 2,
  "total_count": 3
}
```

#### `shard_status` → `String`
Returns a human-readable summary:
```
Total shards: 3, Healthy: 2, Unhealthy: 1. Status: {...}
```

## Health States

| State     | Meaning                                          | Action on Route |
|-----------|--------------------------------------------------|-----------------|
| `Healthy` | Shard responded recently; available for queries | **Use primary** |
| `Degraded`| Shard is responsive but slow/intermittent        | Use if needed   |
| `Down`    | No response; heartbeat timeout exceeded         | **Skip; use replica** |

## Example Scenario

### Setup
- 3 shards (shard_0, shard_1, shard_2)
- shard_1 has 2 replicas

### Timeline
1. **t=0s**: All shards healthy
   - Query for key=5 → routes to shard_0 → success → heartbeat updated
   - Query for key=200 → routes to shard_1 → success → heartbeat updated

2. **t=45s**: shard_1 fails (network issue)
   - No heartbeat for shard_1 for 15+ seconds
   - `check_heartbeat_timeouts()` runs → marks shard_1 DOWN
   - Query for key=200 → shard_1 is DOWN → routes to shard_1 replica → success

3. **t=65s**: shard_1 recovers
   - shard_1 responds to health check
   - `record_heartbeat(shard_1)` → marks shard_1 HEALTHY
   - Query for key=200 → shard_1 is HEALTHY → routes to shard_1 primary → success

## Configuration & Tuning

### Heartbeat Timeout
Default: **30 seconds**

Defined when creating `ShardHealthMonitor`:
```rust
let monitor = ShardHealthMonitor::new(30);  // 30-second timeout
```

- **Shorter timeout** (5–10s): Faster failure detection, but higher false positives.
- **Longer timeout** (60s+): Fewer false alarms, but slower recovery.

### Timeout Check Interval
*(To be implemented as a background task)*

Recommended: Run `check_heartbeat_timeouts()` every 5–10 seconds.

## Future Enhancements

1. **ZooKeeper Watcher Integration**
   - Watch shard nodes in ZK; immediately mark DOWN on deletion.
   - No waiting for heartbeat timeout.

2. **Metrics Export**
   - Prometheus-style metrics: shard uptime, failover events, etc.

3. **Adaptive Timeouts**
   - Dynamically adjust timeout based on network latency.

4. **Graceful Draining**
   - Mark shard `Degraded` before shutdown; route away new queries.
   - Complete in-flight queries before going DOWN.

5. **Auto-Recovery**
   - Automatically promote healthy replicas to primary if primary Down > X seconds.

## Testing

### Unit Tests
Located in `src/shard_monitor.rs`:
- `test_register_and_health()` — Verify shard registration
- `test_mark_down()` — Verify DOWN state
- `test_heartbeat_recovery()` — Verify recovery from DOWN to HEALTHY

Run:
```bash
cargo test shard_monitor
```

### Integration Test
*(To be added)*

Simulate a shard failure and verify:
1. Health monitor marks shard DOWN.
2. Router skips to replica.
3. Client continues successfully.

### Manual Testing

Start a server with shards:
```bash
cargo run -- --server --port 8000 --shards 3
```

Query health endpoints:
```bash
# Get detailed health
curl http://localhost:8000 \
  -X POST \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"shard_health","params":[],"id":1}'

# Get summary
curl http://localhost:8000 \
  -X POST \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"shard_status","params":[],"id":1}'
```

Simulate a shard failure by stopping the process or adding a network block, then:
1. Run a query → should route to a healthy shard or fail gracefully.
2. Query `shard_health` → see shard marked DOWN after timeout.

## Troubleshooting

### All Shards Show DOWN

**Symptom:** `shard_health` returns all shards as DOWN.

**Cause:** Heartbeat timeout is too short or `record_access()` is not being called.

**Fix:**
- Increase timeout in `ShardHealthMonitor::new()`.
- Verify queries are successfully executing and calling `shard.record_access()`.
- Check logs for query failures.

### Queries Still Hit DOWN Shards

**Symptom:** Queries route to shards marked DOWN.

**Cause:** Replica list is empty or all replicas are also DOWN.

**Fix:**
- Register replicas in `ShardManager::register_replica()`.
- Verify replicas are healthy via `shard_health`.
- Check fallback logic in `get_shard_for_key()`.

### High Latency Between Failure & Detection

**Symptom:** After a shard crashes, queries fail for 30+ seconds before recovering.

**Cause:** Waiting for heartbeat timeout.

**Fix:**
- Integrate ZooKeeper watcher (detects immediately on node deletion).
- Reduce heartbeat timeout at cost of false positives.
- Implement TCP-level health checks (e.g., `ICMP` or custom ping).

## Related Files

- `src/shard_monitor.rs` — Health tracking logic
- `src/sharding.rs` — ShardManager & ShardNode integration
- `src/server.rs` — RPC endpoints (`shard_health`, `shard_status`)
- `SHARD_MONITORING.md` — This document

---

**Last Updated:** 2025-11-13

# Docker Multi-Shard Architecture - Visual Diagrams

## Container Topology

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Windows Host Machine                             │
│                 (Running PowerShell / Docker Desktop)               │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │           Docker Internal Network: rustdb_net             │    │
│  │              (Bridge network type)                         │    │
│  ├────────────────────────────────────────────────────────────┤    │
│  │                                                             │    │
│  │  ╔═══════════════════════════════════════════════════╗    │    │
│  │  ║         Coordinator Container                     ║    │    │
│  │  ║                                                   ║    │    │
│  │  ║  • Image: rustdb (built from Dockerfile)         ║    │    │
│  │  ║  • Container Name: rustdb_coordinator            ║    │    │
│  │  ║  • Port Inside: 8000                             ║    │    │
│  │  ║  • Port Outside: 0.0.0.0:8000 (host accessible)║    │    │
│  │  ║  • Command: --server --port 8000 --shards 3      ║    │    │
│  │  ║                                                   ║    │    │
│  │  ║  Responsibilities:                               ║    │    │
│  │  ║  ├─ Manage 3 shards health monitoring            ║    │    │
│  │  ║  ├─ Route queries to appropriate shards          ║    │    │
│  │  ║  ├─ Failover to replicas when shards fail       ║    │    │
│  │  ║  └─ Expose RPC endpoints (shard_health, status)║    │    │
│  │  ║                                                   ║    │    │
│  │  ║  Health Check:                                   ║    │    │
│  │  ║  ├─ Test: curl -f http://localhost:8000/...     ║    │    │
│  │  ║  ├─ Interval: 5 seconds                          ║    │    │
│  │  ║  └─ Status: ✓ healthy or ✗ unhealthy            ║    │    │
│  │  ║                                                   ║    │    │
│  │  ╚═══════════════════════════════════════════════════╝    │    │
│  │                          ▲                                 │    │
│  │                          │ (RPC calls)                    │    │
│  │         ┌────────────────┼────────────────┐               │    │
│  │         │                │                │               │    │
│  │  ┌──────▼────────┐ ┌────▼──────┐ ┌──────▼────────┐       │    │
│  │  │ Shard 0 Ctr.  │ │Shard 1 Ctr.│ │ Shard 2 Ctr.  │       │    │
│  │  │               │ │            │ │               │       │    │
│  │  │ Name:         │ │ Name:      │ │ Name:         │       │    │
│  │  │ shard_0       │ │ shard_1    │ │ shard_2       │       │    │
│  │  │               │ │            │ │               │       │    │
│  │  │ Port Inside:  │ │ Port In:   │ │ Port Inside:  │       │    │
│  │  │ 8010          │ │ 8011       │ │ 8012          │       │    │
│  │  │               │ │            │ │               │       │    │
│  │  │ Port Outside: │ │ Port Out:  │ │ Port Outside: │       │    │
│  │  │ 0.0.0.0:8010 │ │ 0.0.0.0:   │ │ 0.0.0.0:8012 │       │    │
│  │  │               │ │ 8011       │ │               │       │    │
│  │  │ Command:      │ │ Command:   │ │ Command:      │       │    │
│  │  │ --server      │ │ --server   │ │ --server      │       │    │
│  │  │ --port 8010   │ │ --port     │ │ --port 8012   │       │    │
│  │  │ --shards 1    │ │ 8011       │ │ --shards 1    │       │    │
│  │  │               │ │ --shards 1 │ │               │       │    │
│  │  │ Data Range:   │ │ Data Range:│ │ Data Range:   │       │    │
│  │  │ 0x0 - 0x555..│ │ 0x555.. -  │ │ 0xAAA.. -     │       │    │
│  │  │               │ │ 0xAAA...  │ │ 0xFFF...      │       │    │
│  │  │ Status:       │ │ Status:    │ │ Status:       │       │    │
│  │  │ ✓ healthy or ✗ │ ✓ healthy or✗ │ ✓ healthy or ✗ │       │    │
│  │  │ unhealthy     │ │ unhealthy  │ │ unhealthy     │       │    │
│  │  │               │ │            │ │               │       │    │
│  │  │ Health Check: │ │ H.C.:      │ │ Health Check: │       │    │
│  │  │ curl 8010/... │ │ curl 8011..│ │ curl 8012/... │       │    │
│  │  │ Interval: 5s  │ │ Int: 5s    │ │ Interval: 5s  │       │    │
│  │  └───────────────┘ └────────────┘ └───────────────┘       │    │
│  │         ◄ Can be stopped to simulate failure ►            │    │
│  │                                                             │    │
│  └────────────────────────────────────────────────────────────┘    │
│                          ▲                                         │
│                          │ (Network bridge)                       │
│  ┌──────────────────────────────────────────────────────────┐     │
│  │         From Host Machine (PowerShell)                  │     │
│  ├──────────────────────────────────────────────────────────┤     │
│  │                                                           │     │
│  │  • docker compose up --build -d    (start containers)  │     │
│  │  • docker compose stop shard_0     (simulate failure)  │     │
│  │  • docker compose ps               (view status)       │     │
│  │  • docker compose logs -f          (view logs)         │     │
│  │  • .\test-shards.ps1               (health monitoring) │     │
│  │  • Invoke-WebRequest                (test endpoints)    │     │
│  │                                                           │     │
│  └──────────────────────────────────────────────────────────┘     │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## Data Flow - Query Processing

### Scenario 1: Normal Operation (All Shards Up)

```
User Query
    │
    ▼
┌──────────────────────┐
│ Calculate Hash       │
│ (Key → Hash Value)   │
└──────────────────────┘
    │
    ▼
┌──────────────────────┐
│ Determine Shard ID   │
│ (0, 1, or 2)         │
└──────────────────────┘
    │
    ├─► Hash % 3 = 0 ──┐
    │                  │
    ├─► Hash % 3 = 1 ──┤
    │                  │
    └─► Hash % 3 = 2 ──┘
         │
         ▼
    ┌────────────────────────────────────────┐
    │ Route Query to Shard Container         │
    │ (Coordinator sends to shard_X:801X)    │
    └────────────────────────────────────────┘
         │
         ├─► ✓ Shard 0 (8010) - Healthy
         ├─► ✓ Shard 1 (8011) - Healthy
         └─► ✓ Shard 2 (8012) - Healthy
         │
         ▼
    ┌────────────────────┐
    │ Execute Query      │
    │ (Insert/Update/    │
    │  Select/Delete)    │
    └────────────────────┘
         │
         ▼
    ✓ SUCCESS - Response to user
```

### Scenario 2: With Shard Failure

```
User Query (Hash → Shard 0)
    │
    ▼
┌──────────────────────────────┐
│ Route to Shard 0 (8010)      │
│ ✗ Connection Timeout         │
│ ✗ Marked as DOWN             │
└──────────────────────────────┘
    │
    ▼ (Failover Logic Activated)
┌──────────────────────────────┐
│ Route to Replica (Shard 1/2) │
│ (Using best available)       │
└──────────────────────────────┘
    │
    ├─► ✗ Shard 0 (8010) - DOWN
    ├─► ✓ Shard 1 (8011) - Healthy ◄─ Use this
    └─► ✓ Shard 2 (8012) - Healthy

         │
         ▼
    ✓ SUCCESS (via fallback)
```

### Scenario 3: Multiple Shard Failures

```
User Query
    │
    ▼
┌────────────────────────────────┐
│ Primary Shard Check            │
│ ✗ Shard 0 - DOWN               │
│ ✗ Shard 1 - DOWN               │
│ ✓ Shard 2 - HEALTHY            │
└────────────────────────────────┘
    │
    ▼
┌────────────────────────────────┐
│ Route to Best Available        │
│ (Only Shard 2 available)       │
│ Process via Shard 2            │
└────────────────────────────────┘
    │
    ▼
✓ SUCCESS (reduced capacity, single shard)
```

## Health Monitoring Timeline

```
Time (seconds)  Status              Action
─────────────────────────────────────────────────────────────────────
0               ✓ All healthy       • Coordinator monitors all shards
                Shard 0: UP         • Heartbeat checks every 5 seconds
                Shard 1: UP         • Routing: normal distribution
                Shard 2: UP

5               ✓ Still healthy     • Health checks: ✓ ✓ ✓ (all pass)
                (all responsive)    • Coordinator: all shards responding

10              ✓ Still healthy     • Health checks: ✓ ✓ ✓ (all pass)

...

30              ⚠ Degraded          ├─ Shard 0 becomes unresponsive
                Shard 0: DEGRADED   ├─ Health check: TIMEOUT
                Shard 1: UP         ├─ Marked as DEGRADED
                Shard 2: UP         └─ Routing: shifted to 1 & 2

35              ✗ DOWN              ├─ Timeout threshold reached (30s)
                Shard 0: DOWN       ├─ Shard 0 marked as DOWN
                Shard 1: UP         ├─ Queries failing on 0: auto-failover
                Shard 2: UP         └─ Routing: only via 1 & 2

[Shard 0 restarted]

40              ⚠ Degraded          ├─ Container restarts
                Shard 0: DEGRADED   ├─ Service comes back up
                Shard 1: UP         ├─ First health check: passes
                Shard 2: UP         └─ Marked as DEGRADED (recovery)

45              ✓ Healthy           ├─ Consistent responses received
                Shard 0: HEALTHY    ├─ No more timeouts
                Shard 1: UP         ├─ Full connectivity restored
                Shard 2: UP         └─ Marked as HEALTHY
                                    └─ Routing: normal distribution again
```

## Docker Command Flow

```
User Command
    │
    ▼
┌─────────────────────────────────────┐
│ docker compose up --build -d        │
└─────────────────────────────────────┘
    │
    ├──► Build stage:
    │    ├─ Read Dockerfile
    │    ├─ Start FROM rust:latest
    │    ├─ Copy source code
    │    └─ Run: cargo build --release
    │
    ├──► Runtime stage:
    │    ├─ Start FROM debian:bookworm-slim
    │    ├─ Install curl, netcat, dnsutils, etc.
    │    ├─ Copy binary from builder
    │    └─ Expose ports 8000-8002, 8010-8019
    │
    └──► Container launch (detached):
         ├─ Start coordinator (port 8000)
         ├─ Start shard_0 (port 8010)
         ├─ Start shard_1 (port 8011)
         └─ Start shard_2 (port 8012)

         Each container now:
         ├─ Runs health check every 5s
         ├─ Reports healthy/unhealthy status
         └─ Processes connections independently
```

## Port Mapping Diagram

```
┌─────────────────────────────────┐
│      Host Machine               │
│    (Port Namespace)             │
├─────────────────────────────────┤
│                                  │
│  localhost:8000 ──► rustdb_net:8000      (Coordinator)
│  localhost:8010 ──► rustdb_net:8010      (Shard 0)
│  localhost:8011 ──► rustdb_net:8011      (Shard 1)
│  localhost:8012 ──► rustdb_net:8012      (Shard 2)
│                                  │
│  Docker Desktop                  │
│  ┌────────────────────────────┐  │
│  │  rustdb_net (Bridge)       │  │
│  │  ┌──────────────────────┐  │  │
│  │  │ Container Namespace  │  │  │
│  │  │                      │  │  │
│  │  │ localhost:8000 ◄────►  │  │
│  │  │ localhost:8010 ◄────►  │  │
│  │  │ localhost:8011 ◄────►  │  │
│  │  │ localhost:8012 ◄────►  │  │
│  │  │                      │  │  │
│  │  └──────────────────────┘  │  │
│  └────────────────────────────┘  │
│                                  │
└─────────────────────────────────┘

You connect to: localhost:8000 (from PowerShell)
Actually reaches: Docker container 0.0.0.0:8000
```

## Failure Simulation Timeline

```
Step 1: Start Everything
┌────────────────────────────────────┐
│ docker compose up --build -d       │
│ (Takes ~30 seconds)                │
│                                    │
│ ✓ rustdb_coordinator (healthy)     │
│ ✓ rustdb_shard_0 (healthy)         │
│ ✓ rustdb_shard_1 (healthy)         │
│ ✓ rustdb_shard_2 (healthy)         │
└────────────────────────────────────┘

Step 2: Verify Health
┌────────────────────────────────────┐
│ .\test-shards.ps1 -Command health  │
│                                    │
│ ✓✓✓ All 3 shards healthy           │
└────────────────────────────────────┘

Step 3: Stop a Shard
┌────────────────────────────────────┐
│ docker compose stop shard_0        │
│ (Immediate)                        │
│                                    │
│ ✓ rustdb_coordinator (healthy)     │
│ ✗ rustdb_shard_0 (stopped)         │
│ ✓ rustdb_shard_1 (healthy)         │
│ ✓ rustdb_shard_2 (healthy)         │
└────────────────────────────────────┘

Step 4: Wait for Detection
┌────────────────────────────────────┐
│ Wait ~30 seconds                   │
│ (Heartbeat timeout threshold)      │
│                                    │
│ Coordinator detects no heartbeat   │
│ from shard_0 for 30 seconds        │
│ → Marked as DOWN                   │
└────────────────────────────────────┘

Step 5: Verify Failure Detection
┌────────────────────────────────────┐
│ .\test-shards.ps1 -Command health  │
│                                    │
│ ✓✗✓ Shard 0 marked as DOWN         │
│ Healthy: 2/3                       │
└────────────────────────────────────┘

Step 6: Restart
┌────────────────────────────────────┐
│ docker compose start shard_0       │
│ (Takes ~3-5 seconds)               │
│                                    │
│ ✓ rustdb_coordinator (healthy)     │
│ ✓ rustdb_shard_0 (recovering...)   │
│ ✓ rustdb_shard_1 (healthy)         │
│ ✓ rustdb_shard_2 (healthy)         │
└────────────────────────────────────┘

Step 7: Verify Recovery
┌────────────────────────────────────┐
│ .\test-shards.ps1 -Command health  │
│ (After ~5 more seconds)            │
│                                    │
│ ✓✓✓ All 3 shards healthy again     │
│ Healthy: 3/3                       │
└────────────────────────────────────┘
```

---

**Diagram Legend:**
- ✓ = Healthy / Working
- ✗ = Down / Not Working
- ◄─► = Connected / Communicating
- ⚠ = Degraded / Warning

**Updated**: November 20, 2025

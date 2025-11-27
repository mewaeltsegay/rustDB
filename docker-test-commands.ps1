# Docker Shard Testing Commands
# Quick reference for common Docker operations

# Color output
function Write-Success { Write-Host $args -ForegroundColor Green }
function Write-Error { Write-Host $args -ForegroundColor Red }
function Write-Warning { Write-Host $args -ForegroundColor Yellow }
function Write-Info { Write-Host $args -ForegroundColor Cyan }

function Show-Menu {
    Clear-Host
    Write-Info "╔════════════════════════════════════════════════════════╗"
    Write-Info "║      Docker Shard Testing - Command Menu               ║"
    Write-Info "╚════════════════════════════════════════════════════════╝"
    Write-Host ""
    Write-Info "Container Management:"
    Write-Host "  1. Start all containers (docker compose up --build -d)"
    Write-Host "  2. Stop all containers (docker compose stop)"
    Write-Host "  3. View container status (docker compose ps)"
    Write-Host "  4. View logs (all containers)"
    Write-Host ""
    Write-Info "Shard Failure Simulation:"
    Write-Host "  5. Stop shard_0 (simulate failure)"
    Write-Host "  6. Stop shard_1 (simulate failure)"
    Write-Host "  7. Stop shard_2 (simulate failure)"
    Write-Host "  8. Stop multiple shards (chaos test)"
    Write-Host ""
    Write-Info "Recovery & Monitoring:"
    Write-Host "  9. Restart all shards"
    Write-Host "  10. Restart specific shard (interactive)"
    Write-Host "  11. Check shard health (.\test-shards.ps1)"
    Write-Host "  12. Continuous health monitoring"
    Write-Host ""
    Write-Info "Debugging:"
    Write-Host "  13. View coordinator logs"
    Write-Host "  14. View shard_0 logs"
    Write-Host "  15. Test RPC endpoints directly"
    Write-Host ""
    Write-Info "Cleanup:"
    Write-Host "  16. Remove all containers (docker compose down)"
    Write-Host "  17. Remove containers and volumes (docker compose down -v)"
    Write-Host ""
    Write-Host "  0. Exit"
    Write-Host ""
}

function Invoke-Command {
    param([string]$Command, [string]$Description)
    
    Write-Warning "`n[RUNNING] $Description"
    Write-Info "Command: $Command`n"
    
    Invoke-Expression $Command
    
    Write-Success "`n[DONE] $Description`n"
}

function Select-Shard {
    Write-Host ""
    Write-Host "Select shard:"
    Write-Host "  1. shard_0"
    Write-Host "  2. shard_1"
    Write-Host "  3. shard_2"
    Write-Host ""
    $choice = Read-Host "Enter choice (1-3)"
    
    $shards = @("shard_0", "shard_1", "shard_2")
    if ([int]$choice -ge 1 -and [int]$choice -le 3) {
        return $shards[[int]$choice - 1]
    }
    return $null
}

function Test-RpcEndpoint {
    Write-Host ""
    Write-Host "Testing RPC Endpoints on http://localhost:8000"
    Write-Host ""
    
    try {
        Write-Info "Testing shard_health endpoint..."
        $response = Invoke-WebRequest -Uri "http://localhost:8000/shard_health" -Method POST -ErrorAction Stop
        $json = $response.Content | ConvertFrom-Json
        Write-Host "Health Status:" -ForegroundColor Cyan
        $json | ConvertTo-Json -Depth 3 | Write-Host
        
        Write-Host ""
        Write-Info "Testing shard_status endpoint..."
        $response = Invoke-WebRequest -Uri "http://localhost:8000/shard_status" -Method POST -ErrorAction Stop
        $json = $response.Content | ConvertFrom-Json
        Write-Host "Status Summary:" -ForegroundColor Cyan
        $json | ConvertTo-Json -Depth 3 | Write-Host
    }
    catch {
        Write-Error "Failed to connect. Is the coordinator running?"
        Write-Error $_.Exception.Message
    }
}

# Main loop
$running = $true
while ($running) {
    Show-Menu
    $choice = Read-Host "Enter your choice"
    
    switch ($choice) {
        "1" {
            Invoke-Command "docker compose up --build -d" "Starting all containers"
            Start-Sleep -Seconds 3
            Invoke-Command "docker compose ps" "Verifying containers are running"
        }
        
        "2" {
            Invoke-Command "docker compose stop" "Stopping all containers"
        }
        
        "3" {
            Invoke-Command "docker compose ps" "Viewing container status"
        }
        
        "4" {
            Write-Warning "`n[RUNNING] Viewing logs (Ctrl+C to stop)"
            docker compose logs -f
        }
        
        "5" {
            Invoke-Command "docker compose stop shard_0" "Stopping shard_0 (simulating failure)"
            Write-Warning "`nWait 30 seconds for health monitor to mark it as DOWN, then run:"
            Write-Info ".\test-shards.ps1 -Command health`n"
        }
        
        "6" {
            Invoke-Command "docker compose stop shard_1" "Stopping shard_1 (simulating failure)"
            Write-Warning "`nWait 30 seconds for health monitor to mark it as DOWN, then run:"
            Write-Info ".\test-shards.ps1 -Command health`n"
        }
        
        "7" {
            Invoke-Command "docker compose stop shard_2" "Stopping shard_2 (simulating failure)"
            Write-Warning "`nWait 30 seconds for health monitor to mark it as DOWN, then run:"
            Write-Info ".\test-shards.ps1 -Command health`n"
        }
        
        "8" {
            Write-Warning "`n[CHAOS TEST] Stopping all shards"
            Invoke-Command "docker compose stop shard_0 shard_1 shard_2" "Stopping all shards"
            Write-Warning "`nAll shards are down. Run:"
            Write-Info ".\test-shards.ps1 -Command health`n"
            Write-Warning "`nTo recover, press any key..."
            $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
            Invoke-Command "docker compose start shard_0 shard_1 shard_2" "Restarting all shards"
        }
        
        "9" {
            Invoke-Command "docker compose start shard_0 shard_1 shard_2" "Restarting all shards"
            Start-Sleep -Seconds 2
            Write-Info "Shards are restarting. Run the following to check health:"
            Write-Info ".\test-shards.ps1 -Command health`n"
        }
        
        "10" {
            $shard = Select-Shard
            if ($shard) {
                Invoke-Command "docker compose restart $shard" "Restarting $shard"
            }
        }
        
        "11" {
            if (Test-Path ".\test-shards.ps1") {
                & ".\test-shards.ps1" -Command health
            }
            else {
                Write-Error "test-shards.ps1 not found in current directory"
            }
        }
        
        "12" {
            Write-Warning "`n[MONITORING] Continuous health check (Ctrl+C to stop)`n"
            while ($true) {
                Clear-Host
                Write-Info "Shard Health Monitor - $(Get-Date -Format 'HH:mm:ss')"
                Write-Info "════════════════════════════════════════════`n"
                
                if (Test-Path ".\test-shards.ps1") {
                    & ".\test-shards.ps1" -Command health
                }
                else {
                    Write-Error "test-shards.ps1 not found"
                    break
                }
                
                Write-Info "`nRefreshing in 5 seconds... (Ctrl+C to stop)"
                Start-Sleep -Seconds 5
            }
        }
        
        "13" {
            Write-Warning "`n[LOGS] Coordinator logs (Ctrl+C to stop)`n"
            docker compose logs -f coordinator
        }
        
        "14" {
            Write-Warning "`n[LOGS] Shard_0 logs (Ctrl+C to stop)`n"
            docker compose logs -f shard_0
        }
        
        "15" {
            Test-RpcEndpoint
        }
        
        "16" {
            $confirm = Read-Host "Remove all containers? (y/n)"
            if ($confirm -eq 'y') {
                Invoke-Command "docker compose down" "Removing all containers"
            }
        }
        
        "17" {
            $confirm = Read-Host "Remove all containers AND volumes? (y/n)"
            if ($confirm -eq 'y') {
                Invoke-Command "docker compose down -v" "Removing all containers and volumes"
            }
        }
        
        "0" {
            Write-Success "`nGoodbye!"
            $running = $false
        }
        
        default {
            Write-Error "Invalid choice. Please try again."
        }
    }
    
    if ($running) {
        Write-Host ""
        Read-Host "Press Enter to continue"
    }
}

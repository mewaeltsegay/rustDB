# PowerShell helper script for rustDB shard monitoring
# Usage: .\test-shards.ps1 -Command "health" -Port 8000

param(
    [string]$Command = "health",
    [int]$Port = 8000,
    [string]$ServerHost = "localhost"
)

$baseUrl = "http://$ServerHost`:$Port"

function Invoke-ShardRpc {
    param(
        [string]$Method,
        [int]$Id = 1
    )
    
    $body = @{
        jsonrpc = "2.0"
        method = $Method
        params = @()
        id = $Id
    } | ConvertTo-Json
    
    try {
        $response = Invoke-WebRequest -Uri $baseUrl `
            -Method POST `
            -Headers @{"Content-Type"="application/json"} `
            -Body $body `
            -ErrorAction Stop
        
        $jsonResponse = $response.Content | ConvertFrom-Json
        
        if ($jsonResponse.result) {
            return $jsonResponse.result
        } elseif ($jsonResponse.error) {
            Write-Error "RPC Error: $($jsonResponse.error.message)"
            return $null
        }
    } catch {
        Write-Error "Connection failed: $_"
        return $null
    }
}

function Show-ShardHealth {
    Write-Host "`n=== Shard Health Status ===" -ForegroundColor Cyan
    
    $result = Invoke-ShardRpc -Method "shard_health"
    if ($result) {
        Write-Host "`nDetailed Status:`n" -ForegroundColor Green
        
        $result.shards.PSObject.Properties | ForEach-Object {
            $color = if ($_.Value -eq "healthy") { "Green" } else { "Red" }
            Write-Host "  $($_.Name): $($_.Value)" -ForegroundColor $color
        }
        
        Write-Host "`nSummary:" -ForegroundColor Green
        Write-Host "  Healthy shards:  $($result.healthy_count)" -ForegroundColor Green
        Write-Host "  Total shards:    $($result.total_count)" -ForegroundColor Cyan
        Write-Host "  Unhealthy:       $($result.total_count - $result.healthy_count)" -ForegroundColor Yellow
    }
}

function Show-ShardStatus {
    Write-Host "`n=== Shard Status ===" -ForegroundColor Cyan
    
    $result = Invoke-ShardRpc -Method "shard_status"
    if ($result) {
        Write-Host "`n$result`n" -ForegroundColor Green
    }
}

function Show-Help {
    Write-Host @"
rustDB Shard Monitoring Helper

Usage:
  .\test-shards.ps1 [options]

Options:
  -Command <string>    : Command to run (default: health)
                        - health   : Show detailed shard health
                        - status   : Show shard status summary
                        - help     : Show this help
  -Port <int>          : Server port (default: 8000)
  -ServerHost <string> : Server hostname (default: localhost)

Examples:
  # Check shard health on localhost:8000
  .\test-shards.ps1

  # Check shard health on localhost:9000
  .\test-shards.ps1 -Port 9000

  # Check shard status on remote server
  .\test-shards.ps1 -Command status -ServerHost 192.168.1.100

"@
}

# Main execution
switch ($Command.ToLower()) {
    "health" { Show-ShardHealth }
    "status" { Show-ShardStatus }
    "help"   { Show-Help }
    default  { 
        Write-Host "Unknown command: $Command" -ForegroundColor Red
        Show-Help
        exit 1
    }
}

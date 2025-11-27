#!/usr/bin/env pwsh
# Test if SELECT queries return rows via RPC

$coordinatorUrl = "http://localhost:8080"

# Helper function to make JSON-RPC calls
function Invoke-JsonRpc {
    param(
        [string]$Method,
        [object]$Params
    )
    
    $body = @{
        jsonrpc = "2.0"
        id = 1
        method = $Method
        params = $Params
    } | ConvertTo-Json -Depth 10
    
    $response = Invoke-WebRequest -Uri $coordinatorUrl -Method Post -ContentType "application/json" -Body $body -ErrorAction Stop
    $result = $response.Content | ConvertFrom-Json
    
    if ($result.error) {
        Write-Host "❌ RPC Error: $($result.error.message)"
        return $null
    }
    
    return $result.result
}

Write-Host "=== Testing SELECT Query Results ===" -ForegroundColor Cyan
Write-Host ""

# Create a test table
Write-Host "Creating test table 'users'..." -ForegroundColor Yellow
$createResult = Invoke-JsonRpc -Method "execute" -Params @("CREATE TABLE users (id INT, name TEXT, age INT)")
Write-Host "✓ Table created" -ForegroundColor Green
Write-Host ""

# Insert test data
Write-Host "Inserting test data..." -ForegroundColor Yellow
$inserts = @(
    "INSERT INTO users VALUES (1, 'Alice', 30)",
    "INSERT INTO users VALUES (2, 'Bob', 25)",
    "INSERT INTO users VALUES (3, 'Charlie', 35)"
)

foreach ($insert in $inserts) {
    $result = Invoke-JsonRpc -Method "execute" -Params @($insert)
    Write-Host "  ✓ $insert" -ForegroundColor Green
}
Write-Host ""

# Test SELECT * query
Write-Host "Testing SELECT * FROM users..." -ForegroundColor Yellow
$selectResult = Invoke-JsonRpc -Method "execute" -Params @("SELECT * FROM users")

if ($null -ne $selectResult.rows -and $selectResult.rows.Count -gt 0) {
    Write-Host "SELECT returned $($selectResult.rows.Count) rows!" -ForegroundColor Green
    Write-Host ""
    Write-Host "Rows received:" -ForegroundColor Cyan
    foreach ($row in $selectResult.rows) {
        Write-Host "  $row"
    }
} elseif ($null -eq $selectResult) {
    Write-Host "SELECT returned null result" -ForegroundColor Red
} else {
    Write-Host "SELECT returned no rows" -ForegroundColor Yellow
    Write-Host "Full response: $($selectResult | ConvertTo-Json)" -ForegroundColor Gray
}

Write-Host ""

# Test SELECT with WHERE clause
Write-Host "Testing SELECT name FROM users WHERE age > 25..." -ForegroundColor Yellow
$selectWhereResult = Invoke-JsonRpc -Method "execute" -Params @("SELECT name FROM users WHERE age > 25")

if ($null -ne $selectWhereResult.rows -and $selectWhereResult.rows.Count -gt 0) {
    Write-Host "SELECT with WHERE returned $($selectWhereResult.rows.Count) rows!" -ForegroundColor Green
    Write-Host ""
    Write-Host "Rows received:" -ForegroundColor Cyan
    foreach ($row in $selectWhereResult.rows) {
        Write-Host "  $row"
    }
} elseif ($null -eq $selectWhereResult) {
    Write-Host "SELECT with WHERE returned null result" -ForegroundColor Red
} else {
    Write-Host "SELECT with WHERE returned no rows" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "=== Test Complete ===" -ForegroundColor Cyan

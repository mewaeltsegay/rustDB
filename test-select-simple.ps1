$url = "http://localhost:8000"

Write-Host "=== Testing SELECT Query Results ===" -ForegroundColor Cyan
Write-Host ""

# Function to call RPC
function CallRpc($method, $query) {
    $body = @{
        jsonrpc = "2.0"
        id = 1
        method = $method
        params = @($query)
    } | ConvertTo-Json

    try {
        $resp = Invoke-WebRequest -Uri $url -Method Post -ContentType "application/json" -Body $body
        return $resp.Content | ConvertFrom-Json
    } catch {
        Write-Host "Error: $_" -ForegroundColor Red
        return $null
    }
}

# Create table
Write-Host "Creating test table..." -ForegroundColor Yellow
CallRpc "execute" "CREATE TABLE users (id INT, name TEXT, age INT)" > $null

# Insert data
Write-Host "Inserting test data..." -ForegroundColor Yellow
CallRpc "execute" "INSERT INTO users VALUES (1, 'Alice', 30)" > $null
CallRpc "execute" "INSERT INTO users VALUES (2, 'Bob', 25)" > $null
CallRpc "execute" "INSERT INTO users VALUES (3, 'Charlie', 35)" > $null

# Test SELECT *
Write-Host ""
Write-Host "Testing SELECT * FROM users..." -ForegroundColor Yellow
$result = CallRpc "execute" "SELECT * FROM users"
if ($result.result.rows) {
    Write-Host "SUCCESS: Returned $($result.result.rows.Count) rows" -ForegroundColor Green
    foreach ($row in $result.result.rows) {
        Write-Host "  Row: $row"
    }
} else {
    Write-Host "FAILED: No rows returned" -ForegroundColor Red
    Write-Host "Result: $($result | ConvertTo-Json)"
}

# Test SELECT with WHERE
Write-Host ""
Write-Host "Testing SELECT name FROM users WHERE age > 25..." -ForegroundColor Yellow
$result2 = CallRpc "execute" "SELECT name FROM users WHERE age > 25"
if ($result2.result.rows) {
    Write-Host "SUCCESS: Returned $($result2.result.rows.Count) rows" -ForegroundColor Green
    foreach ($row in $result2.result.rows) {
        Write-Host "  Row: $row"
    }
} else {
    Write-Host "FAILED: No rows returned" -ForegroundColor Red
}

Write-Host ""
Write-Host "=== Test Complete ===" -ForegroundColor Cyan

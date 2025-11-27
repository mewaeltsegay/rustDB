$url = "http://localhost:8000"

Write-Host "=== Comprehensive SELECT Test ===" -ForegroundColor Cyan
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

# Create table with fresh name to avoid conflicts
$tableName = "test_users_" + (Get-Random)
Write-Host "Creating test table '$tableName'..." -ForegroundColor Yellow
CallRpc "execute" "CREATE TABLE $tableName (id INT, name TEXT, age INT)" > $null

# Insert data
Write-Host "Inserting test data..." -ForegroundColor Yellow
$inserts = @(
    "INSERT INTO $tableName VALUES (1, 'Alice', 30)",
    "INSERT INTO $tableName VALUES (2, 'Bob', 25)",
    "INSERT INTO $tableName VALUES (3, 'Charlie', 35)",
    "INSERT INTO $tableName VALUES (4, 'Diana', 28)"
)

foreach ($insert in $inserts) {
    CallRpc "execute" $insert > $null
    Write-Host "  ✓ Inserted"
}

# Test SELECT *
Write-Host ""
Write-Host "Test 1: SELECT * FROM $tableName" -ForegroundColor Yellow
$result = CallRpc "execute" "SELECT * FROM $tableName"
if ($result.result.rows) {
    Write-Host "SUCCESS: Returned $($result.result.rows.Count) rows" -ForegroundColor Green
    foreach ($i in 0..($result.result.rows.Count - 1)) {
        Write-Host "  Row $($i+1): $($result.result.rows[$i])"
    }
} else {
    Write-Host "FAILED: No rows returned" -ForegroundColor Red
}

# Test SELECT with WHERE (age > 28)
Write-Host ""
Write-Host "Test 2: SELECT * FROM $tableName WHERE age > 28" -ForegroundColor Yellow
$result2 = CallRpc "execute" "SELECT * FROM $tableName WHERE age > 28"
if ($result2.result.rows) {
    Write-Host "SUCCESS: Returned $($result2.result.rows.Count) rows" -ForegroundColor Green
    foreach ($row in $result2.result.rows) {
        Write-Host "  Row: $row"
    }
} else {
    Write-Host "FAILED: No rows returned" -ForegroundColor Red
}

# Test SELECT with column selection
Write-Host ""
Write-Host "Test 3: SELECT name, age FROM $tableName WHERE age >= 30" -ForegroundColor Yellow
$result3 = CallRpc "execute" "SELECT name, age FROM $tableName WHERE age >= 30"
if ($result3.result.rows) {
    Write-Host "SUCCESS: Returned $($result3.result.rows.Count) rows" -ForegroundColor Green
    foreach ($row in $result3.result.rows) {
        Write-Host "  Row: $row"
    }
} else {
    Write-Host "FAILED: No rows returned" -ForegroundColor Red
}

Write-Host ""
Write-Host "=== All Tests Complete ===" -ForegroundColor Cyan

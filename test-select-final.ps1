$url = "http://localhost:8000"

Write-Host "=== Comprehensive SELECT Test ===" -ForegroundColor Cyan
Write-Host ""

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

$tableName = "test_users_" + (Get-Random)
Write-Host "Creating test table '$tableName'..." -ForegroundColor Yellow
CallRpc "execute" "CREATE TABLE $tableName (id INT, name TEXT, age INT)" | Out-Null

Write-Host "Inserting test data..." -ForegroundColor Yellow
CallRpc "execute" "INSERT INTO $tableName VALUES (1, 'Alice', 30)" | Out-Null
CallRpc "execute" "INSERT INTO $tableName VALUES (2, 'Bob', 25)" | Out-Null
CallRpc "execute" "INSERT INTO $tableName VALUES (3, 'Charlie', 35)" | Out-Null
CallRpc "execute" "INSERT INTO $tableName VALUES (4, 'Diana', 28)" | Out-Null

Write-Host ""
Write-Host "Test 1: SELECT all rows" -ForegroundColor Yellow
$result = CallRpc "execute" "SELECT * FROM $tableName"
if ($result.result.rows) {
    Write-Host "SUCCESS: Returned $($result.result.rows.Count) rows" -ForegroundColor Green
    foreach ($row in $result.result.rows) {
        Write-Host "  - $row"
    }
} else {
    Write-Host "FAILED: No rows" -ForegroundColor Red
}

Write-Host ""
Write-Host "Test 2: SELECT with WHERE" -ForegroundColor Yellow
$whereQuery = "SELECT * FROM $tableName WHERE age" + ">" + "28"
$result2 = CallRpc "execute" $whereQuery
if ($result2.result.rows) {
    Write-Host "SUCCESS: Returned $($result2.result.rows.Count) rows" -ForegroundColor Green
    foreach ($row in $result2.result.rows) {
        Write-Host "  - $row"
    }
}

Write-Host ""
Write-Host "=== Tests Complete ===" -ForegroundColor Cyan

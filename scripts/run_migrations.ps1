param(
    [Parameter(Mandatory = $false)]
    [string]$DbHost,

    [Parameter(Mandatory = $false)]
    [int]$DbPort,

    [Parameter(Mandatory = $false)]
    [string]$DbName,

    [Parameter(Mandatory = $false)]
    [string]$DbUser,

    [Parameter(Mandatory = $false)]
    [string]$DbPassword,

    [Parameter(Mandatory = $false)]
    [string]$MigrationsDir,

    [Parameter(Mandatory = $false)]
    [string]$EnvFile
)

$ErrorActionPreference = "Stop"

function Import-DotEnv {
    param([string]$Path)

    if ([string]::IsNullOrWhiteSpace($Path) -or -not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        return
    }

    $lines = Get-Content -LiteralPath $Path
    foreach ($raw in $lines) {
        $line = $raw.Trim()
        if ([string]::IsNullOrWhiteSpace($line)) { continue }
        if ($line.StartsWith("#")) { continue }

        $parts = $line -split '=', 2
        if ($parts.Count -ne 2) { continue }

        $key = $parts[0].Trim()
        $value = $parts[1].Trim()
        if ($value.Length -ge 2) {
            if (($value.StartsWith('"') -and $value.EndsWith('"')) -or ($value.StartsWith("'") -and $value.EndsWith("'"))) {
                $value = $value.Substring(1, $value.Length - 2)
            }
        }

        if (-not [string]::IsNullOrWhiteSpace($key)) {
            [System.Environment]::SetEnvironmentVariable($key, $value, "Process")
        }
    }
}

function Get-FirstEnvValue {
    param([string[]]$Names)
    foreach ($name in $Names) {
        $value = [System.Environment]::GetEnvironmentVariable($name, "Process")
        if (-not [string]::IsNullOrWhiteSpace($value)) {
            return $value
        }
    }
    return $null
}

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$projectRoot = Split-Path -Parent $scriptDir
if ([string]::IsNullOrWhiteSpace($EnvFile)) {
    $EnvFile = Join-Path $projectRoot ".env"
}

Import-DotEnv -Path $EnvFile

if ([string]::IsNullOrWhiteSpace($DbHost)) {
    $DbHost = Get-FirstEnvValue -Names @("DB_HOST", "FFS_DB_HOST")
}
if ($DbPort -le 0) {
    $rawPort = Get-FirstEnvValue -Names @("DB_PORT", "FFS_DB_PORT")
    if (-not [string]::IsNullOrWhiteSpace($rawPort)) {
        $parsedPort = 0
        if ([int]::TryParse($rawPort, [ref]$parsedPort)) {
            $DbPort = $parsedPort
        }
    }
}
if ([string]::IsNullOrWhiteSpace($DbName)) {
    $DbName = Get-FirstEnvValue -Names @("DB_NAME", "FFS_DB_NAME")
}
if ([string]::IsNullOrWhiteSpace($DbUser)) {
    $DbUser = Get-FirstEnvValue -Names @("DB_USER", "FFS_DB_USER")
}
if ([string]::IsNullOrWhiteSpace($DbPassword)) {
    $DbPassword = Get-FirstEnvValue -Names @("DB_PASSWORD", "FFS_DB_PASSWORD", "MYSQL_PWD")
}

if ([string]::IsNullOrWhiteSpace($MigrationsDir)) {
    $MigrationsDir = Get-FirstEnvValue -Names @("MIGRATIONS_DIR")
}
if ([string]::IsNullOrWhiteSpace($MigrationsDir)) {
    $MigrationsDir = Join-Path $projectRoot "migrations"
}

if ([string]::IsNullOrWhiteSpace($DbHost) -or $DbPort -le 0 -or [string]::IsNullOrWhiteSpace($DbName) -or [string]::IsNullOrWhiteSpace($DbUser)) {
    Write-Error "missing database config. Provide args or set .env vars: DB_HOST, DB_PORT, DB_NAME, DB_USER (also supports FFS_DB_* aliases)."
}

if (-not (Test-Path -LiteralPath $MigrationsDir -PathType Container)) {
    Write-Error "migrations directory not found: $MigrationsDir"
}

try {
    Get-Command mysql -ErrorAction Stop | Out-Null
} catch {
    Write-Error "mysql command not found. Please install MySQL client and ensure 'mysql' is in PATH."
}

if (-not [string]::IsNullOrWhiteSpace($DbPassword)) {
    $env:MYSQL_PWD = $DbPassword
} else {
    Remove-Item Env:MYSQL_PWD -ErrorAction SilentlyContinue
}

$migrationFiles = Get-ChildItem -LiteralPath $MigrationsDir -Filter *.sql -File | Sort-Object Name

if ($migrationFiles.Count -eq 0) {
    Write-Host "no .sql files found in $MigrationsDir"
    exit 0
}

Write-Host "starting migrations to $DbName@$DbHost`:$DbPort"

foreach ($file in $migrationFiles) {
    Write-Host "running: $($file.FullName)"
    Get-Content -LiteralPath $file.FullName -Raw |
        & mysql "--host=$DbHost" "--port=$DbPort" "--user=$DbUser" $DbName

    if ($LASTEXITCODE -ne 0) {
        Write-Error "migration failed: $($file.FullName)"
    }
}

Write-Host "all migrations completed successfully"

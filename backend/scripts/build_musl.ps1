param(
    [string]$Target = "x86_64-unknown-linux-musl",
    [switch]$InstallDeps
)

$ErrorActionPreference = "Stop"

function Ensure-Command {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Name,
        [Parameter(Mandatory = $true)]
        [string]$InstallHint
    )

    if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
        throw "Missing required command '$Name'. $InstallHint"
    }
}

function Ensure-Zig {
    if (Get-Command zig -ErrorAction SilentlyContinue) {
        return
    }

    if (-not $InstallDeps) {
        throw "zig is not installed. Re-run with -InstallDeps, or install manually: winget install zig.zig"
    }

    Ensure-Command -Name winget -InstallHint "Install App Installer (winget) first, or install zig manually."

    Write-Host "Installing zig via winget..."
    winget install zig.zig --accept-source-agreements --accept-package-agreements
    # winget returns a non-zero exit code when the package is already installed
    # ("No applicable update found"). Reset it so the script does not abort.
    $global:LASTEXITCODE = 0

    if (-not (Get-Command zig -ErrorAction SilentlyContinue)) {
        # Current shell might not pick up PATH changes immediately.
        $machinePath = [System.Environment]::GetEnvironmentVariable("Path", "Machine")
        $userPath = [System.Environment]::GetEnvironmentVariable("Path", "User")
        $env:Path = "$machinePath;$userPath"
    }

    Ensure-Command -Name zig -InstallHint "zig installation failed."
}

function Ensure-CargoZigbuild {
    if (Get-Command cargo-zigbuild -ErrorAction SilentlyContinue) {
        return
    }

    if (-not $InstallDeps) {
        throw "cargo-zigbuild is not installed. Re-run with -InstallDeps, or run: cargo install cargo-zigbuild"
    }

    Write-Host "Installing cargo-zigbuild..."
    cargo install cargo-zigbuild
    Ensure-Command -Name cargo-zigbuild -InstallHint "cargo-zigbuild installation failed."
}

Write-Host "Ensuring Rust target '$Target' is installed..."
rustup target add $Target | Out-Host

Ensure-Zig
Ensure-CargoZigbuild

Write-Host "Building release binary for $Target..."
cargo zigbuild --release --target $Target

Write-Host "Build completed. Binary path: backend/target/$Target/release"

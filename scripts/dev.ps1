param(
  [switch]$SkipSetup
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$nodeModules = Join-Path $repoRoot "node_modules"

function Assert-Command {
  param(
    [string]$Name,
    [string]$InstallHint
  )

  if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
    throw "Missing required command '$Name'. $InstallHint"
  }
}

Set-Location $repoRoot

if (-not $SkipSetup -and -not (Test-Path $nodeModules)) {
  & (Join-Path $PSScriptRoot "setup.ps1")
}

Assert-Command "node" "Install Node.js 22 LTS or newer, then reopen PowerShell."
Assert-Command "npm" "Install Node.js 22 LTS or newer, then reopen PowerShell."
Assert-Command "rustc" "Install Rust with the stable MSVC toolchain from https://rustup.rs/."
Assert-Command "cargo" "Install Rust with the stable MSVC toolchain from https://rustup.rs/."

Write-Host "Starting Question Scan desktop development app..."
& npm run tauri dev

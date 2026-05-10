param(
  [switch]$SkipNpmInstall,
  [switch]$SkipCargoFetch
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")

function Write-Step {
  param([string]$Message)
  Write-Host ""
  Write-Host "==> $Message"
}

function Assert-Command {
  param(
    [string]$Name,
    [string]$InstallHint
  )

  if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
    throw "Missing required command '$Name'. $InstallHint"
  }
}

function Invoke-External {
  param(
    [string]$Label,
    [scriptblock]$Command
  )

  Write-Step $Label
  & $Command
  if ($LASTEXITCODE -ne 0) {
    throw "$Label failed with exit code $LASTEXITCODE."
  }
}

Set-Location $repoRoot

Write-Step "Checking required commands"
Assert-Command "node" "Install Node.js 22 LTS or newer, then reopen PowerShell."
Assert-Command "npm" "Install Node.js 22 LTS or newer, then reopen PowerShell."
Assert-Command "rustc" "Install Rust with the stable MSVC toolchain from https://rustup.rs/."
Assert-Command "cargo" "Install Rust with the stable MSVC toolchain from https://rustup.rs/."

Write-Host "node:  $(& node --version)"
Write-Host "npm:   $(& npm --version)"
Write-Host "rustc: $(& rustc --version)"
Write-Host "cargo: $(& cargo --version)"

if (-not $SkipNpmInstall) {
  Invoke-External "Installing npm dependencies" { npm install }
}

if (-not $SkipCargoFetch) {
  Push-Location (Join-Path $repoRoot "src-tauri")
  try {
    Invoke-External "Fetching Rust dependencies" { cargo fetch }
  } finally {
    Pop-Location
  }
}

Write-Step "Setup complete"
Write-Host "Run ./scripts/dev.ps1 to start the desktop development app."

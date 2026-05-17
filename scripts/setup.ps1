# 开发环境初始化脚本
# 用途：新开发者首次克隆仓库后运行，检查必要工具并安装项目依赖。
# 支持参数：
#   -SkipNpmInstall  跳过 npm install
#   -SkipCargoFetch  跳过 cargo fetch

param(
  [switch]$SkipNpmInstall,
  [switch]$SkipCargoFetch
)

# 启用严格模式，遇到错误立即终止
Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

# 定位仓库根目录
$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")

# 输出带格式的步骤提示
function Write-Step {
  param([string]$Message)
  Write-Host ""
  Write-Host "==> $Message"
}

# 检查指定命令是否存在于 PATH 中，缺失则抛出错误并给出安装提示
function Assert-Command {
  param(
    [string]$Name,
    [string]$InstallHint
  )

  if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
    throw "Missing required command '$Name'. $InstallHint"
  }
}

# 执行外部命令并检查退出码，失败则抛出错误
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

# 检查 node、npm、rustc、cargo 四个核心命令是否已安装
Write-Step "Checking required commands"
Assert-Command "node" "Install Node.js 22 LTS or newer, then reopen PowerShell."
Assert-Command "npm" "Install Node.js 22 LTS or newer, then reopen PowerShell."
Assert-Command "rustc" "Install Rust with the stable MSVC toolchain from https://rustup.rs/."
Assert-Command "cargo" "Install Rust with the stable MSVC toolchain from https://rustup.rs/."

# 打印当前工具版本，方便排查环境差异
Write-Host "node:  $(& node --version)"
Write-Host "npm:   $(& npm --version)"
Write-Host "rustc: $(& rustc --version)"
Write-Host "cargo: $(& cargo --version)"

# 安装前端 Node 依赖（根据 package-lock.json）
if (-not $SkipNpmInstall) {
  Invoke-External "Installing npm dependencies" { npm install }
}

# 进入 src-tauri 目录拉取 Rust 依赖
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

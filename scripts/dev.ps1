# 开发环境启动脚本
# 用途：日常开发时一键启动 Tauri 桌面应用。
# 行为：如果 node_modules 不存在，自动先调用 setup.ps1 初始化；然后启动开发服务器。
# 支持参数：
#   -SkipSetup  跳过自动 setup（即使 node_modules 缺失也不运行 setup.ps1）

param(
  [switch]$SkipSetup
)

# 启用严格模式，遇到错误立即终止
Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

# 定位仓库根目录和 node_modules 路径
$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$nodeModules = Join-Path $repoRoot "node_modules"

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

Set-Location $repoRoot

# 如果 node_modules 缺失且未跳过 setup，自动运行环境初始化
if (-not $SkipSetup -and -not (Test-Path $nodeModules)) {
  & (Join-Path $PSScriptRoot "setup.ps1")
}

# 再次确认核心命令可用（setup 后或用户已自行安装）
Assert-Command "node" "Install Node.js 22 LTS or newer, then reopen PowerShell."
Assert-Command "npm" "Install Node.js 22 LTS or newer, then reopen PowerShell."
Assert-Command "rustc" "Install Rust with the stable MSVC toolchain from https://rustup.rs/."
Assert-Command "cargo" "Install Rust with the stable MSVC toolchain from https://rustup.rs/."

# 启动 Tauri 开发模式（同时启动 Vite 前端 dev server 和 Rust 后端）
Write-Host "Starting Question Scan desktop development app..."
& npm run tauri dev

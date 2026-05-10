# Question Scan

Question Scan is a Tauri desktop tool for authorized algorithm practice, self-testing, open problem environments, and personal workflow use. The MVP uses React, TypeScript, Vite, Tailwind CSS, and a Rust/Tauri backend.

The project does not add hidden exam use, proctoring evasion, screen-share evasion, automatic third-party submission, automatic answer filling, or browser credential access.

## Quick Start

Use GitHub as the source of truth, then use the repository scripts for setup and startup.

```powershell
git clone git@github.com:Summus1999/Question_scan.git
cd Question_scan
powershell -ExecutionPolicy Bypass -File .\scripts\setup.ps1
powershell -ExecutionPolicy Bypass -File .\scripts\dev.ps1
```

After the first setup, daily startup is:

```powershell
.\scripts\dev.ps1
```

If local script execution is blocked, use:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\dev.ps1
```

## Environment

The stage 1 foundation was verified with:

- Node.js `v25.7.0`
- npm `11.10.1`
- rustc `1.92.0`
- cargo `1.92.0`
- Tauri 2.x
- Windows desktop development environment

Recommended for new developers:

- Node.js 22 LTS or newer
- Stable Rust MSVC toolchain
- Microsoft C++ Build Tools
- Microsoft Edge WebView2 Runtime
- PowerShell 5.1 or PowerShell 7

More detail is in [docs/developer-setup.md](docs/developer-setup.md).

## Commands

```powershell
npm run setup:win
npm run start:win
npm run dev:desktop
npm run lint
npm test -- --run
npm run build
```

Rust tests run from `src-tauri`:

```powershell
cd src-tauri
cargo test
```

## Development Branches

- `main` is the stable branch for verified work that other developers can pull.
- `feature/question-scan-mvp` is the MVP development branch used for incremental feature work.

When a parent task is completed and verified, merge or fast-forward it into `main`, then push the branch so other developers can pull the latest stable environment.

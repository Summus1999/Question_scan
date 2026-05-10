# Developer Setup

This document records the current development environment for Question Scan and the supported one-command startup path for new developers.

## Recommended Path

Use GitHub as the source of truth and keep setup scripts in the repository. A new developer should only need to clone the repo, run the setup script once, and then use the dev script for daily work.

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

If PowerShell blocks local scripts, use the `powershell -ExecutionPolicy Bypass -File ...` form above.

## Verified Local Versions

These are the versions used when the stage 1 Tauri foundation was verified:

- Windows desktop development environment
- Node.js `v25.7.0`
- npm `11.10.1`
- rustc `1.92.0`
- cargo `1.92.0`
- Tauri CLI from `@tauri-apps/cli` in `package-lock.json`
- Tauri Rust crate from `src-tauri/Cargo.lock`

Recommended minimums for other developers:

- Node.js 22 LTS or newer
- npm bundled with the chosen Node.js version
- Stable Rust MSVC toolchain
- Microsoft C++ Build Tools for Windows desktop builds
- Microsoft Edge WebView2 Runtime
- PowerShell 5.1 or PowerShell 7

## What The Scripts Do

`scripts/setup.ps1`:

- Checks `node`, `npm`, `rustc`, and `cargo`.
- Prints the local tool versions.
- Runs `npm install` using `package-lock.json`.
- Runs `cargo fetch` from `src-tauri` to fetch Rust dependencies.

`scripts/dev.ps1`:

- Runs setup automatically if `node_modules` is missing.
- Checks the required commands.
- Starts the Tauri development app with `npm run tauri dev`.

The npm aliases are also available:

```powershell
npm run setup:win
npm run start:win
npm run dev:desktop
```

## Manual Commands

Use these commands when debugging a specific layer:

```powershell
npm install
npm run lint
npm test -- --run
npm run build
cd src-tauri
cargo test
cd ..
npm run tauri dev
```

## Tracked Configuration

The repository tracks the environment configuration needed for reproducible startup:

- `package.json` and `package-lock.json` for Node dependencies and scripts.
- `src-tauri/Cargo.toml` and `src-tauri/Cargo.lock` for Rust dependencies.
- `vite.config.ts`, `tsconfig.json`, and `biome.json` for frontend build, typing, linting, and formatting.
- `src-tauri/tauri.conf.json` and `src-tauri/capabilities/default.json` for Tauri metadata and command permissions.
- `.gitignore` for generated folders such as `node_modules`, `dist`, `src-tauri/target`, and generated Tauri permission files.

## Troubleshooting

If `node` or `npm` is missing, install Node.js and reopen PowerShell so `PATH` is refreshed.

If `rustc` or `cargo` is missing, install Rust from rustup with the stable MSVC toolchain.

If linking fails on Windows, install Microsoft C++ Build Tools and make sure the MSVC toolchain is available.

If the Tauri window does not open, check that Microsoft Edge WebView2 Runtime is installed.

If port `1420` is already used, stop the process using that port before running `scripts/dev.ps1` again.

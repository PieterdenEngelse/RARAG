# Windows Service Setup (WinSW)

This folder provides a template to run the Agentic RAG API Server as a Windows service using [WinSW](https://github.com/winsw/winsw).

## Contents
- `winsw/ag.xml` — WinSW XML configuration template

## Prerequisites
- Windows 10/11 or Server 2019+
- .NET Framework (for WinSW v2) or .NET 6+ (for WinSW v3), see WinSW releases
- Place `ag.exe` (built with Cargo for Windows) and `WinSW.exe` in the same directory, e.g., `C:\ag`

## Build the binary (on Windows)
- Install Rust toolchain for Windows (MSVC): https://rustup.rs/
- Build release:
  ```powershell
  cargo build --release
  ```
- Copy the binary to your service folder:
  ```powershell
  copy target\release\ag.exe C:\ag\ag.exe
  ```

## Configure WinSW
- Download `WinSW-x.y.z.exe` and rename to `ag-service.exe` (or keep original name)
- Copy `ops\windows\winsw\ag.xml` to the same folder (e.g., `C:\ag\ag.xml`)
- Adjust `ag.xml`:
  - `<executable>`: set to `C:\ag\ag.exe` (or relative `%BASE%\ag.exe` if in same folder)
  - `<workingdirectory>`: e.g., `C:\ag\data` (ensure this exists and is writable)
  - `<env>`: set your environment variables
  - `RATE_LIMIT_ROUTES_FILE`: set absolute path to your JSON or YAML rules file, e.g., `C:\ag\config\rl-routes.json`

## Install service
From an elevated PowerShell:
```powershell
cd C:\ag
# If you renamed WinSW binary, use that name below
./ag-service.exe install
./ag-service.exe start
```

Check status and logs:
```powershell
./ag-service.exe status
./ag-service.exe status --verbose
# Logs are under C:\ag\logs by default
```

## Update configuration
- Stop service: `./ag-service.exe stop`
- Edit `ag.xml` or environment variables, rules files, etc.
- Start service: `./ag-service.exe start`

## Uninstall service
```powershell
./ag-service.exe stop
./ag-service.exe uninstall
```

## Notes
- The application uses relative upload directory `documents` under the working directory.
- PathManager defaults to `%USERPROFILE%\.local\share\ag` on Windows unless overridden — you can control behavior by environment variables.
- For reverse proxy scenarios on Windows, set `TRUST_PROXY=true` only if headers are trustworthy.
- YAML rules require building with the `rl_yaml` feature; otherwise use JSON for `RATE_LIMIT_ROUTES_FILE`.

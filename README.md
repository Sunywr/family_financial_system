# family_financial_system

HFS (Household Financial System) is an independent Rust + Vue 3 household accounting system.

## Structure

- `backend/`: Axum API and scheduler entrypoints
- `frontend/`: Vue 3 + TypeScript + Element Plus client
- `migrations/`: MySQL schema scripts
- `deploy/`: deployment examples
- `docs/`: project notes

## Backend

```powershell
cd backend
cargo run -- serve
```

Health check:

```text
GET /api/health
```

Optional scheduler process:

```powershell
cd backend
cargo run -- scheduler
```

Linux musl release build (from Windows):

```powershell
cd backend
powershell -ExecutionPolicy Bypass -File .\scripts\build_musl.ps1 -InstallDeps
```

If dependencies are already installed, you can skip auto-install:

```powershell
cd backend
powershell -ExecutionPolicy Bypass -File .\scripts\build_musl.ps1
```

Output binary location:

- `backend/target/x86_64-unknown-linux-musl/release/`

Configuration file:

- `backend/hfs.toml`
- `backend/hfs.prod.toml.example`
- set `legacy_database.url` when you want to compare imported data with the old Django `pfm` database in the migration audit page

## Frontend

```powershell
cd frontend
npm.cmd install
npm.cmd run dev
```

Production build:

```powershell
npm.cmd run build
```

Migration audit:

- `GET /api/migration-audit/summary`
- frontend page: `/migration-audit`

## Deployment

- `deploy/`: systemd, nginx, env, logrotate examples
- `scripts/`: release build and MySQL backup/restore helpers
- `docs/deployment.md`: single-host deployment guide for AnolisOS / RHEL compatible Linux

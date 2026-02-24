# FAM (Fresco Account Manager) -- Deployment Guide

This guide covers every aspect of deploying and running FAM, from a single-command Docker Compose start to full production hardening.

---

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Quick Start with Docker Compose](#quick-start-with-docker-compose)
3. [Environment Variables](#environment-variables)
4. [RSA Signing Keys](#rsa-signing-keys)
5. [Database](#database)
6. [Development Setup (Local)](#development-setup-local)
7. [Production Deployment](#production-deployment)
8. [Admin Setup](#admin-setup)
9. [Adding Projects](#adding-projects)
10. [BOINC Client Configuration](#boinc-client-configuration)
11. [Backup and Restore](#backup-and-restore)
12. [CI Pipeline](#ci-pipeline)
13. [Troubleshooting](#troubleshooting)

---

## Architecture Overview

| Layer    | Technology                    | Location              |
|----------|-------------------------------|-----------------------|
| Backend  | Rust, Axum 0.8               | `crates/fam-server`, `crates/fam-core` |
| Frontend | Vue 3 + TypeScript            | `frontend/`           |
| Database | PostgreSQL 16                 | docker-compose service or external |
| Build    | pnpm (frontend), Cargo (backend) | --                |
| Deploy   | Docker Compose                | `docker-compose.yml`  |

The compiled Docker image is a single artifact: it serves the Vue SPA as static files and exposes the Axum HTTP API on port **8080**.

---

## Quick Start with Docker Compose

This is the simplest way to get FAM running.

```bash
# Clone the repository and enter it
cd FAM

# (Optional) Copy and edit environment variables
cp .env.example .env

# Start everything
docker compose up -d
```

FAM is now available at **http://localhost:8080**.

Docker Compose starts two services:

| Service    | Image              | Port  | Notes                           |
|------------|--------------------|-------|---------------------------------|
| `postgres` | postgres:16-alpine | 5432  | Health-checked; data in `pgdata` volume |
| `fam`      | Built from Dockerfile | 8080 | Depends on healthy `postgres`   |

To stop:

```bash
docker compose down
```

To stop and remove the database volume (destroys all data):

```bash
docker compose down -v
```

---

## Environment Variables

All configuration is done through environment variables. Copy `.env.example` to `.env` and adjust as needed.

| Variable               | Default                                  | Description                                          |
|------------------------|------------------------------------------|------------------------------------------------------|
| `FAM_DATABASE_URL`     | `postgres://fam:fam@localhost:5432/fam`  | PostgreSQL connection string                         |
| `FAM_LISTEN_ADDR`      | `0.0.0.0:8080`                           | Address and port the server binds to                 |
| `FAM_SERVER_NAME`      | `Fresco Account Manager`                 | Name shown to BOINC clients and in the web UI        |
| `FAM_REPEAT_SEC`       | `86400`                                  | How often BOINC clients sync, in seconds (default: 24 h) |
| `FAM_PRIVATE_KEY_PATH` | `keys/private.pem`                       | Path to RSA private key                              |
| `FAM_PUBLIC_KEY_PATH`  | `keys/public.pem`                        | Path to RSA public key                               |
| `RUST_LOG`             | `info`                                   | Log level (`debug`, `info`, `warn`, `error`)         |

---

## RSA Signing Keys

FAM signs project-list replies so BOINC clients can verify authenticity. The keys are **critical infrastructure**.

### Auto-Generation

On first startup, if no key files exist at the configured paths, FAM automatically generates a **1024-bit RSA key pair** (1024-bit is a BOINC protocol requirement) and writes them to the `keys/` directory.

### Key Pinning Warning

> **CRITICAL: Once generated, NEVER replace or regenerate these keys.**
>
> BOINC clients pin the signing key the first time they sync with the account manager. If the key changes, **every previously-synced client will reject all future replies**. There is no recovery short of having every user manually re-attach to the account manager.

### What to Do

- After first startup, **back up** `keys/private.pem` and `keys/public.pem` to a secure, off-site location.
- If you migrate servers, copy the `keys/` directory to the new host before starting FAM.
- If deploying with Docker, mount a host directory or a named volume for `keys/` so the keys survive container recreation.

---

## Database

### Engine

PostgreSQL **16 or later** is required.

### Migrations

Migrations run **automatically** on every startup via `sqlx::migrate!`. There are 10 migration files in the `migrations/` directory. No manual migration step is needed.

### Docker Compose Defaults

The Docker Compose file creates a PostgreSQL 16 (Alpine) container with:

- User: `fam`
- Password: `fam`
- Database: `fam`
- Port: `5432` (mapped to host)
- Data volume: `pgdata`
- Built-in healthcheck so FAM waits for Postgres to be ready

---

## Development Setup (Local)

For active development you will want to run the backend and frontend separately with hot-reload.

### Prerequisites

| Tool             | Version   | Install                              |
|------------------|-----------|--------------------------------------|
| Rust toolchain   | stable    | https://rustup.rs                    |
| Node.js          | 22+       | https://nodejs.org                   |
| pnpm             | latest    | `npm install -g pnpm`               |
| PostgreSQL       | 16+       | System package or Docker             |

### 1. Start PostgreSQL

Either install PostgreSQL locally, or use the Compose service on its own:

```bash
docker compose up -d postgres
```

### 2. Configure Environment

```bash
cp .env.example .env
# Edit .env if your Postgres connection details differ
```

### 3. Run the Backend

From the project root:

```bash
cargo run
```

The server starts on `http://localhost:8080` (or whatever `FAM_LISTEN_ADDR` is set to). It serves the compiled frontend static files and the API.

### 4. Run the Frontend Dev Server

In a second terminal:

```bash
cd frontend
pnpm install
pnpm dev
```

Vite starts a dev server (typically on `http://localhost:5173`) with hot module replacement.

**Important -- API Proxy:** The Vite configuration (`frontend/vite.config.ts`) does **not** include a proxy for API requests. During development you have two options:

1. **Add a proxy to `vite.config.ts`** (recommended):

   ```ts
   // frontend/vite.config.ts
   import { defineConfig } from 'vite'
   import vue from '@vitejs/plugin-vue'

   export default defineConfig({
     plugins: [vue()],
     server: {
       proxy: {
         '/api': 'http://localhost:8080',
         '/rpc.php': 'http://localhost:8080',
         '/get_project_config.php': 'http://localhost:8080',
         '/health': 'http://localhost:8080',
       },
     },
   })
   ```

2. **Use the backend directly** at `http://localhost:8080` -- the backend already serves the built frontend files. Run `pnpm build` first, then access the backend URL.

---

## Production Deployment

### Using Docker Compose (Recommended)

1. **Clone the repository** onto the production server.

2. **Set secure credentials.** Edit `.env` or `docker-compose.yml`:

   ```bash
   FAM_DATABASE_URL=postgres://fam:A_STRONG_PASSWORD@postgres:5432/fam
   ```

   Update the Postgres service environment to match:

   ```yaml
   environment:
     POSTGRES_USER: fam
     POSTGRES_PASSWORD: A_STRONG_PASSWORD
     POSTGRES_DB: fam
   ```

3. **Start services:**

   ```bash
   docker compose up -d
   ```

4. **Verify:**

   ```bash
   curl http://localhost:8080/health
   ```

### Docker Image Details (Multi-Stage Build)

The Dockerfile uses three stages:

| Stage | Base Image         | Purpose                                           |
|-------|--------------------|---------------------------------------------------|
| 1     | node:22-alpine     | Installs pnpm, builds the Vue frontend            |
| 2     | rust:1.84-alpine   | Compiles the Rust backend (`SQLX_OFFLINE=true`)   |
| 3     | alpine:3.21        | Minimal runtime: binary + static files + migrations |

The final image exposes port **8080** and includes a healthcheck on `/health`.

### Reverse Proxy with TLS

In production, place a reverse proxy in front of FAM to terminate TLS. Example with **Caddy** (automatic HTTPS):

```
fam.example.com {
    reverse_proxy localhost:8080
}
```

Example with **nginx**:

```nginx
server {
    listen 443 ssl;
    server_name fam.example.com;

    ssl_certificate     /etc/letsencrypt/live/fam.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/fam.example.com/privkey.pem;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

### Production Checklist

- [ ] Strong, unique PostgreSQL password
- [ ] TLS via reverse proxy
- [ ] RSA keys backed up securely
- [ ] Database backups scheduled (see [Backup and Restore](#backup-and-restore))
- [ ] `RUST_LOG` set appropriately (`info` or `warn` for production)
- [ ] Firewall: only expose ports 80/443 publicly; keep 8080 and 5432 internal
- [ ] `keys/` directory mounted as a volume or bind-mount so keys persist across container rebuilds

---

## Admin Setup

### Promoting the First Admin

There is no default admin account. After registering the first user through the web UI, promote them via SQL:

```sql
UPDATE users SET is_admin = true WHERE id = 1;
```

If using Docker Compose:

```bash
docker compose exec postgres psql -U fam -d fam -c "UPDATE users SET is_admin = true WHERE id = 1;"
```

### Admin Panel

Once promoted, the admin panel is available at **/admin** in the web UI. From there, admins can add and manage BOINC projects.

---

## Adding Projects

### Via the Admin Panel (Recommended)

1. Log in as an admin user.
2. Navigate to **/admin**.
3. Add a new project with:
   - **Project URL** -- the URL of the BOINC project (include the trailing slash, e.g., `https://boinc.bakerlab.org/rosetta/`).
   - **Name** -- display name for the project.
   - **Description** -- short description shown to users.
4. The **URL signature** is computed automatically using the RSA private key.

### Via SQL (Advanced)

You can also seed projects directly in the database. Ensure you compute the correct RSA signature for the project URL, or let the application logic handle it.

---

## BOINC Client Configuration

### Connecting a Client

1. A user registers an account on the FAM web UI.
2. In the BOINC client (BOINC Manager or `boinccmd`):
   - Go to **Tools > Use Account Manager**.
   - Enter the FAM URL, e.g., `http://your-server:8080` (or the public HTTPS URL behind your reverse proxy).
   - Enter the username and password from step 1.
3. The client contacts FAM to fetch the project list and attaches to the configured projects.

### How It Works

| Endpoint                   | Method | Purpose                                          |
|----------------------------|--------|--------------------------------------------------|
| `/get_project_config.php`  | GET    | Returns XML identifying FAM as an account manager |
| `/rpc.php`                 | POST   | Main sync endpoint; BOINC clients call this on the `FAM_REPEAT_SEC` interval |

The client syncs every `FAM_REPEAT_SEC` seconds (default: 86400 = 24 hours). During each sync, FAM sends the signed project list and the client verifies the signature against the pinned public key.

---

## Backup and Restore

### What to Back Up

| Asset         | Location                         | Priority   |
|---------------|----------------------------------|------------|
| RSA keys      | `keys/private.pem`, `keys/public.pem` | **Critical** |
| Database      | PostgreSQL `fam` database        | **Critical** |
| `.env`        | Project root                     | Important  |

> **CRITICAL: Never lose the RSA private key.** If the private key is lost, all previously-synced BOINC clients will be unable to verify signed replies. There is no way to recover without having every user manually detach and re-attach.

### Database Backup

```bash
# If running via Docker Compose:
docker compose exec postgres pg_dump -U fam fam > fam_backup_$(date +%Y%m%d).sql

# If running PostgreSQL locally:
pg_dump -U fam fam > fam_backup_$(date +%Y%m%d).sql
```

### Database Restore

```bash
# Docker Compose:
docker compose exec -T postgres psql -U fam fam < fam_backup_20260225.sql

# Local:
psql -U fam fam < fam_backup_20260225.sql
```

### RSA Key Backup

Simply copy the `keys/` directory to secure, off-site storage:

```bash
cp -r keys/ /path/to/secure/backup/fam-keys/
```

---

## CI Pipeline

The GitHub Actions CI workflow (`.github/workflows/ci.yml`) runs the following checks:

### Backend

| Step                | Command                           | Notes                                |
|---------------------|-----------------------------------|--------------------------------------|
| Format check        | `cargo fmt --check`               | Enforces consistent Rust formatting  |
| Lint                | `cargo clippy`                    | Catches common mistakes              |
| Tests               | `cargo test`                      | Runs with `SQLX_OFFLINE=true`        |

### Frontend

| Step   | Command       | Notes                    |
|--------|---------------|--------------------------|
| Lint   | `pnpm lint`   | ESLint / style checks    |
| Build  | `pnpm build`  | Ensures production build succeeds |
| Test   | `pnpm test`   | Runs the test suite      |

---

## Troubleshooting

### Checking Logs

```bash
# All services
docker compose logs

# FAM only, follow mode
docker compose logs -f fam

# Last 100 lines
docker compose logs --tail 100 fam
```

Increase log verbosity by setting `RUST_LOG=debug` in your `.env` file and restarting.

### Health Check

```bash
curl http://localhost:8080/health
```

A healthy server returns a `200 OK` response.

### BOINC Client Cannot Connect

1. Verify the server is reachable from the client machine.
2. Check that `/get_project_config.php` returns valid XML containing `<account_manager/>`:

   ```bash
   curl http://your-server:8080/get_project_config.php
   ```

   You should see an XML response with an `<account_manager/>` element. If not, the server is not running or the URL is wrong.

3. Check firewall rules -- port 8080 (or 80/443 if behind a reverse proxy) must be open.

### Database Connection Errors

- Verify `FAM_DATABASE_URL` is correct.
- Ensure PostgreSQL is running and accepting connections.
- If using Docker Compose, ensure the `postgres` service is healthy:

  ```bash
  docker compose ps
  ```

### Container Fails to Start

```bash
docker compose logs fam
```

Common causes:
- PostgreSQL not ready yet (should be handled by healthcheck dependency, but check).
- Missing or corrupt RSA keys -- delete `keys/` and let FAM regenerate them (only safe if no clients have synced yet).
- Port 8080 already in use -- change `FAM_LISTEN_ADDR` or the Docker port mapping.

### Migrations Fail

Migrations run automatically. If they fail:
- Check that the database exists and the user has sufficient privileges.
- Look for error messages in the logs referencing specific migration files in `migrations/`.
- Ensure you are using PostgreSQL 16 or later.

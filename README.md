# Task API Service (Rust / Axum)

REST API microservice powering task management, AI-assisted goal planning, JWT-protected admin dashboards, and request audit logging — all backed by SQLite.

## Features

| Area | Highlights |
|---|---|
| **Task CRUD** | Create, list (filter/paginate/search), update, and delete tasks |
| **AI Goal Planner** | `POST /api/v1/tasks/plan` proxies to the AI orchestrator for LLM-generated sub-tasks |
| **JWT Authentication** | Bearer-token middleware; role-based admin gating (`AUTH_ENFORCED`) |
| **Admin Metrics** | Aggregated task stats, recent request logs, per-user activity |
| **Request Audit Log** | Every `/api/` call is persisted with subject, method, path, status, latency |
| **Service Info** | `GET /api/v1/info` exposes version and feature flags at runtime |
| **Health / Readiness** | `/health` (process alive) and `/ready` (DB connectivity check) |
| **Shared HTTP Client** | Single `reqwest::Client` in `AppState` — connection pooling across upstream calls |
| **Structured Errors** | Consistent `{ code, message, details }` envelope on every non-2xx response |
| **Load Testing** | k6 harness with baseline scenarios and quality thresholds |

## Tech Stack

| Layer | Technology |
|---|---|
| Language | Rust (edition 2024) |
| Web framework | Axum 0.8 |
| Database | SQLite via sqlx 0.8 (compile-time checked) |
| HTTP client | reqwest 0.12 (rustls) |
| Auth | jsonwebtoken 8.3 |
| Middleware | tower-http 0.6 (CORS, tracing) |
| CI | GitHub Actions (fmt, clippy, test) |
| Deployment | Fly.io (Docker) |

## Project Structure

```
src/
├── main.rs                    # Tokio entrypoint, env parsing
├── lib.rs                     # Facade — re-exports public API
└── lib/
    ├── app_state.rs           # AppState (SqlitePool + reqwest::Client)
    ├── auth.rs                # JWT validation, AuthClaims, middleware helpers
    ├── models.rs              # Request/response DTOs, DB row types
    ├── router.rs              # Route wiring, CORS, audit middleware
    ├── validation.rs          # Input normalisation and guard functions
    └── handlers/
        ├── mod.rs             # Handler barrel export
        ├── tasks.rs           # CRUD handlers (list, create, update, delete)
        ├── tasks_support.rs   # Query builder helpers
        ├── planner.rs         # AI goal-planning proxy
        ├── admin.rs           # Admin metrics, request logs, user activity
        ├── health.rs          # /health and /ready
        ├── info.rs            # /api/v1/info (version + features)
        └── shared.rs          # error_response, pagination, timeout utils
migrations/                    # SQLite schema evolution (sqlx)
tests/api_tasks.rs             # Black-box integration tests (tower one-shot)
load/k6_tasks.js               # k6 load-test harness
```

## Prerequisites

- Rust toolchain (`rustup`, `cargo`, `rustc`)
- Linux build tools (`build-essential`) — for cross-compilation or Docker builds

## Build & Run

```bash
cargo build            # debug
cargo build --release  # optimised

cargo run              # starts on http://0.0.0.0:3000 by default
```

Override defaults with environment variables:

```bash
HOST=127.0.0.1 PORT=8080 DATABASE_URL=sqlite://app.db cargo run
```

Migrations run automatically at startup.

## Environment Variables

| Variable | Default | Description |
|---|---|---|
| `HOST` | `0.0.0.0` | Bind address |
| `PORT` | `3000` | Bind port |
| `DATABASE_URL` | `sqlite://app.db` | SQLx-compatible SQLite URL |
| `AUTH_ENFORCED` | `false` | Enable JWT authentication middleware |
| `ALLOWED_ORIGINS` | *(permissive)* | Comma-separated CORS allowlist |
| `AI_ORCHESTRATOR_PLAN_URL` | `http://127.0.0.1:8081/plan` | Upstream planner endpoint |
| `AI_ORCHESTRATOR_TIMEOUT_SECONDS` | `15` | Upstream HTTP timeout |

## API Endpoints

| Method | Path | Auth | Description |
|---|---|---|---|
| `GET` | `/health` | — | Process liveness |
| `GET` | `/ready` | — | Database readiness (`SELECT 1`) |
| `GET` | `/api/v1/info` | — | Service version and feature flags |
| `GET` | `/api/v1/tasks` | Bearer | List tasks (filters: `limit`, `offset`, `completed`, `status`, `q`) |
| `POST` | `/api/v1/tasks` | Bearer | Create task |
| `PATCH` | `/api/v1/tasks/{id}` | Bearer | Update task fields |
| `DELETE` | `/api/v1/tasks/{id}` | Bearer | Delete task |
| `POST` | `/api/v1/tasks/plan` | Bearer | AI goal → sub-tasks |
| `GET` | `/api/v1/admin/metrics` | Admin | Aggregated task metrics |
| `GET` | `/api/v1/admin/requests` | Admin | Recent API request audit logs |
| `GET` | `/api/v1/admin/users` | Admin | Per-user activity summary |

### Error Envelope

All non-2xx responses follow:

```json
{
  "code": "STABLE_ERROR_CODE",
  "message": "Human-readable message",
  "details": { "optional": "metadata" }
}
```

## API Examples

```bash
# Create task
curl -X POST http://localhost:3000/api/v1/tasks \
  -H "Content-Type: application/json" \
  -d '{"title":"Design landing page"}'

# List with filters
curl "http://localhost:3000/api/v1/tasks?limit=20&offset=0&completed=false&q=design"

# Update task
curl -X PATCH http://localhost:3000/api/v1/tasks/1 \
  -H "Content-Type: application/json" \
  -d '{"completed":true}'

# Delete task
curl -X DELETE http://localhost:3000/api/v1/tasks/1

# Service info
curl http://localhost:3000/api/v1/info
```

## Test

```bash
cargo test
```

Integration tests in `tests/api_tasks.rs` run against isolated on-disk SQLite files per test case.

## Load Test (k6)

```bash
k6 run load/k6_tasks.js
```

Default profile: ramp to 10 VUs → sustain 20 VUs for 2 min → ramp down.
Thresholds: failure rate < 1 %, p95 latency < 300 ms.

## Lint & Format

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
```

## CI / CD

- **CI**: `.github/workflows/ci.yml` — fmt check, clippy (warnings denied), tests
- **Deploy**: `.github/workflows/deploy-fly.yml` — pushes to Fly.io on `main`

## Codebase Reading Guide

| File | Pattern | Why |
|---|---|---|
| `main.rs` | Composition root | Env parsing → state init → router → serve |
| `lib.rs` | Facade | Stable public surface; hides internal layout |
| `models.rs` | DTO / domain boundary | `serde` + `sqlx::FromRow` for compile-time alignment |
| `app_state.rs` | Shared state / DI | `SqlitePool` + `reqwest::Client`, cheap to clone |
| `router.rs` | Middleware pipeline | Route wiring, CORS, auth layers, audit logging |
| `handlers/` | Controller layer | Extractor-based input, `IntoResponse` polymorphism |
| `validation.rs` | Guard utilities | `Option<T>`-driven validation with ergonomic branching |
| `tests/api_tasks.rs` | Integration tests | Per-test DB isolation, `tower::ServiceExt` one-shot |

## Inspect Local Data

```bash
sqlite3 app.db ".tables"
sqlite3 app.db "SELECT id,title,completed,difficulty,goal,status FROM tasks ORDER BY id DESC LIMIT 20;"
sqlite3 app.db "SELECT id,occurred_at,subject,method,path,status_code,duration_ms FROM api_request_logs ORDER BY id DESC LIMIT 20;"
```

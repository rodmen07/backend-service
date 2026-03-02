# Rust Web Backend (Starter)

A minimal Rust backend for a web application, built with Axum.

## Prerequisites

- Rust toolchain (`rustup`, `cargo`, `rustc`)
- Linux build tools (`build-essential`)

## Build

```bash
cargo build
```

Release build:

```bash
cargo build --release
```

## Run

```bash
cargo run
```

Release run:

```bash
cargo run --release
```

The server starts on `http://0.0.0.0:3000` by default.

Override host/port/database with environment variables:

```bash
HOST=127.0.0.1 PORT=8080 DATABASE_URL=sqlite://app.db cargo run
```

`DATABASE_URL` defaults to `sqlite://app.db` and migrations run automatically at startup.

### LLM goal planning configuration

The backend now delegates goal planning to the `ai-orchestrator-service`.

Set the internal planner URL (optional):

```bash
AI_ORCHESTRATOR_PLAN_URL=http://127.0.0.1:8081/plan
```

If unset, the backend defaults to `http://127.0.0.1:8081/plan`.

Provider credentials are configured in `ai-orchestrator-service` (`OPENROUTER_API_KEY`, `OPENROUTER_MODEL`, etc.), not in this backend.

## API endpoints

- `GET /health` -> process liveness check
- `GET /ready` -> database readiness check
- `GET /api/v1/tasks` -> list tasks (`limit`, `offset`, `completed`, `q`)
- `POST /api/v1/tasks` -> create task
- `POST /api/v1/tasks/plan` -> generate composite tasks from a long-term goal (LLM wrapper)
- `PATCH /api/v1/tasks/{id}` -> update title/completed
- `DELETE /api/v1/tasks/{id}` -> delete task

### Health/readiness semantics

- `/health`: service process is up and can respond to HTTP requests.
- `/ready`: database connectivity check succeeded (`SELECT 1`) and service is ready for traffic.

### Validation invariants (v1)

- `title` is required (non-empty after trim)
- `title` max length is `120` characters
- Task list ordering is stable by `id ASC`

### Error envelope

All non-2xx API errors follow:

```json
{
	"code": "STABLE_ERROR_CODE",
	"message": "Human-readable message",
	"details": { "optional": "metadata" }
}
```

### Authentication stance (v1)

- Authentication is **not enforced** in v1.
- Future auth interface is frozen now to avoid client churn:
	- Header: `Authorization`
	- Scheme: `Bearer`
	- Format: `Authorization: Bearer <token>`

## API examples

Create task:

```bash
curl -X POST http://localhost:3000/api/v1/tasks \
	-H "Content-Type: application/json" \
	-d '{"title":"Design landing page"}'
```

List tasks:

```bash
curl http://localhost:3000/api/v1/tasks
```

List tasks with filters:

```bash
curl "http://localhost:3000/api/v1/tasks?limit=20&offset=0&completed=false&q=design"
```

Update task:

```bash
curl -X PATCH http://localhost:3000/api/v1/tasks/1 \
	-H "Content-Type: application/json" \
	-d '{"completed":true}'
```

Delete task:

```bash
curl -X DELETE http://localhost:3000/api/v1/tasks/1
```

## Test

```bash
cargo test
```

## Load test (k6)

Load-test harness: `load/k6_tasks.js`

### Baseline scenario

- Creates tasks
- Lists tasks with filters
- Updates task completion state
- Deletes every other created task

### Default target profile

- Ramp to 10 VUs over 30s
- Sustain 20 VUs for 2m
- Ramp down to 0 over 30s

### Quality thresholds

- failure rate: `< 1%`
- p95 latency: `< 300ms`

### Run

```bash
k6 run load/k6_tasks.js
```

Optional environment overrides:

```bash
BASE_URL=http://localhost:3000 TASK_TITLE_PREFIX=LoadTask k6 run load/k6_tasks.js
```

## Lint and format

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
```

## CI

GitHub Actions workflow is defined in `.github/workflows/ci.yml` and runs:

- formatting check
- clippy with warnings denied
- tests

## Automated releases

Release automation is configured with Release Please:

- workflow: `.github/workflows/release-please.yml`
- config: `.release-please-config.json`
- manifest: `.release-please-manifest.json`

Use Conventional Commits so versions/changelog are computed correctly:

- `feat: ...` for new features (minor bump)
- `fix: ...` for bug fixes (patch bump)
- `feat!: ...` or `BREAKING CHANGE:` in body/footer for major bump

## Release profile

Release defaults are configured in `Cargo.toml`:

- LTO enabled
- single codegen unit
- symbols stripped
- panic abort strategy

## Contributing

- PR template: `.github/pull_request_template.md`
- Branch protection setup: `.github/branch-protection-checklist.md`
- Code owners: `.github/CODEOWNERS`

Current solo-maintainer PR rules:

- PRs and required CI checks are enforced on `main`
- Required approvals are set to `0` (self-review limitation on GitHub)

## How to read this tutorial codebase

Use this map to connect files to backend design patterns and Rust-specific practices.

- `src/main.rs`
	- Pattern: Composition root / application bootstrap
	- Rust nuance: explicit env parsing, typed socket address, async runtime startup
- `src/lib.rs`
	- Pattern: Facade / public API boundary
	- Rust nuance: re-exporting stable crate surface while hiding internal module layout
- `src/lib/app_state.rs`
	- Pattern: Shared state container + dependency injection
	- Rust nuance: cloned `SqlitePool` handle and async DB initialization with migrations
- `src/lib/router.rs`
	- Pattern: Router composition and middleware pipeline
	- Rust nuance: Axum route typing and middleware layering (`CORS`, tracing)
- `src/lib/handlers.rs`
	- Pattern: Controller/handler layer
	- Rust nuance: extractor-based input handling, `IntoResponse` return polymorphism
- `src/lib/models.rs`
	- Pattern: DTO/domain schema boundary
	- Rust nuance: `serde` + `sqlx::FromRow` derives for compile-time type alignment
- `src/lib/validation.rs`
	- Pattern: Input normalization/validation utility layer
	- Rust nuance: `Option<T>`-driven validation flow for ergonomic error branching
- `migrations/`
	- Pattern: Schema evolution via versioned migrations
	- Rust nuance: startup migration execution through `sqlx::migrate!`
- `tests/api_tasks.rs`
	- Pattern: Black-box HTTP integration testing
	- Rust nuance: per-test SQLite isolation and one-shot router execution (`tower::ServiceExt`)

### Suggested reading order

- [ ] 1) Start with `src/main.rs` to understand runtime/bootstrap flow
- [ ] 2) Read `src/lib.rs` to see the facade and module boundaries
- [ ] 3) Review `src/lib/models.rs` for request/response and DB row shapes
- [ ] 4) Read `src/lib/app_state.rs` to understand DB lifecycle and migrations
- [ ] 5) Study `src/lib/router.rs` for route wiring and middleware composition
- [ ] 6) Walk through `src/lib/handlers.rs` for endpoint logic and persistence operations
- [ ] 7) Check `src/lib/validation.rs` for normalization and guard patterns
- [ ] 8) Inspect `tests/api_tasks.rs` to see end-to-end behavior validation

## Debug in VS Code

1. Open the Run and Debug view.
2. Choose **Debug Rust (GDB)**.
3. Start debugging.

## Roadmap TODOs

### Service definition alignment (n=1)

- [x] Adopt n=1 scope as **Tasks API microservice** (not tenant-routing service)
- [ ] Document potential v2 pivot path to tenant-header deterministic routing service

### Definition phase checklist (for this Tasks API)

- [ ] Define canonical request/response JSON schemas for each endpoint
- [ ] Define endpoint failure modes and exact status-code mapping
- [ ] Specify pagination/filter semantics precisely (defaults, max limit, ordering)
- [ ] Choose and document one load-bearing invariant for v1

#### Candidate invariant options

- [x] `title` must be non-empty and capped at max length `120`
- [x] `id` is immutable; PATCH only updates `title` and `completed`
- [x] List endpoint ordering is stable under pagination (`ORDER BY id ASC`)

### Core backend

- [x] Add SQLite persistence with `sqlx` (replace in-memory store)
- [x] Add database migrations and startup migration checks
- [x] Add strong request validation (required fields, length constraints)
- [x] Standardize API error responses (`code`, `message`, optional `details`)
- [x] Add pagination/filtering/sorting for task lists (`limit`, `offset`, `completed`, `q`)

### Contract and errors

- [x] Adopt standard error envelope (`code`, `message`, `details`)
- [ ] Document error codes and response examples per endpoint
- [ ] Add OpenAPI/Swagger spec for endpoint contracts

### Security and access

- [ ] Add authentication middleware (JWT or API key)
- [ ] Add user ownership rules for task access
- [ ] Tighten CORS by environment (dev vs production origins)

### Reliability and operations

- [ ] Add structured logging with request IDs and latency metrics
- [x] Split health checks into `/health` and `/ready` (DB readiness)
- [ ] Add rate limiting on write endpoints
- [x] Define `/health` semantics (process alive) and `/ready` semantics (DB + migrations)

### Product logic

- [ ] Add due dates, priorities, tags, and richer task status transitions
- [ ] Add audit fields (`created_at`, `updated_at`, `deleted_at`)
- [ ] Add soft delete semantics
- [ ] Add idempotency support for `POST` operations

### Auth stance (v1 decision)

- [x] Decide and document v1 auth stance: no auth, API key, or JWT
- [x] If postponed, freeze future auth header contract now to avoid client churn

### Quality and developer experience

- [x] Add HTTP integration tests for full API flows
- [ ] Add deterministic test database setup and seed fixtures
- [x] Add load-test harness (k6 or vegeta)
- [x] Define baseline load scenario (create/list/filter/patch/delete)
- [x] Define target metrics (RPS, duration, p95 latency)

### Deployment tracking (GitHub Pages context)

- [x] Decide and document architecture: GitHub Pages for frontend only, backend hosted separately
- [ ] Select backend host (for example Fly.io/Render/Railway/Azure) and define environment variables
- [ ] Add CORS config allowing the GitHub Pages frontend origin
- [ ] Add production deployment workflow for backend (build, migrate, deploy)
- [ ] Add frontend API base URL strategy for GitHub Pages (`production` vs `local`)

### Deployment architecture

- Frontend: GitHub Pages (static hosting)
- Backend API: separate service host (required for Axum server runtime)
- Integration: frontend calls hosted backend API via configured base URL

### Recommended implementation order

- [x] 1) Persistence + migrations
- [x] 2) Service contract + invariant definition
- [x] 3) Standardized error model
- [x] 4) Validation hardening (including invariant enforcement)
- [x] 5) Pagination/filtering
- [x] 6) Integration tests
- [x] 7) `/health` + `/ready` operational semantics
- [x] 8) Auth stance decision + interface freeze
- [x] 9) Load-test spec and harness

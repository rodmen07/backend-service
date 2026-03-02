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

## API endpoints

- `GET /health` -> health check
- `GET /api/v1/tasks` -> list tasks (`limit`, `offset`, `completed`, `q`)
- `POST /api/v1/tasks` -> create task
- `PATCH /api/v1/tasks/{id}` -> update title/completed
- `DELETE /api/v1/tasks/{id}` -> delete task

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

## Debug in VS Code

1. Open the Run and Debug view.
2. Choose **Debug Rust (GDB)**.
3. Start debugging.

## Roadmap TODOs

### Core backend

- [x] Add SQLite persistence with `sqlx` (replace in-memory store)
- [x] Add database migrations and startup migration checks
- [ ] Add strong request validation (required fields, length constraints)
- [ ] Standardize API error responses (`code`, `message`, optional `details`)
- [x] Add pagination/filtering/sorting for task lists (`limit`, `offset`, `completed`, `q`)

### Security and access

- [ ] Add authentication middleware (JWT or API key)
- [ ] Add user ownership rules for task access
- [ ] Tighten CORS by environment (dev vs production origins)

### Reliability and operations

- [ ] Add structured logging with request IDs and latency metrics
- [ ] Split health checks into `/health` and `/ready` (DB readiness)
- [ ] Add rate limiting on write endpoints

### Product logic

- [ ] Add due dates, priorities, tags, and richer task status transitions
- [ ] Add audit fields (`created_at`, `updated_at`, `deleted_at`)
- [ ] Add soft delete semantics
- [ ] Add idempotency support for `POST` operations

### Quality and developer experience

- [ ] Add HTTP integration tests for full API flows
- [ ] Add deterministic test database setup and seed fixtures
- [ ] Add OpenAPI/Swagger documentation for frontend integration

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
- [ ] 2) Auth middleware
- [x] 3) Pagination/filtering
- [ ] 4) Standardized error model
- [ ] 5) Integration tests

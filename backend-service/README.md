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

Override host/port with environment variables:

```bash
HOST=127.0.0.1 PORT=8080 cargo run
```

## API endpoints

- `GET /health` -> health check
- `GET /api/v1/tasks` -> list tasks
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

## Debug in VS Code

1. Open the Run and Debug view.
2. Choose **Debug Rust (GDB)**.
3. Start debugging.

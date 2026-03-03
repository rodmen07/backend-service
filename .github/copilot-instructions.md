# Backend Service Instructions (Detailed)

Use this file as the repository-specific implementation contract for AI-assisted changes.

## 1) Repository role

- This repository is the canonical API and persistence layer in the microservices workspace.
- Stack: Rust, Axum, SQLite, sqlx migrations.
- Owns task CRUD semantics, validation, and stable response contracts.

## 2) Service boundaries

- Do not add direct LLM/provider integrations in this repository.
- Goal planning must be delegated to ai-orchestrator-service over HTTP.
- Keep provider keys/config out of backend; orchestrator owns those concerns.

## 3) Runtime and configuration

- Default bind: 0.0.0.0:3000.
- Supported env vars: HOST, PORT, DATABASE_URL, AI_ORCHESTRATOR_PLAN_URL, ALLOWED_ORIGINS, AI_ORCHESTRATOR_TIMEOUT_SECONDS.
- Planner URL default when unset: http://127.0.0.1:8081/plan.
- DATABASE_URL defaults to sqlite://app.db unless explicitly overridden.
- Planner timeout default when unset: 15 seconds.
- If ALLOWED_ORIGINS is empty/unset, backend currently uses permissive CORS fallback.

## 4) API contracts (v1 stability required)

- GET /health: process liveness.
- GET /ready: readiness check with successful DB query semantics.
- GET /api/v1/tasks: list tasks with limit, offset, completed, q filters.
- POST /api/v1/tasks: create task.
- POST /api/v1/tasks/plan: plan composite tasks by delegating to orchestrator.
- PATCH /api/v1/tasks/{id}: partial updates for title/completed.
- DELETE /api/v1/tasks/{id}: delete task.

## 5) Validation and data invariants

- title is required, trimmed, and non-empty.
- title maximum length is 120 characters.
- Task ordering must remain stable by id ASC.
- Avoid silent normalization that changes external behavior without documentation.

## 6) Error model

- Preserve the non-2xx error envelope shape:
  - code: stable machine-readable error code.
  - message: human-readable summary.
  - details: optional metadata object.
- Do not return ad-hoc error shapes from handlers.

## 7) Auth posture

- v1 does not enforce authentication.
- Preserve future auth interface compatibility:
  - Authorization: Bearer <token>
- Avoid hard-coding assumptions that block adding auth middleware later.

## 8) Integration behavior with orchestrator

- Expect planner endpoint request: { goal: string }.
- Expect planner response: { tasks: string[] }.
- Use explicit timeout/error mapping when orchestrator is unavailable.
- Keep backend response contract stable even if planner provider behavior changes.

## 9) Code change guidance

- Prefer focused, minimal changes aligned with existing module boundaries.
- Keep handlers thin; place validation/logic in dedicated layers where appropriate.
- Preserve migration-driven schema evolution (no undocumented schema drift).
- Maintain idempotent, predictable list/query behavior.

## 10) Quality gates before completion

Run and pass:
- cargo fmt --all
- cargo clippy --all-targets --all-features -- -D warnings
- cargo test

## 11) Documentation synchronization

When changing contracts, env vars, or endpoint behavior:
- update README.md,
- update tests and examples/curl snippets when applicable,
- keep frontend-service and ai-orchestrator-service compatibility in scope.

## 12) Current code map (authoritative)

- Startup/bootstrap: `src/main.rs`
- Shared state + migration bootstrap: `src/lib/app_state.rs`
- Route wiring + middleware: `src/lib/router.rs`
- HTTP handler logic + planner delegation: `src/lib/handlers.rs`
- DTOs/error envelope: `src/lib/models.rs`
- Validation helpers and title constraints: `src/lib/validation.rs`
- Integration tests: `tests/api_tasks.rs`

## 13) Handler-level implementation constraints

- Keep root `/` mapped to liveness behavior unless explicitly changing public health semantics.
- Preserve permissive CORS + tracing middleware unless deployment hardening is requested.
- Keep list query builder semantics (completed filter + `q` LIKE filter + deterministic ordering).
- Preserve pagination defaults and clamp behavior used by current handler implementation.

## 14) Planner delegation constraints

- Keep planner response sanitization (trim + drop empty) in backend before returning to clients.
- Preserve cap on returned task count from planner path to avoid unexpectedly large payloads.
- Maintain existing upstream-to-backend error code mapping strategy for stable frontend handling.

## 15) Test-first change guidance

- For any endpoint behavior change, update/add integration coverage in `tests/api_tasks.rs`.
- Prefer asserting standardized error envelope fields (`code`, `message`, `details`) in tests.
- Keep auth posture tests intact until v2 auth work is intentionally introduced.

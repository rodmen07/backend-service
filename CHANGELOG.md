# Changelog

## [0.2.0](https://github.com/rodmen07/backend-service/compare/task-api-service-v0.1.0...task-api-service-v0.2.0) (2026-03-07)


### Features

* add admin metrics and api request audit logs ([5158170](https://github.com/rodmen07/backend-service/commit/51581700a23d8cefbb3ae0eead67a4569998e436))
* add LLM goal-to-task planning endpoint ([c900b1a](https://github.com/rodmen07/backend-service/commit/c900b1adbf6af2a8326e755d333846dca5cf9ee7))
* add optional due_date field to tasks ([077921e](https://github.com/rodmen07/backend-service/commit/077921e023d52c4f58729ec5c07c94eb52f7a0e9))
* add readiness endpoint and operational semantics ([10d35fc](https://github.com/rodmen07/backend-service/commit/10d35fcf71bc51a4c4df9ce2fc173e43fb04e17d))
* add SQLite persistence with sqlx and migrations ([ed4b271](https://github.com/rodmen07/backend-service/commit/ed4b271e84626dadebdb4185341770fa9deff41d))
* add task comments API ([abd6b58](https://github.com/rodmen07/backend-service/commit/abd6b5867c500004a6f45d97d1c7b66a25e9fc90))
* add task difficulty for forge progression ([7d75b3e](https://github.com/rodmen07/backend-service/commit/7d75b3eb681800fff7bef3c15d3e0fb53f2b59b7))
* add task labels — migration, model, and handler updates ([25450b3](https://github.com/rodmen07/backend-service/commit/25450b398db2b81098e5fcfe5fee443505dd0a75))
* add task list pagination and filtering ([2f94115](https://github.com/rodmen07/backend-service/commit/2f941159fa368a1e3687c0d62aa6d6e6afed96a4))
* add task status column for Kanban workflow ([e27732c](https://github.com/rodmen07/backend-service/commit/e27732c5abef1226fad224562a28ad57ade15c59))
* AI-aware planning with task context, plan rate limiting, and source tracking ([6bbb776](https://github.com/rodmen07/backend-service/commit/6bbb776ba00f064a13e5aaec589ab967665b30fc))
* enforce bearer auth middleware and jwt validation ([cd98733](https://github.com/rodmen07/backend-service/commit/cd9873330c07b84f39c89d030de7fb5f0a21d53b))
* enforce title invariant and standardize API errors ([03aaad1](https://github.com/rodmen07/backend-service/commit/03aaad1965f368b3f3996295bd7a9319d627a963))
* finalize auth contract and add k6 load test harness ([f886673](https://github.com/rodmen07/backend-service/commit/f886673c7414956728ae65bc8df24acb2db4484d))
* forward feedback and target_count to AI orchestrator ([ed8b093](https://github.com/rodmen07/backend-service/commit/ed8b093944c50706dfb0a8d4be10af70c9e2ff7a))
* harden backend CORS and orchestrator timeout ([aeee6e5](https://github.com/rodmen07/backend-service/commit/aeee6e57fb5d0605df89451cc0471c0f495c4b09))
* map tasks to goals for progress tracking ([acd3980](https://github.com/rodmen07/backend-service/commit/acd39806d43eb79d3ecbeef8ee9e529b1ee512b9))
* update planner error handling for Anthropic backend ([464fa6f](https://github.com/rodmen07/backend-service/commit/464fa6fa119ab643d0f86cebac95d55c20a2aa67))


### Bug Fixes

* **ci:** align rust check name with branch protection ([57fc00f](https://github.com/rodmen07/backend-service/commit/57fc00f95d5caf968b5f3b2f717cb55d81a67234))
* resolve Axum Handler trait error and unused imports ([d4e9a99](https://github.com/rodmen07/backend-service/commit/d4e9a9950a079f6f505987280ac23af2d6970b24))
* upgrade backend rust toolchain in Docker build ([9dc5a6a](https://github.com/rodmen07/backend-service/commit/9dc5a6a0959fed544d76ba3cb8456c4e57d7d0d6))
* use rustc-1.86 compatible jwt dependency ([c47b01d](https://github.com/rodmen07/backend-service/commit/c47b01df0ee0fbfd64d7745b5f74ead3c7571d80))
* use user-only prompt for OpenRouter compatibility ([94401f8](https://github.com/rodmen07/backend-service/commit/94401f899c5c5f163fc99ef03aa593483339366c))

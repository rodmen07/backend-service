# Changelog

## [0.2.0](https://github.com/rodmen07/backend-service/compare/projects-v0.1.0...projects-v0.2.0) (2026-03-04)


### Features

* add admin metrics and api request audit logs ([5158170](https://github.com/rodmen07/backend-service/commit/51581700a23d8cefbb3ae0eead67a4569998e436))
* add LLM goal-to-task planning endpoint ([c900b1a](https://github.com/rodmen07/backend-service/commit/c900b1adbf6af2a8326e755d333846dca5cf9ee7))
* add readiness endpoint and operational semantics ([10d35fc](https://github.com/rodmen07/backend-service/commit/10d35fcf71bc51a4c4df9ce2fc173e43fb04e17d))
* add SQLite persistence with sqlx and migrations ([ed4b271](https://github.com/rodmen07/backend-service/commit/ed4b271e84626dadebdb4185341770fa9deff41d))
* add task difficulty for forge progression ([7d75b3e](https://github.com/rodmen07/backend-service/commit/7d75b3eb681800fff7bef3c15d3e0fb53f2b59b7))
* add task list pagination and filtering ([2f94115](https://github.com/rodmen07/backend-service/commit/2f941159fa368a1e3687c0d62aa6d6e6afed96a4))
* enforce bearer auth middleware and jwt validation ([cd98733](https://github.com/rodmen07/backend-service/commit/cd9873330c07b84f39c89d030de7fb5f0a21d53b))
* enforce title invariant and standardize API errors ([03aaad1](https://github.com/rodmen07/backend-service/commit/03aaad1965f368b3f3996295bd7a9319d627a963))
* finalize auth contract and add k6 load test harness ([f886673](https://github.com/rodmen07/backend-service/commit/f886673c7414956728ae65bc8df24acb2db4484d))
* harden backend CORS and orchestrator timeout ([aeee6e5](https://github.com/rodmen07/backend-service/commit/aeee6e57fb5d0605df89451cc0471c0f495c4b09))
* map tasks to goals for progress tracking ([acd3980](https://github.com/rodmen07/backend-service/commit/acd39806d43eb79d3ecbeef8ee9e529b1ee512b9))


### Bug Fixes

* **ci:** align rust check name with branch protection ([57fc00f](https://github.com/rodmen07/backend-service/commit/57fc00f95d5caf968b5f3b2f717cb55d81a67234))
* upgrade backend rust toolchain in Docker build ([9dc5a6a](https://github.com/rodmen07/backend-service/commit/9dc5a6a0959fed544d76ba3cb8456c4e57d7d0d6))
* use rustc-1.86 compatible jwt dependency ([c47b01d](https://github.com/rodmen07/backend-service/commit/c47b01df0ee0fbfd64d7745b5f74ead3c7571d80))
* use user-only prompt for OpenRouter compatibility ([94401f8](https://github.com/rodmen07/backend-service/commit/94401f899c5c5f163fc99ef03aa593483339366c))

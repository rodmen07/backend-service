FROM rust:1.90-bookworm AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates \
  && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/task-api-service /usr/local/bin/task-api-service
COPY --from=builder /app/migrations ./migrations

# Run as an unprivileged user (SOC 2 CC6.8: containers must not run as root).
RUN useradd --system --uid 1001 --user-group --no-create-home appuser \
    && chown -R appuser:appuser /app
USER appuser

ENV HOST=0.0.0.0
ENV PORT=8080

EXPOSE 8080

CMD ["task-api-service"]

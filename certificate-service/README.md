# Certificate Service

Certificate microservice following **Clean Architecture + Hexagonal**. Built with Axum, SQLx, and PostgreSQL.

## Architecture

| Layer | Path | Responsibility |
|-------|------|----------------|
| **Domain** | `src/domain/` | `Certificate` entity, `ParsedCertificate`, `CertificateParser` trait. No SQLx/Axum. |
| **Application** | `src/application/` | Use cases, repository trait, DTOs, errors, x509 parser impl. |
| **Adapters** | `src/adapters/` | **Driving**: Axum HTTP handlers. **Driven**: SQLx `PostgresCertificateRepository`. |
| **Infrastructure** | `src/infrastructure/` | Repositories, pool, migrations, router wiring. |

## Requirements

- Rust (edition 2021)
- PostgreSQL
- `DATABASE_URL` (default: `postgres://postgres:postgres@localhost:5432/certificate_db`)
- Optional: `APP_PORT` (default: `8080`)

## Run

```bash
export DATABASE_URL="postgres://postgres:postgres@localhost:5432/certificate_db"
cargo run
```

Migrations run automatically on startup.

## API

- **POST /certificates**  
  - With PEM: `{ "pem": "-----BEGIN CERTIFICATE-----\n..." }`. Parses and stores certificate metadata.  
  - With raw metadata: `{ "subject", "issuer", "expiration" (ISO 8601), "san_entries" (optional) }`.
- **GET /certificates** — List with `?page=&limit=&status=` (status: Valid | Expiring Soon | Expired). Max limit 10,000.
- **GET /certificates/:id** — Certificate by id.
- **GET /certificates/:id/pem** — Raw PEM body.
- **GET /certificates/metrics** — `{ "total", "expiring_soon" }` (expiring within 30 days).

## Tests

```bash
cargo test
```

- **Parser**: Unit tests in `src/tests/parser_tests.rs` (invalid/valid PEM).
- **Expiration**: Status logic in `src/tests/expiration_tests.rs`.
- **Repository**: `src/tests/repository_tests.rs` — require `DATABASE_URL` (skipped if unset).
- **API**: `src/tests/api_tests.rs` — POST/GET with mock repository.

## Seed (50k records)

```bash
export DATABASE_URL="postgres://postgres:postgres@localhost:5432/certificate_db"
cargo run --bin seed_inventory
```

Runs migrations and inserts 50,000 certificate rows for performance testing.

## Docker

The **Dockerfile** uses a multi-stage build:

- Builder: `rust:bookworm`, `cargo build --release`.
- Runtime: `debian:bookworm-slim`, non-root user `appuser`, binary in `/usr/local/bin`.

Build and run with docker-compose from the repo root:

```bash
docker-compose up --build
```

Set `DATABASE_URL` in the environment for the certificate-service container.

## Container security

- Non-root user `appuser` (USER in Dockerfile).
- Slim base image: `debian:bookworm-slim`.
- Multi-stage build so the final image has no build tools.

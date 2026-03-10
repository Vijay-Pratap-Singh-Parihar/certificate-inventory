# Certificate Manager

Next.js frontend for the **certificate-service** Rust API. Manage and monitor TLS/mTLS certificates with a responsive inventory dashboard.

## Architecture

| Layer | Technology |
|-------|------------|
| **Frontend** | Next.js 15 (App Router), React 19, TypeScript |
| **Backend** | Rust, Axum, SQLx, PostgreSQL (certificate-service) |
| **Database** | PostgreSQL |
| **Infrastructure** | Docker, docker-compose |

- **Server Components**: Inventory page fetches initial data on the server (`getCertificates`, `getDashboardMetrics`) and passes it to the client.
- **Client state**: SWR for list and metrics revalidation; filtering, sorting, and pagination are client-side.
- **API client**: `lib/api.ts` and `lib/fetcher.ts` with configurable base URL and optional self-signed TLS for development.

## Performance strategy (50k+ records)

- **Virtualization**: When the table has 50+ rows, **@tanstack/react-virtual** is used so only visible rows are rendered. A spacer row pattern keeps table layout and scroll height correct.
- **Pagination**: Backend supports `limit` up to 10,000. Use larger page sizes (e.g. 100–500) with virtualization for smooth scrolling over large datasets.
- **SWR caching**: List and metrics are cached by SWR; keys include page, limit, and status so filtering/pagination don’t refetch unnecessarily.
- **SSR initial load**: The inventory page loads first chunk and metrics on the server, then the client hydrates and can refetch or change page/filter.
- **Sorting**: Client-side sort on the current page (Name, Issuer, Status, Expires On, Last Updated) for stable, fast reordering without extra requests.

## Setup

### Local development

1. **Backend** (from repo root or `certificate-service`):
   ```bash
   cd certificate-service
   export DATABASE_URL="postgres://postgres:postgres@localhost:5432/certificate_db"
   cargo run
   ```
   API runs at `http://localhost:3001` (or `APP_PORT`).

2. **Frontend**:
   ```bash
   cd certificate-manager
   cp .env.example .env.local   # if present
   # Set NEXT_PUBLIC_CERTIFICATE_SERVICE_URL=http://localhost:3001
   npm install
   npm run dev
   ```
   App runs at [http://localhost:3000](http://localhost:3000).

### Docker setup

From the repo root (where `docker-compose.yml` lives):

```bash
export DATABASE_URL="postgres://user:pass@host:5432/dbname"
docker-compose up --build
```

- **certificate-service**: Built from `certificate-service/Dockerfile` (multi-stage, non-root user). Exposes port 3001.
- **certificate-manager**: Uses Node 24 Alpine; depends on certificate-service. Exposes port 3000.

Set `DATABASE_URL` in the environment or in `.env` so the backend can connect to PostgreSQL.

### Running migrations

Migrations run automatically when the backend starts (SQLx `migrate!` in `Repositories::new`). To run them manually with the seed binary:

```bash
cd certificate-service
export DATABASE_URL="..."
cargo run --bin seed_inventory   # also runs migrations, then seeds 50k records
```

### Seeding 50k records

From `certificate-service`:

```bash
export DATABASE_URL="postgres://postgres:postgres@localhost:5432/certificate_db"
cargo run --bin seed_inventory
```

This runs migrations and inserts 50,000 certificate records for load and virtualization testing.

## Configuration

- `NEXT_PUBLIC_CERTIFICATE_SERVICE_URL` – API base URL (e.g. `http://localhost:3001`). Used in the browser.
- `CERTIFICATE_SERVICE_URL` – Used by the Next.js server (e.g. in Docker: `http://certificate-service:3001`).
- `CERTIFICATE_SERVICE_ALLOW_SELF_SIGNED` – Set to `true` in development if the API uses HTTPS with a self-signed certificate (SSR only).

## Features

- **Inventory** (`/inventory`): List certificates with search, status filter, pagination, and column sort (ID, Name, Type, Issuer, Status, Expires On, Last Updated). Row click opens detail modal; supports 50k+ rows with virtualization.
- **Dashboard metrics**: Total certificates and count expiring in the next 30 days (from `/certificates/metrics`).
- **Certificate detail**: View metadata and download PEM from the modal.
- **Create**: Upload or paste PEM to create a new certificate (POST `/certificates`).

Requirements: Node.js >= 18.17.0 (or >= 24 for `package.json` engines).

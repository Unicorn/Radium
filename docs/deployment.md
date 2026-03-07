# Radium Self-Hosting Deployment Guide

This guide covers deploying the Radium workflow orchestration platform using Docker Compose.

## Prerequisites

- **Docker** v20.10+ and **Docker Compose v2** (`docker compose` subcommand)
- At least **4 GB RAM** and **10 GB free disk** for all services
- (Optional) **Rust toolchain** (stable) -- only needed if building Radium services from source
- (Optional) **Node.js 18+** -- only needed for the workflow-builder UI

## Quick Start

```bash
# Clone the repository
git clone https://github.com/Unicorn/Radium.git
cd Radium

# Start infrastructure services (database, auth, gateway, temporal)
docker compose up -d

# To also start the Rust services (radium-workflow, radium-discovery):
docker compose --profile rust up -d

# Verify health
curl http://localhost:3020/health
```

If the health endpoint returns a `200 OK` response, the workflow API is running.

### First API calls

```bash
# Compile a workflow definition (no auth required)
curl -X POST http://localhost:3020/compile \
  -H "Content-Type: application/json" \
  -d '{"yaml": "start:\n  type: start\n  next: stop\nstop:\n  type: stop"}'

# Validate a workflow definition
curl -X POST http://localhost:3020/validate \
  -H "Content-Type: application/json" \
  -d '{"yaml": "start:\n  type: start\n  next: stop\nstop:\n  type: stop"}'

# Check Kong gateway routes
curl http://localhost:8001/routes
```

## Environment Variables Reference

### PostgreSQL (service: `db`)

| Variable | Default | Required | Description |
|---|---|---|---|
| `POSTGRES_USER` | `postgres` | Yes | Database superuser name |
| `POSTGRES_PASSWORD` | `postgres` | Yes | Database superuser password |
| `POSTGRES_DB` | `postgres` | Yes | Default database name |

### GoTrue Auth (service: `auth`)

| Variable | Default | Required | Description |
|---|---|---|---|
| `GOTRUE_API_HOST` | `0.0.0.0` | Yes | Listen address |
| `GOTRUE_API_PORT` | `9999` | Yes | Listen port |
| `API_EXTERNAL_URL` | `http://localhost:8000` | Yes | Public-facing URL for auth callbacks |
| `GOTRUE_DB_DRIVER` | `postgres` | Yes | Database driver |
| `GOTRUE_DB_DATABASE_URL` | (see compose) | Yes | PostgreSQL connection string with `search_path=auth` |
| `GOTRUE_SITE_URL` | `http://localhost:3010` | Yes | Frontend app URL for redirects |
| `GOTRUE_URI_ALLOW_LIST` | `*` | No | Allowed redirect URIs |
| `GOTRUE_DISABLE_SIGNUP` | `false` | No | Disable new user registration |
| `GOTRUE_JWT_ADMIN_ROLES` | `service_role` | Yes | Roles with admin privileges |
| `GOTRUE_JWT_AUD` | `authenticated` | Yes | JWT audience claim |
| `GOTRUE_JWT_DEFAULT_GROUP_NAME` | `authenticated` | Yes | Default group for new users |
| `GOTRUE_JWT_EXP` | `3600` | Yes | JWT expiration in seconds |
| `GOTRUE_JWT_SECRET` | (see compose) | Yes | Secret for signing JWTs -- **change in production** |
| `GOTRUE_EXTERNAL_EMAIL_ENABLED` | `true` | No | Enable email/password auth |
| `GOTRUE_MAILER_AUTOCONFIRM` | `true` | No | Auto-confirm email addresses (disable in production) |
| `GOTRUE_SMTP_ADMIN_EMAIL` | `admin@example.com` | No | Sender address for auth emails |
| `GOTRUE_SMTP_HOST` | `inbucket` | No | SMTP server host |
| `GOTRUE_SMTP_PORT` | `2500` | No | SMTP server port |
| `GOTRUE_SMTP_SENDER_NAME` | `Radium` | No | Display name on auth emails |

### PostgREST (service: `rest`)

| Variable | Default | Required | Description |
|---|---|---|---|
| `PGRST_DB_URI` | (see compose) | Yes | PostgreSQL connection string |
| `PGRST_DB_SCHEMAS` | `public` | Yes | Exposed database schemas |
| `PGRST_DB_ANON_ROLE` | `anon` | Yes | PostgreSQL role for unauthenticated requests |
| `PGRST_JWT_SECRET` | (see compose) | Yes | Must match `GOTRUE_JWT_SECRET` |
| `PGRST_DB_USE_LEGACY_GUCS` | `false` | No | Legacy GUC compatibility |
| `PGRST_APP_SETTINGS_JWT_SECRET` | (see compose) | Yes | App-level JWT secret |
| `PGRST_APP_SETTINGS_JWT_EXP` | `3600` | No | App-level JWT expiration |

### Kong API Gateway (service: `kong`)

| Variable | Default | Required | Description |
|---|---|---|---|
| `KONG_DATABASE` | `postgres` | Yes | Kong datastore type |
| `KONG_PG_HOST` | `kong-database` | Yes | Kong database host |
| `KONG_PG_USER` | `kong` | Yes | Kong database user |
| `KONG_PG_PASSWORD` | `kong` | Yes | Kong database password |
| `KONG_DNS_ORDER` | `LAST,A,CNAME` | No | DNS resolution order |
| `KONG_PLUGINS` | `bundled` | No | Enabled plugins |
| `KONG_ADMIN_LISTEN` | `0.0.0.0:8001` | Yes | Admin API listen address |
| `KONG_PROXY_LISTEN` | `0.0.0.0:8000` | Yes | Proxy listen address |
| `KONG_NGINX_PROXY_PROXY_BUFFER_SIZE` | `160k` | No | Nginx proxy buffer size |
| `KONG_NGINX_PROXY_PROXY_BUFFERS` | `64 160k` | No | Nginx proxy buffer count and size |

### Kong Database (service: `kong-database`)

| Variable | Default | Required | Description |
|---|---|---|---|
| `POSTGRES_USER` | `kong` | Yes | Kong DB user |
| `POSTGRES_PASSWORD` | `kong` | Yes | Kong DB password |
| `POSTGRES_DB` | `kong` | Yes | Kong DB name |

### Temporal (service: `temporal`)

| Variable | Default | Required | Description |
|---|---|---|---|
| `DB` | `postgres12` | Yes | Temporal persistence backend |
| `DB_PORT` | `5432` | Yes | Database port |
| `POSTGRES_USER` | `postgres` | Yes | Database user |
| `POSTGRES_PWD` | `postgres` | Yes | Database password |
| `POSTGRES_SEEDS` | `db` | Yes | Database hostname |

### Neo4j (service: `neo4j`)

| Variable | Default | Required | Description |
|---|---|---|---|
| `NEO4J_AUTH` | `neo4j/radium-dev` | Yes | Username/password pair -- **change in production** |
| `NEO4J_PLUGINS` | `["apoc"]` | No | Plugins to install |
| `NEO4J_server_memory_heap_initial__size` | `256m` | No | JVM initial heap |
| `NEO4J_server_memory_heap_max__size` | `512m` | No | JVM max heap |

### Radium Workflow API (service: `radium-workflow`)

| Variable | Default | Required | Description |
|---|---|---|---|
| `RUST_LOG` | `info` | No | Log level filter (e.g. `debug`, `info`, `warn`) |
| `PORT` | `3000` (mapped to 3020 externally) | No | HTTP listen port |
| `SUPABASE_URL` | -- | Yes (for v1 API) | PostgREST base URL |
| `SUPABASE_SERVICE_ROLE_KEY` | -- | Yes (for v1 API) | Service-role JWT for privileged access |
| `DISCOVERY_SERVICE_URL` | -- | No | Discovery service base URL (enables indexing) |
| `KONG_ADMIN_URL` | `http://localhost:8001` | No | Kong Admin API URL |
| `TEMPORAL_ADDRESS` | `http://localhost:7233` | No | Temporal gRPC frontend address |
| `TEMPORAL_NAMESPACE` | `default` | No | Temporal namespace |

If `SUPABASE_URL` and `SUPABASE_SERVICE_ROLE_KEY` are not set, the server still starts but only exposes `/compile`, `/validate`, and `/health`. The `/v1` routes are disabled.

### Radium Discovery API (service: `radium-discovery`)

| Variable | Default | Required | Description |
|---|---|---|---|
| `RUST_LOG` | `info` | No | Log level filter |
| `PORT` | `3030` | No | HTTP listen port |
| `NEO4J_URI` | -- | Yes | Neo4j Bolt connection URI |
| `NEO4J_USER` | -- | Yes | Neo4j username |
| `NEO4J_PASSWORD` | -- | Yes | Neo4j password |

## Service Ports Reference

| Service | Internal Port | External Port | Purpose |
|---|---|---|---|
| PostgreSQL (`db`) | 5432 | 54332 | Main application database |
| Neo4j HTTP | 7474 | 7474 | Neo4j browser UI |
| Neo4j Bolt | 7687 | 7687 | Neo4j Bolt protocol (driver connections) |
| GoTrue (`auth`) | 9999 | -- | Auth service (accessed via Kong) |
| PostgREST (`rest`) | 3000 | -- | REST API (accessed via Kong) |
| Inbucket SMTP | 2500 | 2500 | Local email capture (SMTP) |
| Inbucket Web | 9000 | 9000 | Local email viewer UI |
| Kong Proxy | 8000 | 8000 | API gateway (public) |
| Kong Admin | 8001 | 8001 | Gateway admin API |
| Temporal Frontend | 7233 | 7233 | Temporal gRPC |
| Temporal Metrics | 7239 | 7239 | Temporal internal metrics |
| Radium Workflow API | 3000 | 3020 | Workflow compiler and management API |
| Radium Discovery API | 3030 | 3030 | Component discovery and graph indexing |

## Database Initialization

### How migrations run

The PostgreSQL database is initialized automatically on first start. Init scripts are mounted from `apps/workflow-builder/supabase/volumes/db/init/` and run in alphabetical order:

1. `00-initial-schema.sql` -- creates core tables, types, and functions
2. `01-kong-database.sql` -- Kong-related schema setup
3. `02-api-keys.sql` -- API key management tables
4. `03-rls-policies.sql` -- Row-level security policies

### How to reset the database

```bash
# Stop services and remove the database volume
docker compose down
docker volume rm radium-network_db-data   # or use: docker compose down -v

# Restart -- init scripts will re-run
docker compose up -d
```

### Neo4j reset

```bash
docker compose down
docker volume rm radium-network_neo4j-data
docker compose up -d
```

## Kong Configuration

Kong uses a dedicated PostgreSQL database (`kong-database`) and runs migrations automatically via the `kong-migration` service on first start.

### Dynamic route management

The Radium Workflow API manages Kong routes programmatically through the Kong Admin API. When a workflow interface is published, the service registers routes and upstream services in Kong automatically.

### Verifying Kong routes

```bash
# List all registered routes
curl http://localhost:8001/routes

# List all registered services
curl http://localhost:8001/services

# Check Kong status
curl http://localhost:8001/status
```

### Kong database reset

```bash
docker compose down
docker volume rm radium-network_kong_data
docker compose up -d
```

## Production Considerations

### Secrets management

**Never use `.env` files in production.** Set environment variables directly through your deployment platform (Kubernetes secrets, AWS SSM/Secrets Manager, HashiCorp Vault, etc.). The application will hard-fail at boot if required environment variables are missing.

At minimum, change these values from the development defaults:

- `POSTGRES_PASSWORD` (both `db` and `kong-database`)
- `GOTRUE_JWT_SECRET` and all matching `PGRST_JWT_SECRET` / `PGRST_APP_SETTINGS_JWT_SECRET`
- `SUPABASE_SERVICE_ROLE_KEY` (regenerate with the new JWT secret)
- `NEO4J_AUTH`
- `KONG_PG_PASSWORD`

### CORS

The default configuration allows all origins (`allow_origin: Any`). For production, restrict CORS origins in the Radium Workflow API source or place a reverse proxy in front that enforces origin restrictions.

### Rate limiting

The Radium Workflow API has built-in per-client rate limiting:

- **API requests:** 100 requests per 60-second window, burst allowance of 20
- **Compilation requests:** 10 requests per 60-second window, burst allowance of 5

For additional rate limiting, configure Kong plugins:

```bash
curl -X POST http://localhost:8001/plugins \
  -d "name=rate-limiting" \
  -d "config.minute=100" \
  -d "config.policy=local"
```

### API key management

API keys are stored in the `api_keys` table and validated per-request on `/v1` endpoints. Keys have expiration dates and are checked against the database on each request.

### TLS

The default Docker Compose setup does not include TLS. For production:

- Terminate TLS at a load balancer or reverse proxy in front of Kong (port 8000)
- Use TLS for the Temporal connection by configuring the Temporal client with appropriate certificates
- Secure the Kong Admin API (port 8001) -- do not expose it publicly

### Database backups

```bash
# Dump the main database
docker exec radium-db pg_dump -U postgres postgres > backup.sql

# Restore
cat backup.sql | docker exec -i radium-db psql -U postgres postgres
```

For Neo4j, use the `neo4j-admin dump` command or file-system-level snapshots of the `neo4j-data` volume.

## Monitoring

### Health endpoints

```bash
# Radium Workflow API
curl http://localhost:3020/health

# GoTrue Auth
curl http://localhost:8000/auth/v1/health   # via Kong

# Kong
curl http://localhost:8001/status

# Temporal
temporal operator cluster health --address localhost:7233
```

### Logging

All Rust services use `tracing` with configurable log levels via the `RUST_LOG` environment variable:

```bash
# Examples
RUST_LOG=debug          # verbose output
RUST_LOG=info           # standard (default)
RUST_LOG=warn           # warnings and errors only
RUST_LOG=radium=debug,tower_http=info   # per-crate filtering
```

View container logs:

```bash
docker compose logs -f radium-workflow
docker compose logs -f --tail=100 kong
```

## Troubleshooting

### Service will not start

```bash
# Check which services are running
docker compose ps

# Check logs for a specific service
docker compose logs <service-name>

# Restart a single service
docker compose restart <service-name>
```

### Kong routes not working

1. Verify PostgREST is running: `docker compose ps rest`
2. Verify Kong can reach PostgREST: `docker compose exec kong curl http://rest:3000/`
3. Check registered routes: `curl http://localhost:8001/routes`
4. Check Kong logs: `docker compose logs kong`

### Temporal connection issues

1. Verify Temporal is healthy: `docker compose ps temporal`
2. Check the Temporal address in radium-workflow logs: `docker compose logs radium-workflow | grep -i temporal`
3. Ensure the `TEMPORAL_ADDRESS` env var uses the Docker network hostname (`temporal:7233`), not `localhost`, when running inside Docker Compose

### Database connection failures

1. Check that the `db` service is healthy: `docker compose ps db`
2. Test connectivity: `docker compose exec db pg_isready -U postgres`
3. Check if init scripts completed: `docker compose logs db | grep "init process complete"`

### Port conflicts

If a port is already in use, either stop the conflicting process or remap the port in `docker-compose.yml`:

```yaml
ports:
  - "3021:3000"   # map to a different host port
```

Common ports to check: 5432/54332 (PostgreSQL), 8000/8001 (Kong), 7233 (Temporal), 3020 (Radium Workflow), 3030 (Radium Discovery).

### Volume permission issues

On Linux, Docker volumes may have root ownership. If services fail with permission errors:

```bash
# Reset volume ownership
docker compose down
docker volume rm radium-network_db-data
docker compose up -d
```

### Docker Compose profiles

The Rust services (`radium-workflow`, `radium-discovery`) are in the `rust` profile and are not started by default. To include them:

```bash
docker compose --profile rust up -d
```

For local development, you may prefer to run the Rust services natively with `cargo run` while Docker provides the infrastructure.

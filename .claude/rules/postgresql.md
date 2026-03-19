# PostgreSQL Rules

## Design Principles

- **Data integrity first**: Enforce constraints (FK, NOT NULL, CHECK) at DB level
- **Measure before optimizing**: Always `EXPLAIN ANALYZE` before tuning queries
- **Zero-downtime schema changes**: Design all DDL for online migration
- **Pool connections, protect Primary**: Route reads to replicas, reserve write headroom

## Naming Conventions

- snake_case for all identifiers. Tables plural (`users`), columns singular (`email`)
- Index naming: `{table}_{columns}_{type}` (e.g., `users_email_unique`, `orders_created_at_brin`)
- Avoid SQL reserved words. Max 63 characters for identifiers

## Data Types

- **PK**: `bigint generated always as identity` (serial/int4 prohibited)
- **Distributed ID**: UUIDv7 (`uuidv7()` PG18+ or `gen_random_uuid()`)
- **Timestamps**: `timestamptz` only (`timestamp` without tz prohibited)
- **Text**: `text` (not `varchar`). **Money**: integer cents (never `money` type)
- Document tables and columns with `COMMENT ON`

## Table Design

- Every table must have a PK. Prefer surrogate key + unique constraint over composite PK
- FK constraints for referential integrity. Use `CASCADE DELETE` cautiously
- `NOT NULL` by default. Allow nullable only with explicit reason
- `CHECK` constraints for domain rules at DB level
- Standard columns: `created_at timestamptz NOT NULL DEFAULT now()`, `updated_at timestamptz`
- Partition tables with 100M+ rows using RANGE (by date or ID)

## Index Strategy

- B-tree is default. Index frequently filtered columns in WHERE clauses
- Partial indexes (`WHERE status = 'active'`) to reduce index size
- Covering indexes (`INCLUDE`) to enable index-only scans
- GIN for JSONB / full-text search. BRIN for time-series data
- Always `CREATE INDEX CONCURRENTLY` in production (avoid locks)
- Detect and drop unused indexes via `pg_stat_user_indexes` (`idx_scan = 0`)

## Query Writing

- `EXPLAIN ANALYZE` every query before production deployment
- CTE (`WITH`) to decompose complex queries (PG12+ auto-inlines CTEs)
- Avoid 3+ table JOINs. If needed, verify with `EXPLAIN`
- `SELECT *` prohibited. Specify only required columns
- Cursor-based pagination (`WHERE id > :last_id LIMIT n`). OFFSET prohibited for large datasets
- Default isolation: `READ COMMITTED`. Use `SERIALIZABLE` for financial transactions
- Always review ORM-generated SQL

## Migration (Zero-Downtime)

- `SET lock_timeout = '5s'` before every DDL statement
- `ADD COLUMN ... DEFAULT` with constant values only (non-constant triggers table rewrite)
- `CREATE INDEX CONCURRENTLY` to avoid blocking writes
- Add FK / CHECK with `NOT VALID`, then `VALIDATE CONSTRAINT` separately
- DDL causing table rewrites is prohibited (column type change, `SET NOT NULL` on large tables)
- Backfills must use strict rate limiting (stability over speed)

## Connection Management

- Connection pooling required (PgBouncer / pgpool)
- Set `idle_in_transaction_session_timeout` (30s recommended)
- Set `statement_timeout` on application side
- Co-locate client, pooler, and replicas in the same region

## Scaling

- Offload reads to replicas. Reserve write headroom on Primary
- Isolate write-heavy workloads (consider sharding)
- Multi-layer rate limiting: application / pooler / proxy / query

## Maintenance

- Enable `autovacuum` with tuned parameters (per-table settings for large tables)
- Run manual `VACUUM ANALYZE` after bulk UPDATE/DELETE
- Monitor table/index bloat with `pgstattuple`. Use `pg_repack` for online reorganization

## Backup & Recovery

- `pg_dump` for logical backup. WAL archiving + PITR for point-in-time recovery
- Test restore from backup regularly. Define RTO/RPO to determine backup strategy

## Security

- Least privilege: GRANT only required permissions to application roles
- No superuser for application connections. Create dedicated roles
- RLS (Row Level Security) for multi-tenant data isolation
- Restrict connections in `pg_hba.conf`. Require SSL
- Audit logging with `pgaudit` extension

## Monitoring

- `pg_stat_statements`: Identify slow queries (top N by `total_time`)
- `pg_stat_activity`: Monitor locks and long-running transactions
- `pg_stat_user_tables`: Flag tables with high sequential scan count for indexing
- Monitor replication lag with threshold alerts

## Anti-patterns

- **`SELECT *`**: Transfer unnecessary columns; use explicit column list
- **OFFSET pagination**: Use cursor-based (`WHERE id > :last_id`) for large datasets
- **3+ table JOINs**: Cascade failures on traffic spikes
- **Table-rewriting DDL**: Causes downtime; use expand-contract pattern
- **Nullable by default**: Degrades data quality; default to `NOT NULL`
- **Unused indexes**: Increase write cost; detect with `pg_stat_user_indexes` and drop

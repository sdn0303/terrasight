# Query Optimization

## Contents

- [EXPLAIN ANALYZE](#explain-analyze)
- [Index Strategy](#index-strategy)
- [Query Writing](#query-writing)
- [Pagination](#pagination)
- [Connection Management](#connection-management)
- [Monitoring](#monitoring)

---

## EXPLAIN ANALYZE

Every query must be verified before production:

```sql
EXPLAIN (ANALYZE, BUFFERS, FORMAT TEXT)
SELECT id, price_per_sqm
FROM transactions
WHERE ST_Within(location, ST_MakeEnvelope(139.7, 35.6, 139.8, 35.7, 4326))
ORDER BY price_per_sqm DESC
LIMIT 100;
```

Look for: Seq Scan on large tables, Nested Loop with high row estimates,
Sort operations without index support.

## Index Strategy

- **B-tree**: default for frequently filtered columns in WHERE
- **GIST**: spatial queries (`geometry` columns)
- **GIN**: JSONB / full-text search
- **BRIN**: time-series data (sequential insert order)
- **Partial indexes**: `WHERE status = 'active'` to reduce index size
- **Covering indexes**: `INCLUDE (column)` for index-only scans
- Always `CREATE INDEX CONCURRENTLY` in production
- Drop unused indexes: `pg_stat_user_indexes` where `idx_scan = 0`

## Query Writing

- `SELECT *` prohibited — specify only required columns
- CTE (`WITH`) to decompose complex queries (PG12+ auto-inlines)
- Avoid 3+ table JOINs; if needed, verify with `EXPLAIN`
- Default isolation: `READ COMMITTED`; `SERIALIZABLE` for financial transactions
- Always review ORM-generated SQL

## Pagination

Cursor-based only:

```sql
SELECT id, price_per_sqm, created_at
FROM transactions
WHERE id > :last_id
ORDER BY id
LIMIT :page_size;
```

OFFSET pagination prohibited for large datasets.

## Connection Management

- Connection pooling required (PgBouncer / pgpool)
- `idle_in_transaction_session_timeout`: 30s recommended
- `statement_timeout` set on application side
- Co-locate client, pooler, and replicas in the same region

## Monitoring

- `pg_stat_statements`: top N queries by `total_time`
- `pg_stat_activity`: locks and long-running transactions
- `pg_stat_user_tables`: high sequential scan count -> needs index
- `pg_stat_user_indexes`: unused indexes (`idx_scan = 0`)
- Replication lag alerts
- `autovacuum` tuned per table for large tables

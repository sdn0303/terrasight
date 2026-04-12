---
name: postgresql-patterns
description: "PostgreSQL and PostGIS patterns for schema design, spatial queries, query optimization, indexing, and zero-downtime migrations. Use when writing SQL, creating tables, optimizing queries, or writing migration files."
metadata:
  version: "1.0.0"
  filePattern:
    - "**/*.sql"
    - "**/migrations/**"
    - "services/backend/src/infra/**"
---

# PostgreSQL Patterns

Production PostgreSQL + PostGIS guidance for this project. Each reference
file covers a specific domain — open only the ones relevant to your task.

## Core Principles

1. **Data integrity first** — enforce constraints (FK, NOT NULL, CHECK) at
   the database level, not application code.
2. **Measure before optimizing** — `EXPLAIN ANALYZE` every query before
   production deployment.
3. **Zero-downtime schema changes** — design all DDL for online migration.
4. **Cursor-based pagination** — `WHERE id > :last_id LIMIT n`.
   OFFSET prohibited for large datasets.

## Quick Reference by Task

- **New table**: [schema-design]
- **Spatial query**: [schema-design] (PostGIS section)
- **Query optimization**: [query-optimization]
- **Index strategy**: [query-optimization] (Index section)
- **Migration**: [migrations]
- **Connection tuning**: [query-optimization] (Connection section)

## Reference Files

- **[schema-design]**: Data types, naming, table design, spatial patterns, constraints
- **[query-optimization]**: EXPLAIN, indexes, pagination, connections, monitoring
- **[migrations]**: Zero-downtime DDL, backfills, expand-contract pattern

## Review Checklist

1. No `SELECT *` — columns specified explicitly?
2. Every query verified with `EXPLAIN ANALYZE`?
3. `CREATE INDEX CONCURRENTLY` for production indexes?
4. `SET lock_timeout = '5s'` before DDL?
5. Spatial queries using GIST indexes?
6. `NOT NULL` by default, nullable only with explicit reason?

## Verification

```bash
# Run migration dry-run
sqlx migrate run --dry-run

# Check for unused indexes
SELECT indexrelname, idx_scan FROM pg_stat_user_indexes WHERE idx_scan = 0;
```

[schema-design]: references/schema-design.md
[query-optimization]: references/query-optimization.md
[migrations]: references/migrations.md

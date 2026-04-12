# Zero-Downtime Migrations

## Contents

- [DDL Safety Rules](#ddl-safety-rules)
- [Adding Columns](#adding-columns)
- [Adding Indexes](#adding-indexes)
- [Adding Constraints](#adding-constraints)
- [Backfills](#backfills)
- [Expand-Contract Pattern](#expand-contract-pattern)

---

## DDL Safety Rules

- `SET lock_timeout = '5s'` before every DDL statement
- No table-rewriting DDL (column type change, `SET NOT NULL` on large tables)
- Test migrations against production-size data before deploying
- Always have a rollback script

## Adding Columns

```sql
SET lock_timeout = '5s';

ALTER TABLE transactions
ADD COLUMN category text;
```

Constant `DEFAULT` values are safe (PG11+, no table rewrite).
Non-constant defaults (e.g., `DEFAULT now()`) trigger table rewrite.

## Adding Indexes

Always use `CONCURRENTLY` to avoid blocking writes:

```sql
CREATE INDEX CONCURRENTLY transactions_category_idx
ON transactions (category);
```

Cannot run inside a transaction block. Monitor `pg_stat_progress_create_index`.

## Adding Constraints

Two-step approach to avoid long locks:

```sql
-- Step 1: Add constraint NOT VALID (instant, no scan)
ALTER TABLE transactions
ADD CONSTRAINT transactions_price_check
CHECK (price_per_sqm > 0) NOT VALID;

-- Step 2: Validate separately (scans table but allows writes)
ALTER TABLE transactions
VALIDATE CONSTRAINT transactions_price_check;
```

Same pattern for foreign keys:

```sql
ALTER TABLE transactions
ADD CONSTRAINT transactions_area_fk
FOREIGN KEY (area_id) REFERENCES areas (id) NOT VALID;

ALTER TABLE transactions
VALIDATE CONSTRAINT transactions_area_fk;
```

## Backfills

Rate-limited updates to prevent lock contention:

```sql
-- Process in batches
UPDATE transactions
SET category = 'residential'
WHERE id IN (
    SELECT id FROM transactions
    WHERE category IS NULL
    ORDER BY id
    LIMIT 1000
);
```

Stability over speed. Monitor `pg_stat_activity` for lock waits.

## Expand-Contract Pattern

For breaking schema changes:

1. **Expand**: Add new column/table alongside old
2. **Migrate**: Dual-write to both, backfill old data
3. **Switch**: Update application to read from new
4. **Contract**: Drop old column/table after verification period

-- Prevent duplicate imports: one price point per address per year.
--
-- Uses CONCURRENTLY so it can be applied to a populated table without
-- blocking reads or writes.  IF NOT EXISTS makes this migration idempotent.
--
-- Note: run this after the initial import (or with no active writers) because
-- CONCURRENTLY cannot run inside a transaction block.  If your migration runner
-- wraps statements in a transaction, execute this file separately:
--
--   psql $DATABASE_URL -f services/backend/migrations/20260323000001_l01_unique_constraint.sql

CREATE UNIQUE INDEX CONCURRENTLY IF NOT EXISTS idx_land_prices_address_year
    ON land_prices (address, year);

# Schema Design

## Contents

- [Naming Conventions](#naming-conventions)
- [Data Types](#data-types)
- [Table Design](#table-design)
- [Spatial Patterns](#spatial-patterns)
- [GeoJSON Output](#geojson-output)

---

## Naming Conventions

- `snake_case` for all identifiers
- Tables: plural (`transactions`, `zoning_areas`)
- Columns: singular (`email`, `price_per_sqm`)
- Index naming: `{table}_{columns}_{type}` (e.g., `transactions_location_gist`)
- Max 63 characters for identifiers
- Avoid SQL reserved words

## Data Types

- **PK**: `bigint GENERATED ALWAYS AS IDENTITY`
- **Distributed ID**: UUIDv7 (`gen_random_uuid()`)
- **Timestamps**: `timestamptz` only (`timestamp` without tz prohibited)
- **Text**: `text` (not `varchar`)
- **Money**: integer cents (never `money` type)
- **Spatial**: `geometry(Point, 4326)` for lat/lng, `geometry(Polygon, 4326)` for areas
- Document tables and columns with `COMMENT ON`

## Table Design

- Every table must have a PK
- `NOT NULL` by default; allow nullable only with explicit reason
- Standard columns: `created_at timestamptz NOT NULL DEFAULT now()`, `updated_at timestamptz`
- FK constraints for referential integrity; `CASCADE DELETE` cautiously
- `CHECK` constraints for domain rules
- Partition tables with 100M+ rows using RANGE (by date or ID)

## Spatial Patterns

### Point data (transactions, facilities)

```sql
CREATE TABLE transactions (
    id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    location geometry(Point, 4326) NOT NULL,
    price_per_sqm integer NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX transactions_location_gist ON transactions USING GIST (location);
```

### Polygon data (zoning, disaster risk)

```sql
CREATE TABLE zoning_areas (
    id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    boundary geometry(Polygon, 4326) NOT NULL,
    zone_type text NOT NULL
);
```

## GeoJSON Output

```sql
SELECT jsonb_build_object(
    'type', 'FeatureCollection',
    'features', COALESCE(jsonb_agg(
        jsonb_build_object(
            'type', 'Feature',
            'geometry', ST_AsGeoJSON(location)::jsonb,
            'properties', jsonb_build_object(
                'price', price_per_sqm,
                'area', area_sqm
            )
        )
    ), '[]'::jsonb)
) AS geojson
FROM transactions
WHERE ST_Within(location, ST_MakeEnvelope($1, $2, $3, $4, 4326));
```

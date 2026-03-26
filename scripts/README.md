# Scripts

## Structure

```
scripts/
  commands/              # Shell scripts (operational commands)
    db-full-reset.sh     # One-command: migrate + import all + ANALYZE
    db-import-all.sh     # Import GeoJSON + L01 into PostGIS
    db-migrate.sh        # Apply SQL migrations only
    db-seed.sh           # Seed dev data only
    db-reset.sh          # migrate + seed (minimal data)
    db-status.sh         # Show DB health, row counts, slow queries
    db-import.sh         # Import GeoJSON only (single/all datasets)
    download-data.sh     # Download government data (10 sections)
    inspect-zip.sh       # Inspect ZIP contents before conversion
    pipeline.sh          # Full pipeline: download → convert → import → build
    build-wasm.sh        # Build WASM spatial engine
  tools/                 # Python scripts (data processing, managed by uv)
    convert_geodata.py   # RAW ZIP → GeoJSON (Tokyo filtered)
    import_geojson.py    # GeoJSON → PostGIS (9 datasets)
    import_l01.py        # L01 land prices → PostGIS (multi-year)
    build_static_data.py # GeoJSON → FlatGeobuf (frontend static)
    fetch_estat.py       # e-Stat API → CSV (census, vacancy)
  pyproject.toml         # uv/Python dependencies
```

## Setup

```bash
# Install uv (if not already installed)
curl -LsSf https://astral.sh/uv/install.sh | sh
```

## Common Workflows

### First time setup (from scratch)
```bash
docker compose up -d db                      # Start DB
./scripts/commands/download-data.sh           # Download all data (~7GB)
uv run scripts/tools/convert_geodata.py       # Convert RAW → GeoJSON
./scripts/commands/db-full-reset.sh           # Migrate + import all
uv run scripts/tools/build_static_data.py     # Build FlatGeobuf
docker compose up -d                          # Start all services
```

### After code changes (rebuild DB)
```bash
./scripts/commands/db-full-reset.sh           # One command does it all
```

### After new data download
```bash
./scripts/commands/pipeline.sh --skip-download  # Convert + import + build
```

### Import only (keep existing schema)
```bash
./scripts/commands/db-import-all.sh             # GeoJSON + L01 + ANALYZE
```

### Debug data issues
```bash
./scripts/commands/inspect-zip.sh data/raw/FILE.zip    # Inspect one ZIP
./scripts/commands/inspect-zip.sh --all                 # Summary of all ZIPs
./scripts/commands/db-status.sh                         # DB health check
./scripts/commands/download-data.sh --status             # Download gap analysis
uv run scripts/tools/import_geojson.py --dry-run         # Preview import counts
```

## Gotchas (learned the hard way)

1. **Always inspect ZIPs before writing conversion code**: `unzip -l` to check actual file names, subfolder structure, encoding
2. **NLNI ZIP naming varies by year/dataset**: No single URL pattern works for all
3. **`__MACOSX/` folders in ZIPs**: Filter them out when reading GeoJSON
4. **NOT NULL + Python None**: Import scripts must return `""` / `0`, not `None`
5. **PostgreSQL geometry types**: Use `geometry(Geometry, 4326)` not specific subtypes — real data often has mixed types
6. **`seq -w` + `printf "%02d"` octal bug**: Use `$((10#$code))` for zero-padded numbers 08, 09

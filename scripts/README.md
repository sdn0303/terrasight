# Scripts

## Structure

- `commands/` -- Shell scripts for operational commands (DB management, data download, build)
- `tools/` -- Python scripts for data processing (import, convert, build static data)

## Setup

Python tools are managed with [uv](https://docs.astral.sh/uv/):

```bash
# Install uv (if not already installed)
curl -LsSf https://astral.sh/uv/install.sh | sh

# Install dependencies (from scripts/ directory)
cd scripts && uv sync
```

## Common Commands

```bash
# Reset database (migrate + seed dev data)
./scripts/commands/db-reset.sh

# Import all GeoJSON data into PostGIS
./scripts/commands/db-import.sh

# Import a single dataset
./scripts/commands/db-import.sh --dataset a29-zoning

# Download government data
./scripts/commands/download-data.sh

# Full pipeline (download -> convert -> import -> build static)
./scripts/commands/pipeline.sh
```

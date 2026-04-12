# Scripts

## Structure

```
scripts/
  commands/                         # Shell scripts (operational commands)
    pipeline.sh                     # Full pipeline v2: schema → convert → fgb → import → reinfolib → validate
    db-full-reset.sh                # One-command: migrate + import all + ANALYZE
    db-import-all.sh                # Import GeoJSON + L01 into PostGIS
    db-migrate.sh                   # Apply SQL migrations only
    db-seed.sh                      # Seed dev data only
    db-reset.sh                     # migrate + seed (minimal data)
    db-status.sh                    # Show DB health, row counts, slow queries
    db-import.sh                    # Import GeoJSON only (single/all datasets)
    download-data.sh                # Download government data (KSJ + e-Stat)
    inspect-zip.sh                  # Inspect ZIP contents before conversion
    build-wasm.sh                   # Build WASM spatial engine
  tools/
    pipeline/                       # Pipeline v2 (data processing framework)
      convert.py                    # RAW → GeoJSON (per-pref, all KSJ datasets)
      build_fgb.py                  # GeoJSON → FlatGeobuf + manifest.json
      import_db.py                  # GeoJSON → PostGIS + REINFOLIB CSV → PostGIS
      validate.py                   # Post-import validation (feature counts)
      registry.py                   # Adapter registry + catalog loader
      adapters/
        base.py                     # DatasetEntry / ConvertResult / BaseAdapter
        zip_utils.py                # GeoJSON-first ZIP reader (UTF-8 pref, CP932 normalize)
        national.py                 # NationalArchiveAdapter (N03, L01, S12, N02)
        per_pref.py                 # PerPrefArchiveAdapter (A29, P29, P04)
        manual_static.py            # ManualStaticAdapter (pre-built FGB)
        reinfolib_csv.py            # REINFOLIB CSV parsers (transaction prices + appraisals)
    fetch_estat.py                  # e-Stat API → CSV (census, vacancy) ※pipeline未統合
  pyproject.toml                    # uv/Python dependencies
```

## Pipeline v2

メインのデータ処理フロー。`pipeline.sh` が全ステップを順次実行する。

```bash
./scripts/commands/pipeline.sh 13 P0    # pref=東京, priority=P0
```

### Steps

| Step | Script | Input | Output |
|------|--------|-------|--------|
| 0 | pipeline.sh | — | DB schema (idempotent migration) |
| 0b | pipeline.sh | — | REINFOLIB schema (partitioned tables + matviews) |
| 1 | convert.py | `data/raw/*.zip` | `data/geojson/{pref}/` (7 datasets) |
| 2 | build_fgb.py | `data/geojson/{pref}/` | `data/fgb/{pref}/` + `manifest.json` |
| 3 | import_db.py | `data/geojson/{pref}/` | PostGIS (7 spatial tables) |
| 3b | import_db.py --reinfolib | `data/raw/不動産情報ライブラリ/` | PostGIS (2 tables + 2 matviews) |
| 4 | validate.py | `data/geojson/` + `data/fgb/` | Validation log |

### Data Sources

**KSJ (国土数値情報)** — GeoJSON/Shapefile in ZIP:
- N03: 行政区画, A29: 用途地域, L01: 地価公示, P29: 学校, P04: 医療施設, S12: 駅, N02: 鉄道

**REINFOLIB (不動産情報ライブラリ)** — CSV in ZIP (CP932):
- 不動産取引価格 (729K rows/東京, 6.3M total)
- 鑑定評価書 (2.5K rows/東京, 50K total)

### Format Handling

Pipeline v2 は以下のデータフォーマット問題を自動処理する:

- **GeoJSON-first**: ZIP 内の GeoJSON を Shapefile より優先（v3.1 Shapefile 属性破損対策）
- **L01 version detection**: v3.0/v3.1 フィールド番号の差異を自動判定 + canonical name 正規化
- **空間フィルタ**: S12/N02 は都道府県コードを持たないため bbox で空間フィルタ
- **文字コード正規化**: Shapefile fallback 時の CP932 → UTF-8 自動変換
- **ジオメトリ変換**: S12 LineString → Point (centroid), N02 LineString → MultiLineString
- **REINFOLIB CSV**: 特殊文字列パース (`30分〜60分`→45, `戦前`→1945, 和暦→西暦)

## Setup

```bash
# Install uv (if not already installed)
curl -LsSf https://astral.sh/uv/install.sh | sh
```

## Common Workflows

### First time setup (from scratch)
```bash
docker compose up -d db                         # Start DB
./scripts/commands/download-data.sh              # Download all data
./scripts/commands/pipeline.sh 13 P0             # Full pipeline for Tokyo
docker compose up -d                             # Start all services
```

### After code changes (rebuild DB)
```bash
./scripts/commands/pipeline.sh 13 P0             # Re-run pipeline
```

### Import REINFOLIB data only
```bash
export DATABASE_URL="postgresql://app:devpass@localhost:5432/realestate"
uv run scripts/tools/pipeline/import_db.py --pref 13 --reinfolib
```

### Individual pipeline steps
```bash
uv run scripts/tools/pipeline/convert.py --pref 13 --dataset land-price    # Single dataset
uv run scripts/tools/pipeline/convert.py --pref 13 --priority P0           # All P0
uv run scripts/tools/pipeline/build_fgb.py --pref 13                       # Build FGB
uv run scripts/tools/pipeline/import_db.py --pref 13 --priority P0         # Import KSJ
uv run scripts/tools/pipeline/import_db.py --pref 13 --reinfolib           # Import REINFOLIB
uv run scripts/tools/pipeline/validate.py --pref 13                        # Validate
```

### Debug data issues
```bash
./scripts/commands/inspect-zip.sh data/raw/FILE.zip     # Inspect one ZIP
./scripts/commands/db-status.sh                          # DB health check
```

### e-Stat data (standalone, pipeline未統合)
```bash
export ESTAT_APP_ID="your-app-id"                            # or set in services/backend/.env
uv run scripts/tools/fetch_estat.py                          # Fetch all
uv run scripts/tools/fetch_estat.py --dataset census --pref 13  # Census, Tokyo only
uv run scripts/tools/fetch_estat.py --dry-run                # List available tables
```

## Standalone Scripts

### fetch_estat.py

e-Stat API (政府統計の総合窓口) から国勢調査人口・住宅空き家率を取得する**スタンドアロンスクリプト**。

- **Status**: 機能的に完成、`download-data.sh` から呼ばれる
- **Output**: `data/estat/census_population_tokyo.csv`, `data/estat/housing_vacancy_municipality.csv`
- **API key**: `ESTAT_APP_ID` (環境変数 or `services/backend/.env`)
- **Pipeline 統合**: 未実装。DB テーブル・インポートパスなし
- **将来**: TLS スコアリング S4 (将来性) / S5 (価格) の入力データとして使用予定。統合時に `population` / `vacancy_rate` テーブル + import パスが必要

## Data Structure

全テーブル定義・FGB レイヤー・パイプラインフローの詳細は [`docs/DATA_STRUCTURE.md`](../docs/DATA_STRUCTURE.md) を参照。

## Gotchas

1. **ZIP は必ず中身を確認**: `unzip -l` でファイル名・サブフォルダ構造・エンコーディングを確認
2. **KSJ ZIP 構造は年度・データセットごとに異なる**: backslash パス、ネストディレクトリ、Shift-JIS/UTF-8 デュアルディレクトリ
3. **v3.1 Shapefile は壊れている**: 144+ カラムで fiona が不正値を返す → GeoJSON-first で回避
4. **REINFOLIB CSV はヘッダの有無が混在**: 取引価格=ヘッダあり, 鑑定評価書=ヘッダなし (1408列)
5. **NOT NULL + Python None**: import で `""` / `0` を返す、`None` は NG
6. **REFRESH MATERIALIZED VIEW CONCURRENTLY**: psycopg2 の autocommit モードが必要
7. **パーティションテーブルの DELETE**: `WHERE pref_code =` でパーティション pruning が効く

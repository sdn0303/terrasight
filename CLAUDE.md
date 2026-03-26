# Real Estate Investment Data Visualizer

不動産投資データ可視化プラットフォーム（東京23区）。MLIT API → Rust Axum → GeoJSON → MapLibre GL 3D Map。

## Tech Stack

- **Backend**: Rust (Axum + Tokio + SQLx + PostGIS)
- **Frontend**: Next.js 16 (App Router) + React 19 + MapLibre GL + shadcn/ui + Tailwind CSS v4
- **Database**: PostgreSQL + PostGIS
- **Cache**: SQLite (24h TTL for MLIT API responses)
- **Infra**: Docker Compose

## Project Structure

```
services/
├── backend/    # Rust Axum (Clean Architecture: handler/usecase/domain/infra)
└── frontend/   # Next.js 16 (features/components/stores/hooks)
```

## Build & Test

```bash
# Backend
cd services/backend && cargo build && cargo test && cargo clippy -- -D warnings

# Frontend
cd services/frontend && pnpm install && pnpm tsc --noEmit && pnpm biome check . && pnpm vitest run
```

## Absolute Rules (MUST follow)

- **No secrets in code**: API keys via env vars only. `.env` in `.gitignore`
- **No `.unwrap()` in Rust non-test code**: Use `?` or `.expect("reason")`
- **No `any` in TypeScript**: Use `unknown` + narrowing
- **No `SELECT *`**: Specify columns explicitly
- **No OFFSET pagination**: Use cursor-based
- **Validate at boundaries**: Zod (frontend) + Axum extractors (backend)
- **Server Components by default**: `'use client'` only when necessary
- **Domain layer is pure**: Zero external dependencies in `src/domain/`
- **GeoJSON coordinates**: Always `[longitude, latitude]` (RFC 7946)
- **No raw innerHTML**: Always sanitize with DOMPurify if rendering user content

## Performance Rules

- **Profile before optimizing**: `chrome://tracing` or DevTools Performance タブでボトルネックを計測してから着手。推測で最適化しない
- **No store-derived query keys without debounce**: Zustand の `viewState` 等の高頻度更新値から TanStack Query の `queryKey` を導出する場合、必ずデバウンス済み state 経由にする。直接購読すると毎フレーム再フェッチが発生する
- **WASM は O(n log n) 以上が対象**: 単純 O(n) ループ（加減算・オブジェクト生成）は JS で十分高速。WASM の FFI シリアライズコストが計算コストを上回る場合が多い
- **リクエスト削減 > 計算高速化**: パフォーマンス問題の大半は「計算が遅い」ではなく「不要な処理が多すぎる」。まずリクエスト数・レンダリング回数を疑う

## API Contract Rules

- **Frontend Zod schema is source of truth**: Backend の Serialize DTO を実装する際は、対応する Zod スキーマのフィールド名・構造・ネスト位置を正確に一致させる。フロントとバックエンドを同時実装する場合でも、Zod スキーマを先に確定させてからバックエンド DTO を合わせる
- **Zod `z.record()` は null を拒否**: Backend で optional/unavailable なオブジェクトフィールドは `json!(null)` ではなく `json!({})` を返す。`z.record(z.string(), z.unknown())` は object のみ受理し、null で Zod parse エラーになる
- **Integration test で API contract を検証**: 新しい API レスポンス形式を実装したら、integration test でフィールドパス（`body["tls"]["label"]` 等）を明示的に assert する。Frontend Zod スキーマと同じフィールド名を検証すること

## Anti-patterns (MUST avoid)

- Secrets in source code, Docker ENV, or CI logs
- `useEffect` for data fetching (use TanStack Query)
- Syncing query data to local state
- Inline `queryFn` without custom hook wrapper
- Zustand store を直接購読して TanStack Query の `queryKey` を生成（デバウンスなしのリクエスト洪水）
- 計測なしのパフォーマンス最適化（WASM・Web Worker 等の導入判断含む）
- 3+ table JOINs without `EXPLAIN ANALYZE`
- `ubuntu-latest` in GitHub Actions (pin `ubuntu-24.04`)
- Floating action tags in CI (pin to full SHA)
- Backend DTO で `#[serde(rename)]` と Frontend Zod フィールド名のズレ放置（実装直後に integration test で検証する）
- レガシーコードの `#[allow(dead_code)]` 温存（新実装が動作確認できたら即削除。並行存在はコードベースのノイズになる）

## Data Pipeline Rules

- **ZIP内容を確認してからスクリプトを書く**: `unzip -l data/raw/FILE.zip | grep -E "\.(geojson|shp|gml)"` で実際のファイル名・パス・エンコーディングを検証。NLNIのZIPは年度・都道府県で命名規則が異なる
- **NOT NULL制約とimportスクリプトの整合性**: スキーマで `NOT NULL DEFAULT ''` にしても、INSERT文で明示的にNULLを渡すとDEFAULTは効かない。Pythonのtransform関数は `None` ではなく空文字列/0を返すこと
- **CHECK制約は緩めに設定**: 公的データは想定外の値を含む。`magnitude >= -2` でも J-SHIS は -999 をセンチネル値として使う。CHECKは安全側（NOT NULLのみ）にして、アプリ層でバリデーション
- **geometry型はGenericを許容**: PostGIS `geometry(MultiLineString, 4326)` で定義しても、実データが `Polygon` や `LineString` の場合がある。`geometry(Geometry, 4326)` + geography functional indexが安全
- **ZIP-in-ZIPに注意**: 500mメッシュ等は外側ZIPの中に都道府県別の内側ZIPがある。二段階展開が必要
- **Shift-JIS / __MACOSX フィルタ**: NLNIのZIPに `__MACOSX/` リソースフォークや Shift-JIS/UTF-8 の重複ファイルが含まれる。geojson読み込み時にフィルタ必須
- **`seq -w` + `printf "%02d"` のoctal問題**: `08`, `09` がoctal numberとして解釈される。`$((10#$code))` で明示的に10進数に変換
- **PostgreSQL AVG/STDDEV は NUMERIC を返す**: `integer` カラムの `AVG()` / `STDDEV()` は `NUMERIC` 型。Rust `f64` にマッピングするには `::double precision` キャストが必須

## Operations Quick Reference

```bash
# DB: 一発リセット（マイグレーション + シード + インポート + ANALYZE）
./scripts/commands/db-full-reset.sh

# DB: マイグレーションのみ
./scripts/commands/db-migrate.sh

# DB: 全データインポート（GeoJSON + L01）
./scripts/commands/db-import-all.sh

# データ: 政府データ全セクションダウンロード
./scripts/commands/download-data.sh

# データ: ダウンロード状況確認
./scripts/commands/download-data.sh --status

# データ: RAW → GeoJSON 変換
uv run scripts/tools/convert_geodata.py

# データ: e-Stat API取得
uv run scripts/tools/fetch_estat.py

# データ: 静的FlatGeobufビルド
uv run scripts/tools/build_static_data.py

# 開発: Docker全体起動
docker compose up -d --build

# 開発: APIテスト
curl -s http://localhost:8000/api/score?lat=35.681&lng=139.767 | python3 -m json.tool

# Git: index.lock 解消
rm -f .git/index.lock
```

## Detailed Rules

See `.claude/rules/` for comprehensive guidelines:
architecture, nextjs, typescript, rust, docker, postgresql, rest-api, security, github-actions, terraform, workflow

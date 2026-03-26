# 2026-03-27 Frontend / WASM / Backend / DB Audit

## Executive summary

- 結論: 現状の最大リスクは、`frontend` / `wasm` / `backend` / `db` の境界契約が複数箇所で壊れており、特に `flood.depth_rank` と WASM `stats` 経路で「型は通るが値が正しくない」状態が発生しうる点です。
- もっとも優先度が高いのは 4 件です。`X-01` flood 契約崩れ、`X-02` WASM stats の静かな誤計算、`DB-01` `land_prices.zone_type` 未投入、`DB-02` railway schema/importer 不整合です。
- ベースラインは `HEAD=e35faed`。レビュー時点の作業ツリー差分は未追跡の `.agents/`、`.claude/skills/rust-backend-rules/`、`AGENTS.md` のみで、既存 tracked file の未コミット変更はありませんでした。

## Severity summary table

| Severity | Count | Primary themes |
| --- | ---: | --- |
| P0 | 4 | 契約不一致、静かな誤計算、import/schema drift |
| P1 | 7 | API/WASM/nullability drift、未完成 feature 公開、再フェッチ設計、rate limit |
| P2 | 5 | UX debt、worker 障害分離不足、observability、migration/import strategy |

## Cross-layer findings

### X-01

- `ID`: `X-01`
- `Severity`: `P0`
- `Layer`: `frontend + backend + db`
- `Type`: `implementation`
- `Evidence`:
  - DB schema は `flood_risk.depth_rank smallint` に変更済み: `services/backend/migrations/20260326000001_schema_redesign.sql:84-93`
  - backend area-data repository は依然 `Option<String>` として decode: `services/backend/src/infra/pg_area_repository.rs:131-156`
  - frontend Zod は `depth_rank: z.string()` を要求: `services/frontend/src/lib/schemas.ts:64-68`
  - flood layer 描画ロジックも文字列値を前提: `services/frontend/src/components/map/layers/flood-layer.tsx:11-34`
- `Impact`:
  - `/api/area-data` の flood レイヤーは DB 実型と API/描画契約が不一致です。
  - 実行時 decode failure、または decode を通しても描画/Popup が誤る可能性があります。
- `Recommendation`:
  - flood depth の canonical representation を 1 つに固定してください。
  - 推奨は「domain は numeric rank、API は frontend Zod に合わせて text へ正規化」か、その逆に frontend/API を numeric に揃えることです。
  - 変更後に flood の API contract test と frontend schema parity test を追加してください。
- `Validation`:
  - integration test で `/api/area-data?...layers=flood` の `properties.depth_rank` を明示 assert。
  - frontend test で `FloodProperties.parse()` と `FloodLayer` の style expression を同じ fixture で検証。
- `Confidence`: `high`

### X-02

- `ID`: `X-02`
- `Severity`: `P0`
- `Layer`: `frontend + wasm`
- `Type`: `implementation`
- `Evidence`:
  - frontend の WASM manifest は 11 レイヤーのみを load: `services/frontend/src/lib/wasm/spatial-engine.ts:8-20`
  - `useStats()` は `wasmReady && zoom >= 10` で WASM 結果を優先: `services/frontend/src/features/stats/api/use-stats.ts:10-39`
  - WASM `compute_stats_inner()` は `landprice` / `schools` / `medical` / `zoning` / `steep_slope` を参照: `services/wasm/src/lib.rs:266-337`
  - worker は partial load でも `init-done` を返し、adapter は `_ready=true` にする: `services/frontend/src/lib/wasm/worker.ts:92-119`, `services/frontend/src/lib/wasm/spatial-engine.ts:222-227`
- `Impact`:
  - `StatsResponse.parse()` は通っても、WASM 側は不足レイヤーを `0` / empty で返すため、ユーザーは気づかないまま誤った統計を見ます。
  - これは「fail-fast しない誤答」なので、単純なクラッシュより危険です。
- `Recommendation`:
  - 直近では WASM stats を feature-flag で無効化し、`/api/stats` を canonical source に戻してください。
  - 継続利用するなら `compute_stats` 必須レイヤーを manifest に追加し、required subset が揃わない限り `ready=false` にしてください。
  - API/WASM parity test を必須化してください。
- `Validation`:
  - 同一 bbox で `spatialEngine.computeStats()` と `/api/stats` を比較し、許容差付きで一致確認。
  - required layer を 1 つ欠損させたとき `ready` が false のままになることを worker unit test で確認。
- `Confidence`: `high`

### X-03

- `ID`: `X-03`
- `Severity`: `P1`
- `Layer`: `frontend + backend`
- `Type`: `implementation`
- `Evidence`:
  - frontend `StatsResponse` は land price fields を required number として定義: `services/frontend/src/lib/schemas.ts:143-162`
  - backend response DTO は `avg_per_sqm` / `median_per_sqm` / `min_per_sqm` / `max_per_sqm` を `Option<_>` で返す: `services/backend/src/handler/response.rs:41-48`
  - repository も SQL 集計結果を `Option` で持つ: `services/backend/src/infra/pg_stats_repository.rs:27-53`
- `Impact`:
  - bbox に land price が存在しない場合、backend は `null` を返せる一方、frontend Zod は parse error になります。
  - 本番で sparse area / zoom 条件によって stats widget が壊れる可能性があります。
- `Recommendation`:
  - 「Frontend Zod schema is source of truth」という repo rule に従い、backend DTO を先に整合させるか、Zod 側を nullable に改めて UI fallback を明示してください。
  - 契約を決めたら integration test と UI test を同時追加してください。
- `Validation`:
  - data 0 件 bbox fixture で `/api/stats` を叩き、nullability と UI fallback を検証。
- `Confidence`: `high`

### X-04

- `ID`: `X-04`
- `Severity`: `P1`
- `Layer`: `frontend + backend + db`
- `Type`: `design`
- `Evidence`:
  - frontend `selectedArea` を読む UI はあるが、設定する call site が存在しない: `services/frontend/src/components/context-panel/explore-panel.tsx:14-36`, `services/frontend/src/components/explore/area-card.tsx:7-89`, `services/frontend/src/components/map/area-highlight.tsx:6-37`, `services/frontend/src/components/explore/breadcrumb-nav.tsx:5-27`
  - `selectArea` の実装は store にのみ存在: `services/frontend/src/stores/map-store.ts:69-83`
  - backend `/api/area-stats` は code を受けるが repository 実装は global aggregate placeholder: `services/backend/src/handler/area_stats.rs:47-81`, `services/backend/src/infra/pg_admin_area_stats_repository.rs:21-94`
  - `db-reset.sh` 実行後も `admin_boundaries` row count は 0 件だった
- `Impact`:
  - feature は UX 上存在しても、end-to-end では動作しません。
  - 実装途中の placeholder を API と UI が「完成機能」として公開している状態です。
- `Recommendation`:
  - 直近では explore drill-down を feature-flag で閉じるか、`selectedArea` を設定する map interaction と `admin_boundaries` import を同時に完了してください。
  - `/api/area-stats` は placeholder のまま公開しないでください。
- `Validation`:
  - `selectedArea` 設定 -> `/api/area-stats` -> `AreaCard` 表示までの integration/E2E test を追加。
- `Confidence`: `high`

### X-05

- `ID`: `X-05`
- `Severity`: `P1`
- `Layer`: `frontend`
- `Type`: `implementation`
- `Evidence`:
  - popup rendering は `layers.ts` の `popupFields.key` をそのまま property lookup に使う: `services/frontend/src/components/map/popup-card.tsx:15-50`
  - 例: `station` popup は `stationName` / `lineName` / `passengerCount`: `services/frontend/src/lib/layers.ts:99-114`
  - flood popup は `name` / `depth` を期待: `services/frontend/src/lib/layers.ts:116-129`
  - しかし frontend schema は flood に `depth_rank` / `river_name`、zoning に `zone_type` 等を定義: `services/frontend/src/lib/schemas.ts:56-96`
  - static layer component は存在しない asset を複数参照: `services/frontend/src/components/map/layers/station-layer.tsx:9-24`; 実在 asset 一覧には該当ファイルがない: `services/frontend/public/geojson/*`
- `Impact`:
  - click-inspect popup が空表示になり、static layer は 404 で無音失敗します。
  - レイヤー定義、スキーマ、asset inventory が別々に進化しており、変更が壊れやすいです。
- `Recommendation`:
  - layer registry を単一 source-of-truth 化してください。
  - 少なくとも `popupFields` と `Zod schema` の parity test、static asset existence test を追加してください。
- `Validation`:
  - `LAYERS` 全件について `popupFields.key` が schema fixture に存在することを unit test で確認。
  - static component の `data="/geojson/..."` が実ファイルに存在することを test で確認。
- `Confidence`: `high`

### X-06

- `ID`: `X-06`
- `Severity`: `P1`
- `Layer`: `frontend + wasm`
- `Type`: `design`
- `Evidence`:
  - `getBBox()` は center + zoom から近似計算: `services/frontend/src/stores/map-store.ts:73-83`
  - `use-static-layer()` も同じ近似 bbox を queryKey に直接入れる: `services/frontend/src/hooks/use-static-layer.ts:26-51`
  - `useStats()` も bbox 座標で直接 re-query: `services/frontend/src/features/stats/api/use-stats.ts:16-35`
  - `MapView` は `move` ごとに store を更新し、`moveend` は debounce されるが bbox source 自体は近似ロジックに依存: `services/frontend/src/components/map/map-view.tsx:52-67`
- `Impact`:
  - pitch / bearing / 実 viewport bounds と一致しない bbox で fetch / compute します。
  - map pan 中の高頻度 state 変化が queryKey に流れ、不要な再フェッチや WASM 再計算を誘発します。
- `Recommendation`:
  - bbox は MapLibre 実インスタンスの `getBounds()` を single source にしてください。
  - query 用 viewport state は `viewState` と分離し、debounce 済み `queryViewport` を導入してください。
- `Validation`:
  - bbox 更新回数と query 発火回数を trace し、pan 1 回あたりの request count を比較。
  - pitch/bearing 変更時に `getBounds()` と近似 bbox が乖離しないことを確認。
- `Confidence`: `high`

## Frontend findings

### FE-01

- `ID`: `FE-01`
- `Severity`: `P1`
- `Layer`: `frontend`
- `Type`: `implementation`
- `Evidence`:
  - `selectedArea` を読む UI は複数あるが、`selectArea(...)` を呼ぶ実装が見当たらない: `services/frontend/src/components/context-panel/explore-panel.tsx:14-36`, `services/frontend/src/components/explore/area-card.tsx:7-89`, `services/frontend/src/components/map/area-highlight.tsx:6-37`
  - `rg "selectArea\\(" services/frontend/src` の結果は breadcrumb reset と store 定義のみ
- `Impact`:
  - Explore mode の drill-down は dead code に近く、保守コストだけが残ります。
- `Recommendation`:
  - 行政界クリック or 検索起点で `selectedArea` を設定する UI interaction を実装するか、機能を一旦閉じてください。
- `Validation`:
  - area selection event から `AreaCard` 表示までの UI test を追加。
- `Confidence`: `high`

### FE-02

- `ID`: `FE-02`
- `Severity`: `P2`
- `Layer`: `frontend`
- `Type`: `design`
- `Evidence`:
  - `page.tsx` は新旧 UI を併存させており、`AnalyzePanel` と旧 `ScoreCard` が同時に残る: `services/frontend/src/app/page.tsx:223-325`
  - map area は `left: 320` 固定で context panel 幅にハードコード依存: `services/frontend/src/app/page.tsx:229-230`
  - `AreaCard` の `population` / `avgTls` は placeholder 的表示のまま: `services/frontend/src/components/explore/area-card.tsx:34-67`
- `Impact`:
  - redesign 移行中の UI debt が残っており、responsive layout と仕様整合性の判断が難しくなっています。
- `Recommendation`:
  - redesign cutover 前に temporary component を明示的に削除し、panel width と map offset を layout primitive に寄せてください。
- `Validation`:
  - 主要 breakpoint で screenshot diff を取り、320px 固定前提が消えたことを確認。
- `Confidence`: `medium`

## WASM findings

### WM-01

- `ID`: `WM-01`
- `Severity`: `P1`
- `Layer`: `wasm + frontend bridge`
- `Type`: `implementation`
- `Evidence`:
  - worker は `Promise.allSettled()` 後、失敗した layer を warning にとどめて `init-done` を送る: `services/frontend/src/lib/wasm/worker.ts:92-119`
  - adapter は `init-done` を受けると無条件に `ready=true` にする: `services/frontend/src/lib/wasm/spatial-engine.ts:222-227`
- `Impact`:
  - 部分ロード失敗が readiness に反映されず、WASM の誤答率が上がります。
- `Recommendation`:
  - `requiredLayers` を定義し、欠損時は `error` か `ready=false` を返してください。
  - `counts` に loaded/failed を両方含めて telemetry 可能にしてください。
- `Validation`:
  - 1 layer 404 fixture で `useSpatialEngineReady()` が false のままかを test で確認。
- `Confidence`: `high`

### WM-02

- `ID`: `WM-02`
- `Severity`: `P2`
- `Layer`: `wasm + frontend bridge`
- `Type`: `implementation`
- `Evidence`:
  - adapter の `error` ハンドラは pending 全件を reject する: `services/frontend/src/lib/wasm/spatial-engine.ts:288-299`
- `Impact`:
  - 単一 query 失敗が無関係の pending stats/query まで巻き込むため、障害分離ができません。
- `Recommendation`:
  - message protocol に `id` を含む error を導入し、失敗した request だけを reject してください。
- `Validation`:
  - query A/B 並列時に A のみ失敗させ、B が成功継続する unit test を追加。
- `Confidence`: `high`

## Backend findings

### BE-01

- `ID`: `BE-01`
- `Severity`: `P1`
- `Layer`: `backend`
- `Type`: `implementation`
- `Evidence`:
  - rate limit layer は `per_second = 60 / rpm` を計算して `GovernorConfigBuilder::per_second()` に渡す: `services/backend/lib/api-core/src/middleware/rate_limit.rs:78-89`
  - 実運用設定は `config.rate_limit_rpm` をそのまま使用: `services/backend/src/main.rs:52-67`
- `Impact`:
  - `rpm=30` なら `per_second=2` となり実効レートは約 120 rpm、`rpm=120` でも clamp されて 60 rpm 相当になります。
  - 低め設定では緩すぎ、高め設定では厳しすぎる非線形な挙動になります。
- `Recommendation`:
  - `tower_governor` の quota API に合わせて「1 秒あたりレート」ではなく「一定期間あたりトークン数」を正しく表現するよう修正してください。
  - 少なくとも `rpm` 別の境界テストを追加してください。
- `Validation`:
  - `rpm=30/60/120` で期待 burst/steady-state を assert する unit test を追加。
- `Confidence`: `high`

### BE-02

- `ID`: `BE-02`
- `Severity`: `P2`
- `Layer`: `backend`
- `Type`: `design`
- `Evidence`:
  - router は api-core の request-id layer を使う: `services/backend/src/lib.rs:48-49`
  - telemetry crate には別実装の request-id layers と status recorder があるが、`main.rs` では `trace_layer()` しか使っていない: `services/backend/lib/telemetry/src/http.rs:56-90`, `services/backend/src/main.rs:63-67`
- `Impact`:
  - request-id と trace span の相関経路が分散しており、ログ/trace/response header を一貫して突合しづらいです。
- `Recommendation`:
  - request-id 生成・伝播・trace span への記録を 1 箇所に統一してください。
  - `http.status_code` の span record も確実に行うよう配線してください。
- `Validation`:
  - test server で `x-request-id` 応答ヘッダと trace log field の一致を確認。
- `Confidence`: `medium`

### BE-03

- `ID`: `BE-03`
- `Severity`: `P2`
- `Layer`: `backend`
- `Type`: `design`
- `Evidence`:
  - `ComputeTlsUsecase` は concrete `mlit_client::jshis::JshisClient` に依存: `services/backend/src/usecase/compute_tls.rs:1-45`
  - domain error taxonomy は `Database` 以外の外部依存失敗を表現できない: `services/backend/src/domain/error.rs:5-24`, `services/backend/src/handler/error.rs:8-29`
- `Impact`:
  - TLS 経路の失敗モードが coarse で、外部 API degraded / timeout / partial unavailable の運用判断がしづらいです。
- `Recommendation`:
  - J-SHIS を trait 抽象化し、外部依存エラーを domain 上で distinguish できるようにしてください。
  - degraded response と hard failure を分ける設計を検討してください。
- `Validation`:
  - timeout / unavailable / partial data fixture で handler status と metadata を検証。
- `Confidence`: `medium`

## DB design findings

### DB-01

- `ID`: `DB-01`
- `Severity`: `P0`
- `Layer`: `db + importer + backend`
- `Type`: `implementation`
- `Evidence`:
  - schema redesign は `land_prices.zone_type` を導入し、TLS z-score がそれに依存: `services/backend/migrations/20260326000001_schema_redesign.sql:27-57`, `services/backend/src/infra/pg_tls_repository.rs:177-218`
  - しかし `import_l01.py` は `INSERT INTO land_prices (price_per_sqm, address, land_use, geom, year)` で `zone_type` を投入しない: `scripts/tools/import_l01.py:215-238`
- `Impact`:
  - 本データ import 後、`zone_type` は null のままになり、TLS 相対割安度計算の前提が崩れます。
  - dev seed は `zone_type` を持つため、開発環境で問題が見えにくいです。
- `Recommendation`:
  - import 時に spatial join で `zone_type` を埋める処理を追加するか、schema redesign を一時 rollback してください。
  - dev seed と本 import の差を CI で検知できるようにしてください。
- `Validation`:
  - import 後に `SELECT count(*) FROM land_prices WHERE zone_type IS NULL;` を 0 件で assert。
  - TLS z-score integration test を real import fixture で追加。
- `Confidence`: `high`

### DB-02

- `ID`: `DB-02`
- `Severity`: `P0`
- `Layer`: `db + importer`
- `Type`: `implementation`
- `Evidence`:
  - 現行 `railways` schema には `station_name` 列がない: `services/backend/migrations/20260326000001_schema_redesign.sql:174-185`
  - importer は `n02-railway` に `station_name` を mapping している: `scripts/tools/import_geojson.py:240-249`
  - 旧 migration には `railways.station_name` が存在したため、途中で schema が変わっている: `services/backend/migrations/20260323000002_geojson_tables.sql:33-42`
- `Impact`:
  - full import / full reset 時に railway import が schema mismatch で壊れます。
  - migration replay と importer の整合性が失われています。
- `Recommendation`:
  - `railways.station_name` を schema に戻すか、importer mapping を削除してください。
  - migration + importer compatibility test を追加してください。
- `Validation`:
  - `db-full-reset.sh` を CI で通し、railway import の成功まで確認。
- `Confidence`: `high`

### DB-03

- `ID`: `DB-03`
- `Severity`: `P1`
- `Layer`: `db`
- `Type`: `design`
- `Evidence`:
  - `db-migrate.sh` は最新 2 migration を手作業で psql apply: `scripts/commands/db-migrate.sh:1-19`
  - `services/backend/scripts/seed-dev.sh` は `sqlx migrate run` を使う別経路を持つ: `services/backend/scripts/seed-dev.sh:26-39`
  - schema redesign migration は `admin_boundaries` を drop しない: `services/backend/migrations/20260326000001_schema_redesign.sql:13-22`
  - 次 migration は `CREATE TABLE admin_boundaries` を無条件実行: `services/backend/migrations/20260326000002_admin_boundaries.sql:6-29`
  - 実際の `db-reset.sh` 実行では `relation "admin_boundaries" already exists` / 各 index already exists が発生
- `Impact`:
  - migration replay が idempotent ではなく、適用経路によって状態が変わります。
  - 開発者がどの script を使うかで DB 状態の再現性が崩れます。
- `Recommendation`:
  - migration runner を 1 経路に統一し、全 migration を replay-safe にしてください。
  - 破壊的 migration を維持するなら full reset 専用 script と通常 migrate を明確に分離してください。
- `Validation`:
  - 空 DB と既存 DB の両方で migration を 2 回連続適用しても成功することを確認。
- `Confidence`: `high`

### DB-04

- `ID`: `DB-04`
- `Severity`: `P2`
- `Layer`: `db`
- `Type`: `design`
- `Evidence`:
  - `import_l01.py` は year ごとに delete -> insert: `scripts/tools/import_l01.py:194-240`
  - `seed-dev.sh` / `db-reset.sh` / `db-full-reset.sh` が別々の reset/import strategy を持つ
  - migration/import/API area-stats を跨ぐ自動テストが見当たらない
- `Impact`:
  - 運用上の partial failure 時に整合性確認が難しく、import 再実行の blast radius が大きいです。
- `Recommendation`:
  - import を stage table + validate + swap 方式へ寄せてください。
  - migration/import/API smoke test を CI へ追加してください。
- `Validation`:
  - import 途中失敗を注入し、前回データが残ることを確認する rollback test を追加。
- `Confidence`: `medium`

## Remediation roadmap

### Quick wins (1 day)

1. `useStats()` の WASM 優先経路を一時停止し、`/api/stats` を canonical source に戻す。
2. `depth_rank` の API 契約を 1 つに固定し、backend DTO / frontend Zod / flood rendering を同時修正する。
3. `db-migrate.sh` を replay-safe にし、`admin_boundaries` migration を idempotent 化する。
4. `rate_limit_layer()` の RPM 計算を修正し、境界テストを追加する。
5. `selectedArea` feature を hide するか、placeholder 表示を明示して誤解を防ぐ。

### Medium (within 1 week)

1. `popupFields`、interactive layer ids、asset path、Zod schema を 1 registry から生成または検証する。
2. viewport query 用 bbox を `map.getBounds()` ベースへ置き換え、query 用 state を debounce 分離する。
3. `import_l01.py` に `zone_type` 補完を実装し、`db-full-reset.sh` を CI smoke test に入れる。
4. `/api/area-stats` を `admin_boundaries` import 完了まで feature-flag 化し、end-to-end 実装後に公開する。

### Strategic (multi-layer redesign)

1. 「stats の source-of-truth」を明文化する。推奨は `API = canonical`, `WASM = preview/cache` で、parity budget を決めること。
2. migration runner、seed、bulk import を一貫した DB lifecycle に再設計する。
3. request-id / trace / degraded response を含む observability 契約を backend 横断で統一する。

## Verification appendix

### Repository baseline

- `git rev-parse --short HEAD` -> `e35faed`
- `git status --short` at review start/end:
  - `?? .agents/`
  - `?? .claude/skills/rust-backend-rules/`
  - `?? AGENTS.md`

### Commands executed

#### Frontend

```bash
cd services/frontend && pnpm tsc --noEmit && pnpm biome check . && pnpm vitest run
```

- Result:
  - `pnpm tsc --noEmit` は `biome` 実行まで進んだため少なくとも typecheck 自体は継続可能でした。
  - `pnpm biome check .` は失敗。概要は `110 errors`, `12 warnings`。
  - `&&` 連結のため、この実行チェーンでは `pnpm vitest run` は未実行です。
- Note:
  - explorer review では別実行として `pnpm vitest run` が `13 files / 119 tests passed` と報告されましたが、メイン検証チェーンでは未再現です。

#### Backend

```bash
cd services/backend && cargo clippy --workspace -- -D warnings && cargo test --workspace
```

- Result:
  - `cargo test --workspace` は失敗。
  - `mlit-client` の wiremock ベース test がローカル権限制約で bind に失敗し、`Operation not permitted` が発生しました。
- Interpretation:
  - 実装不良の可能性と、sandbox/port bind 制約の可能性が混在しています。
  - 少なくとも「この環境では再現可能に green にならない」点は事実です。

#### WASM

```bash
cd services/wasm && cargo test
bash scripts/commands/build-wasm.sh
```

- Result:
  - `cargo test`: `50 passed, 0 failed`
  - `build-wasm.sh`: 成功
  - 出力サイズ: `services/frontend/public/wasm/realestate_wasm_bg.wasm` 約 `482544 bytes`
- Warnings:
  - `wasm-pack` から `description` / `repository` / `license` の optional metadata warning あり

#### DB lint / runtime

```bash
cd services/backend && sqruff lint migrations/
docker compose up -d db
./scripts/commands/db-reset.sh
./scripts/commands/db-status.sh
```

- `sqruff lint migrations/`:
  - 失敗。少なくとも以下の migration で lint violation を検出:
    - `20260322000001_seed_dev.sql`
    - `20260323000001_l01_unique_constraint.sql`
    - `20260326000001_schema_redesign.sql`
- `docker compose up -d db`:
  - 権限昇格後に成功
- `./scripts/commands/db-reset.sh`:
  - 完走はしたが migration `20260326000002_admin_boundaries.sql` で以下を出力:
    - `relation "admin_boundaries" already exists`
    - `relation "idx_admin_geom" already exists`
    - `relation "idx_admin_code" already exists`
    - `relation "idx_admin_level" already exists`
    - `relation "idx_admin_pref" already exists`
  - その後の row count summary:
    - `admin_boundaries 0`
    - `flood_risk 4`
    - `land_prices 15`
    - `liquefaction 0`
    - `medical_facilities 6`
    - `railways 0`
    - `schools 8`
    - `seismic_hazard 0`
    - `stations 0`
    - `steep_slope 3`
    - `zoning 5`
- `./scripts/commands/db-status.sh`:
  - 成功
  - notable config:
    - `work_mem = 32MB`
    - `random_page_cost = 1.1`
    - `shared_buffers = 256MB`
    - `statement_timeout = 30s`
  - slow query 上位には `ANALYZE`, `CREATE EXTENSION IF NOT EXISTS postgis`, `DELETE FROM zoning` が含まれていました

### Runtime scenarios not fully validated

- `WASM stats` と backend `/api/stats` の同一 bbox parity 比較: `Not run`
- partial WASM load 時の UI-level `ready` 判定: `Static review only`
- frontend query key / refetch storm の browser trace 計測: `Not run`
- area selection -> area-stats -> area card の end-to-end flow: `Blocked` (`selectedArea` が未接続)


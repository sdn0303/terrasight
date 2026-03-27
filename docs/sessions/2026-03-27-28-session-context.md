# Session Context: 2026-03-27 ~ 2026-03-28

## Summary

2日間のセッションで P0.5/P1.6 の UX クラフト + データ整備 + WASM Phase 0+1 を完了。

## Completed Work

### P0.5 Data Correctness (all done)
| ID | Task | Commit |
|----|------|--------|
| X-01 | flood depth_rank 契約不整合 | `9a60faf` |
| X-02 | WASM stats サイレント誤計算 | `9a60faf` |
| DB-01 | land_prices.zone_type 未投入 | `9a60faf` |
| DB-02 | railways.station_name schema | `9a60faf` |
| X-04 | selectedArea 配線 | `25f2742` |
| X-05 | popupField キー修正 | `1084aec` |
| X-06 | bbox → map.getBounds() | `5402a4b` |

### P1.6 UX Craft (all done except panel transitions)
| Task | Commit |
|------|--------|
| page.tsx God Component リファクタ | `58add2c` (prior session) |
| モードシステム簡略化 (3→2) | `f431e68` (prior session) |
| エラー状態の可視化 | `ed091ee` |
| ガイド付き初回体験 | `fb87830` |
| 空間ポップアップ | `718686e` |
| デザインシステム実装 | `1d1e158` |
| レイヤーキュレーション | `f1a3ffa` |

### Data Cleanup
| Task | Commit |
|------|--------|
| 404レイヤー削除 (school-district, park, urban-plan) | `e825e07` |
| 全静的レイヤー FGB 移行 | `24650fd` |
| tsunami レイヤー削除 (raw data なし) | `24650fd` |
| admin-boundary プロパティ正規化 | `25f2742` |
| レガシー public/geojson/ 削除 | `e825e07` |

### WASM Phase 0+1 (Correctness)
| Task | Commit | Description |
|------|--------|-------------|
| Task 1 | `79351df` | `canonicalLayerId()` — layer ID 正規化 |
| Task 2 | `656ee4d` | `loadedLayers` + `queryReady` + `computeStats` 封鎖 |
| Task 3 | `6adaf75` | `query-error`/`stats-error` — リクエスト単位エラー分離 |
| Task 4 | `291a6ba` | Performance API 計装 |

## Test Count Progression

136 (pre-session) → 150 (post Phase 0+1)

## Remaining Tasks

### WASM Phase 2: Boundary Optimization (next)

**Spec**: `docs/superpowers/specs/2026-03-28-wasm-optimization-design.md`
**Plan (draft)**: `docs/superpowers/plans/2026-03-28-wasm-phase2.md`

| Task | Status | Description |
|------|--------|-------------|
| 2.1 | **TODO** | Rust `query_layers` JSON flattening (`String` → `Value`) |
| 2.2 | **TODO** | JS adapter: single `JSON.parse` (remove double-parse) |
| 2.3 | **TODO** | New adapter method `queryPerLayer()` returning `Map<string, FC>` |
| 2.4 | **TODO** | `useVisibleStaticLayers` batched hook (2-layer cache) |
| 2.5 | **TODO** | Static layer component migration (self-fetch → prop-receive) |

**IMPORTANT**: Task 2.1 は Rust 変更のみ (`rust-engineer`)、Task 2.2 は JS 変更のみ (`frontend-developer`) に分割すること。1サブエージェントに cross-system タスクを渡さない。

### WASM Phase 3: Value Extension (future)

| Task | Status | Description |
|------|--------|-------------|
| 3.1 | TODO | shared-domain crate (constants + pure computation) |
| 3.2 | TODO | Two-stage layer loading (boot + on-demand) |
| 3.3 | TODO | SpatialEngineProvider (app-level singleton) |
| 3.4 | TODO | Stats data ingestion (API data → WASM R-tree, separate spec) |

### Other Remaining
| Priority | Task | Status |
|----------|------|--------|
| P1.6 | パネルトランジション (framer-motion) | TODO (低優先度) |
| P2 | CI/CD パイプライン | TODO |
| P2 | 認証方針決定 | TODO |
| P2 | 全国対応性能検証 | TODO |
| P2 | FE↔BE スキーマ契約テスト | TODO |

## Learnings (from this session)

### Subagent Failure Incident
- **What happened**: Phase 2 Task 1 の Rust+JS cross-system タスクを sonnet 1エージェントに委譲 → 70分間 Read 44回で停滞
- **Root cause**: Rust (`spatial_index.rs`, `lib.rs`) と JS (`spatial-engine.ts`) の変更を1タスクに詰め込んだ。サブエージェントはコンテキスト不足で Rust ファイルを繰り返し読み込み
- **Fix**: Cross-system タスクは言語境界で必ず分割。`.claude/rules/workflow.md` に anti-pattern として追記済み

### Codex Review Integration
- Design spec と implementation plan は別ラウンドで Codex レビューに出すと効果的
- Codex はファイルキャッシュを持つため、修正後の再レビューでは「まず grep で修正を確認してから読め」と明示指示が必要
- レビュー指摘は修正済みを主張するだけでなく、行番号レベルで evidence を示すこと

### Data Pipeline
- `public/geojson/` はレガシー。全静的データは FlatGeobuf (`data/fgb/`) に統一済み
- Layer ID は UI (underscore) と WASM/FGB (hyphen) で不整合がある。`canonicalLayerId()` で境界変換
- 東京には津波浸水想定区域 (A40) のデータがない — レイヤー自体を削除が正解

## Key File Locations

| Category | Path |
|----------|------|
| WASM spec | `docs/superpowers/specs/2026-03-28-wasm-optimization-design.md` |
| Phase 0+1 plan | `docs/superpowers/plans/2026-03-28-wasm-phase0-phase1.md` |
| Phase 2 plan (draft) | `docs/superpowers/plans/2026-03-28-wasm-phase2.md` |
| TODOS backlog | `docs/plans/TODOS.md` |
| Layer ID normalization | `services/frontend/src/lib/layer-ids.ts` |
| WASM adapter | `services/frontend/src/lib/wasm/spatial-engine.ts` |
| WASM worker | `services/frontend/src/lib/wasm/worker.ts` |
| Rust spatial engine | `services/wasm/src/lib.rs` |
| R-tree index | `services/wasm/src/spatial_index.rs` |
| FGB build script | `scripts/tools/build_static_data.py` |
| Design system | `DESIGN.md` + `services/frontend/src/app/globals.css` |

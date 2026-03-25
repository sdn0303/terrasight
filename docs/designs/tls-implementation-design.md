# TLS (Total Location Score) Implementation Design

> 5-Axis Location Scoring — Backend Implementation Design v1.0
> Based on: `analysis-algorithm-design-integrated.html` v1.0

## Understanding Summary

- **What**: Replace the current 4-component score (trend/risk/access/yield x 25) with 5-axis TLS (0-100)
- **Why**: The design spec requires sub-score granularity, min() penalty for critical risks, weight presets, and grading (S/A/B/C/D/E) that the current model cannot express
- **Who**: Backend API consumers (frontend PROPERTY INTEL panel)
- **Constraints**: No new DB tables (liquefaction/stations/population_mesh); WASM unchanged
- **Non-goals**: `/api/ranking`, weight slider UI, WASM-based score computation

## Architecture

### Scoring Model

```
TLS = w1×S1 + w2×S2 + w3×S3 + w4×S4 + w5×S5

Default weights (Balance preset):
  S1 Disaster:   25%    S2 Terrain:    15%    S3 Livability: 25%
  S4 Future:     15%    S5 Price:      20%
```

### Axis Formulas

**S1 Disaster** (min-penalty):
```
S1 = min(F_flood, F_liq, F_seis, F_tsun, F_land)
     × (0.30×F_flood + 0.25×F_liq + 0.25×F_seis + 0.10×F_tsun + 0.10×F_land) / 100
```

**S2 Terrain**: `S2 = G_avs` (Phase 1, expandable to `0.50×G_avs + 0.30×G_form + 0.20×G_geo`)

**S3 Livability**: `S3 = 0.45×L_transit + 0.25×L_edu + 0.30×L_med` (Phase 1 fallback: `0.45×L_edu + 0.55×L_med`)

**S4 Future**: `S4 = 0.40×P_pop + 0.35×P_price + 0.25×P_far`

**S5 Price**: `S5 = 0.65×V_rel + 0.35×V_vol`

### Missing Data Strategy

- Missing sub-scores default to **100** (no risk / best case)
- Each axis carries a **confidence** value (0.0-1.0) = sum of available sub-score weights
- API response includes `available: bool` per sub-score

### Initial Coverage

| Axis | Available | Missing (=100) | Confidence |
|------|-----------|-----------------|------------|
| S1 Disaster | F_flood, F_seis, F_land | F_liq, F_tsun | 0.60 |
| S2 Terrain | G_avs | G_form, G_geo | 0.50 |
| S3 Livability | L_edu, L_med | L_transit | 0.55 |
| S4 Future | P_price, P_far | P_pop | 0.60 |
| S5 Price | V_rel, V_vol | (none) | 1.00 |

## File Structure

### New Files

```
domain/scoring/
  mod.rs            — pub mod declarations
  constants.rs      — all TLS weights, thresholds, mapping tables
  sub_scores.rs     — 17 pure mapping functions (input → 0-100)
  axis.rs           — S1-S5 composition functions
  tls.rs            — TLS aggregation, Grade, WeightPreset, CrossAnalysis
usecase/
  compute_tls.rs    — orchestration (parallel DB + J-SHIS → pure scoring)
infra/
  pg_tls_repository.rs — TlsRepository PostGIS implementation
```

### Modified Files

```
domain/entity.rs        — add SchoolStats, MedicalStats, ZScoreResult
domain/value_object.rs  — add TlsResult; remove InvestmentScore, ScoreComponent
domain/repository.rs    — add TlsRepository trait; remove ScoreRepository
handler/score.rs        — swap to ComputeTlsUsecase
handler/response.rs     — TlsResponse replacing ScoreResponse
```

### Deleted Files

```
usecase/compute_score.rs
infra/pg_score_repository.rs
```

## Repository Trait

```rust
#[async_trait]
pub trait TlsRepository: Send + Sync {
    async fn find_nearest_prices(&self, coord: &Coord) -> Result<Vec<PriceRecord>, DomainError>;
    async fn find_flood_depth_rank(&self, coord: &Coord) -> Result<Option<i32>, DomainError>;
    async fn has_steep_slope_nearby(&self, coord: &Coord) -> Result<bool, DomainError>;
    async fn find_schools_nearby(&self, coord: &Coord) -> Result<SchoolStats, DomainError>;
    async fn find_medical_nearby(&self, coord: &Coord) -> Result<MedicalStats, DomainError>;
    async fn find_zoning_far(&self, coord: &Coord) -> Result<Option<f64>, DomainError>;
    async fn calc_price_z_score(&self, coord: &Coord) -> Result<ZScoreResult, DomainError>;
    async fn count_recent_transactions(&self, coord: &Coord) -> Result<i64, DomainError>;
}
```

## API Response

```
GET /api/score?lat={lat}&lng={lng}
```

```json
{
  "location": { "lat": 35.6812, "lng": 139.7671 },
  "tls": { "score": 72, "grade": "A", "label": "Very Good" },
  "axes": {
    "disaster": {
      "score": 65, "weight": 0.25, "confidence": 0.60,
      "sub": {
        "flood":        { "score": 50, "available": true,  "detail": { "depth_rank": 2 } },
        "liquefaction": { "score": 100, "available": false, "detail": {} },
        "seismic":      { "score": 55, "available": true,  "detail": { "prob_30yr": 0.12 } },
        "tsunami":      { "score": 100, "available": false, "detail": {} },
        "landslide":    { "score": 40, "available": true,  "detail": { "steep_nearby": true } }
      }
    },
    "terrain":    { "score": 60, "weight": 0.15, "confidence": 0.50, "sub": { ... } },
    "livability": { "score": 82, "weight": 0.25, "confidence": 0.55, "sub": { ... } },
    "future":     { "score": 58, "weight": 0.15, "confidence": 0.60, "sub": { ... } },
    "price":      { "score": 71, "weight": 0.20, "confidence": 1.00, "sub": { ... } }
  },
  "cross_analysis": {
    "value_discovery": 39,
    "demand_signal": 64,
    "ground_safety": 39
  },
  "metadata": {
    "weight_preset": "balance",
    "data_freshness": "2025",
    "disclaimer": "本スコアは参考値です。投資判断は自己責任で行ってください。"
  }
}
```

## Grading

| TLS | Grade | Label |
|-----|-------|-------|
| 85-100 | S | Excellent |
| 70-84 | A | Very Good |
| 55-69 | B | Good |
| 40-54 | C | Fair |
| 25-39 | D | Below Average |
| 0-24 | E | Poor |

## Implementation Order

```
Step 1: domain/scoring/*           pure functions + unit tests
Step 2: domain/ type changes       entity, value_object, repository trait
Step 3: infra/pg_tls_repository    SQL implementation + integration tests
Step 4: usecase/compute_tls        orchestration + mock repo unit tests
Step 5: handler + response         API layer, delete old code
Step 6: frontend                   schema + hook + UI components
```

## Decision Log

| # | Decision | Alternatives | Rationale |
|---|----------|-------------|-----------|
| D1 | Full replacement of 4-component score with 5-axis TLS | Gradual migration / internal remap | Fundamentally different models; half-measures add complexity |
| D2 | TLS backend-only; WASM stays as area-stats | Hybrid / WASM preview | TLS needs J-SHIS API + PostGIS aggregation; WASM can't complete |
| D3 | Max-coverage MVP: all 5 axes with available data | 3-axis MVP / data-first | Stabilizes API contract early; gaps shown via confidence |
| D4 | Missing sub-scores = 100 + axis confidence | Neutral 50 / weight redistribution | Compatible with S1 min() penalty; transparent via confidence |
| D5 | S1 min() penalty from day one | Defer to Phase 3 | Missing=100 won't distort min(); preserves core design intent |
| D6 | Full response structure per design spec | 2-tier / detail-stripped | Enables PROPERTY INTEL sub-score drilldown |
| D7 | Domain-centric architecture (scoring/ submodule) | Usecase-internal | 17+ pure functions need isolation; each threshold testable |
| D8 | New trait name TlsRepository (not extending ScoreRepository) | Same-name extension | Compile errors catch migration gaps; safe parallel existence |

## Assumptions

1. `zoning` table has `zone_type` and `floor_area_ratio` columns (verified)
2. `land_prices` has sufficient recent data for V_vol (500m, 1yr count)
3. PostgreSQL connection pool >= 8 for parallel query execution
4. Weight presets: only "balance" in MVP; preset switching deferred
5. Confidence = sum of available sub-score weights within each axis
6. S3 Phase 1 fallback: when L_transit unavailable, use 0.45×L_edu + 0.55×L_med

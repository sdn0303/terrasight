//! Response DTOs for `GET /api/area-data`.
//!
//! Currently the area-data handler emits a raw `serde_json::Map` whose
//! values are `LayerResponseDto`s. A typed `AreaDataResponseDto` wrapper
//! is added here in Task C3 (handler method-chain rewrite); for now this
//! module only exists to reserve the slot in the response aggregator.

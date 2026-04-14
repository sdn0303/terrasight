//! Error type for the WASM spatial engine.
//!
//! All fallible operations in `terrasight-wasm` propagate a [`WasmError`].
//! At the JavaScript boundary in `lib.rs`, errors are converted to
//! `JsValue` strings before being returned to the caller.

use thiserror::Error;

/// Errors that can occur in the WASM spatial engine.
///
/// Each variant carries a human-readable message describing the failed
/// operation. The display format is forwarded to JavaScript via
/// `JsValue::from_str(&e.to_string())`.
#[derive(Debug, Error)]
pub enum WasmError {
    /// A layer ID was queried that has not been loaded into the engine.
    ///
    /// Occurs when [`crate::SpatialEngine::query`] or
    /// [`crate::SpatialEngine::compute_stats`] is called with an unknown
    /// `layer_id` string.
    #[error("layer not found: {0}")]
    LayerNotFound(String),

    /// The FlatGeobuf header could not be read or the stream could not be opened.
    ///
    /// Typically indicates a corrupted or truncated `.fgb` file, or that
    /// the byte slice passed to [`crate::SpatialEngine::load_layer`] is empty.
    #[error("FGB open error: {0}")]
    FgbOpen(String),

    /// An error occurred while iterating FlatGeobuf features after the header
    /// was successfully read.
    #[error("FGB iteration error: {0}")]
    FgbIteration(String),

    /// A FlatGeobuf feature could not be serialised to a GeoJSON Feature string.
    #[error("GeoJSON serialisation error: {0}")]
    GeoJsonSerialise(String),

    /// A byte buffer produced by the GeoJSON writer contained invalid UTF-8.
    ///
    /// Should not occur in practice because GeoJSON is defined to be UTF-8,
    /// but is handled defensively.
    #[error("UTF-8 conversion error: {0}")]
    Utf8(String),

    /// A `serde_json` error — propagated automatically via `#[from]`.
    ///
    /// Occurs when parsing or serialising JSON values during stats computation
    /// or when assembling GeoJSON output.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// A GeoJSON string was structurally invalid or missing required keys
    /// (`"geometry"`, `"features"`, etc.).
    #[error("GeoJSON parse error: {0}")]
    GeoJsonParse(String),

    /// The bounding box arguments provided by the caller failed validation.
    ///
    /// See [`crate::BBox::new`] for the invariants that must hold.
    #[error("invalid bbox: {0}")]
    InvalidBBox(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wasm_error_display_messages() {
        let e = WasmError::LayerNotFound("test".into());
        assert_eq!(e.to_string(), "layer not found: test");

        let e = WasmError::InvalidBBox("bad".into());
        assert_eq!(e.to_string(), "invalid bbox: bad");
    }

    #[test]
    fn json_error_from_conversion() {
        let json_err = serde_json::from_str::<serde_json::Value>("not json").unwrap_err();
        let wasm_err = WasmError::from(json_err);
        assert!(matches!(wasm_err, WasmError::Json(_)));
    }
}

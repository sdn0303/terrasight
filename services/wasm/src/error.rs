use thiserror::Error;

#[derive(Debug, Error)]
pub enum WasmError {
    #[error("layer not found: {0}")]
    LayerNotFound(String),
    #[error("FGB open error: {0}")]
    FgbOpen(String),
    #[error("FGB iteration error: {0}")]
    FgbIteration(String),
    #[error("GeoJSON serialisation error: {0}")]
    GeoJsonSerialise(String),
    #[error("UTF-8 conversion error: {0}")]
    Utf8(String),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("GeoJSON parse error: {0}")]
    GeoJsonParse(String),
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

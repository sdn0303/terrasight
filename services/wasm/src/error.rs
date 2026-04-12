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

//! Error types for Financial Pipeline

use thiserror::Error;

#[derive(Error, Debug)]
pub enum PipelineError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("HTTP request error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("CSV parsing error: {0}")]
    Csv(#[from] csv::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("No data returned for symbol: {0}")]
    NoData(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Invalid date format: {0}")]
    DateParse(String),
}

pub type Result<T> = std::result::Result<T, PipelineError>;

mod init;

pub use self::init::init_database;

use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;

pub type DatabasePool = sqlx::SqlitePool; // TODO: behind a trait?
pub type DBRow = sqlx::sqlite::SqliteRow;

pub type RequestResult<T, E = RequestError> = std::result::Result<T, E>;

pub type Id = i32;
pub type Score = i32;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreRecord {
    pub player_id: Id,
    pub score: Score,
    pub extra_info: Option<String>,
}

#[derive(thiserror::Error, Debug)]
pub enum RequestError {
    #[error("unathorized request")]
    Unathorized,
    #[error("unathorized request, not enough rights")]
    Forbidden,
    #[error("player key is invalid")]
    InvalidPlayer,
    #[error("invalid level name: {0}")]
    InvalidLevelId(String),
    #[error("a level {0} not found")]
    NoSuchLevel(Uuid),
    #[error("file not found: {0}")]
    FileNotFound(String),
    #[error("database error: {0}")]
    Sql(#[from] sqlx::Error),
}

impl RequestError {
    fn status(&self) -> StatusCode {
        match self {
            RequestError::Unathorized => StatusCode::UNAUTHORIZED,
            RequestError::Forbidden => StatusCode::FORBIDDEN,
            RequestError::InvalidPlayer => StatusCode::FORBIDDEN,
            RequestError::InvalidLevelId(_) => StatusCode::BAD_REQUEST,
            // RequestError::LevelAlreadyExists(_) => StatusCode::CONFLICT,
            RequestError::FileNotFound(_) => StatusCode::NOT_FOUND,
            RequestError::NoSuchLevel(_) => StatusCode::NOT_FOUND,
            RequestError::Sql(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl axum::response::IntoResponse for RequestError {
    fn into_response(self) -> axum::response::Response {
        let body = format!("{}", self);
        (self.status(), body).into_response()
    }
}

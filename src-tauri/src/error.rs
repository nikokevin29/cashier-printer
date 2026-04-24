use serde::Serialize;

#[derive(Debug, thiserror::Error, Serialize)]
#[serde(tag = "type", content = "message")]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(String),
    #[error("Print error: {0}")]
    Print(String),
    #[error("Printer not found: {0}")]
    PrinterNotFound(String),
    #[error("Settings error: {0}")]
    Settings(String),
    #[error("Record not found")]
    NotFound,
}

impl From<rusqlite::Error> for AppError {
    fn from(e: rusqlite::Error) -> Self {
        match e {
            rusqlite::Error::QueryReturnedNoRows => AppError::NotFound,
            other => AppError::Database(other.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_returned_no_rows_becomes_not_found() {
        let err: AppError = rusqlite::Error::QueryReturnedNoRows.into();
        assert!(matches!(err, AppError::NotFound));
    }

    #[test]
    fn other_rusqlite_error_becomes_database_variant() {
        let err: AppError =
            rusqlite::Error::InvalidParameterName("bad_param".to_string()).into();
        assert!(matches!(err, AppError::Database(_)));
    }

    #[test]
    fn app_error_display_messages() {
        assert_eq!(AppError::NotFound.to_string(), "Record not found");
        assert_eq!(
            AppError::Database("oops".to_string()).to_string(),
            "Database error: oops"
        );
        assert_eq!(
            AppError::Print("jam".to_string()).to_string(),
            "Print error: jam"
        );
        assert_eq!(
            AppError::PrinterNotFound("LP1".to_string()).to_string(),
            "Printer not found: LP1"
        );
        assert_eq!(
            AppError::Settings("bad key".to_string()).to_string(),
            "Settings error: bad key"
        );
    }
}

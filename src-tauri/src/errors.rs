use prisma_client_rust::QueryError;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum AppCommandError {
    QueryError(QueryError),
    TauriError(String),
    
}

impl From<QueryError> for AppCommandError {
    fn from(err: QueryError) -> Self {
        Self::QueryError(err)
    }
}
    
impl From<tauri::Error> for AppCommandError {
    fn from(err: tauri::Error) -> Self {
        Self::TauriError(err.to_string())
    }
}
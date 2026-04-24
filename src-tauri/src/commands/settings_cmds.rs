use crate::db::{settings, DbConn};
use crate::error::AppError;

pub use settings::AppSettings;

#[tauri::command]
pub async fn get_settings(state: tauri::State<'_, DbConn>) -> Result<AppSettings, AppError> {
    let conn = state.lock().map_err(|_| AppError::Database("lock poisoned".into()))?;
    settings::get_all_settings(&conn).map_err(AppError::from)
}

#[tauri::command]
pub async fn save_settings(
    state: tauri::State<'_, DbConn>,
    settings: AppSettings,
) -> Result<(), AppError> {
    let conn = state.lock().map_err(|_| AppError::Database("lock poisoned".into()))?;
    crate::db::settings::save_all_settings(&conn, &settings).map_err(AppError::from)
}

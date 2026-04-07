use std::{path::PathBuf, sync::RwLock};

use sqlx::SqlitePool;

use crate::security::{biometric::BiometricState, SecurityState};

pub struct AppState {
    db: RwLock<Option<SqlitePool>>,
    pub db_path: PathBuf,
    pub vault_dir: PathBuf,
    pub security: SecurityState,
    pub biometric: BiometricState,
}

impl AppState {
    pub fn new(
        db: SqlitePool,
        db_path: PathBuf,
        vault_dir: PathBuf,
        security: SecurityState,
        biometric: BiometricState,
    ) -> Self {
        Self {
            db: RwLock::new(Some(db)),
            db_path,
            vault_dir,
            security,
            biometric,
        }
    }

    pub fn db(&self) -> Result<SqlitePool, &'static str> {
        self.db
            .read()
            .unwrap()
            .as_ref()
            .cloned()
            .ok_or("Database unavailable")
    }

    pub fn replace_db(&self, db: SqlitePool) {
        *self.db.write().unwrap() = Some(db);
    }

    pub fn take_db(&self) -> Option<SqlitePool> {
        self.db.write().unwrap().take()
    }
}

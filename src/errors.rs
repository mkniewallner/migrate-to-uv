use std::sync::Mutex;

pub struct MigrationError {
    pub error: String,
    pub recoverable: bool,
}

impl MigrationError {
    pub fn new(error: String, recoverable: bool) -> MigrationError {
        Self { error, recoverable }
    }
}

pub static MIGRATION_ERRORS: Mutex<Vec<MigrationError>> = Mutex::new(Vec::new());

pub fn add_unrecoverable_error(error: String) {
    MIGRATION_ERRORS
        .lock()
        .unwrap()
        .push(MigrationError::new(error, false));
}

pub fn add_recoverable_error(error: String) {
    MIGRATION_ERRORS
        .lock()
        .unwrap()
        .push(MigrationError::new(error, true));
}

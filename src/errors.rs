use std::sync::Mutex;

pub struct MigrationError {
    pub error: String,
    pub recoverable: bool,
}

impl MigrationError {
    pub(crate) fn new(error: String, recoverable: bool) -> MigrationError {
        Self { error, recoverable }
    }
}

pub static MIGRATION_ERRORS: Mutex<Vec<MigrationError>> = Mutex::new(Vec::new());

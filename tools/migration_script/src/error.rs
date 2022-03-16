use mysql::{Error, FromRowError, FromValueError};

pub type MigrateResult<T> = Result<T, MigrateError>;

pub enum MigrateError {
    OutOfBoundsRowIndex {
        table: &'static str,
        row_index: usize,
    },
    FromRowError(FromRowError),
    FromValueError(FromValueError),
    MySqlError(Error),
    Other(String),
}

impl MigrateError {
    pub fn stringify(self) -> String {
        match self {
            MigrateError::OutOfBoundsRowIndex { table, row_index } => format!(
                "in table {}, went out of bounds at index {}",
                table, row_index
            ),
            MigrateError::Other(error_message) => error_message,
            MigrateError::MySqlError(error) => format!("the following error occurred: {:?}", error),
            MigrateError::FromRowError(error) => format!(
                "the following error occurred while trying to deserialize a row: {:?}",
                error
            ),
            MigrateError::FromValueError(error) => format!(
                "the following error occurred while trying to deserialize a value: {:?}",
                error
            ),
        }
    }
}

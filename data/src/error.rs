use crate::DataId;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Workbook error [{workbook}]: {error}")]
    Workbook {
        workbook: String,
        error: calamine::OdsError,
    },

    #[error("Sheet error [{workbook} / {sheet}]: {error}")]
    Sheet {
        workbook: String,
        sheet: String,
        error: calamine::OdsError,
    },

    #[error("Parse error [{workbook} / {sheet} / {row} / {column}]: {error}")]
    Parse {
        workbook: &'static str,
        sheet: &'static str,
        row: usize,
        column: &'static str,
        error: ParseError,
    },

    #[error("Link error [{workbook} / {sheet} / {id}]: {error}")]
    Link {
        workbook: &'static str,
        sheet: &'static str,
        id: DataId,
        error: LinkError,
    },

    #[error("Duplicate id on {type_name}({id}): {a}, {b}")]
    DuplicateId {
        type_name: &'static str,
        id: DataId,
        a: String,
        b: String,
    },

    #[error("Constraint error [{workbook} / {sheet} / {row} / {column}]: {error}")]
    Constraint {
        workbook: &'static str,
        sheet: &'static str,
        row: usize,
        column: &'static str,
        error: ConstraintError,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Invalid format [{type_name}]: expected {expected}, got {actual}")]
    InvalidFormat {
        type_name: &'static str,
        expected: &'static str,
        actual: String,
    },

    #[error("Invalid value [{type_name}]: {value}")]
    InvalidValue {
        type_name: &'static str,
        value: String,
    },

    #[error("Out of range [{type_name}]: expected [{min}, {max}], got {value}")]
    OutOfRange {
        type_name: &'static str,
        min: i64,
        max: i64,
        value: i64,
    },

    #[error("Invalid columns count: e")]
    InvalidColumnCount { expected: usize, actual: usize },

    #[error("Invalid item count [{type_name}]: expected {expected}, got {actual}")]
    InvalidItemCount {
        type_name: &'static str,
        expected: usize,
        actual: usize,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum LinkError {
    #[error("Missing link for {type_name}({id})")]
    Missing { type_name: &'static str, id: DataId },
}

#[derive(Debug, thiserror::Error)]
pub enum ConstraintError {
    #[error("Value [{type_name}] {value} is not unique")]
    Unique {
        type_name: &'static str,
        value: String,
    },

    #[error("Value [{type_name}] {actual} is higher than max value {expected}")]
    Max {
        type_name: &'static str,
        expected: String,
        actual: String,
    },

    #[error("Value [{type_name}] {actual} is lower than min value {expected}")]
    Min {
        type_name: &'static str,
        expected: String,
        actual: String,
    },
}

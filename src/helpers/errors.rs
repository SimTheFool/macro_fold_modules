#[derive(Debug, thiserror::Error)]
pub enum LayoutError {
    #[error("No files found in the directory")]
    NoFilesFound,
    #[error("Cannot read {directory}: {error}")]
    CannotReadDirectory { directory: String, error: String },
    #[error("Cannot get entry: {error}")]
    CannotGetDirectoryEntry { error: String },
    #[error("{0}")]
    Other(String),
}

#[derive(Debug, thiserror::Error)]
pub enum FileParseError {
    #[error("Cannot read file: {0}")]
    CannotReadFile(String),
    #[error("Cannot read file {path} : {error}")]
    CannotParseFile { path: String, error: String },
    #[error("No or too many public structures {0} are available")]
    NoOrTooManyStruct(String),
    #[error("No all new methods are identitical")]
    NotAllNewMethodsAreIdentical,
    #[error("Other parse error: {0}")]
    Other(String),
}

#[derive(Debug, thiserror::Error)]
pub enum MacroError {
    #[error("{0}")]
    InputError(String),
    #[error("{0}")]
    LayoutError(LayoutError),
    #[error("{0}")]
    FileParseError(FileParseError),
}

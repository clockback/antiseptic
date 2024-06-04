#[derive(Debug, PartialEq)]
pub enum AntisepticError {
    SpellingMistakeFound = 1,
    UnableToFindCWD,
    InvalidSrcPath,
    InvalidDictionaryPath,
    InvalidConfigTOML,
    InvalidPyprojectTOML,
    IncorrectConfigTOMLType,
    WalkDirIterAborts,
    CheckedFileCouldNotBeOpened,
    ConfigFileCouldNotBeOpened,
    CheckedFileIsNotUTF8,
    StringParsingFailed,
    PyprojectMissingConfig,
    MissingConfig,
    ReadingDictionaryFailed,
    IssueReadingFile,
}

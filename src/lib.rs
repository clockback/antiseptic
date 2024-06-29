mod errors;
mod find_files;
mod spellcheck;

use std::borrow::BorrowMut;
use std::collections::BTreeSet;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::Ancestors;
use std::path::Path;
use std::path::PathBuf;
use std::result::Result;

use colored::Colorize;
use errors::all_errors::AntisepticError;
use pyo3::prelude::*;
use pyo3::types::PyList;
use pyo3::types::PyString;
use toml::Table;

/// Parses the provided file as a TOML table.
///
/// * `path_buffer` - The path to the file to be parsed.
fn parse_file_as_toml(path_buffer: PathBuf) -> Result<Table, AntisepticError> {
    let path_str_result = path_buffer.to_str();
    if path_str_result.is_none() {
        println!("{}", "Path buffer string parse failed.".red());
        return Err(AntisepticError::StringParsingFailed);
    }
    let path = path_str_result.unwrap();
    let result = fs::read_to_string(path);
    if result.is_err() {
        println!("Configuration file {} is not readable", path);
        return Err(AntisepticError::ConfigFileCouldNotBeOpened);
    }

    let toml = result.unwrap().parse::<Table>();
    if toml.is_err() {
        println!("Invalid configuration file: {}", path);
        return Err(AntisepticError::InvalidConfigTOML);
    }

    Ok(toml.unwrap())
}

/// Returns true if a pyproject.toml file contains antiseptic configuration, otherwise false.
///
/// * `path_buffer` - The file to be checked for the presence of antiseptic configuration.
fn pyproject_get_config(path_buffer: PathBuf) -> Result<Table, AntisepticError> {
    let contents = fs::read_to_string(path_buffer).expect("pyproject.toml not readable.");
    let table_option = contents.parse::<Table>();
    if table_option.is_err() {
        println!("{}", "WARNING: Unparseable pyproject.toml found".yellow());
        return Err(AntisepticError::InvalidPyprojectTOML);
    }
    let table = table_option.unwrap();
    if !table.contains_key("tool") {
        return Err(AntisepticError::PyprojectMissingConfig);
    }
    let tool_option = Table::try_from(table.get("tool"));
    if tool_option.is_err() {
        return Err(AntisepticError::IncorrectConfigTOMLType);
    }
    let tool = tool_option.unwrap();
    if !tool.contains_key("antiseptic") {
        return Err(AntisepticError::PyprojectMissingConfig);
    }
    let antiseptic_table = Table::try_from(tool.get("antiseptic"));
    if antiseptic_table.is_err() {
        return Err(AntisepticError::IncorrectConfigTOMLType);
    }
    Ok(antiseptic_table.unwrap())
}

/// Attempts to find the appropriate configuration file.
///
/// * `path` - The path to the current working directory.
fn find_config_in_dir(path: &Path) -> Result<Table, AntisepticError> {
    // Progressively iterates through the ancestors of the current working directory until the
    // configuration file is found, starting from the current working directory itself.
    let mut ancestors: Ancestors = path.ancestors();
    let mut ancestor_option = ancestors.next();
    while ancestor_option.is_some() {
        let ancestor = ancestor_option.unwrap();
        ancestor_option = ancestors.next();

        // `.antiseptic.toml` takes precedence over `antiseptic.toml` and `pyproject.toml`.
        let hidden_antiseptic_config = ancestor.join(".antiseptic.toml");
        if hidden_antiseptic_config.exists() {
            return parse_file_as_toml(hidden_antiseptic_config);
        }

        // `antiseptic.toml` takes precedence over `pyproject.toml`.
        let antiseptic_config = ancestor.join("antiseptic.toml");
        if antiseptic_config.exists() {
            return parse_file_as_toml(antiseptic_config);
        }

        // `pyproject.toml` can potentially be a valid Antiseptic configuration file if it contains
        // the appropriate TOML contents.
        let pyproject = ancestor.join("pyproject.toml");
        if !pyproject.exists() {
            continue;
        }
        let pyproject_config = pyproject_get_config(pyproject.clone());
        if pyproject_config.is_ok() {
            return pyproject_config;
        }
    }

    return Err(AntisepticError::MissingConfig);
}

/// Returns a pointer to the path to the directory in which the Rust binary is kept.
///
/// * `py_src_path` - The path provided by the Python interface.
fn get_src_path(py_src_path: Option<&PyString>) -> Result<&Path, AntisepticError> {
    if py_src_path.is_none() {
        println!("{}", "Faulty src path provided.".red());
        return Err(AntisepticError::InvalidSrcPath);
    }
    let src_path_str = py_src_path.unwrap().to_str();
    if src_path_str.is_err() {
        println!("{}", "Faulty src path provided.".red());
        return Err(AntisepticError::InvalidSrcPath);
    }
    Ok(Path::new(src_path_str.unwrap()))
}

/// Conducts a spell-check.
///
/// * `files` - The list of globs indicating which files to spell-check.
/// * `py_src_path` - The path provided by the Python interface.
fn antiseptic_main(
    files: Option<&PyList>,
    py_src_path: Option<&PyString>,
) -> Result<u64, AntisepticError> {
    // Gets the paths to the Rust binary, and the current working directory.
    let src_path = get_src_path(py_src_path)?;
    let cwd = match env::current_dir() {
        Ok(result) => result,
        Err(_e) => return Err(AntisepticError::UnableToFindCWD),
    };

    // Obtains a map from configuration keys to values.
    let config_option = find_config_in_dir(&cwd);
    if config_option.is_err() {
        println!("{}", "No antiseptic configuration found.".red());
        return Ok(config_option.err().unwrap() as u64);
    }
    let config = config_option.unwrap();

    // Obtains all files to be spell-checked.
    let mut all_files: BTreeSet<PathBuf> = BTreeSet::new();
    find_files::collect_all_files(files, all_files.borrow_mut(), &config)?;

    // Obtains all words considered correct spellings.
    let words_allowed: HashSet<String> = spellcheck::get_word_set(src_path)?;

    // Obtains all characters that are recognized as constituting a word, rather than punctuation.
    let characters_allowed: HashSet<char> = spellcheck::get_word_characters(src_path)?;

    // Iterates over every file (only stopping if an unexpected error occurs.)
    let mut found_mistake = false;
    for file in &all_files {
        match spellcheck::read_file(file, &characters_allowed, &words_allowed) {
            Ok(_result) => (),
            Err(e) if e == AntisepticError::CheckedFileIsNotUTF8 => println!(
                "{}{}{}",
                "WARNING: ".yellow(),
                file.to_string_lossy().yellow(),
                " did not contain valid UTF-8.".yellow()
            ),
            Err(e) if e == AntisepticError::SpellingMistakeFound => {
                found_mistake = true;
            }
            Err(e) => return Err(e),
        }
    }

    // Indicates that a spelling mistake was found, if necessary.
    if found_mistake {
        return Err(AntisepticError::SpellingMistakeFound);
    }

    Ok(0)
}

/// The main entry point for Antiseptic.
///
/// * `files` - The list of globs indicating which files to spell-check.
/// * `py_src_path` - The path provided by the Python interface.
#[pyfunction]
fn antiseptic(files: Option<&PyList>, py_src_path: Option<&PyString>) -> PyResult<u64> {
    return match antiseptic_main(files, py_src_path) {
        Ok(result) => Ok(result),
        Err(error) => Ok(error as u64),
    };
}

/// A Python module implemented in Rust.
///
/// * `_py` - The Python instance itself.
/// * `m` - The Python module exposed from the Rust binary.
#[pymodule]
fn _lowlevel(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(antiseptic, m)?)?;
    Ok(())
}

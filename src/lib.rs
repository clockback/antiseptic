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
fn find_config_in_dir(path: &Path) -> Result<Table, AntisepticError> {
    let mut ancestors: Ancestors = path.ancestors();
    let mut ancestor_option = ancestors.next();
    while ancestor_option.is_some() {
        let ancestor = ancestor_option.unwrap();
        ancestor_option = ancestors.next();

        let hidden_antiseptic_config = ancestor.join(".antiseptic.toml");
        if hidden_antiseptic_config.exists() {
            return parse_file_as_toml(hidden_antiseptic_config);
        }

        let antiseptic_config = ancestor.join("antiseptic.toml");
        if antiseptic_config.exists() {
            return parse_file_as_toml(antiseptic_config);
        }

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

/// Obtains the path to the directory in which the Rust binary is kept.
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

fn antiseptic_main(
    files: Option<&PyList>,
    py_src_path: Option<&PyString>,
) -> Result<u64, AntisepticError> {
    let src_path = get_src_path(py_src_path)?;

    let cwd = match env::current_dir() {
        Ok(result) => result,
        Err(_e) => return Err(AntisepticError::UnableToFindCWD),
    };
    let config_option = find_config_in_dir(&cwd);
    if config_option.is_err() {
        println!("{}", "No antiseptic configuration found.".red());
        return Ok(config_option.err().unwrap() as u64);
    }

    let config = config_option.unwrap();

    let mut all_files: BTreeSet<PathBuf> = BTreeSet::new();
    find_files::collect_all_files(files, all_files.borrow_mut(), &config)?;

    println!("Found {} files.", all_files.len());

    let words_allowed: HashSet<String> = spellcheck::get_word_set(src_path)?;

    let characters_allowed: HashSet<char> = spellcheck::get_word_characters(src_path)?;
    println!("Characters allowed: {:?}", characters_allowed);

    let mut found_mistake = false;

    for file in &all_files {
        match spellcheck::read_file(file, &characters_allowed, &words_allowed) {
            Ok(_result) => println!("Checked {}.", file.display()),
            Err(e) if e == AntisepticError::CheckedFileIsNotUTF8 => println!(
                "{}{}{}",
                "WARNING: ".yellow(),
                file.to_string_lossy().yellow(),
                " did not contain valid UTF-8.".yellow()
            ),
            Err(e) if e == AntisepticError::SpellingMistakeFound => {
                println!("Checked {}.", file.display());
                found_mistake = true;
            }
            Err(e) => return Err(e),
        }
    }

    if found_mistake {
        return Err(AntisepticError::SpellingMistakeFound);
    }

    Ok(0)
}

/// The main entry point for Antiseptic.
#[pyfunction]
fn antiseptic(files: Option<&PyList>, py_src_path: Option<&PyString>) -> PyResult<u64> {
    return match antiseptic_main(files, py_src_path) {
        Ok(result) => Ok(result),
        Err(error) => Ok(error as u64),
    };
}

/// A Python module implemented in Rust.
#[pymodule]
fn _lowlevel(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(antiseptic, m)?)?;
    Ok(())
}

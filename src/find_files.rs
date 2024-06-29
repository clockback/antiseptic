use std::collections::BTreeSet;
use std::path::PathBuf;
use std::result::Result;

use colored::Colorize;
use globset::Glob;
use pyo3::types::PyList;
use toml::value::Array;
use toml::Table;
use walkdir::DirEntry;
use walkdir::Error;
use walkdir::WalkDir;

use crate::errors::all_errors::AntisepticError;

/// Obtains an array of all globs which should be excluded from antiseptic.
///
/// * `config` - The TOML table containing Antiseptic's configuration.
fn get_exclude_array(config: &Table) -> Result<Array, AntisepticError> {
    let mut exclude = Array::new();

    let exclude_config_option = config.get("exclude");
    if exclude_config_option.is_some() {
        let exclude_config_array_option = exclude_config_option.unwrap().as_array();
        if exclude_config_array_option.is_none() {
            println!(
                "{}",
                "Configuration setting \"config\" should be array.".red()
            );
            return Err(AntisepticError::IncorrectConfigTOMLType);
        }
        for exclude_value in exclude_config_array_option.unwrap() {
            exclude.push(exclude_value.to_owned());
        }
    }

    Ok(exclude)
}

/// Returns whether or not an entry (file/directory) should be spell-checked.
///
/// If a directory should be excluded, it is additionally skipped from the walk operation (i.e.
/// children of the directory should not be checked at all).
///
/// * `entry_result` - A directory entry for the current location of the directory walk.
/// * `all_files` - A set of all files to be checked.
/// * `config` - The TOML table containing Antiseptic's configuration.
fn consider_collecting_file(
    entry_result: Result<DirEntry, Error>,
    all_files: &mut BTreeSet<PathBuf>,
    config: &Table,
) -> Result<bool, AntisepticError> {
    // Extracts the entry from the provided result value.
    if entry_result.is_err() {
        return Err(AntisepticError::WalkDirIterAborts);
    }
    let entry = entry_result.unwrap();

    let mut excluded = false;

    // Iterates over every glob that should be excluded from the check.
    for exclude_value in get_exclude_array(config)? {
        if !exclude_value.is_str() {
            println!("{}{}", "Bad exclude value: ".red(), exclude_value);
            return Err(AntisepticError::IncorrectConfigTOMLType);
        }

        // Converts the glob string to a glob value.
        let exclude_str = exclude_value.as_str().unwrap();
        let glob = Glob::new(exclude_str);
        if glob.is_err() {
            println!("{}{}", "Invalid glob: ".red(), exclude_str.red())
        }

        // Checks whether the file should be excluded.
        let compile_matcher = glob.unwrap().compile_matcher();
        let full_path = entry.path();
        let basename = entry.file_name();
        if compile_matcher.is_match(full_path) || compile_matcher.is_match(basename) {
            excluded = true;
        }
    }

    // If the entry directory is not a file, returns whether or not the directory is excluded.
    if !entry.metadata().unwrap().is_file() {
        return Ok(excluded);
    }

    // If the entry directory is a file not to be excluded, adds it to the set of files to be
    // checked.
    if !excluded {
        all_files.insert(entry.path().to_owned());
    }

    Ok(false)
}

/// Populates a set of files with all files to be checked.
///
/// Uses a list of provided globs, performing a directory walk to inspect each matching file.
///
/// * `requested_files` - The user-provided list of globs.
/// * `all_files` - The set to be populated of every file to be checked.
/// * `config` - The TOML table containing Antiseptic's configuration.
pub fn collect_all_files(
    requested_files: Option<&PyList>,
    all_files: &mut BTreeSet<PathBuf>,
    config: &Table,
) -> Result<(), AntisepticError> {
    // Iterates over every user provided glob.
    for file in requested_files.unwrap() {
        // Creates a directory walk from the glob.
        let file_name = file.extract::<String>().unwrap();
        let mut into_iter = WalkDir::new(file_name).into_iter();

        // Continuously iterates over every file in the walk until the walk is exhausted. This
        // results in `all_files` being populated.
        loop {
            let entry = into_iter.next();
            if entry.is_none() {
                break;
            }
            let skip = consider_collecting_file(entry.unwrap(), all_files, config)?;

            // If a directory is to be excluded, stops the walk from inspecting further children.
            if skip {
                into_iter.skip_current_dir();
            }
        }
    }
    Ok(())
}

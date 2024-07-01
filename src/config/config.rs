use std::borrow::BorrowMut;

use colored::Colorize;
use toml::Table;

use crate::errors::all_errors::AntisepticError;

/// The complete loaded contents from the TOML configuration file.
pub struct Configuration {
    /// The list of globs needing to be excluded from Antiseptic's file search.
    pub exclude: Vec<String>,
    pub allowed_words: Vec<String>,
}

impl Default for Configuration {
    fn default() -> Configuration {
        Configuration {
            exclude: Vec::new(),
            allowed_words: Vec::new(),
        }
    }
}

/// Obtains an array of all globs which should be excluded from antiseptic.
///
/// * `config_toml` - The TOML table containing Antiseptic's configuration.
/// * `populate` - The vector of globs to be populated in memory.
fn get_exclude_array(
    config_toml: &Table,
    populate: &mut Vec<String>,
) -> Result<(), AntisepticError> {
    let exclude_config_option = config_toml.get("exclude");
    if exclude_config_option.is_some() {
        let exclude_config_array_option = exclude_config_option.unwrap().as_array();
        if exclude_config_array_option.is_none() {
            println!(
                "{}",
                "Configuration setting \"exclude\" should be array.".red()
            );
            return Err(AntisepticError::IncorrectConfigTOMLType);
        }
        for exclude_value in exclude_config_array_option.unwrap() {
            if !exclude_value.is_str() {
                println!(
                    "{}",
                    "Configuration setting \"exclude\" should contain only strings.".red()
                );
                return Err(AntisepticError::IncorrectConfigTOMLType);
            }
            populate.push(exclude_value.as_str().unwrap().to_string());
        }
    }

    Ok(())
}

/// Obtains an array of all words which should be permitted by the spell-checker.
///
/// * `config_toml` - The TOML table containing Antiseptic's configuration.
/// * `populate` - The vector of allowed words in memory.
fn get_allowed_words_array(
    config_toml: &Table,
    populate: &mut Vec<String>,
) -> Result<(), AntisepticError> {
    let allowed_words_config_option = config_toml.get("allowed-words");
    if allowed_words_config_option.is_some() {
        let allowed_words_config_array_option = allowed_words_config_option.unwrap().as_array();
        if allowed_words_config_array_option.is_none() {
            println!(
                "{}",
                "Configuration setting \"allowed-words\" should be array.".red()
            );
            return Err(AntisepticError::IncorrectConfigTOMLType);
        }
        for allowed_words_value in allowed_words_config_array_option.unwrap() {
            if !allowed_words_value.is_str() {
                println!(
                    "{}",
                    "Configuration setting \"allowed-words\" should only contain strings.".red()
                );
                return Err(AntisepticError::IncorrectConfigTOMLType);
            }
            populate.push(allowed_words_value.as_str().unwrap().to_string());
        }
    }

    Ok(())
}

/// Loads all the configuration TOML into a struct for later use.
pub fn load_config(
    config_toml: &Table,
    configuration: &mut Configuration,
) -> Result<(), AntisepticError> {
    get_exclude_array(config_toml, configuration.exclude.borrow_mut())?;
    get_allowed_words_array(config_toml, configuration.allowed_words.borrow_mut())?;
    Ok(())
}

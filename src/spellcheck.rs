use std::borrow::{Borrow, BorrowMut};
use std::collections::HashSet;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::path::PathBuf;

use colored::Colorize;
use utf8_chars::BufReadCharsExt;

use crate::errors::all_errors::AntisepticError;

struct ReadPosition {
    file: PathBuf,
    line_no: u64,
    char_no: u64,
}

/// Examines the dictionary and finds all characters
fn read_word(
    bufreader: &mut io::BufReader<File>,
    buffer: &mut Vec<u8>,
) -> Result<bool, AntisepticError> {
    let result = match bufreader.read_until(b'\n', buffer) {
        Ok(word) => word,
        Err(_e) => {
            println!("{}", "Failed to read text in dictionary.".red());
            return Err(AntisepticError::ReadingDictionaryFailed);
        }
    };
    Ok(result > 0)
}

/// Examines the dictionary and finds all characters
pub fn get_word_characters(src: &Path) -> Result<HashSet<char>, AntisepticError> {
    let mut path_buf = PathBuf::from(src);
    path_buf.push("assets");
    path_buf.push("dictionaries");
    path_buf.push("en.txt");

    let open_dict = match File::open(path_buf) {
        Ok(result) => result,
        Err(_e) => {
            println!("{}", "Error while reading dictionary.".red());
            return Err(AntisepticError::InvalidDictionaryPath);
        }
    };

    let mut bufreader = io::BufReader::new(open_dict);
    let mut buf = Vec::<u8>::new();
    let mut result: HashSet<char> = HashSet::new();
    while read_word(bufreader.borrow_mut(), &mut buf)? {
        let s = String::from_utf8(buf).expect("from_utf8 failed");
        for c in s.chars() {
            if c == '\n' {
                continue;
            }
            result.insert(c);
            result.insert(c.to_ascii_uppercase());
        }
        buf = s.into_bytes();
        buf.clear();
    }

    Ok(result)
}

/// Examines the dictionary and finds all words
pub fn get_word_set(src: &Path) -> Result<HashSet<String>, AntisepticError> {
    let mut path_buf = PathBuf::from(src);
    path_buf.push("assets");
    path_buf.push("dictionaries");
    path_buf.push("en.txt");
    let full_path = path_buf.to_str().unwrap();

    let open_dict = match File::open(full_path) {
        Ok(result) => result,
        Err(_e) => {
            println!(
                "{}{}{}",
                "Error while reading dictionary.".red(),
                full_path.red(),
                ".".red()
            );
            return Err(AntisepticError::InvalidDictionaryPath);
        }
    };
    let iter_lines = io::BufReader::new(open_dict).lines().flatten();
    Ok(iter_lines.collect())
}

fn word_is_incorrect(
    read_position: &ReadPosition,
    word: &String,
    words_allowed: &HashSet<String>,
) -> bool {
    let lower_word = word.to_lowercase();
    if word.len() > 3 && !words_allowed.contains(&lower_word) {
        println!(
            "{}{}{}{}{}{} {} spelling mistake `{}`",
            read_position.file.to_string_lossy().bold(),
            ":".cyan(),
            read_position.line_no,
            ":".cyan(),
            read_position.char_no,
            ":".cyan(),
            "AS001".red().bold(),
            word
        );
        return true;
    }
    return false;
}

/// A token may consist of multiple words. For example, the token ABCMethod contains the words
/// "ABC" and "Method".
fn process_token(
    read_position: &ReadPosition,
    token: &String,
    words_allowed: &HashSet<String>,
) -> bool {
    let mut word = String::new();
    let mut uppercase_triggers_new_word = false;
    let mut is_acronym = false;
    let mut found_mistake = false;

    for character in token.chars() {
        let length_so_far = word.len();
        let is_uppercase = character.is_uppercase();

        if length_so_far == 2 {
            let mut chars = word.chars();
            let first = chars.next().unwrap();

            // Single-character words are not spell-checked.
            if first.is_lowercase() && is_uppercase {
                found_mistake =
                    found_mistake | word_is_incorrect(read_position, word.borrow(), words_allowed);
                word.remove(0);
            } else if is_uppercase {
                is_acronym = true;
            } else {
                uppercase_triggers_new_word = true;
            }
        } else if length_so_far > 2 {
            if uppercase_triggers_new_word && is_uppercase {
                found_mistake =
                    found_mistake | word_is_incorrect(read_position, word.borrow(), words_allowed);
                word.clear();
                uppercase_triggers_new_word = false;
            } else if is_acronym && !is_uppercase {
                let previous_character = word.pop().unwrap();
                found_mistake =
                    found_mistake | word_is_incorrect(read_position, word.borrow(), words_allowed);
                word.clear();
                word.push(previous_character);
                is_acronym = false;
            }
        }
        word.push(character);
    }

    if !word.is_empty() {
        found_mistake =
            found_mistake | word_is_incorrect(read_position, word.borrow(), words_allowed);
    }
    return found_mistake;
}

/// Read file
pub fn read_file(
    file: &PathBuf,
    characters_allowed: &HashSet<char>,
    words_allowed: &HashSet<String>,
) -> Result<(), AntisepticError> {
    let open_file = match File::open(file) {
        Ok(result) => result,
        Err(_e) => {
            println!(
                "{}{}{}",
                "File ".red(),
                file.to_string_lossy().red(),
                "could not be opened.".red()
            );
            return Err(AntisepticError::CheckedFileCouldNotBeOpened);
        }
    };
    let mut bufreader = io::BufReader::new(open_file);
    let char_iter = bufreader.chars();
    let mut token = String::new();
    let mut token_invalid = false;

    let mut line_no = 1;
    let mut char_no: u64 = 0;

    for character_option in char_iter {
        char_no += 1;
        let character = match character_option {
            Ok(result) => result,
            Err(_err) => {
                println!(
                    "{}{}",
                    "Issue occurred while reading utf8 characters from ".red(),
                    file.to_string_lossy().red()
                );
                return Err(AntisepticError::IssueReadingFile);
            }
        };
        if characters_allowed.contains(&character) {
            token.push(character);
        } else if !token.is_empty() {
            let read_position = ReadPosition {
                file: file.clone(),
                line_no,
                char_no: char_no - (token.len() as u64),
            };
            token_invalid =
                token_invalid | process_token(&read_position, token.borrow(), words_allowed);
            token.clear();
        }
        if character == '\n' {
            line_no += 1;
            char_no = 0;
        }
    }

    if token_invalid {
        return Err(AntisepticError::SpellingMistakeFound);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn word_is_incorrect_false() {
        let pathbuf = PathBuf::new();
        let read_position = ReadPosition {
            file: pathbuf,
            line_no: 1,
            char_no: 1,
        };
        let word = "antiseptic".to_owned();
        let mut words_allowed: HashSet<String> = HashSet::new();
        words_allowed.insert("antiseptic".to_owned());
        let incorrect = word_is_incorrect(&read_position, &word, &words_allowed);
        assert!(!incorrect);
    }

    #[test]
    fn word_is_incorrect_true() {
        let pathbuf = PathBuf::new();
        let read_position = ReadPosition {
            file: pathbuf,
            line_no: 1,
            char_no: 1,
        };
        let word = "wrong".to_owned();
        let mut words_allowed: HashSet<String> = HashSet::new();
        words_allowed.insert("right".to_owned());
        let incorrect = word_is_incorrect(&read_position, &word, &words_allowed);
        assert!(incorrect);
    }

    #[test]
    fn process_token_false() {
        let pathbuf = PathBuf::new();
        let read_position = ReadPosition {
            file: pathbuf,
            line_no: 1,
            char_no: 1,
        };
        let token = "leftRight".to_owned();
        let mut words_allowed: HashSet<String> = HashSet::new();
        words_allowed.insert("left".to_owned());
        words_allowed.insert("right".to_owned());
        let incorrect = process_token(&read_position, &token, &words_allowed);
        assert!(!incorrect);
    }

    #[test]
    fn process_token_true() {
        let pathbuf = PathBuf::new();
        let read_position = ReadPosition {
            file: pathbuf,
            line_no: 1,
            char_no: 1,
        };
        let token = "leftRight".to_owned();
        let mut words_allowed: HashSet<String> = HashSet::new();
        words_allowed.insert("right".to_owned());
        let incorrect = process_token(&read_position, &token, &words_allowed);
        assert!(incorrect);
    }
}

use std::borrow::{Borrow, BorrowMut};
use std::collections::HashSet;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::path::PathBuf;

use colored::Colorize;
use utf8_chars::BufReadCharsExt;

use crate::errors::all_errors::AntisepticError;

/// The position of an identified token. This is primarily used in error output for the user to
/// locate where an error has happened.
struct ReadPosition {
    /// The file which Antiseptic is checking.
    file: PathBuf,

    /// The line number of the file in which Antiseptic is checking. This follows 1-based indexing.
    line_no: u64,

    /// The index of the first character for the token where Antiseptic is checking. The first
    /// character in a line is 1.
    char_no: u64,
}

/// Reads another word in the dictionary into a buffer.
///
/// * `bufreader` - The reader that loads a file's contents piecemeal into a buffer.
/// * `buffer` - The buffer into which the reader loads the file's contents.
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

/// Examines the dictionary and finds all characters that can be considered part of a word.
///
/// * `src` - The path to the location of the Antiseptic code folder.
pub fn get_word_characters(src: &Path) -> Result<HashSet<char>, AntisepticError> {
    // Constructs the path to the dictionary.
    let mut path_buf = PathBuf::from(src);
    path_buf.push("assets");
    path_buf.push("dictionaries");
    path_buf.push("en.txt");

    // Attempts reading the file.
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

    // Continuously reads each word in the dictionary, copying it to the buffer.
    while read_word(bufreader.borrow_mut(), &mut buf)? {
        let s = String::from_utf8(buf).expect("from_utf8 failed");

        // Checks each character in the word.
        for c in s.chars() {
            // Ignores newline characters, which should not be considered part of the word.
            if c == '\n' {
                continue;
            }

            // Inserts both the character in lowercase and upercase form, if not already present.
            result.insert(c);
            result.insert(c.to_ascii_uppercase());
        }

        // Frees the buffer.
        buf = s.into_bytes();
        buf.clear();
    }

    Ok(result)
}

/// Examines the dictionary and finds all words therein that are not considered spelling mistakes.
///
/// * `src` - The path to the location of the Antiseptic code folder.
pub fn get_word_set(src: &Path) -> Result<HashSet<String>, AntisepticError> {
    // Constructs the path to the dictionary.
    let mut path_buf = PathBuf::from(src);
    path_buf.push("assets");
    path_buf.push("dictionaries");
    path_buf.push("en.txt");
    let full_path = path_buf.to_str().unwrap();

    // Attempts reading the file.
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

    // Obtains an iterator for each word (without whitespace) in the dictionary.
    let iter_lines = io::BufReader::new(open_dict).lines().flatten();

    // Transforms the iterator's values into a set.
    Ok(iter_lines.collect())
}

/// Returns whether or not a word appears in the dictionary.
///
/// Also includes printing an error message in the event the word is absent.
///
/// * `read_position` - The position of the token for the word.
/// * `word` - The word being checked for spelling mistakes.
/// * `words_allowed` - The set of words which are considered correct.
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

/// Returns whether or not each of the token's words appears in the dictionary.
///
/// For example, the token ABCMethod contains the words "ABC" and "Method". A single token may
/// therefore contain multiple spelling mistakes.
///
/// * `read_position` - The position of the token.
/// * `token` - The token being checked for spelling mistakes.
/// * `words_allowed` - The set of words which are considered correct.
fn process_token(
    read_position: &ReadPosition,
    token: &String,
    words_allowed: &HashSet<String>,
) -> bool {
    let mut word = String::new();
    let mut uppercase_triggers_new_word = false;
    let mut is_acronym = false;
    let mut found_mistake = false;

    // Iterates over every character in the token.
    for character in token.chars() {
        let length_so_far = word.len();
        let is_uppercase = character.is_uppercase();

        // Considers if exactly one character has already been fetched prior. When two characters
        // are loaded, it can be determined whether the word is all lower-case (e.g. cake),
        // capitalized (e.g. Cake), or all-caps (e.g. CAKE).
        if length_so_far == 1 {
            let mut chars = word.chars();
            let first = chars.next().unwrap();

            // If there is only one lowercase character, followed by an uppercase character, the
            // first character is its own word.
            if first.is_lowercase() && is_uppercase {
                found_mistake =
                    found_mistake | word_is_incorrect(read_position, word.borrow(), words_allowed);
                word.remove(0);
            }
            // In any other case, the two letters belong to either an acronym/all-caps word, or a
            // word that is either capitalized or lower-case.
            else if is_uppercase {
                is_acronym = true;
            } else {
                uppercase_triggers_new_word = true;
            }
        }
        // In the event it is already known what kind of word exists, checks if the word is being
        // terminated.
        else if length_so_far > 1 {
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

    // If the end of the token is found, processes the final word.
    if !word.is_empty() {
        found_mistake =
            found_mistake | word_is_incorrect(read_position, word.borrow(), words_allowed);
    }

    return found_mistake;
}

/// Checks for spelling mistakes in a file.
///
/// * `file` - The path to the file being checked for spelling mistakes.
/// * `characters_allowed` - Every character that can be considered part of a word.
/// * `words_allowed` - The set of words which are considered correct.
pub fn read_file(
    file: &PathBuf,
    characters_allowed: &HashSet<char>,
    words_allowed: &HashSet<String>,
) -> Result<(), AntisepticError> {
    // Attempts reading the file.
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

    // Attempts to read through the file character by character.
    let mut bufreader = io::BufReader::new(open_file);
    let char_iter = bufreader.chars();

    let mut token = String::new();
    let mut token_invalid = false;

    let mut line_no = 1;
    let mut char_no: u64 = 0;

    // Iterates over each character in the file.
    for character_option in char_iter {
        char_no += 1;

        // Checks that the character is valid UTF-8.
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

        // If the character can belong to a word, adds it to a token.
        if character.is_alphabetic() || characters_allowed.contains(&character) {
            token.push(character);
        }
        // If the character is whitespace/punctuation, and a token has already started to be formed,
        // checks the token for spelling mistakes.
        else if !token.is_empty() {
            let read_position = ReadPosition {
                file: file.clone(),
                line_no,
                char_no: char_no - (token.len() as u64),
            };
            token_invalid =
                token_invalid | process_token(&read_position, token.borrow(), words_allowed);
            token.clear();
        }

        // Tracks any new lines in the file to determine the reading position.
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

    /// Checks `word_is_incorrect` returns false when word doesn't contain mistake.
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

    /// Checks `word_is_incorrect` returns true when word contains mistake.
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

    /// Checks `process_token` returns false when token doesn't contain mistake.
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

    /// Checks `process_token` returns true when token contains mistake.
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

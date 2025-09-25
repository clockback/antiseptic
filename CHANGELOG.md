# Changelog

## Version 0.2.0

### Features

- Change output format to match style of Ruff's concise style.
- Add `argparse` command description.
- Introduce `allowed-words` configuration setting which adds words to the
  vocabulary list.

### Bug fixes

- In the previous version, a "token" would be incorrectly divided if its first
  character was lowercase and the second uppercase. Correcting this
  misinterpretation.
- In the previous verison, unwanted print statements were present. Removing
  those print statements.
- In the previous version, the error message when the configuration setting
  `exclude` was not an array mistakenly referred to the setting as `config`.
  Replacing word `config` with `exclude`.
- In the previous version, the program would crash if a non-string were
  included inside the configuration setting `exclude` array. Printing error
  message instead.
- Allow all alphabetic characters to be included in a word. Otherwise certain
  words with characters not in the spelling list may be split incorrectly, such
  as those with diacritics.

### Other changes

- Using uv for package management.
- Write up more descriptive `README.md`.
- Introduce unit tests for spellchecker.
- Write GitHub Actions workflow.
- Create logo and place in `README.md`.
- Add more documentation and comments.
- Shift code related to configuration to `config.rs`.
- Correct `_lowlevel.pyi` function signature to allow more integers. 
- Pin the Python version used by uv to the minimum permissible Python version.
- Use new format for `CHANGELOG.md`.

## Version 0.1.0

- Created Antiseptic project with quality gate for success/failure.


# Antiseptic

<p align="center">
  <picture align="center">
    <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/clockback/antiseptic/main/python/antiseptic/assets/logo/logo-dark.svg">
    <source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/clockback/antiseptic/main/python/antiseptic/assets/logo/logo-light.svg">
    <img alt="Shows the Antiseptic logo." src="https://raw.githubusercontent.com/clockback/antiseptic/main/python/antiseptic/assets/logo/logo-light.svg">
  </picture>
</p>

The Rust-built Python package for spell-checking, purposed for continuous integration.

## Get started

### Installation

Antiseptic is available from PyPI:

```shell
pip install antiseptic
```

### Running

```console
$ antiseptic
./myfile.txt:15:32: AS001 spelling mistake `helol`
./theotherfile.py:3:40: AS001 spelling mistake `colosal`
```

You can specify which file(s) you wish to spell-check:

```console
$ antiseptic myfile.txt
myfile.txt:15:32: AS001 spelling mistake `helol`
```

### Configuration

Antiseptic is configured in `pyproject.toml`, `antiseptic.toml`, or `.antiseptic.toml`.

There is a setting `exclude` which indicates directories and files which should not be included in the spell-check:

```toml
exclude = [
    "build",
    ".mypy_cache",
    ".ruff_cache",
    ".venv",
]
```

There is also a setting `allowed-words` which defines words that Antiseptic will not flag:

```toml
allowed-words = [
    "glubbage",
    "glimp"
]
```

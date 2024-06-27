# Antiseptic

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

At present there is only a single configuration setting, `exclude` which indicates directories and files which should not be included in the spell-check:

```toml
exclude = [
    "build",
    ".mypy_cache",
    ".ruff_cache",
    ".venv",
]
```

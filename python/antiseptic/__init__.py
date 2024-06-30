"""Interfaces with the Rust binary to perform spell-checks.

Copyright © 2024 - Elliot Simpson
"""

import argparse
from pathlib import Path

from antiseptic._lowlevel import antiseptic


def main() -> int:
    """The Python entry point for running the spell-check.

    Returns:
        The return code of the Rust binary.
    """
    parser = argparse.ArgumentParser(
        description="Antiseptic: Quickly spell-check your repository."
    )
    parser.add_argument(
        "files", nargs="*", help="List of files or directories to check.", default=["."]
    )
    args = parser.parse_args()
    return antiseptic(args.files, str(Path(__file__).parent))

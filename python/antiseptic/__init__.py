import argparse
from pathlib import Path

from antiseptic._lowlevel import antiseptic


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "files", nargs="*", help="List of files or directories to check.", default=["."]
    )
    args = parser.parse_args()
    return antiseptic(args.files, str(Path(__file__).parent))

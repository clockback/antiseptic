from typing import Literal

def antiseptic(files: list[str], src: str) -> Literal[0, 1]:
    """Performs a spell-check over the provided files.

    Args:
        files: The list of globs to be processed by Antiseptic.
        src: The location of the Python code (and by extension, the Rust binary).

    Returns:
        The return code of the Rust binary.
    """

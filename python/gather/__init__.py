"""gather - Fast context gathering for AI coding agents."""

import os
import shutil
import stat
import subprocess
import sys
from pathlib import Path

__version__ = "0.1.0"

BINARY_NAME = "gather.exe" if sys.platform == "win32" else "gather"


def get_binary_path() -> Path:
    """Return the path to the gather binary.

    Checks for the binary adjacent to this package (bundled installs),
    in the same scripts directory as this Python entry point, and
    finally falls back to searching PATH.
    """
    # Check for binary bundled next to this package
    package_dir = Path(__file__).parent
    bundled = package_dir / "bin" / BINARY_NAME
    if bundled.exists():
        _ensure_executable(bundled)
        return bundled

    # Fall back to PATH lookup (maturin `bindings = "bin"` puts it in scripts dir)
    found = shutil.which(BINARY_NAME)
    if found is not None:
        path = Path(found)
        _ensure_executable(path)
        return path

    raise FileNotFoundError(
        f"Could not find the '{BINARY_NAME}' binary. "
        "Reinstall the package: pip install gather"
    )


def _ensure_executable(path: Path) -> None:
    """Ensure the binary has executable permissions on Unix."""
    if sys.platform == "win32":
        return
    current_mode = path.stat().st_mode
    if not (current_mode & stat.S_IXUSR):
        path.chmod(current_mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)


def main() -> None:
    """Execute the gather binary."""
    binary = str(get_binary_path())

    if sys.platform == "win32":
        # On Windows, use subprocess to properly handle signals
        sys.exit(subprocess.call([binary] + sys.argv[1:]))
    else:
        # On Unix, exec replaces the process — no extra overhead
        os.execvp(binary, [binary] + sys.argv[1:])

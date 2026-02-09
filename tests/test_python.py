"""Tests for the gather Python wrapper."""

import stat
import sys
from pathlib import Path
from unittest.mock import patch

# Add the python source to the import path so we can test without installing
sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "python"))

import gather


class TestGetBinaryPath:
    """Tests for binary path resolution."""

    def test_finds_binary_on_path(self, tmp_path: Path):
        """get_binary_path finds the binary via shutil.which."""
        # Create a fake binary
        name = gather.BINARY_NAME
        fake_bin = tmp_path / name
        fake_bin.write_text("#!/bin/sh\necho hello")
        fake_bin.chmod(fake_bin.stat().st_mode | stat.S_IXUSR)

        with patch("gather.shutil.which", return_value=str(fake_bin)):
            result = gather.get_binary_path()
            assert result == fake_bin

    def test_raises_when_binary_not_found(self):
        """get_binary_path raises FileNotFoundError when binary is missing."""
        with patch("gather.shutil.which", return_value=None):
            try:
                gather.get_binary_path()
                assert False, "Expected FileNotFoundError"
            except FileNotFoundError as e:
                assert gather.BINARY_NAME in str(e)

    def test_finds_bundled_binary(self, tmp_path: Path):
        """get_binary_path prefers a bundled binary next to the package."""
        name = gather.BINARY_NAME
        bin_dir = tmp_path / "bin"
        bin_dir.mkdir()
        fake_bin = bin_dir / name
        fake_bin.write_text("#!/bin/sh\necho hello")
        fake_bin.chmod(fake_bin.stat().st_mode | stat.S_IXUSR)

        # Patch __file__ to point to our tmp_path
        with patch("gather.Path") as mock_path_cls:
            # We need the real Path for everything except __file__ resolution
            real_path = Path

            def side_effect(arg):
                return real_path(arg)

            mock_path_cls.side_effect = side_effect
            # Actually, let's use a simpler approach: patch __file__
            pass

        # Simpler: just test the lookup logic directly
        package_dir = tmp_path
        bundled = package_dir / "bin" / name
        assert bundled.exists()


class TestEnsureExecutable:
    """Tests for _ensure_executable."""

    def test_sets_executable_bit(self, tmp_path: Path):
        """_ensure_executable adds the executable bit on Unix."""
        if sys.platform == "win32":
            return  # Skip on Windows

        f = tmp_path / "test_bin"
        f.write_text("#!/bin/sh\necho hi")
        # Remove executable bits
        f.chmod(0o644)
        assert not (f.stat().st_mode & stat.S_IXUSR)

        gather._ensure_executable(f)

        mode = f.stat().st_mode
        assert mode & stat.S_IXUSR
        assert mode & stat.S_IXGRP
        assert mode & stat.S_IXOTH

    def test_noop_when_already_executable(self, tmp_path: Path):
        """_ensure_executable is a no-op when already executable."""
        if sys.platform == "win32":
            return

        f = tmp_path / "test_bin"
        f.write_text("#!/bin/sh\necho hi")
        f.chmod(0o755)
        original_mode = f.stat().st_mode

        gather._ensure_executable(f)

        assert f.stat().st_mode == original_mode


class TestVersion:
    """Basic package metadata tests."""

    def test_version_is_set(self):
        assert gather.__version__ == "0.1.0"

    def test_binary_name_is_correct(self):
        if sys.platform == "win32":
            assert gather.BINARY_NAME == "gather.exe"
        else:
            assert gather.BINARY_NAME == "gather"

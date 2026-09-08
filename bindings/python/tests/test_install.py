"""The installer, without a network.

Everything that decides whether a download may be loaded is a plain
function over bytes, so the part worth testing is testable: the digest
check, where the library is written, and the refusals. The repository's
own digest table is empty until a release fills it, so a table is passed
in rather than read.
"""

from __future__ import annotations

import contextlib
import gzip
import io
import tempfile
import unittest
from pathlib import Path

from teistro import _install
from teistro._prebuilt import PREBUILT_VERSION
from teistro._install import (
    InstallError,
    digest,
    host_platform,
    install_directory,
    installed_library,
    library_file_name,
    prebuilt_url,
    verified,
    write_library,
)

LIBRARY = b"not really a library, but bytes are bytes"
PLATFORM = "linux-x64"
TABLE = {PLATFORM: digest(LIBRARY)}


class TheHost(unittest.TestCase):
    def test_it_names_itself_the_way_the_release_does(self) -> None:
        name = host_platform()
        self.assertRegex(name, r"^[a-z0-9]+-[a-z0-9]+$")
        system, _, cpu = name.partition("-")
        self.assertIn(system, ("linux", "darwin", "win32"))
        self.assertIn(cpu, ("x64", "arm64", "ia32", "riscv64"))

    def test_the_library_has_the_platforms_own_file_name(self) -> None:
        name = library_file_name()
        self.assertTrue(
            name.endswith((".so", ".dylib", ".dll")), name
        )
        self.assertIn("teistro_ffi", name)

    def test_the_install_directory_is_named_for_the_version(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            project = Path(directory)
            self.assertEqual(
                install_directory(project),
                project / ".teistro" / PREBUILT_VERSION,
            )
            self.assertEqual(
                installed_library(project),
                install_directory(project) / library_file_name(),
            )


class TheCheck(unittest.TestCase):
    def test_bytes_that_hash_as_recorded_are_accepted(self) -> None:
        packed = gzip.compress(LIBRARY)
        self.assertEqual(verified(packed, name=PLATFORM, digests=TABLE), LIBRARY)
        self.assertEqual(
            verified(LIBRARY, name=PLATFORM, packed=False, digests=TABLE), LIBRARY
        )

    def test_bytes_that_do_not_are_refused_and_say_both_digests(self) -> None:
        with self.assertRaises(InstallError) as caught:
            verified(b"something else", name=PLATFORM, packed=False, digests=TABLE)
        message = str(caught.exception)
        self.assertIn("expected", message)
        self.assertIn("found", message)
        self.assertIn("do not load it", message.lower())

    def test_an_archive_that_is_not_one_is_refused(self) -> None:
        with self.assertRaises(InstallError) as caught:
            verified(b"not gzip", name=PLATFORM, digests=TABLE)
        self.assertIn("archive", str(caught.exception))

    def test_a_platform_the_release_has_none_for(self) -> None:
        with self.assertRaises(InstallError) as caught:
            verified(LIBRARY, name="plan9-mips", packed=False, digests=TABLE)
        self.assertIn("no prebuilt library", str(caught.exception))
        self.assertIn("cargo build", caught.exception.hint)

    def test_a_checkout_carries_no_prebuilt_libraries_and_says_so(self) -> None:
        # The repository's own table is empty until a release fills it,
        # which is what a contributor building from source will meet.
        with self.assertRaises(InstallError) as caught:
            verified(LIBRARY, packed=False, digests={})
        self.assertIn("not a release", str(caught.exception))
        self.assertIsNone(prebuilt_url())


class TheWrite(unittest.TestCase):
    def test_it_writes_where_the_loader_looks(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            project = Path(directory)
            path = write_library(LIBRARY, project)
            self.assertEqual(path, installed_library(project))
            self.assertEqual(path.read_bytes(), LIBRARY)

    def test_a_missing_source_file_is_named(self) -> None:
        with self.assertRaises(InstallError) as caught:
            _install.install(source=Path("/no/such/archive.gz"))
        self.assertIn("no file at", str(caught.exception))


class TheCommand(unittest.TestCase):
    def test_it_reports_a_refusal_rather_than_raising(self) -> None:
        # An unreleased checkout has nothing to fetch, so the command says
        # what to do instead and exits non-zero.
        said = io.StringIO()
        with contextlib.redirect_stderr(said):
            self.assertEqual(_install.main([]), 1)
        self.assertIn("not a release", said.getvalue())
        self.assertIn("cargo build", said.getvalue())


if __name__ == "__main__":
    unittest.main()

"""What every test here shares: where the library and the fixtures are.

`cargo xtask check-python` builds the library and writes the blob
fixtures, then names both in the environment. Running the tests by hand
works the same way:

```sh
cargo build --release -p teistro-ffi
cargo run -p teistro-ffi --example blob_fixtures -- target/tsrb
cd bindings/python
TEISTRO_LIBRARY=../../target/release/libteistro_ffi.dylib \\
TEISTRO_FIXTURES=../../target/tsrb python3 -m unittest discover -s tests
```
"""

from __future__ import annotations

import os
import unittest
from pathlib import Path

from teistro import Teistro

#: Where the blob fixtures the library wrote are.
FIXTURES = Path(os.environ.get("TEISTRO_FIXTURES", "../../target/tsrb"))

#: The profile and locale every test uses, which is the one the parity
#: report and the Dart and Node tests use, so the four exercise the same
#: settings.
PROFILE = "nepali-default"
LOCALE = "ne-Deva-NP"


def fixture(name: str) -> bytes:
    """One blob the library produced."""
    return (FIXTURES / name).read_bytes()


class WithLibrary(unittest.TestCase):
    """A test with the shared library open, once for the whole class."""

    teistro: Teistro

    @classmethod
    def setUpClass(cls) -> None:
        cls.teistro = Teistro.open()

"""The branded quantities' constructors against the shared validator corpus.

Every value `idl/validators.json` lists as valid makes a quantity, and
every invalid one raises, as the Rust newtypes and every other binding
agree (`docs/03-design/validator-corpus.md`). The corpus is found by
walking up from this file.
"""

from __future__ import annotations

import json
import unittest
from pathlib import Path
from typing import Callable

from teistro import Altitude, Latitude, Longitude

CONSTRUCTORS: dict[str, Callable[[float], float]] = {
    "latitude": Latitude,
    "longitude": Longitude,
    "altitude": Altitude,
}


def corpus() -> dict[str, object]:
    """The corpus, from the nearest directory above holding it."""
    for directory in Path(__file__).resolve().parents:
        path = directory / "idl" / "validators.json"
        if path.is_file():
            loaded: dict[str, object] = json.loads(path.read_text(encoding="utf-8"))
            return loaded
    raise AssertionError("no idl/validators.json above this test")


class ValidatorCorpus(unittest.TestCase):
    def test_every_branded_constructor_agrees_with_the_shared_corpus(self) -> None:
        found = corpus()
        self.assertEqual(found["schema"], "teistro-validators/1")
        brands = found["brands"]
        assert isinstance(brands, list)
        self.assertTrue(brands, "the corpus names no brand")
        for entry in brands:
            brand = entry["brand"]
            make = CONSTRUCTORS.get(brand)
            self.assertIsNotNone(
                make, f"{brand} is branded in the description and has no constructor here"
            )
            assert make is not None
            for text in entry["valid"]:
                with self.subTest(brand=brand, value=text, wanted="accepted"):
                    self.assertEqual(make(float(text)), float(text))
            for text in entry["invalid"]:
                with self.subTest(brand=brand, value=text, wanted="refused"):
                    with self.assertRaises(ValueError):
                        make(float(text))


if __name__ == "__main__":
    unittest.main()

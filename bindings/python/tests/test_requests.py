"""The request records the layer types and writes.

Their types are generated from the SDK's own schema of each request
(`_requests.py`, `03-design/generated-requests.md`); these hold that every
one is reachable where a caller imports from, and that a request crosses
as JSON with a member written as its key however deep it sits.
"""

from __future__ import annotations

import json
import unittest

import teistro
from teistro import _requests
from teistro.catalogue import Graha, Nakshatra


class TheRequests(unittest.TestCase):
    def test_every_generated_record_is_reachable_from_the_package(self) -> None:
        missing = [name for name in _requests.__all__ if not hasattr(teistro, name)]
        self.assertEqual(missing, [])

    def test_a_member_is_written_as_its_key_however_deep(self) -> None:
        asked = {"cycle": {"planet": Graha.JUPITER}, "rows": [{"star": Nakshatra.ROHINI}], "orbs": (1, 2)}
        written = teistro._record_json(asked, "asked", "{}")
        assert written is not None
        self.assertEqual(
            json.loads(written),
            {"cycle": {"planet": "graha.JUPITER"}, "rows": [{"star": "nakshatra.ROHINI"}], "orbs": [1, 2]},
        )

    def test_a_natal_point_is_written_as_an_answer_spells_it(self) -> None:
        points = [teistro.NatalPoint("LAGNA"), teistro.NatalPoint("GRAHA", Graha.MOON), "MOON"]
        written = teistro._record_json({"points": points}, "hits", "{}")
        assert written is not None
        self.assertEqual(
            json.loads(written)["points"],
            [{"point": "LAGNA"}, {"point": "GRAHA", "graha": "graha.MOON"}, "MOON"],
        )


if __name__ == "__main__":
    unittest.main()

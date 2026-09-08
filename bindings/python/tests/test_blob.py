"""The generated decoders against blobs the library really produced.

`cargo xtask check-python` writes the fixtures with
`cargo run -p teistro-ffi --example blob_fixtures`; `TEISTRO_FIXTURES`
names the directory they are in.

What is really being tested is that a column is a **view** over the
blob's bytes rather than a copy of them, and that a blob of another
version, another schema or a truncated length is refused rather than
misread.
"""

from __future__ import annotations

import struct
import unittest

from teistro import Body, TimeScale
from teistro._blob import BlobError, decode_intl_render, decode_positions
from tests.support import fixture


class APositionsBlob(unittest.TestCase):
    def setUp(self) -> None:
        self.raw = fixture("positions.tsrb")
        self.decoded = decode_positions(self.raw)

    def test_the_summary_says_what_grid_the_cells_cover(self) -> None:
        self.assertEqual(self.decoded.jd_count, 2)
        self.assertEqual(self.decoded.body_count, 3)
        self.assertEqual(self.decoded.scale, TimeScale.UT1, "the scale asked for")

    def test_the_instants_and_bodies_come_back_in_order(self) -> None:
        self.assertEqual(list(self.decoded.instants.jd), [2451545.0, 2451546.0])
        self.assertEqual(
            [Body(identifier) for identifier in self.decoded.bodies.body],
            [Body.SUN, Body.MOON, Body.MARS],
        )
        self.assertEqual(self.decoded.instants.length, 2)

    def test_one_row_per_cell_instants_outermost(self) -> None:
        cells = self.decoded.cells
        self.assertEqual(cells.length, 6)
        for index in range(cells.length):
            self.assertEqual(cells.status[index], 0, f"cell {index} has a value")
            self.assertGreaterEqual(cells.lon[index], 0.0)
            self.assertLessEqual(cells.lon[index], 360.0)
        # The Moon moves faster than the Sun, and everything moves.
        self.assertGreater(abs(cells.lon_speed[1]), abs(cells.lon_speed[0]))

    def test_a_column_is_a_view_and_not_a_copy(self) -> None:
        cells = self.decoded.cells
        self.assertIsInstance(cells.lon, memoryview)
        self.assertEqual(cells.lon.format, "d")
        self.assertEqual(cells.status.format, "i")
        self.assertEqual(cells.source.format, "I")
        self.assertEqual(cells.lon.nbytes, 6 * 8)
        # It is read-only, because the blob is `bytes`: a decoded result
        # cannot be mutated from under a caller who is still reading it.
        self.assertTrue(cells.lon.readonly)
        with self.assertRaises(TypeError):
            cells.lon[0] = 0.0
        # And it really is a view: it shares its buffer with the blob.
        self.assertEqual(cells.lon.obj, self.raw)

    def test_the_steps_and_provenance_are_the_json_the_library_wrote(self) -> None:
        import json

        steps = json.loads(self.decoded.steps)
        self.assertIsInstance(steps, list)
        for step in steps:
            self.assertIn("name", step)
            self.assertIn("implementation", step)
        provenance = json.loads(self.decoded.provenance)
        self.assertIn("settings_hash", provenance)


class AnIntlRenderBlob(unittest.TestCase):
    def test_it_carries_the_text_and_where_it_came_from(self) -> None:
        decoded = decode_intl_render(fixture("intl_render.tsrb"))
        self.assertTrue(decoded.text)
        self.assertIsInstance(decoded.resolved_from, str)
        self.assertIn(decoded.is_fallback, (0, 1))
        self.assertGreaterEqual(decoded.warning_count, 0)


class BytesThatAreNotABlob(unittest.TestCase):
    def setUp(self) -> None:
        self.raw = fixture("positions.tsrb")

    def test_too_short_for_a_header(self) -> None:
        with self.assertRaises(BlobError) as caught:
            decode_positions(self.raw[:16])
        self.assertIn("header", str(caught.exception))

    def test_the_wrong_magic(self) -> None:
        broken = bytearray(self.raw)
        broken[0:4] = b"NOPE"
        with self.assertRaises(BlobError):
            decode_positions(bytes(broken))

    def test_a_layout_version_this_decoder_does_not_read(self) -> None:
        broken = bytearray(self.raw)
        struct.pack_into("<I", broken, 4, 99)
        with self.assertRaises(BlobError) as caught:
            decode_positions(bytes(broken))
        self.assertIn("version 99", str(caught.exception))

    def test_another_schema(self) -> None:
        with self.assertRaises(BlobError) as caught:
            decode_intl_render(self.raw)
        self.assertIn("schema", str(caught.exception))

    def test_a_length_the_header_disagrees_with(self) -> None:
        with self.assertRaises(BlobError) as caught:
            decode_positions(self.raw + b"\0" * 8)
        self.assertIn("bytes", str(caught.exception))


if __name__ == "__main__":
    unittest.main()

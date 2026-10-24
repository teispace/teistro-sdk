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
from collections.abc import Callable, Iterator

from teistro import Body, TimeScale
from teistro._blob import (
    BlobError,
    decode_charts,
    decode_intl_render,
    decode_naam,
    decode_panchanga,
    decode_positions,
)
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
        provenance = json.loads(self.decoded.provenance_json)
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


# Every blob of every schema, corrupted the same way in every binding's
# test: a seeded xorshift32 stream picks, two thousand times a blob, one
# of four mutations -- a few bytes flipped past the header, one aligned
# word set to an edge value (0, 1, 3, 4, 8, 2**31 - 1, 2**31, 2**32 - 1 or
# the blob's length), the blob cut short, or the blob grown with zeros,
# the last two with the header's length mended so the table of contents
# is read. Each mutant is decoded or refused as a `BlobError`; anything
# else raised is a fault in the decoder.
_MUTANTS = 2000
_EDGES = (0, 1, 3, 4, 8, 0x7FFFFFFF, 0x80000000, 0xFFFFFFFF, -1)
_SCHEMAS: tuple[tuple[str, Callable[[bytes], object]], ...] = (
    ("positions.tsrb", decode_positions),
    ("intl_render.tsrb", decode_intl_render),
    ("charts.tsrb", decode_charts),
    ("panchanga.tsrb", decode_panchanga),
    ("naam.tsrb", decode_naam),
)


def _mutants(good: bytes) -> Iterator[tuple[int, int, bytes]]:
    x = 0x9E3779B9

    def next_word() -> int:
        nonlocal x
        x ^= (x << 13) & 0xFFFFFFFF
        x ^= x >> 17
        x ^= (x << 5) & 0xFFFFFFFF
        return x

    for i in range(_MUTANTS):
        raw = bytearray(good)
        kind = next_word() % 4
        if kind == 0:
            for _ in range(1 + next_word() % 4):
                raw[20 + next_word() % (len(raw) - 20)] ^= 1 + next_word() % 255
        elif kind == 1:
            at = (next_word() % (len(raw) >> 2)) * 4
            edge = _EDGES[next_word() % len(_EDGES)]
            # The magic, the version and the schema stay, or nothing past
            # them is read.
            if at < 8 or 12 <= at < 20:
                continue
            struct.pack_into("<I", raw, at, len(raw) if edge == -1 else edge)
        else:
            if kind == 2:
                raw = raw[: 32 + next_word() % (len(raw) - 32)]
            else:
                raw += bytes(8 * (1 + next_word() % 8))
            struct.pack_into("<I", raw, 12, len(raw))
        yield i, kind, bytes(raw)


class ACorruptedBlob(unittest.TestCase):
    def test_of_every_schema_is_decoded_or_refused_never_anything_else(
        self,
    ) -> None:
        for name, decode in _SCHEMAS:
            decoded = refused = 0
            for i, kind, raw in _mutants(fixture(name)):
                try:
                    decode(raw)
                    decoded += 1
                except BlobError:
                    refused += 1
                except Exception as error:  # noqa: BLE001 - the finding
                    found = f"{type(error).__name__}: {str(error)[:200]}"
                    self.fail(f"{name}, mutant {i} (mutation {kind}): {found}")
            # Both ways out were taken, so the mutants reach past the header.
            self.assertTrue(decoded and refused, f"{name}: {decoded}, {refused}")


if __name__ == "__main__":
    unittest.main()

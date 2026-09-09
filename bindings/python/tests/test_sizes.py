"""The layout, which is the one thing a `ctypes` binding cannot check for
itself.

The C header asserts all 25 struct sizes at compile time, so a C consumer
that disagrees does not build. A `ctypes` declaration is trusted: it
declares the fields and the interpreter lays them out, and a wrong layout
is not an error but a wrong number. So the generator writes the sizes it
computed from the description, for both targets, and this file holds
`ctypes.sizeof` to them on the machine the library was really built on
(`docs/03-design/binding-surface-measured.md` §3).

This is the test that would catch a `c_long` where a `c_int32` was meant,
which is right on Linux and wrong on Windows.
"""

from __future__ import annotations

import ctypes
import unittest

from teistro import _ffi


def _struct_of(c_name: str) -> type[ctypes.Structure]:
    """The generated `ctypes.Structure` for a C type name.

    `ts_position_request` is `_PositionRequestStruct`, which is the one
    naming rule this file has to know and the same one the generator used.
    """
    words = c_name.removeprefix("ts_").split("_")
    found: object = getattr(
        _ffi, "_" + "".join(word.capitalize() for word in words) + "Struct"
    )
    assert isinstance(found, type) and issubclass(found, ctypes.Structure)
    return found


class Sizes(unittest.TestCase):
    def test_every_struct_is_the_size_the_description_computed(self) -> None:
        self.assertEqual(len(_ffi.SIZES), 27, "every boundary struct is in the table")
        for name, size in _ffi.SIZES.items():
            with self.subTest(struct=name):
                self.assertEqual(
                    ctypes.sizeof(_struct_of(name)),
                    size,
                    f"{name} is {size} bytes to the description",
                )

    def test_the_table_is_the_one_for_this_target(self) -> None:
        pointer = ctypes.sizeof(ctypes.c_void_p)
        self.assertIn(pointer, (4, 8))
        expected = _ffi._SIZES_64 if pointer == 8 else _ffi._SIZES_32
        self.assertEqual(_ffi.SIZES, expected)
        # And the two tables really differ, so choosing between them is
        # not a distinction without a difference: thirteen of the structs
        # hold a pointer, a callback or a `size_t`. `ts_chart_request`
        # joined them when it took a grid of instants rather than one.
        differ = [
            name for name in _ffi._SIZES_64 if _ffi._SIZES_64[name] != _ffi._SIZES_32[name]
        ]
        self.assertEqual(len(differ), 13, differ)

    def test_no_scalar_is_declared_at_the_platforms_width(self) -> None:
        # `c_long` is 8 bytes on Linux and 4 on Windows; every scalar the
        # generator writes is fixed-width, so a struct's layout cannot
        # move between platforms for a reason the description does not
        # know about.
        source = (
            ctypes.__file__  # only to prove ctypes is the standard library
            and __import__("pathlib").Path(_ffi.__file__).read_text(encoding="utf-8")
        )
        for width_dependent in ("c_long", "c_ulong", "c_int)", "c_uint)", "c_short"):
            self.assertNotIn(
                f"ctypes.{width_dependent}",
                source,
                f"{width_dependent} changes width between platforms",
            )

    def test_the_handshake_is_filled_from_sizeof(self) -> None:
        # A struct that carries `struct_size` is refused by the library
        # unless the caller sets it to its own `sizeof`, so the generated
        # marshalling fills it rather than writing a number.
        owned: list[object] = []
        raw = _ffi.Observer(
            longitude_deg=_ffi.Longitude(85.324),
            latitude_deg=_ffi.Latitude(27.7172),
            altitude_m=_ffi.Altitude(1400),
        )._to_c(owned)
        self.assertEqual(ctypes.sizeof(raw), _ffi.SIZES["ts_observer"])
        options = _ffi.ContextOptions(flags=0)._to_c(owned)
        self.assertEqual(
            options.struct_size, ctypes.sizeof(_ffi._ContextOptionsStruct)
        )


if __name__ == "__main__":
    unittest.main()

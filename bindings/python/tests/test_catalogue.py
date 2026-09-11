"""The generated catalogue: ids, keys, and what happens at the edges.

Every enum comes from the same description the C header and the other two
bindings come from, so what is tested here is the shape Python gives it —
an `IntEnum` a caller may pass wherever an id is wanted, a catalogue kind
that answers `UNKNOWN` for a member from a newer library, and a closed
enum that refuses one, because a value outside a closed set is a fault
and not a state.
"""

from __future__ import annotations

import enum
import unittest

from teistro import catalogue


def every_enum() -> list[type[catalogue.Member]]:
    """Every generated enum, found rather than listed."""
    return [
        found
        for name, found in vars(catalogue).items()
        if isinstance(found, type)
        and issubclass(found, catalogue.Member)
        and found not in (catalogue.Member, catalogue.Catalogued)
        and not name.startswith("_")
    ]


class TheCatalogue(unittest.TestCase):
    def test_there_are_as_many_enums_as_the_description_carries(self) -> None:
        self.assertEqual(len(every_enum()), 92)
        self.assertEqual(sum(len(list(found)) for found in every_enum()), 946 + 62)

    def test_every_member_is_an_int_with_a_key(self) -> None:
        for found in every_enum():
            with self.subTest(enum=found.__name__):
                self.assertTrue(issubclass(found, enum.IntEnum))
                for member in found:
                    self.assertIsInstance(int(member), int)
                    self.assertEqual(member.id, int(member))
                    self.assertTrue(member.key, f"{found.__name__}.{member.name}")

    def test_a_member_finds_itself_by_key(self) -> None:
        self.assertIs(catalogue.Graha.by_key("SUN"), catalogue.Graha.SUN)
        self.assertIs(catalogue.Graha.by_key("graha.SUN"), catalogue.Graha.SUN)
        self.assertIsNone(catalogue.Graha.by_key("SUNN"))

    def test_a_catalogued_member_carries_its_kind(self) -> None:
        self.assertEqual(catalogue.Graha.SUN.full_key, "graha.SUN")
        self.assertEqual(catalogue.Graha.SUN.key, "SUN")

    def test_a_member_from_a_newer_library_is_unknown_and_not_an_error(self) -> None:
        self.assertIs(catalogue.Graha(9999), catalogue.Graha.UNKNOWN)
        self.assertEqual(catalogue.Graha.UNKNOWN.key, "UNKNOWN")

    def test_a_closed_enum_refuses_a_value_outside_it(self) -> None:
        # `Status` is not a catalogue kind: its members are the SDK's own
        # and a code outside them is a fault.
        self.assertNotIn("UNKNOWN", catalogue.Status.__members__)
        with self.assertRaises(ValueError):
            catalogue.Status(4242)
        self.assertEqual(catalogue.Status.OK, 0)
        self.assertEqual(catalogue.Status.OK.key, "ok")

    def test_every_member_is_truthy_whatever_its_id(self) -> None:
        # `IntEnum` inherits `int.__bool__`, so the member with id zero
        # would be falsy — and the member with id zero is `Status.OK`,
        # `Graha.SUN`, `Era.VIKRAMA` and the first member of every enum.
        # `if graha:` has to mean "there is a graha".
        self.assertEqual(int(catalogue.Status.OK), 0)
        self.assertEqual(int(catalogue.Graha.SUN), 0)
        for found in every_enum():
            for member in found:
                with self.subTest(member=f"{found.__name__}.{member.name}"):
                    self.assertTrue(member)
        # And the pattern the fix is for: an optional member with id zero
        # reaches its key rather than short-circuiting to itself.
        era: object = catalogue.Era.VIKRAMA
        self.assertEqual(era and catalogue.Era.VIKRAMA.key, "VIKRAMA")

    def test_no_member_takes_a_name_python_keeps(self) -> None:
        # The falsification pass says the Dart emitter has to rename
        # `ChartKind::Return` and Python does not, because a member here
        # is its catalogue key upper-cased and every Python keyword is
        # lower-case. This is that claim, on the generated file.
        import keyword

        for found in every_enum():
            for member in found:
                with self.subTest(member=f"{found.__name__}.{member.name}"):
                    self.assertFalse(keyword.iskeyword(member.name))
                    self.assertNotIn(member.name.lower(), ("name", "value", "mro"))
        self.assertEqual(catalogue.ChartKind.RETURN.key, "RETURN")


if __name__ == "__main__":
    unittest.main()

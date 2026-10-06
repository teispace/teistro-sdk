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
    def test_the_package_exports_every_kind_the_others_do(self) -> None:
        # Node re-exports the whole catalogue and Dart exports its file; a
        # hand-kept list here left forty-eight kinds out of the package or
        # out of `__all__`, `ProviderCode` among them, so a provider could
        # not name the status of its own cell. A name the package declares
        # for itself is a shadow, which `check-lints` holds to one list
        # across all three bindings (`layer-does-not-shadow-a-kind`).
        import teistro

        missing = sorted(
            kind.__name__
            for kind in every_enum()
            if kind.__name__ not in teistro.__all__
            or getattr(teistro, kind.__name__, None) is None
        )
        self.assertEqual(missing, [], "kinds the package root does not export")

    def test_there_are_as_many_enums_as_the_description_carries(self) -> None:
        # 96 since chart_layout joined the catalogue, whose eight members
        # here are its six layouts, its UNKNOWN, and the kind's own member
        # of `Kind`; 97 since the boundary's `Balance` crossed with the
        # dashas, with its two methods; 99 since the Ashtakavarga's two readings
        # crossed, two members each; 100 since the Vimshopaka's two scorings;
        # 101 since the vaiseshikamsa kind, thirty names and its UNKNOWN;
        # 102 since the avastha_cheshta kind, three sub-states and its UNKNOWN,
        # each kind with its own member of `Kind`; 103 since the dasha phala's
        # `DashaPhase`, three phases; 104 since the year lord's
        # `VarsheshaChosen`, the seven steps of its chain; 106 since the
        # Tajika aspects, `TajikaDrishti`'s five kinds and `TajikaYoga`'s
        # three -- four since Table X-3 gave the Ithasala a third kind,
        # `Poorna`, which adds a member and no new enum; 108 since the
        # sixteen Tajika yogas crossed, `YearYoga`'s sixteen and
        # `Affliction`'s five clauses; 109 since the sahams crossed,
        # `Saham`'s forty-one; 113 since their strength crossed,
        # `SahamStrong`'s twelve clauses, `SahamWeak`'s five,
        # `HarshaGrade`'s five and `TajikaRelation`'s four; and
        # `VarsheshaChosen` three steps longer since the Moon's successor
        # and the Nilakanthi's reading of an unaspected lagna; and one more
        # since `Ephemeris` named the Surya Siddhanta; 115 since Jaimini's
        # `BrahmaRule`, two, and `BrahmaOutcome`, four; 120 since gochar's
        # `GocharFrom`, two, `NodeVedha`, two, `NodeObstruction`, three,
        # `GocharVerdict`, three, and `Fruition`, four; 123 since the
        # Ashtakavarga's `AshtakavargaGoodFrom`, two, `KakshyaLord`, eight,
        # and `SarvaStanding`, three; 126 since the hit list's `HitKind`,
        # four, `Motion`, two, and `AspectPhase`, three; 127 since Sade
        # Sati's `Reckoning`; 128 since a span's `Sunrises`, four. The
        # members grew by one more when `Sunrise` named the upper limb
        # unrefracted. 131 since the blackouts and the eclipses were named:
        # `BlackoutKind`'s twelve, `LunarEclipseKind`'s three and
        # `SolarEclipseKind`'s four, each with its `UNKNOWN`,
        # and `Kind`'s three for them. 135 since the essential dignities'
        # `Sect`, two, `SectRule`, four, `Terms`, five, and `Triplicities`,
        # two; 138 since the accidental fortitudes' `Accident`,
        # twenty-four, and `Partile` and `Siege`, two each; 140 since the
        # almuten's `PlaceReading` and `FortuneRule`, two each, and
        # `FortuneRule` one more since Valens's reading of Fortune by night;
        # 141 since his fourteen `Lot`s, and as many since his time lords
        # added three `DashaSystem`s in two `DashaFamily`s, and the
        # firdaria and the decennials one of each; 143 since Lilly's
        # considerations, `PtolemaicAspect` and `RadicalGround`; 146 since
        # his perfection, `ApplicationKind`, `ImpedimentKind` and `Way`;
        # 147 since the Western aspects' `WesternAspect`; 151 since the
        # matching's `VashyaRelation`, `YoniRelation`, `MaitriRelation` and
        # `BhakootDosha`; 153 since the ten considerations' `DhinamRule` and
        # `Rajju`; 155 since the marriage doshas' `DoshaSystem` and
        # `MatchRole`; 157 since naam milan's `NameVarga` and
        # `VargaRelation`.
        self.assertEqual(len(every_enum()), 157)
        self.assertEqual(
            sum(len(list(found)) for found in every_enum()),
            946 + 73 + 8 + 2 + 4 + 2 + 31 + 1 + 4 + 1 + 3 + 7 + 5 + 4 + 16 + 5 + 41 + 12 + 5 + 5 + 4 + 3 + 1 + 2 + 4 + 2 + 2 + 3 + 3 + 4 + 2 + 8 + 3 + 4 + 2 + 3 + 2 + 4 + 1 + 13 + 4 + 5 + 3 + 2 + 4 + 5 + 2 + 24 + 2 + 2 + 2 + 3 + 14 + 5 + 2 + 2 + 5 + 3 + 3 + 3 + 7 + 9 + 4 + 3 + 7 + 4 + 11 + 5 + 3 + 2 + 8 + 3,
        )

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
        self.assertEqual(catalogue.Status.OK.key, "OK")

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

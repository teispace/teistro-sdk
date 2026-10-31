"""What the hand-written layer decides for itself, and the refusals each
area hands on.

`test_binding.py` walks each feature's answer. This file holds the paths
beside them that a consumer reaches by a mistake or an edge: an argument
of the wrong shape refused before the boundary, by the argument it was
given in; a refusal from the library raised with its status and field;
a reader's every variant; and the edges of a day (a sankranti inside it,
a polar night) and of a drawing (its arcs and curves).
"""

from __future__ import annotations

import dataclasses
import math
import re
from typing import Any, Dict

from teistro import (
    Altitude,
    ArcSegment,
    Body,
    Calendar,
    ChartLayout,
    Ephemeris,
    KujaLifts,
    Latitude,
    LineSegment,
    Observer,
    Plugin,
    QuadSegment,
    Scale,
    Status,
    TeistroError,
    UnitPoint,
    Varga,
    YearYoga,
    date,
    decode_calendar_resolution,
    fixed_zone,
)
from teistro._ffi import Longitude
from teistro._records import (
    CalendarResolutionComputed,
    CalendarResolutionDefined,
    CalendarResolutionDivergent,
    CalendarResolutionTabular,
    MonthDay,
    decode_implementation,
)
from teistro.catalogue import Centre, PolarKind, Rashi

from .support import PROFILE, WithLibrary

KATHMANDU = Observer(latitude_deg=Latitude(27.7172), longitude_deg=Longitude(85.324), altitude_m=Altitude(1400))
TROMSO = Observer(latitude_deg=Latitude(69.6492), longitude_deg=Longitude(18.9553), altitude_m=Altitude(0))
AT: Dict[str, Any] = {"instant": 2451545.0, "place": KATHMANDU, "utc_offset_seconds": 20700}


class ArgumentsOfTheWrongShape(WithLibrary):
    """Each refused before the boundary, named by the argument a consumer
    wrote, so the mistake is found where it was made."""

    def setUp(self) -> None:
        self.ctx = self.teistro.context(profile=PROFILE, ephemeris=Ephemeris.BUILTIN)

    def tearDown(self) -> None:
        self.ctx.close()

    def refused(self, call: Any, field: str) -> TeistroError:
        with self.assertRaises(TeistroError) as caught:
            call()
        self.assertEqual((caught.exception.status, caught.exception.field), (Status.INVALID_ARG, field))
        return caught.exception

    def test_a_contexts_own_rows_are_sequences_and_not_one_string(self) -> None:
        # A string is a sequence of characters, so it is refused by name
        # rather than read as rows of one letter each.
        self.refused(lambda: self.teistro.context(layouts="SOUTH_INDIAN"), "layouts")  # type: ignore[arg-type]
        self.refused(lambda: self.teistro.context(dasha_systems=b"[]"), "dasha_systems")  # type: ignore[arg-type]

    def test_what_is_compared_with_is_another_context(self) -> None:
        november = date(Calendar.GREGORIAN, 2025, 11, 1)
        day: Dict[str, Any] = {"from_date": november, "place": KATHMANDU, "utc_offset_seconds": 20700}
        self.refused(lambda: self.ctx.almanac.disagreements("nepali-drik", **day), "other")  # type: ignore[arg-type]
        self.refused(
            lambda: self.ctx.almanac.published_reasons(**day, festivals={"rules": "NEPAL"}, sky="nepali-committee"),  # type: ignore[arg-type]
            "sky",
        )

    def test_a_pakshi_native_is_a_record(self) -> None:
        self.refused(
            lambda: self.ctx.almanac.pakshi(
                from_date=date(Calendar.GREGORIAN, 2025, 11, 1), place=KATHMANDU, utc_offset_seconds=20700, native="OWL"  # type: ignore[arg-type]
            ),
            "native",
        )

    def test_a_rashifal_period_is_a_mapping_of_a_date_and_an_observer(self) -> None:
        first = date(Calendar.GREGORIAN, 2025, 11, 1)
        self.refused(lambda: self.ctx.chart.rashifal_many(["week"]), "rashifal")  # type: ignore[list-item]
        self.refused(
            lambda: self.ctx.chart.rashifal_many([{"first": "2025-11-01", "place": KATHMANDU, "utcOffsetSeconds": 20700}]),  # type: ignore[typeddict-item]
            "first",
        )
        self.refused(
            lambda: self.ctx.chart.rashifal_many([{"first": first, "place": (27.7, 85.3), "utcOffsetSeconds": 20700}]),  # type: ignore[typeddict-item]
            "place",
        )

    def test_a_theme_and_a_lal_kitab_request_are_records(self) -> None:
        drawn = [(ChartLayout.SOUTH_INDIAN, Varga.D1)]
        self.refused(lambda: self.ctx.chart.found(**AT, drawings=drawn, theme=42), "theme")  # type: ignore[arg-type]
        self.refused(lambda: self.ctx.chart.found(**AT, lalkitab=30), "lalkitab")  # type: ignore[arg-type]

    def test_a_place_without_an_observer_is_named_by_the_argument_it_was_in(self) -> None:
        """A partner's and a residence's place each need an `Observer`;
        the refusal says which argument held the place. A month chart's
        residence was once refused as `varsha`'s, an argument the call
        never had."""
        cases: Any = (
            ({"matching": {"partner": {"instant": 2447892.5, "observer": "Kathmandu"}}}, "matching['partner']"),
            ({"varsha": {"through": 3, "place": {"observer": "Kathmandu"}}}, "varsha['place']"),
            ({"masa_pravesha": {"year": 3, "place": {"observer": "Kathmandu"}}}, "masa_pravesha['place']"),
        )
        for asked, root in cases:
            with self.assertRaises(TypeError) as caught:
                self.ctx.chart.found(**AT, **asked)
            self.assertEqual(str(caught.exception), f"{root}['observer']: expected an Observer")


class TheLibrarysRefusals(WithLibrary):
    """A refusal from the boundary crosses with its status and the field
    the library named, through whichever area asked."""

    def setUp(self) -> None:
        self.ctx = self.teistro.context(profile=PROFILE, ephemeris=Ephemeris.BUILTIN)

    def tearDown(self) -> None:
        self.ctx.close()

    def refused(self, call: Any, status: Status, field: str) -> TeistroError:
        with self.assertRaises(TeistroError) as caught:
            call()
        self.assertEqual((caught.exception.status, caught.exception.field), (status, field), str(caught.exception))
        return caught.exception

    def test_a_calendar_refuses_a_day_it_does_not_have(self) -> None:
        self.refused(lambda: self.ctx.calendar.date_of(Calendar.GREGORIAN, 10**15), Status.OUT_OF_RANGE, "fixed")
        self.refused(lambda: self.ctx.calendar.month_length(Calendar.GREGORIAN, 2024, 13), Status.INVALID_ARG, "month")
        leap_day = self.refused(
            lambda: self.ctx.calendar.weekday_of(date(Calendar.GREGORIAN, 2024, 2, 30)), Status.INVALID_ARG, "day"
        )
        self.assertIn("1 to 29", str(leap_day), "2024 is a leap year")
        self.assertEqual(self.ctx.calendar.weekday_of(date(Calendar.GREGORIAN, 2024, 2, 29)), 4, "a Thursday")

    def test_time_refuses_an_instant_that_is_not_a_number(self) -> None:
        nan = float("nan")
        self.refused(lambda: self.ctx.time.civil_of(nan, fixed_zone(0), Calendar.GREGORIAN), Status.INVALID_ARG, "jd_utc")
        self.refused(lambda: self.ctx.time.convert(nan, Scale.UTC, Scale.TT), Status.INVALID_ARG, "jd")
        self.refused(lambda: self.ctx.time.delta_t(nan), Status.INVALID_ARG, "jd_ut1")

    def test_a_key_and_a_frame_the_catalogue_does_not_have(self) -> None:
        self.refused(lambda: self.ctx.keys.name(0x0FFFFFFF), Status.UNSUPPORTED, "id")
        # Bits no frame sets: the free function's refusal carries no
        # record, so it says the status's own sentence.
        bits = self.refused(lambda: self.teistro.unpack_frame(0xFFFFFFFF), Status.INVALID_ARG, "")
        self.assertEqual(str(bits), "invalid argument")

    def test_a_locale_a_script_and_a_pack_the_engine_does_not_have(self) -> None:
        before = self.ctx.intl.locale
        unknown = self.refused(lambda: setattr(self.ctx.intl, "locale", "xx-Bogus-ZZ"), Status.UNSUPPORTED, "locale")
        self.assertIn("ne-Deva-NP", unknown.hint, "the hint lists the locales loaded")
        self.assertEqual(self.ctx.intl.locale, before, "a refused locale changes nothing")
        script = self.refused(lambda: self.ctx.intl.transliterate("क", "Deva", "Zzzz"), Status.UNSUPPORTED, "to")
        self.assertIn("iast", script.hint)
        self.refused(lambda: self.ctx.intl.load_pack(b"not a pack"), Status.PACK, "bytes")

    def test_naming_refuses_a_pada_past_the_fourth_by_where_it_was(self) -> None:
        self.refused(
            lambda: self.ctx.naming.syllables({"of": "PADA", "nakshatra": "ASHWINI", "pada": 9}),
            Status.INVALID_ARG,
            "naming.of.pada",
        )
        self.refused(
            lambda: self.ctx.naming.check(["राम"], {"nakshatra": "ASHWINI", "pada": 9}), Status.INVALID_ARG, "naming.birth.pada"
        )
        self.refused(lambda: self.ctx.naming.of({"nakshatra": "KRITTIKA", "pada": 9}), Status.INVALID_ARG, "naming.birth.pada")

    def test_naming_rules_written_out_are_read_and_not_dropped(self) -> None:
        """Pāraskara's sūtra is the default, so naming it changes no answer;
        naming Āśvalāyana's changes whose clauses a check reports, and
        nothing about the syllables, which no sūtra sets."""
        default: Any = {"sutra": "PARASKARA"}
        other: Any = {"sutra": "ASHVALAYANA"}
        pada: Any = {"of": "PADA", "nakshatra": "ASHWINI", "pada": 1}
        birth: Any = {"nakshatra": "ASHWINI", "pada": 1}
        name = ["चूडामणि"]
        self.assertEqual(self.ctx.naming.syllables(pada, default), self.ctx.naming.syllables(pada))
        self.assertEqual(self.ctx.naming.syllables(pada, other), self.ctx.naming.syllables(pada))
        self.assertEqual(self.ctx.naming.of(birth, default), self.ctx.naming.of(birth))
        self.assertEqual(self.ctx.naming.check(name, birth, default), self.ctx.naming.check(name, birth))
        self.assertNotEqual(self.ctx.naming.check(name, birth, other), self.ctx.naming.check(name, birth))
        self.assertIn("ASHVALAYANA", repr(self.ctx.naming.check(name, birth, other)))

    def test_an_adapter_that_is_not_there_is_refused_by_its_path(self) -> None:
        missing = self.refused(
            lambda: self.teistro.context(ephemeris=Plugin(plugin="/no/such/adapter.dylib")), Status.UNSUPPORTED, "path"
        )
        self.assertIn("/no/such/adapter.dylib", str(missing))


class TheReaders(WithLibrary):
    """A reader's every variant, and a variant it does not know refused
    rather than read as another."""

    def test_a_calendar_resolution_is_read_by_its_kind(self) -> None:
        self.assertEqual(decode_calendar_resolution({"kind": "DEFINED"}), CalendarResolutionDefined())
        self.assertEqual(
            decode_calendar_resolution({"kind": "TABULAR", "authority": "committee", "edition": "2081"}),
            CalendarResolutionTabular(authority="committee", edition="2081"),
        )
        self.assertEqual(
            decode_calendar_resolution({"kind": "COMPUTED", "model": "SURYA_SIDDHANTA"}),
            CalendarResolutionComputed(model="SURYA_SIDDHANTA"),
        )
        divergent = decode_calendar_resolution(
            {"kind": "DIVERGENT", "model": "SURYA_SIDDHANTA", "computed": {"month": 1, "day": 31}, "tabular": {"month": 1, "day": 32}}
        )
        self.assertEqual(
            divergent,
            CalendarResolutionDivergent(
                computed=MonthDay(day=31, month=1), model="SURYA_SIDDHANTA", tabular=MonthDay(day=32, month=1)
            ),
        )
        with self.assertRaises(ValueError):
            decode_calendar_resolution({"kind": "GUESSED"})
        self.assertEqual(decode_implementation("PASS_THROUGH"), "PASS_THROUGH")
        with self.assertRaises(ValueError):
            decode_implementation("GUESSED")

    def test_kuja_lifts_are_none_unless_bit_seven_says_they_were_read(self) -> None:
        self.assertIsNone(KujaLifts.of_bits(0))
        self.assertIsNone(KujaLifts.of_bits(0b0111_1111), "every lift set, but none read")
        lifts = KujaLifts.of_bits(0x80 | 0b100_0101)
        assert lifts is not None
        self.assertEqual(
            dataclasses.astuple(lifts), (True, False, True, False, False, False, True), "bit n is the nth field"
        )

    def test_a_batch_refuses_an_index_outside_it(self) -> None:
        with self.teistro.context(profile=PROFILE, ephemeris=Ephemeris.BUILTIN) as ctx:
            charts = ctx.chart.found_many(instants=[2451545.0, 2451546.0], place=KATHMANDU, utc_offset_seconds=20700)
            self.assertEqual(charts.at(1).day.date.day, charts[1].day.date.day)
            for outside in (2, -1):
                with self.assertRaises(IndexError):
                    charts.at(outside)
            days = ctx.almanac.of(
                from_date=date(Calendar.GREGORIAN, 2024, 6, 1),
                to_date=date(Calendar.GREGORIAN, 2024, 6, 3),
                place=KATHMANDU,
                utc_offset_seconds=20700,
            )
            with self.assertRaises(IndexError) as caught:
                days.at(3)
            self.assertEqual(str(caught.exception), "day 3 is outside a batch of 3")

    def test_every_segment_of_a_drawing_is_read_and_an_arc_keeps_its_radius(self) -> None:
        """The lotus's petals are curves and the wheels' rings arcs; an arc
        ends as far from its centre as it starts."""

        def end(segment: Any) -> UnitPoint:
            point: UnitPoint = segment.to
            return point

        with self.teistro.context(profile=PROFILE, ephemeris=Ephemeris.BUILTIN) as ctx:
            for layout, curve in (
                (ChartLayout.NEPALI_LOTUS, QuadSegment),
                (ChartLayout.WESTERN_WHEEL, ArcSegment),
                (ChartLayout.SUDARSHAN_CHAKRA, ArcSegment),
            ):
                (drawing,) = ctx.chart.found(**AT, drawings=[(layout, Varga.D1)]).drawings
                outlines = [cell.outline for cell in drawing.cells] + drawing.frame
                seen = 0
                for outline in outlines:
                    at = outline.start
                    for segment in outline.segments:
                        self.assertIsInstance(segment, (LineSegment, QuadSegment, ArcSegment))
                        if isinstance(segment, ArcSegment):
                            centre = segment.centre
                            radius = math.hypot(at.x - centre.x, at.y - centre.y)
                            self.assertAlmostEqual(
                                math.hypot(segment.to.x - centre.x, segment.to.y - centre.y), radius, delta=1e-8
                            )
                        seen += isinstance(segment, curve)
                        at = end(segment)
                self.assertGreater(seen, 0, f"{layout} draws a {curve.__name__}")


class TheEdgesOfADay(WithLibrary):
    def test_a_sankranti_is_given_on_the_one_day_that_holds_it(self) -> None:
        """June 2024 at Kathmandu: the Sun entered Mithuna once, inside the
        day of the 14th (00:44 on the 15th by the clock, before its
        sunrise); every other day has none, and only that day saw the Sun
        in two signs."""
        with self.teistro.context(profile=PROFILE, ephemeris=Ephemeris.BUILTIN) as ctx:
            june = ctx.almanac.of(
                from_date=date(Calendar.GREGORIAN, 2024, 6, 1),
                to_date=date(Calendar.GREGORIAN, 2024, 6, 30),
                place=KATHMANDU,
                utc_offset_seconds=20700,
            )
            held = [(day.day.date.day, day) for day in june if day.sankranti is not None]
            self.assertEqual([on for on, _ in held], [14])
            (_, day), = held
            sankranti = day.sankranti
            assert sankranti is not None
            self.assertTrue(day.day.sunrise <= sankranti < day.day.next_sunrise)
            self.assertEqual(day.sun_signs[-1].member, Rashi.GEMINI)
            for one in june:
                self.assertEqual(len(one.sun_signs) == 2, one.sankranti is not None, one.day.date.day)

    def test_a_polar_night_under_civil_midnight_has_no_brahma_muhurta(self) -> None:
        """Tromsø on 20 December 2024 has no sunrise. Civil midnight stands
        in for the missing bounds, so the night before has no length to
        divide and Brahma muhurta is absent; Abhijit is the civil day's."""
        with self.teistro.context(
            profile=PROFILE, ephemeris=Ephemeris.BUILTIN, settings={"day": {"polar_day_policy": "CIVIL_MIDNIGHT"}}
        ) as ctx:
            for day in ctx.almanac.of(
                from_date=date(Calendar.GREGORIAN, 2024, 12, 20),
                to_date=date(Calendar.GREGORIAN, 2024, 12, 21),
                place=TROMSO,
                utc_offset_seconds=3600,
            ):
                polar = day.day.polar
                assert polar is not None
                self.assertEqual(polar.kind, PolarKind.NIGHT)
                self.assertIsNone(day.brahma)
                abhijit = day.abhijit
                assert abhijit is not None
                self.assertTrue(day.window.from_jd <= abhijit.at.from_jd < abhijit.at.to_jd <= day.window.to_jd)

    def test_a_feed_of_one_day_when_no_last_day_is_given(self) -> None:
        """Vijaya Dashami 2025 alone: with `to_date` left out the feed
        holds the first day's events and no other day's."""
        with self.teistro.context(profile=PROFILE, ephemeris=Ephemeris.BUILTIN) as ctx:
            ics = ctx.almanac.festival_calendar(
                from_date=date(Calendar.GREGORIAN, 2025, 10, 2),
                place=KATHMANDU,
                utc_offset_seconds=20700,
                festivals={"rules": "NEPAL_PANCHANGA"},
                stamp_unix_seconds=1767225600,
            )
            days = set(re.findall(r"DTSTART;VALUE=DATE:(\d{8})", ics))
            self.assertEqual(days, {"20251002"})


class WhatAnAnswerCarries(WithLibrary):
    def test_a_topocentric_moon_is_displaced_by_its_parallax(self) -> None:
        """The same instant seen from Kathmandu rather than the Earth's
        centre: the Moon moves by its parallax, under the horizontal
        parallax's 1.03°, and the Sun by under its 9 arcseconds."""
        with self.teistro.context(ephemeris=Ephemeris.BUILTIN) as ctx:
            topocentric = dataclasses.replace(self.teistro.canonical_frame, centre=Centre.TOPOCENTRIC)
            bodies = [Body.MOON, Body.SUN]
            centre = ctx.positions(instants=[2451545.0], bodies=bodies)
            here = ctx.positions(instants=[2451545.0], bodies=bodies, frame=topocentric, observer=KATHMANDU)
            moon = abs(here.at(0, 0).longitude - centre.at(0, 0).longitude)
            sun = abs(here.at(0, 1).longitude - centre.at(0, 1).longitude)
            self.assertTrue(0.1 < moon < 1.03, moon)
            self.assertLess(sun, 9 / 3600)
            with self.assertRaises(TeistroError) as nowhere:
                ctx.positions(instants=[2451545.0], bodies=bodies, frame=topocentric)
            self.assertEqual(nowhere.exception.field, "observer")

    def test_a_running_period_carries_the_sex_and_a_bath_no_days(self) -> None:
        """A man's death, then the same news heard 800 days on: past the
        late table's last row, a bath alone, which has no count of days
        for a later request's overlap rules to read (`03-design/ashauch.md`)."""
        died = 2461051.677083
        death: Any = {
            "key": "grandfather",
            "relation": "SAPINDA",
            "event": {"event": "DEATH", "at": died, "deceased": {"sex": "MALE"}},
        }
        with self.teistro.context(profile=PROFILE, ephemeris=Ephemeris.BUILTIN) as ctx:
            full = ctx.almanac.ashauch(place=KATHMANDU, utc_offset_seconds=20700, request=death).value.running()
            self.assertEqual((full.get("sex"), full.get("days"), full["length"]), ("MALE", 10, {"length": "FULL"}))
            heard: Any = {**death, "heardAt": died + 800}
            bath = ctx.almanac.ashauch(place=KATHMANDU, utc_offset_seconds=20700, request=heard).value.running()
            self.assertEqual(bath["length"], {"length": "BATH"})
            self.assertNotIn("days", bath)
            self.assertEqual(bath["start"], bath["end"], "a bath takes no time")

    def test_an_unanswered_year_yoga_holds_none_and_every_other_a_bool(self) -> None:
        with self.teistro.context(ephemeris=Ephemeris.BUILTIN) as ctx:
            read = ctx.chart.found(
                instant=2447995.4895833335,
                place=KATHMANDU,
                utc_offset_seconds=20700,
                prashna={"question": {"house": 7, "number": 14}},
            ).prashna
            assert read is not None and read.links is not None
            links = read.links
            for yoga in YearYoga:
                answer = links.holds(yoga)
                if yoga in links.unanswered:
                    self.assertIsNone(answer, yoga)
                else:
                    self.assertEqual(answer, any(one.yoga == yoga for one in links.held), yoga)

    def test_a_pakshi_reading_over_a_range_with_its_rules_written_out(self) -> None:
        with self.teistro.context(profile=PROFILE, ephemeris=Ephemeris.BUILTIN) as ctx:
            asked: Dict[str, Any] = {
                "from_date": date(Calendar.GREGORIAN, 2025, 11, 1),
                "to_date": date(Calendar.GREGORIAN, 2025, 11, 2),
                "place": KATHMANDU,
                "utc_offset_seconds": 20700,
                "native": {"bird": "OWL"},
            }
            plain = ctx.almanac.pakshi(**asked)
            self.assertEqual([day.date.day for day in plain.value], [1, 2])
            self.assertEqual(ctx.almanac.pakshi(**asked, rules={}).value, plain.value, "empty rules are the defaults")


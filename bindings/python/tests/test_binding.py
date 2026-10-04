"""The whole surface, end to end, against the real library.

Everything a consumer touches goes through the hand-written layer, which
is thin: what is really being tested is that the generated marshalling
puts a value into the C struct the library expects and reads the answer
back out of the one it filled.
"""

from __future__ import annotations

import json
import os
import unittest
from typing import Any, Optional

from teistro import (
    Air,
    Altitude,
    decode_provenance,
    AvasthaSayanadi,
    DashaDefinition,
    RashiDashaDefinition,
    TajikaDrishti,
    TajikaYoga,
    UduDashaDefinition,
    VarsheshaChosen,
    YearYoga,
    Saham,
    SahamStrong,
    SahamWeak,
    DashaPhase,
    Nature,
    Balance,
    Body,
    Calendar,
    ChartLayout,
    DashaSystem,
    Drawing,
    Ekadhipatya,
    LayoutRow,
    Ephemeris,
    EphemerisProvider,
    Latitude,
    Nakshatra,
    Observer,
    Plugin,
    Rashi,
    Scale,
    Shodhana,
    Status,
    Teistro,
    PolarDay,
    TeistroError,
    TimeScale,
    PlanRequest,
    Plans,
    RuleRequest,
    RulesReading,
    Theme,
    Varga,
    VimshopakaScoring,
    at,
    date,
    fixed_zone,
    iana_zone,
    intl,
    local_mean_zone,
    message_parts,
    message_warnings,
    when_unknown,
)
from teistro._ffi import Longitude
from teistro.catalogue import (
    Ayanamsha,
    BrahmaOutcome,
    BrahmaRule,
    DayPart,
    Ephemeris,
    Era,
    Graha,
    LunarEclipseKind,
    MonthKind,
    PolarDayPolicy,
    PolarKind,
    Ritu,
    SolarEclipseKind,
    Sunrise,
    Sunrises,
    Tithi,
    Vara,
)
from tests.support import LOCALE, PROFILE, WithLibrary, fixture


class TheLibrary(WithLibrary):
    def test_it_says_what_it_is(self) -> None:
        self.assertEqual(self.teistro.abi, 1)
        self.assertTrue(self.teistro.version)
        self.assertGreaterEqual(self.teistro.catalogue, 1)
        self.assertEqual(self.teistro.default_profile_id, "parashari-classical")

    def test_the_build_it_describes_is_the_one_it_is(self) -> None:
        build = self.teistro.build
        self.assertEqual(build.abi, self.teistro.abi)
        self.assertEqual(build.sdk, self.teistro.version)
        self.assertEqual(build.sanitizer, "")
        self.assertTrue(build.target)
        self.assertTrue(build.commit)

    def test_a_library_that_is_not_there_names_where_it_looked(self) -> None:
        with self.assertRaises(OSError):
            Teistro.open(path="/no/such/library.so")

    def test_the_canonical_frame_packs_and_unpacks(self) -> None:
        frame = self.teistro.canonical_frame
        bits = self.teistro.pack_frame(frame)
        again = self.teistro.unpack_frame(bits)
        self.assertEqual(again.centre, frame.centre)
        self.assertEqual(again.coordinates, frame.coordinates)
        self.assertEqual(self.teistro.pack_frame(again), bits)

    def test_the_frame_area_answers_as_the_free_functions_do(self) -> None:
        # The same three answers through the area a consumer with a
        # context reaches for, which is a different path to them.
        ctx = self.teistro.context(test_provider=True)
        try:
            frame = ctx.frame.canonical
            self.assertEqual(frame.centre, self.teistro.canonical_frame.centre)
            bits = ctx.frame.pack(frame)
            self.assertEqual(bits, self.teistro.pack_frame(frame))
            self.assertEqual(ctx.frame.unpack(bits).coordinates, frame.coordinates)
        finally:
            ctx.close()

    def test_a_fixed_day_and_a_julian_day_convert_both_ways(self) -> None:
        jd = self.teistro.julian_day_of_fixed(735702)
        self.assertGreater(jd, 2_400_000)
        fixed, fraction = self.teistro.fixed_of_julian_day(2457126.75)
        self.assertIsInstance(fixed, int)
        self.assertGreaterEqual(fraction, 0.0)
        self.assertLess(fraction, 1.0)


class AContext(WithLibrary):
    def setUp(self) -> None:
        self.ctx = self.teistro.context(
            profile=PROFILE, locale=LOCALE, test_provider=True
        )

    def tearDown(self) -> None:
        self.ctx.close()

    def test_it_carries_the_settings_it_was_asked_for(self) -> None:
        self.assertEqual(self.ctx.profile, PROFILE)
        self.assertEqual(self.ctx.intl.locale, LOCALE)
        self.assertEqual(len(self.ctx.settings_hash), 64)
        settings = self.ctx.settings
        self.assertIsInstance(settings, dict)
        self.assertIn("frame", settings)
        # The document round-trips as its own canonical JSON.
        self.assertEqual(json.loads(self.ctx.settings_json), settings)

    def test_a_pack_loads_and_lays_its_record_over_the_one_standing(self) -> None:
        # The fixture pack is the base locale's, so this context reads that
        # one: a record is a locale's, and loading into `en-Latn` does not
        # touch what `ne-Deva-NP` says of the same key.
        ctx = self.teistro.context(
            profile=PROFILE, locale="en-Latn", test_provider=True
        )
        try:
            before = ctx.intl.entity("graha.SUN")
            self.assertTrue(before.name, "the engine names the Sun")
            self.assertIsNone(before.forms.get("phala"))

            loaded = ctx.intl.load_pack(fixture("overlay.tpack"))
            self.assertEqual(loaded.entries, 1)
            self.assertEqual(loaded.replaced, 0, "nothing was thrown away")
            self.assertEqual(loaded.merged, 1, "the record kept what the pack lacked")
            self.assertEqual(loaded.locale, "en-Latn")
            self.assertEqual(len(loaded.sha256), 64)

            # A record's forms are an open set, so a form no locale of
            # `i18n/` carries is reached through `forms`.
            after = ctx.intl.entity("graha.SUN")
            self.assertEqual(after.forms["phala"], "a reading of the Sun")
            self.assertEqual(after.name, before.name)
            self.assertEqual(after.forms["name"], before.name)
        finally:
            ctx.close()

    def test_a_closed_context_refuses_rather_than_crashes(self) -> None:
        ctx = self.teistro.context(test_provider=True)
        ctx.close()
        with self.assertRaises(TeistroError) as caught:
            _ = ctx.profile
        self.assertEqual(caught.exception.status, Status.INVALID_ARG)
        ctx.close()  # closing twice is allowed

    def test_a_with_block_closes_it(self) -> None:
        with self.teistro.context(test_provider=True) as ctx:
            self.assertTrue(ctx.profile)
        with self.assertRaises(TeistroError):
            _ = ctx.profile


class TheCalendars(WithLibrary):
    def setUp(self) -> None:
        self.ctx = self.teistro.context(profile=PROFILE, locale=LOCALE)

    def tearDown(self) -> None:
        self.ctx.close()

    def test_the_new_year_of_2072_bs_is_14_april_2015(self) -> None:
        bs = self.ctx.calendar.convert(date(Calendar.GREGORIAN, 2015, 4, 14), Calendar.BIKRAM_SAMBAT)
        self.assertEqual((bs.year, bs.month, bs.day), (2072, 1, 1))
        self.assertIsNotNone(bs.era)
        assert bs.era is not None
        self.assertEqual(bs.era.key, "VIKRAMA")
        self.assertEqual(bs.era_year, 2072)

    def test_a_date_round_trips_through_its_fixed_day(self) -> None:
        day = date(Calendar.GREGORIAN, 2015, 4, 14)
        fixed = self.ctx.calendar.fixed_of(day)
        again = self.ctx.calendar.date_of(Calendar.GREGORIAN, fixed)
        self.assertEqual((again.year, again.month, again.day), (2015, 4, 14))
        self.assertEqual(self.ctx.calendar.weekday_of(day), 2, "a Tuesday")

    def test_a_leap_year_and_a_month_length(self) -> None:
        self.assertTrue(self.ctx.calendar.is_leap(Calendar.GREGORIAN, 2024))
        self.assertFalse(self.ctx.calendar.is_leap(Calendar.GREGORIAN, 2023))
        self.assertEqual(self.ctx.calendar.month_length(Calendar.GREGORIAN, 2024, 2), 29)
        self.assertEqual(self.ctx.calendar.month_length(Calendar.GREGORIAN, 2023, 2), 28)

    def test_a_month_the_calendar_does_not_have_is_refused_by_field(self) -> None:
        with self.assertRaises(TeistroError) as caught:
            self.ctx.calendar.fixed_of(date(Calendar.GREGORIAN, 2015, 13, 1))
        self.assertNotEqual(caught.exception.status, Status.OK)
        self.assertTrue(str(caught.exception))


class Time(WithLibrary):
    def setUp(self) -> None:
        self.ctx = self.teistro.context(profile=PROFILE, locale=LOCALE)

    def tearDown(self) -> None:
        self.ctx.close()

    def test_a_kathmandu_birth_time_resolves_with_its_metadata(self) -> None:
        civil = at(date(Calendar.GREGORIAN, 1986, 1, 1), hour=0, minute=20)
        resolved = self.ctx.time.resolve(civil, iana_zone("Asia/Kathmandu"))
        self.assertEqual(resolved.offset_seconds, 20700, "+05:45")
        self.assertGreater(resolved.instant_jd_utc, 2_446_000)
        self.assertTrue(resolved.time_known)
        self.assertTrue(resolved.tzdb_version)

    def test_an_unknown_time_is_refused_rather_than_guessed(self) -> None:
        # Under this profile an instant with no time of day has no
        # answer, and the boundary says so with the field and the choices
        # rather than picking one.
        with self.assertRaises(TeistroError) as caught:
            self.ctx.time.resolve(
                when_unknown(date(Calendar.GREGORIAN, 1986, 1, 1)),
                iana_zone("Asia/Kathmandu"),
            )
        error = caught.exception
        self.assertEqual(error.field, "time")
        self.assertIn("NOON", error.hint)

    def test_an_instant_reads_back_as_the_civil_time_it_was(self) -> None:
        zone = iana_zone("Asia/Kathmandu")
        civil = at(date(Calendar.GREGORIAN, 1986, 1, 1), hour=0, minute=20)
        resolved = self.ctx.time.resolve(civil, zone)
        back, resolution = self.ctx.time.civil_of(
            resolved.instant_jd_utc, zone, Calendar.GREGORIAN
        )
        self.assertEqual(back.date.year, 1986)
        self.assertEqual(back.time.minute, 20)
        self.assertEqual(resolution.offset_seconds, resolved.offset_seconds)

    def test_the_other_two_kinds_of_zone(self) -> None:
        civil = at(date(Calendar.GREGORIAN, 2000, 1, 1), hour=12)
        self.assertEqual(self.ctx.time.resolve(civil, fixed_zone(3600)).offset_seconds, 3600)
        mean = self.ctx.time.resolve(civil, local_mean_zone(Longitude(85.324)))
        self.assertNotEqual(mean.offset_seconds, 0)

    def test_utc_to_tt_reads_the_leap_seconds(self) -> None:
        # `Scale` and not `TimeScale`: the time layer knows UTC and the
        # port does not, and the two enums agree on the ids they share, so
        # the wrong one would convert from the wrong scale in silence.
        converted = self.ctx.time.convert(2451544.5, Scale.UTC, Scale.TT)
        self.assertGreater(converted.jd, 2451544.5, "TT runs ahead of UTC")
        self.assertEqual(converted.delta_t_source.key, "LEAP_SECONDS")
        self.assertAlmostEqual(converted.delta_t_seconds, 64.184, places=3)

    def test_ut1_to_tt_is_the_delta_t_the_model_gives(self) -> None:
        # `delta_t` takes a UT1 instant, so the conversion it must agree
        # with is the one that starts on UT1 as well.
        converted = self.ctx.time.convert(2451544.5, Scale.UT1, Scale.TT)
        delta = self.ctx.time.delta_t(2451544.5)
        self.assertAlmostEqual(delta.seconds, converted.delta_t_seconds, places=6)
        self.assertEqual(delta.source, converted.delta_t_source)
        self.assertTrue(delta.source.key)


class Keys(WithLibrary):
    def setUp(self) -> None:
        self.ctx = self.teistro.context(profile=PROFILE, locale=LOCALE)

    def tearDown(self) -> None:
        self.ctx.close()

    def test_a_key_and_its_id_are_each_other(self) -> None:
        identifier = self.ctx.keys.id("graha.SUN")
        self.assertEqual(self.ctx.keys.name(identifier), "graha.SUN")

    def test_a_key_that_is_not_one_is_refused_with_a_suggestion(self) -> None:
        with self.assertRaises(TeistroError) as caught:
            self.ctx.keys.id("graha.SUNN")
        error = caught.exception
        self.assertNotEqual(error.status, Status.OK)
        self.assertIn("SUN", error.hint or error.message)

    def test_a_context_that_cannot_be_built_says_which_field_and_why(self) -> None:
        # No context exists to keep this refusal, so the record crosses
        # whole from the call that failed (ffi-abi-and-api-description.md
        # §6.1) — the same field and hint a context's refusal carries.
        with self.assertRaises(TeistroError) as caught:
            self.teistro.context(profile="vedic-classic")
        error = caught.exception
        self.assertEqual(error.status, Status.UNSUPPORTED)
        self.assertIn("no shipped profile `vedic-classic`", error.message)
        self.assertEqual(error.field, "profile")
        self.assertIn("parashari-classical", error.hint)
        with self.assertRaises(TeistroError) as caught:
            self.teistro.context(locale="xx-Latn")
        self.assertEqual(caught.exception.field, "locale")
        self.assertIn("ne-Deva-NP", caught.exception.hint)


class TheLocaleEngine(WithLibrary):
    def setUp(self) -> None:
        self.ctx = self.teistro.context(profile=PROFILE, locale=LOCALE)

    def tearDown(self) -> None:
        self.ctx.close()

    def test_a_message_renders_in_the_contexts_locale(self) -> None:
        rendered = self.ctx.intl.render(
            "sdk.reason.grahaInBhava",
            {"graha": {"$entity": "graha.JUPITER"}, "bhava": 7},
        )
        self.assertTrue(rendered.text)
        self.assertFalse(rendered.is_fallback, "the locale carries it")

    def test_the_typed_accessor_renders_the_same_message(self) -> None:
        typed = self.ctx.intl.messages.sdk.reason.graha_in_bhava(
            graha=intl.GrahaKey.JUPITER, bhava=7
        )
        loose = self.ctx.intl.render(
            "sdk.reason.grahaInBhava",
            {"graha": {"$entity": "graha.JUPITER"}, "bhava": 7},
        ).text
        self.assertEqual(typed, loose)

    def test_a_lunar_month_is_said_with_its_kind_an_adhika_one_the_nepali_way(self) -> None:
        self.ctx.intl.locale = "ne-Deva-NP"
        said = self.ctx.intl.messages.sdk.calendar.lunar_month
        self.assertEqual(said(kind=MonthKind.ADHIKA.key, masa=intl.MasaKey.JYESHTHA), "अधिक ज्येष्ठ")
        self.assertEqual(said(kind=MonthKind.NIJA.key, masa=intl.MasaKey.JYESHTHA), "ज्येष्ठ")

    def test_an_instant_reads_in_the_zone_a_message_is_given(self) -> None:
        self.ctx.intl.locale = "en-Latn"
        in_zone = self.ctx.intl.messages.sdk.calendar.datetime.in_zone
        # Noon at Greenwich on 24 February 2023, and noon on 1 July.
        self.assertEqual(in_zone(at=2460000.0, zone="Asia/Kathmandu"), "2023-02-24, 17:45")
        self.assertEqual(in_zone(at=2460127.0, zone="America/New_York"), "2023-07-01, 08:00")
        self.assertEqual(in_zone(at=2460000.0, zone="-03:00"), "2023-02-24, 09:00")
        unknown = self.ctx.intl.render(
            "sdk.calendar.datetime.inZone",
            {"at": {"$instant": 2460000.0}, "zone": "Asia/Kathmandoo"},
        )
        self.assertTrue(any("Asia/Kathmandu" in w for w in message_warnings(unknown)))

    def test_a_rendered_message_carries_its_markup_in_parts(self) -> None:
        self.ctx.intl.locale = "en-Latn"
        # `sdk.reason.lordship` is one of the two shipped messages that
        # use MF2 markup. Without the parts a renderer can only ever
        # print the text, which is why the message may as well not have
        # had the tag.
        rich = self.ctx.intl.render(
            "sdk.reason.lordship",
            {"graha": {"$entity": "graha.JUPITER"}, "bhava": 5},
        )
        self.assertEqual(rich.text, "Jupiter rules house 5")
        parts = message_parts(rich)
        self.assertEqual(
            [str(part) for part in parts],
            ["<open b>", "Jupiter", "<close b>", " rules house 5"],
        )
        self.assertEqual(dict(parts[0].options), {})
        # A renderer that knows no tag joins the text parts and loses
        # nothing.
        self.assertEqual(
            "".join(part.value for part in parts if part.is_text), rich.text
        )

    def test_a_message_without_markup_is_the_one_text_part(self) -> None:
        plain = self.ctx.intl.render(
            "sdk.reason.grahaInBhava",
            {"graha": {"$entity": "graha.JUPITER"}, "bhava": 7},
        )
        # The boundary sends nothing for it; the part is made here rather
        # than carried, so the text is never written twice.
        self.assertEqual(plain.parts, "[]")
        made = message_parts(plain)
        self.assertEqual([part.value for part in made], [plain.text])
        self.assertTrue(made[0].is_text)

    def test_a_message_the_locale_lacks_is_reported_rather_than_invented(self) -> None:
        self.assertTrue(self.ctx.intl.has("sdk.reason.grahaInBhava"))
        self.assertFalse(self.ctx.intl.has("sdk.nope.missing"))

    def test_an_entity_carries_its_forms(self) -> None:
        sun = self.ctx.intl.entity("graha.SUN")
        self.assertTrue(sun.name)
        self.assertTrue(sun.iast)

    def test_transliteration_changes_the_script(self) -> None:
        latin = self.ctx.intl.transliterate("सूर्य")
        self.assertTrue(latin)
        self.assertNotEqual(latin, "सूर्य")


class Positions(WithLibrary):
    def setUp(self) -> None:
        self.ctx = self.teistro.context(profile=PROFILE, locale=LOCALE, test_provider=True)

    def tearDown(self) -> None:
        self.ctx.close()

    def test_a_grid_comes_back_instants_outermost(self) -> None:
        sky = self.ctx.positions(
            instants=[2451545.0, 2451546.0],
            bodies=[Body.SUN, Body.MOON, Body.MARS],
        )
        self.assertEqual((sky.instant_count, sky.body_count), (2, 3))
        self.assertEqual(sky.cell_count, 6)
        self.assertEqual(sky.time_scale, TimeScale.UT1)
        self.assertEqual(sky.body_keys, [Body.SUN, Body.MOON, Body.MARS])
        for instant in range(2):
            for body in range(3):
                cell = sky.at(instant, body)
                self.assertEqual(cell.status, 0, "every cell has a value")
                self.assertGreaterEqual(cell.longitude, 0.0)
                self.assertLess(cell.longitude, 360.0)
        # The Moon moves faster than the Sun, whatever the ephemeris.
        self.assertGreater(
            abs(sky.at(0, 1).longitude_speed), abs(sky.at(0, 0).longitude_speed)
        )

    def test_the_grid_refuses_a_cell_outside_it(self) -> None:
        sky = self.ctx.positions(instants=[2451545.0], bodies=[Body.SUN])
        with self.assertRaises(IndexError):
            sky.at(1, 0)
        with self.assertRaises(IndexError):
            sky.at(0, 1)

    def test_an_empty_request_is_refused_before_the_boundary(self) -> None:
        with self.assertRaises(ValueError):
            self.ctx.positions(instants=[], bodies=[Body.SUN])
        with self.assertRaises(ValueError):
            self.ctx.positions(instants=[2451545.0], bodies=[])

    def test_the_provenance_says_what_computed_it(self) -> None:
        sky = self.ctx.positions(instants=[2451545.0], bodies=[Body.SUN])
        provenance = sky.provenance
        self.assertEqual(provenance.profile, PROFILE)
        self.assertEqual(provenance.settings_hash, self.ctx.settings_hash)
        self.assertIsInstance(sky.steps_applied, list)
        self.assertEqual(
            sky.frame(self.teistro).centre, self.teistro.canonical_frame.centre
        )

    def test_a_context_with_no_ephemeris_refuses_by_capability(self) -> None:
        with self.teistro.context(profile=PROFILE) as bare:
            with self.assertRaises(TeistroError) as caught:
                bare.positions(instants=[2451545.0], bodies=[Body.SUN])
            self.assertEqual(caught.exception.status, Status.CAPABILITY)
            # The field and the hint name the option this binding sets,
            # not the C entry point it has no access to -- the same pair
            # Node, Dart and C are held to.
            self.assertEqual(caught.exception.field, "ephemeris")
            self.assertIn("BUILTIN", caught.exception.hint or "")

    def test_a_birth_with_no_time_is_refused_or_reported_never_guessed(self) -> None:
        day = date(Calendar.BIKRAM_SAMBAT, 2042, 9, 17)
        zone = iana_zone("Asia/Kathmandu")

        # No policy: refused by name, with the hint naming the choices.
        with self.teistro.context(profile=PROFILE, test_provider=True) as strict:
            with self.assertRaises(TeistroError) as caught:
                strict.time.resolve(when_unknown(day), zone)
            self.assertIn("has no time of day", caught.exception.message)
            self.assertIn("NOON, MIDNIGHT or SUNRISE", caught.exception.hint or "")
            self.assertEqual(caught.exception.field, "time")

            # A known time on the same date resolves with the time known
            # and no warning: this record sits on the day Nepal moved to
            # +05:45.
            exact = strict.time.resolve(at(day, hour=0, minute=20), zone)
            self.assertTrue(exact.time_known)
            self.assertEqual(exact.offset_seconds, 5 * 3600 + 45 * 60)
            self.assertEqual(list(exact.warnings), [])

        # NOON: answered, and said twice — the resolution reports the time
        # as unknown *and* warns, so a stored chart cannot claim a time it
        # never had. `settings` as a mapping is the shape the Node and
        # Dart bindings take.
        with self.teistro.context(
            profile=PROFILE,
            test_provider=True,
            settings={"time": {"unknown_time": "NOON"}},
        ) as noon:
            resolved = noon.time.resolve(when_unknown(day), zone)
            self.assertFalse(resolved.time_known)
            self.assertIn(
                "TIME_UNKNOWN_FALLBACK", [w.key for w in resolved.warnings]
            )

    def test_settings_given_twice_is_refused(self) -> None:
        with self.assertRaises(ValueError):
            self.teistro.context(profile=PROFILE, settings={}, settings_json="{}")

    def test_a_closed_context_says_so_and_closing_twice_is_allowed(self) -> None:
        ctx = self.teistro.context(profile=PROFILE, test_provider=True)
        self.assertTrue(ctx.profile)
        ctx.close()
        # Idempotent: a `with` block and an explicit call both run it.
        ctx.close()
        # Named here rather than at the boundary, which would only say
        # `invalid argument` and not which argument. The Node and Dart
        # bindings answer the same way.
        for call in (
            lambda: ctx.profile,
            lambda: ctx.positions(instants=[2451545.0], bodies=[Body.SUN]),
        ):
            with self.assertRaises(TeistroError) as caught:
                call()
            self.assertIn("closed", str(caught.exception))


if __name__ == "__main__":
    unittest.main()


class AnEngine(WithLibrary):
    """The engine's own operations, through the proxy.

    The test provider ships a two-function manifest, so this runs with no
    real engine present and still exercises the whole route: the manifest
    crosses, a call crosses, and the names come from the engine rather
    than from this package.
    """

    def setUp(self) -> None:
        self.ctx = self.teistro.context(
            profile=PROFILE, locale=LOCALE, test_provider=True
        )

    def tearDown(self) -> None:
        self.ctx.close()

    def test_the_engine_names_its_own_operations(self) -> None:
        engine = self.ctx.engine
        self.assertIn("tp_echo", engine.names)
        self.assertIn("tp_echo", engine)
        self.assertEqual(len(engine), len(engine.names))
        self.assertEqual(engine.manifest["engine"], "test-provider")

    def test_an_operation_is_called_by_the_name_the_engine_gives_it(self) -> None:
        engine = self.ctx.engine
        self.assertEqual(engine.tp_echo(value=6.0), {"value": 6.0})
        self.assertEqual(engine.call("tp_echo", value=6.0), {"value": 6.0})
        summed = engine.tp_sum(values=[1.0, 2.0, 3.5])
        self.assertEqual(summed["total"], 6.5)

    def test_the_names_come_from_the_engine_and_not_from_this_package(self) -> None:
        engine = self.ctx.engine
        # `dir` lists what the engine offers, so a REPL completes them
        # without this package ever holding a list.
        self.assertIn("tp_sum", dir(engine))
        # And a name it does not have is an AttributeError that says so.
        with self.assertRaises(AttributeError) as caught:
            engine.tm_eclipse_when  # noqa: B018
        self.assertIn("tm_eclipse_when", str(caught.exception))

    def test_the_manifest_carries_the_role_of_every_parameter(self) -> None:
        engine = self.ctx.engine
        signature = engine.signature("tp_sum")
        assert signature is not None
        roles = [param["role"] for param in signature["params"]]
        self.assertEqual(roles, ["array_in", "array_len", "scalar_out"])
        self.assertIsNone(engine.signature("tm_no_such_thing"))

    def test_the_engines_own_refusal_comes_back(self) -> None:
        engine = self.ctx.engine
        with self.assertRaises(TeistroError) as caught:
            engine.call("tm_eclipse_when")
        self.assertIn("tm_eclipse_when", str(caught.exception))

    def test_an_ephemeris_is_plugged_in_by_naming_its_platform_binary(self) -> None:
        """**An engine, plugged in** (ADR-0029): the 98% path.

        A consumer names an adapter's platform binary and never sees a
        vtable. It runs only where the adapter has been built and its
        data is present, because a checkout has neither and a test that
        failed for that would fail for everyone.
        `TEISTRO_TEIMERIS_ADAPTER` names the library -- the same variable
        `crates/ffi/tests/abi.rs` reads for the same reason.
        """
        plugin = os.environ.get("TEISTRO_TEIMERIS_ADAPTER")
        if not plugin:
            self.skipTest(
                "the adapter is built separately; "
                "set TEISTRO_TEIMERIS_ADAPTER to its library"
            )
        with self.teistro.context(profile=PROFILE, ephemeris=Plugin(plugin)) as ctx:
            sky = ctx.positions(instants=[2451545.0], bodies=[Body.SUN])
            # The Sun at J2000 is near 280.4 degrees, which is astronomy
            # rather than this package: what is tested is that a real
            # engine answered.
            self.assertAlmostEqual(sky.at(0, 0).longitude, 280.37, delta=0.5)
            # And its own functions came with it, which no SDK operation
            # offers.
            self.assertEqual(ctx.engine.manifest["engine"], "teimeris")
            self.assertEqual(ctx.engine.call("tm_body_name", body=0)["buf"], "Sun")

    def test_an_ephemeris_chain_is_tried_in_order_and_refuses_naming_each(
        self,
    ) -> None:
        """A chain is **ordered and explicit** (ADR-0029).

        Tried in order, and a refusal names every entry that failed
        rather than only the last, which would hide the one the caller
        actually wanted. Needs no adapter.
        """
        # An adapter that is not there, then the built-in: the fallback
        # the caller wrote down.
        with self.teistro.context(
            profile=PROFILE,
            ephemeris=[Plugin("/nowhere/adapter.so"), Ephemeris.BUILTIN],
        ) as fell_back:
            sky = fell_back.positions(instants=[2451545.0], bodies=[Body.SUN])
            self.assertAlmostEqual(sky.at(0, 0).longitude, 280.37, delta=0.5)

        # Nothing in the chain opening is one refusal that names each.
        with self.assertRaises(ValueError) as caught:
            self.teistro.context(ephemeris=[Plugin("/a.so"), Plugin("/b.so")])
        self.assertIn("/a.so", str(caught.exception))
        self.assertIn("/b.so", str(caught.exception))

        # A chain of none names nothing, which is a mistake rather than a
        # default.
        with self.assertRaises(ValueError) as empty:
            self.teistro.context(ephemeris=[])
        self.assertIn("names nothing", str(empty.exception))

    def test_a_provider_and_a_named_ephemeris_together_are_refused(self) -> None:
        """Each answers one question, so both together is a refusal."""
        # The refusal happens before anything touches the provider, so
        # the base class as it stands is provider enough: `name`,
        # `bodies` and `positions` are attributes with defaults.
        with self.assertRaises(ValueError) as caught:
            self.teistro.context(
                ephemeris=Ephemeris.BUILTIN, provider=EphemerisProvider()
            )
        self.assertIn("give one of them", str(caught.exception))

    def test_a_context_without_an_ephemeris_says_so(self) -> None:
        with self.teistro.context(profile=PROFILE) as bare:
            with self.assertRaises(TeistroError):
                bare.engine  # noqa: B018

    def test_an_area_is_a_value_that_can_be_held_and_passed(self) -> None:
        """An area is built once with the context and kept.

        That is what makes the grouping worth having rather than merely
        tidy: a consumer may hold one and pass it to something that needs
        only that much of the SDK (`03-design/surface-areas.md`).
        """
        calendar = self.ctx.calendar
        self.assertIs(calendar, self.ctx.calendar, "the same object every read")
        self.assertTrue(calendar.is_leap(Calendar.GREGORIAN, 2024))
        self.assertGreater(self.ctx.time.delta_t(2451545.0).seconds, 60)
        self.assertEqual(self.ctx.keys.name(self.ctx.keys.id("graha.SUN")), "graha.SUN")

    def test_a_drawing_names_a_layout_and_a_varga_or_is_refused(self) -> None:
        """A drawing is a `(ChartLayout, Varga)` pair; anything else is
        refused naming its place in the list, before the boundary is
        crossed (`03-design/chart-geometry.md`)."""
        observer = Observer(
            latitude_deg=Latitude(27.7172), longitude_deg=Longitude(85.324), altitude_m=Altitude(0)
        )
        for wrong in ((Varga.D1, ChartLayout.NORTH_INDIAN), (ChartLayout.NORTH_INDIAN,), "north_indian"):
            with self.subTest(wrong=wrong), self.assertRaises(TeistroError) as caught:
                self.ctx.chart.found(
                    instant=2451545.0,
                    place=observer,
                    utc_offset_seconds=0,
                    drawings=[(ChartLayout.SOUTH_INDIAN, Varga.D9), wrong],  # type: ignore[list-item]
                )
            self.assertEqual(caught.exception.field, "drawings[1]")

    def test_a_chart_handed_out_alone_carries_its_own_hash(self) -> None:
        """A batch's provenance hashes the list; a chart of it carries the
        hash of its own value, which is what a stored chart is checked
        against (STATUS 2h). The provenance is typed, and a key the SDK does
        not write is refused rather than passed through."""
        place = Observer(
            latitude_deg=Latitude(27.7172), longitude_deg=Longitude(85.324), altitude_m=Altitude(1400)
        )
        batch = self.ctx.chart.found_many(
            instants=[2451545.0, 2451546.0], place=place, utc_offset_seconds=20700
        )
        first, second = batch.at(0).provenance, batch.at(1).provenance
        whole = batch.provenance
        self.assertEqual(len({first.content_hash, second.content_hash, whole.content_hash}), 3)
        self.assertEqual(first.settings_hash, whole.settings_hash)
        self.assertEqual(first.sdk_version, whole.sdk_version)
        stored = decode_provenance(json.loads(batch.provenance_json))
        self.assertEqual(stored, whole)
        with self.assertRaises(ValueError):
            decode_provenance({**json.loads(batch.provenance_json), "confidence": "MAYBE"})

    def test_the_surya_siddhanta_opens_by_name_and_its_chart_says_so(self) -> None:
        """A classical astronomy's chart is the text's throughout
        (docs/03-design/classical-chart.md), and the envelope says which
        parts; a modern chart's says nothing."""
        place = Observer(
            latitude_deg=Latitude(27.7172), longitude_deg=Longitude(85.324), altitude_m=Altitude(1400)
        )
        at: dict[str, Any] = {
            "instant": 2447995.4895833335,
            "place": place,
            "utc_offset_seconds": 20700,
        }
        with self.teistro.context(
            profile="surya-siddhanta",
            ephemeris=Ephemeris.SURYA_SIDDHANTA,
        ) as text:
            deviation = text.chart.found(**at).provenance.deviation
        assert deviation is not None
        self.assertEqual(deviation.model, "SURYA_SIDDHANTA")
        self.assertEqual(
            deviation.detail,
            "the zodiac, the places, the angles and the day are the provider's own",
        )
        self.assertIsNone(self.ctx.chart.found(**at).provenance.deviation)

    def test_a_charts_day_is_the_almanacs_and_its_date_converts(self) -> None:
        """A chart's day and an almanac's are one record, and its date is the
        one `calendar.convert` takes. Before, Python flattened the weekday
        and the sunrise onto the chart and left the rest out."""
        place = Observer(
            latitude_deg=Latitude(27.7172),
            longitude_deg=Longitude(85.324),
            altitude_m=Altitude(1400),
        )
        day = self.ctx.chart.found(
            instant=2451545.0, place=place, utc_offset_seconds=20700
        ).day
        self.assertEqual(
            self.ctx.almanac.day(date=day.date, place=place, utc_offset_seconds=20700).day,
            day,
        )
        self.assertEqual(
            (day.date.calendar, day.date.era, day.date.year, day.date.month, day.date.day),
            (Calendar.BIKRAM_SAMBAT, Era.VIKRAMA, 2056, 9, 17),
        )
        gregorian = self.ctx.calendar.convert(day.date, Calendar.GREGORIAN)
        self.assertEqual((gregorian.year, gregorian.month, gregorian.day), (2000, 1, 1))
        self.assertEqual(day.vara, Vara.SHANIVARA)
        self.assertTrue(day.sunrise < day.sunset < 2451545.0 < day.next_sunrise)
        self.assertIsNone(day.polar)
        self.assertEqual(day.convention, Sunrise.CENTRE_NO_REFRACTION)
        self.assertIsNone(day.custom_altitude_deg)

        # A custom altitude is a number and no named convention.
        custom = {"day": {"sunrise": {"kind": "CUSTOM", "altitude_deg": -0.5}}}
        with self.teistro.context(
            profile=PROFILE, settings=custom, test_provider=True
        ) as ctx:
            own = ctx.chart.found(instant=2451545.0, place=place, utc_offset_seconds=20700).day
            self.assertEqual((own.convention, own.custom_altitude_deg, own.air), (None, -0.5, None))

        # An air named for a refracted convention: what was left out comes
        # back resolved at the place, 856 hPa at 1400 m, and the thinner
        # air lifts the Sun less, so it clears the horizon later than under
        # the fixed 34 arcminutes.
        def refracted(sunrise: dict[str, Any]) -> Any:
            with self.teistro.context(
                profile=PROFILE, settings={"day": {"sunrise": sunrise}}, test_provider=True
            ) as each:
                return each.chart.found(
                    instant=2451545.0, place=place, utc_offset_seconds=20700
                ).day

        almanac = refracted({"kind": "NAMED", "which": "UPPER_LIMB_REFRACTION"})
        standard = refracted(
            {"kind": "ATMOSPHERIC", "which": "UPPER_LIMB_REFRACTION", "air": {}}
        )
        self.assertIsNone(almanac.air)
        self.assertEqual(standard.convention, Sunrise.UPPER_LIMB_REFRACTION)
        self.assertAlmostEqual(standard.air.pressure_hpa, 855.99, delta=0.01)
        self.assertEqual(standard.air.temperature_c, 15.0)
        later = (standard.sunrise - almanac.sunrise) * 86400.0
        self.assertTrue(20.0 < later < 35.0, later)
        weather = refracted(
            {
                "kind": "ATMOSPHERIC",
                "which": "LOWER_LIMB_REFRACTION",
                "air": {"pressure_hpa": 870.0, "temperature_c": -4.5},
            }
        )
        self.assertEqual(weather.air, Air(pressure_hpa=870.0, temperature_c=-4.5))
        with self.assertRaisesRegex(TeistroError, "does not refract"):
            refracted({"kind": "ATMOSPHERIC", "which": "CENTRE_NO_REFRACTION", "air": {}})

        # Tromsø at midsummer: civil midnight holds the instant and says so;
        # the nearest real sunrise is weeks away, and the refusal names the
        # policy rather than the instant.
        tromso = Observer(
            latitude_deg=Latitude(69.65), longitude_deg=Longitude(18.96), altitude_m=Altitude(0)
        )
        def policy(name: str) -> dict[str, object]:
            return {"day": {"polar_day_policy": name}}

        with self.teistro.context(
            profile=PROFILE, settings=policy("CIVIL_MIDNIGHT"), test_provider=True
        ) as ctx:
            polar = ctx.chart.found(
                instant=2451716.5, place=tromso, utc_offset_seconds=7200
            ).day.polar
            self.assertEqual(polar, PolarDay(kind=PolarKind.DAY, policy=PolarDayPolicy.CIVIL_MIDNIGHT))
        with self.teistro.context(
            profile=PROFILE, settings=policy("NEAREST_EVENT"), test_provider=True
        ) as ctx:
            with self.assertRaises(TeistroError) as caught:
                ctx.chart.found(instant=2451716.5, place=tromso, utc_offset_seconds=7200)
            self.assertEqual(caught.exception.field, "day.polar_day_policy")

    def test_a_chart_carries_the_day_it_belongs_to_and_both_house_readings(self) -> None:
        """A founded chart knows more than where the grahas are: which arc
        of its day it fell in and how far through, the lagna at the sunrise
        that opened it, the ayanamsha applied, and the twelve bhavas under
        the chalit beside the ones under the placement system. None of it
        had been read from Python
        (`03-design/binding-exercise-measured.md`)."""
        observer = Observer(
            latitude_deg=Latitude(27.7172),
            longitude_deg=Longitude(85.324),
            altitude_m=Altitude(1400),
        )
        chart = self.ctx.chart.found(
            instant=2451545.0, place=observer, utc_offset_seconds=20700, houses=True
        )

        self.assertIn(chart.day_part, (DayPart.DAYLIGHT, DayPart.NIGHT))
        self.assertGreaterEqual(chart.day_elapsed, 0.0)
        self.assertLessEqual(chart.day_elapsed, 1.0)
        self.assertGreaterEqual(chart.day_lagna_deg, 0.0)
        self.assertLess(chart.day_lagna_deg, 360.0)
        # The context is sidereal, so an ayanamsha was applied -- and it is
        # named, where the blob had carried it and nothing had read it.
        self.assertNotEqual(chart.ayanamsha_offset_deg, 0.0)
        self.assertEqual(chart.ayanamsha, Ayanamsha.LAHIRI)
        self.assertFalse(chart.ayanamsha_custom)
        # A tropical chart has none, which is not an ayanamsha of nought.
        with self.teistro.context(
            profile=PROFILE,
            settings={"frame": {"zodiac": "TROPICAL"}},
            test_provider=True,
        ) as tropical:
            western = tropical.chart.found(
                instant=2451545.0, place=observer, utc_offset_seconds=20700
            )
            self.assertIsNone(western.ayanamsha)
            self.assertFalse(western.ayanamsha_custom)

        # Both house readings are kept; the chalit is the other twelve,
        # each with its own centre and opening cusp.
        chalit = chart.chalit
        self.assertEqual(len(chalit), 12)
        for bhava in chalit:
            self.assertGreaterEqual(bhava.madhya_deg, 0.0)
            self.assertLess(bhava.madhya_deg, 360.0)
            self.assertGreaterEqual(bhava.sandhi_deg, 0.0)
            self.assertLess(bhava.sandhi_deg, 360.0)

    def test_a_divisional_chart_says_where_a_body_moved_and_where_it_stayed(self) -> None:
        """A varga placement knows the sign the division put a body in and
        whether that is the sign it was already in — the D1 leaves every
        body where it was, and a D9 rarely does."""
        observer = Observer(
            latitude_deg=Latitude(27.7172),
            longitude_deg=Longitude(85.324),
            altitude_m=Altitude(1400),
        )
        chart = self.ctx.chart.found(
            instant=2451545.0,
            place=observer,
            utc_offset_seconds=20700,
            vargas=[Varga.D1, Varga.D9],
        )
        first, ninth = chart.vargas
        self.assertEqual(first.varga, Varga.D1)
        self.assertTrue(
            all(placed.at.keeps_its_sign for placed in first.grahas),
            "the D1 is the rashi chart and moves nothing",
        )
        self.assertEqual(ninth.varga, Varga.D9)
        moved = [p for p in ninth.grahas if not p.at.keeps_its_sign]
        self.assertTrue(moved, "a navamsha moves most bodies")
        for placed in moved:
            self.assertNotEqual(placed.at.sign, placed.at.rashi)

    def test_the_last_error_is_the_failing_call_s_own(self) -> None:
        """A refusal crosses whole, and the context keeps the last one: a
        consumer that caught the exception can still read what the library
        said about it."""
        with self.assertRaises(TeistroError):
            self.ctx.intl.entity("graha.SUNN")
        last = self.ctx.last_error
        self.assertEqual(last.status, Status.UNSUPPORTED)
        self.assertIn("SUNN", last.message)

    def test_a_theme_writes_each_drawing_as_svg_and_a_wrong_one_is_refused(self) -> None:
        """A theme writes every drawing as SVG in the context's locale, and
        a wrong one is refused by its path (`03-design/render-svg.md`)."""
        observer = Observer(
            latitude_deg=Latitude(27.7172), longitude_deg=Longitude(85.324), altitude_m=Altitude(1400)
        )
        drawings = [(ChartLayout.NORTH_INDIAN, Varga.D1), (ChartLayout.WESTERN_WHEEL, Varga.D1)]

        def found(theme: Optional[Theme]) -> list[Drawing]:
            return self.ctx.chart.found(
                instant=2451545.0, place=observer, utc_offset_seconds=20700, drawings=drawings, theme=theme
            ).drawings

        self.assertIsNone(found(None)[0].svg, "no theme, no SVG")
        north, wheel = found("DARK")
        assert north.svg is not None and wheel.svg is not None
        self.assertTrue(north.svg.startswith('<svg xmlns="http://www.w3.org/2000/svg"'))
        self.assertIn('data-body="graha.SUN">सू', north.svg)
        self.assertIn('fill="#121212"', north.svg)
        self.assertIn("<line ", wheel.svg)

        glyphs = found({"extends": "LIGHT", "style": {"size": 600}, "content": {"body_form": "GLYPH"}})[0].svg
        assert glyphs is not None
        self.assertIn('viewBox="0 0 600 600"', glyphs)
        self.assertIn('data-body="graha.SUN">☉', glyphs)

        with self.assertRaises(TeistroError) as wrong:
            found({"style": {"ink": "black"}})
        self.assertEqual(wrong.exception.field, "theme.style.ink")
        # A theme is named by its key, as every other word is; the lowercase
        # name is refused with the keys it could have been.
        for name in ("sepia", "dark"):
            with self.assertRaises(TeistroError) as unknown:
                found(name)  # type: ignore[arg-type]
            self.assertEqual(unknown.exception.field, "theme.extends")
            self.assertIn('"DARK"', unknown.exception.hint or "")

    def test_rules_are_answered_in_the_same_crossing_and_a_wrong_one_is_refused(self) -> None:
        """A request's rules come back as each chart's `rules`, a consumer's own
        rule naming a shipped one by key, with the longevity readings; a rule
        that does not read is refused by its place
        (`03-design/rules-at-the-boundary.md`)."""
        observer = Observer(
            latitude_deg=Latitude(27.7172), longitude_deg=Longitude(85.324), altitude_m=Altitude(1400)
        )

        def found(rules: Optional[RuleRequest]) -> Optional[RulesReading]:
            return self.ctx.chart.found(
                instant=2451545.0, place=observer, utc_offset_seconds=20700, rules=rules
            ).rules

        self.assertIsNone(found(None), "no rules, no answers")
        answered = found({"shipped": ["NABHASAS"], "longevity": True})
        assert answered is not None
        present = answered["present"]
        self.assertTrue(present)
        for held in present:
            self.assertIsInstance(held["rule"], str)
            self.assertIs(held["result"]["present"], True)
        self.assertIsInstance(answered["longevity"]["ayurdaya"]["pindayu"]["years"], float)

        mine = {"key": "MINE", "category": "raja", "source": {"text": "BPHS"},
                "conditions": [{"type": "rule", "key": present[0]["rule"]}]}
        with_mine = found({"shipped": ["NABHASAS"], "rules": [mine]})
        assert with_mine is not None
        self.assertTrue(any(held["rule"] == "MINE" for held in with_mine["present"]))

        with self.assertRaises(TeistroError) as wrong:
            found({"rules": [{"key": "X", "category": "raja"}]})
        self.assertEqual(wrong.exception.field, "rules.rules[0]")

    def test_plans_compose_in_the_same_crossing_and_render_with_nothing_in_between(self) -> None:
        """A request's plans come back as each chart's `plans`, holding no
        words — and this says them, by handing each item's `params` straight
        to `intl.render`. That is the property the crossing exists for: no
        step between the plan and the renderer
        (`03-design/plans-at-the-boundary.md`)."""
        observer = Observer(
            latitude_deg=Latitude(27.7172), longitude_deg=Longitude(85.324), altitude_m=Altitude(1400)
        )

        def found(interpret: Optional[PlanRequest], rules: Optional[RuleRequest] = None) -> Optional[Plans]:
            return self.ctx.chart.found(
                instant=2451545.0,
                place=observer,
                utc_offset_seconds=20700,
                rules=rules,
                interpret=interpret,
            ).plans

        self.assertIsNone(found(None), "no composer, no plans")
        plans = found(
            {
                "placements": True,
                "readings": True,
                "strength": True,
                "houses": True,
                "positions": True,
                "aspects": True,
                "conditions": True,
                "karakas": True,
                "chalit": True,
                "states": True,
                "bhavaBala": True,
                "vimshopaka": True,
                "panchanga": True,
                "dashaPhala": True,
                "ashtakavarga": True,
            },
            {"shipped": ["NABHASAS"]},
        )
        assert plans is not None
        self.assertTrue(plans["placements"], "every chart places its grahas")
        self.assertIsInstance(plans["readings"], list)
        # The strengths read the Shadbala, which this request never asked
        # for: a composer's own section is computed for it.
        self.assertEqual(
            len(plans["strength"]), 7 * 2, "a score and a sufficiency each"
        )
        # And the houses read the bhavas, which it never asked for either.
        self.assertEqual(len(plans["houses"]), 12 * 2, "a sign and a lord each")
        # And the positions read the same states the placements do.
        self.assertEqual(len(plans["positions"]), 10, "the lagna, then the nine")
        # The chalit needs no section: both readings are on the grahas.
        self.assertLessEqual(len(plans["chalit"]), 9, "at most one a graha")
        # The drishtis read the aspects section, never asked for either.
        self.assertTrue(plans["aspects"], "every chart holds a drishti")
        # The conditions and the karakas read the same states the
        # placements do, so one section serves four composers.
        self.assertGreaterEqual(
            len(plans["conditions"]), 18, "a dignity and a navamsha each"
        )
        self.assertEqual(len(plans["karakas"]), 15, "seven and eight")
        # The dasha phala reads its own section, computed for it like the
        # rest: three items a graha always, and a fourth only where the
        # placement tilts the dasha one way or the other.
        self.assertGreaterEqual(len(plans["dashaPhala"]), 9 * 3, "three each")
        self.assertLessEqual(len(plans["dashaPhala"]), 9 * 4, "four at most")
        # The states say the half of that section a placement never carried.
        self.assertGreaterEqual(len(plans["states"]), 9 * 3, "three a graha")
        # The almanac says the five limbs and the Moon's pada.
        self.assertEqual(len(plans["bhavaBala"]), 12, "one a bhava")
        self.assertEqual(len(plans["vimshopaka"]), 7 * 4, "seven by four")
        self.assertGreaterEqual(len(plans["panchanga"]), 6, "limbs and pada")
        self.assertLessEqual(len(plans["panchanga"]), 7, "and the day")
        # The Ashtakavarga reads its own section *and* the placements: a
        # graha's bindus are the ones of the sign it stands in.
        self.assertEqual(
            len(plans["ashtakavarga"]), 7 + 12, "a graha each, then a sign each"
        )

        said = 0
        for item in [
            *plans["placements"],
            *plans["readings"],
            *plans["strength"],
            *plans["houses"],
            *plans["positions"],
            *plans["aspects"],
            *plans["conditions"],
            *plans["karakas"],
            *plans["dashaPhala"],
            *plans["states"],
            *plans["panchanga"],
            *plans["bhavaBala"],
            *plans["vimshopaka"],
            *plans["ashtakavarga"],
        ]:
            self.assertTrue(item["key"].startswith("sdk."))
            rendered = self.ctx.intl.render(item["key"], item["params"])
            self.assertTrue(rendered.text, f"{item['key']} said nothing")
            self.assertEqual(rendered.is_fallback, 0, f"{item['key']} fell back")
            self.assertEqual(rendered.warning_count, 0, f"{item['key']} warned")
            said += 1
        self.assertGreater(said, 20, f"only {said} items said")

        # A reading says what the rules answered, so it needs rules beside it.
        with self.assertRaises(TeistroError) as alone:
            found({"readings": True})
        self.assertEqual(alone.exception.field, "interpret.readings")
        # The Sade Sati plan says what a window found, so it needs one beside
        # it; with one and no pack loaded it is present and empty, which is
        # an answer.
        with self.assertRaises(TeistroError) as unsearched:
            found({"sadeSati": True})
        self.assertEqual(unsearched.exception.field, "interpret.sadeSati")
        periods = self.ctx.chart.found(
            instant=2451545.0,
            place=observer,
            utc_offset_seconds=20700,
            sade_sati={"from": 2451545.0, "to": 2462502.5},
            interpret={"sadeSati": True},
        )
        assert periods.plans is not None and periods.sade_sati is not None
        self.assertEqual(periods.plans["sadeSati"], [], "no pack, no words")
        self.assertTrue(periods.sade_sati.sade_sati, "the report is on the chart beside the plan")
        # And a composer that is not one is refused beside the ones that are.
        with self.assertRaises(TeistroError) as typo:
            found({"readigns": True})  # type: ignore[arg-type]
        self.assertEqual(typo.exception.field, "interpret")

    def test_a_chart_carries_its_ashtakavarga_and_each_graha_s_reductions(self) -> None:
        """A chart's Ashtakavarga crosses whole: each graha's bindus holding the
        classical totals, the sum, and each graha's reductions under the default
        reading; None unless asked."""
        observer = Observer(
            latitude_deg=Latitude(27.7172), longitude_deg=Longitude(85.324), altitude_m=Altitude(1400)
        )
        chart = self.ctx.chart.found(instant=2451545.0, place=observer, utc_offset_seconds=20700, ashtakavarga=True)
        self.assertIsNone(self.ctx.chart.found(instant=2451545.0, place=observer, utc_offset_seconds=20700).ashtakavarga)
        av = chart.ashtakavarga
        assert av is not None
        self.assertEqual((av.shodhana, av.ekadhipatya), (Shodhana.EACH_GRAHA, Ekadhipatya.BPHS))
        self.assertEqual([sum(g.bindus) for g in av.grahas], [48, 49, 39, 54, 56, 52, 39])
        self.assertEqual(sum(av.sarva), 337)
        reduced = [g.reduced for g in av.grahas]
        assert all(r is not None for r in reduced)
        self.assertEqual(
            av.reduced,
            tuple(sum(r[sign] for r in reduced if r is not None) for sign in range(12)),
        )
        self.assertTrue(all(g.yoga_pinda == g.rashi_pinda + g.graha_pinda for g in av.grahas))

    def test_a_chart_carries_its_bhava_bala_each_bhava_s_strength(self) -> None:
        """A chart's Bhava bala crosses whole: every bhava's components under the
        default reading, the verses', whose totals are their parts'; None unless
        asked."""
        observer = Observer(
            latitude_deg=Latitude(27.7172), longitude_deg=Longitude(85.324), altitude_m=Altitude(1400)
        )
        chart = self.ctx.chart.found(instant=2451545.0, place=observer, utc_offset_seconds=20700, bhava_bala=True)
        self.assertIsNone(self.ctx.chart.found(instant=2451545.0, place=observer, utc_offset_seconds=20700).bhava_bala)
        bb = chart.bhava_bala
        assert bb is not None
        self.assertEqual([b.bhava for b in bb.bhavas], list(range(1, 13)))
        for b in bb.bhavas:
            self.assertAlmostEqual(b.adhipati + b.dig + b.drishti + b.special, b.virupas, places=9)
            self.assertTrue(0.0 <= b.dig <= 60.0)

    def test_a_chart_carries_its_shadbala_each_graha_s_six_strengths(self) -> None:
        """A chart's Shadbala crosses whole: every graha's six strengths under
        the default reading, the chapter's, whose natural strengths are 28
        sevenths of a rupa and whose totals are their components'; None unless
        asked."""
        observer = Observer(
            latitude_deg=Latitude(27.7172), longitude_deg=Longitude(85.324), altitude_m=Altitude(1400)
        )
        chart = self.ctx.chart.found(instant=2451545.0, place=observer, utc_offset_seconds=20700, shadbala=True)
        self.assertIsNone(self.ctx.chart.found(instant=2451545.0, place=observer, utc_offset_seconds=20700).shadbala)
        sb = chart.shadbala
        assert sb is not None
        self.assertEqual(len(sb.grahas), 7)
        self.assertAlmostEqual(sum(g.naisargika for g in sb.grahas), 240.0, places=9)
        for g in sb.grahas:
            six = g.sthana.total + g.dig + g.kaala.total + g.cheshta + g.naisargika + g.drik
            self.assertAlmostEqual(six, g.virupas, places=9)
            self.assertEqual(g.strong, g.rupas >= g.required_rupas)

    def test_a_chart_carries_its_hit_list_the_sky_once_for_the_batch(self) -> None:
        """The transit hit list crosses whole: empty unless asked; the sky's
        events every chart's alike and the aspects each chart's own; sorted;
        each event its own class; an aspect's `to` taken back as a point; and
        a bad request refused by the field the caller wrote. On the built-in
        ephemeris, whose Mercury turns retrograde in the window."""
        from teistro import AspectHit, Hit, HitKind, HitRequest, NakshatraIngress, NatalPoint, SignIngress, Station

        observer = Observer(
            latitude_deg=Latitude(27.7172), longitude_deg=Longitude(85.324), altitude_m=Altitude(1400)
        )
        with self.teistro.context(profile=PROFILE, ephemeris=Ephemeris.BUILTIN) as ctx:
            self.assertEqual(ctx.chart.found(instant=2451545, place=observer, utc_offset_seconds=20700).hits, [])
            asked: HitRequest = {
                "from": 2460676.5,
                "to": 2460866.5,
                "grahas": [Graha.SUN, "graha.MERCURY", "SATURN"],
                "aspects": [0, 90, 180],
                "orbDeg": 2,
            }
            batch = ctx.chart.found_many(
                instants=[2447995.4895833335, 2451545], place=observer, utc_offset_seconds=20700, hits=asked
            )
            first, second = batch.at(0).hits, batch.at(1).hits

            def sky(hits: list[Hit]) -> list[Hit]:
                return [hit for hit in hits if not isinstance(hit.event, AspectHit)]

            self.assertEqual(sky(first), sky(second))
            self.assertNotEqual(first, second)
            self.assertEqual(first, sorted(first, key=lambda hit: hit.instant))
            kinds = {hit.event.kind for hit in first}
            self.assertEqual(kinds, set(HitKind))
            for hit in first:
                self.assertIn(hit.graha, (Graha.SUN, Graha.MERCURY, Graha.SATURN))
                if isinstance(hit.event, (SignIngress, NakshatraIngress)):
                    self.assertIsNotNone(hit.event.into)
                elif isinstance(hit.event, Station):
                    self.assertIn(hit.event.turns.key, ("DIRECT", "RETROGRADE"))
                else:
                    self.assertIn(hit.event.angle, (0, 90, 180))
            aspect = next(hit.event for hit in first if isinstance(hit.event, AspectHit))
            to_it: HitRequest = {**asked, "kinds": ["ASPECT"], "points": [aspect.to]}
            again = ctx.chart.found(
                instant=2447995.4895833335, place=observer, utc_offset_seconds=20700, hits=to_it
            ).hits
            self.assertTrue(again)
            self.assertTrue(all(isinstance(hit.event, AspectHit) and hit.event.to == aspect.to for hit in again))
            self.assertIsInstance(aspect.to, NatalPoint)
            refusals: list[tuple[HitRequest, str]] = [
                ({"from": 2460676.5, "to": 2460600.5}, "hits.to"),
                ({**asked, "orbDeg": 20}, "hits.orbDeg"),
                ({**asked, "grahas": ["PLUTO", "PLUTO"]}, "hits.grahas"),
            ]
            for bad, field in refusals:
                with self.assertRaises(TeistroError) as refused:
                    ctx.chart.found(instant=2451545, place=observer, utc_offset_seconds=20700, hits=bad)
                self.assertEqual(refused.exception.field, field)

    def test_a_lunar_return_is_the_moon_back_on_her_own_natal_place(self) -> None:
        """A body's returns are the hit list asked for its conjunction with
        its own natal place: the Moon's thirteen a year, a sidereal month
        apart, each a chart whose Moon stands on the radical one."""
        from teistro import AspectHit, HitKind, NatalPoint, returns_request

        paris = Observer(latitude_deg=Latitude(48.8534), longitude_deg=Longitude(2.3488), altitude_m=Altitude(0))
        asked = returns_request(2451546, 2451911.25)
        self.assertEqual(
            (asked["grahas"], asked["kinds"], asked["points"], asked["aspects"]),
            ([Graha.MOON], [HitKind.ASPECT], [Graha.MOON], [0]),
        )
        with self.teistro.context(profile=PROFILE, ephemeris=Ephemeris.BUILTIN) as ctx:
            lunar = ctx.chart.found(instant=2451545, place=paris, utc_offset_seconds=0, hits=asked).hits
            self.assertEqual(len(lunar), 13)
            for k, hit in enumerate(lunar):
                self.assertEqual(hit.graha, Graha.MOON)
                self.assertIsInstance(hit.event, AspectHit)
                assert isinstance(hit.event, AspectHit)
                self.assertEqual((hit.event.to, hit.event.angle), (NatalPoint("GRAHA", Graha.MOON), 0))
                if k > 0:
                    self.assertTrue(27 < hit.instant - lunar[k - 1].instant < 27.7)

            def moon(instant: float) -> float:
                chart = ctx.chart.found(instant=instant, place=paris, utc_offset_seconds=0)
                return next(g.longitude_deg for g in chart.grahas if g.graha == Graha.MOON)

            self.assertAlmostEqual(moon(lunar[0].instant), moon(2451545), delta=1 / 3600)

    def test_a_chart_carries_its_sade_sati_each_period_whole(self) -> None:
        """Sade Sati crosses whole: `None` unless asked; each Sade Sati its
        three phases in order; a period asked about at one instant inside it
        the one a decade's window finds; a batch each chart alone; what a
        report names taken back by a request; and a bad request refused by
        the field the caller wrote."""
        from teistro import GocharFrom, Reckoning, SadeSatiRequest

        observer = Observer(
            latitude_deg=Latitude(27.7172), longitude_deg=Longitude(85.324), altitude_m=Altitude(1400)
        )
        births = [2447995.4895833335, 2451545.2]
        at: dict[str, Any] = {"place": observer, "utc_offset_seconds": 20700}
        with self.teistro.context(profile=PROFILE, ephemeris=Ephemeris.BUILTIN) as ctx:
            self.assertIsNone(ctx.chart.found(instant=births[0], **at).sade_sati)
            for reckoning in Reckoning:
                asked: SadeSatiRequest = {"from": 2460676.5, "to": 2464329.0, "reckoning": reckoning, "spells": (4, 7, 8)}
                batch = ctx.chart.found_many(instants=births, sade_sati=asked, **at)
                for k, instant in enumerate(births):
                    report = batch.at(k).sade_sati
                    assert report is not None
                    self.assertEqual(report, ctx.chart.found(instant=instant, sade_sati=asked, **at).sade_sati)
                    self.assertIs(report.reckoning, reckoning)
                    self.assertIs(report.reference.from_, GocharFrom.MOON)
                    self.assertTrue(report.sade_sati or report.spells, "a decade holds a period")
                    for one in report.sade_sati:
                        self.assertEqual([spell.house for spell in one.phases], [12, 1, 2])
                        visits = sorted((v for spell in one.phases for v in spell.visits), key=lambda v: v.from_ or 0)
                        for before, after in zip(visits, visits[1:]):
                            assert before.to is not None and after.from_ is not None
                            self.assertLessEqual(before.to, after.from_)
                        peak = one.phases[1].visits[0]
                        assert peak.from_ is not None and peak.to is not None
                        # Asked at one instant inside its peak, from what the
                        # report named: the same Sade Sati, whole.
                        now: SadeSatiRequest = {
                            "from": (peak.from_ + peak.to) / 2,
                            "countedFrom": report.reference.from_,
                            "reckoning": report.reckoning,
                        }
                        found = ctx.chart.found(instant=instant, sade_sati=now, **at).sade_sati
                        assert found is not None
                        self.assertEqual(found.sade_sati, (one,))
                    for spell in report.spells:
                        self.assertIn(spell.house, (4, 7, 8))
            refusals: list[tuple[SadeSatiRequest, str]] = [
                ({"from": 2460676.5, "to": 2460600.5}, "sadeSati.to"),
                ({"from": 2460676.5, "spells": [2]}, "sadeSati.spells"),
            ]
            for bad, field in refusals:
                with self.assertRaises(TeistroError) as refused:
                    ctx.chart.found(instant=births[0], sade_sati=bad, **at)
                self.assertEqual(refused.exception.field, field)

    def test_a_chart_carries_its_kp_reading(self) -> None:
        """KP crosses whole, its keys made members: the lords bracket each
        planet, a horary number's lagna is the exact start of its sub while
        the ruling planets stay the moment's, a batch is each chart alone,
        and a chart in another zodiac is refused by name unless the request
        takes any (`03-design/kp.md`)."""
        from teistro import HouseSystem, KpReason, KpRequest

        observer = Observer(latitude_deg=Latitude(13.08), longitude_deg=Longitude(80.27), altitude_m=Altitude(6))
        births = [2447995.4895833335, 2451545.2]
        at: dict[str, Any] = {"place": observer, "utc_offset_seconds": 19800}
        with self.teistro.context(profile="kp-default", ephemeris=Ephemeris.BUILTIN) as ctx:
            self.assertIsNone(ctx.chart.found(instant=births[0], **at).kp)
            reading = ctx.chart.found(instant=births[0], kp={}, **at).kp
            assert reading is not None
            self.assertEqual(reading.chart.system, HouseSystem.PLACIDUS)
            self.assertEqual(len(reading.chart.cusps), 12)
            self.assertEqual(len(reading.significators.houses), 12)
            for planet in reading.chart.planets:
                self.assertIsInstance(planet.graha, Graha)
                span = planet.lords.sub_sub.span
                self.assertTrue(span.start <= planet.longitude < span.end, planet.graha)
            self.assertEqual(reading.ruling.rules.count, "FIVE")
            self.assertTrue(all(ruler.reasons for ruler in reading.ruling.rulers))
            self.assertTrue(all(isinstance(why, KpReason) for ruler in reading.ruling.rulers for why in ruler.reasons))
            self.assertLessEqual(len(reading.ruling.accepted), len(reading.ruling.rulers))

            asked: KpRequest = {"number": 74}
            horary = ctx.chart.found(instant=births[0], kp=asked, **at).kp
            assert horary is not None
            lagna = horary.chart.cusps[0]
            self.assertEqual(lagna.longitude, lagna.lords.sub.span.start, "the number opens its sub")
            self.assertEqual(horary.ruling, reading.ruling)

            batch = ctx.chart.found_many(instants=births, kp=asked, **at)
            for k, instant in enumerate(births):
                self.assertEqual(batch.at(k).kp, ctx.chart.found(instant=instant, kp=asked, **at).kp)

            refusals: list[tuple[KpRequest, str]] = [({"number": 250}, "kp.number"), ({"clock": 90000}, "kp.clock")]
            for bad, field in refusals:
                with self.assertRaises(TeistroError) as refused:
                    ctx.chart.found(instant=births[0], kp=bad, **at)
                self.assertEqual(refused.exception.field, field)
            with self.assertRaises(TeistroError) as refused:
                ctx.chart.found(instant=births[0], kp=74, **at)  # type: ignore[arg-type]
            self.assertEqual(refused.exception.field, "kp")
        with self.teistro.context(profile=PROFILE, ephemeris=Ephemeris.BUILTIN) as lahiri:
            with self.assertRaises(TeistroError) as refused:
                lahiri.chart.found(instant=births[0], kp={}, **at)
            self.assertEqual(refused.exception.field, "frame.ayanamsha")
            taken = lahiri.chart.found(instant=births[0], kp={"anyAyanamsha": True}, **at).kp
            assert taken is not None
            self.assertEqual(len(taken.chart.cusps), 12)

    def test_a_chart_carries_its_essential_dignities(self) -> None:
        """The essential dignities cross whole, members resolved: the sect and
        every rule applied reported back, the seven in the Chaldean order with
        the score their flags give, a polar-night noon a night chart, a table
        of the caller's own obeyed, and a refusal named in the record
        (`03-design/essential-dignities.md`)."""
        from teistro import DignityRequest, Sect, SectRule, Terms, Triplicities

        kathmandu: dict[str, Any] = {
            "place": Observer(latitude_deg=Latitude(27.7172), longitude_deg=Longitude(85.324), altitude_m=Altitude(0)),
            "utc_offset_seconds": 20700,
        }
        instants = [2460676.5, 2460676.75]
        lilly = {
            "house": 5,
            "exaltation": 4,
            "triplicity": 3,
            "term": 2,
            "face": 1,
            "detriment": -5,
            "fall": -4,
            "peregrine": -5,
        }
        flags = ("house", "exaltation", "triplicity", "term", "face", "detriment", "fall")
        with self.teistro.context(profile="conformance-baseline", ephemeris=Ephemeris.BUILTIN) as ctx:
            self.assertIsNone(ctx.chart.found(instant=instants[0], **kathmandu).dignities)
            read = ctx.chart.found(instant=instants[0], dignities={}, **kathmandu).dignities
            assert read is not None
            self.assertIs(read.sect_rule, SectRule.HORIZON)
            self.assertEqual((read.rules.terms, read.rules.triplicities), (Terms.PTOLEMAIC_LILLY, Triplicities.LILLY))
            self.assertEqual(vars(read.scores), lilly)
            self.assertEqual(
                [at.planet for at in read.planets],
                [Graha.SATURN, Graha.JUPITER, Graha.MARS, Graha.SUN, Graha.VENUS, Graha.MERCURY, Graha.MOON],
            )
            for at in read.planets:
                held = [flag for flag in flags if getattr(at.dignity, flag)]
                self.assertEqual(at.peregrine, not any(flag in flags[:5] for flag in held), at.planet)
                score = sum(lilly[flag] for flag in held) + (lilly["peregrine"] if at.peregrine else 0)
                self.assertEqual(at.score, score, at.planet)
            # Each reception whole both ways, in the Chaldean order, and
            # scored only when mutual by house or by exaltation.
            order = [at.planet for at in read.planets]
            self.assertTrue(read.receptions)
            for one in read.receptions:
                first, second = one.planets
                self.assertLess(order.index(first), order.index(second))
                for side in (one.first_in, one.second_in):
                    self.assertTrue(any(getattr(side, flag) for flag in flags[:5]), one)
                self.assertEqual(
                    one.mutual, tuple(f for f in flags[:5] if getattr(one.first_in, f) and getattr(one.second_in, f))
                )
            for at in read.planets:

                def by(kind: str, planet: Graha = at.planet) -> bool:
                    return any(planet in one.planets and kind in one.mutual for one in read.receptions)

                points = (lilly["house"] if by("house") else 0) + (lilly["exaltation"] if by("exaltation") else 0)
                self.assertEqual(at.reception, points, at.planet)

            asked: DignityRequest = {
                "sectRule": SectRule.NIGHT,
                "rules": {"triplicities": Triplicities.PTOLEMY},
                "scores": {"peregrine": 0},
            }
            night = ctx.chart.found(instant=instants[0], dignities=asked, **kathmandu).dignities
            assert night is not None
            self.assertIs(night.sect, Sect.NIGHT)
            self.assertIs(night.rules.triplicities, Triplicities.PTOLEMY)
            self.assertEqual(vars(night.scores), {**lilly, "peregrine": 0})

            # 21 December 1988 at Tromsø: the Sun culminates under the horizon.
            tromso = Observer(latitude_deg=Latitude(69.6492), longitude_deg=Longitude(18.9553), altitude_m=Altitude(0))
            polar = ctx.chart.found(instant=2447516.9583333335, place=tromso, utc_offset_seconds=3600, dignities={})
            assert polar.dignities is not None
            self.assertIs(polar.dignities.sect, Sect.NIGHT)

            # A table of the caller's own: Aries' Egyptian terms in every sign,
            # the lords as members and as keys.
            row: list[tuple[Any, int]] = [
                (Graha.JUPITER, 6),
                ("VENUS", 12),
                ("graha.MERCURY", 20),
                (Graha.MARS, 25),
                (Graha.SATURN, 30),
            ]
            table = [[{"lord": lord, "end": end} for lord, end in row] for _ in range(12)]
            own = ctx.chart.found(instant=instants[0], dignities={"rules": {"terms": {"TABLE": table}}}, **kathmandu)
            assert own.dignities is not None
            self.assertIs(own.dignities.rules.terms, Terms.TABLE)
            lords = [Graha.JUPITER, Graha.VENUS, Graha.MERCURY, Graha.MARS, Graha.SATURN]
            for at in own.dignities.planets:
                degree = at.longitude_deg % 30
                lord = next(lord for lord, (_, end) in zip(lords, row) if degree < end)
                self.assertEqual(at.dignity.term, lord is at.planet, at.planet)

            batch = ctx.chart.found_many(instants=instants, dignities={}, **kathmandu)
            for k, instant in enumerate(instants):
                alone = ctx.chart.found(instant=instant, dignities={}, **kathmandu).dignities
                self.assertEqual(batch.at(k).dignities, alone)

            refusals: list[tuple[Any, str]] = [
                ({"sectRule": "DUSK"}, "dignities.sectRule"),
                ({"scores": {"peregrin": 0}}, "dignities.scores.peregrin"),
                ({"rules": {"terms": {"TABLE": table[1:]}}}, "dignities.rules.terms.TABLE"),
                ("LILLY", "dignities"),
            ]
            for bad, field in refusals:
                with self.assertRaises(TeistroError) as refused:
                    ctx.chart.found(instant=instants[0], dignities=bad, **kathmandu)
                self.assertEqual(refused.exception.field, field)

    def test_a_chart_carries_its_accidental_fortitudes(self) -> None:
        """The accidental fortitudes cross whole, members resolved: the sky
        and every rule and score applied reported back, the seven with their
        lines, each line's points and Lilly's net, the essential half the
        chart's own dignities, an answer's rules and scores handed back as a
        request, and a refusal named in the record
        (`03-design/essential-dignities.md` §Accidental fortitudes)."""
        from teistro import (
            Accident,
            AlmutenRules,
            FortitudeRequest,
            FortuneRule,
            HouseSystem,
            Partile,
            PlaceReading,
            Siege,
        )

        kathmandu: dict[str, Any] = {
            "place": Observer(latitude_deg=Latitude(27.7172), longitude_deg=Longitude(85.324), altitude_m=Altitude(0)),
            "utc_offset_seconds": 20700,
        }
        instants = [2460676.5, 2460676.75]
        solar = {Accident.CAZIMI, Accident.COMBUST, Accident.UNDER_BEAMS, Accident.FREE_FROM_COMBUSTION}
        with self.teistro.context(profile="conformance-baseline", ephemeris=Ephemeris.BUILTIN) as ctx:
            self.assertIsNone(ctx.chart.found(instant=instants[0], **kathmandu).fortitudes)
            chart = ctx.chart.found(instant=instants[0], fortitudes={}, **kathmandu)
            read = chart.fortitudes
            assert read is not None
            self.assertEqual(read.dignities, chart.dignities)
            self.assertIs(read.sky.houses, HouseSystem.REGIOMONTANUS)
            self.assertEqual((len(read.sky.cusps_deg), len(read.sky.speeds_deg_per_day)), (12, 7))
            rules = read.rules
            self.assertEqual(
                (rules.combustion_deg, rules.combustion_in_sign, rules.beams_deg, rules.cusp_orb_deg),
                (8.5, True, 17.0, 5.0),
            )
            self.assertIs(rules.partile, Partile.SAME_DEGREE)
            self.assertIs(rules.siege, Siege.SAME_SIGN)
            self.assertEqual(read.scores.houses, (5, 3, 1, 4, 3, -2, 4, -2, 2, 5, 4, -5))
            self.assertEqual(read.scores.regulus, 6)
            self.assertEqual([at.planet for at in read.planets], [at.planet for at in read.dignities.planets])
            for own, at in zip(read.dignities.planets, read.planets):
                points = [read.scores.houses[at.house - 1], *(line.points for line in at.accidents)]
                self.assertEqual(at.fortitude, sum(n for n in points if n > 0), at.planet)
                self.assertEqual(at.debility, -sum(n for n in points if n < 0), at.planet)
                self.assertEqual(at.net, own.score + own.reception + at.fortitude - at.debility, at.planet)
                held = [line.accident for line in at.accidents if line.accident in solar]
                self.assertEqual(len(held), 0 if at.planet is Graha.SUN else 1, at.planet)

            # The almutens: Lilly's of the figure is the greatest net, Fortune
            # the ascendant plus the Moon less the Sun, and every house has one.
            almutens = read.almutens
            self.assertEqual(almutens.rules, AlmutenRules())
            self.assertEqual([at.total for at in almutens.figure.totals], [at.net for at in read.planets])
            greatest = max(at.net for at in read.planets)
            self.assertEqual(almutens.figure.almutens, tuple(at.planet for at in read.planets if at.net == greatest))
            longitude = {at.planet: at.longitude_deg for at in read.dignities.planets}
            fortune = (read.sky.ascendant_deg + longitude[Graha.MOON] - longitude[Graha.SUN]) % 360
            self.assertAlmostEqual(almutens.fortune_deg, fortune, places=9)
            self.assertEqual(len(almutens.houses), 12)
            for almuten in (almutens.figure, almutens.places, *almutens.houses):
                self.assertTrue(almuten.almutens)
                self.assertFalse(set(almuten.partakers) & set(almuten.almutens))

            # The answer's rules and scores are a request as they stand, and
            # one changed is obeyed.
            fed_back = ctx.chart.found(
                instant=instants[0],
                fortitudes={"rules": read.rules, "scores": read.scores, "almuten": almutens.rules},
                **kathmandu,
            )
            self.assertEqual(fed_back.fortitudes, read)
            asked: FortitudeRequest = {
                "rules": {"beamsDeg": 15, "partile": {"WITHIN": {"orbDeg": 1}}, "siege": {"WITHIN": {"spanDeg": 30}}},
                "scores": {"regulus": 5},
                "almuten": AlmutenRules(place=PlaceReading.SIGN, fortune=FortuneRule.REVERSED_BY_NIGHT),
            }
            other = ctx.chart.found(instant=instants[0], fortitudes=asked, **kathmandu).fortitudes
            assert other is not None
            self.assertEqual(other.almutens.rules, AlmutenRules(PlaceReading.SIGN, FortuneRule.REVERSED_BY_NIGHT))
            self.assertEqual(other.rules.beams_deg, 15.0)
            self.assertEqual((other.rules.partile, other.rules.partile_orb_deg), (Partile.WITHIN, 1.0))
            self.assertEqual((other.rules.siege, other.rules.siege_span_deg), (Siege.WITHIN, 30.0))
            self.assertEqual(other.scores.regulus, 5)

            batch = ctx.chart.found_many(instants=instants, fortitudes={}, **kathmandu)
            for k, instant in enumerate(instants):
                alone = ctx.chart.found(instant=instant, fortitudes={}, **kathmandu).fortitudes
                self.assertEqual(batch.at(k).fortitudes, alone)

            refusals: list[tuple[dict[str, Any], str]] = [
                ({"fortitudes": {"rules": {"beamDeg": 15}}}, "fortitudes.rules.beamDeg"),
                ({"fortitudes": {"rules": {"beamsDeg": -1}}}, "fortitudes.rules.beamsDeg"),
                ({"fortitudes": {"almuten": {"place": "CUSP"}}}, "fortitudes.almuten.place"),
                ({"fortitudes": {}, "dignities": {}}, "dignities"),
            ]
            for bad, field in refusals:
                with self.assertRaises(TeistroError) as refused:
                    ctx.chart.found(instant=instants[0], **bad, **kathmandu)
                self.assertEqual(refused.exception.field, field)

    def test_a_chart_carries_its_considerations(self) -> None:
        """Lilly's considerations cross whole, members resolved: each clause
        read from the chart's own fortitudes, the Moon's two readings of her
        course, the rules read back and handed back as a request, a batch
        the charts one at a time, and a refusal named in the record
        (`03-design/hellenistic-considerations.md`)."""
        from teistro import ConsiderationRequest, ConsiderationRules, RadicalGround

        london: dict[str, Any] = {
            "place": Observer(latitude_deg=Latitude(51.5), longitude_deg=Longitude(-0.12), altitude_m=Altitude(0)),
            "utc_offset_seconds": 0,
        }
        instants = [2451545 + k / 8 for k in range(16)]
        with self.teistro.context(profile="conformance-baseline", ephemeris=Ephemeris.BUILTIN) as ctx:
            self.assertIsNone(ctx.chart.found(instant=instants[0], **london).considerations)
            voids = 0
            for instant in instants:
                chart = ctx.chart.found(instant=instant, considerations={}, fortitudes={}, **london)
                read = chart.considerations
                assert read is not None and chart.fortitudes is not None
                self.assertEqual(read.rules, ConsiderationRules(27.0, (10.0, 12.0, 7.5, 17.0, 8.0, 7.0, 12.5)))
                sky = chart.fortitudes.sky
                self.assertEqual(read.ascendant.sign.id, int(sky.ascendant_deg // 30))
                self.assertEqual(read.seventh.cusp_deg, sky.cusps_deg[6])
                self.assertEqual(read.ascendant.early, read.ascendant.degree < 3)
                course = read.moon.course
                if course.next is None:
                    self.assertIsNone(course.within_orb)
                if course.next is None or course.within_orb is None:
                    voids += 1
                self.assertEqual(
                    RadicalGround.ONE_LORD in read.radicality.grounds,
                    read.radicality.hour_lord is read.radicality.ascendant_lord,
                )
                # The answer's rules are a request as they stand.
                fed_back = ctx.chart.found(instant=instant, considerations=read.rules, fortitudes={}, **london)
                self.assertEqual(fed_back.considerations, read)
            self.assertGreater(voids, 0, "a void Moon in the batch")

            asked: ConsiderationRequest = {"moonLateFromDeg": 25, "orbsDeg": [9, 9, 7, 15, 7, 7, 12]}
            other = ctx.chart.found(instant=instants[0], considerations=asked, **london).considerations
            assert other is not None
            self.assertEqual(other.rules, ConsiderationRules(25.0, (9.0, 9.0, 7.0, 15.0, 7.0, 7.0, 12.0)))

            batch = ctx.chart.found_many(instants=instants, considerations={}, **london)
            for k, instant in enumerate(instants):
                self.assertEqual(
                    batch.at(k).considerations, ctx.chart.found(instant=instant, considerations={}, **london).considerations
                )
            refusals: list[tuple[dict[str, Any], str]] = [
                ({"moonLateFromDeg": 31}, "considerations.moonLateFromDeg"),
                ({"moonLate": 25}, "considerations.moonLate"),
            ]
            for request, field in refusals:
                with self.assertRaises(TeistroError) as caught:
                    ctx.chart.found(instant=instants[0], considerations=request, **london)  # type: ignore[arg-type]
                self.assertEqual(caught.exception.field, field)

    def test_a_chart_carries_its_perfection(self) -> None:
        """Lilly's perfection crosses whole, members resolved: the
        significators the asked house names, where each stands on the
        chart's own fortitudes, the ways held agreeing with the application
        they rest on, the rules read back and handed back, a batch the
        charts one at a time, and refusals named in the record
        (`03-design/hellenistic-perfection.md`)."""
        from teistro import ImpedimentKind, PerfectionRequest, PerfectionRules, Way

        london: dict[str, Any] = {
            "place": Observer(latitude_deg=Latitude(51.5), longitude_deg=Longitude(-0.12), altitude_m=Altitude(0)),
            "utc_offset_seconds": 0,
        }
        instants = [2451545 + 23 * k + k / 7 for k in range(12)]
        lilly = (10.0, 12.0, 7.5, 17.0, 8.0, 7.0, 12.5)
        with self.teistro.context(profile="conformance-baseline", ephemeris=Ephemeris.BUILTIN) as ctx:
            self.assertIsNone(ctx.chart.found(instant=instants[0], **london).perfection)
            applying = hindered = 0
            for instant in instants:
                chart = ctx.chart.found(instant=instant, perfection={"house": 7}, fortitudes={}, **london)
                read = chart.perfection
                assert read is not None and chart.fortitudes is not None
                self.assertEqual(read.rules, PerfectionRules(lilly, None, True))
                self.assertIsNot(read.querent, read.quesited)
                houses = {at.planet: at.house for at in chart.fortitudes.planets}
                self.assertEqual(read.ways.querent.house, houses[read.querent])
                self.assertEqual(read.ways.quesited.house, houses[read.quesited])
                if read.application is not None:
                    applying += 1
                    self.assertTrue(0 <= read.application.days <= read.horizon_days)
                    self.assertIn(read.application.applying, (read.querent, read.quesited))
                for way in (Way.CONJUNCTION, Way.SEXTILE_OR_TRINE, Way.SQUARE, Way.OPPOSITION):
                    if way in read.ways.held:
                        self.assertIsNotNone(read.application, way)
                for impediment in read.impediments:
                    hindered += 1
                    self.assertEqual(impediment.third is None, impediment.kind is ImpedimentKind.REFRANATION)
                for translation in read.translations:
                    self.assertEqual({translation.from_, translation.to}, {read.querent, read.quesited})
                # The answer's rules are a request as they stand.
                fed_back = ctx.chart.found(
                    instant=instant, perfection={"house": 7, "rules": read.rules}, fortitudes={}, **london
                )
                self.assertEqual(fed_back.perfection, read)
            self.assertTrue(applying > 0 and hindered > 0, "the sweep applies and is hindered")

            named: PerfectionRequest = {"querent": "VENUS", "quesited": "graha.MARS", "rules": {"horizonDays": 30}}
            other = ctx.chart.found(instant=instants[0], perfection=named, **london).perfection
            assert other is not None
            self.assertEqual(
                (other.querent, other.quesited, other.rules.horizon_days, other.horizon_days),
                (Graha.VENUS, Graha.MARS, 30.0, 30.0),
            )
            every = ctx.chart.found(
                instant=instants[0], perfection={**named, "rules": {"withinSign": False}}, **london
            ).perfection
            assert every is not None
            self.assertFalse(every.rules.within_sign)

            batch = ctx.chart.found_many(instants=instants, perfection={"house": 7}, **london)
            for k, instant in enumerate(instants):
                self.assertEqual(
                    batch.at(k).perfection, ctx.chart.found(instant=instant, perfection={"house": 7}, **london).perfection
                )
            refusals: list[tuple[dict[str, Any], str]] = [
                ({}, "perfection.quesited"),
                ({"house": 7, "quesited": "MARS"}, "perfection.house"),
                ({"house": 7, "rules": {"horizonDays": -1}}, "perfection.rules.horizonDays"),
            ]
            for request, field in refusals:
                with self.assertRaises(TeistroError) as caught:
                    ctx.chart.found(instant=instants[0], perfection=request, **london)  # type: ignore[arg-type]
                self.assertEqual(caught.exception.field, field)

    def test_a_chart_carries_its_progressions(self) -> None:
        """Progressions cross whole on Leo's own birth: his progressed map's
        sidereal time, his Appendix V contact on the day each year measure
        gives, the planets and the direction a row a graha, a batch the
        charts one at a time, and refusals named in the record
        (`03-design/western-progressions.md`)."""
        from teistro import Motion, NatalPoint, ProgressionContacts, ProgressionsRequest

        london: dict[str, Any] = {
            "place": Observer(latitude_deg=Latitude(51.5), longitude_deg=Longitude(0), altitude_m=Altitude(0)),
            "utc_offset_seconds": 0,
        }
        birth = 2400629.742361111
        with self.teistro.context(profile="western-tropical-default", ephemeris=Ephemeris.BUILTIN) as ctx:
            self.assertIsNone(ctx.chart.found(instant=birth, **london).progressions)

            # His forty-seventh year: the map at sidereal time 5h 54m 16s (p. 35).
            at = birth + 46 * 365.242189
            read = ctx.chart.found(instant=birth, progressions={"at": at}, **london).progressions
            assert read is not None and read.progressed is not None and read.directed is not None
            self.assertAlmostEqual(read.progressed.sky, birth + 46, delta=1e-9)
            self.assertAlmostEqual(read.progressed.armc_deg / 15, 5 + 54 / 60 + 16 / 3600, delta=2 / 3600)
            self.assertEqual(len(read.progressed.grahas), len(read.directed.planets))
            self.assertIsNone(read.contacts)
            sun = next(g for g in read.progressed.grahas if g.graha is Graha.SUN)
            directed_sun = next(g for g in read.directed.planets if g.graha is Graha.SUN)
            self.assertAlmostEqual(sun.longitude_deg, directed_sun.longitude_deg, delta=1e-9)

            # The Moon sesquiquadrate Mercury (p. 305): the 21st by a year, the 22nd by his rule.
            october: ProgressionContacts = {
                "from": 2417484.5,
                "to": 2417515.5,
                "grahas": ["MOON"],
                "points": [Graha.MERCURY],
                "aspects": [135],
            }
            for year, day in (("TROPICAL", 21), ("NOON_SIDEREAL_TIME", 22)):
                asked: ProgressionsRequest = {"year": year, "contacts": october}  # type: ignore[typeddict-item]
                found = ctx.chart.found(instant=birth, progressions=asked, **london).progressions
                assert found is not None and found.contacts is not None
                self.assertIsNone(found.progressed)
                (contact,) = found.contacts
                self.assertEqual(
                    (contact.graha, contact.to, contact.angle, contact.motion),
                    (Graha.MOON, NatalPoint("GRAHA", Graha.MERCURY), 135, Motion.DIRECT),
                )
                self.assertEqual(int(contact.life - 2417484.5) + 1, day, year)
            none = ctx.chart.found(
                instant=birth, progressions={"contacts": {**october, "aspects": [90]}}, **london
            ).progressions
            assert none is not None
            self.assertEqual(none.contacts, (), "a window asked holding none is empty, not None")

            instants = [birth, birth + 3000.25, birth + 9000.5]
            many: ProgressionsRequest = {"at": 2430000.5, "angles": "SOLAR_ARC_LONGITUDE", "direction": "NAIBOD"}
            batch = ctx.chart.found_many(instants=instants, progressions=many, **london)
            for k, instant in enumerate(instants):
                self.assertEqual(
                    batch.at(k).progressions, ctx.chart.found(instant=instant, progressions=many, **london).progressions
                )
            refusals: list[tuple[dict[str, Any], str]] = [
                ({}, "progressions.at"),
                ({"at": at, "year": "SIDEREAL"}, "progressions.year"),
                ({"at": at, "direction": {"PER_YEAR": 0}}, "progressions.direction"),
                ({"contacts": {"from": 2, "to": 1}}, "progressions.contacts.to"),
            ]
            for request, field in refusals:
                with self.assertRaises(TeistroError) as caught:
                    ctx.chart.found(instant=birth, progressions=request, **london)  # type: ignore[arg-type]
                self.assertEqual(caught.exception.field, field)

    def test_a_chart_carries_its_western_aspects(self) -> None:
        """The Western aspects cross whole on King Edward VII's nativity:
        Leo's four (*How to Judge a Nativity*, pp. 295–296) under his orbs,
        Lilly's moieties refusing the outer three they give no orb, a batch
        the charts one at a time, and refusals named in the record
        (`03-design/western-aspects.md`)."""
        from teistro import WesternAspect, WesternAspectRequest

        palace: dict[str, Any] = {
            "place": Observer(latitude_deg=Latitude(51.501), longitude_deg=Longitude(-0.142), altitude_m=Altitude(0)),
            "utc_offset_seconds": 0,
        }
        birth = 2393783.95
        with self.teistro.context(profile="western-tropical-default", ephemeris=Ephemeris.BUILTIN) as ctx:
            self.assertIsNone(ctx.chart.found(instant=birth, **palace).western_aspects)

            rows = ctx.chart.found(instant=birth, outer_planets=True, western_aspects={}, **palace).western_aspects
            assert rows is not None
            held = {(frozenset((row.first, row.second)), row.aspect) for row in rows}
            for a, aspect, b in (
                (Graha.SUN, WesternAspect.TRINE, Graha.URANUS),
                (Graha.SUN, WesternAspect.SEXTILE, Graha.MARS),
                (Graha.SUN, WesternAspect.SQUARE, Graha.NEPTUNE),
                (Graha.MOON, WesternAspect.SQUARE, Graha.SATURN),
            ):
                self.assertIn((frozenset((a, b)), aspect), held)
            self.assertTrue(all(row.from_exact_deg <= row.orb_deg for row in rows))

            # Lilly's moieties over the seven: the Moon (12½) and Saturn (10) square within 11¼.
            moieties = (("SUN", 17), (Graha.MOON, 12.5), ("MERCURY", 7), ("VENUS", 8), ("MARS", 7.5), ("JUPITER", 12), ("SATURN", 10))
            lilly: WesternAspectRequest = {
                "aspects": [WesternAspect.CONJUNCTION, "SEXTILE", "SQUARE", "TRINE", "OPPOSITION"],
                "orbs": {"model": "MOIETIES", "orbs": [{"graha": graha, "orbDeg": orb} for graha, orb in moieties]},
            }
            seven = ctx.chart.found(instant=birth, western_aspects=lilly, **palace).western_aspects
            assert seven is not None
            square = next(row for row in seven if (row.first, row.second) == (Graha.MOON, Graha.SATURN))
            self.assertEqual((square.aspect, square.orb_deg), (WesternAspect.SQUARE, 11.25))

            instants = [birth, birth + 3000.25, birth + 9000.5]
            two: WesternAspectRequest = {"aspects": ["TRINE", "SQUARE"]}
            batch = ctx.chart.found_many(instants=instants, western_aspects=two, **palace)
            for k, instant in enumerate(instants):
                self.assertEqual(
                    batch.at(k).western_aspects,
                    ctx.chart.found(instant=instant, western_aspects=two, **palace).western_aspects,
                )
            refusals: list[tuple[Any, str, bool]] = [
                ({"aspects": []}, "westernAspects.aspects", False),
                ({"aspects": ["TRINE", "TRINE"]}, "westernAspects.aspects", False),
                ({"aspects": ["QUINTILE"]}, "westernAspects.aspects[0]", False),
                (lilly, "westernAspects.orbs.orbs", True),
                ([], "westernAspects", False),
            ]
            for request, field, outer in refusals:
                with self.assertRaises(TeistroError) as caught:
                    ctx.chart.found(instant=birth, outer_planets=outer, western_aspects=request, **palace)
                self.assertEqual(caught.exception.field, field)

    def test_a_chart_carries_its_declinations_and_parallels(self) -> None:
        """The declinations and parallels cross whole on King George V (Leo,
        *How to Judge a Nativity*, p. 130): the recast's declinations, his
        four parallels, a batch the charts one at a time, and refusals named
        in the record (`03-design/western-declinations.md`)."""
        from teistro import ParallelRequest

        george: dict[str, Any] = {
            "place": Observer(latitude_deg=Latitude(51.5045), longitude_deg=Longitude(-0.1366), altitude_m=Altitude(0)),
            "utc_offset_seconds": 0,
        }
        birth = 2402390.554166667
        with self.teistro.context(profile="western-tropical-default", ephemeris=Ephemeris.BUILTIN) as ctx:
            bare = ctx.chart.found(instant=birth, **george)
            self.assertIsNone(bare.declinations)
            self.assertIsNone(bare.parallels)

            chart = ctx.chart.found(instant=birth, outer_planets=True, parallels={}, **george)
            read = chart.declinations
            assert read is not None
            self.assertEqual(len(read.grahas), 10, "the nodes are not read")
            self.assertAlmostEqual(read.graha(Graha.SUN) or 0.0, 22.2997, delta=0.01)
            self.assertAlmostEqual(read.lagna_deg, 0.8366, delta=0.01)
            self.assertAlmostEqual(read.midheaven_deg, -23.452, delta=0.01)
            rows = chart.parallels
            assert rows is not None
            self.assertEqual(
                [(row.first, row.second, row.contrary) for row in rows],
                [
                    (Graha.MOON, Graha.NEPTUNE, True),
                    (Graha.SUN, Graha.JUPITER, True),
                    (Graha.JUPITER, Graha.URANUS, True),
                    (Graha.MERCURY, Graha.VENUS, False),
                ],
            )

            instants = [birth, birth - 3000.25]
            asked: ParallelRequest = {"orbDeg": 1.5}
            batch = ctx.chart.found_many(instants=instants, parallels=asked, **george)
            for k, instant in enumerate(instants):
                one = ctx.chart.found(instant=instant, parallels=asked, **george)
                self.assertEqual(batch.at(k).parallels, one.parallels)
                self.assertEqual(batch.at(k).declinations, one.declinations)
            refusals: list[tuple[Any, str]] = [
                ({"orbDeg": 0}, "parallels.orbDeg"),
                ({"orbDeg": 11}, "parallels.orbDeg"),
                ({"orb": 1}, "parallels.orb"),
                ([], "parallels"),
            ]
            for request, field in refusals:
                with self.assertRaises(TeistroError) as caught:
                    ctx.chart.found(instant=birth, parallels=request, **george)
                self.assertEqual(caught.exception.field, field)

    def test_a_chart_carries_its_equal_distances(self) -> None:
        """The equal distances cross whole on King George V (Leo, *How to
        Judge a Nativity*, p. 130): his recast's one under the default,
        Pluto on the far point of the Moon and Jupiter, eight at 1.5°, a
        batch the charts one at a time, and refusals named in the record
        (`03-design/western-midpoints.md`)."""
        from teistro import MidpointRequest

        george: dict[str, Any] = {
            "place": Observer(latitude_deg=Latitude(51.5045), longitude_deg=Longitude(-0.1366), altitude_m=Altitude(0)),
            "utc_offset_seconds": 0,
        }
        birth = 2402390.554166667
        with self.teistro.context(profile="western-tropical-default", ephemeris=Ephemeris.BUILTIN) as ctx:
            self.assertIsNone(ctx.chart.found(instant=birth, **george).midpoints)
            rows = ctx.chart.found(instant=birth, outer_planets=True, midpoints={}, **george).midpoints
            assert rows is not None
            self.assertEqual(len(rows), 1)
            row = rows[0]
            self.assertEqual((row.first, row.second, row.middle, row.far), (Graha.MOON, Graha.JUPITER, Graha.PLUTO, True))
            self.assertAlmostEqual(row.from_axis_deg, 0.052, delta=0.01)
            self.assertAlmostEqual(row.distance_deg, 137.69, delta=0.01)
            self.assertEqual(row.orb_deg, 0.5)

            wide: MidpointRequest = {"orbDeg": 1.5}
            eight = ctx.chart.found(instant=birth, outer_planets=True, midpoints=wide, **george).midpoints
            assert eight is not None
            self.assertEqual(len(eight), 8)
            self.assertEqual([at.from_axis_deg for at in eight], sorted(at.from_axis_deg for at in eight))
            instants = [birth, birth - 3000.25]
            batch = ctx.chart.found_many(instants=instants, midpoints=wide, **george)
            for k, instant in enumerate(instants):
                self.assertEqual(batch.at(k).midpoints, ctx.chart.found(instant=instant, midpoints=wide, **george).midpoints)
            refusals: list[tuple[Any, str]] = [
                ({"orbDeg": 11}, "midpoints.orbDeg"),
                ({"orb": 1}, "midpoints.orb"),
                ([], "midpoints"),
            ]
            for request, field in refusals:
                with self.assertRaises(TeistroError) as caught:
                    ctx.chart.found(instant=birth, midpoints=request, **george)
                self.assertEqual(caught.exception.field, field)

    def test_a_chart_carries_its_antiscia(self) -> None:
        """The antiscia cross whole on King George V (Leo, *How to Judge a
        Nativity*, p. 130): his recast's one pair under Lilly's moieties,
        the outer three unpaired, Leo's orbs pairing them, a batch the
        charts one at a time, and refusals named in the record
        (`03-design/western-antiscia.md`)."""
        from teistro import AntisciaRequest, HouseSystem

        george: dict[str, Any] = {
            "place": Observer(latitude_deg=Latitude(51.5045), longitude_deg=Longitude(-0.1366), altitude_m=Altitude(0)),
            "utc_offset_seconds": 0,
        }
        birth = 2402390.554166667
        with self.teistro.context(profile="western-tropical-default", ephemeris=Ephemeris.BUILTIN) as ctx:
            self.assertIsNone(ctx.chart.found(instant=birth, **george).antiscia)
            read = ctx.chart.found(instant=birth, outer_planets=True, antiscia={}, **george).antiscia
            assert read is not None
            sun = next(at for at in read.points if at.graha == Graha.SUN)
            self.assertAlmostEqual(sun.antiscion_deg, 107.5685, delta=0.01)
            self.assertAlmostEqual((sun.contrantiscion_deg - sun.antiscion_deg) % 360.0, 180.0, delta=1e-9)
            self.assertEqual(len(read.pairs), 1)
            pair = read.pairs[0]
            self.assertEqual((pair.first, pair.second, pair.contrary), (Graha.MARS, Graha.MERCURY, False))
            self.assertAlmostEqual(pair.apart_deg, 5.935, delta=0.02)
            self.assertEqual(read.unpaired, (Graha.URANUS, Graha.NEPTUNE, Graha.PLUTO))
            self.assertEqual((read.on_cusps, read.cusp_system), ((), None), "no cusps unless asked")

            # On the cusps, Lilly's Regiomontanus unless named: his Uranus
            # reflects 0.63° past the fourth cusp, into the next degree.
            on = ctx.chart.found(instant=birth, outer_planets=True, antiscia={"cusps": {}}, **george).antiscia
            assert on is not None
            self.assertEqual((on.on_cusps, on.cusp_system), ((), HouseSystem.REGIOMONTANUS))
            placidus: AntisciaRequest = {"cusps": {"system": HouseSystem.PLACIDUS}}
            named = ctx.chart.found(instant=birth, antiscia=placidus, **george).antiscia
            assert named is not None
            self.assertEqual(named.cusp_system, HouseSystem.PLACIDUS)

            leo: AntisciaRequest = {"orbs": {"model": "LEO"}}
            wide = ctx.chart.found(instant=birth, outer_planets=True, antiscia=leo, **george).antiscia
            assert wide is not None
            self.assertEqual(wide.unpaired, ())
            instants = [birth, birth - 3000.25]
            batch = ctx.chart.found_many(instants=instants, antiscia=leo, **george)
            for k, instant in enumerate(instants):
                self.assertEqual(batch.at(k).antiscia, ctx.chart.found(instant=instant, antiscia=leo, **george).antiscia)
            refusals: list[tuple[Any, str]] = [
                ({"orbs": {"model": "BY_ASPECT", "orbs": [{"aspect": "TRINE", "orbDeg": 3}]}}, "antiscia.orbs.orbs"),
                ({"orb": 1}, "antiscia.orb"),
                ({"cusps": {"system": "NOWHERE"}}, "antiscia.cusps.system"),
                ([], "antiscia"),
            ]
            for request, field in refusals:
                with self.assertRaises(TeistroError) as caught:
                    ctx.chart.found(instant=birth, antiscia=request, **george)
                self.assertEqual(caught.exception.field, field)

    def test_a_chart_carries_its_western_houses(self) -> None:
        """Leo's own illustration (*How to Judge a Nativity*, p. 150), "a
        female born at 2.42 A.M. 13th December, 1835, London", against the
        SDK test's Moshier recast: Placidus, Saturn rising; another
        division by name, a batch the charts one at a time, and a refusal
        named by its field (`03-design/western-houses.md`)."""
        from teistro import HouseSystem, WesternHouseRequest, WesternHouses

        london: dict[str, Any] = {
            "place": Observer(latitude_deg=Latitude(51.5), longitude_deg=Longitude(-0.1), altitude_m=Altitude(0)),
            "utc_offset_seconds": 0,
        }
        birth = 2391625.6125

        def near(a: float, b: float) -> bool:
            return abs((a - b + 540.0) % 360.0 - 180.0) < 0.01

        with self.teistro.context(profile="western-tropical-default", ephemeris=Ephemeris.BUILTIN) as ctx:
            self.assertIsNone(ctx.chart.found(instant=birth, **london).western_houses)
            houses = ctx.chart.found(instant=birth, outer_planets=True, western_houses={}, **london).western_houses
            assert isinstance(houses, WesternHouses)
            self.assertEqual(houses.system, HouseSystem.PLACIDUS)
            self.assertEqual(len(houses.cusps_deg), 12)
            self.assertTrue(near(houses.cusps_deg[0], 202.1436) and near(houses.cusps_deg[9], 119.3147), houses.cusps_deg)
            self.assertTrue(near(houses.reach_deg, 191.6089), houses.reach_deg)
            placed = {one.graha: (one.house, one.with_ascendant) for one in houses.planets}
            self.assertEqual(placed[Graha.SATURN], (1, True))
            self.assertEqual(placed[Graha.SUN], (2, False))
            self.assertEqual(placed[Graha.MARS][0], 3)

            for system in (HouseSystem.KOCH, "house_system.KOCH", "KOCH"):
                koch: WesternHouseRequest = {"system": system}
                named = ctx.chart.found(instant=birth, western_houses=koch, **london).western_houses
                assert named is not None
                self.assertEqual(named.system, HouseSystem.KOCH)
            instants = [birth, birth + 100.5]
            batch = ctx.chart.found_many(instants=instants, western_houses={}, **london)
            for k, instant in enumerate(instants):
                self.assertEqual(
                    batch.at(k).western_houses, ctx.chart.found(instant=instant, western_houses={}, **london).western_houses
                )
            refusals: list[tuple[Any, str]] = [
                ({"system": "NOWHERE"}, "westernHouses.system"),
                ({"sistem": "KOCH"}, "westernHouses.sistem"),
                ([], "western_houses"),
            ]
            for request, field in refusals:
                with self.assertRaises(TeistroError) as caught:
                    ctx.chart.found(instant=birth, western_houses=request, **london)
                self.assertEqual(caught.exception.field, field)

    def test_a_chart_carries_its_match(self) -> None:
        """A birth matched with itself: one sign and one nakshatra, so every
        koota but Nadi takes its whole points and the shared nadi none, 28,
        whatever the Moon (*Muhurta Chintamani* VI.21–34); a batch the
        charts one at a time, the sides swapping Varna's reading, and
        refusals named by field (`03-design/matching.md`)."""
        from teistro import (
            AshtaKoota,
            BhakootKoota,
            DhinamPorutham,
            GanaKoota,
            Koota,
            Kuja,
            MaitriKoota,
            MaitriRelation,
            MatchingRequest,
            NadiKoota,
            Porutham,
            RajjuPorutham,
            SynastryPartner,
            TaraKoota,
            VarnaKoota,
            VashyaKoota,
            VashyaRelation,
        )

        kathmandu: dict[str, Any] = {
            "place": Observer(latitude_deg=Latitude(27.7172), longitude_deg=Longitude(85.324), altitude_m=Altitude(1400)),
            "utc_offset_seconds": 20700,
        }
        birth = 2451545.0
        with self.teistro.context(ephemeris=Ephemeris.BUILTIN) as ctx:
            self.assertIsNone(ctx.chart.found(instant=birth, **kathmandu).matching)
            partner: SynastryPartner = {"instant": birth, "observer": kathmandu["place"], "utc_offset_seconds": 20700}
            itself: MatchingRequest = {"partner": partner, "partnerRole": "BRIDE"}
            matched = ctx.chart.found(instant=birth, matching=itself, **kathmandu).matching
            assert isinstance(matched, AshtaKoota)
            self.assertEqual(matched.total, 28)
            self.assertEqual(
                [row.reading.koota for row in matched.kootas],
                [Koota.VARNA, Koota.VASHYA, Koota.TARA, Koota.YONI, Koota.GRAHA_MAITRI, Koota.GANA, Koota.BHAKOOT, Koota.NADI],
            )
            self.assertEqual([row.max_points for row in matched.kootas], [1, 2, 3, 4, 5, 6, 7, 8])
            vashya, tara, maitri = (matched.kootas[n].reading for n in (1, 2, 4))
            assert isinstance(vashya, VashyaKoota) and isinstance(tara, TaraKoota)
            self.assertEqual((vashya.relation, tara.bride_to_groom, tara.groom_to_bride), (VashyaRelation.MUTUAL, 1, 1))
            self.assertEqual(getattr(maitri, "relation"), MaitriRelation.ONE_LORD)
            bhakoot, nadi = matched.kootas[6].reading, matched.kootas[7].reading
            assert isinstance(bhakoot, BhakootKoota) and isinstance(nadi, NadiKoota)
            self.assertEqual((bhakoot.apart, bhakoot.dosha, bhakoot.lifted, bhakoot.exceptions.one_lord), (1, None, False, True))
            self.assertTrue(nadi.dosha and nadi.bride == nadi.groom)
            # One star in one pada is the nadi dosha VI.36 does not lift; one
            # gana and one lord leave nothing to lift.
            gana = matched.kootas[5].reading
            assert isinstance(gana, GanaKoota) and isinstance(maitri, MaitriKoota)
            self.assertEqual((nadi.lifted, gana.dosha, gana.lifted, maitri.lifted), (False, False, False, False))

            # The ten considerations ride on the same request: one star in
            # one sign shares its Rajju, which the one lord lifts.
            ten = ctx.chart.found(instant=birth, matching=itself, **kathmandu).porutham
            assert isinstance(ten, Porutham)
            self.assertEqual(
                [row.reading.koota for row in ten.considerations],
                [
                    Koota.TARA,
                    Koota.GANA,
                    Koota.MAHENDRA,
                    Koota.STREE_DEERGHA,
                    Koota.YONI,
                    Koota.BHAKOOT,
                    Koota.GRAHA_MAITRI,
                    Koota.VASHYA,
                    Koota.RAJJU,
                    Koota.VEDHA,
                ],
            )
            dhinam, rajju = ten.considerations[0], ten.considerations[8]
            assert isinstance(dhinam.reading, DhinamPorutham) and isinstance(rajju.reading, RajjuPorutham)
            self.assertEqual(dhinam.reading.count, 1)
            self.assertTrue(dhinam.reading.rule.key.startswith("COMMON_"))
            self.assertEqual((rajju.reading.bride == rajju.reading.groom, rajju.agrees, rajju.lifted), (True, True, True))
            self.assertEqual((ten.exception.one_lord, ten.exception.opposite), (True, False))
            self.assertEqual(ten.agreeing, sum(row.agrees for row in ten.considerations))
            self.assertIsNone(ctx.chart.found(instant=birth, **kathmandu).porutham)

            # The Kuja dosha rides on it too (Manasagari): one birth on both
            # sides reads Mars alike, so both carry it or neither does.
            everywhere: MatchingRequest = {**itself, "kuja": {"from": "LAGNA_MOON_VENUS"}}
            mars = ctx.chart.found(instant=birth, matching=everywhere, **kathmandu).kuja
            assert isinstance(mars, Kuja)
            self.assertEqual(mars.bride, mars.groom)
            self.assertEqual([r.reference for r in mars.bride.readings], ["LAGNA", "MOON", "VENUS"])
            for reading in mars.bride.readings:
                self.assertEqual(reading.in_houses, reading.house in (1, 4, 7, 8, 12))
            self.assertEqual(mars.bride.dosha, any(r.in_houses for r in mars.bride.readings))
            self.assertEqual(mars.both, mars.bride.dosha)
            self.assertIsNone(ctx.chart.found(instant=birth, **kathmandu).kuja)

            asked: MatchingRequest = {
                "partner": {"instant": 2447892.5, "observer": kathmandu["place"], "utc_offset_seconds": 20700},
                "partnerRole": "GROOM",
                "rules": {"nadiDosha": "MIDDLE_ONLY"},
            }
            instants = [birth, birth + 9.5, birth + 17.25]
            batch = ctx.chart.found_many(instants=instants, matching=asked, **kathmandu)
            for k, instant in enumerate(instants):
                one = ctx.chart.found(instant=instant, matching=asked, **kathmandu)
                alone = one.matching
                self.assertEqual(batch.at(k).matching, alone)
                self.assertEqual(batch.at(k).porutham, one.porutham)
                self.assertEqual(batch.at(k).kuja, one.kuja)
                swapped = ctx.chart.found(instant=instant, matching={**asked, "partnerRole": "BRIDE"}, **kathmandu).matching
                assert alone is not None and swapped is not None
                ours, theirs = alone.kootas[0].reading, swapped.kootas[0].reading
                assert isinstance(ours, VarnaKoota) and isinstance(theirs, VarnaKoota)
                self.assertEqual((theirs.bride, theirs.groom), (ours.groom, ours.bride))
            refusals: list[tuple[Any, str]] = [
                ({**asked, "partnerRole": "UNCLE"}, "matching.partnerRole"),
                ({**asked, "rules": {"nadi": "ANY"}}, "matching.rules.nadi"),
                ({**asked, "porutham": {"deergha": "SEVENTH"}}, "matching.porutham.deergha"),
                ({**asked, "kuja": {"house": "WITH_SECOND"}}, "matching.kuja.house"),
                ({"partner": asked["partner"]}, "matching"),
            ]
            for request, field in refusals:
                with self.assertRaises(TeistroError) as caught:
                    ctx.chart.found(instant=birth, matching=request, **kathmandu)
                self.assertEqual(caught.exception.field, field)
        with self.teistro.context(profile="western-tropical-default", ephemeris=Ephemeris.BUILTIN) as western:
            with self.assertRaises(TeistroError) as caught:
                western.chart.found(instant=birth, matching=asked, **kathmandu)
            self.assertEqual(caught.exception.field, "matching.partner")

    def test_a_chart_carries_its_harmonic(self) -> None:
        """Churchill's 9th harmonic as Addey reads it (*Harmonics in
        Astrology*, pp. 97–98): the Moon on Saturn in the third, Venus
        rising, Pluto in the tenth; a batch the charts one at a time, and
        refusals named by field (`03-design/western-harmonics.md`)."""
        from teistro import HarmonicChart, HarmonicPoint, HarmonicRequest

        blenheim: dict[str, Any] = {
            "place": Observer(latitude_deg=Latitude(51.8414), longitude_deg=Longitude(-1.3611), altitude_m=Altitude(0)),
            "utc_offset_seconds": 0,
        }
        birth = 2405857.564892
        with self.teistro.context(profile="western-tropical-default", ephemeris=Ephemeris.BUILTIN) as ctx:
            self.assertIsNone(ctx.chart.found(instant=birth, **blenheim).harmonic)
            ninth = ctx.chart.found(instant=birth, outer_planets=True, harmonic={"number": 9}, **blenheim).harmonic
            assert isinstance(ninth, HarmonicChart)
            self.assertEqual((ninth.harmonic, len(ninth.points)), (9, 12))
            house = {one.point: one.house for one in ninth.points}

            def planet(graha: Graha) -> HarmonicPoint:
                return HarmonicPoint("GRAHA", graha)

            self.assertEqual(
                [house[planet(g)] for g in (Graha.MOON, Graha.SATURN, Graha.VENUS, Graha.PLUTO)], [3, 3, 1, 10]
            )
            self.assertEqual(house[HarmonicPoint("ASCENDANT")], 1)
            self.assertEqual(ninth.points[-1].point, HarmonicPoint("MIDHEAVEN"))
            row = next(one for one in ninth.rows if (one.first, one.second) == (planet(Graha.MOON), planet(Graha.SATURN)))
            self.assertLess(row.apart_deg, 0.6)
            self.assertEqual((row.multiple, row.orb_deg), (4, 12.0))

            fifth: HarmonicRequest = {"number": 5, "orbDeg": 3}
            instants = [birth, birth + 100.5]
            batch = ctx.chart.found_many(instants=instants, harmonic=fifth, **blenheim)
            for k, instant in enumerate(instants):
                self.assertEqual(batch.at(k).harmonic, ctx.chart.found(instant=instant, harmonic=fifth, **blenheim).harmonic)
            refusals: list[tuple[Any, str]] = [
                ({"number": 0}, "harmonic.number"),
                ({"number": 9, "orbDeg": 31}, "harmonic.orbDeg"),
                ({}, "harmonic"),
                ([], "harmonic"),
            ]
            for request, field in refusals:
                with self.assertRaises(TeistroError) as caught:
                    ctx.chart.found(instant=birth, harmonic=request, **blenheim)
                self.assertEqual(caught.exception.field, field)

    def test_a_chart_carries_its_synastry_with_a_partner(self) -> None:
        """A synastry crosses whole on King George V and Queen Mary (Leo,
        *How to Judge a Nativity*, p. 130): the recast's closest contacts,
        the lagna left out on request, a batch the charts one at a time,
        and refusals named in the record (`03-design/western-synastry.md`)."""
        from teistro import NatalPoint, SynastryPartner, SynastryRequest, WesternAspect

        george: dict[str, Any] = {
            "place": Observer(latitude_deg=Latitude(51.5045), longitude_deg=Longitude(-0.1366), altitude_m=Altitude(0)),
            "utc_offset_seconds": 0,
        }
        birth = 2402390.554166667
        mary: SynastryPartner = {
            "instant": 2403113.499305556,
            "observer": Observer(latitude_deg=Latitude(51.5058), longitude_deg=Longitude(-0.1878), altitude_m=Altitude(0)),
        }
        with self.teistro.context(profile="western-tropical-default", ephemeris=Ephemeris.BUILTIN) as ctx:
            self.assertIsNone(ctx.chart.found(instant=birth, **george).synastry)

            rows = ctx.chart.found(instant=birth, outer_planets=True, synastry={"partner": mary}, **george).synastry
            assert rows is not None
            mars = NatalPoint("GRAHA", Graha.MARS)
            for first, aspect, second, from_exact_deg in (
                (mars, WesternAspect.OPPOSITION, NatalPoint("LAGNA"), 0.32),
                (mars, WesternAspect.SEXTILE, NatalPoint("GRAHA", Graha.SUN), 0.39),
                (NatalPoint("GRAHA", Graha.PLUTO), WesternAspect.CONJUNCTION, NatalPoint("GRAHA", Graha.PLUTO), 1.69),
            ):
                row = next(row for row in rows if (row.first, row.aspect, row.second) == (first, aspect, second))
                self.assertAlmostEqual(row.from_exact_deg, from_exact_deg, delta=0.01)
            self.assertTrue(all(row.from_exact_deg <= row.orb_deg for row in rows))
            self.assertEqual([row.from_exact_deg for row in rows], sorted(row.from_exact_deg for row in rows))

            without = ctx.chart.found(instant=birth, synastry={"partner": mary, "lagna": False}, **george).synastry
            assert without is not None
            self.assertTrue(all(row.first.graha is not None and row.second.graha is not None for row in without))

            instants = [birth, birth - 3000.25]
            asked: SynastryRequest = {"partner": mary, "aspects": [WesternAspect.SEXTILE, "OPPOSITION"]}
            batch = ctx.chart.found_many(instants=instants, synastry=asked, **george)
            for k, instant in enumerate(instants):
                self.assertEqual(
                    batch.at(k).synastry,
                    ctx.chart.found(instant=instant, synastry=asked, **george).synastry,
                )

            # The parallels across: none unless asked, then the recast's
            # closest (Uranus with Uranus, 0.05°) and a contrary pair.
            self.assertIsNone(ctx.chart.found(instant=birth, synastry={"partner": mary}, **george).synastry_parallels)
            level: SynastryRequest = {"partner": mary, "parallels": {}}
            parallels = ctx.chart.found(instant=birth, outer_planets=True, synastry=level, **george).synastry_parallels
            assert parallels is not None
            uranus = NatalPoint("GRAHA", Graha.URANUS)
            self.assertEqual((parallels[0].first, parallels[0].second), (uranus, uranus))
            self.assertAlmostEqual(parallels[0].apart_deg, 0.049, delta=0.005)
            self.assertTrue(any(row.contrary and row.apart_deg < 0.95 for row in parallels))
            self.assertTrue(all(row.apart_deg <= row.orb_deg == 1 for row in parallels))
            self.assertEqual([row.apart_deg for row in parallels], sorted(row.apart_deg for row in parallels))
            narrow: SynastryRequest = {"partner": mary, "parallels": {"orbDeg": 0.000001}}
            self.assertEqual(ctx.chart.found(instant=birth, synastry=narrow, **george).synastry_parallels, ())

            # The antiscia across: none unless asked, then the recast's
            # seven under Lilly's moieties, closest first (Saturn's
            # antiscion on Jupiter, 0.09°).
            self.assertIsNone(ctx.chart.found(instant=birth, synastry={"partner": mary}, **george).synastry_antiscia)
            mirrored: SynastryRequest = {"partner": mary, "antiscia": {}}
            reflected = ctx.chart.found(instant=birth, synastry=mirrored, **george).synastry_antiscia
            assert reflected is not None
            self.assertEqual(
                [(row.first, row.second, row.contrary) for row in reflected],
                [
                    (Graha.SATURN, Graha.JUPITER, False),
                    (Graha.SATURN, Graha.MOON, False),
                    (Graha.MERCURY, Graha.MARS, False),
                    (Graha.MARS, Graha.SATURN, True),
                    (Graha.VENUS, Graha.MARS, False),
                    (Graha.MARS, Graha.MERCURY, False),
                    (Graha.MARS, Graha.SUN, False),
                ],
            )
            self.assertAlmostEqual(reflected[0].apart_deg, 0.089, delta=0.005)
            refusals: list[tuple[Any, str]] = [
                ({"partner": {**mary, "born": "London"}}, "synastry.partner.born"),
                ({"partner": mary, "zodiac": "SIDEREAL"}, "synastry.zodiac"),
                ({"partner": mary, "orbs": {"model": "MOIETIES", "orbs": [{"graha": Graha.SUN, "orbDeg": 17}]}}, "synastry.lagna"),
                ({"lagna": False}, "synastry.partner"),
                ({"partner": mary, "parallels": {"orbDeg": 11}}, "synastry.parallels.orbDeg"),
                (
                    {"partner": mary, "antiscia": {"orbs": {"model": "BY_ASPECT", "orbs": [{"aspect": WesternAspect.TRINE, "orbDeg": 3}]}}},
                    "synastry.antiscia.orbs.orbs",
                ),
                ([], "synastry"),
            ]
            for request, field in refusals:
                with self.assertRaises(TeistroError) as caught:
                    ctx.chart.found(instant=birth, synastry=request, **george)
                self.assertEqual(caught.exception.field, field)

    def test_a_synastry_reads_the_equal_distances_and_makes_the_composite_and_davison(self) -> None:
        """The composite and the Davison birth cross on King George V and
        Queen Mary against the SDK test's Moshier recast; the Davison birth
        founds a chart as a birth does (`03-design/western-composites.md`)."""
        from teistro import Composite, DavisonBirth, SynastryPartner, SynastryRequest

        george: dict[str, Any] = {
            "place": Observer(latitude_deg=Latitude(51.5045), longitude_deg=Longitude(-0.1366), altitude_m=Altitude(0)),
            "utc_offset_seconds": 0,
        }
        birth = 2402390.554166667
        mary: SynastryPartner = {
            "instant": 2403113.499305556,
            "observer": Observer(latitude_deg=Latitude(51.5058), longitude_deg=Longitude(-0.1878), altitude_m=Altitude(0)),
        }

        def near(a: float, b: float) -> bool:
            return abs((a - b + 540.0) % 360.0 - 180.0) < 0.01

        with self.teistro.context(profile="western-tropical-default", ephemeris=Ephemeris.BUILTIN) as ctx:
            plain = ctx.chart.found(instant=birth, outer_planets=True, synastry={"partner": mary}, **george)
            self.assertIsNone(plain.synastry_composite)
            self.assertIsNone(plain.synastry_davison)

            asked: SynastryRequest = {"partner": mary, "composite": True, "davison": True}
            chart = ctx.chart.found(instant=birth, outer_planets=True, synastry=asked, **george)
            composite = chart.synastry_composite
            assert isinstance(composite, Composite)
            self.assertEqual(len(composite.planets), 10)
            for graha, longitude in [(Graha.SUN, 68.8193), (Graha.MOON, 259.7289), (Graha.MARS, 130.5171), (Graha.PLUTO, 44.2555)]:
                at = next(one for one in composite.planets if one.graha == graha)
                self.assertTrue(near(at.longitude_deg, longitude), f"{graha} {at.longitude_deg}")
            self.assertTrue(near(composite.midheaven_deg, 258.1797), composite.midheaven_deg)
            self.assertTrue(near(composite.lagna_deg, 334.007), composite.lagna_deg)
            self.assertFalse(composite.lagna_turned)
            # Its Placidus cusps, the near midpoints of the two charts', the
            # first and tenth its lagna and midheaven.
            assert composite.cusps_deg is not None
            self.assertEqual(len(composite.cusps_deg), 12)
            self.assertTrue(near(composite.cusps_deg[2], 57.8601), composite.cusps_deg[2])
            self.assertEqual((composite.cusps_deg[0], composite.cusps_deg[9]), (composite.lagna_deg, composite.midheaven_deg))
            # Cusps are one chart's, so a synastry's antiscia refuses them.
            with self.assertRaises(TeistroError) as caught:
                ctx.chart.found(instant=birth, synastry={"partner": mary, "antiscia": {"cusps": {}}}, **george)
            self.assertEqual(caught.exception.field, "synastry.antiscia.cusps")

            davison = chart.synastry_davison
            assert isinstance(davison, DavisonBirth)
            self.assertAlmostEqual(davison.instant, 2402752.026736111, delta=1e-8)
            self.assertAlmostEqual(float(davison.place.longitude_deg), -0.1622, delta=1e-9)
            self.assertEqual(davison.utc_offset_seconds, 0)
            between = ctx.chart.found(
                instant=davison.instant, place=davison.place, utc_offset_seconds=davison.utc_offset_seconds
            )
            mars = next(one for one in between.grahas if one.graha == Graha.MARS)
            self.assertTrue(near(mars.tropical_deg, 20.1267), mars.tropical_deg)

            # The equal distances across: none unless asked, then the SDK
            # test's recast within 1°, her Venus on his Sun and Neptune the
            # closest.
            self.assertIsNone(plain.synastry_midpoints)
            level: SynastryRequest = {"partner": mary, "midpoints": {"orbDeg": 1}}
            equal = ctx.chart.found(instant=birth, outer_planets=True, synastry=level, **george).synastry_midpoints
            assert equal is not None
            self.assertEqual(len(equal), 10)
            self.assertEqual(
                (equal[0].first, equal[0].second, equal[0].middle, equal[0].partners_pair, equal[0].far),
                (Graha.SUN, Graha.NEPTUNE, Graha.VENUS, True, False),
            )
            self.assertAlmostEqual(equal[0].from_axis_deg, 0.14, delta=0.01)

            refusals: list[tuple[Any, str]] = [
                ({"partner": mary, "midpoints": {"orbDeg": 11}}, "synastry.midpoints.orbDeg"),
                ({"partner": mary, "composite": "yes"}, "synastry.composite"),
                ({"partner": mary, "davison": 1}, "synastry.davison"),
            ]
            for request, field in refusals:
                with self.assertRaises(TeistroError) as caught:
                    ctx.chart.found(instant=birth, synastry=request, **george)
                self.assertEqual(caught.exception.field, field)

    def test_a_chart_carries_the_outer_planets_when_asked(self) -> None:
        """The outer planets cross when asked: none unless `outer_planets`,
        then Uranus, Neptune and Pluto in the grahas' shape with the nine
        unmoved, a progression's later charts carrying them, and Leo's
        progressed Moon quincunx Uranus in April 1907 (p. 41)
        (`03-design/western-outer-planets.md`)."""
        from teistro import NatalPoint, ProgressionContacts

        london: dict[str, Any] = {
            "place": Observer(latitude_deg=Latitude(51.5), longitude_deg=Longitude(0), altitude_m=Altitude(0)),
            "utc_offset_seconds": 0,
        }
        birth = 2400629.742361111
        with self.teistro.context(profile="western-tropical-default", ephemeris=Ephemeris.BUILTIN) as ctx:
            bare = ctx.chart.found(instant=birth, **london)
            self.assertEqual(bare.outer, [])
            asked = ctx.chart.found(instant=birth, outer_planets=True, **london)
            self.assertEqual([at.graha for at in asked.outer], [Graha.URANUS, Graha.NEPTUNE, Graha.PLUTO])
            self.assertEqual(asked.grahas, bare.grahas, "the nine are unmoved")
            for placed in asked.outer:
                self.assertGreater(placed.distance_au, 15)
                self.assertIn(placed.house.bhava, range(1, 13))

            at = birth + 46 * 365.242189
            later = ctx.chart.found(instant=birth, outer_planets=True, progressions={"at": at}, **london).progressions
            assert later is not None and later.progressed is not None and later.directed is not None
            self.assertEqual(len(later.progressed.grahas), 12)
            self.assertEqual(len(later.directed.planets), 12)
            self.assertIs(later.progressed.grahas[11].graha, Graha.PLUTO)

            contacts: ProgressionContacts = {
                "from": 2417484.5,
                "to": 2417941.5,
                "grahas": ["MOON"],
                "points": [Graha.URANUS],
            }
            found = ctx.chart.found(
                instant=birth, outer_planets=True, progressions={"contacts": contacts}, **london
            ).progressions
            assert found is not None and found.contacts is not None
            (contact,) = found.contacts
            self.assertEqual((contact.to, contact.angle), (NatalPoint("GRAHA", Graha.URANUS), 150))
            self.assertTrue(2417635.5 <= contact.life < 2417696.5, f"{contact.life}: March or April 1907")
            with self.assertRaises(TeistroError):
                ctx.chart.found(instant=birth, progressions={"contacts": contacts}, **london)

    def test_a_chart_carries_its_lots(self) -> None:
        """Valens's lots cross whole, members resolved: the sect and the rules
        read back and handed back as a request, all fourteen in the
        catalogue's order, Fortune where its formula puts it and Daimon its
        mirror in the ascendant, a batch the charts one at a time, and a
        refusal named in the record (`03-design/hellenistic-lots.md`)."""
        from teistro import FortuneRule, Lot, LotRequest, LotRules, Sect, SectRule

        kathmandu: dict[str, Any] = {
            "place": Observer(latitude_deg=Latitude(27.7172), longitude_deg=Longitude(85.324), altitude_m=Altitude(0)),
            "utc_offset_seconds": 20700,
        }
        instants = [2460676.5, 2460676.75]

        def apart(one: float, other: float) -> float:
            return min((one - other) % 360, (other - one) % 360)

        with self.teistro.context(profile="conformance-baseline", ephemeris=Ephemeris.BUILTIN) as ctx:
            self.assertIsNone(ctx.chart.found(instant=instants[0], **kathmandu).lots)
            for instant in instants:
                chart = ctx.chart.found(instant=instant, lots={}, fortitudes={}, **kathmandu)
                read = chart.lots
                assert read is not None and chart.fortitudes is not None
                self.assertEqual(read.request, LotRules(SectRule.HORIZON, FortuneRule.REVERSED_BY_NIGHT))
                self.assertEqual(read.fortune_reversed, read.sect is Sect.NIGHT)
                self.assertEqual([placed.lot for placed in read.lots], list(Lot))
                at = {placed.lot: placed.place.longitude_deg for placed in read.lots}
                longitude = {own.planet: own.longitude_deg for own in chart.fortitudes.dignities.planets}
                ascendant = chart.fortitudes.sky.ascendant_deg
                start, end = (Graha.MOON, Graha.SUN) if read.fortune_reversed else (Graha.SUN, Graha.MOON)
                fortune = (ascendant + longitude[end] - longitude[start]) % 360
                self.assertLess(apart(at[Lot.FORTUNE], fortune), 1e-9)
                self.assertLess(apart(at[Lot.DAIMON], 2 * ascendant - fortune), 1e-9, "Daimon mirrors Fortune")
                for placed in read.lots:
                    self.assertTrue(0 <= placed.place.longitude_deg < 360)
                    self.assertIn(placed.place.house, range(1, 13))

                # The answer's rules are a request as they stand.
                fed_back = ctx.chart.found(instant=instant, lots=read.request, **kathmandu)
                self.assertEqual(fed_back.lots, read)

            asked: LotRequest = {"sectRule": SectRule.DAYLIGHT, "fortune": FortuneRule.REVERSED_WHILE_MOON_UP}
            other = ctx.chart.found(instant=instants[0], lots=asked, **kathmandu).lots
            assert other is not None
            self.assertEqual(other.request, LotRules(SectRule.DAYLIGHT, FortuneRule.REVERSED_WHILE_MOON_UP))

            batch = ctx.chart.found_many(instants=instants, lots={}, **kathmandu)
            for k, instant in enumerate(instants):
                self.assertEqual(batch.at(k).lots, ctx.chart.found(instant=instant, lots={}, **kathmandu).lots)

            refusals: list[tuple[dict[str, Any], str]] = [
                ({"lots": {"fortuna": "DAY_AND_NIGHT"}}, "lots.fortuna"),
                ({"lots": {"fortune": "REVERSED"}}, "lots.fortune"),
            ]
            for bad, field in refusals:
                with self.assertRaises(TeistroError) as refused:
                    ctx.chart.found(instant=instants[0], **bad, **kathmandu)
                self.assertEqual(refused.exception.field, field)

    def test_an_almanac_carries_the_muhurta_search_it_was_asked_for(self) -> None:
        """A muhurta search crosses beside the days it judged
        (`03-design/muhurta-at-the-boundary.md`): its clauses a class a kind,
        members resolved, its days the almanac's own, and a clause it gives
        handed straight back as a bar (§2.5)."""
        from teistro import (
            ChoghadiyaClause,
            MuhurtaAnswer,
            MuhurtaRequest,
            NakshatraClause,
            TarabalaClause,
            date,
        )
        from teistro.catalogue import BlackoutKind, Calendar, Choghadiya, Nakshatra, Rashi

        observer = Observer(
            latitude_deg=Latitude(27.7172), longitude_deg=Longitude(85.324), altitude_m=Altitude(1400)
        )
        days: dict[str, Any] = {
            "from_date": date(Calendar.GREGORIAN, 2026, 11, 25),
            "to_date": date(Calendar.GREGORIAN, 2026, 12, 3),
            "place": observer,
            "utc_offset_seconds": 20700,
        }
        with self.teistro.context(ephemeris=Ephemeris.BUILTIN) as ctx:
            plain = ctx.almanac.of(**days)
            self.assertIsNone(plain.muhurta)
            asked: MuhurtaRequest = {
                "rules": "RAMAN_MARRIAGE",
                "native": {"star": Nakshatra.ROHINI, "moonSign": "rashi.TAURUS", "lagna": Rashi.LEO},
                "daysWithWindows": 9,
                "most": 1000,
            }
            almanac = ctx.almanac.of(**days, muhurta=asked)
            answer = almanac.muhurta
            assert answer is not None
            self.assertIsInstance(answer, MuhurtaAnswer)
            self.assertTrue(answer.windows)
            self.assertEqual(answer.ranking, "TEXTS")
            # The days are the ones asked without a search.
            for k in range(len(plain)):
                self.assertEqual(almanac.at(k).provenance.content_hash, plain.at(k).provenance.content_hash)
            kinds = [clause.kind for window in answer.windows for clause in window.clauses]
            self.assertTrue(all(isinstance(k.nakshatra, Nakshatra) for k in kinds if isinstance(k, NakshatraClause)))
            self.assertTrue(any(isinstance(k, TarabalaClause) for k in kinds), "the native is read")
            knobs = {c.knob for c in answer.provenance.applied_conventions}
            self.assertLessEqual({"muhurta.asta", "muhurta.zodiacAt"}, knobs)

            # A clause answered is a bar a request may name, as it was read.
            amrit = next(k for k in kinds if isinstance(k, ChoghadiyaClause) and k.choghadiya == Choghadiya.AMRIT)
            graded = {"best": [], "middling": [], "rejected": [], "otherwise": "MIDDLING"}
            rules = {
                "day": {
                    "tithis": graded,
                    "nakshatras": graded,
                    "yogas": graded,
                    "karanas": graded,
                    "varas": graded,
                    "chandrabala": {"avoid": []},
                },
                "months": {"reckoning": "ANY"},
                "lagnas": graded,
                "padas": [],
                "heeds": [],
                "bars": [amrit],
                "unjudged": [],
                "baseline": None,
            }
            one_day = {**days, "to_date": days["from_date"]}
            barred = ctx.almanac.of(**one_day, muhurta={"rules": rules, "daysWithWindows": 1, "most": 100000}).muhurta
            assert barred is not None
            struck = [window for window in barred.windows if window.barred_by]
            self.assertTrue(struck, "the bar read back strikes the windows it names")
            self.assertTrue(all(window.barred_by == (amrit,) for window in struck))

            # A closed day names its blackouts as members, and a request
            # takes one in either spelling: 1 June 2026 is in Jyeshtha's
            # adhika month.
            june = date(Calendar.GREGORIAN, 2026, 6, 1)

            def closed_by(muhurta: MuhurtaRequest) -> tuple[BlackoutKind, ...]:
                found = ctx.almanac.of(**{**days, "from_date": june, "to_date": june}, muhurta=muhurta).muhurta
                assert found is not None
                return found.closed[0].by

            self.assertIn(BlackoutKind.ADHIKA_MASA, closed_by({"rules": "RAMAN_MARRIAGE"}))
            for heed in (BlackoutKind.ADHIKA_MASA, "ADHIKA_MASA"):
                heeding = {**rules, "bars": [], "heeds": [heed]}
                self.assertEqual(closed_by({"rules": heeding}), (BlackoutKind.ADHIKA_MASA,))

            refusals: list[tuple[MuhurtaRequest, str]] = [
                ({"rules": "RAMAN"}, "muhurta.rules"),  # type: ignore[typeddict-item]
                ({"rules": "RAMAN_MARRIAGE", "most": 0}, "muhurta.most"),
            ]
            for bad, field in refusals:
                with self.assertRaises(TeistroError) as refused:
                    ctx.almanac.of(**days, muhurta=bad)
                self.assertEqual(refused.exception.field, field)
            with self.assertRaises(TeistroError) as refused:
                ctx.almanac.of(**days, muhurta="RAMAN_MARRIAGE")  # type: ignore[arg-type]
            self.assertEqual(refused.exception.field, "muhurta")

            # A rite beyond marriage, by name: its unwanted placements come
            # back typed, with their house and grahas as members, and the
            # 8th the thread ceremony says must be empty bars as itself.
            from teistro import UnwantedPlacementClause
            from teistro.catalogue import Graha

            thread = ctx.almanac.of(
                **days, muhurta={"rules": "RAMAN_UPANAYANA", "daysWithWindows": 9, "most": 1000}
            ).muhurta
            assert thread is not None
            placed = [c.kind for w in thread.windows for c in w.clauses if isinstance(c.kind, UnwantedPlacementClause)]
            self.assertTrue(placed)
            self.assertTrue(all(1 <= p.house <= 12 and all(isinstance(g, Graha) for g in p.by) for p in placed))
            barring = [b for w in thread.windows for b in w.barred_by if isinstance(b, UnwantedPlacementClause)]
            self.assertTrue(all(b.house == 8 for b in barring))
            nowhere = {**rules, "unwanted": [{"grahas": ["MARS"], "houses": [13]}]}
            with self.assertRaises(TeistroError) as refused:
                ctx.almanac.of(**days, muhurta={"rules": nowhere})
            self.assertEqual(refused.exception.field, "muhurta.rules.unwanted[0].houses")

    def test_an_almanac_carries_the_lunar_years_it_was_asked_for(self) -> None:
        """The lunar years cross beside the days they hold
        (`03-design/calendar-indian-lunisolar.md` §10): members as members,
        abutting, frozen, and the days the almanac's own."""
        import dataclasses

        from teistro import LunarYears, date
        from teistro.catalogue import Calendar, Samvatsara

        observer = Observer(
            latitude_deg=Latitude(27.7172), longitude_deg=Longitude(85.324), altitude_m=Altitude(1400)
        )
        # Across Chaitra Shukla Pratipada of VS 2083, 19 March 2026.
        days: dict[str, Any] = {
            "from_date": date(Calendar.GREGORIAN, 2026, 3, 10),
            "to_date": date(Calendar.GREGORIAN, 2026, 4, 10),
            "place": observer,
            "utc_offset_seconds": 20700,
        }
        with self.teistro.context(ephemeris=Ephemeris.BUILTIN) as ctx:
            plain = ctx.almanac.of(**days)
            self.assertIsNone(plain.years)
            almanac = ctx.almanac.of(**days, years=True)
            answer = almanac.years
            assert answer is not None
            self.assertIsInstance(answer, LunarYears)
            years = answer.value
            self.assertEqual([y.samvatsara for y in years], [Samvatsara.SIDDHARTHI, Samvatsara.RAUDRA])
            self.assertEqual([y.vikrama for y in years], [2082, 2083])
            self.assertEqual([y.count for y in years], ["BARHASPATYA", "BARHASPATYA"])
            self.assertEqual(years[0].ended, years[1].began)
            self.assertLess(years[1].opened, years[1].began)
            self.assertIsInstance(years[1].jovian[0].member, Samvatsara)
            with self.assertRaises(dataclasses.FrozenInstanceError):
                years[0].vikrama = 0  # type: ignore[misc]
            self.assertTrue(answer.provenance.content_hash)
            for k in range(len(plain)):
                self.assertEqual(almanac.at(k).provenance.content_hash, plain.at(k).provenance.content_hash)

    def test_an_almanac_carries_the_eclipses_it_was_asked_for(self) -> None:
        """The eclipses cross beside the days they fall in
        (`03-design/eclipses.md` §5): frozen, their kinds bare keys, each
        with how the almanac's place sees it, and the shadow knob moving
        the umbra."""
        import dataclasses

        from teistro import Eclipses, date
        from teistro.catalogue import Calendar

        observer = Observer(
            latitude_deg=Latitude(27.7172), longitude_deg=Longitude(85.324), altitude_m=Altitude(1400)
        )
        # September 2025 at Kathmandu: the total lunar eclipse of the 7th,
        # seen whole near midnight, and the partial solar eclipse of the
        # 21st over the South Pacific, which Nepal does not see.
        days: dict[str, Any] = {
            "from_date": date(Calendar.GREGORIAN, 2025, 9, 1),
            "to_date": date(Calendar.GREGORIAN, 2025, 9, 30),
            "place": observer,
            "utc_offset_seconds": 20700,
        }

        def asked(settings: Optional[dict[str, Any]] = None) -> tuple[Eclipses, Optional[Eclipses]]:
            with self.teistro.context(ephemeris=Ephemeris.BUILTIN, settings=settings) as ctx:
                found = ctx.almanac.of(**days, eclipses=True).eclipses
                assert found is not None
                return found, ctx.almanac.of(**days).eclipses

        answer, plain = asked()
        self.assertIsNone(plain)
        self.assertIsInstance(answer, Eclipses)
        self.assertEqual([e.eclipse.kind for e in answer.value.lunar], [LunarEclipseKind.TOTAL])
        self.assertEqual([e.eclipse.kind for e in answer.value.solar], [SolarEclipseKind.PARTIAL])
        lunar = answer.value.lunar[0]
        self.assertEqual(lunar.eclipse.shadow, "DANJON")
        u2 = lunar.eclipse.contacts.u2
        assert u2 is not None
        self.assertLess(u2, lunar.eclipse.greatest)
        self.assertGreater(lunar.here.greatest.altitude_deg, 40)
        seen = lunar.here.seen
        assert seen is not None
        self.assertEqual((seen.from_, seen.to), (lunar.here.p1.at, lunar.here.p4.at))
        umbral, u1, u4 = lunar.here.umbral_seen, lunar.here.u1, lunar.here.u4
        assert umbral is not None and u1 is not None and u4 is not None
        self.assertEqual((umbral.from_, umbral.to), (u1.at, u4.at))
        with self.assertRaises(dataclasses.FrozenInstanceError):
            seen.to = 0  # type: ignore[misc]
        solar = answer.value.solar[0]
        self.assertTrue(solar.here is None or solar.here.seen is None)
        self.assertIn("eclipse.window", [c.knob for c in answer.provenance.applied_conventions])

        chauvenet, _ = asked({"panchanga": {"eclipse_shadow": "CHAUVENET"}})
        self.assertEqual(chauvenet.value.lunar[0].eclipse.shadow, "CHAUVENET")
        self.assertGreater(
            chauvenet.value.lunar[0].eclipse.umbral_magnitude, lunar.eclipse.umbral_magnitude
        )
        self.assertNotEqual(chauvenet.provenance.settings_hash, answer.provenance.settings_hash)

    def test_an_almanac_carries_each_days_nepal_sambat_date(self) -> None:
        """Each day's Nepal Sambat date crosses beside the days
        (`03-design/calendar-indian-lunisolar.md` §11): one a day, frozen,
        the year turning at Kachhala's first day, and said as the
        committee's page header prints it."""
        import dataclasses

        from teistro import NepalSambatDates, date
        from teistro.catalogue import Calendar, MonthKind, Paksha

        observer = Observer(
            latitude_deg=Latitude(27.7172), longitude_deg=Longitude(85.324), altitude_m=Altitude(1400)
        )
        days: dict[str, Any] = {
            "from_date": date(Calendar.GREGORIAN, 2025, 10, 20),
            "to_date": date(Calendar.GREGORIAN, 2025, 10, 23),
            "place": observer,
            "utc_offset_seconds": 20700,
        }
        with self.teistro.context(
            profile=PROFILE, locale="ne-Deva-NP", ephemeris=Ephemeris.BUILTIN
        ) as ctx:
            self.assertIsNone(ctx.almanac.of(**days).nepal_sambat)
            almanac = ctx.almanac.of(**days, nepal_sambat=True)
            answer = almanac.nepal_sambat
            assert answer is not None
            self.assertIsInstance(answer, NepalSambatDates)
            self.assertEqual(len(answer.value), len(almanac))
            self.assertEqual(
                [(d.year, d.month, d.kind, d.paksha) for d in answer.value],
                [
                    (1145, 12, MonthKind.NIJA, Paksha.KRISHNA),
                    (1145, 12, MonthKind.NIJA, Paksha.KRISHNA),
                    (1146, 1, MonthKind.NIJA, Paksha.SHUKLA),
                    (1146, 1, MonthKind.NIJA, Paksha.SHUKLA),
                ],
            )
            self.assertEqual(answer.provenance.input_hash, almanac.provenance.input_hash)
            with self.assertRaises(dataclasses.FrozenInstanceError):
                answer.value[0].year = 0  # type: ignore[misc]
            first = answer.value[2]
            said = ctx.intl.messages.sdk.calendar.nepal_sambat_date(
                year=first.year, month=first.month, kind=first.kind.key, paksha=first.paksha.key
            )
            self.assertEqual(said, "ने.सं. ११४६ कछलाथ्व")

    def test_an_almanac_carries_the_festivals_it_was_asked_for(self) -> None:
        """Festival rules cross beside the days they fall on
        (`03-design/festival-rules.md` §7): dates in this binding's shape,
        the days the almanac's own, a shipped rule replaced by its key with
        members given as members, a day counted from another rule's, and a
        refusal named by the item and field that was wrong."""
        from teistro import EkadashiFast, FestivalAnswer, FestivalDecided, FestivalRequest, date
        from teistro.catalogue import Calendar, Masa, Tithi

        observer = Observer(
            latitude_deg=Latitude(27.7172), longitude_deg=Longitude(85.324), altitude_m=Altitude(1400)
        )
        days: dict[str, Any] = {
            "from_date": date(Calendar.GREGORIAN, 2026, 10, 15),
            "to_date": date(Calendar.GREGORIAN, 2026, 11, 10),
            "place": observer,
            "utc_offset_seconds": 20700,
        }
        with self.teistro.context(ephemeris=Ephemeris.BUILTIN) as ctx:
            plain = ctx.almanac.of(**days)
            self.assertIsNone(plain.festivals)
            almanac = ctx.almanac.of(**days, festivals={"rules": "DHARMASINDHU"})
            answer = almanac.festivals
            assert answer is not None
            self.assertIsInstance(answer, FestivalAnswer)
            self.assertEqual(
                [o.rule for o in answer.observances], ["VIJAYA_DASHAMI", "LAKSHMI_PUJA", "BALI_PRATIPADA"]
            )
            dashami = answer.observances[0]
            self.assertEqual((dashami.day.calendar, dashami.day.month), (Calendar.GREGORIAN, 10))
            self.assertEqual(dashami.extents[0].day.calendar, Calendar.GREGORIAN)
            self.assertIn(dashami.decided_by.by, ("GUARD", "OTHERWISE"))
            self.assertEqual(answer.unjudged, ())
            observers = ["EKADASHI_VAISHNAVA", "EKADASHI_SMARTA", "EKADASHI_SMARTA_RENUNCIANT"]
            self.assertEqual([f.rule for f in answer.ekadashis], observers * 2)
            fast = answer.ekadashis[0]
            self.assertIsInstance(fast, EkadashiFast)
            self.assertEqual((fast.tithi, fast.month), (Tithi.SHUKLA_EKADASHI, Masa.ASHWINA))
            self.assertEqual(fast.days[0].calendar, Calendar.GREGORIAN)
            self.assertIn(fast.choice, ("EARLIER", "LATER"))
            self.assertEqual(fast.day, fast.days[0 if fast.choice == "EARLIER" else 1])
            for k in range(len(plain)):
                self.assertEqual(almanac.at(k).provenance.content_hash, plain.at(k).provenance.content_hash)
            self.assertIn("festival.days", {c.knob for c in answer.provenance.applied_conventions})

            sunrise = {
                "key": "LAKSHMI_PUJA",
                "source": "the tithi at sunrise",
                "month": Masa.ASHWINA,
                "tithi": Tithi.AMAVASYA,
                "at": {"window": "SUNRISE"},
                "decide": [],
                "otherwise": "LATER",
            }
            moved = ctx.almanac.of(**days, festivals={"rules": ["DHARMASINDHU", sunrise]}).festivals
            assert moved is not None
            self.assertEqual(moved.observances[1].decided_by.by, "OTHERWISE")
            self.assertIsNone(moved.observances[1].decided_by.index)
            self.assertNotEqual(moved.provenance.input_hash, answer.provenance.input_hash)

            # Nepal's pack, and a day of the consumer's own counted two days
            # from Lakshmi puja's: it names the rule it counts from and the count.
            following = {"key": "TWO_AFTER", "source": "mine", "after": "LAKSHMI_PUJA", "days": 2}
            counted = ctx.almanac.of(**days, festivals={"rules": ["NEPAL", following]}).festivals
            assert counted is not None
            by_rule = {o.rule: o for o in counted.observances}
            two, lakshmi = by_rule["TWO_AFTER"], by_rule["LAKSHMI_PUJA"]
            self.assertEqual(two.decided_by, FestivalDecided(by="AFTER", index=None, rule="LAKSHMI_PUJA", days=2))
            self.assertEqual(two.day.day, lakshmi.day.day + 2)
            self.assertEqual(two.tithi, lakshmi.tithi)

            # Every observance says its month; Nepal's monthly full-moon fast
            # is judged at the instant of sunset, and a rule of one's own kept
            # every month leaves its month out.
            self.assertEqual((dashami.month, dashami.adhika), (Masa.ASHWINA, False))
            vrata = by_rule["PURNIMA_VRATA"]
            self.assertEqual(vrata.month, Masa.ASHWINA)
            self.assertEqual(vrata.extents[0].window.from_jd, vrata.extents[0].window.to_jd)
            every_month = {
                "key": "EVERY_FULL_MOON",
                "source": "mine",
                "tithi": "tithi.PURNIMA",
                "inAdhika": True,
                "at": {"window": "SUNSET"},
                "decide": [],
                "otherwise": "LATER",
            }
            mine = ctx.almanac.of(**days, festivals={"rules": [every_month]}).festivals
            assert mine is not None
            self.assertEqual([(o.rule, o.month) for o in mine.observances], [("EVERY_FULL_MOON", Masa.ASHWINA)])
            hasta = {k: v for k, v in every_month.items() if k != "tithi"}
            hasta.update(key="DARK_HASTA", nakshatra="nakshatra.HASTA", paksha="paksha.KRISHNA")
            kept = ctx.almanac.of(**days, festivals={"rules": [hasta]}).festivals
            assert kept is not None
            self.assertEqual([(o.rule, o.month) for o in kept.observances], [("DARK_HASTA", Masa.ASHWINA)])

            refusals: list[tuple[FestivalRequest, str]] = [
                ({"rules": "DHARMA"}, "festivals.rules"),  # type: ignore[typeddict-item]
                ({"rules": ["DHARMASINDHU", {**sunrise, "key": ""}]}, "festivals.rules[1].key"),
                ({"rules": [{**sunrise, "at": {"window": "DUSK"}}]}, "festivals.rules[0].at.window"),
                ({"rules": [{**sunrise, "nakshatra": "HASTA", "paksha": "SHUKLA"}]}, "festivals.rules[0]"),
                (
                    {"rules": [{**sunrise, "at": {"window": "NIGHT_MUHURTA", "muhurta": 16}}]},
                    "festivals.rules[0].at.muhurta",
                ),
                (
                    {"rules": ["DHARMASINDHU", {"key": "MINE", "source": "", "vedha": "DUSK", "table": {}}]},
                    "festivals.rules[1].vedha",
                ),
                (
                    {"rules": ["DHARMASINDHU", {"key": "MINE", "source": "", "after": "HOLIKA", "days": 16}]},
                    "festivals.rules[1].days",
                ),
                (
                    {"rules": ["DHARMASINDHU", {"key": "MINE", "source": "", "after": "NOBODY", "days": 1}]},
                    "festivals.following[0].after",
                ),
            ]
            for bad, field in refusals:
                with self.assertRaises(TeistroError) as refused:
                    ctx.almanac.of(**days, festivals=bad)
                self.assertEqual(refused.exception.field, field)

    def test_a_chart_carries_its_transits_each_verdict_its_own_house_and_vedha(self) -> None:
        """A chart's transits cross whole: a reading an instant in the order
        asked, counted from what was asked, nine grahas each whose verdict
        agrees with its own house, vedha and obstructors; the nodes always
        opposite; empty unless asked; and no instant refused by the field."""
        from teistro import Fruition, GocharFrom, GocharVerdict, NodeObstruction, NodeVedha

        observer = Observer(
            latitude_deg=Latitude(27.7172), longitude_deg=Longitude(85.324), altitude_m=Altitude(1400)
        )
        birth = 2447995.4895833335
        self.assertEqual(self.ctx.chart.found(instant=birth, place=observer, utc_offset_seconds=20700).gochar, ())
        instants = [2460676.5 + 30 * k for k in range(24)]
        verdicts = set()
        for from_ in (GocharFrom.MOON, GocharFrom.LAGNA):
            readings = self.ctx.chart.found(
                instant=birth,
                place=observer,
                utc_offset_seconds=20700,
                gochar={"instants": instants, "from": "MOON" if from_ is GocharFrom.MOON else "LAGNA"},
            ).gochar
            self.assertEqual(len(readings), len(instants))
            for reading, instant in zip(readings, instants):
                self.assertEqual(reading.instant, instant)
                self.assertIs(reading.reference.from_, from_)
                self.assertIs(reading.rules.node_vedha, NodeVedha.LIKE_THE_SUN)
                self.assertIs(reading.rules.node_obstruction, NodeObstruction.NOT_EACH_OTHER)
                self.assertEqual(len(reading.grahas), 9)
                for g in reading.grahas:
                    self.assertTrue(1 <= g.house <= 12, g.house)
                    self.assertTrue(0 <= g.transit.degrees < 30, g.transit.degrees)
                    expected = (
                        GocharVerdict.NOT_GOOD
                        if not g.good_house
                        else GocharVerdict.OBSTRUCTED
                        if g.obstructed_by
                        else GocharVerdict.GOOD
                    )
                    self.assertIs(g.verdict, expected, f"{g.graha} in {g.house}")
                    self.assertIsInstance(g.fruition, Fruition)
                    if not g.good_house:
                        self.assertIsNone(g.vedha_house)
                    verdicts.add(g.verdict)
                rahu, ketu = reading.grahas[7], reading.grahas[8]
                self.assertEqual((ketu.house - rahu.house) % 12, 6, "the nodes stand opposite")
                self.assertNotIn(Graha.KETU, rahu.obstructed_by, "C140: the nodes spare each other")
        self.assertEqual(verdicts, set(GocharVerdict))
        judged = self.ctx.chart.found(
            instant=birth, place=observer, utc_offset_seconds=20700, gochar={"instants": instants, "ashtakavarga": True}
        ).gochar
        for reading in judged:
            assert reading.ashtakavarga is not None
            self.assertEqual(len(reading.ashtakavarga), 7)
            for one, moving in zip(reading.ashtakavarga, reading.grahas):
                self.assertIs(one.graha, moving.graha)
                self.assertEqual(one.good, one.bindus >= 5)
                self.assertEqual(one.kakshya.index, int(moving.transit.degrees // 3.75) + 1)
        self.assertIsNone(
            self.ctx.chart.found(
                instant=birth, place=observer, utc_offset_seconds=20700, gochar={"instants": instants}
            ).gochar[0].ashtakavarga
        )
        with self.assertRaises(TeistroError) as refused:
            self.ctx.chart.found(instant=birth, place=observer, utc_offset_seconds=20700, gochar={"instants": []})
        self.assertEqual(refused.exception.field, "gochar.instants")

    def test_a_chart_carries_its_karakamsha_and_its_brahma_or_why_not(self) -> None:
        """A chart's Jaimini significators cross whole: the karakamsha with
        nine houses in each chart, the Atmakaraka's own navamsha the first
        from it, and the Brahma graha found or its absence named -- never
        both, never neither; None unless asked."""
        observer = Observer(
            latitude_deg=Latitude(27.7172), longitude_deg=Longitude(85.324), altitude_m=Altitude(1400)
        )
        self.assertIsNone(self.ctx.chart.found(instant=2451545.0, place=observer, utc_offset_seconds=20700).jaimini)
        outcomes = set()
        for step in range(40):
            chart = self.ctx.chart.found(
                instant=2451545.0 + step * 0.37, place=observer, utc_offset_seconds=20700, jaimini=True
            )
            reading = chart.jaimini
            assert reading is not None
            k, b = reading.karakamsha, reading.brahma
            for houses in (k.in_rasi, k.in_navamsha):
                self.assertEqual(len(houses), 9)
                self.assertTrue(all(1 <= house <= 12 for house in houses), houses)
            self.assertEqual(k.in_navamsha[int(k.atmakaraka)], 1, "the Atmakaraka stands in the karakamsha")
            self.assertIs(b.rule, BrahmaRule.VERSES)
            self.assertEqual(b.graha is None, b.none is not None)
            self.assertIsNot(b.none, BrahmaOutcome.FOUND)
            if b.graha is not None:
                self.assertIn(b.passed_from or b.graha, b.qualified)
            outcomes.add(b.none or BrahmaOutcome.FOUND)
            self.assertEqual(len(reading.graha_arudhas), 9)
            self.assertTrue(all(isinstance(sign, Rashi) for sign in reading.graha_arudhas[:7]))
            # The default co-lordship gives the nodes no own sign, so no arudha.
            self.assertEqual(reading.graha_arudhas[7:], (None, None))
        self.assertIn(BrahmaOutcome.FOUND, outcomes)
        self.assertGreater(len(outcomes), 1, outcomes)

    def test_a_chart_carries_its_dasha_phala_and_the_shadbala_its_rays(self) -> None:
        """A chart's dasha phala crosses whole: the nine grahas' Subhankas within
        each varga's share and complementary in total, a nature and a phase
        each; None unless asked, and the Shadbala's rays beside the phalas."""
        observer = Observer(
            latitude_deg=Latitude(27.7172), longitude_deg=Longitude(85.324), altitude_m=Altitude(1400)
        )
        chart = self.ctx.chart.found(
            instant=2451545.0, place=observer, utc_offset_seconds=20700, dasha_phala=True, shadbala=True
        )
        self.assertIsNone(self.ctx.chart.found(instant=2451545.0, place=observer, utc_offset_seconds=20700).dasha_phala)
        reading = chart.dasha_phala
        assert reading is not None
        self.assertEqual([g.graha.name for g in reading.grahas][-2:], ["RAHU", "KETU"])
        for g in reading.grahas:
            self.assertEqual(len(g.subhankas), 7)
            for k, points in enumerate(g.subhankas):
                self.assertTrue(0 <= points <= (60 if k == 0 else 30))
            self.assertAlmostEqual(g.subhanka + g.asubhanka, 240.0, places=9)
            self.assertIsInstance(g.nature, Nature)
            self.assertIsInstance(g.phase, DashaPhase)
        shadbala = chart.shadbala
        assert shadbala is not None
        for s in shadbala.grahas:
            self.assertTrue(1 <= s.subha_rashmi <= 7)
            self.assertAlmostEqual(s.subha_rashmi + s.ashubha_rashmi, 8.0, places=9)

    def test_a_graha_s_state_carries_its_sayanadi_and_a_sub_state_for_every_anka(self) -> None:
        """Every graha's state carries its Sayanadi: the nine grahas a state and
        a sub-state under each of the five ankas, the outer planets none."""
        observer = Observer(
            latitude_deg=Latitude(27.7172), longitude_deg=Longitude(85.324), altitude_m=Altitude(1400)
        )
        states = self.ctx.chart.found(instant=2451545.0, place=observer, utc_offset_seconds=20700, state=True).states
        nine = {"SUN", "MOON", "MARS", "MERCURY", "JUPITER", "VENUS", "SATURN", "RAHU", "KETU"}
        for state in states:
            if state.graha.name not in nine:
                self.assertIsNone(state.sayanadi)
                continue
            sayanadi = state.sayanadi
            assert sayanadi is not None
            self.assertIsInstance(sayanadi.avastha, AvasthaSayanadi)
            self.assertEqual(len(sayanadi.cheshtas), 5)
            self.assertEqual(sayanadi.cheshta(3), sayanadi.cheshtas[2])
            with self.assertRaises(ValueError):
                sayanadi.cheshta(6)

    def test_a_chart_carries_its_vaiseshikamsa_each_scheme_s_count_and_name(self) -> None:
        """A chart's Vaiseshikamsa crosses whole: each scheme's count within its
        vargas, a name for every count from two; None unless asked."""
        observer = Observer(
            latitude_deg=Latitude(27.7172), longitude_deg=Longitude(85.324), altitude_m=Altitude(1400)
        )
        chart = self.ctx.chart.found(instant=2451545.0, place=observer, utc_offset_seconds=20700, vaiseshikamsa=True)
        self.assertIsNone(self.ctx.chart.found(instant=2451545.0, place=observer, utc_offset_seconds=20700).vaiseshikamsa)
        reading = chart.vaiseshikamsa
        assert reading is not None
        self.assertEqual(len(reading.grahas), 7)
        for g in reading.grahas:
            for standing, vargas in (
                (g.shadvarga, 6),
                (g.saptavarga, 7),
                (g.dashavarga, 10),
                (g.shodashavarga, 16),
            ):
                self.assertLessEqual(standing.good_vargas, vargas)
                self.assertEqual(standing.name is None, standing.good_vargas < 2)

    def test_a_chart_carries_its_vimshopaka_each_graha_s_four_scores(self) -> None:
        """A chart's Vimshopaka crosses whole: every graha's four scores out of
        20 under the default reading, the text's, whose least in any varga is
        5; None unless asked."""
        observer = Observer(
            latitude_deg=Latitude(27.7172), longitude_deg=Longitude(85.324), altitude_m=Altitude(1400)
        )
        chart = self.ctx.chart.found(instant=2451545.0, place=observer, utc_offset_seconds=20700, vimshopaka=True)
        self.assertIsNone(self.ctx.chart.found(instant=2451545.0, place=observer, utc_offset_seconds=20700).vimshopaka)
        vs = chart.vimshopaka
        assert vs is not None
        self.assertIs(vs.scoring, VimshopakaScoring.BPHS)
        self.assertEqual([g.graha.name for g in vs.grahas], ["SUN", "MOON", "MARS", "MERCURY", "JUPITER", "VENUS", "SATURN"])
        for g in vs.grahas:
            for score in (g.shadvarga, g.saptavarga, g.dashavarga, g.shodashavarga):
                self.assertTrue(5.0 <= score <= 20.0, f"{g.graha}: {score}")

    def test_a_consumer_system_counted_with_abhijit_reads_as_the_text_row(self) -> None:
        """A consumer's system may count over the twenty-eight nakshatras
        with Abhijit in groups of its own: Shashtihayani stated as BPHS
        states it reads as the catalogued one (crux C1)."""
        years = (("JUPITER", 10), ("SUN", 10), ("MARS", 10), ("MOON", 6), ("MERCURY", 6),
                 ("VENUS", 6), ("SATURN", 6), ("RAHU", 6))
        shashti: UduDashaDefinition = {
            "kernel": "UDU",
            "key": "ACME_SHASHTI",
            "lords": [{"graha": graha, "years": count} for graha, count in years],
            "reference": "ASHWINI",
            "groups": [3, 4, 3, 4, 3, 4, 3, 4],
            "wheel": "WITH_ABHIJIT",
            "repeats": False,
        }
        observer = Observer(
            latitude_deg=Latitude(27.7172), longitude_deg=Longitude(85.324), altitude_m=Altitude(1400)
        )
        with self.teistro.context(test_provider=True, dasha_systems=[shashti]) as ctx:
            consumer, shipped = ctx.chart.found(
                instant=2451545.0,
                place=observer,
                utc_offset_seconds=20700,
                dashas=["dasha_system.ACME_SHASHTI", DashaSystem.SHASHTIHAYANI],
            ).dashas
            self.assertEqual(consumer.periods, shipped.periods)
            self.assertEqual(consumer.balance, shipped.balance)
        short: UduDashaDefinition = {**shashti, "groups": [3, 4]}
        with self.assertRaises(TeistroError) as refused:
            self.teistro.context(test_provider=True, dasha_systems=[short])
        self.assertEqual(refused.exception.field, "options.dashas_json[0].groups")

    def test_a_consumer_dasha_system_registers_is_asked_for_by_key_and_reads_as_its_twin(self) -> None:
        """A consumer's own dasha system crosses: registered on the context,
        asked for by its key, named by it in the answer, and every period its
        catalogued twin's; a definition the checks refuse is named by its place
        and field (`03-design/dasha-kernels.md`)."""
        years = (("KETU", 7), ("VENUS", 20), ("SUN", 6), ("MOON", 10), ("MARS", 7),
                 ("RAHU", 18), ("JUPITER", 16), ("SATURN", 19), ("MERCURY", 17))
        twin: DashaDefinition = {
            "kernel": "UDU",
            "key": "ACME_VIMSHOTTARI",
            "lords": [{"graha": graha, "years": count} for graha, count in years],
            "reference": "ASHWINI",
        }
        sign_twin: DashaDefinition = {"kernel": "RASHI", "key": "ACME_CHARA"}
        observer = Observer(
            latitude_deg=Latitude(27.7172), longitude_deg=Longitude(85.324), altitude_m=Altitude(1400)
        )
        with self.teistro.context(test_provider=True, dasha_systems=[twin, sign_twin]) as ctx:
            chart = ctx.chart.found(
                instant=2451545.0,
                place=observer,
                utc_offset_seconds=20700,
                dashas=["dasha_system.ACME_VIMSHOTTARI", DashaSystem.VIMSHOTTARI],
            )
            consumer, shipped = chart.dashas
            self.assertEqual(consumer.system, "dasha_system.ACME_VIMSHOTTARI")
            self.assertIs(shipped.system, DashaSystem.VIMSHOTTARI)
            self.assertEqual(consumer.periods, shipped.periods)
            self.assertEqual(consumer.balance, shipped.balance)
            # The other kernel, the same way: a sign-based system of one's
            # own answers as the catalogued row it copies
            # (`03-design/dasha-coverage-measured.md`).
            signs = ctx.chart.found(
                instant=2451545.0,
                place=observer,
                utc_offset_seconds=20700,
                dashas=["dasha_system.ACME_CHARA", DashaSystem.CHARA],
            )
            own, chara = signs.dashas
            self.assertEqual(own.system, "dasha_system.ACME_CHARA")
            self.assertIs(chara.system, DashaSystem.CHARA)
            self.assertEqual(own.periods, chara.periods)
            self.assertIsNone(own.seed)
            with self.assertRaises(TeistroError) as stray:
                ctx.chart.found(
                    instant=2451545.0, place=observer, utc_offset_seconds=20700, dashas=["dasha_system.ACME_OTHER"]
                )
            self.assertEqual(stray.exception.field, "dashas[0]")
        too_narrow: UduDashaDefinition = {
            "kernel": "UDU",
            "key": "ACME_VIMSHOTTARI",
            "lords": [{"graha": graha, "years": count} for graha, count in years],
            "reference": "ASHWINI",
            "span": 0,
        }
        with self.assertRaises(TeistroError) as narrow:
            self.teistro.context(test_provider=True, dasha_systems=[too_narrow])
        self.assertEqual(narrow.exception.field, "options.dashas_json[0].span")
        thirteenth: RashiDashaDefinition = {
            "kernel": "RASHI",
            "key": "ACME_THIRTEEN",
            "stronger_of": [1, 13],
        }
        with self.assertRaises(TeistroError) as houses:
            self.teistro.context(test_provider=True, dasha_systems=[thirteenth])
        self.assertEqual(houses.exception.field, "options.dashas_json[0].stronger_of[1]")
        # A field in another spelling is refused by name, not read as its
        # default.
        camel = {"kernel": "RASHI", "key": "ACME_CAMEL", "namedLord": "FIRST"}
        with self.assertRaises(TeistroError) as misspelt:
            self.teistro.context(test_provider=True, dasha_systems=[camel])  # type: ignore[list-item]
        self.assertEqual(misspelt.exception.field, "options.dashas_json[0].namedLord")

    def test_a_chart_carries_the_annual_charts_its_birth_opens(self) -> None:
        """The annual charts cross: a request's `varsha=` answers each chart's
        returns in year order, ragged per chart, and the instant founds as a
        chart of its own (`03-design/annual-chart.md`)."""
        observer = Observer(
            latitude_deg=Latitude(27.7172), longitude_deg=Longitude(85.324), altitude_m=Altitude(1400)
        )
        birth = 2447995.4895833335
        chart = self.ctx.chart.found(
            instant=birth,
            place=observer,
            utc_offset_seconds=20700,
            varsha={"reading": "SIDEREAL", "through": 12},
        )
        years = chart.praveshas
        self.assertEqual([one.year for one in years], list(range(1, 13)))
        self.assertGreater(years[0].instant, birth)
        # Eleven sidereal years between the first and the twelfth, to a day.
        span = years[11].instant - years[0].instant
        self.assertLess(abs(span - 11 * 365.2564), 1)

        # The Muntha advances one sign a year from the birth lagna and comes
        # home after twelve, which is the whole of its rule.
        natal = self.ctx.chart.found(
            instant=birth, place=observer, utc_offset_seconds=20700
        )
        lagna = int(natal.lagna_deg // 30)
        self.assertEqual(
            [one.muntha.sign for one in years],
            [Rashi((lagna + one.year) % 12) for one in years],
        )
        self.assertEqual(years[11].muntha.sign, Rashi(lagna % 12))
        self.assertTrue(all(one.muntha.lord is not None for one in years))

        # Both readings of the degree give the same sign and part inside it.
        carried = self.ctx.chart.found(
            instant=birth,
            place=observer,
            utc_offset_seconds=20700,
            varsha={"through": 12, "muntha": "NATAL_DEGREE"},
        ).praveshas
        self.assertEqual(
            [one.muntha.sign for one in carried], [one.muntha.sign for one in years]
        )
        for carried_year, opened in zip(carried, years):
            self.assertGreaterEqual(
                carried_year.muntha.longitude_deg, opened.muntha.longitude_deg
            )

        # No place, no chart founded: the instants alone, as before.
        self.assertTrue(all(one.annual is None for one in years))

        # At the birthplace each year's chart comes back with its five
        # office-bearers; the birth lagna's lord is shared by every year and
        # the Muntha's lord is the one already on the return.
        cast = self.ctx.chart.found(
            instant=birth,
            place=observer,
            utc_offset_seconds=20700,
            varsha={"through": 12, "place": "birth"},
        ).praveshas
        charts = [one.annual for one in cast]
        self.assertTrue(all(chart is not None for chart in charts))
        first = charts[0]
        assert first is not None
        for one in cast:
            assert one.annual is not None
            self.assertEqual(
                one.annual.office_bearers.janma_lagna, first.office_bearers.janma_lagna
            )
            self.assertEqual(one.annual.office_bearers.muntha, one.muntha.lord)
        again = self.ctx.chart.found(
            instant=cast[3].instant, place=observer, utc_offset_seconds=20700
        )
        third = cast[3].annual
        assert third is not None
        self.assertEqual(again.lagna_deg, third.lagna_deg)

        # Each year names a lord, chosen among its own claimants.
        for one in cast:
            assert one.annual is not None
            lord = one.annual.year_lord
            self.assertTrue(1 <= len(lord.claims) <= 5)
            # Among its own claimants, unless it succeeds the Moon, whose
            # Ithasala may be with any planet.
            succeeds_the_moon = lord.chosen in (VarsheshaChosen.MOONS_ITHASALA, VarsheshaChosen.MOONS_SIGN_LORD)
            if succeeds_the_moon:
                self.assertTrue(lord.moon_passed_over)
            else:
                self.assertIn(lord.graha, [claim.graha for claim in lord.claims])
            ranked = [claim.vishwa.total for claim in lord.claims]
            self.assertEqual(ranked, sorted(ranked, reverse=True))
            self.assertRegex(str(lord.vishwa), r"^\d\d:\d\d:\d\d$")
            self.assertEqual(lord.vishwa.units, lord.vishwa.total // 3600)
            self.assertIsInstance(lord.moon_passed_over, bool)

        # The pairs that make a yoga: never a neutral aspect, and each of
        # Table X-3's four kinds standing where its own degrees put it.
        kinds = set()
        for one in cast:
            assert one.annual is not None
            for pair in one.annual.yogas:
                self.assertNotEqual(pair.drishti, TajikaDrishti.NONE)
                self.assertGreater(pair.orb_deg, 0)
                kinds.add(pair.yoga)
                if pair.yoga == TajikaYoga.ITHASALA_VARTAMANA:
                    self.assertGreaterEqual(pair.apart_deg, 1)
                if pair.yoga == TajikaYoga.ITHASALA_POORNA:
                    self.assertLess(abs(pair.apart_deg), 1)
                if pair.yoga == TajikaYoga.ISHRAFA:
                    self.assertLessEqual(pair.apart_deg, -1)
        # The corpus's own years reach every kind the boundary can say,
        # so a variant that stopped crossing would be caught here and not
        # only in the enum's member count.
        self.assertEqual(kinds, set(TajikaYoga))

        # At a residence, in the parts `found` takes, the lagnas move.
        delhi = self.ctx.chart.found(
            instant=birth,
            place=observer,
            utc_offset_seconds=20700,
            varsha={
                "through": 12,
                "place": {
                    "observer": Observer(
                        latitude_deg=Latitude(28.6139),
                        longitude_deg=Longitude(77.209),
                        altitude_m=Altitude(216),
                    ),
                    "utc_offset_seconds": 19800,
                },
            },
        ).praveshas
        for there, here in zip(delhi, cast):
            assert there.annual is not None and here.annual is not None
            self.assertGreater(abs(there.annual.lagna_deg - here.annual.lagna_deg), 0.1)
        with self.assertRaises(TeistroError) as wrong:
            self.ctx.chart.found(
                instant=birth,
                place=observer,
                utc_offset_seconds=20700,
                varsha={"through": 12, "place": "home"},  # type: ignore[arg-type]
            )
        self.assertEqual(wrong.exception.field, "varsha.place")

        # The instant founds as a chart of its own; the place is the caller's.
        annual = self.ctx.chart.found(
            instant=years[11].instant, place=observer, utc_offset_seconds=20700
        )
        self.assertEqual(annual.instant, years[11].instant)

        # Not asked for is empty, not zeroes.
        self.assertEqual(
            self.ctx.chart.found(
                instant=birth, place=observer, utc_offset_seconds=20700
            ).praveshas,
            [],
        )

        # The rivals are asked for by name and are not the same instant.
        tropical = self.ctx.chart.found(
            instant=birth,
            place=observer,
            utc_offset_seconds=20700,
            varsha={"reading": "TROPICAL", "through": 12},
        ).praveshas
        self.assertNotEqual(tropical[11].instant, years[11].instant)

        with self.assertRaises(TeistroError) as wide:
            self.ctx.chart.found(
                instant=birth,
                place=observer,
                utc_offset_seconds=20700,
                varsha={"reading": "SIDEREAL", "through": 0},
            )
        self.assertEqual(wide.exception.field, "varsha.through")

    def test_a_years_chart_answers_the_tajika_yogas_for_the_matters_asked(self) -> None:
        """The sixteen Tajika yogas cross for the matters `varsha=` names, in
        its order, each carrying its question, the lords' pair and what it
        could not answer (`03-design/tajika-yogas.md`)."""
        observer = Observer(
            latitude_deg=Latitude(27.7172), longitude_deg=Longitude(85.324), altitude_m=Altitude(1400)
        )

        def years(varsha: Any) -> Any:
            return self.ctx.chart.found(
                instant=2447995.4895833335,
                place=observer,
                utc_offset_seconds=20700,
                varsha=varsha,
            ).praveshas

        for one in years({"through": 6, "place": "birth", "matters": [7, 1]}):
            annual = one.annual
            assert annual is not None
            self.assertEqual([matter.house for matter in annual.matters], [7, 1])
            self.assertTrue(all(isinstance(graha, Graha) for graha in annual.retrograde + annual.combust))
            for matter in annual.matters:
                # All sixteen are built and the façade reads the states, so
                # every one answers True or False.
                self.assertEqual(matter.unanswered, [])
                self.assertIsInstance(matter.holds(YearYoga.KUTTHA), bool)
                self.assertIsInstance(matter.holds(YearYoga.ITHASALA), bool)
                self.assertEqual(matter.karyesha != matter.lagnesha, not matter.same_lord)
                for held in matter.held:
                    if held.between is not None:
                        self.assertEqual(held.between, matter.between)
                    self.assertEqual(
                        held.afflictions is not None,
                        held.yoga in (YearYoga.RUDDA, YearYoga.DURAPHA),
                    )
                pair_yoga = matter.between.yoga if matter.between else None
                self.assertEqual(
                    matter.holds(YearYoga.ITHASALA),
                    pair_yoga is not None and pair_yoga != TajikaYoga.ISHRAFA,
                )
            first = annual.matters[1]
            self.assertTrue(first.same_lord)
            self.assertIsNone(first.between)
            self.assertTrue(
                all(held.yoga in (YearYoga.IKABALA, YearYoga.INDUVARA) for held in first.held)
            )

        every = years({"through": 2, "place": "birth", "matters": "all"})
        assert every[0].annual is not None
        self.assertEqual([m.house for m in every[0].annual.matters], list(range(1, 13)))
        unasked = years({"through": 2, "place": "birth"})
        assert unasked[0].annual is not None
        self.assertEqual(unasked[0].annual.matters, [])

        # The rule records are written in Python's own keys.
        years({"through": 2, "place": "birth", "varshesha": {"none_aspects": "ANNUAL_LAGNA_LORD"}})
        years({
            "through": 2,
            "place": "birth",
            "varshesha": {
                "none_aspects": "STRONGEST",
                "moon": "ITHASALA",
                "moon_partner": "OFFICE_BEARER",
                "drishti": {"sub_degree": "ISHRAFA"},
            },
        })
        years({
            "through": 2,
            "place": "birth",
            "matters": [10],
            "yogas": {"weak_below": 4 * 3600, "strong_from": 12 * 3600, "drishti": {"sub_degree": "ISHRAFA"}},
        })

        # The commentary's full Moon can only take a Kuttha away.
        def kutthas(found: Any) -> int:
            return sum(
                bool(matter.holds(YearYoga.KUTTHA))
                for one in found
                for matter in one.annual.matters
            )

        every = years({"through": 2, "place": "birth", "matters": "all"})
        waxing = years({"through": 2, "place": "birth", "matters": "all", "yogas": {"moon_benefic": "WAXING"}})
        self.assertLessEqual(kutthas(waxing), kutthas(every))

        for varsha, field in [
            ({"through": 2, "matters": [7]}, "varsha.matters"),
            ({"through": 2, "place": "birth", "matters": [7, 7]}, "varsha.matters"),
            ({"through": 2, "place": "birth", "matters": [13]}, "varsha.matters"),
            (
                {"through": 2, "place": "birth", "matters": [7],
                 "yogas": {"weak_below": 12 * 3600, "strong_from": 4 * 3600}},
                "varsha.yogas.strongFrom",
            ),
        ]:
            with self.subTest(field=field, varsha=varsha):
                with self.assertRaises(TeistroError) as refused:
                    years(varsha)
                self.assertEqual(refused.exception.field, field)

    def test_a_years_chart_answers_the_sahams_asked_for(self) -> None:
        """The sahams cross for the ones `varsha=` names, in its order, each
        where it fell and what it fell in (`03-design/tajika-sahams.md`)."""
        observer = Observer(
            latitude_deg=Latitude(27.7172), longitude_deg=Longitude(85.324), altitude_m=Altitude(1400)
        )

        def years(varsha: Any) -> Any:
            return self.ctx.chart.found(
                instant=2447995.4895833335,
                place=observer,
                utc_offset_seconds=20700,
                varsha=varsha,
            ).praveshas

        # A member or its key, as the caller has it.
        asked = years({"through": 4, "place": "birth", "sahams": [Saham.KARYA_SIDDHI, "PUNYA"]})
        for one in asked:
            annual = one.annual
            assert annual is not None
            self.assertEqual([p.saham for p in annual.sahams], [Saham.KARYA_SIDDHI, Saham.PUNYA])
            lagna = int(annual.lagna_deg // 30)
            for point in annual.sahams:
                self.assertTrue(0 <= point.longitude_deg < 360)
                sign = int(point.longitude_deg // 30)
                self.assertEqual(point.sign, Rashi(sign))
                self.assertEqual(point.house, (sign - lagna) % 12 + 1)
                self.assertIsInstance(point.lord, Graha)
                self.assertIsInstance(point.added_sign, bool)

        # "all" is the forty-one in the source's order; unasked is none.
        every = years({"through": 1, "place": "birth", "sahams": "all"})[0].annual
        assert every is not None
        self.assertEqual([p.saham for p in every.sahams], list(Saham))
        # A key read back names the saham again.
        again = years({"through": 1, "place": "birth", "sahams": [p.saham.key for p in every.sahams]})
        self.assertEqual(again[0].annual, every)
        unasked = years({"through": 1, "place": "birth"})[0].annual
        assert unasked is not None
        self.assertEqual(unasked.sahams, [])

        # The rules are written in Python's own keys.
        never = years({"through": 2, "place": "birth", "sahams": "all", "saham_rules": {"add_sign": "NEVER"}})
        self.assertTrue(all(not p.added_sign for one in never for p in one.annual.sahams))
        years({"through": 1, "place": "birth", "sahams": ["MRITYU"], "saham_rules": {"houses": "EQUAL", "roga": "SATURN"}})

        for varsha, field in [
            ({"through": 2, "place": "birth", "sahams": ["PUNYA", Saham.PUNYA]}, "varsha.sahams"),
            ({"through": 2, "place": "birth", "sahams": ["pnya"]}, "varsha.sahams[0]"),
            (
                {"through": 2, "place": "birth", "sahams": ["PUNYA"], "saham_rules": {"add_sgn": "NEVER"}},
                "varsha.sahamRules.addSgn",
            ),
            (
                {"through": 1, "sahams": ["PUNYA"], "saham_strength": {"natures": "vedic"}},
                "varsha.sahamStrength.natures",
            ),
            (
                {"through": 1, "place": "birth", "harsha_rules": {"venus": "sixth"}},
                "varsha.harshaRules.venus",
            ),
        ]:
            with self.subTest(field=field, varsha=varsha):
                with self.assertRaises(TeistroError) as refused:
                    years(varsha)
                self.assertEqual(refused.exception.field, field)

    def test_a_years_chart_answers_the_annual_dashas_asked_for(self) -> None:
        """The annual dashas cross for the ones `varsha=` names, in its
        order: each opens on its return and closes on the next, and its
        periods run end to end (`03-design/annual-dashas.md`)."""
        observer = Observer(
            latitude_deg=Latitude(27.7172), longitude_deg=Longitude(85.324), altitude_m=Altitude(1400)
        )

        def years(varsha: Any) -> Any:
            return self.ctx.chart.found(
                instant=2447995.4895833335,
                place=observer,
                utc_offset_seconds=20700,
                varsha=varsha,
            ).praveshas

        # A member or its key, bare or full, as the caller has it.
        asked = years({"through": 3, "place": "birth", "dashas": [DashaSystem.MUDDA, "dasha_system.PATYAYINI"]})
        for k, one in enumerate(asked):
            annual = one.annual
            assert annual is not None
            self.assertEqual([d.system for d in annual.dashas], [DashaSystem.MUDDA, DashaSystem.PATYAYINI])
            for dasha in annual.dashas:
                self.assertEqual(dasha.year.from_jd, one.instant)
                if k + 1 < len(asked):
                    self.assertLess(abs(dasha.year.to_jd - asked[k + 1].instant), 2e-7)
                self.assertEqual(dasha.first_lord, dasha.ring.shares[dasha.ring.first].lord)
                # The mahadashas run end to end across the year.
                mahas = [p for p in dasha.periods if p.level == 1]
                self.assertEqual(mahas[0].span.from_jd, dasha.year.from_jd)
                self.assertEqual(mahas[-1].span.to_jd, dasha.year.to_jd)
                for before, after in zip(mahas, mahas[1:]):
                    self.assertEqual(after.span.from_jd, before.span.to_jd)
                # Mid-year a mahadasha runs, and one of its own under it.
                chain = dasha.at((dasha.year.from_jd + dasha.year.to_jd) / 2)
                self.assertEqual(len(chain), 2)
                self.assertTrue(chain[1].path.startswith(chain[0].path + "/"))
                self.assertEqual(dasha.at(dasha.year.to_jd), [])
            mudda, patyayini = annual.dashas
            self.assertIsInstance(mudda.seed, Nakshatra)
            self.assertEqual(len(mudda.ring.shares), 9)
            assert mudda.ring.remaining is not None
            self.assertTrue(0 < mudda.ring.remaining <= 1)
            self.assertIsNone(patyayini.seed)
            self.assertIsNone(patyayini.ring.remaining)
            self.assertEqual(sum(share.sign is not None for share in patyayini.ring.shares), 1)
            self.assertTrue(any(p.sign is not None for p in patyayini.periods))
        firsts = [one.annual.dashas[0].ring.first for one in asked]
        self.assertEqual(firsts[1:], [(first + 1) % 9 for first in firsts[:-1]])

        # "all" is the three in the catalogue's order; unasked is none.
        every = years({"through": 1, "place": "birth", "dashas": "all"})[0].annual
        assert every is not None
        self.assertEqual(
            [d.system for d in every.dashas],
            [DashaSystem.PATYAYINI, DashaSystem.MUDDA, DashaSystem.VARSHA_YOGINI],
        )
        # A system read back names it again.
        again = years({"through": 1, "place": "birth", "dashas": [d.system for d in every.dashas]})
        self.assertEqual(again[0].annual.dashas, every.dashas)
        unasked = years({"through": 1, "place": "birth"})[0].annual
        assert unasked is not None
        self.assertEqual(unasked.dashas, [])

        # The rules are written in Python's own keys.
        days = years({
            "through": 1,
            "place": "birth",
            "dashas": ["MUDDA"],
            "dasha_rules": {"clock": {"DAYS": 360}, "depth": 1, "birth_period": "ELAPSED", "measure": "TEMPORAL"},
        })[0].annual.dashas[0]
        self.assertEqual(days.year.to_jd - days.year.from_jd, 360)
        self.assertTrue(all(p.level == 1 for p in days.periods))

        for varsha, field in [
            ({"through": 2, "dashas": [DashaSystem.MUDDA]}, "varsha.dashas"),
            ({"through": 2, "place": "birth", "dashas": [DashaSystem.VIMSHOTTARI]}, "varsha.dashas"),
            ({"through": 2, "place": "birth", "dashas": ["MUDDA", DashaSystem.MUDDA]}, "varsha.dashas"),
            (
                {"through": 2, "place": "birth", "dashas": "all", "dasha_rules": {"clock": {"DAYS": 0}}},
                "varsha.dashaRules.clock",
            ),
            (
                {"through": 2, "place": "birth", "dashas": "all", "dasha_rules": {"birth_perod": "ELAPSED"}},
                "varsha.dashaRules.birthPerod",
            ),
        ]:
            with self.subTest(field=field, varsha=varsha):
                with self.assertRaises(TeistroError) as refused:
                    years(varsha)
                self.assertEqual(refused.exception.field, field)

    def test_a_sahams_strength_the_harsha_bala_and_the_births_sahams_cross(self) -> None:
        """Each saham carries its strength clause by clause, every founded
        year its Harsha bala, and every birth its own sahams, which need no
        place (`03-design/tajika-saham-strength.md`)."""
        observer = Observer(
            latitude_deg=Latitude(27.7172), longitude_deg=Longitude(85.324), altitude_m=Altitude(1400)
        )

        def found(varsha: Any) -> Any:
            return self.ctx.chart.found(
                instant=2447995.4895833335,
                place=observer,
                utc_offset_seconds=20700,
                varsha=varsha,
            )

        chart = found({
            "through": 2,
            "place": "birth",
            "sahams": "all",
            "saham_strength": {"natures": "CHAPTER", "friendship": "POSITIONAL", "weak_below": 5 * 3600},
            "harsha_rules": {"venus": "TWELFTH"},
        })
        for one in chart.praveshas:
            annual = one.annual
            assert annual is not None
            for saham in annual.sahams:
                near = SahamStrong.LORD_CONJOINS in saham.strong or SahamStrong.LORD_ASPECTS_SAHAM in saham.strong
                # The two (c) clauses negate each other: exactly one holds.
                self.assertNotEqual(near, SahamWeak.LORD_APART in saham.weak)
                self.assertEqual(len(saham.seven), 7)
                lord = next(s for s in saham.seven if s.graha == saham.lord)
                self.assertEqual(lord.company, SahamStrong.LORD_CONJOINS in saham.strong)
                self.assertIsInstance(saham.in_node_axis, bool)
                self.assertEqual(saham.handicapped, saham.house in (6, 8, 12))
            self.assertEqual(len(annual.harsha), 7)
            for h in annual.harsha:
                parts = sum([h.sthana, h.uchcha_swakshetra, h.stri_purusha, h.dina_ratri])
                self.assertEqual(h.total, 5 * parts)
        self.assertEqual(len(chart.sahams), 41)
        self.assertTrue(all(SahamStrong.WITH_YEAR_LORD not in one.strong for one in chart.sahams))
        # Without a place, the birth's sahams are what is answered.
        natal = found({"through": 1, "sahams": [Saham.PUNYA]})
        self.assertEqual([one.saham for one in natal.sahams], [Saham.PUNYA])
        self.assertIsNone(natal.praveshas[0].annual)

    def test_a_chart_carries_its_dashas_their_periods_and_the_chain_at_an_instant(self) -> None:
        """A chart's dashas cross whole: the balance, the periods to the
        settings' depth with their paths, and the chain at an instant read
        off them (`03-design/dasha-kernels.md`)."""
        observer = Observer(
            latitude_deg=Latitude(27.7172), longitude_deg=Longitude(85.324), altitude_m=Altitude(1400)
        )
        chart = self.ctx.chart.found(
            instant=2451545.0,
            place=observer,
            utc_offset_seconds=20700,
            dashas=[DashaSystem.VIMSHOTTARI, DashaSystem.CHARA],
        )
        self.assertEqual(
            self.ctx.chart.found(instant=2451545.0, place=observer, utc_offset_seconds=20700).dashas, []
        )
        dasha, chara = chart.dashas
        self.assertIs(dasha.system, DashaSystem.VIMSHOTTARI)
        assert dasha.balance is not None
        self.assertIs(dasha.balance.method, Balance.SPATIAL)
        self.assertTrue(0 < dasha.balance.remaining <= 1)
        self.assertIsNone(dasha.moon_span)
        self.assertEqual(dasha.depth, 3)
        self.assertEqual(len(dasha.periods), 9 + 81 + 729)
        first, second = dasha.periods[:2]
        self.assertEqual((first.path, first.level, first.span.from_jd), ("0", 1, 2451545.0))
        self.assertIs(first.lord, dasha.first_lord)
        self.assertEqual((second.path, second.level, second.lord), ("0/0", 2, dasha.first_lord))
        self.assertEqual(dasha.periods[-1].path, "8/8/8")
        self.assertIsNone(first.sign, "a nakshatra-seeded period is its lord's")

        # A sign-based dasha: no seed, no balance, twelve signs each divided
        # in twelve from its own sign.
        self.assertIs(chara.system, DashaSystem.CHARA)
        self.assertEqual((chara.seed, chara.balance), (None, None))
        self.assertEqual(len(chara.periods), 12 + 144 + 1728)
        maha, own = chara.periods[:2]
        self.assertEqual((maha.path, own.path, own.sign, maha.span.from_jd), ("0", "0/0", maha.sign, 2451545.0))
        self.assertIsInstance(maha.sign, Rashi)
        self.assertIs(chara.first_lord, maha.lord)
        self.assertEqual(len({p.sign for p in chara.periods if p.level == 1}), 12, "every sign once")
        self.assertEqual(len(chara.at(2451545.0 + 5000)), 3)

        instant = 2451545.0 + 5000
        chain = dasha.at(instant)
        self.assertEqual([period.level for period in chain], [1, 2, 3])
        self.assertTrue(all(p.span.from_jd <= instant < p.span.to_jd for p in chain))
        self.assertEqual(dasha.at(2451544.0), [], "before birth")

        with self.assertRaises(TeistroError) as caught:
            self.ctx.chart.found(
                instant=2451545.0,
                place=observer,
                utc_offset_seconds=0,
                dashas=[DashaSystem.VIMSHOTTARI, DashaSystem.SUDARSHANA_CHAKRA],
            )
        self.assertEqual(caught.exception.field, "dashas[1]")

    def test_a_layout_of_your_own_is_registered_drawn_by_its_key_and_refused_by_its_field(self) -> None:
        """A shipped row copied, renamed and registered, drawn by its key,
        and a wrong row refused by its place and field
        (`03-design/chart-geometry.md` §7f)."""
        row = self.ctx.chart.layout("SOUTH_INDIAN")
        self.assertEqual(self.ctx.chart.layout(ChartLayout.SOUTH_INDIAN), row, "bare or full")
        self.assertEqual((row["shape"]["kind"], row["shape"]["direction"]), ("GRID", "CLOCKWISE"))
        with self.assertRaises(TeistroError) as unknown:
            self.ctx.chart.layout("ACME_KERALA")
        self.assertEqual(unknown.exception.field, "key")

        kerala: LayoutRow = {**row, "key": "ACME_KERALA"}
        observer = Observer(
            latitude_deg=Latitude(27.7172), longitude_deg=Longitude(85.324), altitude_m=Altitude(1400)
        )
        with self.teistro.context(profile=PROFILE, test_provider=True, layouts=[kerala]) as ctx:
            self.assertEqual(ctx.chart.layout("chart_layout.ACME_KERALA"), kerala)
            self.assertEqual(ctx.keys.name(ctx.keys.id("chart_layout.ACME_KERALA")), "chart_layout.ACME_KERALA")
            south, own = ctx.chart.found(
                instant=2451545.0,
                place=observer,
                utc_offset_seconds=20700,
                drawings=[(ChartLayout.SOUTH_INDIAN, Varga.D1), ("chart_layout.ACME_KERALA", Varga.D1)],
            ).drawings
            self.assertEqual(own.layout, "chart_layout.ACME_KERALA")
            self.assertEqual(south.layout, ChartLayout.SOUTH_INDIAN)
            self.assertEqual(own.cells, south.cells, "the same row draws the same chart")
            with self.assertRaises(TeistroError) as unregistered:
                ctx.chart.found(
                    instant=2451545.0,
                    place=observer,
                    utc_offset_seconds=0,
                    drawings=[("chart_layout.ACME_ODIA", Varga.D1)],
                )
            self.assertEqual(unregistered.exception.field, "drawings[0]")

        def refused(layouts: list[LayoutRow]) -> Optional[str]:
            with self.assertRaises(TeistroError) as caught:
                self.teistro.context(test_provider=True, layouts=layouts)
            return caught.exception.field

        self.assertEqual(refused([kerala, row]), "options.layouts_json[1].key")
        misspelt = {**kerala, "shape": {**kerala["shape"], "heading": "CLOCKWISE"}}
        self.assertEqual(refused([misspelt]), "options.layouts_json[0].shape.heading")  # type: ignore[list-item]


class ASpansTurns(WithLibrary):
    """A limb's member naming two days or none, and its end in ghatis
    (`03-design/nepal-day-measured.md`): Nepal's print, under the
    committee's Surya Siddhanta."""

    def tithis(self, y: int, m: int, d: int) -> list[Any]:
        """The day's tithi spans, read while the context is open."""
        with self.teistro.context(
            profile="nepali-committee", ephemeris=Ephemeris.SURYA_SIDDHANTA
        ) as ctx:
            days = ctx.almanac.of(
                from_date=date(Calendar.GREGORIAN, y, m, d),
                to_date=date(Calendar.GREGORIAN, y, m, d),
                place=Observer(
                    latitude_deg=Latitude(27.7172),
                    longitude_deg=Longitude(85.324),
                    altitude_m=Altitude(1400),
                ),
                utc_offset_seconds=20_700,
            )
            return list(days[0].tithi)

    def test_a_tithi_printed_day_and_night_holds_both_sunrises(self) -> None:
        tithis = self.tithis(2025, 4, 13)
        self.assertEqual(
            [span.member for span in tithis if span.sunrises is Sunrises.BOTH],
            [Tithi.KRISHNA_PRATIPADA],
        )

    def test_a_tithi_between_two_sunrises_holds_neither(self) -> None:
        tithis = self.tithis(2025, 4, 26)
        self.assertEqual(
            [span.sunrises for span in tithis],
            [Sunrises.OPENING, Sunrises.NEITHER, Sunrises.NEXT],
        )
        self.assertEqual(tithis[1].member, Tithi.KRISHNA_CHATURDASHI)

    def test_an_end_reads_in_ghatis_from_sunrise(self) -> None:
        # Chaturdashi ended at 22:16, sunrise 05:54: past 40 ghatis.
        ends = self.tithis(2026, 9, 25)[0].ends
        self.assertIn(ends.ghati, (40, 41))
        self.assertLess(max(ends.pala, ends.vipala), 60)


class ADaysSeason(WithLibrary):
    """A day's season is its solar month's, and a month begins on the day
    Nepal's calendar begins it (`03-design/ritu-measured.md`)."""

    def seasons(self, settings: Optional[dict[str, object]] = None) -> list[Ritu]:
        with self.teistro.context(
            profile=PROFILE, ephemeris=Ephemeris.BUILTIN, settings=settings
        ) as ctx:
            days = ctx.almanac.of(
                from_date=date(Calendar.GREGORIAN, 2026, 3, 13),
                to_date=date(Calendar.GREGORIAN, 2026, 3, 16),
                place=Observer(
                    latitude_deg=Latitude(27.7172),
                    longitude_deg=Longitude(85.324),
                    altitude_m=Altitude(1400),
                ),
                utc_offset_seconds=20_700,
            )
            return [day.ritu for day in days]

    def test_the_season_turns_on_the_first_of_chaitra(self) -> None:
        # 15 March 2026 is 1 Chaitra 2082: Vasanta from that day.
        self.assertEqual(
            self.seasons(),
            [Ritu.SHISHIRA, Ritu.SHISHIRA, Ritu.VASANTA, Ritu.VASANTA],
        )

    def test_the_lunar_month_names_its_own_season(self) -> None:
        # Amanta Phalguna runs to the new moon of 19 March: Shishira.
        self.assertEqual(
            self.seasons({"panchanga": {"ritu": "LUNAR"}}),
            [Ritu.SHISHIRA] * 4,
        )


class AnAlmanacDay(WithLibrary):
    """The day's own columns, read through the accessors that decode them.

    Every one of these had never run from Python: the Node and Dart
    bindings exercise their own decoders and this one's examples print
    the limbs and the periods, leaving the rest of a day — its ayana, its
    Brahma muhurta, the arcs and the signs the luminaries stood in —
    decoded by nothing (`03-design/binding-exercise-measured.md`).
    """

    def setUp(self) -> None:
        self.ctx = self.teistro.context(
            profile=PROFILE, locale=LOCALE, test_provider=True
        )
        self.week = self.ctx.almanac.of(
            from_date=date(Calendar.GREGORIAN, 2024, 6, 17),
            to_date=date(Calendar.GREGORIAN, 2024, 6, 19),
            place=Observer(
                latitude_deg=Latitude(27.7172),
                longitude_deg=Longitude(85.324),
                altitude_m=Altitude(1400),
            ),
            utc_offset_seconds=20_700,
        )

    def tearDown(self) -> None:
        self.ctx.close()

    def test_a_batch_carries_the_place_and_each_day_its_window(self) -> None:
        self.assertAlmostEqual(float(self.week.place.latitude_deg), 27.7172, places=4)
        # The window is the day's, not the batch's: it is what that day's
        # spans are clipped to.
        window = self.week.at(0).window
        self.assertLess(window.from_jd, window.to_jd)
        # A per-day list's rows are found by the range the batch keeps.
        start, end = self.week.range("kaalas", 0)
        self.assertLessEqual(start, end)

    def test_every_day_decodes_the_columns_nothing_else_reads(self) -> None:
        for day in self.week:
            # Which half of the year, and where not to travel.
            self.assertTrue(day.ayana.full_key.startswith("ayana."))
            # Mid-June: the Sun in Gemini, Grishma by every reading.
            self.assertEqual(day.ritu, Ritu.GRISHMA)
            self.assertTrue(day.disha_shool.full_key.startswith("direction."))
            # The signs the luminaries stood in: two only on a sankranti.
            self.assertGreaterEqual(len(day.moon_signs), 1)
            self.assertGreaterEqual(len(day.sun_signs), 1)
            # Lists that are empty on most days and must still decode.
            self.assertIsInstance(day.panchaka, list)
            self.assertIsInstance(day.muhurta_yogas, list)
            # Brahma muhurta is absent only when the night before is not
            # known, which is the polar case and not this one.
            brahma = day.brahma
            self.assertIsNotNone(brahma)
            assert brahma is not None
            self.assertLess(brahma.from_jd, brahma.to_jd)

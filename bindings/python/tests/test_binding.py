"""The whole surface, end to end, against the real library.

Everything a consumer touches goes through the hand-written layer, which
is thin: what is really being tested is that the generated marshalling
puts a value into the C struct the library expects and reads the answer
back out of the one it filled.
"""

from __future__ import annotations

import json
import unittest

from teistro import (
    Body,
    Calendar,
    Scale,
    Status,
    Teistro,
    TeistroError,
    TimeScale,
    at,
    date,
    fixed_zone,
    iana_zone,
    intl,
    local_mean_zone,
    when_unknown,
)
from teistro._ffi import Longitude
from tests.support import LOCALE, PROFILE, WithLibrary


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
        self.assertEqual(converted.delta_t_source.key, "leap-seconds")
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
        provenance = sky.provenance_of
        self.assertEqual(provenance["profile"], PROFILE)
        self.assertEqual(provenance["settings_hash"], self.ctx.settings_hash)
        self.assertIsInstance(sky.steps_applied, list)
        self.assertEqual(
            sky.frame(self.teistro).centre, self.teistro.canonical_frame.centre
        )

    def test_a_context_with_no_ephemeris_refuses_by_capability(self) -> None:
        with self.teistro.context(profile=PROFILE) as bare:
            with self.assertRaises(TeistroError) as caught:
                bare.positions(instants=[2451545.0], bodies=[Body.SUN])
            self.assertEqual(caught.exception.status, Status.CAPABILITY)

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
                "time-unknown-fallback", [w.key for w in resolved.warnings]
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


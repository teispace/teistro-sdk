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
from typing import Optional

from teistro import (
    Altitude,
    AvasthaSayanadi,
    DashaDefinition,
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
    Observer,
    Plugin,
    Rashi,
    Scale,
    Shodhana,
    Status,
    Teistro,
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
    when_unknown,
)
from teistro._ffi import Longitude
from teistro.catalogue import DayPart
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
            # The field and the hint name the option this binding sets,
            # not the C entry point it has no access to -- the same pair
            # Node, Dart and C are held to.
            self.assertEqual(caught.exception.field, "ephemeris")
            self.assertIn("builtin", caught.exception.hint or "")

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
        # The context is sidereal, so an ayanamsha was applied.
        self.assertNotEqual(chart.ayanamsha_offset_deg, 0.0)

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
        north, wheel = found("dark")
        assert north.svg is not None and wheel.svg is not None
        self.assertTrue(north.svg.startswith('<svg xmlns="http://www.w3.org/2000/svg"'))
        self.assertIn('data-body="graha.SUN">सू', north.svg)
        self.assertIn('fill="#121212"', north.svg)
        self.assertIn("<line ", wheel.svg)

        glyphs = found({"extends": "light", "style": {"size": 600}, "content": {"body_form": "glyph"}})[0].svg
        assert glyphs is not None
        self.assertIn('viewBox="0 0 600 600"', glyphs)
        self.assertIn('data-body="graha.SUN">☉', glyphs)

        with self.assertRaises(TeistroError) as wrong:
            found({"style": {"ink": "black"}})
        self.assertEqual(wrong.exception.field, "theme_json.style.ink")
        with self.assertRaises(TeistroError) as unknown:
            found("sepia")  # type: ignore[arg-type]
        self.assertEqual(unknown.exception.field, "theme_json.extends")

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
        answered = found({"shipped": ["nabhasas"], "longevity": True})
        assert answered is not None
        present = answered["present"]
        self.assertTrue(present)
        for held in present:
            self.assertIsInstance(held["rule"], str)
            self.assertIs(held["result"]["present"], True)
        self.assertIsInstance(answered["longevity"]["ayurdaya"]["pindayu"]["years"], float)

        mine = {"key": "MINE", "category": "raja", "source": {"text": "BPHS"},
                "conditions": [{"type": "rule", "key": present[0]["rule"]}]}
        with_mine = found({"shipped": ["nabhasas"], "rules": [mine]})
        assert with_mine is not None
        self.assertTrue(any(held["rule"] == "MINE" for held in with_mine["present"]))

        with self.assertRaises(TeistroError) as wrong:
            found({"rules": [{"key": "X", "category": "raja"}]})
        self.assertEqual(wrong.exception.field, "rules_json.rules[0]")

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
            },
            {"shipped": ["nabhasas"]},
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
        self.assertEqual(alone.exception.field, "interpret_json.readings")
        # And a composer that is not one is refused beside the ones that are.
        with self.assertRaises(TeistroError) as typo:
            found({"readigns": True})  # type: ignore[arg-type]
        self.assertEqual(typo.exception.field, "interpret_json")

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

    def test_a_consumer_dasha_system_registers_is_asked_for_by_key_and_reads_as_its_twin(self) -> None:
        """A consumer's own dasha system crosses: registered on the context,
        asked for by its key, named by it in the answer, and every period its
        catalogued twin's; a definition the checks refuse is named by its place
        and field (`03-design/dasha-kernels.md`)."""
        years = (("KETU", 7), ("VENUS", 20), ("SUN", 6), ("MOON", 10), ("MARS", 7),
                 ("RAHU", 18), ("JUPITER", 16), ("SATURN", 19), ("MERCURY", 17))
        twin: DashaDefinition = {
            "key": "ACME_VIMSHOTTARI",
            "lords": [{"graha": graha, "years": count} for graha, count in years],
            "reference": "ASHWINI",
        }
        observer = Observer(
            latitude_deg=Latitude(27.7172), longitude_deg=Longitude(85.324), altitude_m=Altitude(1400)
        )
        with self.teistro.context(test_provider=True, dasha_systems=[twin]) as ctx:
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
            with self.assertRaises(TeistroError) as stray:
                ctx.chart.found(
                    instant=2451545.0, place=observer, utc_offset_seconds=20700, dashas=["dasha_system.ACME_OTHER"]
                )
            self.assertEqual(stray.exception.field, "dashas[0]")
        with self.assertRaises(TeistroError) as narrow:
            self.teistro.context(test_provider=True, dasha_systems=[{**twin, "span": 0}])
        self.assertEqual(narrow.exception.field, "options.dashas_json[0].span")

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
        misspelt = {**kerala, "shape": {**kerala["shape"], "heading": "clockwise"}}
        self.assertEqual(refused([misspelt]), "options.layouts_json[0].shape.heading")  # type: ignore[list-item]


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

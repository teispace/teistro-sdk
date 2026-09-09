"""One scenario through the Python binding, printed as the parity report.

`key<TAB>value` lines, sorted by key. `cargo xtask check-parity` runs
this, `bindings/node/parity.mjs` and `bindings/dart/bin/parity.dart` and
compares what they print, so a difference between the bindings' layers is
a failed gate rather than something a reader has to notice.

Every value is what this binding's own surface gives: an enum as the key
it spells, a number formatted to nine decimals, a JSON section as its
length and its FNV-1a hash, because the point is that the bindings agree,
not that they agree with a literal written here.
"""

from __future__ import annotations

import sys
from typing import Any

import json

from teistro import (
    Altitude,
    Body,
    Calendar,
    ChartKind,
    Latitude,
    Longitude,
    Observer,
    Scale,
    Teistro,
    TeistroError,
    at,
    date,
    iana_zone,
    intl,
)

report: dict[str, str] = {}


def number(value: float | int) -> str:
    """A number as every binding spells it: nine decimals, never an
    exponent, and an integer value written plainly."""
    if isinstance(value, int):
        return str(value)
    if value == int(value) and abs(value) < 1e15:
        return str(int(value))
    return f"{value:.9f}"


def fnv(text: str) -> str:
    """FNV-1a over UTF-8 bytes, so a JSON section can be compared without
    a parser."""
    digest = 0x811C9DC5
    for byte in text.encode("utf-8"):
        digest = ((digest ^ byte) * 0x01000193) & 0xFFFFFFFF
    return f"{digest:08x}"


def put(key: str, value: Any) -> None:
    if isinstance(value, bool):
        report[key] = "true" if value else "false"
    elif isinstance(value, (int, float)):
        report[key] = number(value)
    elif value is None:
        report[key] = "null"
    else:
        report[key] = str(value)


def main() -> None:
    teistro = Teistro.open()

    # ── The library itself ────────────────────────────────────────────
    put("abi", teistro.abi)
    put("sdk", teistro.version)
    put("catalogue-version", teistro.catalogue)
    put("default-profile", teistro.default_profile_id)
    put("build-sdk", teistro.build.sdk)
    put("build-abi", teistro.build.abi)
    put("build-catalogue", teistro.build.catalogue)
    put("build-commit", teistro.build.commit)
    put("build-dirty", teistro.build.dirty)
    put("build-target", teistro.build.target)

    # ── A context ─────────────────────────────────────────────────────
    ctx = teistro.context(
        profile="nepali-default", locale="ne-Deva-NP", test_provider=True
    )
    put("profile", ctx.profile)
    put("locale", ctx.locale)
    put("settings-hash", ctx.settings_hash)
    put("settings-fnv", fnv(ctx.settings_json))

    # ── The calendars ─────────────────────────────────────────────────
    day = date(Calendar.GREGORIAN, 2015, 4, 14)
    bs = ctx.convert(day, Calendar.BIKRAM_SAMBAT)
    put("bs-year", bs.year)
    put("bs-month", bs.month)
    put("bs-day", bs.day)
    put("bs-era", None if bs.era is None else bs.era.full_key)
    put("bs-era-year", bs.era_year)
    put("bs-resolution", bs.resolution.key)
    fixed = ctx.fixed_of(day)
    put("fixed", fixed)
    put("weekday", ctx.weekday_of(day))
    put("month-length", ctx.month_length(Calendar.GREGORIAN, 2024, 2))
    put("is-leap", ctx.is_leap(Calendar.GREGORIAN, 2024))
    put("jd-of-fixed", teistro.julian_day_of_fixed(fixed))
    back, fraction = teistro.fixed_of_julian_day(2457126.75)
    put("fixed-of-jd", back)
    put("fraction-of-jd", fraction)

    # ── Time ──────────────────────────────────────────────────────────
    civil = at(date(Calendar.GREGORIAN, 1986, 1, 1), hour=0, minute=20)
    zone = iana_zone("Asia/Kathmandu")
    resolved = ctx.resolve(civil, zone)
    put("resolve-jd", resolved.instant_jd_utc)
    put("resolve-offset", resolved.offset_seconds)
    put("resolve-era", resolved.era.key)
    put("resolve-source", resolved.source.key)
    put("resolve-time-known", resolved.time_known)
    put("resolve-tzdb", resolved.tzdb_version)
    put("resolve-warnings", len(resolved.warnings))
    civil_back, resolution = ctx.civil_of(
        resolved.instant_jd_utc, zone, Calendar.GREGORIAN
    )
    put("civil-year", civil_back.date.year)
    put("civil-minute", civil_back.time.minute)
    put("civil-offset", resolution.offset_seconds)
    tt = ctx.convert_time(2451544.5, Scale.UTC, Scale.TT)
    put("tt-jd", tt.jd)
    put("tt-delta-t", tt.delta_t_seconds)
    put("tt-delta-t-source", tt.delta_t_source.key)
    put("tt-delta-t-model", tt.delta_t_model)
    delta = ctx.delta_t(2451544.5)
    put("delta-t-seconds", delta.seconds)
    put("delta-t-source", delta.source.key)

    # ── Keys ──────────────────────────────────────────────────────────
    identifier = ctx.key_id("graha.SUN")
    put("key-id", identifier)
    put("key-name", ctx.key_name(identifier))
    try:
        ctx.key_id("graha.SUNN")
        put("refusal", "none")
    except TeistroError as error:
        put("refusal-status", error.status.key)
        put("refusal-detail", error.detail)
        put("refusal-hint-names-sun", "SUN" in error.hint)

    # ── The locale engine ─────────────────────────────────────────────
    rendered = ctx.render(
        "sdk.reason.grahaInBhava",
        {"graha": {"$entity": "graha.JUPITER"}, "bhava": 7},
    )
    put("render-fnv", fnv(rendered.text))
    put("render-length", len(rendered.text))
    put("render-resolved-from", rendered.resolved_from)
    put("render-fallback", bool(rendered.is_fallback))
    put("has-message", ctx.has("sdk.reason.grahaInBhava"))
    put("has-missing-message", ctx.has("sdk.nope.missing"))
    put("transliterated", ctx.transliterate("सूर्य बृहस्पति"))
    sun = ctx.entity("graha.SUN")
    put("entity-sun-name", sun.name)
    put("entity-sun-iast", sun.iast)
    put("entity-sun-glyph", sun.glyph)
    put("entity-sun-gender", None if sun.gender is None else sun.gender.value)
    put(
        "message-graha-in-bhava",
        ctx.messages.sdk.reason.graha_in_bhava(graha=intl.GrahaKey.JUPITER, bhava=7),
    )
    put(
        "message-bs-date",
        ctx.messages.sdk.calendar.bikram_sambat.date.long(
            day=1, month_name="बैशाख", year=2072
        ),
    )

    # ── Positions ─────────────────────────────────────────────────────
    frame = teistro.canonical_frame
    put("frame-centre", frame.centre.key)
    put("frame-coordinates", frame.coordinates.key)
    put("frame-bits", teistro.pack_frame(frame))
    put(
        "frame-round-trip",
        teistro.unpack_frame(teistro.pack_frame(frame)).centre == frame.centre,
    )
    sky = ctx.positions(
        instants=[2451545.0, 2451546.0],
        bodies=[Body.SUN, Body.MOON, Body.MARS],
    )
    put("cells", sky.cell_count)
    put("positions-scale", sky.time_scale.key)
    put("positions-bodies", ",".join(body.key for body in sky.body_keys))
    for index in range(sky.cell_count):
        cell = sky.at(index // sky.body_count, index % sky.body_count)
        put(f"cell-{index}-lon", cell.longitude)
        put(f"cell-{index}-lat", cell.latitude)
        put(f"cell-{index}-dist", cell.distance)
        put(f"cell-{index}-lon-speed", cell.longitude_speed)
        put(f"cell-{index}-status", cell.status)
    put(
        "steps",
        ",".join(f"{step['name']}:{step['implementation']}" for step in sky.steps_applied),
    )
    put("provenance-fnv", fnv(sky.provenance))
    put("provenance-profile", sky.provenance_of["profile"])
    put("provenance-settings-hash", sky.provenance_of["settings_hash"])
    put("provenance-provider-frame", sky.provenance_of["provider"]["frame"])

    # ── A chart and an almanac, under a geocentric profile ────────────
    # The scenario above runs under `nepali-default`, whose frame is
    # **topocentric**, and a chart cannot be founded under it at all: the
    # completion's centre step is Phase 3's, so the provider is asked for
    # a frame it does not answer. That refusal is itself worth comparing
    # — the three bindings must fail the same way — and everything after
    # it needs a second context on the SDK's own default profile, which
    # is geocentric.
    place = Observer(
        latitude_deg=Latitude(27.7172),
        longitude_deg=Longitude(85.324),
        altitude_m=Altitude(1400),
    )
    try:
        ctx.found(instant=2451545, place=place, utc_offset_seconds=20700)
        put("chart-under-topocentric", "founded")
    except TeistroError as error:
        put("chart-under-topocentric", error.status.key)

    with teistro.context(
        profile="parashari-classical", locale="ne-Deva-NP", test_provider=True
    ) as geo:
        put("geo-profile", geo.profile)
        put("geo-settings-hash", geo.settings_hash)

        # ── Charts ────────────────────────────────────────────────────
        # Two instants, so a per-chart section that ran charts-outermost
        # the wrong way round shows up as the second chart's values in
        # the first's place rather than as nothing at all.
        charts = geo.found_many(
            instants=[2460482.5, 2460600.25],
            place=place,
            utc_offset_seconds=20700,
        )
        put("chart-count", len(charts))
        put("chart-kind", charts.kind.full_key)
        put("chart-place-lat", charts.place.latitude_deg)
        put("chart-place-lon", charts.place.longitude_deg)
        put("chart-model-fnv", fnv(charts.model))
        put("chart-steps", ",".join(charts.steps_applied))
        put("chart-provenance-fnv", fnv(charts.provenance))
        put("chart-provenance-profile", json.loads(charts.provenance)["profile"])
        put("chart-graha-count", charts.decoded.graha_count)

        for chart in charts:
            i = chart.index
            chart_columns = charts.decoded
            put(f"chart-{i}-instant", chart.instant)
            put(f"chart-{i}-lagna", chart.lagna_deg)
            put(f"chart-{i}-day-lagna", chart.day_lagna_deg)
            put(f"chart-{i}-ayanamsha", chart.ayanamsha_offset_deg)
            put(f"chart-{i}-day-part", chart.day_part.key)
            put(f"chart-{i}-day-elapsed", chart.day_elapsed)
            put(f"chart-{i}-vara", chart.vara.full_key)
            put(f"chart-{i}-sunrise", chart_columns.day.sunrise[i])
            put(f"chart-{i}-sunset", chart_columns.day.sunset[i])
            put(
                f"chart-{i}-date",
                f"{chart_columns.day.year[i]}-{chart_columns.day.month[i]}"
                f"-{chart_columns.day.day_of_month[i]}",
            )
            put(f"chart-{i}-ghati", chart_columns.timing.ghati[i])
            put(f"chart-{i}-pala", chart_columns.timing.pala[i])
            put(f"chart-{i}-vipala", chart_columns.timing.vipala[i])
            put(f"chart-{i}-hora-number", chart_columns.timing.hora_number[i])
            put(f"chart-{i}-hora-lord", chart.hora_lord.full_key)
            for j, graha in enumerate(chart.grahas):
                put(f"chart-{i}-graha-{j}", graha.graha.full_key)
                put(f"chart-{i}-graha-{j}-lon", graha.longitude_deg)
                put(f"chart-{i}-graha-{j}-lat", graha.latitude_deg)
                put(f"chart-{i}-graha-{j}-speed", graha.speed_deg_per_day)
                put(f"chart-{i}-graha-{j}-retro", graha.retrograde)
                put(f"chart-{i}-graha-{j}-house", graha.house.bhava)
                put(f"chart-{i}-graha-{j}-house-method", graha.house.method.full_key)
                put(f"chart-{i}-graha-{j}-placement", graha.placement.bhava)
            for k, bhava in enumerate(chart.houses):
                put(f"chart-{i}-house-{k}-madhya", bhava.madhya_deg)
                put(f"chart-{i}-house-{k}-sandhi", bhava.sandhi_deg)
            for k, bhava in enumerate(chart.chalit):
                put(f"chart-{i}-chalit-{k}-madhya", bhava.madhya_deg)

        # `found` is the batch of one unwrapped, and must agree with it.
        single = geo.found(
            instant=2460482.5, place=place, utc_offset_seconds=20700
        )
        put("chart-single-lagna", single.lagna_deg)
        put("chart-single-agrees", single.lagna_deg == charts.at(0).lagna_deg)

        # ── An almanac ────────────────────────────────────────────────
        # Three days, because a day's lists are ragged and two
        # consecutive days with the same counts would not exercise the
        # offsets.
        week = geo.almanac(
            from_date=date(Calendar.GREGORIAN, 2024, 6, 17),
            to_date=date(Calendar.GREGORIAN, 2024, 6, 19),
            place=place,
            utc_offset_seconds=20700,
        )
        put("almanac-days", len(week))
        put("almanac-calendar", week.calendar.full_key)
        put("almanac-place-lat", week.decoded.latitude_deg)
        put("almanac-model-fnv", fnv(week.model))
        put("almanac-provenance-fnv", fnv(week.decoded.provenance))

        for almanac_day in week:
            i = almanac_day.index
            day_columns = week.decoded
            put(f"day-{i}-vara", almanac_day.vara.full_key)
            put(f"day-{i}-sunrise", almanac_day.sunrise)
            put(f"day-{i}-sunset", almanac_day.sunset)
            put(f"day-{i}-next-sunrise", day_columns.day.next_sunrise[i])
            put(
                f"day-{i}-date",
                f"{day_columns.day.year[i]}-{day_columns.day.month[i]}"
                f"-{day_columns.day.day_of_month[i]}",
            )
            put(f"day-{i}-window-from", almanac_day.window.from_jd)
            put(f"day-{i}-window-to", almanac_day.window.to_jd)
            put(f"day-{i}-month", almanac_day.month.month.full_key)
            put(f"day-{i}-amanta", almanac_day.month.amanta.full_key)
            put(f"day-{i}-purnimanta", almanac_day.month.purnimanta.full_key)
            put(f"day-{i}-paksha", almanac_day.month.paksha.full_key)
            put(f"day-{i}-convention", almanac_day.month.convention.key)
            put(f"day-{i}-month-kind", almanac_day.month.kind.key)
            put(f"day-{i}-ayana", almanac_day.ayana.full_key)
            put(f"day-{i}-disha-shool", almanac_day.disha_shool.full_key)
            # An absent value must be absent in all three, not nought in
            # one.
            put(
                f"day-{i}-sankranti",
                "none" if almanac_day.sankranti is None else number(almanac_day.sankranti),
            )
            abhijit = almanac_day.abhijit
            put(
                f"day-{i}-abhijit",
                "none" if abhijit is None else number(abhijit.at.from_jd),
            )
            put(
                f"day-{i}-abhijit-effective",
                "none" if abhijit is None else ("true" if abhijit.effective else "false"),
            )
            brahma = almanac_day.brahma
            put(
                f"day-{i}-brahma",
                "none" if brahma is None else number(brahma.from_jd),
            )
            # The counts are what the ragged layout turns on: if a
            # binding's prefix sum were off by a day, these would still
            # agree and the spans below would not.
            put(f"day-{i}-tithi-count", len(almanac_day.tithi))
            put(f"day-{i}-nakshatra-count", len(almanac_day.nakshatra))
            put(f"day-{i}-yoga-count", len(almanac_day.yoga))
            put(f"day-{i}-karana-count", len(almanac_day.karana))
            put(f"day-{i}-kaala-count", len(almanac_day.kaalas))
            put(f"day-{i}-choghadiya-count", len(almanac_day.choghadiya))
            put(f"day-{i}-hora-count", len(almanac_day.horas))
            put(f"day-{i}-muhurta-count", len(almanac_day.muhurtas))
            put(f"day-{i}-moon-event-count", len(almanac_day.moon_events))
            put(f"day-{i}-panchaka-count", len(almanac_day.panchaka))
            put(f"day-{i}-moon-sign-count", len(almanac_day.moon_signs))
            put(f"day-{i}-sun-sign-count", len(almanac_day.sun_signs))
            put(f"day-{i}-muhurta-yoga-count", len(almanac_day.muhurta_yogas))
            limbs: list[tuple[str, list[Any]]] = [
                ("tithi", list(almanac_day.tithi)),
                ("nakshatra", list(almanac_day.nakshatra)),
                ("yoga", list(almanac_day.yoga)),
                ("karana", list(almanac_day.karana)),
            ]
            for name, spans in limbs:
                for j, span in enumerate(spans):
                    put(f"day-{i}-{name}-{j}", span.member.full_key)
                    put(f"day-{i}-{name}-{j}-whole-from", span.whole.from_jd)
                    put(f"day-{i}-{name}-{j}-inside-to", span.inside.to_jd)
            for j, kaala in enumerate(almanac_day.kaalas):
                put(f"day-{i}-kaala-{j}", kaala.kaala.full_key)
                put(f"day-{i}-kaala-{j}-from", kaala.at.from_jd)
            horas = almanac_day.horas
            put(f"day-{i}-hora-0-lord", horas[0].lord.full_key)
            put(f"day-{i}-hora-0-start", horas[0].start)
            put(f"day-{i}-hora-23-lord", horas[-1].lord.full_key)
            choghadiya = almanac_day.choghadiya
            put(f"day-{i}-choghadiya-0", choghadiya[0].choghadiya.full_key)
            put(f"day-{i}-choghadiya-0-daytime", choghadiya[0].daytime)
            muhurtas = almanac_day.muhurtas
            put(f"day-{i}-muhurta-0-from", muhurtas[0].at.from_jd)
            put(f"day-{i}-muhurta-last-daylight", muhurtas[-1].daylight)
            for j, event in enumerate(almanac_day.moon_events):
                put(f"day-{i}-moon-{j}-kind", "rise" if event.rise else "set")
                put(f"day-{i}-moon-{j}-instant", event.instant)
            for j, held in enumerate(almanac_day.muhurta_yogas):
                put(f"day-{i}-yoga-held-{j}", held.yoga.full_key)
                put(
                    f"day-{i}-yoga-held-{j}-cause",
                    "vara-nakshatra"
                    if held.tithi is None
                    else "vara-tithi-nakshatra",
                )
                put(
                    f"day-{i}-yoga-held-{j}-tithi",
                    "none" if held.tithi is None else held.tithi.full_key,
                )

        # `almanac_day` is the range of one unwrapped, and must agree.
        one_day = geo.almanac_day(
            date=date(Calendar.GREGORIAN, 2024, 6, 17),
            place=place,
            utc_offset_seconds=20700,
        )
        put("almanac-single-agrees", one_day.sunrise == week.at(0).sunrise)

    for key in sorted(report):
        sys.stdout.write(f"{key}\t{report[key]}\n")
    ctx.close()


if __name__ == "__main__":
    main()

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

from teistro import (
    Body,
    Calendar,
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

    for key in sorted(report):
        sys.stdout.write(f"{key}\t{report[key]}\n")
    ctx.close()


if __name__ == "__main__":
    main()

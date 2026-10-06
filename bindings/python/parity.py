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
from dataclasses import fields as dataclass_fields
from typing import Any, Iterable, Optional, Sequence, cast

import json

from teistro import (
    AshtaKoota,
    Context,
    NaamRules,
    Porutham,
    BhakootKoota,
    DhinamPorutham,
    GanamPorutham,
    KootaReading,
    PoruthamReading,
    RajjuPorutham,
    RasiPorutham,
    RasyadhipathiPorutham,
    VasyamPorutham,
    VedhaiPorutham,
    YoniPorutham,
    GanaKoota,
    MaitriKoota,
    NadiKoota,
    TaraKoota,
    VashyaKoota,
    YoniKoota,
    Almuten,
    HarmonicPoint,
    AntiscionRow,
    EclipseMoment,
    EclipseSeen,
    KpLevel,
    KpLords,
    KpRejection,
    AspectHit,
    NakshatraIngress,
    SignIngress,
    Station,
    LocalDay,
    FestivalAnswer,
    FestivalRequest,
    MuhurtaAnswer,
    MuhurtaNative,
    MuhurtaRequest,
    UnwantedPlacementClause,
    VarshaRequest,
    DashaDefinition,
    Altitude,
    MessagePart,
    Body,
    Calendar,
    Ephemeris,
    ChartLayout,
    ChartKind,
    DashaSystem,
    LayoutRow,
    Latitude,
    Longitude,
    NatalPoint,
    Observer,
    EssentialDignity,
    Perfection,
    PlanItem,
    Scale,
    Teistro,
    TeistroError,
    Varga,
    at,
    date,
    iana_zone,
    intl,
    message_parts,
)

report: dict[str, str] = {}


def natal_key(point: NatalPoint) -> str:
    """A natal point as every runner prints it: `LAGNA`, or the graha's
    full key."""
    return "LAGNA" if point.graha is None else point.graha.full_key


def harmonic_key(point: HarmonicPoint) -> str:
    """A harmonic point as every runner prints it: a graha's full key, or
    the angle's name."""
    return point.point if point.graha is None else point.graha.full_key


def reading_text(reading: KootaReading) -> str:
    """A koota's reading as every runner prints it: its fields in serde's
    order, a member by its full key, a flag as 0 or 1 and no dosha as
    `NONE`."""
    if isinstance(reading, VashyaKoota):
        return reading.relation.key
    if isinstance(reading, TaraKoota):
        return f"{reading.bride_to_groom} {reading.groom_to_bride}"
    if isinstance(reading, YoniKoota):
        return f"{reading.bride.full_key} {reading.groom.full_key} {reading.relation.key}"
    if isinstance(reading, MaitriKoota):
        sides = f"{reading.bride.full_key} {reading.groom.full_key}"
        return f"{sides} {reading.relation.key} {int(reading.lifted)}"
    if isinstance(reading, GanaKoota):
        return f"{reading.bride.full_key} {reading.groom.full_key} {int(reading.dosha)} {int(reading.lifted)}"
    if isinstance(reading, BhakootKoota):
        e = reading.exceptions
        flags = (e.one_lord, e.lords_friends, e.navamsha_lords_friends, e.tara_pure, e.vashya, reading.lifted)
        dosha = "NONE" if reading.dosha is None else reading.dosha.key
        return " ".join([str(reading.apart), dosha, *(str(int(flag)) for flag in flags)])
    if isinstance(reading, NadiKoota):
        return f"{reading.bride.full_key} {reading.groom.full_key} {int(reading.dosha)} {int(reading.lifted)}"
    return f"{reading.bride.full_key} {reading.groom.full_key}"


def porutham_text(reading: PoruthamReading) -> str:
    """A consideration's reading as every runner prints it: its fields in
    serde's order, a member by its full key (a boundary enum by its key)
    and a flag as 0 or 1."""
    if isinstance(reading, DhinamPorutham):
        return f"{reading.count} {reading.rule.key}"
    if isinstance(reading, GanamPorutham):
        return f"{reading.bride.full_key} {reading.groom.full_key} {int(reading.diminished)}"
    if isinstance(reading, YoniPorutham):
        return f"{reading.bride.full_key} {reading.groom.full_key} {int(reading.hostile)}"
    if isinstance(reading, RasiPorutham):
        return str(reading.apart)
    if isinstance(reading, RasyadhipathiPorutham):
        return (
            f"{reading.bride.full_key} {reading.groom.full_key} "
            f"{int(reading.bride_calls_friend)} {int(reading.groom_calls_friend)}"
        )
    if isinstance(reading, VasyamPorutham):
        return f"{int(reading.bride_to_groom)} {int(reading.groom_to_bride)}"
    if isinstance(reading, RajjuPorutham):
        return f"{reading.bride.key} {reading.groom.key}"
    if isinstance(reading, VedhaiPorutham):
        return str(int(reading.pierced))
    return str(reading.count)


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


def listed(items: Iterable[str]) -> str:
    """The items joined by spaces, or `none`."""
    return " ".join(items) or "none"


def put_antiscion_rows(key: str, rows: Sequence[AntiscionRow]) -> None:
    """Pairs in antiscion as every runner prints them, a chart's own or
    across a synastry: their count, then each pair's planets, side, gap
    and orb."""
    put(f"{key}-count", str(len(rows)))
    for n, row in enumerate(rows):
        put(
            f"{key}-{n}",
            f"{row.first.full_key} {row.second.full_key} {int(row.contrary)} {number(row.apart_deg)} {number(row.orb_deg)}",
        )


def put_muhurta(prefix: str, answer: MuhurtaAnswer) -> None:
    """A muhurta answer's counts, hash and every window, as every runner
    prints them."""
    put(
        f"{prefix}-counts",
        " ".join(
            str(n)
            for n in (
                len(answer.windows),
                len(answer.closed),
                answer.days_judged,
                answer.days_cut,
                answer.windows_blacked_out,
                answer.ranking,
            )
        ),
    )
    put(f"{prefix}-hash", answer.provenance.content_hash)
    for k, window in enumerate(answer.windows):
        put(f"{prefix}-{k}", f"{number(window.at.from_jd)} {number(window.at.to_jd)}")
        put(f"{prefix}-{k}-clauses", " ".join(clause.kind.CLAUSE for clause in window.clauses))
        put(
            f"{prefix}-{k}-bars",
            listed(bar if isinstance(bar, str) else bar.CLAUSE for bar in window.barred_by),
        )
        put(
            f"{prefix}-{k}-placed",
            listed(
                f"{c.kind.house}:{','.join(g.full_key for g in c.kind.by)}"
                for c in window.clauses
                if isinstance(c.kind, UnwantedPlacementClause)
            ),
        )
        score = window.score
        put(
            f"{prefix}-{k}-score",
            "none"
            if score is None
            else " ".join(
                (
                    str(score.value),
                    "none" if score.capped_at is None else str(score.capped_at),
                    listed(
                        f"{f.dimension}:{f.weight}:{'none' if f.graha is None else f.graha.full_key}"
                        for f in score.factors
                    ),
                )
            ),
        )
    for j, day in enumerate(answer.closed):
        put(f"{prefix}-closed-{j}", f"{day.date.month}-{day.date.day} {listed(k.full_key for k in day.by)}")


def put_festivals(prefix: str, answer: FestivalAnswer) -> None:
    """A festival answer's counts, hash, every observance and every
    Ekadashi fast, as every runner prints them."""
    put(f"{prefix}-counts", f"{len(answer.observances)} {len(answer.unjudged)}")
    put(f"{prefix}-hash", answer.provenance.content_hash)
    for k, observance in enumerate(answer.observances):
        decided = observance.decided_by
        if decided.by == "GUARD":
            by = f"guard:{decided.index}"
        elif decided.by == "AFTER":
            by = f"after:{decided.rule}:{decided.days}"
        else:
            by = "otherwise"
        earlier, later = observance.extents
        put(
            f"{prefix}-{k}",
            " ".join(
                (
                    observance.rule,
                    observance.month.full_key,
                    str(observance.adhika).lower(),
                    f"{observance.day.month}-{observance.day.day}",
                    observance.case,
                    by,
                    observance.choice,
                    number(observance.tithi.from_jd),
                    number(earlier.held),
                    number(later.held),
                )
            ),
        )
    for k, fast in enumerate(answer.ekadashis):
        put(
            f"{prefix}-ekadashi-{k}",
            " ".join(
                (
                    fast.rule,
                    fast.tithi.full_key,
                    fast.month.full_key,
                    str(fast.adhika).lower(),
                    f"{fast.day.month}-{fast.day.day}",
                    fast.pierced_at or "-",
                    str(fast.pierced).lower(),
                    fast.excess,
                    fast.choice,
                    number(fast.tithis[1].from_jd),
                )
            ),
        )


def put_ashta(prefix: str, matched: AshtaKoota) -> None:
    """An Ashta Koota as every runner prints it, under `prefix`."""
    put(f"{prefix}-matching", number(matched.total))
    for koota in matched.kootas:
        put(
            f"{prefix}-matching-{koota.reading.koota.full_key}",
            f"{number(koota.points)} {number(koota.max_points)} {reading_text(koota.reading)}",
        )


def put_porutham(prefix: str, ten: Porutham) -> None:
    """Ten considerations as every runner prints them, under `prefix`."""
    clauses = ten.exception
    put(
        f"{prefix}-porutham",
        f"{ten.agreeing} {ten.chief_agreeing} {int(clauses.one_lord)} "
        f"{int(clauses.lords_friendly)} {int(clauses.opposite)}",
    )
    for consideration in ten.considerations:
        put(
            f"{prefix}-porutham-{consideration.reading.koota.full_key}",
            f"{int(consideration.agrees)} {int(consideration.lifted)} {porutham_text(consideration.reading)}",
        )


def put_naam(ctx: Context) -> None:
    """Two pairs of names, as every runner asks them: a Devanagari pair
    whose groom's syllable is Abhijit's, placed in Shravana, and an IAST
    pair."""
    pairs: list[tuple[str, str, NaamRules]] = [
        ("प्रिया", "ज़ोया", {"name": {"abhijit": "SHRAVANA"}, "koota": {"nadiDosha": "MIDDLE_ONLY"}}),
        ("kṛṣṇā", "śyāma", {"name": {"latin": "IAST"}, "porutham": {"deerghaBeyond": "SEVENTH"}}),
    ]
    for n, (bride, groom, rules) in enumerate(pairs):
        read = ctx.matching.naam(bride, groom, rules)
        for who, name in (("bride", read.bride), ("groom", read.groom)):
            star = "NONE" if name.nakshatra is None else name.nakshatra.full_key
            put(f"naam-{n}-{who}", f"{name.cell} {star} {name.quarter} {name.varga.key}")
        varga = read.varga
        put(f"naam-{n}-varga", f"{varga.bride.key} {varga.groom.key} {varga.relation.key}")
        put_ashta(f"naam-{n}", read.ashta)
        put_porutham(f"naam-{n}", read.porutham)


def put_day(prefix: str, day: LocalDay) -> None:
    """A local day's every field, under the same keys for a chart's day and
    an almanac's, because the two layers hand back one record."""
    put(f"{prefix}-vara", day.vara.full_key)
    put(f"{prefix}-sunrise", day.sunrise)
    put(f"{prefix}-sunset", day.sunset)
    put(f"{prefix}-next-sunrise", day.next_sunrise)
    put(f"{prefix}-date", f"{day.date.year}-{day.date.month}-{day.date.day}")
    put(f"{prefix}-calendar", day.date.calendar.full_key)
    put(f"{prefix}-era", "none" if day.date.era is None else day.date.era.full_key)
    put(f"{prefix}-era-year", day.date.era_year)
    put(f"{prefix}-resolution", day.date.resolution.key)
    put(
        f"{prefix}-polar",
        "none" if day.polar is None else f"{day.polar.kind.key}/{day.polar.policy.key}",
    )
    if day.air is not None and day.convention is not None:
        convention = (
            f"{day.convention.key} {number(day.air.pressure_hpa)} hPa"
            f" {number(day.air.temperature_c)} C"
        )
    elif day.convention is not None:
        convention = day.convention.key
    else:
        convention = f"custom {number(day.custom_altitude_deg or 0.0)}"
    put(f"{prefix}-convention", convention)


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
    put("locale", ctx.intl.locale)
    put("settings-hash", ctx.settings_hash)
    put("settings-fnv", fnv(ctx.settings_json))
    put_naam(ctx)

    # ── The calendars ─────────────────────────────────────────────────
    day = date(Calendar.GREGORIAN, 2015, 4, 14)
    bs = ctx.calendar.convert(day, Calendar.BIKRAM_SAMBAT)
    put("bs-year", bs.year)
    put("bs-month", bs.month)
    put("bs-day", bs.day)
    put("bs-era", None if bs.era is None else bs.era.full_key)
    put("bs-era-year", bs.era_year)
    put("bs-resolution", bs.resolution.key)
    fixed = ctx.calendar.fixed_of(day)
    put("fixed", fixed)
    put("weekday", ctx.calendar.weekday_of(day))
    put("month-length", ctx.calendar.month_length(Calendar.GREGORIAN, 2024, 2))
    put("is-leap", ctx.calendar.is_leap(Calendar.GREGORIAN, 2024))
    put("jd-of-fixed", teistro.julian_day_of_fixed(fixed))
    back, fraction = teistro.fixed_of_julian_day(2457126.75)
    put("fixed-of-jd", back)
    put("fraction-of-jd", fraction)

    # ── Time ──────────────────────────────────────────────────────────
    civil = at(date(Calendar.GREGORIAN, 1986, 1, 1), hour=0, minute=20)
    zone = iana_zone("Asia/Kathmandu")
    resolved = ctx.time.resolve(civil, zone)
    put("resolve-jd", resolved.instant_jd_utc)
    put("resolve-offset", resolved.offset_seconds)
    put("resolve-era", resolved.era.key)
    put("resolve-source", resolved.source.key)
    put("resolve-time-known", resolved.time_known)
    put("resolve-tzdb", resolved.tzdb_version)
    put("resolve-warnings", len(resolved.warnings))
    civil_back, resolution = ctx.time.civil_of(
        resolved.instant_jd_utc, zone, Calendar.GREGORIAN
    )
    put("civil-year", civil_back.date.year)
    put("civil-minute", civil_back.time.minute)
    put("civil-offset", resolution.offset_seconds)
    tt = ctx.time.convert(2451544.5, Scale.UTC, Scale.TT)
    put("tt-jd", tt.jd)
    put("tt-delta-t", tt.delta_t_seconds)
    put("tt-delta-t-source", tt.delta_t_source.key)
    put("tt-delta-t-model", tt.delta_t_model)
    delta = ctx.time.delta_t(2451544.5)
    put("delta-t-seconds", delta.seconds)
    put("delta-t-source", delta.source.key)

    # ── Keys ──────────────────────────────────────────────────────────
    identifier = ctx.keys.id("graha.SUN")
    put("key-id", identifier)
    put("key-name", ctx.keys.name(identifier))
    try:
        ctx.keys.id("graha.SUNN")
        put("refusal", "none")
    except TeistroError as error:
        put("refusal-status", error.status.key)
        put("refusal-detail", error.detail)
        put("refusal-hint-names-sun", "SUN" in error.hint)

    # ── The locale engine ─────────────────────────────────────────────
    rendered = ctx.intl.render(
        "sdk.reason.grahaInBhava",
        {"graha": {"$entity": "graha.JUPITER"}, "bhava": 7},
    )
    put("render-fnv", fnv(rendered.text))
    put("render-length", len(rendered.text))
    put("render-resolved-from", rendered.resolved_from)
    put("render-fallback", bool(rendered.is_fallback))
    # A rendered message's parts, which is what a rich renderer walks.
    # `sdk.reason.lordship` is one of the two shipped messages carrying
    # `{#b}`; the plain one beside it holds every binding to the rule that
    # no markup means the one text part, made rather than carried.

    def part_shape(parts: "list[MessagePart]") -> str:
        return "|".join(
            f"text:{part.value}"
            if part.is_text
            else f"{part.kind}:{part.name}("
            + ",".join(f"{name}={value}" for name, value in sorted(part.options.items()))
            + ")"
            for part in parts
        )

    rich = ctx.intl.render(
        "sdk.reason.lordship",
        {"graha": {"$entity": "graha.JUPITER"}, "bhava": 5},
    )
    put("render-rich-parts", part_shape(message_parts(rich)))
    put("render-plain-parts", part_shape(message_parts(rendered)))
    put("has-message", ctx.intl.has("sdk.reason.grahaInBhava"))
    put("has-missing-message", ctx.intl.has("sdk.nope.missing"))
    put("transliterated", ctx.intl.transliterate("सूर्य बृहस्पति"))
    sun = ctx.intl.entity("graha.SUN")
    put("entity-sun-name", sun.name)
    put("entity-sun-iast", sun.iast)
    put("entity-sun-glyph", sun.glyph)
    put("entity-sun-gender", None if sun.gender is None else sun.gender.value)
    put(
        "message-graha-in-bhava",
        ctx.intl.messages.sdk.reason.graha_in_bhava(graha=intl.GrahaKey.JUPITER, bhava=7),
    )
    put(
        "message-bs-date",
        ctx.intl.messages.sdk.calendar.bikram_sambat.date.long(
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
        ",".join(f"{step.name}:{step.implementation}" for step in sky.steps_applied),
    )
    put("provenance-fnv", fnv(sky.provenance_json))
    put("provenance-profile", sky.provenance.profile)
    put("provenance-settings-hash", sky.provenance.settings_hash)
    put("provenance-provider-frame", sky.provenance.provider.frame)

    # ── The chart the topocentric profile founds ──────────────────────
    # The scenario above runs under `nepali-default`, whose frame is
    # **topocentric** — inherited from the baseline engine, and what
    # every recorded chart in the corpus is. Until the completion's
    # centre step this could not found a chart at all, and the refusal
    # was what the three bindings compared. Now the chart itself is,
    # which is the stronger comparison: the step runs per body, per
    # instant, inside the library, so three bindings agreeing on its
    # output is three bindings agreeing on the whole of it.
    place = Observer(
        latitude_deg=Latitude(27.7172),
        longitude_deg=Longitude(85.324),
        altitude_m=Altitude(1400),
    )
    placed = ctx.chart.found(instant=2451545, place=place, utc_offset_seconds=20700)
    put("chart-under-topocentric", "founded")
    put("topocentric-steps", ",".join(placed.batch.steps_applied))
    put("topocentric-lagna", placed.lagna_deg)
    for j, graha in enumerate(placed.grahas):
        put(f"topocentric-graha-{j}", graha.graha.full_key)
        put(f"topocentric-graha-{j}-lon", graha.longitude_deg)
        put(f"topocentric-graha-{j}-lat", graha.latitude_deg)
        put(f"topocentric-graha-{j}-speed", graha.speed_deg_per_day)

    # ── A chart founded on a classical astronomy ──────────────────────
    # The Surya Siddhanta by name: the text's zodiac, places, Lagna and
    # day (docs/03-design/classical-chart.md), which every binding reaches
    # through the selector and must read back alike, deviation and all.
    with teistro.context(
        profile="surya-siddhanta",
        ephemeris=Ephemeris.SURYA_SIDDHANTA,
    ) as classical:
        text = classical.chart.found(instant=2447995.4895833335, place=place, utc_offset_seconds=20700)
        put("classical-steps", ",".join(text.batch.steps_applied))
        put("classical-lagna", text.lagna_deg)
        put("classical-sunrise", text.day.sunrise)
        deviation = text.provenance.deviation
        assert deviation is not None
        put("classical-deviation", f"{deviation.model}: {deviation.detail}")
        for j, graha in enumerate(text.grahas):
            put(f"classical-graha-{j}-lon", graha.longitude_deg)

    # ── A chart and an almanac, under a geocentric profile ────────────
    # Everything after this runs on the SDK's own default profile, which
    # is geocentric, so that the two centres are both exercised.

    # A layout of the consumer's own, registered on the context the charts
    # are drawn under: the South Indian row renamed, as every runner
    # registers it (`03-design/chart-geometry.md` §7f).
    with teistro.context(test_provider=True) as shipped:
        kerala: LayoutRow = {**shipped.chart.layout("SOUTH_INDIAN"), "key": "ACME_KERALA"}
    # A dasha system of the consumer's own, the same definition every runner
    # registers (`03-design/dasha-kernels.md`).
    parity_dasha: DashaDefinition = json.loads('{"kernel":"UDU","key":"ACME_PARITY","sources":["the parity scenario"],"lords":[{"graha":"SUN","years":5},{"graha":"MOON","years":10},{"graha":"MARS","years":7},{"graha":"MERCURY","years":12}],"reference":"MULA","count":"TO_REFERENCE","span":2,"offset":1,"repeats":true,"year_length":"SAVANA_360","depth":2}')
    with teistro.context(
        profile="parashari-classical",
        locale="ne-Deva-NP",
        test_provider=True,
        layouts=[kerala],
        dasha_systems=[parity_dasha],
    ) as geo:
        put("geo-profile", geo.profile)
        put("geo-settings-hash", geo.settings_hash)

        # ── Charts ────────────────────────────────────────────────────
        # Two instants, so a per-chart section that ran charts-outermost
        # the wrong way round shows up as the second chart's values in
        # the first's place rather than as nothing at all.
        # Two divisional charts asked for, and two rather than one
        # because the layout is charts outermost then charts asked for:
        # only two of each can catch a transposed stride.
        charts = geo.chart.found_many(
            instants=[2460482.5, 2460600.25],
            place=place,
            utc_offset_seconds=20700,
            vargas=[Varga.D9, Varga.D10],
            dashas=[
                DashaSystem.VIMSHOTTARI,
                DashaSystem.CHARA,
                DashaSystem.KALACHAKRA,
                DashaSystem.RELEASING_FORTUNE,
                DashaSystem.PROFECTION,
                DashaSystem.FIRDARIA,
                DashaSystem.DECENNIALS,
                "dasha_system.ACME_PARITY",
            ],
            drawings=[
                (ChartLayout.NORTH_INDIAN, Varga.D1),
                (ChartLayout.SOUTH_INDIAN, Varga.D9),
                (ChartLayout.WESTERN_WHEEL, Varga.D1),
                ("chart_layout.ACME_KERALA", Varga.D9),
            ],
            theme="DARK",
            rules={
                "shipped": ["NABHASAS"],
                "longevity": True,
                "ayurdaya": {"enemy_exempt": "mars", "enmity": "compound", "rising": "every"},
            },
            # Every composer, so the four agree on what every chart *says*
            # and not only on what it computes
            # (`03-design/plans-at-the-boundary.md`).
            interpret={
                "placements": True,
                "readings": True,
                "strength": True,
                "houses": True,
                "positions": True,
                "aspects": True,
                "conditions": True,
                "karakas": True,
            },
            aspects=True,
            points=True,
            houses=True,
            ashtakavarga=True,
            vimshopaka=True,
            vaiseshikamsa=True,
            dasha_phala=True,
            jaimini=True,
            avakahada=True,
            outer_planets=True,
            gochar={"instants": [2460676.5, 2460736.5], "ashtakavarga": True},
            hits={
                "from": 2460676.5,
                "to": 2460736.5,
                "grahas": ["SUN", "MERCURY", "SATURN"],
                "aspects": [0, 90, 180],
                "orbDeg": 2,
            },
            sade_sati={"from": 2460676.5, "to": 2464329.0, "reckoning": "DEGREE", "spells": [4, 7, 8]},
            kp={"number": 74, "anyAyanamsha": True},
            fortitudes={
                "dignities": {
                    "sectRule": "DAYLIGHT",
                    "rules": {"terms": "EGYPTIAN", "triplicities": "PTOLEMY"},
                    "scores": {"peregrine": 0},
                },
                "rules": {
                    "beamsDeg": 15,
                    "combustionInSign": False,
                    "partile": {"WITHIN": {"orbDeg": 1}},
                    "siege": {"WITHIN": {"spanDeg": 30}},
                },
                "scores": {"regulus": 5},
                "almuten": {"fortune": "REVERSED_BY_NIGHT"},
            },
            lots={"fortune": "REVERSED_WHILE_MOON_UP"},
            considerations={"moonLateFromDeg": 25},
            perfection={"house": 7, "rules": {"horizonDays": 120}},
            western_aspects={
                "aspects": ["CONJUNCTION", "SEXTILE", "SQUARE", "TRINE", "QUINCUNX", "OPPOSITION"],
                "orbs": {
                    "model": "MOIETIES",
                    "orbs": [
                        {"graha": graha, "orbDeg": orb}
                        for graha, orb in (
                            ("SUN", 17),
                            ("MOON", 12.5),
                            ("MERCURY", 7),
                            ("VENUS", 8),
                            ("MARS", 7.5),
                            ("JUPITER", 12),
                            ("SATURN", 10),
                            ("URANUS", 5),
                            ("NEPTUNE", 5),
                            ("PLUTO", 5),
                        )
                    ],
                },
            },
            parallels={"orbDeg": 1.5},
            antiscia={"cusps": {}},
            midpoints={"orbDeg": 1.5},
            western_houses={},
            harmonic={"number": 5},
            matching={
                "partner": {
                    "instant": 2451545.25,
                    "observer": Observer(latitude_deg=Latitude(-33.87), longitude_deg=Longitude(151.21), altitude_m=Altitude(0)),
                    "utc_offset_seconds": 36000,
                },
                "partnerRole": "BRIDE",
                "rules": {"bhakootLift": "GARGA"},
                "porutham": {"lordsFriendship": "ONE_WAY"},
                "kuja": {"houses": "WITH_SECOND", "from": "LAGNA_MOON_VENUS"},
            },
            synastry={
                "partner": {
                    "instant": 2451545.25,
                    "observer": Observer(latitude_deg=Latitude(-33.87), longitude_deg=Longitude(151.21), altitude_m=Altitude(0)),
                    "utc_offset_seconds": 36000,
                },
                "aspects": ["CONJUNCTION", "SQUARE", "TRINE", "OPPOSITION"],
                "zodiac": "CHARTS",
                "parallels": {"orbDeg": 1.5},
                "antiscia": {"orbs": {"model": "LEO"}},
                "midpoints": {"orbDeg": 1.5},
                "composite": True,
                "davison": True,
            },
            progressions={
                "at": 2470000.5,
                "year": "NOON_SIDEREAL_TIME",
                "angles": "SOLAR_ARC_LONGITUDE",
                "direction": "NAIBOD",
                "contacts": {
                    "from": 2462000.5,
                    "to": 2465652.5,
                    "grahas": ["MOON", "SUN"],
                    "points": ["LAGNA", "MARS", "VENUS"],
                    "aspects": [0, 45, 90, 135, 180],
                },
            },
            shadbala=True,
            bhava_bala=True,
            state=True,
        )
        put("chart-varga-count", charts.decoded.varga_count)
        put("chart-drishti-table", charts.decoded.drishti_table)
        put("chart-count", len(charts))
        put("chart-kind", charts.kind.full_key)
        put("chart-place-lat", charts.place.latitude_deg)
        put("chart-place-lon", charts.place.longitude_deg)
        put("chart-model-fnv", fnv(charts.model))
        put("chart-steps", ",".join(charts.steps_applied))
        put("chart-provenance-fnv", fnv(charts.provenance_json))
        put("chart-provenance-profile", charts.provenance.profile)
        put("chart-graha-count", charts.decoded.graha_count)

        for chart in charts:
            i = chart.index
            put(f"chart-{i}-content-hash", chart.provenance.content_hash)
            put(f"chart-{i}-instant", chart.instant)
            put(f"chart-{i}-lagna", chart.lagna_deg)
            put(f"chart-{i}-day-lagna", chart.day_lagna_deg)
            put(f"chart-{i}-ayanamsha", chart.ayanamsha_offset_deg)
            put(f"chart-{i}-day-part", chart.day_part.key)
            put(f"chart-{i}-day-elapsed", chart.day_elapsed)
            put_day(f"chart-{i}", chart.day)
            timing = chart.timing
            put(f"chart-{i}-ghati", timing.ghati)
            put(f"chart-{i}-pala", timing.pala)
            put(f"chart-{i}-vipala", timing.vipala)
            put(f"chart-{i}-hora-number", timing.hora_number)
            put(f"chart-{i}-hora-lord", timing.hora_lord.full_key)
            for j, state in enumerate(chart.states):
                key = f"chart-{i}-state-{j}"
                put(key, state.graha.full_key)
                put(f"{key}-sign", state.sign.full_key)
                put(f"{key}-house", state.house)
                put(f"{key}-dignity", state.dignity.full_key)
                put(f"{key}-natural", state.friendship.natural.full_key)
                put(f"{key}-compound", state.friendship.compound.full_key)
                put(
                    f"{key}-dispositor",
                    state.friendship.dispositor.full_key
                    if state.friendship.dispositor
                    else "none",
                )
                put(f"{key}-burning", state.combustion.burning.key)
                put(
                    f"{key}-from-sun",
                    state.combustion.from_sun_deg
                    if state.combustion.from_sun_deg is not None
                    else "none",
                )
                put(
                    f"{key}-orb",
                    state.combustion.orb_deg
                    if state.combustion.orb_deg is not None
                    else "none",
                )
                put(f"{key}-age", state.age.full_key)
                put(f"{key}-wakefulness", state.wakefulness.full_key)
                put(f"{key}-deeptadi", state.deeptadi.full_key if state.deeptadi else "none")
                put(
                    f"{key}-sayanadi",
                    f"{state.sayanadi.avastha.full_key} "
                    + ",".join(c.full_key for c in state.sayanadi.cheshtas)
                    if state.sayanadi
                    else "none",
                )
                put(
                    f"{key}-holding",
                    ",".join(m.full_key for m in state.lajjitadi.holding) or "none",
                )
                put(
                    f"{key}-undecided",
                    ",".join(m.full_key for m in state.lajjitadi.undecided) or "none",
                )
                put(
                    f"{key}-war",
                    f"{state.war.opponent.full_key}:{str(state.war.is_winner).lower()}"
                    if state.war
                    else "none",
                )
                put(f"{key}-sign-edge", state.boundaries.sign_deg)
            for read in chart.bhavas:
                put(f"chart-{i}-bhava-{read.number}-sign", read.sign.full_key)
                put(f"chart-{i}-bhava-{read.number}-lord", read.lord.full_key)
                put(f"chart-{i}-bhava-{read.number}-quadrant", read.quadrant.key)
            found = chart.points
            put(f"chart-{i}-point-count", len(found))
            for k, derived in enumerate(found):
                put(f"chart-{i}-point-{k}", derived.point.full_key)
                put(f"chart-{i}-point-{k}-lon", derived.longitude_deg)
                put(f"chart-{i}-point-{k}-sign", derived.sign.full_key)
                put(f"chart-{i}-point-{k}-sign-edge", derived.boundaries.sign_deg)
            # Every drishti, because the count differs from chart to
            # chart -- which is why the section is ragged.
            drishti = chart.aspects
            put(f"chart-{i}-aspect-count", len(drishti))
            for k, one in enumerate(drishti):
                put(f"chart-{i}-aspect-{k}", f"{one.from_graha.full_key}>{one.to.full_key}")
                put(f"chart-{i}-aspect-{k}-houses", one.houses)
                put(f"chart-{i}-aspect-{k}-strength", one.strength.key)
                put(f"chart-{i}-aspect-{k}-from-sign", one.from_edge.sign_deg)
                put(f"chart-{i}-aspect-{k}-to-sign", one.to_edge.sign_deg)
            answered = chart.rules
            assert answered is not None
            put(f"chart-{i}-rules-present", ",".join(held["rule"] for held in answered["present"]))
            put(f"chart-{i}-rules-pindayu", answered["longevity"]["ayurdaya"]["pindayu"]["years"])
            put(f"chart-{i}-rules-rays", answered["longevity"]["rasmi"]["total"])
            put(f"chart-{i}-rules-span", answered["longevity"]["choice"]["ayus"] or "")
            # **Every item said**, not merely counted: the only place the
            # four bindings are compared on text, which exercises the
            # composers, the params shape and the locale engine at once.
            plans = chart.plans
            assert plans is not None
            # Over whatever the request asked for, not a list written
            # here: a composer added is a key in `plans`, and a runner
            # naming them itself goes quietly out of step with the other
            # three — which is what this one did when `houses` landed.
            # `Plans` is a TypedDict, so `.items()` hands back `object`
            # and the runtime shape is restored here rather than guessed.
            composed: list[tuple[str, list[PlanItem]]] = [
                (composer, cast("list[PlanItem]", items))
                for composer, items in plans.items()
            ]
            for composer, items in composed:
                put(f"chart-{i}-plan-{composer}-count", len(items))
                for n, item in enumerate(items):
                    said = ctx.intl.render(item["key"], item["params"]).text
                    put(f"chart-{i}-plan-{composer}-{n}", f"{item['key']}: {said}")
            for d, drawing in enumerate(chart.drawings):
                key = f"chart-{i}-drawing-{d}"
                put(key, drawing.layout_key)
                put(f"{key}-varga", drawing.varga.full_key)
                put(f"{key}-cells", len(drawing.cells))
                put(f"{key}-frames", len(drawing.frame))
                put(f"{key}-marks", len(drawing.marks))
                put(f"{key}-svg", drawing.svg)
                for c, drawn in enumerate(drawing.cells):
                    where = f"{key}-cell-{c}"
                    put(f"{where}-sign", drawn.sign.full_key)
                    put(f"{where}-house", drawn.house)
                    put(f"{where}-lagna", drawn.lagna)
                    put(f"{where}-ring", drawn.ring)
                    put(f"{where}-bodies", ",".join(drawn.bodies) or "none")
                    put(f"{where}-label", f"{number(drawn.label.x)},{number(drawn.label.y)}")
                    put(f"{where}-anchor", f"{number(drawn.anchor.x)},{number(drawn.anchor.y)}")
                    put(f"{where}-start", f"{number(drawn.outline.start.x)},{number(drawn.outline.start.y)}")
                    put(
                        f"{where}-steps",
                        ",".join(type(step).__name__.removesuffix("Segment").upper() for step in drawn.outline.segments),
                    )
                for m, mark in enumerate(drawing.marks):
                    where = f"{key}-mark-{m}"
                    put(where, mark.body)
                    put(f"{where}-at", f"{number(mark.at.x)},{number(mark.at.y)}")
                    put(f"{where}-lon", mark.longitude_deg)
            put(f"chart-{i}-dasha-count", len(chart.dashas))
            av = chart.ashtakavarga
            assert av is not None
            put(f"chart-{i}-ashtakavarga", f"{av.shodhana.key} {av.ekadhipatya.key}")
            for g in av.grahas:
                key = f"chart-{i}-ashtakavarga-{g.graha.full_key}"
                put(key, ",".join(str(b) for b in g.bindus))
                put(f"{key}-reduced", ",".join(str(b) for b in g.reduced) if g.reduced else None)
                put(f"{key}-pindas", f"{g.rashi_pinda},{g.graha_pinda},{g.yoga_pinda}")
            put(
                f"chart-{i}-sarvashtakavarga",
                ";".join(",".join(str(b) for b in row) for row in (av.sarva, av.trikona, av.reduced)),
            )
            sb = chart.shadbala
            assert sb is not None
            for strength in sb.grahas:
                key = f"chart-{i}-shadbala-{strength.graha.full_key}"
                st, ka = strength.sthana, strength.kaala
                parts: list[float] = [st.uchcha, st.saptavargaja, st.ojayugma, st.kendradi, st.drekkana, strength.dig]
                parts += [ka.nathonnatha, ka.paksha, ka.tribhaga, ka.abda, ka.masa, ka.vara, ka.hora, ka.ayana, ka.yuddha]
                parts += [strength.cheshta, strength.naisargika, strength.drik]
                put(key, ",".join(number(value) for value in parts))
                put(
                    f"{key}-total",
                    f"{number(strength.virupas)},{number(strength.rupas)},{number(strength.required_rupas)},{str(strength.strong).lower()},{number(strength.ishta)},{number(strength.kashta)},{number(strength.subha_rashmi)},{number(strength.ashubha_rashmi)}",
                )
            bb = chart.bhava_bala
            assert bb is not None
            for house in bb.bhavas:
                values = (house.adhipati, house.dig, house.drishti, house.special, house.virupas)
                put(
                    f"chart-{i}-bhava-bala-{house.bhava}",
                    f"{house.lord.full_key} " + ",".join(number(value) for value in values),
                )
            vk = chart.vaiseshikamsa
            assert vk is not None
            for named_graha in vk.grahas:
                standings = (
                    named_graha.shadvarga,
                    named_graha.saptavarga,
                    named_graha.dashavarga,
                    named_graha.shodashavarga,
                )
                named = ",".join(
                    f"{st.good_vargas}:{st.name.full_key if st.name is not None else 'null'}" for st in standings
                )
                put(
                    f"chart-{i}-vaiseshikamsa-{named_graha.graha.full_key}",
                    f"{named} {str(named_graha.impaired).lower()}",
                )
            dp = chart.dasha_phala
            assert dp is not None
            for phala in dp.grahas:
                put(
                    f"chart-{i}-dasha-phala-{phala.graha.full_key}",
                    ",".join(number(value) for value in phala.subhankas)
                    + f" {phala.nature.full_key} {phala.phase.key}"
                    + f" {str(phala.favourable).lower()} {str(phala.unfavourable).lower()}",
                )
            jr = chart.jaimini
            assert jr is not None
            karakamsha, brahma_graha = jr.karakamsha, jr.brahma
            put(
                f"chart-{i}-jaimini",
                f"{karakamsha.atmakaraka.full_key} {karakamsha.sign.full_key} "
                + ",".join(str(h) for h in karakamsha.in_rasi)
                + " "
                + ",".join(str(h) for h in karakamsha.in_navamsha),
            )
            put(
                f"chart-{i}-graha-arudhas",
                ",".join(sign.full_key if sign else "-" for sign in jr.graha_arudhas),
            )
            birth = chart.avakahada
            assert birth is not None
            put(
                f"chart-{i}-avakahada",
                f"{birth.nakshatra.full_key} {birth.pada} {birth.rashi.full_key} {birth.nakshatra_lord.full_key} "
                + f"{birth.rashi_lord.full_key} {birth.varna.full_key} {birth.yoni.full_key} {birth.gana.full_key} {birth.nadi.full_key}",
            )
            syllable = birth.syllable
            put(
                f"chart-{i}-avakahada-syllable",
                f"{syllable.cell} {syllable.devanagari} {syllable.iast} {syllable.varga.key}",
            )
            put(
                f"chart-{i}-brahma",
                f"{brahma_graha.rule.key} {brahma_graha.counted_from.full_key} "
                + (",".join(g.full_key for g in brahma_graha.qualified) or "-")
                + f" {brahma_graha.graha.full_key if brahma_graha.graha else '-'}"
                + f" {brahma_graha.passed_from.full_key if brahma_graha.passed_from else '-'}"
                + f" {brahma_graha.none.key if brahma_graha.none else '-'}",
            )
            for slot, transit in enumerate(chart.gochar):
                ref, rules = transit.reference, transit.rules
                put(
                    f"chart-{i}-gochar-{slot}",
                    f"{number(transit.instant)} {ref.from_.key} {ref.sign.full_key}"
                    + f" {rules.node_vedha.key} {rules.node_obstruction.key} {rules.ashtakavarga_good_from.key}",
                )
                for k, judged in enumerate(transit.ashtakavarga or ()):
                    put(
                        f"chart-{i}-gochar-{slot}-av-{k}",
                        f"{judged.graha.full_key} {judged.bindus} {str(judged.good).lower()} {judged.kakshya.index}"
                        + f" {judged.kakshya.lord.key} {str(judged.kakshya_bindu).lower()} {judged.sarva}"
                        + f" {judged.sarva_standing.key}",
                    )
                for k, moving in enumerate(transit.grahas):
                    put(
                        f"chart-{i}-gochar-{slot}-{k}",
                        f"{moving.graha.full_key} {moving.transit.sign.full_key} {number(moving.transit.degrees)} {moving.house}"
                        + f" {str(moving.good_house).lower()} {moving.vedha_house if moving.vedha_house is not None else '-'}"
                        + f" {','.join(o.full_key for o in moving.obstructed_by) or '-'}"
                        + f" {moving.verdict.key} {moving.fruition.key} {str(moving.fruitful_now).lower()}",
                    )
            for k, hit in enumerate(chart.hits):
                e = hit.event
                into = e.into.full_key if isinstance(e, (SignIngress, NakshatraIngress)) else "-"
                motion = e.turns.key if isinstance(e, Station) else e.motion.key
                if isinstance(e, AspectHit):
                    to = natal_key(e.to)
                    angle, phase = str(e.angle), e.phase.key
                else:
                    to, angle, phase = "-", "-", "-"
                put(
                    f"chart-{i}-hit-{k}",
                    f"{number(hit.instant)} {hit.graha.full_key} {e.kind.key} {into} {motion} {to} {angle} {phase}",
                )
            ss = chart.sade_sati
            assert ss is not None
            put(f"chart-{i}-sade-sati", f"{ss.reference.from_.key} {ss.reference.sign.full_key} {ss.reckoning.key}")

            def bound(jd: float | None) -> str:
                return "-" if jd is None else number(jd)

            periods = [one.phases for one in ss.sade_sati] + [(spell,) for spell in ss.spells]
            lines = [
                f"{period} {spell.house} {bound(v.from_)} {bound(v.to)}"
                for period, spells in enumerate(periods)
                for spell in spells
                for v in spell.visits
            ]
            for k, line in enumerate(lines):
                put(f"chart-{i}-sade-sati-{k}", line)
            kp = chart.kp
            assert kp is not None

            def keys(members: Sequence[Any]) -> str:
                return ",".join(member.full_key for member in members) or "-"

            def level(at: KpLevel) -> str:
                return f"{at.lord.full_key} {at.span.start} {at.span.end}"

            def lords(of: KpLords) -> str:
                return f"{of.sign.full_key} {level(of.star)} {level(of.sub)} {level(of.sub_sub)}"

            def rejection(by: Optional[KpRejection]) -> str:
                return "-" if by is None else f"{by.retrograde.full_key}:{str(by.by_star).lower()}"

            kp_rules = kp.ruling.rules
            put(
                f"chart-{i}-kp",
                f"{kp.chart.system.full_key} {kp_rules.count} {kp_rules.node_rulers} {kp_rules.retrograde_rejection}",
            )
            for cusp in kp.chart.cusps:
                put(f"chart-{i}-kp-cusp-{cusp.house}", f"{cusp.longitude} {lords(cusp.lords)}")
            for p in kp.chart.planets:
                put(
                    f"chart-{i}-kp-planet-{p.graha.full_key}",
                    f"{p.longitude} {str(p.retrograde).lower()} {p.house} {lords(p.lords)}",
                )
            for h in kp.significators.houses:
                put(
                    f"chart-{i}-kp-house-{h.house}",
                    f"{keys(h.in_occupants_stars)} {keys(h.occupants)} {keys(h.in_lords_star)} {h.lord.full_key} "
                    f"{keys(h.conjoined)} {keys(h.aspected)} {keys(h.intercepted)}",
                )
            for agency in kp.significators.nodes:
                put(
                    f"chart-{i}-kp-node-{agency.node.full_key}",
                    f"{keys(agency.conjoined)} {agency.star_lord.full_key} {keys(agency.aspecting)} "
                    f"{agency.sign_lord.full_key}",
                )
            for k, r in enumerate(kp.ruling.rulers):
                reasons = ",".join(
                    f"AGENT:{why.of.full_key}:{why.by}" if why.of is not None else why.kind for why in r.reasons
                )
                put(
                    f"chart-{i}-kp-ruler-{k}",
                    f"{r.graha.full_key} {reasons} {str(r.retrograde).lower()} "
                    f"{rejection(r.rejected_by)} {rejection(r.rejected_by_sub)}",
                )
            dg = chart.dignities
            assert dg is not None
            sc = dg.scores
            worth = [sc.house, sc.exaltation, sc.triplicity, sc.term, sc.face, sc.detriment, sc.fall, sc.peregrine]
            put(
                f"chart-{i}-dignities",
                f"{dg.sect.key} {dg.sect_rule.key} {dg.rules.terms.key} {dg.rules.triplicities.key} "
                + ",".join(str(one) for one in worth),
            )
            dignity_flags = ("house", "exaltation", "triplicity", "term", "face", "detriment", "fall")
            for planet_dignity in dg.planets:
                flags_held = [flag for flag in dignity_flags if getattr(planet_dignity.dignity, flag)]
                flags_held += ["peregrine"] if planet_dignity.peregrine else []
                put(
                    f"chart-{i}-dignity-{planet_dignity.planet.full_key}",
                    f"{number(planet_dignity.longitude_deg)} {','.join(flags_held) or '-'} "
                    f"{planet_dignity.score} {planet_dignity.reception}",
                )
            for k, reception in enumerate(dg.receptions):
                sides = [
                    ",".join(flag for flag in dignity_flags if getattr(side, flag))
                    for side in (reception.first_in, reception.second_in)
                ]
                put(
                    f"chart-{i}-reception-{k}",
                    f"{reception.planets[0].full_key} {reception.planets[1].full_key} {sides[0]} {sides[1]} "
                    f"{','.join(reception.mutual) or '-'}",
                )
            ft = chart.fortitudes
            assert ft is not None
            put(
                f"chart-{i}-fortitudes",
                f"{ft.sky.houses.full_key} "
                + " ".join(number(v) for v in (ft.sky.north_node_deg, ft.sky.regulus_deg, ft.sky.spica_deg, ft.sky.algol_deg)),
            )
            fr = ft.rules
            partile = f"WITHIN:{number(fr.partile_orb_deg)}" if fr.partile.key == "WITHIN" else fr.partile.key
            siege = f"WITHIN:{number(fr.siege_span_deg)}" if fr.siege.key == "WITHIN" else fr.siege.key
            put(
                f"chart-{i}-fortitude-rules",
                f"{number(fr.combustion_deg)} {int(fr.combustion_in_sign)} "
                + " ".join(number(v) for v in (fr.beams_deg, fr.cazimi_deg, fr.cusp_orb_deg, fr.star_orb_deg))
                + f" {partile} {siege} "
                + ",".join(number(v) for v in fr.mean_motion_deg),
            )
            lines_scored = [getattr(ft.scores, f.name) for f in dataclass_fields(ft.scores) if f.name != "houses"]
            put(
                f"chart-{i}-fortitude-scores",
                ",".join(str(v) for v in ft.scores.houses) + " " + ",".join(str(v) for v in lines_scored),
            )
            put(f"chart-{i}-fortitude-houses", ",".join(number(v) for v in ft.sky.cusps_deg))
            for k, planet_accidents in enumerate(ft.planets):
                met = ",".join(f"{line.accident.key}:{line.points}" for line in planet_accidents.accidents)
                put(
                    f"chart-{i}-fortitude-{planet_accidents.planet.full_key}",
                    f"{number(ft.sky.speeds_deg_per_day[k])} {planet_accidents.house} {met or '-'} "
                    f"{planet_accidents.fortitude} {planet_accidents.debility} {planet_accidents.net}",
                )
            al = ft.almutens
            put(
                f"chart-{i}-almuten-rules",
                f"{al.rules.place.key} {al.rules.fortune.key} "
                + " ".join(number(v) for v in (al.fortune_deg, ft.sky.ascendant_deg, ft.sky.midheaven_deg)),
            )

            def ranked(almuten: Almuten) -> str:
                totals = ",".join(str(at.total) for at in almuten.totals)
                tops = ",".join(planet.full_key for planet in almuten.almutens) or "-"
                partakers = ",".join(planet.full_key for planet in almuten.partakers) or "-"
                return f"{totals} {tops} {partakers}"

            put(f"chart-{i}-almuten-figure", ranked(al.figure))
            put(f"chart-{i}-almuten-places", ranked(al.places))
            for house_number, almuten in enumerate(al.houses, start=1):
                put(f"chart-{i}-almuten-house-{house_number}", ranked(almuten))
            lt = chart.lots
            assert lt is not None
            put(
                f"chart-{i}-lots",
                f"{lt.sect.key} {lt.request.sect_rule.key} {lt.request.fortune.key} {int(lt.fortune_reversed)}",
            )
            for placed_lot in lt.lots:
                lot_at = placed_lot.place
                put(
                    f"chart-{i}-lot-{placed_lot.lot.key}",
                    f"{number(lot_at.longitude_deg)} {lot_at.sign.full_key} {lot_at.lord.full_key} {lot_at.house}",
                )
            cs = chart.considerations
            assert cs is not None

            def commas(values: Iterable[str]) -> str:
                return ",".join(values) or "-"

            def perfection(found: Optional[Perfection]) -> str:
                if found is None:
                    return "-"
                return f"{found.planet.full_key} {found.aspect.key} {number(found.days)} {number(found.gap_deg)}"

            rd, asc, mn, sv = cs.radicality, cs.ascendant, cs.moon, cs.seventh
            put(
                f"chart-{i}-considerations",
                f"{rd.hour_lord.full_key} {rd.ascendant_lord.full_key} {commas([g.key for g in rd.grounds])}"
                f" {asc.sign.full_key} {number(asc.degree)} {int(asc.early)} {int(asc.late)} {int(asc.short_ascension)}",
            )
            put(
                f"chart-{i}-considerations-moon",
                f"{mn.sign.full_key} {number(mn.degree)} {int(mn.late)} {int(mn.late_sign)} {int(mn.via_combusta)}"
                f" {number(mn.course.days_in_sign)} {int(mn.course.eased)}",
            )
            put(f"chart-{i}-considerations-next", perfection(mn.course.next))
            put(f"chart-{i}-considerations-within", perfection(mn.course.within_orb))
            put(
                f"chart-{i}-considerations-seventh",
                f"{number(sv.cusp_deg)} {sv.lord.full_key} {commas([g.full_key for g in sv.infortunes_in_house])}"
                f" {int(sv.lord_retrograde)} {int(sv.lord_combust)} {int(sv.lord_in_fall)}"
                f" {int(sv.lord_in_infortune_term)} {sv.lord_net}",
            )
            put(
                f"chart-{i}-considerations-saturn",
                f"{cs.saturn_house} {int(cs.saturn_retrograde)} {int(cs.ascendant_lord_combust)}",
            )
            put(
                f"chart-{i}-considerations-rules",
                f"{number(cs.rules.moon_late_from_deg)} {','.join(number(orb) for orb in cs.rules.orbs_deg)}",
            )
            pf = chart.perfection
            assert pf is not None

            def dignities_held(dignity: EssentialDignity) -> str:
                return commas(f.name for f in dataclass_fields(dignity) if getattr(dignity, f.name))

            put(
                f"chart-{i}-perfection",
                f"{pf.querent.full_key} {pf.quesited.full_key} {number(pf.horizon_days)}"
                f" {len(pf.impediments)} {len(pf.translations)} {len(pf.collections)}",
            )
            ap = pf.application
            put(
                f"chart-{i}-perfection-application",
                "-"
                if ap is None
                else f"{ap.aspect.key} {number(ap.days)} {ap.applying.full_key} {ap.kind.key}"
                f" {number(ap.gap_deg)} {int(ap.within_moieties)}",
            )
            sp = pf.separation
            put(f"chart-{i}-perfection-separation", "-" if sp is None else f"{sp.aspect.key} {number(sp.past_deg)}")
            wy = pf.ways
            put(
                f"chart-{i}-perfection-ways",
                f"{wy.querent.house} {dignities_held(wy.querent.dignity)} {wy.quesited.house} {dignities_held(wy.quesited.dignity)}"
                f" {int(wy.mutual_by_house)} {commas(g.full_key for g in wy.infortunes_between)}"
                f" {int(wy.moon_relays)} {int(wy.quesited_in_ascendant)} {commas(w.key for w in wy.held)}",
            )
            for n, im in enumerate(pf.impediments):
                put(
                    f"chart-{i}-perfection-impediment-{n}",
                    f"{im.kind.key} {im.significator.full_key} {im.third.full_key if im.third else '-'}"
                    f" {im.aspect.key} {number(im.days)}",
                )
            for n, tr in enumerate(pf.translations):
                put(
                    f"chart-{i}-perfection-translation-{n}",
                    f"{tr.translator.full_key} {tr.from_.full_key} {tr.to.full_key} {tr.separating.aspect.key}"
                    f" {number(tr.separating.past_deg)} {tr.aspect.key} {number(tr.days)} {dignities_held(tr.received)}",
                )
            for n, co in enumerate(pf.collections):
                put(
                    f"chart-{i}-perfection-collection-{n}",
                    f"{co.collector.full_key} {co.from_querent.aspect.key} {number(co.from_querent.days)}"
                    f" {co.from_quesited.aspect.key} {number(co.from_quesited.days)}"
                    f" {dignities_held(co.collector_in_querent)} {dignities_held(co.collector_in_quesited)}"
                    f" {dignities_held(co.querent_in_collector)} {dignities_held(co.quesited_in_collector)}",
                )
            put(
                f"chart-{i}-perfection-rules",
                f"{','.join(number(orb) for orb in pf.rules.orbs_deg)} {int(pf.rules.within_sign)}",
            )
            pr = chart.progressions
            assert pr is not None and pr.progressed is not None and pr.directed is not None
            assert pr.contacts is not None
            pg, dr = pr.progressed, pr.directed
            put(
                f"chart-{i}-progressed",
                f"{number(pg.life)} {number(pg.sky)} {number(pg.armc_deg)}"
                f" {number(pg.angles.ascendant_deg)} {number(pg.angles.midheaven_deg)}",
            )
            for n, gr in enumerate(pg.grahas):
                put(
                    f"chart-{i}-progressed-graha-{n}",
                    f"{gr.graha.full_key} {number(gr.longitude_deg)} {number(gr.tropical_deg)}"
                    f" {number(gr.speed_deg_per_day)}",
                )
            put(
                f"chart-{i}-directed",
                f"{number(dr.arc_deg)} {number(dr.ascendant_deg)} {number(dr.midheaven_deg)}",
            )
            for n, directed in enumerate(dr.planets):
                put(f"chart-{i}-directed-graha-{n}", f"{directed.graha.full_key} {number(directed.longitude_deg)}")
            put(f"chart-{i}-progressed-contact-count", str(len(pr.contacts)))
            for n, ct in enumerate(pr.contacts):
                put(
                    f"chart-{i}-progressed-contact-{n}",
                    f"{number(ct.life)} {number(ct.sky)} {ct.graha.full_key} {natal_key(ct.to)} {ct.angle} {ct.motion.key}",
                )
            western = chart.western_aspects
            assert western is not None
            put(f"chart-{i}-western-aspect-count", str(len(western)))
            for n, row in enumerate(western):
                put(
                    f"chart-{i}-western-aspect-{n}",
                    f"{row.first.full_key} {row.second.full_key} {row.aspect.key} {number(row.apart_deg)} "
                    f"{number(row.from_exact_deg)} {number(row.orb_deg)} {int(row.applying)}",
                )
            declined = chart.declinations
            assert declined is not None
            put(
                f"chart-{i}-declinations",
                f"{number(declined.obliquity_deg)} {number(declined.lagna_deg)} {number(declined.midheaven_deg)}",
            )
            for declined_at in declined.grahas:
                put(f"chart-{i}-declination-{declined_at.graha.full_key}", number(declined_at.declination_deg))
            parallels = chart.parallels
            assert parallels is not None
            put(f"chart-{i}-parallel-count", str(len(parallels)))
            for n, pair in enumerate(parallels):
                put(
                    f"chart-{i}-parallel-{n}",
                    f"{pair.first.full_key} {pair.second.full_key} {int(pair.contrary)} "
                    f"{number(pair.apart_deg)} {number(pair.orb_deg)}",
                )
            reflected = chart.antiscia
            assert reflected is not None
            for reflection in reflected.points:
                put(
                    f"chart-{i}-antiscion-{reflection.graha.full_key}",
                    f"{number(reflection.antiscion_deg)} {number(reflection.contrantiscion_deg)}",
                )
            put(
                f"chart-{i}-antiscia-unpaired",
                ",".join(one.full_key for one in reflected.unpaired) or "-",
            )
            put_antiscion_rows(f"chart-{i}-antiscia", reflected.pairs)
            assert reflected.cusp_system is not None
            put(f"chart-{i}-antiscia-cusps", f"{reflected.cusp_system.full_key} {len(reflected.on_cusps)}")
            for n, upon in enumerate(reflected.on_cusps):
                put(f"chart-{i}-antiscia-cusp-{n}", f"{upon.graha.full_key} {upon.house} {int(upon.contrary)}")
            houses = chart.western_houses
            assert houses is not None
            put(
                f"chart-{i}-western-houses",
                f"{houses.system.full_key} {number(houses.ascendant_deg)} {number(houses.reach_deg)} {len(houses.planets)}",
            )
            for n, western_cusp in enumerate(houses.cusps_deg, start=1):
                put(f"chart-{i}-western-cusp-{n}", number(western_cusp))
            for counted in houses.planets:
                put(f"chart-{i}-western-house-{counted.graha.full_key}", f"{counted.house} {int(counted.with_ascendant)}")
            matched = chart.matching
            assert matched is not None
            put_ashta(f"chart-{i}", matched)
            ten = chart.porutham
            assert ten is not None
            put_porutham(f"chart-{i}", ten)
            mars = chart.kuja
            assert mars is not None
            for who, side in (("bride", mars.bride), ("groom", mars.groom)):
                readings = " ".join(f"{r.reference} {r.house} {int(r.in_houses)}" for r in side.readings)
                put(f"chart-{i}-kuja-{who}", f"{readings} {int(side.dosha)}")
            put(f"chart-{i}-kuja", str(int(mars.both)))
            doshas = chart.marriage_doshas
            assert doshas is not None
            put(f"chart-{i}-doshas", str(len(doshas)))
            for n, dosha in enumerate(doshas):
                named = "NONE" if dosha.koota is None else dosha.koota.full_key
                carrier = "NONE" if dosha.side is None else dosha.side.key
                put(f"chart-{i}-dosha-{n}", f"{dosha.system.key} {named} {carrier} {int(dosha.lifted)}")
            fifth = chart.harmonic
            assert fifth is not None
            put(f"chart-{i}-harmonic", f"{fifth.harmonic} {len(fifth.points)} {len(fifth.rows)}")
            for raised in fifth.points:
                put(f"chart-{i}-harmonic-{harmonic_key(raised.point)}", f"{number(raised.longitude_deg)} {raised.house}")
            for n, meeting in enumerate(fifth.rows):
                put(
                    f"chart-{i}-harmonic-row-{n}",
                    f"{harmonic_key(meeting.first)} {harmonic_key(meeting.second)} {number(meeting.apart_deg)} "
                    f"{meeting.multiple} {number(meeting.orb_deg)}",
                )
            between = chart.midpoints
            assert between is not None
            put(f"chart-{i}-midpoint-count", str(len(between)))
            for n, equal in enumerate(between):
                put(
                    f"chart-{i}-midpoint-{n}",
                    f"{equal.first.full_key} {equal.second.full_key} {equal.middle.full_key} {int(equal.far)} "
                    f"{number(equal.distance_deg)} {number(equal.from_axis_deg)} {number(equal.orb_deg)}",
                )
            synastry = chart.synastry
            assert synastry is not None
            put(f"chart-{i}-synastry-count", str(len(synastry)))
            for n, across in enumerate(synastry):
                put(
                    f"chart-{i}-synastry-{n}",
                    f"{natal_key(across.first)} {natal_key(across.second)} {across.aspect.key} "
                    f"{number(across.apart_deg)} {number(across.from_exact_deg)} {number(across.orb_deg)}",
                )
            levelled = chart.synastry_parallels
            assert levelled is not None
            put(f"chart-{i}-synastry-parallel-count", str(len(levelled)))
            for n, across_level in enumerate(levelled):
                put(
                    f"chart-{i}-synastry-parallel-{n}",
                    f"{natal_key(across_level.first)} {natal_key(across_level.second)} {int(across_level.contrary)} "
                    f"{number(across_level.apart_deg)} {number(across_level.orb_deg)}",
                )
            across_reflected = chart.synastry_antiscia
            assert across_reflected is not None
            put_antiscion_rows(f"chart-{i}-synastry-antiscia", across_reflected)
            across_equal = chart.synastry_midpoints
            assert across_equal is not None
            put(f"chart-{i}-synastry-midpoint-count", str(len(across_equal)))
            for n, between_pair in enumerate(across_equal):
                put(
                    f"chart-{i}-synastry-midpoint-{n}",
                    f"{between_pair.first.full_key} {between_pair.second.full_key} {between_pair.middle.full_key} "
                    f"{int(between_pair.partners_pair)} {int(between_pair.far)} {number(between_pair.distance_deg)} "
                    f"{number(between_pair.from_axis_deg)} {number(between_pair.orb_deg)}",
                )
            composite = chart.synastry_composite
            assert composite is not None
            put(
                f"chart-{i}-composite",
                f"{number(composite.lagna_deg)} {number(composite.midheaven_deg)} "
                f"{int(composite.lagna_turned)} {len(composite.planets)}",
            )
            for n, middle in enumerate(composite.planets):
                put(
                    f"chart-{i}-composite-{n}",
                    f"{middle.graha.full_key} {number(middle.longitude_deg)} {number(middle.speed_deg_per_day)}",
                )
            put(
                f"chart-{i}-composite-cusps",
                "-" if composite.cusps_deg is None else " ".join(number(degree) for degree in composite.cusps_deg),
            )
            davison = chart.synastry_davison
            assert davison is not None
            put(
                f"chart-{i}-davison",
                f"{number(davison.instant)} {number(davison.place.latitude_deg)} {number(davison.place.longitude_deg)} "
                f"{number(davison.place.altitude_m)} {davison.utc_offset_seconds}",
            )
            vs = chart.vimshopaka
            assert vs is not None
            put(f"chart-{i}-vimshopaka", vs.scoring.key)
            for scored in vs.grahas:
                put(
                    f"chart-{i}-vimshopaka-{scored.graha.full_key}",
                    ",".join(
                        number(score)
                        for score in (scored.shadvarga, scored.saptavarga, scored.dashavarga, scored.shodashavarga)
                    ),
                )
            for j, dasha in enumerate(chart.dashas):
                key = f"chart-{i}-dasha-{j}"
                balance = dasha.balance
                put(key, dasha.system if isinstance(dasha.system, str) else dasha.system.full_key)
                put(f"{key}-seed", dasha.seed.full_key if dasha.seed else None)
                put(f"{key}-first-lord", dasha.first_lord.full_key)
                put(f"{key}-overflow", dasha.overflow)
                put(f"{key}-balance", balance.method.key if balance else None)
                put(f"{key}-remaining", balance.remaining if balance else None)
                put(f"{key}-balance-days", balance.days if balance else None)
                written = balance.written if balance else None
                put(
                    f"{key}-balance-written",
                    f"{written.years},{written.months},{written.days},{written.hours},{written.minutes}"
                    if written
                    else None,
                )
                put(f"{key}-moon-span-from", dasha.moon_span.from_jd if dasha.moon_span else None)
                put(f"{key}-moon-span-to", dasha.moon_span.to_jd if dasha.moon_span else None)
                put(f"{key}-depth", dasha.depth)
                put(f"{key}-periods", len(dasha.periods))
                for k, period in enumerate(dasha.periods):
                    if period.level > 2:
                        continue
                    sign = f" {period.sign.full_key}" if period.sign else ""
                    put(f"{key}-period-{k}", f"{period.path}{sign} {period.lord.full_key}")
                    put(f"{key}-period-{k}-from", period.span.from_jd)
                    put(f"{key}-period-{k}-to", period.span.to_jd)
                put(f"{key}-at", ",".join(period.path for period in dasha.at(chart.instant + 5000)))
            for v, varga in enumerate(chart.vargas):
                put(f"chart-{i}-varga-{v}", varga.varga.full_key)
                put(f"chart-{i}-varga-{v}-lagna-rashi", varga.lagna.rashi.full_key)
                put(f"chart-{i}-varga-{v}-lagna-part", varga.lagna.part)
                put(f"chart-{i}-varga-{v}-lagna-sign", varga.lagna.sign.full_key)
                for j, in_varga in enumerate(varga.grahas):
                    put(f"chart-{i}-varga-{v}-graha-{j}", in_varga.graha.full_key)
                    put(f"chart-{i}-varga-{v}-graha-{j}-rashi", in_varga.at.rashi.full_key)
                    put(f"chart-{i}-varga-{v}-graha-{j}-part", in_varga.at.part)
                    put(f"chart-{i}-varga-{v}-graha-{j}-sign", in_varga.at.sign.full_key)
            for j, graha in enumerate(chart.grahas):
                put(f"chart-{i}-graha-{j}", graha.graha.full_key)
                put(f"chart-{i}-graha-{j}-lon", graha.longitude_deg)
                put(f"chart-{i}-graha-{j}-lat", graha.latitude_deg)
                put(f"chart-{i}-graha-{j}-speed", graha.speed_deg_per_day)
                put(f"chart-{i}-graha-{j}-retro", graha.retrograde)
                put(f"chart-{i}-graha-{j}-house", graha.house.bhava)
                put(f"chart-{i}-graha-{j}-house-method", graha.house.method.full_key)
                put(f"chart-{i}-graha-{j}-placement", graha.placement.bhava)
            # Uranus, Neptune and Pluto, which the request asks beside the nine.
            for j, outer in enumerate(chart.outer):
                put(
                    f"chart-{i}-outer-{j}",
                    f"{outer.graha.full_key} {number(outer.longitude_deg)} {number(outer.latitude_deg)} "
                    f"{number(outer.speed_deg_per_day)} {outer.house.bhava} {outer.placement.bhava}",
                )
            for k, bhava in enumerate(chart.houses):
                put(f"chart-{i}-house-{k}-madhya", bhava.madhya_deg)
                put(f"chart-{i}-house-{k}-sandhi", bhava.sandhi_deg)
            for k, bhava in enumerate(chart.chalit):
                put(f"chart-{i}-chalit-{k}-madhya", bhava.madhya_deg)

        # `found` is the batch of one unwrapped, and must agree with it.
        # **The annual charts, under all three readings.** One crossing
        # each, because `varsha_json` names one reading per request — and
        # all three, because a reading that crossed as another would be
        # invisible in a report that only printed the default.
        # Each reading also asks the sixteen yogas a different way, so all
        # three ways cross: every matter under the source's readings, every
        # matter under Tambira's "some authorities", and no matter at all.
        def pair_said(p: Any) -> str:
            yoga = p.yoga.key if p.yoga is not None else "-"
            return f"{p.faster.full_key}>{p.slower.full_key}:{p.drishti.key}:{yoga}:{p.apart_deg:.6f}"

        def clauses_said(clauses: Any) -> str:
            return "+".join(c.key for c in clauses) if clauses else "none"

        def held_said(h: Any) -> str:
            return ":".join([
                h.yoga.key,
                h.through.full_key if h.through is not None else "-",
                h.entering.full_key if h.entering is not None else "-",
                "pair" if h.between is not None else "-",
                "/".join(pair_said(leg) for leg in h.legs) if h.legs is not None else "-",
                (
                    f"{clauses_said(h.afflictions.lagnesha)}/{clauses_said(h.afflictions.karyesha)}"
                    if h.afflictions is not None
                    else "-"
                ),
            ])

        def saham_said(p: Any) -> str:
            """One saham as every runner prints it."""
            axis = "null" if p.in_node_axis is None else str(p.in_node_axis).lower()
            seven = " ".join(f"{s.drishti.key}/{s.relation.key}/{int(s.company)}" for s in p.seven)
            return " | ".join([
                f"{p.longitude_deg:.6f} {p.sign.full_key} {p.lord.full_key} {p.house} {str(p.added_sign).lower()}",
                f"S:{','.join(c.key for c in p.strong)} W:{','.join(c.key for c in p.weak)}",
                f"{p.lord_vishwa} {p.lord_harsha.key} {axis}",
                seven,
            ])

        for reading in ("SIDEREAL", "TROPICAL", "MEAN"):
            varsha: VarshaRequest = {"reading": reading, "through": 12, "place": "birth"}
            # The sahams likewise: every one under the source's rules,
            # every one under each rival rule, and none.
            # And the annual dashas: every one under the sources' readings,
            # every one under a rival clock, balance and birth period three
            # levels deep, and none.
            if reading != "MEAN":
                varsha["matters"] = "all"
                varsha["sahams"] = "all"
                varsha["dashas"] = "all"
            if reading == "TROPICAL":
                varsha["yogas"] = {"tambira": "EITHER_LORD"}
                varsha["saham_rules"] = {"add_sign": "SIGNS", "houses": "EQUAL", "roga": "SATURN"}
                varsha["dasha_rules"] = {
                    "clock": "EVEN",
                    "balance": "ENTRY_MOON",
                    "birth_period": "ELAPSED",
                    "depth": 3,
                }
            years = geo.chart.found_many(
                instants=[2460482.5, 2460600.25],
                place=place,
                utc_offset_seconds=20700,
                varsha=varsha,
            )
            for i, chart in enumerate(years):
                returns = chart.praveshas
                put(f"chart-{i}-varsha-{reading}-count", len(returns))
                for point in chart.sahams:
                    put(f"chart-{i}-varsha-{reading}-natal-saham-{point.saham.key}", saham_said(point))
                for pravesha in returns:
                    stem = f"chart-{i}-varsha-{reading}-{pravesha.year}"
                    put(stem, pravesha.instant)
                    put(f"{stem}-muntha", pravesha.muntha.sign.full_key)
                    put(f"{stem}-muntha-lord", pravesha.muntha.lord.full_key)
                    put(f"{stem}-muntha-deg", pravesha.muntha.longitude_deg)
                    annual = pravesha.annual
                    assert annual is not None
                    put(f"{stem}-annual-lagna", annual.lagna_deg)
                    put(f"{stem}-annual-by-day", annual.by_day)
                    b = annual.office_bearers
                    put(
                        f"{stem}-annual-bearers",
                        " ".join(
                            lord.full_key
                            for lord in (b.muntha, b.janma_lagna, b.varsha_lagna, b.tri_rashi, b.dina_ratri)
                        ),
                    )
                    year_lord = annual.year_lord
                    put(f"{stem}-year-lord", year_lord.graha.full_key)
                    put(f"{stem}-year-lord-chosen", year_lord.chosen.key)
                    put(f"{stem}-year-lord-bala", str(year_lord.vishwa))
                    put(
                        f"{stem}-yogas",
                        " ".join(
                            f"{p.faster.full_key}>{p.slower.full_key}:"
                            f"{p.drishti.key}:{p.yoga.key}:{p.apart_deg:.6f}"
                            for p in annual.yogas
                        ),
                    )
                    put(
                        f"{stem}-states",
                        f"R:{','.join(g.full_key for g in annual.retrograde)} "
                        f"C:{','.join(g.full_key for g in annual.combust)}",
                    )
                    for matter in annual.matters:
                        asked = f"{stem}-matter-{matter.house}"
                        put(
                            asked,
                            f"{matter.sign.full_key} {matter.lagnesha.full_key}>"
                            f"{matter.karyesha.full_key} {str(matter.same_lord).lower()}",
                        )
                        put(f"{asked}-pair", pair_said(matter.between) if matter.between is not None else "-")
                        put(f"{asked}-unanswered", ",".join(y.key for y in matter.unanswered))
                        put(f"{asked}-held", " ".join(held_said(h) for h in matter.held))
                    for point in annual.sahams:
                        put(f"{stem}-saham-{point.saham.key}", saham_said(point))
                    for year_dasha in annual.dashas:
                        said = f"{stem}-dasha-{year_dasha.system.full_key}"
                        ring = year_dasha.ring
                        left = "null" if ring.remaining is None else f"{ring.remaining:.9f}"
                        put(
                            said,
                            f"{year_dasha.seed.full_key if year_dasha.seed is not None else '-'} {ring.first} {left} "
                            f"{year_dasha.year.from_jd:.9f} {year_dasha.year.to_jd:.9f} | "
                            + " ".join(
                                f"{s.lord.full_key}/{s.sign.full_key if s.sign is not None else '-'}/{s.weight:.3f}"
                                for s in ring.shares
                            ),
                        )
                        put(
                            f"{said}-periods",
                            " ".join(
                                f"{p.path}:{p.lord.full_key}:{p.sign.full_key if p.sign is not None else '-'}:"
                                f"{p.span.from_jd:.9f}:{p.span.to_jd:.9f}"
                                for p in year_dasha.periods
                            ),
                        )
                    put(
                        f"{stem}-harsha",
                        " ".join(
                            f"{h.graha.full_key}:{h.house}:{int(h.sthana)}{int(h.uchcha_swakshetra)}"
                            f"{int(h.stri_purusha)}{int(h.dina_ratri)}:{h.total}:{h.grade.key}"
                            for h in annual.harsha
                        ),
                    )
                    put(
                        f"{stem}-year-claims",
                        " ".join(
                            f"{c.graha.full_key}:{c.vishwa}:{c.portfolios}:{str(c.aspects_lagna).lower()}"
                            for c in year_lord.claims
                        ),
                    )

        single = geo.chart.found(
            instant=2460482.5, place=place, utc_offset_seconds=20700
        )
        put("chart-single-lagna", single.lagna_deg)
        put("chart-single-agrees", single.lagna_deg == charts.at(0).lagna_deg)

        # ── An almanac ────────────────────────────────────────────────
        # Three days, because a day's lists are ragged and two
        # consecutive days with the same counts would not exercise the
        # offsets.
        week = geo.almanac.of(
            from_date=date(Calendar.GREGORIAN, 2024, 6, 17),
            to_date=date(Calendar.GREGORIAN, 2024, 6, 19),
            place=place,
            utc_offset_seconds=20700,
        )
        put("almanac-days", len(week))
        put("almanac-calendar", week.calendar.full_key)
        put("almanac-place-lat", week.decoded.latitude_deg)
        put("almanac-model-fnv", fnv(week.model))
        put("almanac-provenance-fnv", fnv(week.provenance_json))

        for almanac_day in week:
            i = almanac_day.index
            put(f"day-{i}-content-hash", almanac_day.provenance.content_hash)
            put_day(f"day-{i}", almanac_day.day)
            put(f"day-{i}-window-from", almanac_day.window.from_jd)
            put(f"day-{i}-window-to", almanac_day.window.to_jd)
            put(f"day-{i}-month", almanac_day.month.month.full_key)
            put(f"day-{i}-amanta", almanac_day.month.amanta.full_key)
            put(f"day-{i}-purnimanta", almanac_day.month.purnimanta.full_key)
            put(f"day-{i}-paksha", almanac_day.month.paksha.full_key)
            put(f"day-{i}-convention", almanac_day.month.convention.key)
            put(f"day-{i}-month-kind", almanac_day.month.kind.key)
            put(f"day-{i}-ayana", almanac_day.ayana.full_key)
            put(f"day-{i}-ritu", almanac_day.ritu.full_key)
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
                    put(f"day-{i}-{name}-{j}-sunrises", span.sunrises.key)
                    put(
                        f"day-{i}-{name}-{j}-ends",
                        f"{span.ends.ghati}-{span.ends.pala}-{span.ends.vipala}",
                    )
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
                put(f"day-{i}-moon-{j}-kind", "RISE" if event.rise else "SET")
                put(f"day-{i}-moon-{j}-instant", event.instant)
            for j, held in enumerate(almanac_day.muhurta_yogas):
                put(f"day-{i}-yoga-held-{j}", held.yoga.full_key)
                put(
                    f"day-{i}-yoga-held-{j}-cause",
                    "VARA_NAKSHATRA"
                    if held.tithi is None
                    else "VARA_TITHI_NAKSHATRA",
                )
                put(
                    f"day-{i}-yoga-held-{j}-tithi",
                    "none" if held.tithi is None else held.tithi.full_key,
                )

        # `almanac_day` is the range of one unwrapped, and must agree.
        one_day = geo.almanac.day(
            date=date(Calendar.GREGORIAN, 2024, 6, 17),
            place=place,
            utc_offset_seconds=20700,
        )
        put("almanac-single-agrees", one_day.day.sunrise == week.at(0).day.sunrise)

        # ── A muhurta search ──────────────────────────────────────────
        # Both rankings over 2024-11-25..27: the texts bar the windows
        # for different reasons and the baseline scores them; and a
        # thread ceremony, whose rules want grahas out of houses.
        native: MuhurtaNative = {"star": "ROHINI", "moonSign": "TAURUS", "lagna": "LEO"}
        searches: tuple[tuple[str, MuhurtaRequest], ...] = (
            ("raman", {"rules": "RAMAN_MARRIAGE", "ranking": "TEXTS"}),
            ("baseline", {"rules": "BASELINE_MARRIAGE", "ranking": "BASELINE"}),
            ("upanayana", {"rules": "RAMAN_UPANAYANA", "ranking": "TEXTS"}),
        )
        for name, search in searches:
            muhurta = geo.almanac.of(
                from_date=date(Calendar.GREGORIAN, 2024, 11, 25),
                to_date=date(Calendar.GREGORIAN, 2024, 11, 27),
                place=place,
                utc_offset_seconds=20700,
                muhurta={**search, "native": native, "daysWithWindows": 3, "most": 12},
            ).muhurta
            assert muhurta is not None
            put_muhurta(f"muhurta-{name}", muhurta)

        # ── Festivals ─────────────────────────────────────────────────
        # The shipped pack, the pack amended by a rule of the consumer's
        # own (Lakshmi puja on whichever day holds the new moon at
        # sunrise), and the Nepal pack with a following rule of the
        # consumer's own (two days after Lakshmi puja), over
        # 2024-10-10..11-03.
        own_rule = {
            "key": "LAKSHMI_PUJA",
            "source": "the tithi at sunrise",
            "month": "masa.ASHWINA",
            "tithi": "tithi.AMAVASYA",
            "at": {"window": "SUNRISE"},
            "decide": [],
            "otherwise": "LATER",
        }
        own_following = {
            "key": "TWO_AFTER",
            "source": "two days after Lakshmi puja",
            "after": "LAKSHMI_PUJA",
            "days": 2,
        }
        packs: tuple[tuple[str, FestivalRequest], ...] = (
            ("shipped", {"rules": "DHARMASINDHU"}),
            ("amended", {"rules": ["DHARMASINDHU", own_rule]}),
            ("nepal", {"rules": ["NEPAL", own_following]}),
        )
        for name, pack in packs:
            festivals = geo.almanac.of(
                from_date=date(Calendar.GREGORIAN, 2024, 10, 10),
                to_date=date(Calendar.GREGORIAN, 2024, 11, 3),
                place=place,
                utc_offset_seconds=20700,
                festivals=pack,
            ).festivals
            assert festivals is not None
            put_festivals(f"festivals-{name}", festivals)

        # The lunar years over 2024-03-20..04-20, which holds a Chaitra
        # Shukla Pratipada: two years, their bounds and their Jovian years.
        lunar = geo.almanac.of(
            from_date=date(Calendar.GREGORIAN, 2024, 3, 20),
            to_date=date(Calendar.GREGORIAN, 2024, 4, 20),
            place=place,
            utc_offset_seconds=20700,
            years=True,
        ).years
        assert lunar is not None
        put("years-count", len(lunar.value))
        put("years-hash", lunar.provenance.content_hash)
        for k, year in enumerate(lunar.value):
            put(
                f"years-{k}",
                " ".join(
                    (
                        year.samvatsara.full_key,
                        year.count,
                        str(year.vikrama),
                        str(year.shaka),
                        number(year.opened),
                        number(year.began),
                        number(year.ended),
                        "-" if year.lupta is None else year.lupta.full_key,
                    )
                ),
            )
            put(
                f"years-{k}-jovian",
                listed(f"{j.member.full_key}:{j.count}:{number(j.from_)}" for j in year.jovian),
            )

        # The Nepal Sambat dates over 2024-10-30..11-03, which holds
        # Kartika's new moon, where the year turns.
        nepal_sambat = geo.almanac.of(
            from_date=date(Calendar.GREGORIAN, 2024, 10, 30),
            to_date=date(Calendar.GREGORIAN, 2024, 11, 3),
            place=place,
            utc_offset_seconds=20700,
            nepal_sambat=True,
        ).nepal_sambat
        assert nepal_sambat is not None
        put("nepal-sambat-hash", nepal_sambat.provenance.content_hash)
        put(
            "nepal-sambat",
            listed(f"{d.year}:{d.month}:{d.kind.key}:{d.paksha.full_key}" for d in nepal_sambat.value),
        )

    # ── The eclipses ──────────────────────────────────────────────────
    # September 2025 at Kathmandu over the built-in sky, which the test
    # provider cannot complete: a total lunar eclipse seen whole and a
    # partial solar one the place does not see (`03-design/eclipses.md`).
    with teistro.context(profile="nepali-default", ephemeris=Ephemeris.BUILTIN) as builtin:
        eclipses = builtin.almanac.of(
            from_date=date(Calendar.GREGORIAN, 2025, 9, 1),
            to_date=date(Calendar.GREGORIAN, 2025, 9, 30),
            place=place,
            utc_offset_seconds=20700,
            eclipses=True,
        ).eclipses
    assert eclipses is not None

    def eclipse_number(value: Optional[float]) -> str:
        return "-" if value is None else number(value)

    def eclipse_moment(m: Optional[EclipseMoment]) -> str:
        return "-" if m is None else f"{number(m.at)}@{number(m.altitude_deg)}"

    def eclipse_seen(stretch: Optional[EclipseSeen]) -> str:
        return "-" if stretch is None else f"{number(stretch.from_)}..{number(stretch.to)}"

    put("eclipses-hash", eclipses.provenance.content_hash)
    put("eclipses-count", f"{len(eclipses.value.lunar)} {len(eclipses.value.solar)}")
    for k, lunar_eclipse in enumerate(eclipses.value.lunar):
        lunar_found, lunar_view = lunar_eclipse.eclipse, lunar_eclipse.here
        contacts = lunar_found.contacts
        put(
            f"eclipses-lunar-{k}",
            " ".join(
                (
                    lunar_found.kind.full_key,
                    lunar_found.shadow,
                    number(lunar_found.greatest),
                    number(lunar_found.gamma),
                    number(lunar_found.umbral_magnitude),
                    number(lunar_found.penumbral_magnitude),
                )
            ),
        )
        put(f"eclipses-lunar-{k}-contacts", " ".join(eclipse_number(at) for at in (contacts.p1, contacts.u1, contacts.u2, contacts.u3, contacts.u4, contacts.p4)))
        put(
            f"eclipses-lunar-{k}-here",
            " ".join([eclipse_moment(m) for m in (lunar_view.p1, lunar_view.u1, lunar_view.u2, lunar_view.greatest, lunar_view.u3, lunar_view.u4, lunar_view.p4)] + [eclipse_seen(lunar_view.seen), eclipse_seen(lunar_view.umbral_seen)]),
        )
    for k, solar_eclipse in enumerate(eclipses.value.solar):
        solar_found, solar_view = solar_eclipse.eclipse, solar_eclipse.here
        put(
            f"eclipses-solar-{k}",
            " ".join(
                (
                    solar_found.kind.full_key,
                    number(solar_found.greatest),
                    number(solar_found.gamma),
                    number(solar_found.magnitude),
                    number(solar_found.point.latitude),
                    number(solar_found.point.longitude),
                )
            ),
        )
        put(
            f"eclipses-solar-{k}-here",
            "-"
            if solar_view is None
            else " ".join(
                [solar_view.kind.full_key, number(solar_view.magnitude), number(solar_view.obscuration)]
                + [eclipse_moment(m) for m in (solar_view.first, solar_view.second, solar_view.third, solar_view.fourth, solar_view.maximum)]
                + [eclipse_seen(solar_view.seen)]
            ),
        )

    # ── The surface's shape ───────────────────────────────────────────
    #
    # The lines above compare what the bindings ANSWER. These compare
    # where an operation LIVES: every key is the canonical
    # `area.operation` path, and the member each binding references
    # beside it is its own spelling of it. A binding that moved an
    # operation to another area, or renamed one, prints a key the others
    # do not and the gate fails -- which is what
    # `03-design/surface-areas.md` asks of this runner, and what
    # `check-parity` could not see before.
    #
    # Referenced rather than called, and referenced by attribute rather
    # than by `getattr`, so the strict type check reads them too.
    for path, member in (
        ("calendar.date_of", ctx.calendar.date_of),
        ("calendar.fixed_of", ctx.calendar.fixed_of),
        ("calendar.convert", ctx.calendar.convert),
        ("calendar.weekday_of", ctx.calendar.weekday_of),
        ("calendar.month_length", ctx.calendar.month_length),
        ("calendar.is_leap", ctx.calendar.is_leap),
        ("time.resolve", ctx.time.resolve),
        ("time.civil_of", ctx.time.civil_of),
        ("time.convert", ctx.time.convert),
        ("time.delta_t", ctx.time.delta_t),
        ("intl.locale", ctx.intl.locale),
        ("intl.render", ctx.intl.render),
        ("intl.has", ctx.intl.has),
        ("intl.transliterate", ctx.intl.transliterate),
        ("intl.entity", ctx.intl.entity),
        ("intl.messages", ctx.intl.messages),
        ("intl.load_pack", ctx.intl.load_pack),
        ("keys.id", ctx.keys.id),
        ("keys.name", ctx.keys.name),
        ("matching.naam", ctx.matching.naam),
        ("frame.canonical", ctx.frame.canonical),
        ("frame.pack", ctx.frame.pack),
        ("frame.unpack", ctx.frame.unpack),
        ("chart.layout", ctx.chart.layout),
        ("chart.found", ctx.chart.found),
        ("chart.found_many", ctx.chart.found_many),
        ("almanac.of", ctx.almanac.of),
        ("almanac.day", ctx.almanac.day),
        ("engine.names", ctx.engine.names),
        ("engine.signature", ctx.engine.signature),
        ("engine.call", ctx.engine.call),
        ("engine.call_json", ctx.engine.call_json),
        ("engine.manifest", ctx.engine.manifest),
        ("engine.manifest_json", ctx.engine.manifest_json),
        ("(root).engine", ctx.engine),
        ("(root).positions", ctx.positions),
        ("(root).profile", ctx.profile),
        ("(root).settings", ctx.settings),
        ("(root).settings_json", ctx.settings_json),
        ("(root).settings_hash", ctx.settings_hash),
        ("(root).dispose", ctx.close),
    ):
        put(f"surface.{path}", "missing" if member is None else "present")

    for key in sorted(report):
        sys.stdout.write(f"{key}\t{report[key]}\n")
    ctx.close()


if __name__ == "__main__":
    main()

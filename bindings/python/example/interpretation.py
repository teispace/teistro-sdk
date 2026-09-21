"""An interpretation: the same birth record, said in two languages.

`chart_reading.py` read the chart in full. This is what comes after: a
**composer** turns what was read into a narrative plan — an ordered list of
message keys and their slots — and the locale engine says it. The plan
holds no words at all, which is why one plan says the same chart in English
and in Nepali without the composer knowing either language
(`docs/03-design/plans-at-the-boundary.md`).

What it teaches:

1.  **The plan comes back in the same crossing as the chart.** `interpret`
    is an argument to `found`, like `rules` and `vargas`; nothing is
    founded or evaluated twice, and a chart with no `interpret` pays
    nothing.
2.  **An item's `params` are `intl.render`'s own params.** There is no
    conversion step in this file, and there is none in the binding either:
    that is the whole design.
3.  **One plan, every locale.** The same plan is said twice below, and the
    rendering says which locale answered — so you can prove the locale had
    the message rather than quietly falling back to English.
4.  **A reading says what the rules found.** `readings` needs `rules`
    beside it, because it composes their answers rather than re-deriving
    them; asking for it alone is refused, by name.
5.  **A composer says what it can say.** The lagna stands in the chart and
    is in no placement item: those messages read a graha, and the lagna is
    a point. A composer that guessed would be worse than one that is quiet.

The record is `birth_chart.py`'s own, so the examples can be read side by
side.

Run it:

    PYTHONPATH=. python3 example/interpretation.py
"""

from __future__ import annotations

import json

from teistro import Calendar, Ephemeris, Teistro, TeistroError, at, date, iana_zone
from teistro._ffi import Altitude, Latitude, Longitude, Observer


def main() -> None:
    teistro = Teistro.open()
    with teistro.context(
        profile="nepali-default",
        locale="en-Latn",
        ephemeris=Ephemeris.BUILTIN,
    ) as ctx:
        born = date(Calendar.BIKRAM_SAMBAT, 2042, 9, 17)
        when = ctx.time.resolve(at(born, hour=0, minute=20), iana_zone("Asia/Kathmandu"))
        place = Observer(
            latitude_deg=Latitude(27.7172),
            longitude_deg=Longitude(85.324),
            altitude_m=Altitude(1400),
        )

        # ── The chart, and what it has to say, in one call ─────────────
        chart = ctx.chart.found(
            instant=when.instant_jd_utc,
            place=place,
            utc_offset_seconds=when.offset_seconds,
            # The rules whose answers the readings composer will say. The
            # sections they read are computed whether or not they are
            # asked for here.
            rules={"shipped": ["nabhasas", "arishtas"]},
            interpret={
                "placements": True,
                "readings": True,
                "strength": True,
                "houses": True,
                "positions": True,
                "aspects": True,
            },
        )
        plans = chart.plans
        assert plans is not None
        placements, readings = plans["placements"], plans["readings"]
        weights, ruled = plans["strength"], plans["houses"]
        degrees, looks = plans["positions"], plans["aspects"]

        print("BS 2042-09-17  00:20  Kathmandu")
        print(
            f"plan     {len(placements)} placement items, "
            f"{len(readings)} reading items, {len(weights)} strengths, "
            f"{len(ruled)} lordships, {len(degrees)} positions, "
            f"{len(looks)} drishtis"
        )
        keys = dict.fromkeys(
            item["key"]
            for item in [*placements, *readings, *weights, *ruled, *degrees, *looks]
        )
        print(f"keys     {', '.join(keys)}")

        # ── The same plan, said twice ──────────────────────────────────
        # Nothing between an item and the renderer: `item["params"]` is
        # what `render` takes, so this loop is the whole consumer story.
        for locale in ("en-Latn", "ne-Deva-NP"):
            ctx.intl.locale = locale
            print(f"\n{locale}")
            for item in [*placements, *weights, *ruled, *degrees, *looks]:
                said = ctx.intl.render(item["key"], item["params"])
                print(f"  {said.text}{'  (fallback)' if said.is_fallback else ''}")
            # A reading names its rule in a slot the message does not
            # print, so a consumer can group a plan by rule.
            for item in readings:
                said = ctx.intl.render(item["key"], item["params"])
                print(f"  {item['params']['rule']}: {said.text}")

        # ── What it does not say, and what it refuses ──────────────────
        lagna = any("LAGNA" in json.dumps(item["params"]) for item in placements)
        print(f"\nthe lagna is in the placements: {lagna}")
        # The Shadbala says whether a graha reaches its required rupas; no
        # locale says it, so the plan does not either.
        strong = any("strong" in json.dumps(item["params"]) for item in weights)
        print(f'a strength item claims "strong": {strong}')
        # A bhava knows its sign, its cusps and its class; no locale says
        # any of them, so the houses plan says the lord and stops there.
        signed = any("rashi" in json.dumps(item["params"]) for item in ruled)
        print(f"a houses item claims a sign: {signed}")
        # A position crosses as a number: the degree signs a reader sees
        # are the locale's rendering, never a string the composer wrote.
        angled = any("\u00b0" in json.dumps(item["params"]) for item in degrees)
        print(f"a position item carries a rendered angle: {angled}")

        try:
            ctx.chart.found(
                instant=when.instant_jd_utc,
                place=place,
                utc_offset_seconds=when.offset_seconds,
                interpret={"readings": True},
            )
        except TeistroError as error:
            print(f"refused  {error.field}: {error}")


if __name__ == "__main__":
    main()

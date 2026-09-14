"""A chart reading: one call for the whole document a reader interprets.

`birth_chart.py` placed the grahas. A reading is what comes after: the
divisional charts, the houses, what each graha **is** rather than where it
is, which grahas look at which, and the derived points. Each is a section a
request asks for by name, and each is computed from the same founded chart
in the same crossing — so asking for all of them costs one call, and asking
for none of them costs nothing (`docs/03-design/chart-reading.md`).

What it teaches:

1.  **Sections are asked for.** `vargas`, `aspects`, `points`, `houses` and
    `state` are off by default, so a birth chart does not pay for
    twenty-one divisional charts it will not show.
2.  **Vargottama is a comparison, not a flag**: a graha whose navamsha sign
    is the sign it stands in. The layer gives both signs.
3.  **A dignity and a house are different questions**, answered by
    different sections: `states` says how a graha fares in its sign,
    `bhavas` who rules a house.
4.  **The drishti are ragged**: how many there are depends on where the
    grahas stand, not on how many grahas there are.
5.  **A drawing is geometry, not pixels**: each cell's outline in a unit
    square, the sign and house it shows and the grahas in it, so any
    renderer draws the same chart.

The record is `birth_chart.py`'s own, so the two can be read side by side.

Run it:

    PYTHONPATH=. python3 example/chart_reading.py
"""

from __future__ import annotations

from teistro import Calendar, Context, Ephemeris, Teistro, at, date, iana_zone
from teistro._ffi import Altitude, Latitude, Longitude, Observer
from teistro.catalogue import Catalogued, ChartLayout, Graha, Point, Rashi, Varga


def name(ctx: Context, member: Catalogued) -> str:
    """A catalogue member's name in the context's locale."""
    return ctx.intl.entity(member.full_key).name


def main() -> None:
    teistro = Teistro.open()
    with teistro.context(
        profile="nepali-default",
        locale="ne-Deva-NP",
        ephemeris=Ephemeris.BUILTIN,
    ) as ctx:
        born = date(Calendar.BIKRAM_SAMBAT, 2042, 9, 17)
        when = ctx.time.resolve(at(born, hour=0, minute=20), iana_zone("Asia/Kathmandu"))

        # ── One call for every section ─────────────────────────────────
        chart = ctx.chart.found(
            instant=when.instant_jd_utc,
            place=Observer(
                latitude_deg=Latitude(27.7172),
                longitude_deg=Longitude(85.324),
                altitude_m=Altitude(1400),
            ),
            utc_offset_seconds=when.offset_seconds,
            vargas=[Varga.D9, Varga.D10],
            aspects=True,
            points=True,
            houses=True,
            state=True,
            drawings=[(ChartLayout.NORTH_INDIAN, Varga.D9)],
        )
        navamsha, dasamsha = chart.vargas[0], chart.vargas[1]

        print("reading  BS 2042-09-17  00:20  Kathmandu")
        lagna = Rashi(int(chart.lagna_deg // 30))
        print(
            f"lagna    {name(ctx, lagna)} {chart.lagna_deg % 30:.4f}°"
            f"   D9 {name(ctx, navamsha.lagna.sign)}   D10 {name(ctx, dasamsha.lagna.sign)}"
        )

        # ── What each graha is ─────────────────────────────────────────
        print()
        print("graha        house  dignity          age            D9 sign      vargottama  combust")
        print("─" * 88)
        for j, state in enumerate(chart.states):
            in_navamsha = navamsha.grahas[j].at
            vargottama = "yes" if in_navamsha.sign == in_navamsha.rashi else "no"
            print(
                f"{name(ctx, state.graha):12} {state.house:5}  "
                f"{name(ctx, state.dignity):16} {name(ctx, state.age):14} "
                f"{name(ctx, in_navamsha.sign):12} {vargottama:11} "
                f"{state.combustion.burning.key}"
            )

        # ── The houses ─────────────────────────────────────────────────
        # The tenth house, by the houses service: the sign its middle falls
        # in, that sign's lord, and which kind of house it is.
        tenth = chart.bhavas[9]
        print()
        print(f"bhava 10 {name(ctx, tenth.sign)}, ruled by {name(ctx, tenth.lord)} ({tenth.quadrant.key})")

        # ── Which grahas look at the Moon ──────────────────────────────
        # `houses` counts inclusively from the looking graha's sign, so the
        # seventh is the house opposite it.
        print(f"drishti  {len(chart.aspects)} under {chart.batch.decoded.drishti_table}; on the Moon:")
        for drishti in chart.aspects:
            if drishti.to == Graha.MOON:
                print(
                    f"         {name(ctx, drishti.from_graha):12} house {drishti.houses:2} from it"
                    f"  {drishti.strength.key}"
                )

        # ── The derived points ─────────────────────────────────────────
        # Gulika is Saturn's portion of the day's arc, which is why a chart
        # with no day to divide has none — the section is ragged for that.
        gulika = next((found for found in chart.points if found.point == Point.GULIKA), None)
        where = (
            f"{name(ctx, gulika.sign)} {gulika.longitude_deg % 30:.4f}°" if gulika else "none"
        )
        print(f"gulika   {where}   {len(chart.points)} points")

        # ── The navamsha, drawn ────────────────────────────────────────
        # A North Indian chart keeps its houses still and moves the signs,
        # so the lagna is always the top diamond; the cell says which sign
        # landed there.
        drawn = chart.drawings[0]
        risen = next(cell for cell in drawn.cells if cell.lagna)
        print(
            f"drawing  {drawn.layout_key.rsplit(".", 1)[-1].lower()} {drawn.varga.key.lower()}: {len(drawn.cells)} cells, "
            f"lagna in house {risen.house} ({name(ctx, risen.sign)}), grahas there: {len(risen.bodies)}"
        )
        print(f"settings hash  {ctx.settings_hash[:16]}…")


if __name__ == "__main__":
    main()

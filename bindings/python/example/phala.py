"""What the chart *is*, read aloud: the state readings, loaded beside the
rule readings.

`readings.py` loaded a pack of readings for the yogas and doshas a chart
**holds**. This one loads the other half of that corpus: a reading for what
the chart *is* without holding anything — Jupiter in the first house, the
lagna's sign, the tithi, the nakshatra the Moon stands in
(`docs/03-design/state-readings.md`).

What it teaches:

1. **Two packs, one engine.** Each root builds one pack a locale, and a
   consumer loads the ones it wants. They are loaded in either order.
2. **A record gains forms; it does not lose them.** Both corpora describe
   some of the same subjects, and so may yours: a pack carrying one form
   adds that form and leaves the rest of the record standing.
   `loaded.merged` counts the records that kept something.
3. **A composer says nothing it has no words for.** `phala` asks the base
   locale for each subject and is silent where the answer is no, so a chart
   composes exactly as it did before until a pack is loaded.

The packs are the files `teistro-intl build` writes, one a locale, which
`cargo xtask check-parity` builds before it runs any example:

    cargo run -p teistro-intl -- --root packs/readings build --out target/packs/readings
    cargo run -p teistro-intl -- --root packs/states build --out target/packs/states
    PYTHONPATH=. python3 example/phala.py
"""

from __future__ import annotations

import os
from pathlib import Path

from teistro import Altitude, Ephemeris, Latitude, Longitude, Observer, Teistro

# The two corpora, each built from its source under `packs/` into packs of
# its own (`packs/README.md`).
CORPORA = ("readings", "states")

# Where the built packs are: `TEISTRO_PACKS`, or the repository's
# `target/packs`.
PACKS = Path(
    os.environ.get("TEISTRO_PACKS")
    or Path(__file__).resolve().parents[3] / "target" / "packs"
)


def packs_of(corpus: str) -> list[Path]:
    """Every pack a corpus built, one a locale, in name order."""
    return sorted((PACKS / corpus).glob("*.tpack"))


def shortened(text: str) -> str:
    """Enough of a passage to show it is there, without printing an essay."""
    return text[:88] + "…" if len(text) > 88 else text


def main() -> None:
    teistro = Teistro.open()
    with teistro.context(profile="nepali-default", ephemeris=Ephemeris.BUILTIN) as ctx:
        # ── The packs ────────────────────────────────────────────────
        for corpus in CORPORA:
            for file in packs_of(corpus):
                data = file.read_bytes()
                loaded = ctx.intl.load_pack(data)
                print(
                    f"{'packs/' + corpus:<15} {loaded.locale:<12} {loaded.entries:>5} records, "
                    f"{loaded.merged:>4} merged, {len(data):>7} bytes"
                )

        # ── The plan, said twice ─────────────────────────────────────
        # A composer is a member of `interpret` — `{"phala": True}` — off
        # unless asked for, so a chart says nothing new until a consumer
        # asks for it. The chart is founded with what the composer reads,
        # its states and the panchanga's limbs among them, in the same call.
        chart = ctx.chart.found(
            instant=2447995.4895833335,
            place=Observer(
                latitude_deg=Latitude(27.7172),
                longitude_deg=Longitude(85.324),
                altitude_m=Altitude(1400),
            ),
            utc_offset_seconds=20700,
            interpret={"phala": True},
        )
        assert chart.plans is not None
        plan = chart.plans["phala"]
        print(f"\n{len(plan)} items")
        for locale in ("en-Latn", "ne-Deva-NP"):
            ctx.intl.locale = locale
            print(f"\n{locale}")
            for item in plan[:4]:
                said = ctx.intl.render(item["key"], item["params"])
                fallback = "  (fallback)" if said.is_fallback else ""
                print(f"  {shortened(said.text)}{fallback}")

        # ── The record that two corpora describe ─────────────────────
        # `nakshatra-phala` says what the nakshatra portends and
        # `namakarana-nakshatra` what to name a child born under it. Both
        # are forms on the record the SDK already names, beside its own
        # `name` and `iast` — which is what the merge on load is for. A
        # pack's forms are read through `forms`, since a record's forms are
        # an open set.
        ctx.intl.locale = "en-Latn"
        ashwini = ctx.intl.entity("nakshatra.ASHWINI").forms
        print("\nnakshatra.ASHWINI")
        for form in ("name", "iast", "phala", "namakarana"):
            if form in ashwini:
                print(f"  {form:<12} {shortened(ashwini[form])}")

        # ── A reading no composer says ───────────────────────────────
        # Half the corpus is glossary rather than narrative: what it means
        # for a graha to be exalted is true of every exalted graha, so no
        # composer says it per chart. It is a record like any other, and any
        # catalogue key you can name you can ask for — which is how a
        # consumer builds a legend beside the plan.
        print("\ndignity.EXALTED")
        exalted = ctx.intl.entity("dignity.EXALTED").forms
        for form in ("name", "phala"):
            if form in exalted:
                print(f"  {form:<12} {shortened(exalted[form])}")


if __name__ == "__main__":
    main()

"""A rule's own reading, in the reader's language — loaded, not embedded.

`interpretation.py` composed a plan and said it in two languages. One thing
it could not say in Nepali was **what a rule's verse states**: that crosses
as the words the rule cites, in the language the rule was written in, and a
Nepali reading shows the seam.

This is how the seam closes. The SDK carries a corpus of readings — one for
each of 649 yogas and doshas, in Sanskrit, Nepali, English and Hindi — and
it is **not compiled into the library**: it is several times the size of
every message pack together, and a consumer computing a Julian day should
not carry every Nepali yoga reading to do it. It is a pack that is loaded
(`docs/03-design/interpretation-records.md`).

What it teaches:

1. **A pack is bytes.** `intl.load_pack` takes them from wherever you got
   them — a file beside your program, a download, an asset in your
   application bundle. This example reads the files `teistro-intl build`
   wrote from the SDK's own source root, as every binding's does.
2. **Loading changes what a composer says**, not how it is called.
   `readings` asks the base locale whether it carries a reading of each
   rule: with the pack, the item is the locale's own reading; without it,
   the verse's cited words. The same code composes both.
3. **A plan item is a sentence.** The record also holds the full passage
   and its named facets; those are read from the entity directly, which is
   one call on an engine you already have.

    cargo run -p teistro-intl -- --root packs/readings build --out target/packs/readings
    PYTHONPATH=. python3 example/readings.py
"""

from __future__ import annotations

import os
from pathlib import Path

from teistro import Altitude, Ephemeris, Latitude, Longitude, Observer, Teistro

# Where the built packs are: `TEISTRO_PACKS`, or the repository's
# `target/packs`.
PACKS = Path(
    os.environ.get("TEISTRO_PACKS")
    or Path(__file__).resolve().parents[3] / "target" / "packs"
)


def packs_of(corpus: str) -> list[Path]:
    """Every pack a corpus built, one a locale, in name order."""
    return sorted((PACKS / corpus).glob("*.tpack"))


def first_sentence(text: str) -> str:
    """Enough of a passage to show it is there, without printing an essay."""
    return text[:96] + "…" if len(text) > 96 else text


def main() -> None:
    teistro = Teistro.open()
    with teistro.context(profile="nepali-default", ephemeris=Ephemeris.BUILTIN) as ctx:
        # ── The pack ─────────────────────────────────────────────────
        # One pack a locale, because a Nepali application wants Nepali and
        # its fallback and not four languages' worth of prose.
        for file in packs_of("readings"):
            data = file.read_bytes()
            # This is the call a consumer makes, whatever the bytes came
            # from.
            loaded = ctx.intl.load_pack(data)
            print(
                f"loaded {loaded.locale:<12} {loaded.entries:>6} readings, "
                f"{len(data):>7} bytes"
            )

        # ── A chart, and the rules it holds ──────────────────────────
        # The computed yogas are the set whose rules the corpus wrote
        # readings for; the nabhasas are the kernel's own and have none,
        # which is the gap `interpret-measured.md` counts.
        chart = ctx.chart.found(
            instant=2447995.4895833335,
            place=Observer(
                latitude_deg=Latitude(27.7172),
                longitude_deg=Longitude(85.324),
                altitude_m=Altitude(1400),
            ),
            utc_offset_seconds=20700,
            rules={"shipped": ["YOGAS", "DOSHAS"]},
            interpret={"readings": True},
        )
        assert chart.plans is not None
        plan = chart.plans["readings"]

        # ── The plan, said twice ─────────────────────────────────────
        print(f"\n{len(plan)} items")
        for locale in ("en-Latn", "ne-Deva-NP"):
            ctx.intl.locale = locale
            print(f"\n{locale}")
            for item in plan:
                said = ctx.intl.render(item["key"], item["params"])
                # A fallback would mean this locale had no reading of its
                # own, which is exactly what an example must not hide.
                fallback = "  (fallback)" if said.is_fallback else ""
                print(f"  {said.text}{fallback}")

        # ── The passage, which the plan does not carry ───────────────
        # A plan item is a sentence. The record holds the essay and its
        # named facets beside it, for a page that wants them.
        says = next((item for item in plan if item["key"] == "sdk.reading.says"), None)
        reading = says["params"].get("reading") if says is not None else None
        if isinstance(reading, dict) and "$entity" in reading:
            key = reading["$entity"]
            forms = ctx.intl.entity(key).forms
            print(f"\n{key}")
            for form in ("prose", "career", "mind", "spirituality"):
                if form in forms:
                    print(f"  {form:<14} {first_sentence(forms[form])}")


if __name__ == "__main__":
    main()

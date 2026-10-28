"""Naming a child (nāmakaraṇa): the syllable a birth pada names a child
by, names checked against it, a Latin spelling read every way it can be,
a family's own book of names searched, and the clauses a name in daily
use is held to, said in English and in Nepali.

`cargo xtask check-python` runs this file, and every binding's `naming`
prints these lines (`03-design/namakarana.md`).

What it is really teaching:

1.  **Facts, not a score.** A check says whether a name begins in the
    birth star's row, in the pada's own cell, or by the patro's
    substitute letter, and what each clause of the gṛhyasūtras makes of
    it. Nothing adds them up: which of them matters is the family's.
2.  **A Latin spelling is read every way it can be.** `CASUAL` answers
    each cell a spelling reaches and says it is ambiguous, rather than
    choosing one.
3.  **The SDK ships no names.** A book is the consumer's, loaded once and
    named by its id; a name the cakra cannot read is listed, not dropped.
4.  **A clause is a key; the locale says it.** `sdk.naming`'s messages
    turn each key into words, so one answer reads in every locale.

It reads no sky, so it names no ephemeris.
"""

from teistro import Birth, Teistro, intl


def yes(flag: bool) -> str:
    """A flag as every binding prints it."""
    return "yes" if flag else "no"


def main() -> None:
    teistro = Teistro.open()
    with teistro.context(locale="en-Latn") as ctx:
        # ── The birth's syllable ───────────────────────────────────────
        # The Moon in Ardra's third pada. Its syllable is ङ, which no name
        # begins with, so the Nepali patro glosses it with ग: a
        # substitute, listed after the cell's own letter.
        birth: Birth = {"nakshatra": "ARDRA", "pada": 3}
        names = ctx.naming.of(birth)
        syllable = names.syllable
        star = syllable.nakshatra.full_key if syllable.nakshatra is not None else "Abhijit"
        print(f"{star} {birth['pada']}: {syllable.devanagari} {syllable.iast}, deity {names.deity.full_key}")
        print(f"initials {', '.join(f'{initial.devanagari} {initial.by}' for initial in syllable.initials)}")

        # ── Names checked against it ───────────────────────────────────
        # Each name is answered on its own: ऐ is no cell of the cakra, and
        # its refusal refuses no other name.
        asked = ["गणेश", "चूडामणि", "ऐश्वर्य"]
        for name, checked in zip(asked, ctx.naming.check(asked, birth)):
            c = checked.check
            if c is None:
                assert checked.refused is not None
                print(f"{name}: refused, {checked.refused.message}")
            else:
                print(
                    f"{name}: same star {yes(c.same_star)}, same pada {yes(c.same_pada)}, "
                    f"by substitute {yes(c.by_substitute)}, {c.clauses.syllables} syllables"
                )

        # ── A Latin spelling, read every way ───────────────────────────
        spellings = ["Krishna", "Shiva"]
        for name, read in zip(spellings, ctx.naming.read(spellings, {"name": {"latin": "CASUAL"}})):
            reading = read.reading
            assert reading is not None
            cells = " | ".join(
                f"{one.spelled} {one.syllable.nakshatra.full_key if one.syllable.nakshatra is not None else 'Abhijit'} "
                f"{one.syllable.quarter}"
                for one in reading.readings
            )
            print(f"{name}: ambiguous {yes(reading.ambiguous)}, {cells}")

        # ── A family's own book ────────────────────────────────────────
        # The SDK ships no names. A book is loaded once into the context
        # and named after by its id; औषधि begins with औ, no cell, and is
        # listed.
        loaded = ctx.naming.load(
            {
                "names": [
                    {"devanagari": "गणेश", "sex": "MALE"},
                    {"devanagari": "गीता", "sex": "FEMALE"},
                    {"devanagari": "गोपाल"},
                    {"devanagari": "औषधि"},
                ]
            }
        )
        refused = ", ".join(f"{one.index} {one.error.field}" for one in loaded.refused)
        print(f"a book of {loaded.names} names, refused: {refused}")
        # A boy's names: the book's boys and the names it gives no sex.
        found = ctx.naming.suggest(loaded.book, birth, {"sex": "MALE"})
        print(f"suggested for a boy: {', '.join(f'{one.reading.spelled} {one.by}' for one in found.names)}")

        # ── Said in each locale ────────────────────────────────────────
        # The answer holds keys; `sdk.naming`'s messages say them.
        ganesh = ctx.naming.check(["गणेश"], birth)[0].check
        assert ganesh is not None
        say = ctx.intl.messages.sdk.naming
        for locale in ("en-Latn", "ne-Deva-NP"):
            ctx.intl.locale = locale
            print(locale)
            print(f"  {say.pada(nakshatra=intl.NakshatraKey.ARDRA, pada=birth['pada'], syllable=syllable.devanagari)}")
            clauses = ganesh.clauses
            print(f"  गणेश: {say.syllables(count=clauses.syllables)}, {say.counted(counted=clauses.counted)}")
            for row in clauses.rows:
                print(
                    f"    {say.clause(clause=row.clause)}: {say.holds(holds=yes(row.holds))} "
                    f"({say.source(source=row.source)})"
                )
            for one in found.names:
                print(f"  {one.reading.spelled}: {say.by(by=one.by)}")


if __name__ == "__main__":
    main()

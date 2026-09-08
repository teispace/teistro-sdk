"""A Bikram Sambat calendar page, and why the conversions are not arithmetic.

The Nepali calendar is not a formula. Its month lengths are decided by
where the Sun stands at the moment a month begins, so they vary year to
year — Baisakh is 30 or 31 or 32 days depending on the year — and the
authoritative table only covers BS 1970 to 2095. Outside that span the
SDK computes the months from the Surya Siddhanta as the text prints it.

Every date the SDK returns therefore says **how it was decided**:
`TABULAR` from the official table, `COMPUTED` from the engine, or
`DIVERGENT` where the two disagree and the table wins. A calendar
application that shows a date without showing that is hiding the one
thing a user might need to know.

This example builds a real calendar page: a whole BS year of month
lengths, then one month laid out as a grid with its Gregorian span.

Run it:

    PYTHONPATH=. python3 example/calendar.py
"""

from __future__ import annotations

from teistro import Calendar, CalendarDate, Context, Teistro, date

#: The days of the week, from the boundary's ISO numbering.
WEEK = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]


def month_page(ctx: Context, year: int, month: int) -> str:
    """One BS month as a calendar grid.

    Every cell is a real date the SDK converted, not a number counted up:
    a month that gains or loses a day at either end is then right by
    construction.
    """
    length = ctx.month_length(Calendar.BIKRAM_SAMBAT, year, month)
    first = date(Calendar.BIKRAM_SAMBAT, year, month, 1)
    # ISO weekday 1..7; a calendar page starts on Monday, so the first of
    # the month sits at column `weekday - 1`.
    lead = ctx.weekday_of(first) - 1

    cells = ["   "] * lead
    cells += [f"{day:3}" for day in range(1, length + 1)]
    rows = [cells[at : at + 7] for at in range(0, len(cells), 7)]

    lines = ["  ".join(f"{name:>3}" for name in WEEK)]
    lines += ["  ".join(f"{cell:>3}" for cell in row) for row in rows]
    return "\n".join(lines)


def described(ctx: Context, day: CalendarDate) -> str:
    """A date with its era and how it was decided."""
    era = "" if day.era is None else f" {day.era.key} {day.era_year}"
    return (
        f"{day.year}-{day.month:02}-{day.day:02}{era}"
        f" [{day.resolution.key}]"
    )


def main() -> None:
    teistro = Teistro.open()
    with teistro.context(profile="nepali-default", locale="ne-Deva-NP") as ctx:
        year = 2082

        # ── A whole year, with its Gregorian spans ────────────────────
        print(f"BS {year}")
        total = 0
        for month in range(1, 13):
            length = ctx.month_length(Calendar.BIKRAM_SAMBAT, year, month)
            total += length
            first = date(Calendar.BIKRAM_SAMBAT, year, month, 1)
            last = date(Calendar.BIKRAM_SAMBAT, year, month, length)
            starts = ctx.convert(first, Calendar.GREGORIAN)
            ends = ctx.convert(last, Calendar.GREGORIAN)
            name = ctx.messages.sdk.calendar.bikram_sambat.month_name(month=month)
            print(
                f"  {month:2}  {name:10} {length:2} days   "
                f"{starts.year}-{starts.month:02}-{starts.day:02}"
                f" to {ends.year}-{ends.month:02}-{ends.day:02}"
            )
        print(f"      {'':10} {total} days in the year")
        # A BS year is 365 or 366 days like any solar year, but its
        # months are not: the shortest here is 29 days and the longest
        # 32, which is why a month length is asked for and never assumed.

        # ── One month as a page ──────────────────────────────────────
        print()
        print(f"Baisakh {year}")
        print(month_page(ctx, year, 1))

        # ── The round trip, and what each date says about itself ──────
        print()
        new_year = date(Calendar.BIKRAM_SAMBAT, year, 1, 1)
        gregorian = ctx.convert(new_year, Calendar.GREGORIAN)
        back = ctx.convert(gregorian, Calendar.BIKRAM_SAMBAT)
        print(f"  BS   {described(ctx, new_year)}")
        print(f"  ->   {described(ctx, gregorian)}")
        print(f"  ->   {described(ctx, back)}")
        print(f"  fixed day {ctx.fixed_of(new_year)}, weekday {ctx.weekday_of(new_year)}")

        # ── Inside the table, and outside it ──────────────────────────
        # A date a caller *states* is always `DEFINED`: it is what was
        # asked for. A date the SDK *returns* says how it was decided, so
        # the resolution to read is the one on the answer.
        print()
        for asked in (2082, 2200, 1960):
            gregorian = ctx.convert(
                date(Calendar.BIKRAM_SAMBAT, asked, 1, 1), Calendar.GREGORIAN
            )
            answer = ctx.convert(gregorian, Calendar.BIKRAM_SAMBAT)
            print(
                f"  BS {asked} began {gregorian.year}-{gregorian.month:02}"
                f"-{gregorian.day:02}, and the answer is [{answer.resolution.key}]"
            )
        print(
            "       the official table runs BS 1970 to 2095; on either side"
            " the SDK's own engine answers, and says so"
        )

        # ── The typed message accessors ───────────────────────────────
        # A date rendered for a reader goes through the locale, not
        # through string concatenation: the key is spelled once, in the
        # generator, and the parameters are typed.
        print()
        print(
            "  rendered  "
            + ctx.messages.sdk.calendar.bikram_sambat.date.long(
                day=1,
                month_name=ctx.messages.sdk.calendar.bikram_sambat.month_name(month=1),
                year=year,
            )
        )


if __name__ == "__main__":
    main()

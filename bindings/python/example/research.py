"""A two-group study, as `03-design/research.md` asks one to be run: the
predicates named, the labels and the test fixed, and the input hash
published before any data are read.

`cargo xtask check-python` runs this file, and every binding's `research`
prints these lines.
"""

from teistro import (
    Altitude,
    Ephemeris,
    Latitude,
    Longitude,
    Observer,
    ResearchBirth,
    ResearchDesign,
    RuleRequest,
    Teistro,
)


def main() -> None:
    teistro = Teistro.open()
    with teistro.context(profile="nepali-default", ephemeris=Ephemeris.BUILTIN) as ctx:
        # Forty-eight births at Kathmandu, two months and an hour apart, and
        # the first sixteen called the cases. The labels mean nothing, so a
        # study that reads them honestly finds nothing: that is what the
        # corrections are for.
        place = Observer(latitude_deg=Latitude(27.7172), longitude_deg=Longitude(85.324), altitude_m=Altitude(1400))
        births = [
            ResearchBirth(instant=2444240.5 + 61.37 * i + (i % 24) / 24, place=place, utc_offset_seconds=20700)
            for i in range(48)
        ]
        rules: RuleRequest = {"shipped": ["YOGAS"]}
        design: ResearchDesign = {"groups": [1 if i < 16 else 0 for i in range(48)]}

        table = ctx.research.counts(births=births, rules=rules, design=design)
        print(f"{len(table.rows)} rules over {len(births)} births")

        tested = ctx.research.compare(
            births=births,
            rules=rules,
            design=design,
            test={"seed": 2026, "permutations": 999, "contrast": {"kind": "CASE_VS_REST", "case": 1}, "alpha": 0.05},
        )
        # The registration: the births, the rules, the labels and the test.
        print(f"registered as {tested.provenance.input_hash}")
        print(f"{tested.permutations} permutations, none can say less than p = {tested.resolution:.4f}")

        def under(method: str) -> int:
            return sum(1 for row in tested.rows if row.under_alpha is not None and getattr(row.under_alpha, method))

        print(f"under 0.05: {under('raw')} raw, {under('max_t')} after max-T, {under('holm')} after Holm")

        # The smallest raw p, the first such row on a tie, and what the
        # family makes of it.
        best = min(tested.rows, key=lambda row: row.p.value)
        rest, cases = best.counts
        print(
            f"{best.predicate}: {cases.present} of {cases.present + cases.absent} cases, "
            f"{rest.present} of {rest.present + rest.absent} others, "
            f"p {best.p.value:.3f}, max-T {best.adjusted.max_t:.3f}"
        )
        if best.effect is not None:
            d = best.effect.risk_difference
            print(f"difference {d.estimate:.3f} ({d.low:.3f} to {d.high:.3f})")


if __name__ == "__main__":
    main()

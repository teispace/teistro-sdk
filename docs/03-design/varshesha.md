# The lord of the year (Varshesha)

Status: `built`, 2026-09-23. Measured against the source's own worked year
in [`muntha-measured.md`](muntha-measured.md) §7 (`check-muntha`), and
built on the office-bearers ([`muntha.md`](muntha.md)) and the
Panchavargiya bala ([`panchavargiya.md`](panchavargiya.md)). Crux C106.

`crates/tajika`'s `varshesha` holds it, and
`sdk.chart().varshesha(&natal, &annual, completed_years, rules)` is the
whole chain in one call.

## Why it is a chain and not a maximum

The obvious reading — "the year lord is the strongest of the five
office-bearers" — is wrong, and the source's own worked year is the case
that proves it:

| claimant | Vishwa bala | portfolios | aspects the lagna |
|---|---|---|---|
| Jupiter | 14:46:00 | 1 | **no** |
| Sun | 14:20:15 | 2 | yes |
| Mars | 14:01:00 | 2 | yes |

Jupiter is the strongest and stands in the **second** from the annual
lagna, which is a neutral house and gives no Tajika aspect at all, so the
source disqualifies it in as many words. The **Sun** holds the year. And
Saturn, stronger than all three at 16:47:45, holds no portfolio and never
enters the reckoning.

Three separate things therefore decide it, and a build that has only the
strength has none of it.

## The chain, as the source gives it

1. An office-bearer under **five units** cannot hold the year; if every
   one of them is under five, the **Muntha's lord** takes it.
2. Only those that **aspect the annual lagna** may hold it. The aspect is
   the Tajika one — friendly at houses 3, 5, 9 and 11, inimical at the
   kendras 1, 4, 7 and 10, and **nothing at all** at 2, 6, 8 and 12 — and
   the source says in the same breath that a friendly aspect counts no
   differently from an inimical one here.
3. Of those, the **strongest**.
4. On a tie of strength, the one holding **most portfolios**.
5. On a tie of that too, the **Muntha's lord**.
6. If **none** aspects the lagna, the Muntha's lord.

And the **Moon is passed over**. "Mild in nature and therefore unable to
govern unless extraordinarily strong", it does not take the year even when
it is the strongest office-bearer aspecting the lagna; the next one down
does. The source states this as the general rule, so it is the default.

## The readings, each a named knob

The source records three places where authorities differ, and none of them
is left as a silence (crux C106):

| reading | default | the other |
|---|---|---|
| nobody aspects the lagna | the Muntha's lord | the annual lagna's lord, which "some authorities" give |
| an outright tie of strength, aspect and portfolios | the Muntha's lord | the Dina-Ratri Pati, which "still others" give |
| the Moon as year lord | passed over | allowed like any other, for the extraordinarily strong Moon the source admits and leaves to the reader |

## What the answer carries

Not a planet. A planet **and why**:

- `chosen` names which step of the chain decided it — `Strongest`,
  `MostPortfolios`, `MunthaLordAllWeak`, `MunthaLordUnaspected`,
  `MunthaLordTied`, `DinaRatriTied`, `AnnualLagnaLordUnaspected` — with
  `on_strength()` for the one distinction a reader usually wants, because
  a year lord chosen on strength and one arrived at by a fallback are
  different statements about the year and the planet alone cannot tell
  them apart;
- `claims`, every office-bearer strongest first with its strength, its
  portfolio count and whether it aspects — so the decision can be read
  rather than trusted, which is how the table above is generated;
- `moon_passed_over`, when the Moon led and stepped aside.

## What is decided and what is not

| | |
|---|---|
| **decided** | the chain and its order; the Tajika aspect as the house relation, the neutral houses giving none; five units as the floor; the Moon passed over by default |
| **a setting** | `VarsheshaRules` — the three readings above |
| **not decided** | the "special circumstances" in which the source says the aspect is not required, which it never names; and whether a Moon is "extraordinarily strong", which it leaves to the reader (crux C106) |
| **built** | `teistro_tajika::varshesha`; `sdk.chart().varshesha`, which composes the office-bearers and the strengths and needs **no ephemeris** |
| **not built** | the crossing, which comes next and carries the year lord rather than the table it is computed from; the sixteen yogas, which need the Tajika aspects with their deeptamsha orbs — a different thing from the aspect this rule uses |

## The aspect here is not the aspect the yogas want

This rule needs only "does it aspect the lagna", which is a relation
between two **signs**. The sixteen Tajika yogas need the aspect between
two **planets** with its orb — each planet's deeptamsha (Sun 15°, Moon
12°, Mars 8°, Mercury 7°, Jupiter 9°, Venus 7°, Saturn 9°), the operative
orb being the *mean* of the two — together with which of them is faster
and whether they are applying or separating. That is the next module, and
nothing here pretends to have it.

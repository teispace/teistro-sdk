# The lord of the year (Varshesha)

Status: `built`, 2026-09-23; the Moon's case and the *Tajika
Nilakanthi*'s readings added 2026-09-24. Measured against the source's own worked year
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
Where there is **no** next one down, the Moon still does not rule: see
*The Moon's successor* below.

## The readings, each a named knob

The source records three places where authorities differ, and none of them
is left as a silence (crux C106):

| reading | default | the other |
|---|---|---|
| nobody aspects the lagna | the Muntha's lord | the annual lagna's lord, which "some authorities" give |
| an outright tie of strength, aspect and portfolios | the Muntha's lord | the Dina-Ratri Pati, which "still others" give |
| the Moon as year lord | passed over | its **Ithasala successor** at once, the *Nilakanthi*'s own view and Charak's "some authorities"; or allowed like any other, for the extraordinarily strong Moon the source admits and leaves to the reader |
| the Moon's successor | any planet in Ithasala with it | only an **office-bearer**, as one of the *Nilakanthi*'s two commentaries reads its verse |
| nobody aspects the lagna | (above) | **the strongest of the five**, the *Nilakanthi*'s v. 11, beside Charak's two |

## The Moon's successor, read from both books

**Built 2026-09-24.** Charak states the Moon's case in two steps:

1. When the Moon leads the claimants that aspect the lagna, take "the
   planet next lower in strength, out of the five office-bearers, which
   also aspects the lagna". His second worked year (Chart VII-1) is this
   case: the Moon leads, Mercury is stronger than Venus but stands in the
   sixth, and Venus holds the year.
2. When the Moon is the only one that aspects, "it still does not become
   the year lord". Instead, (a) the planet with which the Moon forms an
   **Ithasala** takes the year; (b) the strongest of them, if several do;
   (c) if none does, **the lord of the Moon's sign**.

**Before 2026-09-24 the build answered the second case with the Moon
itself**, because the step-down had nobody to step down to and fell
through to the Moon. That contradicts the source. It is now the
successor, and `Chosen` names it: `MoonsIthasala` or `MoonsSignLord`.

The *Tajika Nilakanthi* (Varshatantra v. 12, repeated at Samjñā v. 65)
gives the successor as the author's **own** view. It applies whenever the
chain makes the Moon year lord, with no step down first: "the Moon being
year lord, the planet it forms Ithasala with is the year lord; otherwise
the lord of the Moon's sign". Charak reports the same thing as "some
authorities". It ships as `MoonMayRule::Ithasala`. Its commentary answers
the case the rule seems to leave open: a Moon in its **own** Cancer is its
own sign's lord and **holds the year**, which is why Varshatantra gives
results for a Moon year lord at all. The build follows it:
`MoonsSignLord` may name the Moon.

**Which step is "the Moon becoming year lord"** is read the same way in
both: any step of the chain that would answer with the Moon. A Moon that
holds the Muntha's portfolio and inherits the year through a fallback is
still the Moon, and is still "unable to govern". Under the default the
step down comes first, and the successor only where there is nobody to
step down to.

**Who may succeed it.** Charak's text and the Nilakanthi's verse both say
"the planet". One of the Nilakanthi's two commentaries on that verse
narrows it to the planets "among the five office-bearers", and the other
does not. Any of the seven is the default, and `MoonPartner::OfficeBearer`
is the narrower reading. The Ithasala is the one the yogas read,
`drishti` and its `DrishtiRules`, so `VarsheshaRules::drishti` moves the
same sub-degree band the yogas' does.

**Chart VII-1 is the acceptance test for both readings**, read off the
page rather than transposed. It has Gemini rising at 12°53′, with the Moon
at 18°57′ and Jupiter (retrograde) at 16°35′ in Gemini. Mars and the
Muntha are in Libra, the Sun and Mercury in Scorpio, and Venus and Saturn
in Sagittarius. The chain reproduces the book's **Moon at 12:28:15 and
Mercury at 10:49:15 exactly**. Under the default Venus holds the year, as
printed. Under the Nilakanthi's reading the Moon is past every planet it
aspects: Jupiter, Venus, Saturn and Mars are all Ishrafa, and the Sun and
Mercury are in the sixth. It forms no Ithasala, and **Mercury**, the lord
of Gemini, takes the year, exactly as the book says: "the next
consideration falls on the lord of the Moon sign, which is Mercury".

**One printed number is not reproduced**, and the test records it rather
than fitting it. The book prints Venus at 9:06:30, and the chain gives
7:14:00. The gap is 1:52:30 of Vishwa bala, which is exactly the house
part read as **neutral** instead of an **enemy**: Venus in Sagittarius
with Jupiter, its lord, in the opposite sign, the seventh, which is an
enemy under positional friendship. Natural friendship cannot explain it,
because the Moon's own house part (Gemini, Mercury's sign) reproduces
only positionally. So the book is inconsistent on that one relation, and
the answer does not turn on it: Venus holds the year either way.

**Not built: an Ishrafa between two benefics.** Charak adds that "an
Ishrafa between two natural benefics is, however, not considered bad",
and that Venus as year lord would justify the year's marriage. Read as a
rule, that would let a benefic's Ishrafa with the Moon count. But in
Chart VII-1 it admits **Jupiter** as well as Venus, and the book never
says which it means or how to choose. Recorded, not built.

## The Nilakanthi's chain, beside Charak's

Its Varshatantra vv. 9–11 state the chain itself, and it differs from
Charak's in two places:

- **Nobody aspects the lagna** (v. 11): "the strongest of the five", and
  not the Muntha's lord. It ships as `NoneAspects::Strongest`
  (`Chosen::StrongestUnaspected`).
- **A tie of strength** (v. 10): the one that aspects the lagna "more",
  where Charak counts portfolios. That needs a strength of the Tajika
  aspect, which neither book in reach grades. **Not built**, and recorded
  under crux C106.

## What the answer carries

Not a planet. A planet **and why**:

- `chosen` names which step of the chain decided it — `Strongest`,
  `MostPortfolios`, `MunthaLordAllWeak`, `MunthaLordUnaspected`,
  `MunthaLordTied`, `DinaRatriTied`, `AnnualLagnaLordUnaspected`,
  `StrongestUnaspected`, `MoonsIthasala`, `MoonsSignLord`, all in
  `Chosen::ALL` — with `on_strength()` for the one distinction a reader
  usually wants and `succeeds_the_moon()` for the Moon's case, because
  a year lord chosen on strength and one arrived at by a fallback are
  different statements about the year and the planet alone cannot tell
  them apart;
- `claims`, every office-bearer strongest first with its strength, its
  portfolio count and whether it aspects — so the decision can be read
  rather than trusted, which is how the table above is generated. The
  lord is always one of them **except** where it succeeds the Moon,
  since the Moon's Ithasala may be with a planet that holds no portfolio;
- `moon_passed_over`, when the chain would have given the Moon the year
  and it stepped aside, whether down to the next claimant or to its
  successor.

The pass over the corpus (`muntha-measured.md` §18) counts every step
under both readings. It found **26 of 2 159 years that the build had
given to the Moon**, against the source. The Nilakanthi's reading changes
134 years.

## What is decided and what is not

| | |
|---|---|
| **decided** | the chain and its order; the Tajika aspect as the house relation, the neutral houses giving none; five units as the floor; the Moon passed over by default, and succeeded through its Ithasala wherever there is no one to step down to |
| **a setting** | `VarsheshaRules`: the readings above, and `drishti` for the Ithasala the Moon's successor reads |
| **not decided** | the "special circumstances" in which the source says the aspect is not required, which it never names; whether a Moon is "extraordinarily strong", which it leaves to the reader; the *Nilakanthi*'s tie broken by the greater aspect; and a benefic's Ishrafa with the Moon (crux C106) |
| **across the boundary** | `varsha_json.varshesha` names the three readings; each year's chart answers its lord in the `annual_charts` section — `year_lord`, `year_lord_chosen` (a `TsVarsheshaChosen`), `year_lord_vishwa` and `moon_passed_over` — and its claimants in the **ragged** `year_claims` section, counted by `claim_count`. Node, Dart and Python read it as `pravesha.annual.yearLord`, each binding's test holds the ranking and the lord's membership of its own claims (unless it succeeds the Moon), and all four parity runners print the lord, its step, its strength and every claim: **4 bindings agree on 8 018 values** |
| **built** | `teistro_tajika::varshesha`; `sdk.chart().varshesha`, which composes the office-bearers and the strengths and needs **no ephemeris** |
| **not built** | the sixteen yogas, which need the Tajika aspects with their deeptamsha orbs — a different thing from the aspect this rule uses |

## The strength crosses as an integer

A `Bala` crosses as **sub-sub units**, 3600 to a unit, in an `i32` — not
as a float. Two office-bearers a sub-sub unit apart decide a year between
them, and the source's own worked chart separates its first two claimants
by 25 sub-units. Each binding turns that integer back into
`{units, subUnits, subSub, total}` with a `toString()` that writes it the
way the sources do, `14:20:15`, so a reader compares `units` and a
machine compares `total`.

The seven-planet strength table itself does **not** cross. A consumer at
the boundary wants the year and its reckoning, which the claims carry; the
full five-parts-by-seven-planets table is what that reckoning was computed
from, and `sdk.chart().panchavargiya` gives it to a Rust caller who wants
it. Crossing it would be five more columns per planet per year for a
reader nobody has yet.

## The aspect here is not the aspect the yogas want

This rule needs only "does it aspect the lagna", which is a relation
between two **signs**. The sixteen Tajika yogas need the aspect between
two **planets** with its orb — each planet's deeptamsha (Sun 15°, Moon
12°, Mars 8°, Mercury 7°, Jupiter 9°, Venus 7°, Saturn 9°), the operative
orb being the *mean* of the two — together with which of them is faster
and whether they are applying or separating. That is the next module, and
nothing here pretends to have it.

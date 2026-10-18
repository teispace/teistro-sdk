# The tithi pravesha

Status: `built`, 2026-10-17. `sdk.chart().tithi_praveshas(&birth, rule, through)` answers the
instants; the chart is the caller's to found at each one. A chart
request's `tithiPravesha` record finds them for every chart in a batch
and section 136 carries them across the C boundary ("The record", below).
Cruxes C443 to C447.

## What it is

The tithi pravesha is a year's chart cast for the instant the Moon's
elongation from the Sun comes back to the birth's while the Sun is back
in the sign it held at birth. It is the birthday by the lunar day, where
the Varsha Pravesha ([`annual-chart.md`](annual-chart.md)) is the
birthday by the Sun.

**No classical text has been found that defines it.** The practice's own
writers present it as a modern technique: one popular account credits
Sanjay Rath with bringing it out of a family tradition, and another calls
it "a modern synthesis from traditional principles" and cites no text.
The precedents they point to are not this chart: a yearly observance on
the birth tithi, which they attribute to the dharmashastra without a
passage this page has read, and Tajika's chart for each year, which
the texts this repository has read for it time by the Sun's return alone
([`annual-chart.md`](annual-chart.md)). For this page the Internet
Archive's prints were not searched verse by verse; web searches for a
classical source found none. So every rule below is practice, rank 3,
and each crux says so rather than citing a verse (C443).

## The rule, in the practice's words

The account the rule is taken from says the chart is cast "when the Sun
returns back to the same sign after a year and the exact amount of the
birth tithi has elapsed". Three things are read from that sentence:

- **The exact elongation, not the tithi's span.** "The exact amount of
  the birth tithi has elapsed" is the birth's Moon less Sun to the
  degree, not the start of the 12° tithi the birth fell in. The instant
  is a **phase return** as the hit list already finds it (`PHASE_RETURN`,
  C420): the directed elongation, once a synodic month, searched by
  `Founder::elongation_events`, the crossing search a tithi's ends use.
  Nothing new is solved.
- **The Sun in its natal sign.** The year is chosen by the Sun's sign,
  not by its degree, and the sign is the sidereal one in the chart's own
  zodiac: the sojourn the year's sidereal solar return falls in, read off
  the same search's sign ingresses. That is `TithiRule::NatalSign`, the
  default (C444).
- **One a year.** The year is counted as the Varsha Pravesha counts it:
  `year` 1 is chosen around the first solar return, a year after birth
  (`Pravesha::year`).

## Where the sentence runs out

A sidereal sign holds the Sun 29.4 to 31.5 days and a synodic month runs
29.3 to 29.8, so the sojourn can hold **two** phase returns (a long sign,
Gemini or Cancer) or **none** (a short one, Sagittarius or Capricorn, in
a rare year). The sentence says nothing of either, so the SDK decides
both, names each decision and reports which happened:

| | the reading taken | the rival kept or named |
|---|---|---|
| **two in the sign** | the one nearer the year's solar return (C445) | the first of the two, the "earliest birthday" reading; not built, a later rule beside the two |
| **none in the sign** | fall back to the phase return nearest the solar return, reported as `picked: NEAREST_RETURN` (C446) | the instant the Sun enters the sign if the birth's tithi is then running, which reads "tithi" as the span; not built |
| **the anchor itself** | `TithiRule::NearestReturn` asks for the phase return nearest the solar return whatever the sign; always one, within about fifteen days | the lunar month of the birth (the janma tithi in the janma masa) — the calendar birthday, which can stand a month from this; not built |

Every answer carries `in_sign`, the count of phase returns inside the
sojourn (0, 1 or 2) under either rule, and `picked`, the rule that chose
the instant. `fell_back()` is `rule != picked`. A consumer who reads the
sentence another way has the count and the solar return in hand and the
crossings one call away (`sdk.chart().hits` with `PHASE_RETURN`); nothing
is a dead end.

## Seen from where, in which zodiac

The phase return is read where the hit list reads it (C447): in the
chart's frame, which for an elongation is any zodiac, the ayanamsha
cancelling; topocentric under a topocentric profile, seen from the birth
place, as the birth's own elongation was. The Sun's sign is read in the
chart's own zodiac on the settings' ayanamsha basis, as the solar return
is (`annual-chart.md`, "Read as the chart reads it"). Under a profile
that is geocentric the place does not move the instant; under a
topocentric one the Moon's parallax moves it by up to about two hours,
and the birth place is used, as `praveshas` uses it. Where the year's
chart is **cast** is the caller's: the birthplace or the residence is the
same open question as the Varsha Pravesha's.

## What the search costs

Each year is two scans over the 66 days around its solar return, 33
either side (`TITHI_WINDOW_DAYS`): the
elongation at its default step (a day; the elongation never turns) and
the Sun's sign ingresses at a day. Forty years are some five thousand
samples, against the fifteen thousand one scan of forty years of
elongation would take. A year whose window passes the provider's
coverage is left out, as a solar return past it is: fewer instants, not
the provider's `OutOfRange` on a day the caller never named.

## The record

A chart request's `tithiPravesha` record (`TithiPraveshaRequest::from_json`:
`through`, one to two hundred, and optionally `rule`, `NATAL_SIGN` by
default) is checked before anything is founded, a refusal named from the
record (`tithiPravesha.through`), and answers each chart in the batch what
`tithi_praveshas` answers it alone (`tests/tithi_pravesha.rs`). Section
136, `tithi_pravesha`, carries every chart's years across the C boundary
as canonical JSON, a list of years a chart (`tests/abi.rs` holds it to the
façade's own), and the agent server reads the record as it reads the
others. A chart whose years run past the ephemeris holds fewer rows, as
the call answers fewer.

## What is decided and what is not

| | |
|---|---|
| **decided** | the exact natal elongation, directed; the Sun's natal sidereal sign around each solar return; the nearer of two; the fall-back with its report; the year counted in returns |
| **a setting** | `TithiRule`: `NATAL_SIGN` (default) or `NEAREST_RETURN`; how many years, one to two hundred (`through`) |
| **not decided** | where the year's chart is cast, the caller's, as for the Varsha Pravesha |
| **built** | `TithiRule`, `TithiPravesha`, `TithiPraveshaRequest`, `TITHI_WINDOW_DAYS`, `ChartArea::tithi_praveshas` (`crates/sdk/src/tithi_pravesha.rs`); `crates/sdk/tests/tithi_pravesha.rs` reads every instant back through charts founded a second either side of it; the `tithiPravesha` record and section 136 |
| **not built** | the lunar-month rival; the yoga and nakshatra praveshas the research row names beside it (the Moon plus the Sun, the Moon alone: the same choice over another quantity); monthly tithi praveshas; the bindings' typed readers |

## What the corpus cannot settle

The conformance repository records no tithi pravesha, and no text read
gives a worked one. So the read-back is the only check: at each instant
the elongation of a chart founded a second before is just short of the
birth's and a second after just past it, the Sun of a chart founded at
it stands in the natal sign under the natal-sign rule, and the years fall
twelve or thirteen synodic months apart. A recorded tithi pravesha from
the practice would settle C445 and C446, and is named here so a later
export knows what to ask for.

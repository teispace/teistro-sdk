# Sade Sati: Saturn's spells from the natal Moon

Status: `draft`, 2026-09-29; §7 steps 1 to 4 **built** the same day.
Written before any code; the building corrected it (§7).

Derives from `transit-hit-list.md` §6 step 5 and the research page's P0 row
"Sade Sati and Dhaiyya (Saturn from the Moon) with phases and exact dates,
plus Kantaka Shani, Ashtama Shani", whose closing checklist asks for the
phase boundaries "by sign entry (default) versus by degree from the Moon
(45° windows); offer both" (`01-research/feature-universe/11-transits-gochar.md`).

## 1. What the sources say, and do not

**No classical text read for this project names it.** BPHS, Brihat Jataka,
Saravali, Phaladeepika, Jataka Parijata, Kalaprakasika and Uttara Kalamrita
were searched for the doshas under C89, and Sade Sati is in none of them
(`04-yogas-doshas.md`). The nearest verses are Phaladeepika ch. 26's
gochar (`gochar.md`): v. 2 makes Saturn good only in the 3rd, 6th and
11th from the natal Moon's sign, so the 12th, the 1st and the 2nd are bad
**one sign at a time**, and v. 22 reads Saturn over the Janma rashi. No
verse joins the three into one seven-and-a-half-year period. The joining
is practice.

What the practice agrees on, read in the baseline engine, the Wikipedia
article (whose one citation is Defouw and Svoboda, *Light on Life*, 1996)
and the calculators surveyed (Drik Panchang, Astro-Seek, Astrograha and
others):

- **The span.** Saturn in the 12th, the 1st and the 2nd sign from the
  natal Moon's sign, about seven and a half years.
- **The phases.** One a sign, named **rising** (the 12th), **peak** (the
  1st) and **setting** (the 2nd).
- **A degree reading beside it.** Saturn from 45° before the natal Moon to
  45° past it, the "alternate calculation" of the article. Its example
  puts the start at the degree (18° Pisces for a Moon at 3° Taurus) and
  names the nakshatra that degree falls in; this page reads the degree.
- **The smaller spells.** Saturn in the **4th** from the Moon (Kantaka
  Shani, the small Panoti, Dhaiya) and the **8th** (Ashtama Shani), about
  two and a half years each. The article adds the 7th; the baseline engine
  counts the 4th and 8th.

Per C89 such a feature ships **at rank 4, with a note that no verse was
found**, and every choice below is a knob, not a verdict.

## 2. The forks (cruxes)

| crux | question | readings | default | why |
|---|---|---|---|---|
| C147 | what Sade Sati and its phases are reckoned in | whole signs from the natal Moon's sign; 30° houses centred on the natal Moon's degree (the 45° reading) | **whole signs**, `Reckoning::Degree` for the other | the sign reading is what every calculator surveyed prints and what the baseline engine computes; the degree reading is the same three houses with the Moon at the middle of the first, so it is one knob, not a second algorithm |
| C148 | how a retrograde re-entry counts | each visit a period of its own; visits to one sign merged when the gap is under a fixed number of days (the baseline engine's 270); visits grouped by **which circuit of the zodiac** they belong to | **grouped by circuit, every visit kept** | a gap threshold is a constant no source gives and could, in principle, split or join wrongly; the circuit is exact (§3), and keeping every visit lets a caller who wants the first entry or the net stay read either |
| C149 | which smaller spells are counted | the 4th and 8th; the 4th, 7th and 8th; the kendras (1, 4, 7, 10) as Kantaka | **the 4th and the 8th**, the request naming any house outside the Sade Sati's three | the baseline engine and the calculators surveyed; the 7th is the article's alone |

The reference point is gochar's own knob (C139): the natal **Moon** by
default, the lagna on request (`GocharFrom`), so the "Sade Sati from the
lagna" some software prints costs nothing more.

## 3. The model

**One lattice.** Both reckonings are Saturn's longitude crossing a
lattice of 30° lines: at the reference sign's start (every sign's line)
under `Sign`, and at the reference degree ± 15° under `Degree`, which puts
the reference in the middle of the first house and the Sade Sati from 45°
before it to 45° past it. A crossing names the house entered as a sign
ingress names the sign (`hits::entered`): the one past the line moving
forward, the one before it moving back. The search is the hit list's
(`Founder::transit_events`), in the chart's own zodiac, so a boundary is
the instant a chart founded there would change its answer.

**The circuit, not a gap.** Walking the crossings in order, count a turn
each time Saturn passes from the 12th house into the 1st forward, and
take one back when it passes the other way. House and turn together are
an **unwound** house, `12 × turn + house`, which only ever changes by one
at a crossing. A Sade Sati is every visit whose unwound house is one of
`12k + 12`, `12k + 13`, `12k + 14` (the 12th, 1st and 2nd of the same
circuit), and a smaller spell every visit of `12k + h`. A retrograde loop
out of a house and back into it keeps its circuit, so it stays in its
period; two Sade Satis are a circuit — about 29.5 years — apart, so they
never join. No constant decides it.

**True bounds, outside the window.** A caller asks about a window (or an
instant) and gets every period overlapping it, **whole**: its first entry
and its last exit, however far outside the window they fall. The search
starts at the window, widens by a step, and widens again until the
answer cannot change:

- Before the earliest period that overlaps the window, Saturn must stand
  at least **two** houses before the period's first house when the search
  starts. A retrograde arc is about 7°, far less than a house, so from
  two houses back Saturn cannot have visited the period yet.
- After the latest one, likewise at least two houses past its last house
  when the search ends.

The step decides only the cost, not the answer, and at least one crossing
is always in hand (Saturn stays in no house for six years), so the house
Saturn stood in when the search began is read from the first crossing and
needs no position of its own.

**Where the ephemeris ends.** The search never asks past the provider's
coverage. A bound it would need from beyond it is **absent** rather than
guessed: a visit's start or end is `None`, and a caller reads that as
"before (or after) what this ephemeris covers".

## 4. The answer

```text
Report
  reference: Reference            // the natal point and its sign (C139)
  reckoning: Reckoning            // Sign or Degree (C147)
  sade_sati: [SadeSati]           // every one overlapping the window, whole
  spells:    [Spell]              // the smaller spells asked for (C149), whole
  phase_at(t), spell_at(t)

SadeSati
  phases: [Spell]                 // rising (12th), peak (1st), setting (2nd), in order
  begins(), ends(), phase_at(t)

Spell
  house: u8                       // 1..=12 from the reference
  visits: [Visit]                 // every stay, retrograde re-entries included
  begins(), ends(), contains(t)

Visit { from: Option<instant>, to: Option<instant> }   // half-open
```

`Phase` names the Sade Sati's three houses (`Rising`, `Peak`, `Setting`)
for a caller who reads by name; `Spell::phase()` gives it.

A period reaches into a window across its **whole span**, from its first
entry to its last exit, the days Saturn stepped back out between two
visits included: asked on such a day, the Sade Sati is reported (it has
begun and not ended) and `phase_at` answers none.

## 5. The batch

`sade_sati_many` over many charts shares the sky as `hits_many` does.
Under `Sign` the lattice is every chart's, so Saturn is scanned **once**
for the whole batch; under `Degree` each chart has its own lattice and
`Search::each` tests all of them in one scan. A topocentric frame groups
the charts by place.

## 6. What is measured

`sade-sati-measured.md`, generated by `cargo xtask sade-sati` and gated by
`check-sade-sati`, over the corpus's births and a century of Saturn:

- **Read back.** Each visit's bounds are read back through a chart founded
  a second either side, at the birth's own place: Saturn stands in the
  house the visit says, and outside it on the other side.
- **Circuits.** Every Sade Sati's phases come in order, and every retrograde
  re-entry the century holds is inside its own period.
- **The two reckonings.** How far the degree reading moves each bound from
  the sign reading, which is a property of the Moon's place in its sign:
  it begins earlier exactly when the Moon stands in the first half of its
  sign.
- **The baseline's rule.** Whether joining visits under 270 days apart
  groups them as the circuit does, from the longest pause inside a period
  and the nearest two periods of one house.
- **Bits.** A period asked about at an instant inside it is the one the
  century's search finds, and a batch answers as each chart alone.

The read-back reads the gochar, a second consumer of the same
longitudes, and reckons the houses itself rather than through the SDK's
`origin_deg`.

## 7. Order of work

1. This page, the cruxes and the pure model in `teistro-gochar::sade_sati`,
   tested on crossings built by hand (the circuit, re-entries, open ends):
   **done**.
2. The façade (`sdk.chart().sade_sati(&natal, &SadeSatiRequest)` and
   `_many`) over `transit_events`, with the widening search, and the
   read-back tests: **done**. The search reaches seven and a half years
   either side of the window at first and widens by as much again, at
   most eight times, on whole Julian days.

   **Found building it:** asked at one instant inside a phase, a bound
   came back 40 µs from the one a lifetime's search found. The scan's
   samples were already an anchored grid, but it unwrapped the curve by
   a sum carried from the window's first sample, which rounds differently
   for every start; a bracket is now unwrapped from its own sample, and a
   crossing is the same bits in every window that holds it
   (`astro-events-and-crossings.md` §4, proved red at 37.5 days).
3. The measured pass: **done** (`sade-sati-measured.md`,
   `check-sade-sati`). Every claim holds over 55 births and a century;
   the baseline engine's 270-day rule holds too, with 33 days to spare.

   **Found building it:** the first read-back reckoned the houses through
   the SDK's `Reckoning::origin_deg`, the function under test, so a
   degree origin broken by a degree read back perfectly and only the
   page's numbers moved. It reckons them itself now, and the same break
   turns two claims red (4 096 of 16 372 bounds, 8 of 203 starts). The
   pass costs about 40 s, half of it sampling Saturn at the scan's daily
   cap; plan A1g proposes sizing a slow body's step by its stations.
4. The boundary and the bindings: **done**. `sade_sati_json` on the chart
   request, read by `SadeSatiRequest::from_json` and answered by one
   `sade_sati_many` over the request's charts. The report is nested (a
   Sade Sati holds phases, a phase holds visits), so it crosses as two
   sections, the hit list's pattern: `sade_sati`, a row a chart with the
   reference and reckoning, and `sade_sati_visits`, a row a visit, ragged
   by `cast.sade_sati_visit_count`. A visit's `period` numbers the chart's
   periods, its Sade Satis first and then its smaller spells, which is
   all a binding needs to rebuild the Rust `Report`, since a Sade Sati's
   houses (12, 1, 2) and a smaller spell's (3 to 11) never overlap. An
   absent bound is NaN, as the balance's span is. Node, Python and Dart
   rebuild `{ reference, reckoning, sadeSati: [{ phases }], spells }`,
   Dart with value equality so a period found twice compares equal;
   parity agrees on every visit of ten years in five runners, and the ABI
   test holds each cell to the façade's own report, to the bit.

   **Found building it:** the boundary answers a batch of no charts with
   an empty blob, but it handed the hit list's `hits_many` an empty list,
   which refuses one by `natals`, a field no caller of the boundary wrote.
   Both searches now ask the façade nothing for an empty batch; a test
   asks for both over no charts and was red first.
5. The readings: **done**. The baseline corpus's five `sade-sati-phala`
   records are Saturn's house from the Moon, so they land on the open
   kind `gochar_bhava` (`SATURN_IN_12`, `SATURN_IN_1`, `SATURN_IN_2`,
   `SATURN_IN_4`, `SATURN_IN_8`) under the form `sadeSati`, the shape
   `graha_bhava` has for the natal chart. `sdk.interpret().sade_sati`
   says each house a report holds once, in the order Saturn first
   reaches it, and says nothing of a report counted from the lagna. A
   test holds the migration's names to `Phase::house` and
   `DEFAULT_SPELLS`, so a phase that moved house on one side fails.

   **Found building it:** the first rendering left out the frame's
   house, and every one of the ten fell back with a warning; the
   state-readings pass wrote *falsified* onto its page and passed. It
   refuses a claim that does not hold now, and was red first.

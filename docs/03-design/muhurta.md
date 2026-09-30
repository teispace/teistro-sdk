# Muhurta: electing a time

Status: `draft`, 2026-09-30; §6 steps 1 to 5 **built** 2026-09-30. Written from
the sources before any code; the building is expected to correct it.

Derives from `01-research/feature-universe/08-panchanga-calendar-muhurta.md`
(P0: the activity catalogue, the blackouts, panchanga shuddhi, Tara and
Chandra bala against a native, the lagna at the elected instant, the
event karaka, the Mahadosha veto, intra-day windows and a ranked search
over a range) and the roadmap's Phase 7 `muhurta`, "search with
blackouts and event rules", whose exit criterion is that "the muhurta
regression ranking matches".

## 1. What the sources say

B. V. Raman's *Muhurtha or Electional Astrology* (rank 2; the Internet
Archive's `in.ernet.dli.2015.128092`, the 1948 edition). The book is
under copyright: this page states its facts and quotes a few words,
never a table or a passage. Muhurta Chintamani, the text Raman and the
baseline engine both lean on, is on the Archive only in Sanskrit and
Hindi; the 1979 Motilal Banarsidass edition (with the *Piyushadhara*
and Kedardatta Joshi's Hindi) has an OCR text that can be searched but
renders few tables legibly. Where it settles a fork it is cited at rank
1; where Raman is the only source, the fork says so.

**What an election is.** Raman is explicit that no moment is free of
every defect: "an absolutely good muhurtha is inconceivable", and the
art is *gunabahulya* and *dosha swalpa*, an excess of good and a
deficiency of evil (ch. V). A defect can be **neutralised**, and a
neutraliser is itself a named combination. So an election is not a
score the texts define; it is a set of clauses, some holding, some
cancelled, and a judgement over them.

**Three things common to almost every election** (ch. III):

- **Tarabala.** Count from the native's birth star to the day's star,
  reduce by nine: 1 Janma, 2 Sampat, 3 Vipat, 4 Kshema, 5 Pratyak,
  6 Sadhana, 7 Naidhana, 8 Mitra, 9 Parama Mitra. The 1st, 3rd, 5th and
  7th are unfavourable, but only their opening ghatis (1, 7, 3 and 8
  respectively) need be avoided when the day is otherwise good. Janma
  is favourable for some rites and not others.
- **Chandrabala.** The day's Moon should not stand 6th, 8th or 12th
  from the native's birth sign.
- **Panchaka.** The number of the tithi, the vara (Sunday 1), the
  nakshatra (from Ashwini) and the **lagna** (from Aries), summed and
  reduced by nine: 1 mrityu, 2 agni, 4 raja, 6 chora, 8 roga; 3, 5, 7
  and 0 are good. Which of the five an activity must avoid depends on
  the activity (occupation raja; house-building raja and agni; travel
  chora; marriage and upanayana roga and mrityu).

**The negative periods** (ch. II): each lagna has a *tyajya* part, the
first, middle or last half ghati by sign; each nakshatra has a
*tyajya kala* of four ghatis starting at a ghati the text tabulates per
star (the almanacs' *varjyam*). Tuesday and Saturday are avoided; the
4th, 8th, 12th and 14th tithis in either paksha are unsuitable.

**The twenty-one Mahadoshas** (ch. V), with their neutralisations.
They divide by what they read:

| reads | Mahadoshas |
|---|---|
| the day's panchanga | panchanga shuddhi (bad tithis, Bharani and Krittika, the ends of Ashlesha, Jyeshtha and Revati, Atiganda, Shula, Ganda, Vyatipata and Vaidhriti, Vishti); vara dosha; Vyatipata and Vaidhriti as Mahapata |
| the sky over days | Surya sankramana (sixteen ghatis either side of a sankranti); grahanotpatha (the eclipse's star, six months for a marriage); krurasamyuta (the Sun's star and its neighbours) |
| the elected instant | kartari (malefics either side of the lagna); shashtashta vyaya Chandra (the Moon 6th, 8th or 12th from the lagna); sagraha Chandra (the Moon with any graha); udayasta shuddhi; durmuhurta (named muhurtas of the day and night, and two per weekday); gandanta (tithi, sign and nakshatra junctions); papa shadvarga; Bhrigu shatka (Venus 6th); Kujashtama (Mars 8th); kunavamsa (the lagna in a malefic's navamsa); rasi visha ghatika (the lagna tyajya) |
| the native | ashtama lagna (the lagna 8th from a party's birth lagna) |
| the weather | akala garjita vrishti (unseasonal thunder and rain) |

and eleven neutralisations, among them: the lagna tyajya holds only on
named weekdays by navamsa; Tuesday is not evil after midday; Vyatipata
and Vaidhriti lapse after midday; a vara is unblemished when its lord is
strong; Venus, Mercury or Jupiter in the lagna, or an exalted graha
there, removes the rest; the Moon or the Sun in the 11th; Jupiter or
Venus in a kendra with malefics in 3, 6 or 11.

**Special yogas** (ch. VI): Siddha yoga by vara, tithi and nakshatra,
and Amrita Siddhi by vara and nakshatra — which `panchanga::omen`
already computes from its own tables.

**Per activity** (chs. VII to XVII): each rite names its months, tithis,
varas, stars, lagnas and the placements to seek or avoid. Marriage
(ch. IX) is the fullest: the **lunar** months Magha, Phalguna, Vaisakha
and Jyeshtha good, Kartika and Margashira ordinary; the 2nd, 3rd, 5th,
7th, 10th, 11th and 13th tithis best; eleven stars (Rohini, Mrigashira,
Magha, Uttara Phalguni, Hasta, Swati, Anuradha, Mula, Uttara Ashadha,
Uttara Bhadrapada, Revati) with the first quarter of Magha and Mula and
the last of Revati rejected; ten yogas and Vishti rejected; Gemini,
Virgo and Libra the best lagnas; and six placements that matter most —
the 7th empty, no Mars in the 8th, no Venus in the 6th, no kartari, no
malefic in the lagna, the Moon alone.

## 2. What the baseline engine does

The baseline engine's search (rank 3) is a scorer in three passes:

1. **The season, as vetoes.** Chaturmas, adhika masa, kharmas, pitru
   paksha, Guru asta and Shukra asta, computed once for the range as
   whole-day windows and applied per activity (travel and medicine gate
   on none). Asta is the heliacal disappearance, pinned to Kathmandu,
   with the combustion orb as a fallback.
2. **The day, as a score.** Tithi, nakshatra, weekday and Rahu kaal at
   sunrise with fixed weights (Nanda +20, Rikta −10, a matching star
   +25, a harsh one −15, a gentle day +15, Saturday −10, noon in Rahu
   kaal −30), plus panchanga shuddhi, event rules, Tara and Chandra bala
   against a native; the activity's star set and solar month as gates.
3. **The window, as a score.** The good choghadiyas and Abhijit, less the
   kaalas the caller avoids, each scored by the day plus a choghadiya
   bonus, then the lagna (lord, placements, 8th, 7th for marriage,
   kartari, a benefic in the lagna) and the event karaka (dignity,
   combustion, retrogression) at the window's start, and a Mahadosha
   cap (55, or 30 for two) unless a benefic in the lagna cancels one.

The **structure** is right and this design keeps it: the season is a
property of the range and not of the request, a veto is not a weight
(the regression it was rebuilt for had 24 of 25 marriage dates inside
Chaturmas, the top three scoring 100), and the expensive instant-level
work is done for the days that survive. The **weights** are the engine's
own; no text gives them, so they ship as one named ranking and not as
the answer (C162).

## 3. Forks

| # | fork | default | the other reading |
|---|---|---|---|
| C158 | where panchaka begins | **Dhanishtha's third quarter**, the Moon in Aquarius: Muhurta Chintamani, *Nakshatra Prakarana* v. 48 ("झषकुम्भगे विधौ"), with its commentary and four authorities it quotes, and Raman ch. IV, p. 26 (**closed at rank 1**) | `panchanga.panchaka_start = NAKSHATRA`, the whole of Dhanishtha, what the SDK ships today |
| C159 | the five panchaka kinds | **the remainder** of tithi, vara, nakshatra and lagna by nine (Raman ch. III), a clause of an instant | the recording engine's one kind per nakshatra, kept on the almanac as it is and named as that engine's |
| C160 | Chandrabala's houses | **avoid 6, 8, 12** from the birth sign (Raman ch. III) | the baseline's good 1, 3, 6, 7, 10, 11 and draining 2, 4, 5, 9, 12, a table the caller names |
| C161 | the month a rite is permitted in | **the lunar month** where the source names one (Raman) | the Sun's sign (the baseline, after the published almanacs), named per activity |
| C162 | how candidates are ordered | **fewest uncancelled doshas, then most favourable clauses**, the sources' own *dosha swalpa, gunabahulya*, no number invented | `BASELINE`, the engine's weights, which the roadmap's regression is stated in |
| C163 | the marriage yogas to reject | Raman's list as printed (p. 102, read off the page image): Vyatipata, Dhruva, Ganda, Vajra, Shula, Vishkambha, Atiganda, Vyaghata, Parigha | — ; the printed list also names **Mrityu**, which is not one of the 27 yogas but one of the 28 Anandadi yogas the SDK does not compute, so it is reported as unjudged rather than dropped; *Dhruva* is printed, though other lists count it auspicious |
| C165 | where each nakshatra's tyajya kala (varjyam) begins | a table the caller names, as `panchanga.muhurta_tables` is; none shipped as the default until a rank-1 table is read | Raman p. 15 as printed differs from the commonly published table in five stars (Bharani 4 against 24, Uttara Phalguni 1 against 18, Mula 20 against 56, Purva Ashadha 20 against 24, Uttara Bhadrapada 30 against 24); some look like dropped digits, which is a guess and not a reading |
| C164 | where a graha's asta is seen | **the place of the search**, by the caller's visibility criterion | a fixed observer the caller names (the baseline's Kathmandu: one window for a country, as its almanac prints) |

C158 corrects `panchanga-day-conventions.md` §7, which said the SDK ships
"the nakshatra rule, which is the one the texts state": both texts in
hand state the other, and the corpus cannot tell them apart (it has no
day with the Moon in Dhanishtha's first half). C159 explains why the
same page found that "no numbering of the three reproduces" the
recording engine's kinds: the classical remainder has **four** terms,
and the fourth is the lagna, which a day does not have. The two are
different things with one name, and the almanac's kind is the engine's
table.

## 4. The design

A new crate, `teistro-muhurta`, over `panchanga`, `astro`'s visibility
and the chart founder. Three layers, each usable alone.

### 4.1 The season

`Season::over(range, place, &SeasonRules) -> Vec<Blackout>`: each
blackout a kind and a window, computed once. The kinds are a catalogue
kind, `blackout`, because a consumer names them in a gate: `CHATURMAS`,
`ADHIKA_MASA`, `KHARMAS`, `PITRU_PAKSHA`, `GURU_ASTA`, `SHUKRA_ASTA`,
`SANKRANTI` (Surya sankramana's sixteen ghatis) and `ECLIPSE_STAR`
(grahanotpatha). Asta is the heliacal event pair from
`visibility::Heliacal` under the caller's `Criterion`, which already
offers the Surya Siddhanta's degrees of time, the tradition's
longitudes and the astronomers' arcus visionis; Venus has two episodes a
synodic cycle and both are windows. The windows are **instants**, not
whole days: a day-level gate reads "any blackout overlaps the day", and
a caller who wants the almanac's whole days asks for them.

Built (`season.rs`) as two calls rather than one type: `blackouts` over
a longitude source for everything the Sun and the Moon decide, and
`asta_over` over a visibility reckoner for Guru and Shukra asta, because
the second needs a place and a criterion and the first needs neither.
The months are found from the new moons; each is named by the Sun's sign
at its opening and marked by the sankrantis inside it, and Chaturmas and
Pitru paksha are read off the **nija** Ashadha, Kartika and Bhadrapada.
Four findings:

- **The baseline's Chaturmas is a month late in 2025** (crux C166). It
  anchors each end on the Sun's sign at the bright eleventh, which is
  the named month's eleventh only when that falls after the sankranti.
  The month-name reading matches the published 2025 dates and the
  baseline's agrees with it in 2026, the regression's year; a test pins
  both.
- **The visibility reckoner read Jupiter unseen about its opposition.**
  The body's own rising was searched half a day either side of the
  sunrise, which near opposition holds two risings about twelve hours
  off, and the one after the sunrise was taken; two criteria of three
  then read a body up all night as hidden for about four weeks. The
  search now leans the way the side does (`astro::visibility`), held by
  a test over the built-in ephemeris proved red on the old window. The
  first asta test here is what found it: a Guru asta in January 2026.
- **An asta already running when the range opens starts at the range**,
  not at the first dawn it is read at: unseen on the first day, it was
  unseen before.
- **`ECLIPSE_STAR` is not built**: the grahana nakshatra needs the
  eclipses, which the SDK does not yet find. It is the one kind this
  section named that the season lacks.

### 4.2 The clauses

A **clause** is one named condition from a source: `Clause { kind,
holds, cancelled_by: Vec<Neutraliser>, source }`. The kinds are a
catalogue kind, `muhurta_clause`, one per Mahadosha, shuddhi and named
yoga, because the interpretation corpus's `muhurta-factor` category
keys by them and a reader must name them. Each clause is a pure function
of what it reads, and says which layer it belongs to:

- **of a day**: the limbs at sunrise (and their spans across the day),
  Tarabala and Chandrabala against a `Native { star, moon_sign,
  lagna }`, the special yogas from `panchanga::omen`.
- **of an instant**: everything that reads the lagna or a placement.

### 4.3 The window

A **window** is a maximal interval over which no clause changes. The
day is cut at every boundary any instant-level clause can move at — the
lagna's sign and navamsa, its tyajya part, the kaalas, the durmuhurtas,
the choghadiya, the horas, the varjyam, the gandantas, a limb's end,
midday for the lapsing doshas — and each piece is judged once. That
makes the answer **exact rather than sampled**: the baseline judges a
window by its start, so a choghadiya that crosses a lagna boundary
reports the first lagna for all of it. A graha that changes sign within
the day adds its crossing as one more cut.

Built (`window.rs`), three findings from building it:

- **The lagna is the one cut found by search.** It is sampled once a
  minute and each change of navamsa bisected to about a millisecond; a
  change of sign is a change of navamsa, so one search gives both. A
  navamsa rises in no less than about four minutes outside the polar
  circles, so a minute's step misses none. Beyond them the lagna jumps
  the arc that never rises, and the bisection closes on the jump as one
  cut that skips several navamsas, which is what the sky does. The
  integration test reads the lagna back a tenth of a second inside each
  window's two ends, through the same `Founder::ascendant_at` a chart's
  own derived points use, and was proved red by loosening the
  tolerance to 86 s.
- **A tyajya needs its sign whole.** Raman's rule places the half ghati
  at the first, middle or last of the sign's rising, so a sign already
  rising when the search began, or still rising when it ends, cannot be
  judged. `window::tyajya` judges only the signs bounded by two cuts, and
  a search asks the lagna over a margin either side of the day.
- **Every tithi end is a karana end**, so dropping the tithi's cuts
  alone left the whole-window test green; dropping both turned it red. The
  tithi's are kept, since a karana convention that is not a half tithi
  would otherwise lose them silently. The varjyam and the gandantas are
  not in an almanac day yet; when they are, they become cuts the same
  way. The other grahas' ingresses need the provider and are added by
  the search.

### 4.4 The search

`sdk.muhurta().search(&Request)`: an activity's `ActivityRules`, a
range, a place, an optional native, the kaalas to avoid and a
`Ranking`. The passes are the baseline's, with its cost property: the
season for the range, then the days its gates leave, then windows for
the best `days_with_windows` of them, each cut and judged. The answer is
every window with its clauses, ranked, and **what was vetoed and why**
— a day refused by a blackout names the blackout, so "why is November
empty" has an answer.

`ActivityRules` is data (serde, JSON Schema): the gates (blackouts,
months with the kind of month, stars, tithis, varas, lagnas, panchaka
remainders, placements) and the preferences, each citing its source.
The SDK ships Raman's per activity and the baseline's as a second set;
a consumer passes their own, so an activity no one ships is one JSON
document and not a dead end.

Built first (`activity.rs`, `grade.rs`): the rules without the search.
The texts **grade** rather than only reject, so a rule is three lists
and what an unlisted member is (`Graded<T>`), and a clause is reported
for the best and the rejected; a middling member reports nothing, which
keeps a clause's verdict two-valued. A rite's chapter is laid over the
general shuddhi member by member (`Graded::over`): where the chapter
speaks it wins — Raman's marriage calls Saturday middling though his
shuddhi rejects it — and where it is silent the shuddhi holds, so
Vaidhriti stays rejected. `ActivityRules::raman_marriage` carries the
lunar months (C161), the lagnas, the padas, the blackouts Muhurta
Chintamani forbids a marriage in (ch. I, vv. 46–47: Jupiter or Venus
set, an adhika month), the six considerations as **bars** named by
`ClauseKey`, and what the texts ask that is not judged yet as a list
(`unjudged`: the Mrityu yoga, C163; the bala and vriddha days either
side of an asta; the kshaya month) rather than a silence. The
baseline engine's marriage rules come with the regression (step 6),
where they are measured.

`Ranking` is an enum rather than a trait, so a request and its answer
stay data (C162). A judgement is never collapsed into a number the SDK
invents; the ranking a request used is reported with the answer.
`Ranking::Texts` is Raman's excess of good and deficiency of evil: open
windows first, then the fewest clauses against, the most for, the
earliest. `BASELINE` joins it with the regression (step 6).

Built (`search.rs`, `judge.rs`, `sources.rs`): the three passes, over a
`Sources` trait so the orchestration is the same over any sky, and
`ProviderSources` answering it over a provider. The kaalas are not a
request field: a day reports them as clauses, and a tradition that
refuses Rahu kaala lists `KAALA` among its bars. A window is cut, beyond
the day's own spans and the lagna's navamsa (§4.3), at every slow
graha's sign ingress and the Moon's navamsa, because the instant clauses
read a graha's house and the rejected padas the Moon's quarter; and at a
blackout's edge, so a window a partial blackout touches lies inside it
whole and is **counted** and left out (`windows_blacked_out`). One chart
zodiac serves the search, taken at the instant the caller names: the
ayanamsha moves 0.14″ a day, 13″ over three months, about a second of a
window's edge. Over the built-in ephemeris at Kathmandu the baseline's
six heeds close 1 September to 19 November 2026 through the search
itself, every day by Chaturmas, and judge the eleven from Devuthani on
(`tests/search.rs`, which also holds, on every window of the days cut,
that the grahas' signs and the lagna's and the Moon's navamsas read the
same a second inside either end — red with the ingress cuts removed).

## 5. What is measured

`cargo xtask muhurta` (`muhurta-measured.md`, gated by `check-muhurta`):

- **The regression the baseline engine was rebuilt for.** A marriage
  search over 2026-09-01 to 2026-11-30 vetoes 2026-09-21, 2026-10-19 and
  2026-11-05, leaves nothing open before Devuthani Ekadashi, and keeps
  2026-11-25; under `BASELINE` the ranking is the engine's. The
  engine's own test of it reads the **season alone** — six heeded
  blackouts over UTC days — and that half holds already over the built-in
  ephemeris (`tests/season.rs`, red with Chaturmas left out).
- **The season against the published almanac** where the baseline engine
  recorded it (Shukra and Guru asta for BS 2083), per criterion.
- **A window is constant**: the judgement at a window's first and last
  instant agree, for every window of the search, and at the midpoint of
  two adjacent windows they differ.
- **C158 and C159 counted**: the days over a century where the two
  panchaka starts part, and how often the remainder and the per-star kind
  agree.

## 6. Order of work

1. The corrections first: C158 and C159 in `panchanga`, with a test per
   reading.
2. `teistro-muhurta`: the clause catalogue, the day's clauses, Tarabala
   and Chandrabala, the panchaka remainder.
3. The instant's clauses over a founded chart, and the cuts.
4. The season: blackouts from the lunar calendar and the heliacal events.
5. `ActivityRules`, Raman's marriage first, then the others; the search
   and the two rankings.
6. The measured pass and the regression.
7. The boundary: `muhurta_json` on a request, a blob section, every
   binding.

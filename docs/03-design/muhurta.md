# Muhurta: electing a time

Status: `built`, 2026-09-30; §6 steps 1 to 7 built 2026-09-30, the eclipse
blackouts (§4.1.1) and step 8, the rites beyond marriage (§4.6), 2026-10-01. Written from
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
| C189 | the eclipse's star | **the Moon's sidereal nakshatra at the greatest eclipse**, the *grahana nakshatra* | the Sun's, which at a solar eclipse is the Moon's but for an eclipse at a star's edge |
| C190 | which eclipses bar | **those the place sees**, a lunar one by its umbral phase (*Dharmasindhu*: an eclipse's time lasts while it can be seen) | every eclipse anywhere on the Earth |
| C191 | Raman's "six months" | **six synodic months** from the greatest eclipse, the months a muhurta counts | six solar months, about five days longer, which can add one passage of the Moon through the star |
| C193 | Holashtaka | **a kind no shipped activity heeds**, from the start of Phalguna's bright eighth to the full moon (*Shighrabodha* I.137–138: eight days, Ashtami to Purnima, barred for marriage and the like only on the Shutudri, Vipasha and Iravati and at Tripushkara, "auspicious elsewhere") | heeded everywhere, as many north Indian almanacs print it; or eight civil days rather than eight tithis |
| C194 | the horizon an eclipse is seen above | **the eye's**, the upper limb with refraction (`panchanga.eclipse_horizon`): *Dharmasindhu*'s test is the eye, and the committee's 2026-03-03 moonrise is 18:03 against 18:03.7 | the almanac's sunrise convention, 2 to 5 minutes later at a rising |
| C192 | the vedha as a blackout | **a kind no shipped activity heeds**: *Dharmasindhu*'s vedha is a rule about eating, and holding rites off in it is the almanacs' practice | heeded by a marriage, which no source in hand says |
| C167 | the malefics neutralisation 11 places in the 3rd, 6th or 11th | **the Sun, Mars and Saturn**: counted with both nodes, which always stand opposite, it could never hold | Rahu counted as well, Ketu not |

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

The season is asked of the `Sources` trait (`crates/muhurta/src/search.rs`):
`fn season(&self, range: Interval, kinds: &[BlackoutKind]) -> Result<Season,
Error>`, where `Season` is `{ blackouts, unjudged }`, the blackouts in order
of their start and the kinds asked for that the sources cannot judge. (The
design's `Season::over(range, place, &SeasonRules)` was not built.) Each
blackout is a kind and a window, computed once. The kinds are a catalogue
kind, `blackout`, because a consumer names them in a gate: `CHATURMAS`,
`ADHIKA_MASA`, `SAMSARPA`, `KSHAYA_MASA`, `KHARMAS`, `PITRU_PAKSHA`,
`GURU_ASTA`, `SHUKRA_ASTA`, `SANKRANTI` (Surya sankramana's sixteen
ghatis), `HOLASHTAKA` (§4.1.3), `ECLIPSE_STAR` (grahanotpatha) and
`ECLIPSE_VEDHA`. A kshaya
year splits its months three ways (§4.1.2). Asta is the heliacal event pair from
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
- **The combustion orbs are too narrow for the published Shukra asta.**
  Against the windows published for BS 2083 (`muhurta-measured.md`
  §2), the Surya Siddhanta's criterion and Ptolemy's hold both
  published windows whole, a day to eleven wider. The combustion orbs
  open Shukra asta 4 days after the almanac, and so would offer 14 to
  17 October, days the country treats as closed. The search's default
  stays the Surya Siddhanta's.
- **`ECLIPSE_STAR` waited for the eclipses**, which the SDK did not yet
  find; §4.1.1 builds it, and the vedha beside it.

#### 4.1.1 The eclipse blackouts

Two blackouts read an eclipse, and both read only the eclipses the place
**sees** (`eclipses.md` §4.5): *Dharmasindhu*'s eclipse chapter holds an
eclipse's time to last only while it can be seen with the eye
("चाक्षुषदर्शनयोग्य"), and after it sets eclipsed there is none, though
it goes on elsewhere (C190). A lunar eclipse is seen by its **umbral**
phase (`LunarView::umbral_seen`), so a penumbral one, which the eye
cannot see, makes neither.

- **`ECLIPSE_STAR`**, Raman's Mahadosha 16, *grahanotpatha*: the stars
  in which eclipses appear are avoided, and for a marriage "for six
  months" (ch. V). The star is the Moon's sidereal nakshatra at the
  greatest eclipse (C189). The blackout is every stretch the Moon stands
  in that star from the greatest eclipse to six synodic months after it
  (C191); an eclipse up to six months before a range still bars stars
  inside it, so the eclipses are found from that far back. Raman gives no
  span for other rites, so only `raman_marriage` heeds it.
- **`ECLIPSE_VEDHA`**, the almanacs' *sutak*: *Dharmasindhu* (p. 28, read
  off the page image) puts the vedha **four praharas** before a solar
  eclipse and **three** before a lunar one, counted in the day's
  praharas: the prahara holding the eclipse's first seen moment is found
  among the eight quarters of the day (sunrise to sunset) and the night
  (sunset to the next sunrise), and the vedha opens at the start of the
  prahara four (three) before it. Its own examples hold that reading: a
  solar eclipse in the day's first prahara bars the whole night before,
  a lunar one in the night's first bars from the day's second. A Moon
  that **rises eclipsed** takes four, which bars the day before its
  rising. The vedha closes when the eclipse ends as seen, or, when the
  body sets eclipsed, at its next rising, when the text has the
  observer bathe and eat. The knob `panchanga.eclipse_vedha` chooses
  `DHARMASINDHU` (the above) or `FULL_LUNAR_FOUR`, the view the text
  gives as "some say": four praharas for a lunar eclipse the umbra covers
  whole. A third member, `FIXED_HOURS`, is what Nepal's committee
  prints: fixed three-hour praharas counted back from the first moment
  seen, twelve hours before a solar eclipse and nine before a lunar one,
  a Moon rising eclipsed included. Three eclipses, as the press reported
  the committee, establish it, and `nepali-default` (version 4) takes it:

  | eclipse | first moment seen | no food from | until |
  |---|---|---|---|
  | 2025-09-07, lunar | 22:11 | 13:11 | 01:41 |
  | 2026-03-03, lunar, the Moon rising eclipsed | 18:03 | 09:03 | 19:02 |
  | 2022-10-25, solar, the Sun setting eclipsed | 16:52 | 04:52 | the next sunrise |

  A test holds all three to three minutes, the printed minutes being
  rounded. The committee's close follows the *Dharmasindhu*'s, and its
  exemption of children, the old and the sick is the text's. The text's shorter vedha for the young, the old and the sick
  (one and a half praharas, or three muhurtas) is declared and not
  built. The vedha is a rule about **food**; holding rites off in it is
  the almanacs' practice, so no shipped activity heeds it, and a
  consumer's rules name it (C192).

The praharas are read off the almanac's own days, so they move with
`panchanga.sunrise` as every other part of the day does.

A sky that cannot see an eclipse does not fail the search. Both kinds
are then reported among the answer's `unjudged`, each with why, and the
rest of the search goes on. That happens in two cases. A classical sky's
eclipse is its own method (C188). And some providers return a frame the
SDK cannot complete to an apparent topocentric Sun and Moon (the test
provider is one); their refusal is quoted in the reason. The search
learns which kinds were left unjudged from the season itself
(`Sources::season` returns a `Season` of blackouts and unjudged kinds).
That keeps the decision and its reason with the computation that
reached it. The parity runners found the second case. Before it was
handled, Raman's marriage over such a provider failed outright, in
every binding.

#### 4.1.2 A kshaya year

*Dharmasindhu* (p. 3, read off the page image) names the three months a
kshaya year marks. The adhika month **before** the kshaya month is
*samsarpa*, and it is fit for every rite and not to be given up for an
auspicious one. The kshaya month itself, *amhaspati*, and the adhika
month **after** it are avoided in every rite. So is an ordinary adhika
month, one that no kshaya month follows. The text puts the kshaya month
only in Kartika, Margashirsha or Pausha. The lunisolar pass holds that
over a millennium: 19 kshaya months, every one in those three, and
every one with an adhika month on each side
(`calendar-indian-lunisolar-measured.md` §5).

The season therefore makes three kinds:

- `SAMSARPA` is the adhika month whose next marked month is a kshaya
  month.
- `KSHAYA_MASA` is the kshaya month.
- `ADHIKA_MASA` is every other adhika month.

Whether an adhika month inside a range is the samsarpa depends on a
month that may lie past the range. The season looks
`SAMSARPA_REACH_MONTHS` (six) synodic months on. The lunisolar pass
measures the gap back from each kshaya month to its samsarpa, at most
five months, and refuses to write its page if the constant falls short
of any gap.

Raman's marriage heeds `ADHIKA_MASA` and `KSHAYA_MASA`, and so takes the
samsarpa as fit. That reading is *Dharmasindhu*'s; Raman's own text
names only the adhika month (C179). A caller who closes the samsarpa
too adds `SAMSARPA` to the activity's `heeds`. The baseline engine's
marriage closes every adhika month, so it heeds `ADHIKA_MASA` and
`SAMSARPA` together, and its regression is unchanged.

The kinds are the catalogue's `blackout_kind` (67). A closed day's
blackouts cross inside the muhurta answer's JSON as full keys
(`blackout_kind.ADHIKA_MASA`), written from the answer's one spelling
table, so each binding reads them as the generated type. A request's
`heeds` takes a member in either spelling. The kinds were spelled by
hand in each binding until then, held by a lint that is now retired
with the lists it held.

#### 4.1.3 Holashtaka

Kashinath's *Shighrabodha* (I.137–138, with Vaijnath Prasad's tika,
read off the page image, printed p. 33; the first verse is misnumbered
१६७ in that print) gives the eight days before Holi. They run from
Phalguna's bright eighth with the Purnima as their limit, and are
avoided for marriage and other auspicious acts. The text confines that
to the lands of the Shutudri, the Vipasha and the Iravati and to
Tripushkara, and says that elsewhere they are auspicious ("अन्यत्र
शुभ"). `HOLASHTAKA` is therefore built and heeded by no shipped
activity: Kathmandu is not on those rivers, and a consumer whose place
is names it (C193). The days are read as tithis: from the start of the
bright eighth (elongation 84°) of the nija Phalguna to the full moon,
eight tithis.

### 4.2 The clauses

A **clause** is one named condition from a source. As built
(`crates/muhurta/src/clause.rs`) it is `Clause { kind: ClauseKind, at:
Interval }`: what it says and when it held, clipped to the day; a
neutralisation is a clause of its own, not a `cancelled_by` list. The
design proposed a catalogue kind, `muhurta_clause`, one per Mahadosha,
shuddhi and named yoga; it was not spent, because the interpretation
corpus does not key by the clauses
([`muhurta-at-the-boundary.md`](muhurta-at-the-boundary.md) §2.4). Each clause is a pure function
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
side of an asta) rather than a silence. The
baseline engine's marriage rules come with the regression (step 6),
where they are measured.

`Ranking` is an enum rather than a trait, so a request and its answer
stay data (C162). A judgement is never collapsed into a number the SDK
invents; the ranking a request used is reported with the answer.
`Ranking::Texts` is Raman's excess of good and deficiency of evil: open
windows first, then the fewest clauses against, the most for, the
earliest. `Ranking::Baseline` joins it with the regression (§4.5).

Built (`search.rs`, `judge.rs`, `sources.rs`): the three passes, over a
`Sources` trait so the orchestration is the same over any sky, and
`ProviderSources` answering it over a provider. The kaalas are not a
request field: a day reports them as clauses, and a tradition that
refuses Rahu kaala lists `KAALA` among its bars. A window is cut, beyond
the day's own spans and the lagna's navamsa (§4.3), at every slow
graha's sign ingress and the Moon's navamsa, because the instant clauses
read a graha's house and the rejected padas the Moon's quarter; and at a
blackout's edge, so a window a partial blackout touches lies inside it
whole and is **counted** and left out (`windows_blacked_out`). And at
every held clause's own edges, by construction rather than by list: a
lagna tyajya's half ghati ends inside a navamsa, and the first list of
cut sources missed it. One chart
zodiac serves the search, taken at the instant the caller names: the
ayanamsha moves 0.14″ a day, 13″ over three months, about a second of a
window's edge. Over the built-in ephemeris at Kathmandu the baseline's
six heeds close 1 September to 19 November 2026 through the search
itself, every day by Chaturmas, and judge the eleven from Devuthani on
(`tests/search.rs`, which also holds, on every window of the days cut,
that the grahas' signs and the lagna's and the Moon's navamsas read the
same a second inside either end — red with the ingress cuts removed —
and that every clause a window holds covers it whole, red on a Cancer
tyajya before the clause edges were cut).

### 4.5 The baseline's ranking

`Ranking::Baseline` is the engine's weights (C162), which the
regression is stated in. Read in full, they are five scorers stacked,
each clamped to 0–100 as it is added:

1. **The day**, read at sunrise: a Nanda tithi +20, a Rikta −10, any
   other +10; the activity's star +25, else a harsh star −15; Monday,
   Wednesday, Thursday or Friday +15, Saturday −10; solar noon inside
   Rahu kaala −30.
2. **The day's shuddhi and the event**: the sunrise yoga −25 when
   highly inauspicious, −12 when inauspicious, +8 when auspicious (the
   catalogue's `Auspiciousness`, which agrees member for member); a
   Vishti anywhere in the day −18; panchaka −10; the special yogas +6
   each to 15; the Moon's brightness from the sunrise tithi +8 at 0.7 or
   more and −10 at 0.25 or less; the event's weekday −12 or +8 and its
   tithi +6; and against a native, the tara +12, or −10, −7 or −3 by
   the cycle for the 3rd, 5th and 7th, and the Moon's house from the
   birth Moon +10, −14 on the 8th, −6 when draining.
3. **The window's period**: Amrita +18, Shubha +14, Labha +12, and
   Abhijit +16 over the choghadiya it falls in. A window in no good
   choghadiya and not Abhijit is **not a candidate**, since the engine
   never builds it.
4. **The lagna** at the window: the lord exalted or in its own sign +12,
   debilitated −12, else in a kendra or trikona +6. The placements are
   summed and clamped to ±12: a benefic in a kendra or trikona +3, a
   malefic in an upachaya +2, a malefic in a kendra or trikona −3. Any
   graha in the 8th −8. For a marriage, the 7th empty +4, else −12.
   Malefics hemming the lagna −8. A cancelling benefic in the lagna:
   Jupiter +16, Venus +10, Mercury +7. **Only the first found counts**,
   in the engine's ephemeris order, so Mercury is taken before Jupiter.
   This is a reading of its loop, recorded, not corrected.
5. **The karakas** at the window (Venus and Jupiter for a marriage):
   exalted or in its own sign +8, debilitated −12; within 10° of the
   Sun (Venus) or 11° (Jupiter) −12; retrograde −6.

Then **the Mahadosha cap**. The Mahadoshas are:

- a highly inauspicious or inauspicious sunrise yoga;
- a Vishti in the day;
- a debilitated karaka;
- a combust karaka.

One cancelling benefic lifts one Mahadosha, except a combust karaka, which
no benefic lifts. What is left caps the score at 55, or at 30 when two or
more remain.

The SDK reads each quantity **where the engine does**: the day's at its
sunrise, taking the first span of each limb, and the window's at its
start. Its day, its sunrise and its window are the SDK's own. The
score is reported **with its factors**, one per dimension with its
signed weight, so a reader sees why a window scored 55 and not 92. The
engine's acceptance values are its own tests': a Saturday on the
bright eleventh with an auspicious yoga scores 92 as a day; the same
day scores 59 with Vyatipata, 74 with Vishti, 82 with panchaka and 100
with two special yogas; and it scores 68 on the dark fourteenth.

What the engine's rules decide is **rules**, not ranking.
`ActivityRules::baseline_marriage` gates by:

- the Sun's sign: Aries, Taurus, Gemini, Scorpio, Capricorn and
  Aquarius;
- the eleven stars;
- the six heeded blackouts;
- Rahu kaala.

Each gate is a clause the rules **bar**. Abhijit is not one of them. The
engine drops Abhijit's own candidate for a marriage but keeps the
choghadiya windows over the same minutes, so its prohibition takes away
Abhijit's bonus and does not remove the time. That is a flag on the
event, not a bar. A bar names a clause key, or one
clause exactly: `KAALA` bars all three kaalas, while
`{"clause": "KAALA", "kaala": "RAHU_KAALA"}` bars only the one the engine
avoids by default. The inputs the weights take per activity (the star
set, the favoured and avoided weekdays, the favoured tithis, the
karakas, whether the 7th must be empty, whether Abhijit is forbidden)
are data on the rules
(`BaselineEvent`). A request for `BASELINE` over rules without them is
refused, naming the field.

Where the SDK is exact and the engine samples, the two can part:

- The engine gates a **whole day** by its sunrise star and solar month,
  and vetoes a day any blackout touches. The SDK bars the **windows**
  inside a rejected star's span or a blackout, and closes only a day a
  blackout covers whole.
- The engine's windows are a choghadiya less the avoided kaala, scored
  at their start. The SDK's are cut wherever a clause changes.

The measured pass counts every such parting rather than hiding it (§5).

Built (`baseline.rs`): each scorer is split into what the engine
**reads** (`DayReading`, `Period`, the `Sky` at the window's start,
which now carries the grahas' speeds) and a pure score over that
reading, so the engine's own arithmetic is testable without a day. Its
tests' values cannot be taken whole: they fix the day part at 70, and
the day part the engine computes reaches **60 at most** (a Nanda 20,
the star 25, the weekday 15). So the unit tests assert the engine's
stated moves over a day the scorer does compute: Vyatipata −33 where
Siddhi was, Vishti −18, panchaka −10, and two special yogas clamped at
100. The search judges a **day** as well as its windows. A bar bars a
judgement only when its clause covers the judgement whole, so a day is
not barred by the Rahu kaala inside it and its windows in Rahu kaala
are. Over the built-in ephemeris the search under `Ranking::Baseline`
keeps the regression (1 September to 19 November closed), and scores an
open window **exactly** when the engine would have offered it. Of the
open windows, those in Char, Kaala, Udvega or Roga carry no score and
rank after the scored.

### 4.6 The rites beyond marriage (saait)

Nepal elects a saait for more than a marriage, and its national
panchanga committee prints the days for each. Four of Raman's chapters
are shipped as `ActivityRules` values beside his marriage
(`crates/muhurta/src/rites.rs`), each named by its key in every binding:

| key | Raman, *Muhurtha* | Nepal |
|---|---|---|
| `RAMAN_NAMAKARANA` | naming, ch. VIII pp. 58–59 | nwaran |
| `RAMAN_ANNAPRASANA` | first feeding, ch. VIII pp. 59–60 | pasni |
| `RAMAN_UPANAYANA` | thread ceremony, ch. VIII pp. 64–65 | bratabandha |
| `RAMAN_GRIHA_PRAVESHA` | house entry, ch. XII pp. 134–136 | griha pravesh |

Each was read off the page images, and each is laid over Raman's
general shuddhi as the marriage is: where a chapter names a member its
grade wins, and where it is silent the shuddhi's does. A chapter that
lists the good and is silent on the rest leaves the rest middling; one
that says the others "should be avoided" rejects them. The scan lacks
pp. 62–63 and 66–67, where the thread ceremony's opening and its
malefic yogas are; those are listed in `unjudged` rather than guessed.

**Unwanted placements.** The chapters name houses a graha should not
stand in, and houses that should stand empty, which no clause of the
marriage expressed. `ActivityRules.unwanted` is a list of
`Unwanted { grahas, houses, bars }`, and a window gets one
`UNWANTED_PLACEMENT { house, by }` clause per house where a named graha
stands, naming every one of them. `Unwanted::vacant(houses)` names all
nine grahas. A house outside 1–12 is refused at
`rules.unwanted[i].houses`.

**"Must" bars, "should" weighs** (crux C205). Raman writes some
placements "must" not be (the 10th for the first feeding, the 8th for
the thread ceremony) and others "should" not (Mercury in the 7th, a
benefic in the 6th). An entry marked `bars` bars a window it covers
whole, as its own `Bar::Clause` in `barredBy`; the rest are reported
and count against the time without closing it. Every placement barring
was built first, and closed nearly every thread-ceremony window.

**What a chapter asks and the SDK does not judge** — most often the
child's age, which a muhurta request does not carry — is listed in
`unjudged`. Two readings are recorded as cruxes: "Uttara" in the first
feeding is Uttara Phalguni (C203), and the house entry's months are
graded as printed, though two of them fall outside the northern course
the same chapter asks (C204).

**Measured** (`saait-measured.md`, gated by `check-saait`): the
committee's VS 2083 muhurta sheet, 94 rows read off its page images,
each printed day founded at Kathmandu over the committee's sky and the
modern one and searched with the rite's rules. Six rows print a weekday
their own date contradicts, and are read neither way. Every readable
bratabandha and griha pravesh day has a clean window; 24 of 35 pasni
days do, most of the rest parting at the 10th that must be empty; and
5 of 39 vivaha days do under Raman's marriage, which parts from the
committee by Tuesday, Pausha, dark-half tithis and yogas (C206). The
page also counts the days of the year each rite leaves open.

## 5. What is measured

`cargo xtask muhurta` (`muhurta-measured.md`, gated by `check-muhurta`),
over the built-in ephemeris at Kathmandu:

- **The regression the baseline engine was rebuilt for**, through the
  search under `ActivityRules::baseline_marriage` and
  `Ranking::Baseline`: 2026-09-21, 2026-10-19 and 2026-11-05 vetoed,
  nothing open before Devuthani Ekadashi, every closed day closed by
  Chaturmas, 2026-11-25 kept. It also holds that an open window is
  scored exactly when the engine would offer it.
- **The season against the published almanac**: Guru and Shukra asta
  against the BS 2083 windows, per criterion, claimed as holding the
  published window whole (a veto's safe direction).
- **A window is constant**: every window of two searches read a moment
  inside either end, and every clause held to covering its window. The
  design's first claim, that adjacent windows always differ, is
  **false** and is counted instead. About a third of neighbours are
  judged alike, meeting at a cut no clause the rules read changed at,
  such as a hora's edge.
- **Where the engine's sampling parts from the SDK**: the days the
  engine drops by the star at sunrise that the SDK keeps for a star
  later in the day (one way only, since the sunrise star runs in the
  day), and the scored windows shorter than the engine's six minutes.
- **C159 counted**: how often Raman's remainder names the kind the star
  does. C158 is not counted, being closed at rank 1, and its rival
  parts from it at every panchaka by construction.

`cargo xtask saait` (`saait-measured.md`, gated by `check-saait`) holds
the rites of §4.6 to the committee's printed days.

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
8. The rites beyond marriage (§4.6), the unwanted placements, and the
   saait pass.

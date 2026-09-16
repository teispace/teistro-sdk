# Yogas, doshas and the rule engine

Status: `research`, 2026-09-04. Checked against the baseline engine's data-driven rule
corpus (562 yoga rules, 62 dosha rules, evaluated by one algebraic
evaluator) and the counts published by JHora (184 yoga types), PyJHora
(about 284) and Shri Jyoti Star (a custom yoga builder that searches
databases).

## The observation that shapes the module

Every yoga, dosha, muhurta rule, matching koota, horary consideration and
Western configuration is a **predicate over chart state**, optionally with a
**strength** and a **cancellation** clause. The baseline engine already treats yogas and
doshas this way: each rule is a record of an algebraic condition tree over
primitives such as "body in house from reference", "body in dignity",
"aspect between", "lord of house in house", "count of bodies in kendra",
with citations. The SDK generalises this into one rule engine with typed
primitives and several rule packs, so that a consumer can author rules
without touching the engine and Shri Jyoti Star's "custom yoga builder"
becomes a data format.

## Primitive vocabulary the engine needs (from reading the baseline engine's rules)

| primitive family | examples |
|---|---|
| placement | body in sign, body in house from lagna or Moon or Sun or any body, body in a varga sign or house, body in nakshatra or pada, body in degree band, in kendra, trikona, dusthana, upachaya, panapara, apoklima |
| lordship | lord of house N (with co-lords for Scorpio and Aquarius), lord in house M, lords exchanging, lord conjunct, lord aspected, lord's dignity, lord's varga placement |
| dignity and state | exalted, debilitated, own, mooltrikona, friend, enemy, combust, retrograde, in war, vargottama, gandanta, hemmed between benefics or malefics (kartari) |
| relationship | conjunction (same sign, optionally within orb), graha drishti (full or partial), mutual aspect, rashi drishti, argala, exchange (parivartana) |
| counting and comparison | number of bodies satisfying a predicate, benefic/malefic classification (natural and functional per lagna), strength comparisons (Shadbala, degree), ordering (who is stronger) |
| context | day or night birth, waxing or waning Moon, gender, lagna sign, tithi, nakshatra of Moon |
| cancellation | neecha bhanga rules, kemadruma cancellations, Kaal Sarp exceptions, Manglik cancellations (which are themselves rules) |
| strength | contribution weights and multipliers, minimum thresholds, "percentage of formation" |

## Yoga families to cover

| family | count in classical sources | baseline | notes | tier |
|---|---|---|---|---|
| Pancha Mahapurusha (Ruchaka, Bhadra, Hamsa, Malavya, Sasa) | 5 | yes | strength depends on kendra and dignity | P0 |
| Chandra yogas (Sunapha, Anapha, Durudhara, Kemadruma, Adhi, Gaja Kesari, Chandra Mangala, Amala) with Kemadruma cancellations | ~12 | yes | | P0 |
| Surya yogas (Vesi, Vosi, Ubhayachari, Budha Aditya, Nipuna) | ~6 | yes | | P0 |
| Nabhasa yogas (Ashraya 3, Dala 2, Akriti 20, Sankhya 7) | 32 | yes | | P0 |
| Raja yogas (kendra–trikona lord relationships, Dharma-Karmadhipati, Viparita Raja, Neecha Bhanga Raja, Parivartana Raja, and the specific named ones) | dozens | yes | strength and "which houses" reporting | P0 |
| Dhana yogas and Daridra yogas | dozens | yes | | P0 |
| Arishta and Balarishta yogas with their cancellations | dozens | yes | | P0 |
| Parivartana (Maha, Khala, Dainya) | 3 classes over 66 pairs | yes | | P0 |
| Sanyasa, Vahana, Vidya, Kalatra, Putra, Matru, Pitru, Bhratru yogas by topic | dozens | yes | topic classification is part of the rule record | P0 |
| Lunar and solar eclipse yogas (Grahan) | | yes (dosha) | | P0 |
| Kala Sarpa and its 12 named types, partial and complete, with exceptions | 12 | yes | ascending and descending; Kala Amrita variant | P0 |
| Manglik (Kuja dosha) with the house sets by lagna, Moon and Venus, cancellations, severity | 1 with many variants | yes (severity 0–100) | | P0 |
| Doshas: Pitru, Guru Chandala, Gandanta, Ganda Moola, Kalathra, Ghata, Shrapit, Angarak, Vish, Chandal, Sade Sati flag, Daridra, Kemadruma | ~20 | yes (62 rules) | | P0 |
| Tajika yogas (16: Ithasala kinds, Ishrafa, Nakta, Yamaya, Manau, Kamboola, Gairi Kamboola, Khallasara, Radda, Duphali Kutta, Dutthotha Davira, Tambira, Kuttha, Durapha) | 16 | partial | orb tables (deeptamsha) | P0 |
| Yogas in vargas (D9, D10 specific) | | partial | | P1 |
| BV Raman's 300 Important Combinations | 300 | overlap | PyJHora's 284 include these | P1 |
| Western configurations (grand trine, T-square, grand cross, yod, kite, mystic rectangle, stellium) | ~10 | no | orb-based rule pack | P1 |
| Horary considerations before judgement (Lilly) | 7 | no | | P1 |

## Yoga strength, timing and interpretation

- **Strength**: the baseline engine reports presence plus a strength derived from the
  participants' Shadbala and dignity. JHora reports presence per rule.
  Parashara's Light gives yoga pages with strength commentary. The SDK
  should compute a rule-declared strength formula and expose the
  contributing factors as data.
- **Timing**: when a yoga fructifies (dasha or antardasha of participants,
  transit triggers). The baseline engine's `YogaTimingComposer` produces this text from
  the active dasha chain; the SDK exposes the underlying relation (rule
  participants versus dasha lords) as data.
- **Search**: Shri Jyoti Star and Kala search a chart database for yogas.
  This is the batch form of rule evaluation and is a first-class API.
- **Custom rules**: a rule pack format (versioned, validated, cited) that
  consumers can author. See `02-architecture/08-extensibility.md`.

## Closing checklist

- Export the baseline engine's 624 rules to the SDK rule format mechanically and diff the
  evaluation results on 100 charts (this is the golden-vector set for the
  engine).
- Decide the strength model and document it with citations.
- Define benefic and malefic classification variants (natural, functional
  per lagna with the trikona override, Moon phase dependent for Moon and
  Mercury's association rule).
- Confirm Rahu and Ketu treatment in Nabhasa and Kala Sarpa rules.

## The classical corpus, surveyed (2026-09-16)

Two research passes read the public-domain translations rather than the
secondary web, and downloaded them for re-checking: BPHS (Santhanam and
Sharma), Brihat Jataka (Chidambaram Iyer 1885), Saravali, Phaladeepika
(V. Subrahmanya Sastri 1937), Jataka Parijata (Sastri 1932), Kalaprakasika
and Uttara Kalamrita. Only some translations are free of copyright — Iyer
1885, Suryanarain Rao 1899, Sastri's Phaladeepika 1950 and Jataka Parijata
1932–33 — so the SDK cites locations and never reproduces passages.

### How many yogas there are

**"About 800 yogas" is not a classical number.** It is the yoga screen of a
commercial program, which counts text references matched against one chart.
Saravali ch. 21 does say its 32 Nabhasa yogas are "out of 1800 kinds
explained by the Yavanas", but that is one family expanded across lagnas, not
1800 rules. The defensible figure for distinct, deduplicated, *named* yogas
across every major text is roughly **400 to 700**, and which end depends on
three editorial choices: whether a planet-in-house reading counts as a yoga,
whether a family expanded per sign counts once or twelve times, and whether
the same combination named twice counts twice. **A count without its counting
rule is a claim the SDK cannot support**, so any number it publishes carries
one. The recording engine's 605 sit inside that band, with 255 attributed to
Phaladeepika, which almost certainly counts placement readings.

### What the corpus holds, by family

The yoga families and their authorities: Nabhasa 32 (BPHS ch. 35, Brihat
Jataka XII, Saravali 21); lunar 6 to 12 and solar 3 (BPHS chs. 37 and 38);
Pancha Mahapurusha 5 (BPHS ch. 75); raja 60 to 150 (BPHS chs. 39 and 40,
Phaladeepika 7, Bhavartha Ratnakara); dhana and daridra (BPHS chs. 41 and
42); arishta and ayur (BPHS chs. 43 and 44, Brihat Jataka VI); sanyasa (BPHS
ch. 79, Brihat Jataka XV); Jaimini (Upadesa Sutras); Tajika 16 (Tajika
Neelakanthi); and the dwigraha readings, which are 93 rules of one shape and
turned out to be two shapes rather than one (Brihat Jataka XIV's 21 pairs of
grahas in a sign, which Phaladeepika 18 vv. 1 to 5 repeats, and Phaladeepika
18 vv. 6 to 11's Moon in each of 12 signs under each of 6 aspects).

The dosha side is thinner than its reputation. What the texts actually carry
is the **arishta corpus** — about 70 short-life and parental-evil rules (BPHS
ch. 9, Saravali chs. 10 and 11, Brihat Jataka VI, Jataka Parijata IV) — and
about **49 arishta-bhanga cancellations** (BPHS ch. 10, Saravali ch. 12,
Phaladeepika XIII, Jataka Parijata IV), none of which the SDK has yet. The
curses of BPHS ch. 83, the inauspicious births of ch. 85 and their eleven
remedial chapters, the Gandanta of ch. 92 in its three kinds, and
Phaladeepika XIII's Vishaghatika, Thyajya and Dinamrityu are all rank 1 and
all unbuilt.

### What the texts do not say

Six of the recording engine's doshas have **no verse in seven full texts**:
Kalsarpa and its twelve named forms, Kala Amrita, Shrapit, Guru Chandal,
Angarak, Vish yoga and the three Rina doshas (which are Lal Kitab, 1939 to
1952, not Parashari). Sade Sati is not in any of them either; its classical
substitute is Phaladeepika XXVI's gochara with vedha. Others are
over-generalised rather than unsourced: the popular six-house Mangal against
BPHS ch. 80 v. 47's four, Mool dosha over all four padas where Phaladeepika
XIII v. 8 makes the fourth prosperous, a whole dark fortnight where BPHS
ch. 87 names Krishna Chaturdashi, and a generic tithi dosha where ch. 88
names tithi-kshaya. These are cruxes C88 to C91.

**Severity has no classical scale.** The texts give categorical outcomes and
one ordinal: Phaladeepika XIII v. 6's age bands (8, 20, 32, 70, 100 years)
and BPHS ch. 9 v. 2's twenty-fourth year. The numeric systems the texts do
carry — Shadbala in rupas, Ishta and Kashta out of 60, Vimshopaka out of 20,
Ashtakavarga rekhas with BPHS ch. 72's thresholds, and Guna Milan's 36 points
— grade strength, not affliction. Every 0 to 100 dosha score is software's
own, which is why `Severity` is rule data and the SDK reports what a rule
declares rather than a number of its own.

### What that means for the kernel

A citation now carries its rank (`Source.rank`, 1 to 4), so a consumer can
ask for only what a text supports; every rule and table the SDK ships sets
one, and the Kalsarpa family ships at rank 3 saying plainly that no verse was
found. The order of work below is by rules unlocked per unit of work:

| # | addition | unlocks | rules |
|---|---|---|---|
| 1 | the arishta corpus and its bhangas (needs 2 and 3) | BPHS chs. 9 and 10, Saravali 10 to 12, Brihat Jataka VI, Jataka Parijata IV | ~120 |
| 2 | papa and shubha kartari, and houses counted from any reference | Phaladeepika VI, BPHS ch. 80 v. 42, a third of the arishta corpus | 15 to 20 |
| 3 | the ghatika layer: sunrise-relative time, day and night gating, sandhya | Gandanta's three kinds, Abhukta Moola, Vishaghatika, Thyajya, Dinamrityu | 25 to 35 |
| 4 | upagrahas as bodies a rule can name, Gulika and Maandi first | BPHS ch. 83's curses, ch. 25's positional effects | ~70 |
| 5 | ~~the dwigraha generator over conjunction and sign~~ **built 2026-09-16** | Brihat Jataka XIV, Phaladeepika 18, Jataka Parijata | 212 of one shape: 21 pairs, the Moon in 12 signs under 6 aspects, and every combination of the seven from two to six |
| 6 | ~~rashi drishti and argala~~ **built 2026-09-16** | BPHS chs. 26 and 31 give both, so no Jaimini text was needed; 7 rules so far from chs. 29 and 39 | 40 to 80 |
| 7 | nakshatra and its lord for any body, with pada | BPHS ch. 48, the nakshatra family | 30 to 60 |
| 8 | D3 and D30 in the chart the kernel already steps into | Balarishta by drekkana, BPHS ch. 44's serpent decanate, ch. 80's trimsamsa | ~40 |
| 9 | ~~strength ranking and thresholds~~ **built 2026-09-16** | the chart carries the numbers and the kernel compares them; "the strongest of these" is `and` of the comparisons | 25 to 50 |
| 10 | the longevity band as an outcome, replacing invented scores | Phaladeepika XIII v. 6, BPHS ch. 43 | ~15 |

Two-chart matching (the ten kootas of Kalaprakasika XIII and the eight of
Muhurta Chintamani, which the popular Ashtakuta merges), transits for Sade
Sati, and the nadi texts stay out of Phase 6: each needs a context the kernel
does not have — a second chart, an ephemeris over time, or the nadiamsa.


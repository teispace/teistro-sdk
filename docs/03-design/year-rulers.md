# The year's rulers

Status: `implemented` 2026-11-01 (SR-46). Code: `crates/panchanga/src/rulers.rs`,
answered on every year `almanac.years` gives (`LunarYear.rulers`).

A panchanga opens its year with the year's office-bearers: the raja, the
mantri, and the lords of the crops, the grain, the rains and the rest.
Each office goes to the lord of a weekday, and each weekday is the day
of an event: Chaitra Shukla Pratipada for the raja, an ingress of the
Sun for every other office. This page records which texts say so, where
they differ, and how the SDK answers without choosing between them.

## 1. The texts

**Jyotirnibandha** (Shivaraja; ed. Anandashrama Sanskrit Series 85,
Poona 1919, on the Internet Archive as `Jyotirnibandh`), p. 29, the
section *atha rājādīnāṃ nirṇayaḥ*, vv. 1–10 (rank 1). Verses 7–10,
quoted from *Jyotiṣaphalodaya*, give six offices and the event of each;
v. 10 sums them up: *caitrādi-meṣa-cāpa-ārdrā-tulā-karka-dineśvarāḥ /
nṛpa-mantri-dhānya-toya-rasa-sasyādhipeśvarāḥ*, the lords of the days
of Chaitra's start, Mesha, Chapa (Dhanu), Ardra, Tula and Karka are the
king, the minister and the lords of grain, water (the clouds), juices
and crops. Pages 30–32 then give each office's results by its graha,
under the headings *rājaphalam*, *mantriphalam*, *agradhānyeśaphalam*
(the lord of the crops, Karka's), *megheśaphalam*, *rasādhipaphalam* and
*paścimadhānyeśaphalam* (the lord of grain, Dhanu's).

**Narada Samhita**, ch. 2 vv. 1–3 (ed. Ram Janma Mishra with the Vimala
Hindi commentary, Kashi Sanskrit Series 40, Chaukhamba 1984, pp. 6–7;
rank 1). Verse 1 makes the lord of Chaitra Shukla Pratipada's weekday
the year's lord, v. 2 the lord of Mesha's ingress the *bhūpati* (the
commentary: the minister, some read the commander) and of Karka's the
lord of crops, v. 3 the lord of Tula's the lord of juices and of
Makara's the lord of dry goods (*nīrasa*), whom *Jyotirnibandha* does
not name.

**The southern nine** (*navanāyaka*), printed every year in Telugu and
Kannada panchangas and their web copies: the seven above with the
commander (*senādhipati*) at Simha's ingress and the lord of prices
(*arghādhipati*) at Mithuna's. No verse for these two has been read
(C1071); they are answered and named as this practice's, rank 3.

So the texts agree on every event they share and differ only in which
offices they list. Each office in the answer names its texts
(`texts`): `JYOTIRNIBANDHA`, `NARADA_SAMHITA`, `NAVANAYAKA`.

| office | event | texts |
|---|---|---|
| `RAJA` | Chaitra Shukla Pratipada | all three |
| `MANTRI` | Mesha, 0° | all three |
| `SENADHIPATI` | Simha, 120° | the southern nine |
| `SASYADHIPATI` | Karka, 90° | all three |
| `DHANYADHIPATI` | Dhanu, 240° | *Jyotirnibandha*, the southern nine |
| `ARGHADHIPATI` | Mithuna, 60° | the southern nine |
| `MEGHADHIPATI` | Ardra, 66°40′ | *Jyotirnibandha*, the southern nine |
| `RASADHIPATI` | Tula, 180° | all three |
| `NIRASADHIPATI` | Makara, 270° | *Narada Samhita*, the southern nine |

The ingresses are the Sun's sidereal ones in the almanac's own sky, the
one that found the year's opening and named its months, so a year read
under the Surya Siddhanta takes the text's sankrantis. Each is the
first after the opening new moon, which has the Sun in Meena, so all
fall inside the year.

## 2. Which day an event's weekday is

### 2.1 The raja

*Jyotirnibandha* vv. 1–6 give four readings of the raja's day, and the
answer carries three of them, each named (`rule`):

- `SUNRISE`, vv. 2–4 and *Narada Samhita* 2.1: the weekday at the
  sunrise Chaitra Shukla Pratipada runs at (*raver udayakālikaḥ*, "said
  by many"). V. 3 settles the two cases v. 2 leaves open: of two such
  sunrises the earlier, and when the pratipada runs at none
  (*nodayayugale 'pi pūrvaḥ syāt*) the earlier day as well, the one the
  pratipada began in. This is the first reading, the texts' own
  conclusion.
- `NEW_MOON`, v. 1, from *Ratnāvalī*: the weekday holding the junction
  of the new moon and the pratipada (*darśa-pratipat-sandhau*).
- `GARGA`, v. 6: the day of the new moon if it falls before that day's
  midday, else the next. Midday is halfway from sunrise to sunset.

The fourth, v. 1's second half (*aparaiḥ pratipadi madhyāhnakāle yaḥ*,
the weekday whose midday the pratipada holds), is not built: a short
pratipada that begins after one midday and ends before the next holds
none, and the verse gives no day for it.

The kshaya case is not rare. VS 2083's pratipada began at 07:08 at
Kathmandu on Thursday 19 March 2026 and ended before Friday's sunrise,
so the lunar year's first sunrise (`began`) is Friday's while the raja,
by v. 3, is Thursday's Jupiter, as the press printed in the north and
the south.

### 2.2 The other offices

The verses say the lord of "the day of the entry" (*yasmin dine*,
*praveśe 'hani*, *divasādhinātha*). A weekday runs from sunrise to
sunrise, but *dina*, *ahan* and *divasa* are also the daylight, and the
printed panchangas read them so (C1072). The answer carries both:

- `DAYLIGHT`, first: the day whose daylight holds the ingress, and for an
  ingress at night the day that begins at the next sunrise.
- `VARA`: the weekday, sunrise to sunrise, holding the ingress.

The southern nine printed for Hyderabad for 2026–27 (§3) put Karka's
ingress at 23:44 on Thursday 16 July under Venus, Tula's at 19:57 on
Saturday 17 October under the Sun and Makara's on the night of 14
January 2027 under Venus: each the next day, the daylight reading. The
weekday reading gives Jupiter, Saturn and Jupiter.

## 3. What the tests hold

`crates/sdk/tests/year_rulers.rs`, over the built-in ephemeris and the
`nepali-drik` profile:

| year | place | printed | source |
|---|---|---|---|
| VS 2080 | Kathmandu | Mercury raja, Venus mantri | a patro printed in Nepal, as reported by the e-Himalaya Times, March 2023, citing *Jyotirnibandha* v. 3 |
| VS 2082 | Hyderabad | the Sun raja, the Moon mantri (Mesha at 03:30 on a Monday) | hindupad.com's navanayaka for Vishwavasu |
| VS 2083 | Hyderabad | all nine: Jupiter, Mars, the Moon, Venus, Mercury, the Moon, the Moon, the Sun, Venus | onlinejyotish.com's Ugadi panchangam, Parabhava |
| VS 2083 | Kathmandu | Jupiter raja on a kshaya pratipada | §2.1 |

Each is asserted under the rule that printed it, and each reading the
other rules give where they part is asserted too. These are rank-3
printings; none of the three texts gives a worked year.

One printing does not fit either day reading: the same source's
Vishwavasu (2025–26) gives Saturn as commander, Simha's ingress being at
01:41 on Sunday 17 August 2025 by the modern sky, which is Saturday's
weekday but Sunday's daylight, while its mantri needs the daylight
reading. A panchanga reading the Surya Siddhanta's sankrantis, hours
from the modern ones, would explain it; it is not asserted (C1072).

## 4. Shape

Each year's `rulers` lists the nine offices in the southern order, each
`{office, event, texts, readings}`, and each reading
`{rule, vara, lord, day}`: the rule, the weekday and its lord (in full at
the boundary, `vara.GURUVARA`, `graha.JUPITER`) and the sunrise that
opened that weekday. `YearRuler::lord` is the first reading's lord.

The rulers ride on the `years` answer and section, which every binding
reads; a sankalpa's year leaves them empty, since it asks only the name
and the numbers and the nine ingresses would cost it a search a day.

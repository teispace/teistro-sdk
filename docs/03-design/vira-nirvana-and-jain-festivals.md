# The Vira Nirvana Samvat and the Jain festivals

Status: `built`, 2026-11-01 (SR-59, `docs/consumers/addendum-3.md` §10).
Cruxes C1111 to C1114 (`01-research/feature-universe/19-verification-cruxes.md`).

The consumer asked for the Jain era and a Jain festival pack. Research came
first, because the sources give the era's epoch and its year boundary
differently, and the two sects and the Shvetambara gacchas keep some days on
different tithis. Every such difference is a setting or a named community
here, never a guess.

## 1. Sources

| rank | source | what it gives |
|---|---|---|
| 1 | *Kalpa Sutra*, Life of Mahavira §122 and §127 (Jacobi, *SBE* 22, 1884) | the nirvana on the fifteenth of Kartika's dark fortnight (purnimanta), in its last night; the illumination on that new moon |
| 1 | *Kalpa Sutra*, rules for ascetics §1 (Jacobi, 1884) | Paryushana a month and twenty nights into the rains: Bhadrapada's bright 5th |
| 1 | Samantabhadra, *Ratnakaranda Shravakachara* v. 106 | the proshadhopavasa on the parva, the 8th and the 14th (the Digambaras' monthly fasts) |
| 1 | Hemachandra, *Parishishtaparvan* VIII.339 | Chandragupta crowned 155 years after the nirvana |
| 1 | Jinasena, *Harivamsha* (783 CE) | the nirvana 605 years and 5 months before the Shaka era |
| 2 | Sircar, *Indian Epigraphy* (1965), pp. 321-322 | the era in use: 470 years before the Vikrama, 527 BCE |
| 2 | *Tithidin ane Parvaradhan tatha Arhattithibhaskar* (Jain Pravachan Pracharak Trust, 1977), p. 257 | Umasvati's rule: *kṣaye pūrvā tithiḥ kāryā, vṛddhau kāryā tathottarā* |
| 2 | JAINA Education Committee, *Compendium of Jainism* (3rd ed.), pp. 246-250 | Paryushana and Das Lakshana; Mahavir Jayanti on Chaitra's bright 13th; Diwali the nirvana and the new year the day after; the Navpad Oli in Chaitra and Ashvina; the twelve parva tithis |
| 2 | Sangave, *Jaina Community* (2nd ed., 1980), p. 243; Doniger (ed.), *Encyclopedia of World Religions* (1999), p. 555 | Das Lakshana from Bhadrapada's bright 5th to the 14th; Samvatsari on the bright 4th |
| 3 | Drik Panchang's Jain calendar for New Delhi, 2025 and 2026; community calendars (JCNC 2025, JSMW 2026, USC 2026, Harvard FAS 2025) | the acceptance days of §4 |

## 2. The era

`teistro_calendar::vira_nirvana` reads the era off the lunisolar calendar the
almanac already keeps, as Nepal Sambat is read: its year is a Kartikadi lunar
year, counted by the civil year the year's middle falls in
(`lunisolar::kartikadi_middle`, shared with Nepal Sambat). VS 2082 (Kartikadi)
opened on 22 October 2025, and so did VNS 2552. The catalogue gains the era
member `VIRA_NIRVANA`.

Two points differ between the sources, and each is a field of `Reading`:

| field | default | the rival | why the default |
|---|---|---|---|
| `epoch` | `TRADITIONAL`: 470 years before the Vikrama (527 BCE), which Sircar and the *Harivamsha*'s 605 years and 5 months before the Shaka both give | `HEMACHANDRA`: 410 years before the Vikrama (467 BCE), since the *Parishishtaparvan* puts Chandragupta 60 years nearer the nirvana than the count in use | every Jain calendar in hand prints the traditional number |
| `yearStart` | `KARTIKA_SHUKLA_PRATIPADA`: the day after the nirvana, when Gautama attained kevala (*Compendium* p. 248), the day the Kartikadi Vikrama year opens | `NIRVANA_DAY`: the day whose sunrise holds a nija Ashwina's new moon (amanta), as the USC calendar prints the Jain new year on 9 November 2026 | the manual's reading, and the Gujarati new year it shares |

A day answers its year through `Panchanga::vira_nirvana(reading)`, and a lunar
year carries `viraNirvana`, the traditional year its Kartika opens (the
Vikrama plus 470), in the `years` section every binding reads. The reading is
not yet a profile knob (C1114).

## 3. The pack

`JAIN` holds 19 rules by the tithi and one counted from another's day. Every
rule reads the tithi at sunrise. A parva tithi (the 2nd, 5th, 8th, 11th, 14th
and 15th of a half) is never lost or doubled in a Jain calendar: Umasvati's
rule moves the loss or the doubling onto the tithi before it. So a lost parva
tithi is kept on the day of the one before it (the `NEITHER` case takes the
earlier day) and a doubled one on its second day (`BOTH` takes the later),
where Nepal's sunrise rule keeps a doubled tithi's first day. Any other tithi
is kept on the first sunrise that holds it. The first measurement read every
tithi by Umasvati's rule, and the Ashvina Oli of 2026, whose bright 7th holds
two sunrises, came a day after the 17 October Drik prints; the rule applies to
the parva tithis alone, and read so the day agrees.

| key | tithi (amanta) | kept by |
|---|---|---|
| `MAHAVIR_JAYANTI` | Chaitra, bright 13th | both sects |
| `MAHAVIR_NIRVANA` | Ashwina's new moon (purnimanta Kartika's dark 15th) | both |
| `VIRA_NIRVANA_NEW_YEAR` | the day after `MAHAVIR_NIRVANA` | both |
| `PARYUSHANA`, `SAMVATSARI` | Shravana dark 12th; Bhadrapada bright 4th | Shvetambara: Tapagaccha, Kharataragaccha |
| `PARYUSHANA_TRAYODASHI`, `SAMVATSARI_PANCHAMI` | Shravana dark 13th; Bhadrapada bright 5th | Shvetambara: Achalagaccha, Sthanakvasi, Terapanth |
| `DASA_LAKSHANA`, `ANANTA_CHATURDASHI` | Bhadrapada bright 5th; bright 14th | Digambara |
| `CHAITRA_AYAMBIL_OLI`, `…_END` | Chaitra bright 7th; full moon | Shvetambara |
| `ASHVINA_AYAMBIL_OLI`, `…_END` | Ashwina bright 7th; full moon | Shvetambara |
| `JAIN_PARVA_ASHTAMI`, `JAIN_PARVA_CHATURDASHI` | each half's 8th; 14th | both |
| `JAIN_PARVA_DVITIYA`, `…_PANCHAMI`, `…_EKADASHI`, `…_PURNIMA`, `…_AMAVASYA` | each half's 2nd, 5th, 11th; the full and the new moon | Shvetambara |

The sects and gacchas are the rules' `communities` (`SHVETAMBARA`,
`DIGAMBARA`, `TAPAGACHHA`, `KHARATARA_GACHHA`, `ACHALGACHHA`, `STHANAKVASI`,
`TERAPANTH`), so a request's `only` filter keeps one tradition's days. Every
rule is `RELIGIOUS_CULTURAL` and `JAIN`; no source in hand gives a region.

## 4. Against the published calendars

`crates/sdk/tests/jain.rs` holds the pack, at New Delhi over the built-in
ephemeris, to 21 days the calendars of §1 print for 2025 and 2026.

All 21 agree:

| year | key | printed | by |
|---|---|---|---|
| 2025 | `MAHAVIR_JAYANTI` | 10 April | Drik |
| 2025 | `CHAITRA_AYAMBIL_OLI`, `…_END` | 4 and 12 April | Drik |
| 2025 | `PARYUSHANA_TRAYODASHI`, `SAMVATSARI_PANCHAMI` | 21 and 28 August | Drik |
| 2025 | `SAMVATSARI` | 27 August | JCNC's Shvetambara Paryushan |
| 2025 | `ASHVINA_AYAMBIL_OLI`, `…_END` | 29 September and 7 October | Drik |
| 2025 | `MAHAVIR_NIRVANA`, `VIRA_NIRVANA_NEW_YEAR` | 21 and 22 October | Harvard FAS |
| 2026 | `MAHAVIR_JAYANTI` | 31 March | Drik |
| 2026 | `CHAITRA_AYAMBIL_OLI`, `…_END` | 25 March and 2 April | Drik |
| 2026 | `PARYUSHANA`, `SAMVATSARI` | 8 and 15 September | Drik |
| 2026 | `DASA_LAKSHANA`, `ANANTA_CHATURDASHI` | 16 and 25 September | JSMW's Das Lakshan |
| 2026 | `ASHVINA_AYAMBIL_OLI`, `…_END` | 17 and 26 October | Drik |
| 2026 | `MAHAVIR_NIRVANA` | 9 November | USC |
| 2026 | `VIRA_NIRVANA_NEW_YEAR` | 10 November | Drik (the Kartikadi new year) |

The 2025 rows bear out the gacchas' split: the calendars print Samvatsari on
both the bright 4th (27 August) and the bright 5th (28 August), and each
reading's rule finds its own.

## 5. What waits

- C1111: the published calendars disagree on the Shvetambara Samvatsari: Drik
  Panchang prints 28 August 2025 (the bright 5th) and 15 September 2026 (the
  bright 4th). Both gaccha readings ship; which a consumer keeps is theirs.
- C1112: the gacchas' two reckonings are relayed by an encyclopaedia, not read
  from a gaccha's own calendar; and whether the Digambara calendars apply
  Umasvati's rule to a doubled tithi is unsourced.
- C1113: the Ayambil Oli's bright 7th to full moon is the community calendars'
  reckoning; the manual gives only the months and the nine days.
- C1114: the era's `Reading` is a Rust argument, not yet a `calendars` knob
  in the settings document, which would change the settings schema.
- Kshamavani (the day after Das Lakshana, or the dark 1st) is not shipped: the
  calendars in hand print it on two different days.

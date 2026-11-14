# Festival metadata

Status: `built`, 2026-10-30; the fast (§6, SR-62) 2026-11-01. SR-02. Crux C850
(`01-research/feature-universe/19-verification-cruxes.md`). The values are
held to `festival-classification.md`'s tables and to two governments'
notices by the rows of `crates/panchanga/src/festival/meta.rs`.

A consumer asked for kind, tradition, communities and regions on every rule,
"so any consumer filters a pack without its own mapping". This page says what
each field means, where its values come from, and what a rule says when no
source gives one.

## 1. The shape

A rule of any of the four kinds a pack holds (lunar, Ekadashi, counted, dated)
carries `meta`, a `FestivalMeta`. A published `ADD` entry carries one too,
since it has no rule to carry it.

| field | type | meaning |
|---|---|---|
| `kind` | `festival_kind`, closed | `GAZETTED`, `RELIGIOUS_CULTURAL`, `REGIONAL_ETHNIC`, `OBSERVANCE` |
| `tradition` | `tradition`, closed | `HINDU`, `BUDDHIST`, `JAIN`, `SIKH`, `ISLAMIC`, `CHRISTIAN` |
| `communities` | open keys | `NEWAR`, `THARU`; a consumer's own pack may add others |
| `regions` | open keys | an ISO 3166 country first, then a part a notice names |
| `optional` | boolean | a notice leaves the day to each office's choice |
| `fast` | list of `VrataFast` | the fast kept on the day, one reading a clause (§6) |

Communities and regions are open because a consumer's own rules name their
own, which a closed list would refuse. A rule the consumer wrote has an empty
`meta` until it sets one, and serialises without the field, so a rule without
values hashes as it did.

## 2. Where each value comes from

Nothing is inferred from a festival's name. In order:

**`GAZETTED`.** A government's notice lists the day as a public holiday.

- Nepal: the Ministry of Home Affairs' lists for 2082 and 2083 (Nepal Gazette,
  relayed in Pradhan & Associates' alerts of those lists). 49 keys. The
  notice's limits become values: Gai jatra is `NEWAR` in `KATHMANDU_VALLEY`,
  Indra and Ghode jatra `KATHMANDU_VALLEY`, Fagu purnima's rule `HILLS` (the 56
  districts) and Terai Holi `TERAI` (the 21).
- India: the Department of Personnel and Training's memoranda
  (`india-central-holidays.md`).

**`REGIONAL_ETHNIC`.** Not gazetted, and `festival-classification.md`'s community
column names a people: Newar, Kirat, Tharu, Thakali, Gurung, Magar, Majhi, Sherpa,
Tamang, Maithil, Bhojpuri, Rajbanshi or Kanu.

**`RELIGIOUS_CULTURAL`.** Not gazetted, and either that column names a religion
(Buddhist, Sikh, Jain, Muslim, Christian) or the rule cites a Hindu text
(*Dharmasindhu*, *Nirnayasindhu*, *Hari Bhakti Vilasa*).

**`OBSERVANCE`.** Not gazetted and in the classification's fixed civil class: a
date fixed by law, an institution or an organisation.

**`tradition`** is the religion the community column names, or `HINDU` when the
rule cites one of those texts. `communities` is that column's peoples, a
composite row split (`Maithil and Bhojpuri` is two), plus the notice's. `regions`
is `NP` for a rule of Nepal's packs, which the national panchanga prints, plus
what the notice names (a province's too: Bagmati's notice makes `LANGUAGE_DAY`
`GAZETTED` in `NP-P3`, C1802), and `IN` for India's pack.

**Unset.** Of the 482 keys of Nepal's packs, 50 are `GAZETTED`, 65
`REGIONAL_ETHNIC`, 48 `RELIGIOUS_CULTURAL`, 142 `OBSERVANCE` and 177 have no kind.
422 have no tradition. They are a fitted day of a mela, a fast or a jatra whose
source is the committee's print, which says when and not what. No value is
guessed for them, and the classification's blank is not one either: its own
words are "kept across communities, or the community is not known here". The
town in a key such as `BHAKTAPUR_BRAHMAYANI_JATRA` is not read as a region, since
that would be inferring from a name. C850 holds what would close them.

`*Dharmasindhu*`'s packs (`DHARMASINDHU`, `PITRI_PAKSHA`) and `GAUDIYA` are
`RELIGIOUS_CULTURAL` and `HINDU` by their text, and name no region.

## 2a. Regions by ISO code (SR-89)

`regions` names a country by its ISO 3166-1 alpha-2 code (`IN`, `NP`) and a
state by its ISO 3166-2 code, so a place a geo provider resolves (SR-87)
selects its days with no table of the consumer's. `region::REGIONS` lists
every key a shipped set names:

| key | first shipped as | kept by |
|---|---|---|
| `IN` | | `INDIA_CENTRAL`, and every state's days besides its own code |
| `IN-AP` | | `INDIA_ANDHRA_PRADESH` |
| `IN-AS` | `ASSAM` | `INDIA_ASSAM` |
| `IN-GJ` | | `INDIA_GUJARAT` |
| `IN-KA` | | `INDIA_KARNATAKA` |
| `IN-KL` | `KERALA` | `INDIA_KERALA` |
| `IN-MH` | `MAHARASHTRA` | `INDIA_MAHARASHTRA` |
| `IN-OD` | | `INDIA_ODISHA` |
| `IN-TG` | | `INDIA_TELANGANA` |
| `IN-TN` | `TAMIL_NADU` | `INDIA_TAMIL_NADU` |
| `IN-UP` | | `INDIA_UTTAR_PRADESH` |
| `IN-WB` | `WEST_BENGAL` | `INDIA_WEST_BENGAL` |
| `NP` | | `NEPAL_PANCHANGA` and `NEPAL` |
| `NP-P3` | | `NEPAL_PANCHANGA`'s `LANGUAGE_DAY`, Bagmati Province's holiday by its own notice (C1802) |
| `KATHMANDU_VALLEY`, `HILLS`, `TERAI` | | rules of Nepal's packs a notice limits |

**Aliases.** The name a state was first shipped under is read as its code
wherever a region is read, in a filter or in a caller's own rule, so
`{"regions": ["TAMIL_NADU"]}` keeps what `{"regions": ["IN-TN"]}` keeps. An
answer carries the code. Nepal's three parts keep their own keys because no
ISO code names them: ISO 3166-2's codes for Nepal are its provinces, and the
Kathmandu valley's three districts are a part of one, the 56 hill districts
and the 21 of the Terai parts of several.

**A region names its pack.** Wherever a request names a pack, in `rules`
or in `almanac.festivalPack`, it may name a region instead
(`FestivalPack::for_region`): the pack whose `region()` it is, else its
country's, so `IN-HR`, a state with no pack of its own, keeps
`INDIA_CENTRAL`, and `NP-P3` keeps `NEPAL_PANCHANGA`, the national print's
set. A code is read as written, upper case; a country no pack keeps, or a
key no ISO code is, is refused under the request's field with the regions
listed.

## 3. Reading a pack

`almanac.festivalPack`, `ts_almanac_festival_pack`, answers every row of a pack:
its key, form, source, `meta`, and `unsourced`, the fields no source gives by
name: `kind`, `tradition`, `communities`, `regions`, `fast`, and `fast.kind`,
`fast.start` or `fast.end` when the fast is known and no reading gives that
part. That list is how a rule "says so": a reader cannot take a blank for an
answer. Nothing is reckoned, so a context with no ephemeris answers.

## 4. Filtering

A festival request takes `only`, a `MetaFilter`:

```json
{"rules": "NEPAL_PANCHANGA", "only": {"kinds": ["GAZETTED"], "regions": ["KATHMANDU_VALLEY"]}}
```

A rule stands when, of each list given, its `meta` holds a member. A rule with
that field unset fails a list that is given, because a filter promises what it
keeps. Every rule is still reckoned, so a day counted from one filtered out
stands. The filter runs on the answer, after the published sets and before the
saits, so a filtered rule pays for no sait. A family's anniversaries carry no
values, so they stand only when no filter is set.

SR-11's feed filter by kind or tradition reads this field.

## 5. What it costs

Each pack rule holds a few short strings more, written into the request's
content hash, so **every shipped pack's `input_hash` moved once** with this
change. No day moved. The table is 22 groups and 482 sorted keys, searched by
bisection, so stamping a pack does not scan.

Names: `festival_kind` and `tradition` are named in `en-Latn` and `ne-Deva-NP`;
the Nepali strings await the reviewer of Q42. Communities and regions are keys
and have no names yet.

## 6. The fast (SR-62)

The consumer asked for "the fast's kind (nirjala, phalahar, ekabhukta), its
start and its parana window as catalogue fields on every vrata". `meta.fast`
is a list of readings, one for each way a text in hand keeps the fast, in
the text's order, so a text that lists ways (Hemadri's "उपवासेन नक्तेन
तथैवायाचितेन च") is reported as its clauses rather than reduced to one.

| field | type | meaning |
|---|---|---|
| `kind` | `fast_kind`, closed | how it is kept; unset when the clause names no kind |
| `ifUnable` | boolean | the text allows it only to one unable to keep the reading before it |
| `start` | `AFTER_EKABHAKTA`, `MORNING` | when it begins; unset when the clause does not say |
| `end` | `PARANA`, `NIGHT`, `MOONRISE` | when it is broken; unset when the clause does not say |
| `source` | text | the page and the words |

**The kinds** are the texts' own. *Dharmasindhu* (1888 Nirnaya-sagara
edition) p. 6 defines four observances of the body: `EKABHAKTA`, one meal of
one grain at midday; `NAKTA`, the one meal at night, in the pradosha after
the three ghatis of dusk (pp. 6–7); `AYACHITA`, what comes unasked; and
`UPAVASA`, no meal by day or night (अहोरात्रभोजनाभावः). P. 18 lets one unable
to keep the Ekadashi take `JALAHARA` (water), `KSHIRAHARA` (milk),
`PHALAHARA` (fruit) or a night meal instead, and p. 52 fruit at Janmashtami.
The consumer's "ekabhukta" is `EKABHAKTA` and "phalahar" `PHALAHARA`.
"Nirjala" has no member: no text in hand says "without water", and DS's
Ekadashi allows water to the unable (C1080).

**The end** is the parana window as a catalogue can say it; the instants are
on the answer. `PARANA` is a later day: DS p. 7 breaks every ekabhakta, nakta,
ayachita and upavasa on the next day at the tithi's end, and a rule's
`parana` readings (`festival-rules.md` §8.7) or an Ekadashi's `ParanaRule`
answer the window. A pack that carries no readings, as Nepal's do not (C706),
leaves it unreckoned while saying the fast (C1084). `NIGHT` is the night meal
of a nakta. `MOONRISE` is the meal after the arghya to the Moon, and the
observance's `moonrise` gives the instant and the place it was found for.

**What is sourced** (`crates/panchanga/src/festival/meta.rs`, `FASTS`):

| key | readings | source |
|---|---|---|
| every Ekadashi: `EKADASHI_SMARTA`, `EKADASHI_VAISHNAVA`, `EKADASHI_SMARTA_RENUNCIANT` and the 25 named ekadashis `NEPAL_PANCHANGA` prints | `UPAVASA` after the 10th's ekabhakta, to the parana; then, if unable, `JALAHARA`, `KSHIRAHARA`, `PHALAHARA` to the parana, and `NAKTA` to the night | DS pp. 17–18 |
| `JANMASHTAMI` | `UPAVASA` from the morning, to the parana; if unable, `PHALAHARA` | DS p. 52 |
| `SHIVARATRI` | `UPAVASA` after the 13th's ekabhakta, to the parana | DS p. 90 |
| `RAMA_NAVAMI` | `UPAVASA` after the 8th's ekabhakta, to the parana | DS pp. 33–34 |
| `MAHA_NAVAMI` | `UPAVASA`, to the parana | DS pp. 69–71 |
| `HARITALIKA` | `UPAVASA`, enjoined on women | DS p. 55 |
| `PRADOSHA_VRATA` | `NAKTA`, to the night | DS p. 20 |
| `SANKASHTA_CHATURTHI` | the meal at night; kind and start unsourced | *Nirnayasindhu* (1949 Nirnaya Sagar edition) p. 22; C1081 |
| `ASHUNYA_SHAYANA` | `NAKTA`, to the moonrise; then `UPAVASA` and `AYACHITA`, ends unsaid | DS p. 44; Hemadri, *Vratakhanda* (Bibliotheca Indica, 1878) ch. 9, pp. 172–173 |
| `EKADASHI_GAUDIYA` | `UPAVASA`, to the parana; if unable, `NAKTA` to the night, `AYACHITA`, `EKABHAKTA`, then `PHALAHARA`, `KSHIRAHARA` and `JALAHARA` to the parana | HBV 12.94–98 |
| `JANMASHTAMI_VAISHNAVA` | `UPAVASA`, to the parana | HBV 15.377, 15.385 |
| `RAMA_NAVAMI_VAISHNAVA` | `UPAVASA` after the 8th's meal, to the parana | HBV 14.252, 14.255–259 |
| `NRISIMHA_CHATURDASHI` | `UPAVASA` after the day before's ekabhakta; end unsaid | HBV 14.457, 14.462's commentary |
| `GAURA_PURNIMA` | to the moonrise; kind unsourced | a published Gaudiya calendar (C1123) |
| `RADHASHTAMI_VAISHNAVA` | `EKABHAKTA`: "till noon" read as the one meal at midday; start and end unsourced | a published Gaudiya calendar (C1123) |

The named ekadashis take DS's Ekadashi because their rule is kept on an 11th,
which a test checks against the rule rather than the name. `NIRJALA_EKADASHI`
is left out (C1080). `EKADASHI_GAUDIYA` and the `GAUDIYA` pack's festivals
carry *Hari Bhakti Vilasa*'s (HBV, Haridas Shastri's edition, vol. 2), read
for SR-17, and Gaura Purnima's and Radhashtami's the published Gaudiya
calendar's: its "till noon" is read as the one meal at midday that
`EKABHAKTA` names, and its "till dusk" and "till midnight" no `end` names
(C1123). The Karaka chaturthi
carries none: DS p. 76 and NS p. 146 give its day and not its fast (C1081).
Every other key's `unsourced` names `fast`, which does not say it has none.

**What moved.** A rule's `meta` gains `fast` only on the keys above, so only
their packs' `input_hash` moves; `unsourced` gains `fast` on every row
without one. No day moved.


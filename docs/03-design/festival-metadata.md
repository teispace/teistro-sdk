# Festival metadata

Status: `built`, 2026-10-30. SR-02. Crux C850
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

Communities and regions are open because a consumer's own rules name their
own, which a closed list would refuse. A rule the consumer wrote has an empty
`meta` until it sets one, and serialises without the field, so a rule without
values hashes as it did.

## 2. Where each value comes from

Nothing is inferred from a festival's name. In order:

**`GAZETTED`.** A government's notice lists the day as a public holiday.

- Nepal: the Ministry of Home Affairs' lists for 2082 and 2083 (Nepal Gazette,
  relayed in Pradhan & Associates' alerts of those lists). 48 keys. The
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
what the notice names, and `IN` for India's pack.

**Unset.** Of the 359 keys of Nepal's packs, 48 are `GAZETTED`, 60
`REGIONAL_ETHNIC`, 44 `RELIGIOUS_CULTURAL`, 32 `OBSERVANCE` and 175 have no kind.
302 have no tradition. They are a fitted day of a mela, a fast or a jatra whose
source is the committee's print, which says when and not what. No value is
guessed for them, and the classification's blank is not one either: its own
words are "kept across communities, or the community is not known here". The
town in a key such as `BHAKTAPUR_BRAHMAYANI_JATRA` is not read as a region, since
that would be inferring from a name. C850 holds what would close them.

`*Dharmasindhu*`'s packs (`DHARMASINDHU`, `PITRI_PAKSHA`) and `GAUDIYA` are
`RELIGIOUS_CULTURAL` and `HINDU` by their text, and name no region.

## 3. Reading a pack

`almanac.festivalPack`, `ts_almanac_festival_pack`, answers every row of a pack:
its key, form, source, `meta`, and `unsourced`, the fields no source gives by
name. That list is how a rule "says so": a reader cannot take a blank for an
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
change. No day moved. The table is 21 groups and 359 sorted keys, searched by
bisection, so stamping a pack does not scan.

Names: `festival_kind` and `tradition` are named in `en-Latn` and `ne-Deva-NP`;
the Nepali strings await the reviewer of Q42. Communities and regions are keys
and have no names yet.

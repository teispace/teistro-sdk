# The state readings: a reading for what a chart *is*, not only for what it triggers

Status: `design`, 2026-09-21. The measurements in §2 were taken from the
baseline engine's built corpus, this repository's catalogue and its shipped
rule packs on that date; §6 says which of them become a generated page so
they cannot go stale here.

Related:
[`interpretation-records.md`](interpretation-records.md) (the rule
readings, the same mechanism pointed at a rule),
[`interpret-composers.md`](interpret-composers.md) (the composers that say
one), [`intl-engine-and-packs.md`](intl-engine-and-packs.md) (what a pack
is and how one is loaded), ADR-0010 (the shipped packs are embedded).

## 1. Purpose and scope

A rule reading answers *this yoga is present, and here is what it means*.
It is silent about everything a chart is without triggering a rule: Jupiter
is in the first house, the Moon is in Ashwini, the lagna is Aries, the
tithi is Shukla Pratipada. The baseline engine has a reading for each of
those too, in the same four languages, and
[`interpretation-records.md`](interpretation-records.md) §8 deferred them
to a page of their own.

This is that page. In scope: what the corpus is, where its keys land in
this catalogue, what a record becomes when two corpora describe the same
subject, the kind the one composite key wants, the gates, and the order of
work.

Out of scope: the nine milan files, which are Phase 8's `matching` and have
no subject here yet; and the composer's own message shapes, which §5 sets
the constraint for and `interpret-composers.md` owns.

## 2. What the research found

Measured from the corpus as the engine builds it, against this
repository's `catalogue/catalogue.json` and its shipped rule packs.

**It is one export, not a dozen files.** The engine exports a single
`STATE_INTERPRETATIONS` of **38 categories** and **554 records**, in the
same `{sa, ne, en, hi} × {summary, full, effects}` shape the rule readings
use — so `migrate::ReadingRow` reads it unchanged and the importer already
written needs a mapping and nothing else.

**It is complete.** 2 216 language-records, four per record, and **not one
is missing a summary or a passage**. 481 of them carry named effect facets,
from a set of five (`career`, `finance`, `general`, `health`,
`relationships`) — a different five from the rule readings' nine, which is
the point of the facets being an open set rather than a record's fields.

**Most of it needs no new key, and the spellings agree.**
[`interpretation-records.md`](interpretation-records.md) §8 guessed that
the spellings would differ and named `ASHWINI` against a catalogue
`ASHWINA`. Measured, that was wrong: the catalogue spells it `ASHWINI` too.

| | categories | records | what they need |
|---|---:|---:|---|
| key an existing kind exactly | 21 | 233 | nothing |
| name a shipped rule | 1 | 62 | nothing |
| key an existing kind by prefix | 1 | 12 | twelve written aliases |
| a graha and a house together | 1 | 108 | one open kind |
| no subject in the SDK yet | 14 | 139 | a decision each |

The 21 are `avastha-baladi`, `avastha-jagradadi`, `avastha-deeptadi`,
`avastha-lajjitadi`, `gana`, `nadi`, `yoni`, `varna`, `tatwa`,
`special-lagna`, `vara-phala`, `tithi-phala`, `nakshatra-phala`,
`yoga-phala`, `namakarana-nakshatra`, `ishta-devata`, `graha-direction`,
`graha-color`, `mantra-ritual`, `dasha-lord-effect` and
`dasha-lord-activation`. **216 of their 217 keys are the catalogue's own,
character for character**; the one exception is `VISHKAMBA` for
`yoga.VISHKAMBHA`, which the catalogue already carries as an alias. Nothing
is inferred to get there and nothing needs to be.

**`dosha-timing` closes the rule readings' own gap exactly.**
[`interpretation-records-measured.md`](interpretation-records-measured.md)
lists 18 shipped rules with no reading and 10 readings naming no shipped
rule. Of `dosha-timing`'s 62 keys, **52 name a shipped rule — including all
18** — and the remaining 10 are `ANSHIK_MANGAL`, `PURNA_MANGAL`,
`BHAKOOT_DOSHA`, `GANA_DOSHA`, `NADI_DOSHA`, `VEDHA_DOSHA`, `RAJJU_DOSHA`,
`MAHENDRA_ABSENCE`, `STREE_DEERGH_ABSENCE` and `KUJA_BHANGA_ABSENT`: the
same ten, the milan rules waiting on Phase 8. The two corpora are one
corpus split by subject.

What that closes is worth stating exactly, because it is nearly a
stronger claim than it is. Every shipped rule ends with text in four
languages and none is silent — but the 18 gain a **`timing`** and not a
reading. A timing says when a dosha acts, not what it means, and calling
it a reading would be a claim the corpus does not make. The measured page
prints the three numbers apart for that reason.

**`lagna-rashi` is a prefix and `graha-bhava` is a pair.** `LAGNA_ARIES`
is `rashi.ARIES` behind a prefix, and stripping it would be exactly the
inference §4 of the rule-readings page forbids — a rule that worked for
twelve keys and would quietly mis-file the thirteenth. Twelve written
aliases cost twelve lines and cannot mis-file anything.
`graha-bhava`'s 108 keys are `JUP_IN_1` … `KETU_IN_12`: nine grahas by
abbreviation against twelve houses, and the only key in the whole corpus
that names two subjects at once.

## 3. What a record becomes: forms, laid over

A record here is an entity record, as a rule reading is: `name` the
summary, `prose` the passage, one form per effect facet.

**The form name is the category's contribution, and that is what makes the
corpus fit.** 38 of its keys appear in more than one category —
`nakshatra-phala` and `namakarana-nakshatra` both key onto
`nakshatra.ASHWINI`, and `dosha-timing` keys onto rules that already carry
a reading. They are not duplicates and neither is wrong: one says what the
nakshatra portends, the other what to name a child born under it. Each
becomes its own form on one record, so `nakshatra.ASHWINI` ends with
`name`, `iast`, `phala` and `namakarana`, and `rule.MANGAL_DOSHA` gains a
`timing` beside the reading it already has.

That only works if a record **merges** rather than replaces, and it did
not. Both places that put a record where one already stood replaced it
whole:

- `migrate::push_record`, so a corpus read second erased the one read
  first;
- `Intl::load_pack`, so a pack giving every nakshatra a `phala` would have
  left each record with a `phala` **and nothing else** — no name, no
  transliteration, no glyph.

The second is the more serious, because it is a consumer's own extension
point: a consumer loading a pack of its own corrections or additions is
the supported way to change what the SDK says, and it silently deleted
what it did not carry. One rule now decides both, in one function
(`Entity::overlaid`): **a form the file carries is the file's, a form only
the standing record carries is kept, and the gender and the glyph are the
file's where it has one.** A message entry still replaces, because a
message is one string and has nothing to merge.

`Loaded` and `ts_intl_loaded` report the two outcomes apart — `replaced`
for an entry that kept nothing of what stood, `merged` for a record that
kept something — so what was applied is a number a consumer can read, in
Rust and in all three bindings. The `merged` count took the `reserved`
word the boundary struct already held, so no ABI grew.

**A consumer that does mean to replace a record whole** ships every form it
wants, which is what building a pack from a source tree of its own already
does. This page declares that as the escape hatch rather than adding a
policy flag: whole replacement has no consumer to measure, and one
semantic that is right is worth more than two that must both be explained.

## 4. Where the keys land

Nothing is inferred from resemblance. Every mapping is a written table,
refused both ways by the gate.

| the engine's category | this catalogue | the form |
|---|---|---|
| the 21 above | the kind of the same subject | the category's own name, camel-cased |
| `dosha-timing` | `rule.<KEY>`, the open kind | `timing` |
| `lagna-rashi` | `rashi.<SIGN>`, by twelve written aliases | `lagnaPhala` |
| `graha-bhava` | `graha_bhava.<GRAHA>_IN_<house>` | `name`, `prose`, facets |

`graha_bhava` is the catalogue's **second open kind**. It is open for the
same reason `rule` is — the catalogue holds the kind and not the members —
but for a different cause: `rule`'s members arrive in a consumer's rule
packs, and `graha_bhava`'s are a product of two closed kinds that the
catalogue would have to enumerate to hold. 108 rows of generated table
would buy a `resolve` that the migration's own alias table already
performs, so the key is held to being well formed and the packs decide the
rest, exactly as a rule key is.

Adding it made the open kinds **data rather than code**: they were a
`const OPEN_KINDS` inside the generator, so a kind that has no members
was declared somewhere no contributor reading `catalogue/` would look.
An open kind is now a `catalogue/<kind>.yaml` with `open: true` and no
members, beside every closed kind's file, and the generator reads one list.

The nine graha abbreviations (`JUP` → `JUPITER`, `VEN` → `VENUS`, `MER` →
`MERCURY`, `SAT` → `SATURN`, and five that are already the catalogue's
spelling) are a written table too, not a prefix rule.

## 5. What reads one

A composer, as `readings` reads a rule's reading: it asks the **base**
locale whether a reading exists before choosing what to say, so the plan's
shape is a fact of the build and not of the reader
(`interpret-composers.md` §5).

`Vocabulary` was that question, spelled for one subject
(`has_reading(rule)`). A second subject makes it a general one: the trait
asks about a **catalogue key**, and `has_reading` becomes the spelling
that names a rule. A composer that wants to know whether Jupiter in the
first house has a reading asks the same trait the same way.

The composer itself is `phala`, a ninth member of `PlanRequest`, off unless
asked for — because a plan that grew by 108 items the moment a readings
pack was loaded would change every consumer's page without being asked.

## 6. The measurement, and the gates

`cargo xtask state-readings` writes
[`state-readings-measured.md`](state-readings-measured.md) and
`check-state-readings` regenerates it in memory and fails on any
difference. It measures, over the migrated packs rather than over the
engine:

- every category that was mapped, its kind, its form and its record count;
- every category that was **not** mapped, by name and with the reason —
  a count alone is a claim that goes stale, and this list is short enough
  to be exhaustive and must fail in both directions;
- the keys that met a record already standing, and what each kept, which
  is the overlay in §3 measured rather than asserted;
- the bytes each locale's source and pack take, beside the rule readings',
  because the two are loaded together;
- that every locale's pack loads into an engine that never read the source
  tree, and that every record answers from it afterwards.

`check-interpretations` keeps its own page, and the claim it could not make
before — *every shipped rule carries a reading* — becomes one it can.

## 7. Order of work

1. **The overlay**, in `Entity`, `Namespace::overlay`, `load_pack` and
   `push_record`, with the `merged` count at the boundary. **Built**:
   nothing else in this page can be migrated until a second corpus can
   describe a subject the first already describes.
2. **The exporter**, in the baseline engine's repository beside its
   `export-*-vectors.mjs`: one JSON document, all 38 categories, every
   language, the facets as they are. **Built**
   (`export-state-readings.mjs`): 38 categories, 554 records, 5 facets.
3. **`teistro-intl migrate states`**, with the category table of §4, the
   twelve lagna aliases and the nine graha abbreviations, reporting every
   category it did not map. **Built**: 24 categories, 415 readings into
   340 records in each of four locales, 0 unknown keys, 14 categories
   reported unmapped.

   Migrating an **overlay** root found three things the loader and the
   validator had only ever seen complete roots do. A record was
   recognised by having a `name`, which an overlay record has not; a
   record was required to have one, which is a rule about a locale that
   declares itself complete and not about a root; and a form name was
   held to lowercase letters, which `phalaProse` and `ishtaDevata` are
   not. A record is now recognised by its shape — an object whose values
   are all text, where a group's are objects — a name is owed by a locale
   at `strict` completeness, and a form name is a `camelCase` word.
4. **The second open kind**, as a `catalogue/` file, with the generator
   reading one list. **Built**: `catalogue/rule.yaml` and
   `catalogue/graha_bhava.yaml`. A kind costs the bindings nothing — only
   a **closed** kind's members become an enum at the boundary, and an open
   kind has none — so the second one is a file and a number.
5. **The measured page and `check-state-readings`.** **Built**.
6. **The `phala` composer**, its messages, its `PlanRequest` member and a
   worked example.

## 8. What this design does not settle

- **The 14 categories with no subject here yet**, 139 records: the eight
  `ayurdaya-*` families (41), `muhurta-factor` (47),
  `shadbala-strength` (28), `planet-condition` (8), `sade-sati-phala` (5),
  `auspicious-kaal` (5) and `inauspicious-kaal` (5). Three shapes among
  them, and each is a decision rather than a task. `shadbala-strength` and
  `muhurta-factor` are composite keys like `graha-bhava`, so they would
  take the same treatment once something says them. `planet-condition`
  splits across two kinds this catalogue already has — three of its eight
  keys are `state` members, four are `dignity` members, and
  `COMBUST_CANCELLED` is neither. The `ayurdaya-*` and `-kaal` families key
  onto subjects the SDK has not modelled at all. None of them blocks the
  other 24, and the gate lists them by name so the list cannot rot into
  prose.
- **Native review.** The corpus is the baseline engine's own Nepali,
  Sanskrit and Hindi, not a machine translation, and it has not been
  reviewed here. It joins the roadmap's `ne`/`hi` sign-off.
- **Whether a category should become a `form` or a record of its own.**
  This page chooses the form, because 38 keys are shared between
  categories and a record per category would split one subject across
  several. A category whose reading is long enough to want its own record
  is a case that has not appeared.
- **Where the built packs are published**, which is Phase 9's release
  pipeline and is settled for the rule readings in the same place
  ([`interpretation-records.md`](interpretation-records.md) §8).

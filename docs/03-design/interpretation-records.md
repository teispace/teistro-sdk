# The interpretation records: a reading for a rule, in four languages

Status: `design`, 2026-09-21. The measurements in §2 were taken from the
baseline engine's built corpus and this repository's shipped rule packs on
that date; §6 says which of them become a generated page so they cannot go
stale here.

Related: [`interpret-composers.md`](interpret-composers.md) (the composers
that would say a reading), [`rules-engine.md`](rules-engine.md) (what a rule
is), [`intl-engine-and-packs.md`](intl-engine-and-packs.md) (what a pack is
and how one is loaded), ADR-0010 (the packs are embedded).

## 1. Purpose and scope

`readings` composes what the rules answered, and for the rule's own
statement it emits `sdk.reading.effect` carrying **the words the rule
cites, untranslated**. The measured page counts the seam: 2 449 of 19 061
items print a translator's English inside an otherwise Nepali reading, and
`hi-Deva-IN`, `sa-Deva` and `sa-Latn` carry no message at all, only
`sdk.entity`.

The baseline engine has a reading for nearly every rule the SDK ships, in
Sanskrit, Nepali, English and Hindi. This page decides **what those records
become here**: where they live, what shape they take, what reads them, and
what is gated.

In scope: the record's shape and home, the key mapping, the composer's
choice between a general message and a rule's own reading, the gates, and
the order of work.

Out of scope: the other interpretation corpora the same package carries —
graha-in-bhava cells, panchanga phala, dasha phala, shadbala strength,
avastha, planetary condition, sade sati, ayurdaya, avakhada, namakarana,
muhurta and the nine milan files. They are the same mechanism pointed at a
different subject, and §8 says why each waits.

## 2. What the research found

Measured from the corpus as the engine builds it, not from its sources, so
the numbers are what a consumer would get.

**The corpus is complete in four languages.** 649 records — 605 yoga, 44
dosha — and every one carries `summary`, `full` and `effects` in all four:
2 596 of each, which is 649 × 4 exactly. **Not one record is missing a
language and not one string is empty.** That is worth stating plainly,
because it is the fact that makes this worth doing: `hi` and `sa` go from
carrying no message at all to carrying a reading for almost every rule.

**It very nearly matches what the SDK ships.** Against the 657 rules of the
`yogas` and `doshas` packs:

| | |
|---|---|
| rules with a reading | **639** (97%) |
| rules without | 18, all in `doshas` |
| readings naming no rule | 10 |

The 18 are named and the list is short enough to be exhaustive:
`KALSARPA_ANANT`, `KALSARPA_KULIK`, `KALSARPA_VASUKI`,
`KALSARPA_SHANKHPAL`, `KALSARPA_PADMA`, `KALSARPA_MAHAPADMA`,
`KALSARPA_TAKSHAK`, `KALSARPA_KARKOTAK`, `KALSARPA_SHANKHACHUD`,
`KALSARPA_GHATAK`, `KALSARPA_VISHDHAR`, `KALSARPA_SHESHNAG`,
`KALA_AMRITA_YOGA`, `VISH_YOGA_DOSHA`, `CHANDRA_DOSHA`, `SHRAPIT_DOSHA`,
`ANGARAK_DOSHA`, `CHANDAL_DOSHA`.

Twelve of those eighteen are the Kalsarpa family, and the corpus **does**
carry a `KALSARPA` record; it carries `VISH` and `GURU_CHANDAL` too. §4
decides what to do about that, and the answer is *nothing automatic*.

The 10 that name no rule are not errors. Every one is a marriage-compatibility
key — `NADI_DOSHA`, `BHAKOOT_DOSHA`, `GANA_DOSHA`, `VEDHA_DOSHA`,
`RAJJU_DOSHA`, `STREE_DEERGH_ABSENCE`, `MAHENDRA_ABSENCE`, `ANSHIK_MANGAL`,
`PURNA_MANGAL`, `KUJA_BHANGA_ABSENT` — so they are readings waiting for
Phase 8's `matching`, not readings pointing at nothing.

**`effects` is an open set of named facets, not a fixed record.** Across
2 596 language-records: `general` 1 312, `mind` 1 048, `career` 655,
`spirituality` 583, `relationships` 420, `material` 357, `wealth` 128,
`body` 31, `longevity` 27. A record carries the facets its reading has and
no others, so the shape is a map and not a struct, and a schema that fixed
the nine would be wrong on the next corpus.

**And it is large.** As JSON in the pack's own shape:

| locale | records |
|---|---|
| `sa` | 623 KB |
| `ne` | 744 KB |
| `en` | 450 KB |
| `hi` | 788 KB |
| **four** | **2.54 MB** |

Against `i18n/` today, which is **360 KB for all five locales together**.
The corpus is seven times the entire pack set.

## 3. The decision the size forces: a loadable pack, not a bundle

`crates/sdk/build.rs` compiles every locale in `i18n/` into the binary, so
a consumer needs no files to render a message (ADR-0010). That is right for
the engine's own messages — a few hundred short strings — and **wrong for
an interpretation corpus**. Putting 2.54 MB there would spend it on every
artefact the SDK produces: the Rust library, the C dynamic library, the
Node, Dart and Python packages, and wasm, where the quality bar allows a
pull request to grow a profile's gzipped module by 2% without a note
(`05-testing/01-quality-bar.md`). A consumer computing a Julian day would
carry every Nepali yoga reading to do it.

So the records ship as a **pack that is loaded, not embedded**. The
mechanism already exists and is already at the boundary:
`ts_intl_load_pack`, and `Intl` resolves a loaded pack ahead of the
bundled one. Nothing new is needed to read them; what is new is where the
file comes from and how it is built.

This keeps the promise ADR-0010 actually made — *the engine renders with no
files* — while letting the corpus be opt-in, versioned separately, and
translated without rebuilding the SDK.

## 4. What a record becomes

A reading has **no slots**. It is static prose per key per language with
named parts, which is precisely what an `Entity` already is: a key, and per
locale a `name` plus arbitrary `forms`. So a record becomes an entity of the
catalogue's `rule` kind — `rule.RUCHAKA` with forms `summary`, `full` and
one per facet — and not a message.

**Not a message, and the reason is measurable.** The generator writes a
struct and a `TypedMessage` impl per message: 70 messages are 48 KB of
`crates/intl/src/messages.rs` and 134 KB of
`bindings/python/teistro/messages.py`. 649 records × 3 fields would be
about 1 950 messages — roughly thirty times that, in four generated
surfaces, for text that takes no parameters. An entity is looked up, not
generated.

**One thing blocked it, and it was a real decision rather than a bug.**
`Kind::Rule` is the catalogue's **only open kind** — its members arrive at
runtime, because a consumer ships its own rule packs and the catalogue
cannot know their keys. `catalogue::resolve` therefore has no arm for it,
so `key::resolve("rule.RUCHAKA")` fails. It refused in **two** places, and
building found the second: the validator says "not a catalogue key", and
before it ever gets there the *loader* refuses the key segment as "neither
camelCase nor a catalogue key".

The fix is not to make the catalogue pretend to know rule keys. It is that
**an open kind's key is well-formed rather than catalogued**, and the
authority for which rule keys exist is the *rule packs*, not the catalogue.
So:

- the loader and the validator accept a well-formed key of an open kind,
  through **one** predicate (`source::is_member_key`) rather than two;
- a gate holds the readings against the shipped rule packs **both ways**,
  which is a stronger check than `resolve` could give — it is the
  `KEYS`-against-the-packs pattern the composers already use.

**And "well formed" is not "screaming snake case", which the corpus said
rather than this page guessing.** Both the loader and this page's first
draft assumed upper case and digits. Five of the 657 shipped rules carry a
karaka's own abbreviation mid-key — `JAIMINI_AK_AmK_BOTH_OWN_EXALT`,
`JAIMINI_AK_AmK_KENDRA_TOGETHER`, `JAIMINI_AK_AmK_PARIVARTANA_PROXY`,
`JAIMINI_AK_AmK_TRIKONA_TOGETHER`, `JAIMINI_AmK_10` — and they are the only
five that are not all upper case. The first migration refused exactly those
five readings, which is the check working: the assumption was wrong, the
corpus said so on the first run, and one predicate now holds both ends.

**No key is mapped by resemblance.** `KALSARPA_ANANT` does not read the
`KALSARPA` record, and `CHANDAL_DOSHA` does not read `GURU_CHANDAL`, however
obviously related they look. A rule's reading is a claim about that rule;
inferring it from a shared prefix is how a product whose worth is fidelity
starts saying things no one wrote. The 18 stay uncovered, the gate lists
them, and a maintainer who wants the family reading writes the record.

## 5. What reads one

`readings` emits `sdk.reading.effect` with the rule's cited text as a slot.
With a corpus loaded it chooses: **the locale's own reading where there is
one, the cited text where there is not.** **Built.**

That choice is the one §5 of `interpret-composers.md` reserved — *where a
composer chooses between a general key and a specific one it asks the base
locale, not the reader's* — and the trait it said would "arrive with the
composer that chooses" arrived here: `Vocabulary`, with one question,
`has_reading(rule)`. It is a trait rather than an `&Intl` so a composer is
testable without a locale engine and so the question is visible in the
signature; `NoReadings` is the named answer for a consumer that has loaded
no pack, because a silent default would be the composer making the
consumer's choice.

**A reading is said for the rule, not for a statement.** A rule citing three
statements gets one reading, not three copies of the same passage — and a
rule citing **none** says its reading too. That last case is the one the
first draft missed and the corpus corrected: of the kernel's shipped rules,
every single one that has a reading states no effect of its own. They were
the silent rules, and a written reading is exactly what they lacked.

**It says the summary, not the passage.** `sdk.reading.says` renders the
record's `name` form. A plan item is a sentence — it sits beside "Moon
takes part" and "in force" — and some of the corpus's `full` passages run
to several hundred words, a few of them describing the *rule* rather than
the native. So the plan says the sentence and the record keeps the essay: a
consumer wanting the passage or a facet reads the entity directly, which is
one call on an engine it already has.

**How much this closes is a measurement, and a modest one.** The readings
were written against the recording engine's rule keys; the kernel ships
packs written independently. They are not two spellings of one set:
dropping the kernel's leading segment matches 44 of its 263 nabhasas and
**none** of its 73 arishtas. 12 of the 365 rules the measured pass composes
carry a reading, and they produce 229 items. The gap is reported rather
than closed with a guess — see §4 — and closing it means either the shipped
packs adopting the engine's keys or readings written for the kernel's.

## 6. The measurement, and the gates

A generated page, `interpretation-records-measured.md`, held by
`check-interpretations`, over the shipped packs and the loaded corpus:

- how many shipped rules have a reading, and **the exhaustive list of those
  that do not**, which fails both ways: a rule that gains a reading and a
  reading that loses its rule both change the page;
- the readings that name no shipped rule, each with its reason, so "waiting
  for `matching`" is recorded rather than remembered;
- the facets the corpus uses and how often, because a fixed nine would be a
  claim about the next corpus;
- the bytes per locale, so the pack's cost is a number on a page rather
  than a surprise in a package;
- that every record renders in its own locale with no fallback and nothing
  to warn about — the same check the composers are held to.

## 7. Order of work

1. **The exporter**, in the baseline engine's repository beside its
   `export-*-vectors.mjs`: one JSON document, every record, every language,
   the facets as they are. It is untracked there, as the others are.
   **Built** (`export-rule-readings.mjs`): 649 readings, 9 facets.
2. **`teistro-intl migrate readings`**, mapping the document onto entity
   records of the `rule` kind, the packs staying the authority: a key that
   is not well-formed is reported and skipped, and nothing is inferred.
   **Built**: 649 records into each of four locales, 0 errors.
3. **The open-kind rule in the loader and the validator**, and the gate that
   holds the readings against the rule packs both ways. **Built**:
   `check-interpretations` over
   [`interpretation-records-measured.md`](interpretation-records-measured.md).

   Migrating a second corpus also found a **latent defect in the first**.
   `new_locale_meta` gave every locale it created a fallback to the base
   locale and `base` completeness — including the base locale itself, which
   the validator refuses ("a locale cannot fall back to itself"). It had
   never fired, because `i18n/en-Latn/_meta.json` already existed and was
   kept; a root created from nothing is what ran that branch. A base locale
   falls back to nothing, is complete by definition, and keeps Latin digits.
4. **The pack as an artefact**: built from the migrated records, versioned,
   and loaded rather than embedded.
5. **The composer's choice**, with the trait that asks the base locale.
   **Built**: `Vocabulary`, `NoReadings`, `sdk.reading.says`, and the
   renderer taught the same open-kind rule the loader and the validator
   learned — it resolved the key a third time and would have warned on
   every reading.
6. **The measured page and `check-interpretations`.**
7. **The boundary and the bindings**, so a reading crosses as the rest does.

## 8. What this design does not settle

- **The other corpora.** Graha-in-bhava (108 cells with condition modifiers
  and conjunction synthesis), panchanga phala, dasha phala, shadbala
  strength, avastha, planetary condition, sade sati, ayurdaya, avakhada,
  namakarana, muhurta and the nine milan files are the same mechanism
  pointed elsewhere. Each waits on the composer that would say it —
  graha-in-bhava on `placements` becoming an interpretation, milan on
  Phase 8's `matching` — and none waits on this page.
- **Whether a family reading may stand for its variants.** §4 refuses to
  infer one. Writing twelve Kalsarpa records, or giving a rule an explicit
  `reads:` pointer to another rule's record, are both defensible and both
  are somebody's decision rather than a migration's.
- **Native review.** The corpus is the baseline engine's own Nepali and
  Hindi, not a machine translation, but it has not been reviewed here. It
  joins `sdk.reading`, `sdk.aspect`, `sdk.condition` and `sdk.karaka` in
  the roadmap's `ne`/`hi` sign-off.
- **The pack's distribution.** A file beside the binding, a separate
  package per language, or a download — a packaging decision that wants
  the binding surface settled first.

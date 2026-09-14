# Entity names for what a document carries

Status: `built`, written 2026-09-14. It derives from
[`intl-engine-and-packs.md`](intl-engine-and-packs.md) (the packs and
their completeness) and from
[`document-schema.md`](document-schema.md), whose schema is what makes
the scope below computable. The rule it keeps is
`02-architecture/03-localization-architecture.md`'s: *no
machine-translated stubs, because a plausible wrong astrological term is
worse than a visible fallback*.

## 1. The gap, measured

A chart document carries catalogue keys: a graha, a dignity, a point, a
house system, an ayanamsha. A reader renders each through
`intl.entity(key).name`. The locale engine reports catalogue coverage but
gates only a locale's completeness against the **base locale's** keys,
so a kind nobody had named was complete by definition.

Measured over the 28 catalogue kinds the document schema names, and the
five locales the SDK ships:

| kind | members | named before |
|---|---:|---:|
| `ayanamsha` | 47 | 0 |
| `point` | 49 | 1 (the lagna) |
| `house_system` | 22 | 0 |
| `varga` | 21 | 0 |
| `masa` | 12 | 0 |
| `avastha_deeptadi` | 9 | 0 |
| `direction` | 8 | 0 |
| `chart_kind` | 7 | 0 |
| `avastha_lajjitadi` | 6 | 0 |
| `calendar` | 6 | 0 |
| `avastha_jagradadi` | 3 | 0 |
| `graha` | 12 | 9 (the outer three missing) |

That is 192 members with no name in either strict locale (`en-Latn`,
`ne-Deva-NP`), and nine more (`era`) missing only from the two
base-completeness Devanagari locales. The `chart_reading` example could print a Gulika's sign
and not the word *Gulika*.

## 2. Where names may come from

The clean-room policy ranks sources, and a name is data like any other:

- **Rank 2, the baseline engine**, keeps four-language name tables that
  a production Nepali app has shown to readers: English, Nepali, Hindi,
  and Sanskrit with its IAST. They come with a stated naming policy:
  - Indian systems keep their traditional name (श्रीपति, राशि-भाव);
  - personal names are transliterated, not translated (प्लासिडस);
  - descriptive systems use the established term (सम भाव);
  - alphanumeric designators stay Latin (MC, APC).

  The policy allows rule data from it to be taken.
- **Not a translation of mine.** A Devanagari spelling of a Western
  proper name, or a Nepali gloss for a chart kind, is exactly the
  plausible stub the policy refuses.

So the import takes **every name the baseline engine vets, and nothing
else**. It is read by loading the source modules, not by transcribing
them, so no Devanagari passes through a keyboard:

| kind | source table | members |
|---|---|---:|
| `ayanamsha` | `AYANAMSA_DEFINITIONS[].names` | 47 |
| `house_system` | `HOUSE_SYSTEM_NAMES` | 22 |
| `masa` | `HINDU_MONTH_NAMES` | 12 |
| `avastha_deeptadi` | `DEEPTADI_NAMES` | 9 |
| `direction` | `DISHA_NAMES` | 8 |
| `point` | the upagraha service's table | 7 |
| `avastha_lajjitadi` | `LAJJITADI_NAMES` | 6 |
| `avastha_jagradadi` | `JAGRADADI_NAMES` | 3 |

That is 114 of the 192. Keys match the catalogue's exactly except for three
spellings, and each is a stated rename rather than a fuzzy match:

- `ASHVIN` is `masa.ASHWINA` (the same difference the conformance corpus
  has);
- `PARIVESH` is `point.PARIVESHA`;
- the directions' kebab case (`south-east`) is the catalogue's
  `SOUTHEAST`.

## 3. The record each name becomes

The forms follow the kinds already named, so a consumer cannot tell an
imported record from an authored one:

- **`name` and `prose`** are the language's primary name.
- **`iast`** is the Sanskrit transliteration **where the source gives
  one**, in every Devanagari locale and in English. It is never computed:
  the transliterator writes a Western name in Devanagari as nonsense, and
  a form that is absent is better than a wrong one.
- **No `short`, `glyph` or `gender`.** The source vets none of them, and
  `name` stands in for a missing form, as the engine already allows.
- **`sa-Latn` is not written by hand.** `gen intl` derives it from
  `sa-Deva`, as it does for every other kind, and building showed where
  that is wrong. Transliterating a Western name that was itself
  transliterated into Devanagari yields nonsense (`प्लासिडस` becomes
  `Plāsiḍasa`, `फेगन-ब्रैडली` becomes `Phegana-braiḍalī`), while the
  descriptive Sanskrit names come out right (`Sampūrṇa Rāśi`,
  `Ākāśagaṅgā Kendra 0° Dhanu`). So the 35 names that contain a
  non-Sanskrit proper noun (14 house systems, 21 ayanamshas) are listed
  in `i18n/sa-Latn/_overrides.json`, the mechanism the localisation
  standard keeps for "hand overrides where the mechanical result is
  wrong". Each override is the **source's own English** for that key, so
  the list is a classification and not a translation.

**A form's absence must not change a type.** The binding gates found
this the first time they ran. The generated accessors typed a form as
always present when *every* base-locale record happened to carry it. Before
the import that was true of `iast`, so Dart's `entity.iast` was a
`String` and Node's `entityForms` gave it `''`. The Western ayanamshas and
house systems rightly carry no `iast`, so the inferred set shrank, Dart's
form became nullable and an example that pads it stopped compiling. The
data was right and the inference was the defect. So
`generate::GUARANTEED_FORMS` now declares `name`, `prose` and `iast` as
the forms every binding types as present (empty when a record lacks one),
which is what the bindings' own tests already asserted.

Where the source's English is a gloss (`Awake` for जाग्रत्), the record
keeps the gloss as `name` and the Sanskrit as `iast`, which is what
`avastha_baladi` already does (`Infant`, `Bāla`).

## 4. The gate: a kind a document carries is named, or listed

`check-intl` gains a rule. Each **strict** locale names every member of
every catalogue kind the document schema names, **except the members
listed** in `xtask/src/intl.rs` as unnamed, each kind with the reason
it has no vetted source.

The list fails both ways, as the project's other lists do:

- an unnamed member not on the list fails, so a new kind a document
  starts carrying cannot arrive silently unnamed;
- a named member still on the list fails, so the list only ever shrinks.

The kinds come from the schema, not a second list. Each catalogue key's
schema carries `x-teistro-kind`, which is also what lets a consumer's
tooling map a string back to its catalogue kind.

Left on the list after this step, 78 members, each for want of a vetted
source rather than for want of trying: the other 41 points (the special
lagnas, the arudhas, the sphutas and the yogi points), the 21 divisional
charts, the 7 chart kinds, the 6 calendars and the 3 outer planets.
`era` in the two base-completeness locales (`hi-Deva-IN`, `sa-Deva`) is
outside the rule, which covers strict locales.

## 5. What this does not settle

- **Names for the listed members.** Each needs a rank-1 or rank-2
  source, or a native reviewer's sign-off recorded with the change. The
  list is where that work is tracked.
- **`short` forms and genders** for the imported kinds, which the
  source does not vet.

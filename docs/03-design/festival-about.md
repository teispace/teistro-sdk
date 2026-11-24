# Festival descriptions (SR-153)

Status: `building`, 2026-11-24. Stage (a): forty-two subjects in English and
Nepali; stage (b): the rest of the catalogue, and Hindi and Sanskrit.
Cruxes C2410 to C2417.

Every festival has a name in each locale but no words about it. The
consumer's brief (addendum 13 §2, tithipatra hub,
`docs/04-architecture/sdk-briefs/teistro-teimeris-brief-addendum-13.md`)
asks for three things on each:

- **About:** two to four sentences on what it is and why it is kept.
- **Observance:** what people do on it, as a short list.
- **Who and where:** the communities and regions that keep it, as the
  catalogue already knows them.

These are in the reading register of the rashifal's voice
([`rashifal-reading.md`](rashifal-reading.md)), optional, and carried in
the packs. Its decision D-324 holds here too: every domain word lives in
teistro, and the wording loads as data.

## 1. Facts in the SDK, words in the packs

The voice's rule is kept. A source decides each **fact**: whom the day
honours, the story it recalls, what is done on it. A pack writes the
**words**. So the SDK holds, for each subject, the list of its observances
as keys, and the sources they come from. Each locale words the same list.
A Nepali page and an English one then say the same acts, and a locale that
misses one is caught by a check rather than by a reader.

The `about` sentences are a pack's own paraphrase of the cited sources. No
translation still in copyright is quoted. Each sentence says only what a
cited source says (C2410).

## 2. Subjects

A subject is a rule's key (`INDRA_JATRA`) or a group's key (`TIHAR`,
[`festival-groups.md`](festival-groups.md)). A group's text says what the
festival is as a whole. Its members' texts say what their own day adds.

Stage (a) has forty-two subjects, chosen as the consumer asked: Nepal's
national holidays first, then the days its calendars print most.

| group | subjects |
|---|---|
| Dashain | `DASHAIN`, `NAVARATRA_ARAMBHA`, `PHULPATI`, `MAHA_ASHTAMI`, `VIJAYA_DASHAMI` |
| Tihar | `TIHAR`, `KUKUR_TIHAR`, `LAKSHMI_PUJA`, `GOVARDHAN_PUJA`, `MHA_PUJA`, `YAMA_DWITIYA` |
| Teej | `TEEJ`, `HARITALIKA`, `RISHI_PANCHAMI` |
| Chhath | `CHHATH` (the group and its main day share the key) |
| Hindu days | `SHIVARATRI`, `HOLI_HILLS`, `HOLI_TERAI`, `RAKSHABANDHAN`, `JANMASHTAMI`, `RAMA_NAVAMI`, `MAKARA_SANKRANTI`, `MESHA_SANKRANTI`, `BASANTA_PANCHAMI`, `JITIYA` |
| the Kathmandu valley | `INDRA_JATRA`, `GAI_JATRA`, `GHODE_JATRA`, `RATO_MACHHINDRANATH_RATH_JATRA_BEGINS`, `NEPAL_SAMBAT_NEW_YEAR` |
| peoples | `UBHAULI`, `UDHAULI`, `MAGHI_THARU`, `GAURA_PARVA`, `SONAM_LHOSAR`, `TAMU_LHOSAR`, `GYALPO_LHOSAR` |
| other traditions | `BUDDHA_JAYANTI`, `EID_AL_FITR`, `EID_AL_ADHA`, `CHRISTMAS`, `GURU_NANAK_JAYANTI` |

`RAKSHABANDHAN` is the rule that Nepal leads with as Janai Purnima. Its text
says both: the sacred thread renewed and the rakhi tied. Bhoto Jatra has no
rule (the gazette leaves its day to the jatra), so the chariot festival's
first day, which a pack does hold, carries its text and the bhoto shown.

## 3. The shape

The SDK's data, `crates/panchanga/data/festival-about.json`, has one
entry per subject:

| field | holds |
|---|---|
| `key` | the subject, a rule's or a group's key |
| `observance` | the acts, in the order a page lists them, each an `observance` key (`TIKA_FROM_ELDERS`, `KITES`) |
| `sources` | the works the facts come from, each cited to its page, verse or section |

The words are entity records, not messages (C2416):

- **A festival's `about`** is a form of its `festival` record, beside
  `name` and `prose`. A group's key (`DASHAIN`, `TIHAR`) gains a
  `festival` record so that it has a name and an `about`.
- **An act** is a member of the open kind `observance`, named once by
  its record's `name` ("Tika and jamara from the elders"). Acts recur
  across festivals (`FAST`, `LAMPS`, `FEAST`), so a locale words each act
  once, and the data reuses a key wherever its words fit.

A message per festival would generate a typed accessor for each one in
every binding, and the whole catalogue would add thousands of them.
Records add none: their forms are read by key. Three typed messages say
them:

| intent | slots | says |
|---|---|---|
| `sdk.voice.festival.about` | `festival`, an entity | its `about` form |
| `sdk.voice.festival.observance` | `act`, an entity of kind `observance` | the act's name, as a list line |
| `sdk.voice.festival.who` | `tradition`, `communities`, `regions` | who keeps it and where |

The answer is a `FestivalAbout` per subject:

| field | what it holds |
|---|---|
| `key` | the subject |
| `about` | one item, `sdk.voice.festival.about` |
| `observance` | one item per act, `sdk.voice.festival.observance` |
| `who` | one item, `sdk.voice.festival.who` |
| `sources` | the entry's sources, as written |

Each item is a plan item (intl's `Item`, an intent with its params), so
`intl.render(item.key, item.params)` is the whole call. `who` reads the
rule's `meta` ([`festival-metadata.md`](festival-metadata.md)). For a
group, it reads the main member's `meta`. Its slots are:

- `tradition`, an entity of kind `tradition`, absent when no source places
  the rule;
- `communities`, a list of entities of kind `community`;
- `regions`, a list of entities of kind `region`.

Communities and regions are open keys, so they become open kinds, each
named in the packs for every value a shipped pack's meta holds (C2412).
A key from a consumer's own pack that has no name renders as itself, as
any unnamed key does. A region's key writes its ISO hyphen as an
underscore (`IN-TN` is `region.IN_TN`), since an entity's member key
holds letters, digits and underscores only.

## 4. Where it is asked

A pack listing ([`festival-metadata.md`](festival-metadata.md) §3) takes
`about: true` and answers `about`. That is a `FestivalAbout` for every
subject the pack holds that has an entry: its rules first, then its groups.
A subject without an entry is listed under `unwritten`, by key, so a
consumer sees what is missing rather than inferring it (C2413). Asked
without `about`, the listing is unchanged, and so is its hash.

## 5. The wording

The en-Latn and ne-Deva-NP texts ship final in the `sdk.entity` files.
The maintainer reviews them after shipping (Q42). Each `about` has two to
four sentences, and each act is one short line. Nepali is written the way
the national panchanga and the Ministry of Home Affairs' notices name the
days (`नेपाल सम्बत`, `घटस्थापना`).

A test renders every subject in both locales with no fallback and no
warning. It also requires every act the data lists to have a record in
each written locale, and every `observance` record to be listed by some
subject (C2414).

## 6. Cruxes

- **C2410.** An `about` is the pack's paraphrase of its cited sources. A
  sentence no listed source supports is removed, not hedged.
- **C2411.** A group's text and its main day's text say different things.
  `DASHAIN` says the fifteen days, and `VIJAYA_DASHAMI` says the tika.
  Where a group's main day has the same key as the group (`CHHATH`), one
  entry serves both.
- **C2412.** `community` and `region` become entity kinds named in the
  packs, one member for each key the shipped packs' meta holds. The kinds
  stay open: an unnamed key renders as itself.
- **C2413.** A listing names each subject with no entry under
  `unwritten`, never an empty `about`.
- **C2414.** The acts are the SDK's keys, so every locale says the same
  acts for a subject. A test requires each act and each `about` the data
  names to be worded in en-Latn and ne-Deva-NP.
- **C2415.** The register is the voice's (`sdk.voice`): plain words, the
  way a calendar app or a daily prints them, with no ritual instructions a
  priest would give.
- **C2416.** The texts are entity forms and records, read by key, not a
  message per festival. The catalogue's 776 festivals would otherwise
  generate thousands of typed accessors in each binding. The bundle's
  size is then the only cost, and stage (b) measures it against the pack
  budgets before it writes the rest.

## 7. Stage (b): the rest as a loaded corpus

Stage (a) put forty-two descriptions in the embedded bundle and measured
about 1 KB per subject (45 KB in `ne-Deva-NP`). The catalogue's other
subjects at that rate are about 0.7 MB a locale, and in the embedded
bundle that would ride in every artefact the SDK builds, which is the case
`interpretation-records.md` §3 settled for the readings: **a corpus too
large to embed is a pack that is loaded, not a bundle** (C2417).

- **The facts stay in the SDK** (`festival-about.json`): every subject's
  acts and sources. A listing asked with `about` answers them whatever the
  consumer loaded, and a subject with no entry is still `unwritten`.
- **An act's name stays embedded.** An act is one short line, recurs
  across festivals, and a listing names it in every answer, so the
  `observance` records stay in `i18n/`.
- **Stage (a)'s forty-two `about` forms stay embedded,** because the
  consumer's launch shows those days with no download (addendum 13's
  "top 40 first").
- **Every other `about` form is in `packs/festivals/<locale>/sdk.entity.json`.**
  A loaded record merges its forms onto the embedded one, so the pack
  carries only `about` and the name stands. The release ships it as
  `teistro-festivals-<version>-<tag>.tbundle`, beside the readings and the
  state readings (`packs/README.md`).

Without the corpus, a subject from stage (b) still lists its acts, its
who and its sources. Only its `about` item has no form to say, and the
renderer reports that as it reports any missing form. The test that every
subject renders whole loads the corpus first, as a consumer would.

The texts were written per pack from each day's rule, and each was checked
against its cited sources (C2410). Twenty-seven subjects are held back
because no source confirms them, or because the date rule they would
describe looked wrong. They stay `unwritten` until the rule is checked:
`CHAMPA_SHASHTHI` (the Khandoba day is Margashirsha's bright sixth, and
the print's Magha day is Skanda Shashthi), `BHAKTAPUR_CHANDI_BHAGAWATI_JATRA`
(sources give Baishakh's full moon), and the local jatras whose only
witness is the panchanga's own entry.

- **C2417.** Descriptions beyond stage (a)'s are a loaded corpus, one
  bundle a locale and a release asset each. The embedded bundle grows by
  the acts' names alone.

# The state readings, measured

Status: `generated` by `cargo xtask state-readings` over `packs/states`,
`packs/readings` and the shipped rule packs, 2026-09-22. Do not edit:
`check-state-readings` regenerates this page and fails on any
difference. The design it measures is
[`state-readings.md`](state-readings.md).

## Where the corpus landed

26 of the engine's 38 state categories map onto a subject this SDK has, and they become 350 records under 21 kinds. The mapping is a written table and not a resemblance: a category with no subject here is skipped and named below rather than guessed at.

| category | kinds | form | records |
|---|---|---|---:|
| `avastha-baladi` | `avastha_baladi` | `phala` | 5 |
| `avastha-deeptadi` | `avastha_deeptadi` | `phala` | 9 |
| `avastha-jagradadi` | `avastha_jagradadi` | `phala` | 3 |
| `avastha-lajjitadi` | `avastha_lajjitadi` | `phala` | 6 |
| `dasha-lord-activation` | `graha` | `dashaActivation` | 9 |
| `dasha-lord-effect` | `graha` | `dashaPhala` | 9 |
| `dosha-timing` | `rule` | `timing` | 62 |
| `gana` | `gana` | `phala` | 3 |
| `graha-bhava` | `graha_bhava` | a record of its own | 108 |
| `graha-color` | `graha` | `colour` | 9 |
| `graha-direction` | `graha` | `direction` | 9 |
| `inauspicious-kaal` | `kaala` | `phala` | 3 |
| `ishta-devata` | `rashi` | `ishtaDevata` | 12 |
| `lagna-rashi` | `rashi` | `lagnaPhala` | 12 |
| `mantra-ritual` | `graha` | `mantra` | 9 |
| `nadi` | `nadi` | `phala` | 3 |
| `nakshatra-phala` | `nakshatra` | `phala` | 27 |
| `namakarana-nakshatra` | `nakshatra` | `namakarana` | 27 |
| `planet-condition` | `dignity`, `state` | `phala` | 7 |
| `special-lagna` | `point` | `phala` | 5 |
| `tatwa` | `tatwa` | `phala` | 5 |
| `tithi-phala` | `tithi` | `phala` | 30 |
| `vara-phala` | `vara` | `phala` | 7 |
| `varna` | `varna` | `phala` | 5 |
| `yoga-phala` | `yoga` | `phala` | 27 |
| `yoni` | `yoni` | `phala` | 14 |

## What it did not map

12 categories, 126 records. The list is exhaustive rather than counted, because a category that gains a subject and a corpus that gains a category both have to change this page. Each is a decision rather than a task, and `state-readings.md` §8 says which kind of one.

| category | records |
|---|---:|
| `auspicious-kaal` | 5 |
| `ayurdaya-balarishta` | 4 |
| `ayurdaya-classical-rule` | 5 |
| `ayurdaya-harana` | 4 |
| `ayurdaya-maraka` | 9 |
| `ayurdaya-maraka-trigger` | 3 |
| `ayurdaya-method` | 3 |
| `ayurdaya-tier` | 4 |
| `ayurdaya-vulnerability` | 9 |
| `muhurta-factor` | 47 |
| `sade-sati-phala` | 5 |
| `shadbala-strength` | 28 |

## Where two corpora meet

48 subjects carry forms from more than one category. They are why a reading is a form on a record rather than a record of its own: a migration that replaced would keep whichever category it read last, and the subject would lose the other reading silently.

| kind | subjects described more than once | the categories describing them |
|---|---:|---|
| `graha` | 9 | `dasha-lord-activation`, `dasha-lord-effect`, `graha-color`, `graha-direction`, `mantra-ritual` |
| `nakshatra` | 27 | `nakshatra-phala`, `namakarana-nakshatra` |
| `rashi` | 12 | `ishta-devata`, `lagna-rashi` |

## What it adds to the rule readings

`dosha-timing` keys onto rules, so the two corpora meet on the same
records. Against the 657 rules the shipped packs hold:

| | |
|---|---:|
| rules that had a reading and gain a timing | 34 |
| rules that had no reading and gain a timing | 18 |
| rules with neither | 0 |
| timings naming no shipped rule | 10 |

**The rules that gain their only text** are `ANGARAK_DOSHA`,
`CHANDAL_DOSHA`, `CHANDRA_DOSHA`, `KALA_AMRITA_YOGA`, `KALSARPA_ANANT`,
`KALSARPA_GHATAK`, `KALSARPA_KARKOTAK`, `KALSARPA_KULIK`,
`KALSARPA_MAHAPADMA`, `KALSARPA_PADMA`, `KALSARPA_SHANKHACHUD`,
`KALSARPA_SHANKHPAL`, `KALSARPA_SHESHNAG`, `KALSARPA_TAKSHAK`,
`KALSARPA_VASUKI`, `KALSARPA_VISHDHAR`, `SHRAPIT_DOSHA`,
`VISH_YOGA_DOSHA`. They are exactly the list
`interpretation-records-measured.md` carries as rules with no reading,
and what they gain is a `timing` rather than a reading: it says when the
dosha acts, not what it means, and calling it a reading would be a claim
the corpus does not make.

**The timings naming no shipped rule** are `ANSHIK_MANGAL`,
`BHAKOOT_DOSHA`, `GANA_DOSHA`, `KUJA_BHANGA_ABSENT`, `MAHENDRA_ABSENCE`,
`NADI_DOSHA`, `PURNA_MANGAL`, `RAJJU_DOSHA`, `STREE_DEERGH_ABSENCE`,
`VEDHA_DOSHA`. They are the same ten the readings' page lists: the milan
rules, waiting on Phase 8's `matching` rather than on anything here.

## What a composer can say

A pack that loads is half of it; the other half is that a record reaches
a reader. Every reading a composer has a message for — `phala` for the
chart's subjects and what its nakshatra is, `readings` for a rule's
timing — is rendered here through that message, in each strict locale,
and must answer from the locale's own record without a fallback and
without a warning: 660 renderings.

**95 readings have no composer that says them yet**, by the category
they came from. They are not errors — the records load and a consumer
reads them directly — but a reading nothing says is work that has not
reached a reader, so the list is here rather than in a sentence.

| category | readings nothing says |
|---|---:|
| `avastha-baladi` | 5 |
| `avastha-deeptadi` | 9 |
| `avastha-jagradadi` | 3 |
| `avastha-lajjitadi` | 6 |
| `dasha-lord-activation` | 9 |
| `dasha-lord-effect` | 9 |
| `graha-color` | 9 |
| `graha-direction` | 9 |
| `inauspicious-kaal` | 3 |
| `ishta-devata` | 12 |
| `mantra-ritual` | 9 |
| `planet-condition` | 7 |
| `special-lagna` | 5 |

| proposed rule | verdict | measured |
|---|---|---|
| every reading a composer says renders from the locale's own record, with no fallback and no warning | **holds** | 0 of 660 disagree |

## What it leaves open

**3 readings refused.** A key of a mapped category that names no member
of any of its kinds is a reading this SDK has no subject for. It is
written down with the reason rather than reported afresh each run, and
the list fails both ways: a key here that names a member now fails, and
one that names none and is not here fails.

| category | key | why |
|---|---|---|
| `inauspicious-kaal` | `dur-muhurta` | the SDK divides the day into muhurtas but names none of them, so the censured one has no key |
| `inauspicious-kaal` | `varjyam` | the SDK computes no varjyam window, and no kind names one |
| `planet-condition` | `COMBUST_CANCELLED` | the SDK computes combustion but not its cancellation, and no `state` member names one |

**26 readings land on a record the base locale does not name.** The
reading is there and answers; the subject's own name is not, because
`03-design/entity-names.md` §4 refuses a translated stub and these
kinds have no vetted table. A renderer asking for the name gets nothing,
so the count is here.

| kind | records with a reading and no name |
|---|---:|
| `point` | 5 |
| `rule` | 18 |
| `state` | 3 |

| proposed rule | verdict | measured |
|---|---|---|
| every refused key is absent from the packs | **holds** | 0 of 3 disagree |
| every record carried is one the base locale can resolve or an open kind's own | **holds** | 0 of 1750 disagree |

## What the state readings cost

**Loaded, not embedded**, for the reason the rule readings are
(`interpretation-records.md` §3): `crates/sdk`'s build script compiles
`i18n/` into every artefact, and a consumer computing a Julian day
should not carry a Sanskrit passage on Jupiter in the first house to do
it. A consumer loads one pack a locale from each root it wants.

| locale | source | pack |
|---|---:|---:|
| `en-Latn` | 203 KB | 197 KB |
| `hi-Deva-IN` | 387 KB | 381 KB |
| `ne-Deva-NP` | 365 KB | 359 KB |
| `sa-Deva` | 357 KB | 351 KB |
| `sa-Latn` | 198 KB | 191 KB |
| **all** | **1513 KB** | **1481 KB** |

Beside the rule readings' 2920 KB, which is the other pack a consumer
that wants both would load: 4401 KB in all for every language, and 1096
KB for one.

## What the packs decide

| proposed rule | verdict | measured |
|---|---|---|
| every record names a catalogue member, or a well-formed key of an open kind | **holds** | 0 of 350 disagree |
| every locale carries every record the base locale carries | **holds** | 0 of 1750 disagree |
| every record carries a form to be read by | **holds** | 0 of 1750 disagree |
| every locale's state readings build into a pack an engine can load | **holds** | 0 of 5 disagree |
| every form answers from the loaded packs, with no source tree behind them | **holds** | 0 of 4850 disagree |
| a rule's reading still answers after the state readings are loaded over it | **holds** | 0 of 660 disagree |


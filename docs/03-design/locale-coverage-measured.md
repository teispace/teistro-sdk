# What each locale says

Status: `generated` by `cargo xtask locale-coverage`. Do not edit: `check-locale-coverage` regenerates this page, and fails when a locale's count falls below its floor in `docs/03-design/locale-coverage-floor.json`, which is raised by hand as a pack grows.

A **member** is a catalogue key a document can carry (its schema names the kind), and a locale names it when its `sdk.entity` namespace holds the record. A **message** is one of the base locale's messages, and a locale translates it when it holds the same key. A strict locale is complete but for the members `xtask/src/intl.rs` lists as unnamed, and `check-intl` gates it; a base locale falls back along its chain for the rest, so this page is where what it leaves out shows.

| locale | completeness | members named | | messages translated | |
|---|---|---:|---:|---:|---:|
| `en-Latn` | strict | 429 of 515 | 83.3% | 181 of 181 | 100.0% |
| `hi-Deva-IN` | base | 332 of 515 | 64.5% | 74 of 181 | 40.9% |
| `ne-Deva-NP` | strict | 429 of 515 | 83.3% | 181 of 181 | 100.0% |
| `sa-Deva` | base | 332 of 515 | 64.5% | 68 of 181 | 37.6% |
| `sa-Latn` | base | 332 of 515 | 64.5% | 0 of 181 | 0.0% |

515 members of 40 kinds, 181 messages.

`hi-Deva-IN` ships unreviewed, marked draft on every render (QUESTIONS.md Q42): `sdk.calendar`.

`sa-Deva` ships unreviewed, marked draft on every render (QUESTIONS.md Q42): `sdk.calendar`.

## By kind

| kind | members | `en-Latn` | `hi-Deva-IN` | `ne-Deva-NP` | `sa-Deva` | `sa-Latn` |
|---|---:|---:|---:|---:|---:|---:|
| `avastha_baladi` | 5 | 5 | 5 | 5 | 5 | 5 |
| `avastha_cheshta` | 3 | 3 | 0 | 3 | 0 | 0 |
| `avastha_deeptadi` | 9 | 9 | 9 | 9 | 9 | 9 |
| `avastha_jagradadi` | 3 | 3 | 3 | 3 | 3 | 3 |
| `avastha_lajjitadi` | 6 | 6 | 6 | 6 | 6 | 6 |
| `avastha_sayanadi` | 12 | 12 | 0 | 12 | 0 | 0 |
| `ayana` | 2 | 2 | 2 | 2 | 2 | 2 |
| `ayanamsha` | 47 | 47 | 47 | 47 | 47 | 47 |
| `calendar` | 14 | 0 | 0 | 0 | 0 | 0 |
| `chart_kind` | 7 | 0 | 0 | 0 | 0 | 0 |
| `choghadiya` | 7 | 7 | 7 | 7 | 7 | 7 |
| `dasha_system` | 45 | 29 | 18 | 29 | 18 | 18 |
| `dignity` | 11 | 11 | 11 | 11 | 11 | 11 |
| `direction` | 8 | 8 | 8 | 8 | 8 | 8 |
| `era` | 10 | 9 | 0 | 9 | 0 | 0 |
| `gana` | 3 | 3 | 3 | 3 | 3 | 3 |
| `graha` | 12 | 9 | 9 | 9 | 9 | 9 |
| `house_system` | 22 | 22 | 22 | 22 | 22 | 22 |
| `kaala` | 3 | 3 | 3 | 3 | 3 | 3 |
| `karana` | 11 | 11 | 11 | 11 | 11 | 11 |
| `masa` | 12 | 12 | 12 | 12 | 12 | 12 |
| `muhurta_yoga` | 5 | 5 | 5 | 5 | 5 | 5 |
| `nadi` | 3 | 3 | 3 | 3 | 3 | 3 |
| `nakshatra` | 27 | 27 | 27 | 27 | 27 | 27 |
| `nature` | 3 | 2 | 0 | 2 | 0 | 0 |
| `paksha` | 2 | 2 | 2 | 2 | 2 | 2 |
| `panchaka` | 5 | 5 | 5 | 5 | 5 | 5 |
| `point` | 49 | 19 | 8 | 19 | 8 | 8 |
| `rashi` | 12 | 12 | 12 | 12 | 12 | 12 |
| `relationship` | 5 | 5 | 5 | 5 | 5 | 5 |
| `ritu` | 6 | 6 | 6 | 6 | 6 | 6 |
| `shiva_vasa` | 7 | 7 | 0 | 7 | 0 | 0 |
| `tithi` | 30 | 30 | 30 | 30 | 30 | 30 |
| `tithi_class` | 5 | 0 | 0 | 0 | 0 | 0 |
| `vaiseshikamsa` | 30 | 27 | 0 | 27 | 0 | 0 |
| `vara` | 7 | 7 | 7 | 7 | 7 | 7 |
| `varga` | 21 | 15 | 0 | 15 | 0 | 0 |
| `varna` | 5 | 5 | 5 | 5 | 5 | 5 |
| `yoga` | 27 | 27 | 27 | 27 | 27 | 27 |
| `yoni` | 14 | 14 | 14 | 14 | 14 | 14 |

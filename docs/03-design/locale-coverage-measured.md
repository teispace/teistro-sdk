# What each locale says

Status: `generated` by `cargo xtask locale-coverage`. Do not edit: `check-locale-coverage` regenerates this page, and fails when a locale's count falls below its floor in `docs/03-design/locale-coverage-floor.json`, which is raised by hand as a pack grows.

A **member** is a catalogue key a document can carry (its schema names the kind), and a locale names it when its `sdk.entity` namespace holds the record. A **message** is one of the base locale's messages, and a locale translates it when it holds the same key. A strict locale is complete but for the members `xtask/src/intl.rs` lists as unnamed, and `check-intl` gates it; a base locale falls back along its chain for the rest, so this page is where what it leaves out shows.

| locale | completeness | members named | | messages translated | |
|---|---|---:|---:|---:|---:|
| `en-Latn` | strict | 541 of 541 | 100.0% | 230 of 230 | 100.0% |
| `hi-Deva-IN` | base | 427 of 541 | 78.9% | 217 of 230 | 94.3% |
| `ne-Deva-NP` | strict | 541 of 541 | 100.0% | 230 of 230 | 100.0% |
| `sa-Deva` | base | 426 of 541 | 78.7% | 214 of 230 | 93.0% |
| `sa-Latn` | base | 426 of 541 | 78.7% | 214 of 230 | 93.0% |

541 members of 44 kinds, 230 messages.

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
| `calendar` | 16 | 16 | 16 | 16 | 16 | 16 |
| `chart_frame` | 4 | 4 | 4 | 4 | 4 | 4 |
| `chart_kind` | 8 | 8 | 8 | 8 | 8 | 8 |
| `choghadiya` | 7 | 7 | 7 | 7 | 7 | 7 |
| `dasha_system` | 45 | 45 | 34 | 45 | 34 | 34 |
| `day_marker` | 3 | 3 | 0 | 3 | 0 | 0 |
| `dignity` | 11 | 11 | 11 | 11 | 11 | 11 |
| `direction` | 8 | 8 | 8 | 8 | 8 | 8 |
| `era` | 12 | 12 | 2 | 12 | 1 | 1 |
| `gana` | 3 | 3 | 3 | 3 | 3 | 3 |
| `gowri` | 8 | 8 | 0 | 8 | 0 | 0 |
| `graha` | 12 | 12 | 12 | 12 | 12 | 12 |
| `house_system` | 22 | 22 | 22 | 22 | 22 | 22 |
| `kaala` | 3 | 3 | 3 | 3 | 3 | 3 |
| `karana` | 11 | 11 | 11 | 11 | 11 | 11 |
| `masa` | 12 | 12 | 12 | 12 | 12 | 12 |
| `muhurta_yoga` | 7 | 7 | 5 | 7 | 5 | 5 |
| `nadi` | 3 | 3 | 3 | 3 | 3 | 3 |
| `nakshatra` | 27 | 27 | 27 | 27 | 27 | 27 |
| `nature` | 3 | 3 | 1 | 3 | 1 | 1 |
| `paksha` | 2 | 2 | 2 | 2 | 2 | 2 |
| `panchaka` | 5 | 5 | 5 | 5 | 5 | 5 |
| `paya` | 4 | 4 | 0 | 4 | 0 | 0 |
| `point` | 49 | 49 | 38 | 49 | 38 | 38 |
| `rashi` | 12 | 12 | 12 | 12 | 12 | 12 |
| `relationship` | 5 | 5 | 5 | 5 | 5 | 5 |
| `ritu` | 6 | 6 | 6 | 6 | 6 | 6 |
| `shiva_vasa` | 7 | 7 | 0 | 7 | 0 | 0 |
| `tithi` | 30 | 30 | 30 | 30 | 30 | 30 |
| `tithi_class` | 5 | 5 | 5 | 5 | 5 | 5 |
| `vaiseshikamsa` | 30 | 30 | 3 | 30 | 3 | 3 |
| `vara` | 7 | 7 | 7 | 7 | 7 | 7 |
| `varga` | 21 | 21 | 7 | 21 | 7 | 7 |
| `varna` | 5 | 5 | 5 | 5 | 5 | 5 |
| `yoga` | 27 | 27 | 27 | 27 | 27 | 27 |
| `yoni` | 14 | 14 | 14 | 14 | 14 | 14 |

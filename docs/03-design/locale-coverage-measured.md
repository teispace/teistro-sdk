# What each locale says

Status: `generated` by `cargo xtask locale-coverage`. Do not edit: `check-locale-coverage` regenerates this page, and fails when a locale's count falls below its floor in `docs/03-design/locale-coverage-floor.json`, which is raised by hand as a pack grows.

A **member** is a catalogue key a document can carry (its schema names the kind), and a locale names it when its `sdk.entity` namespace holds the record. A **message** is one of the base locale's messages, and a locale translates it when it holds the same key. A strict locale is complete but for the members `xtask/src/intl.rs` lists as unnamed, and `check-intl` gates it; a base locale falls back along its chain for the rest, so this page is where what it leaves out shows.

| locale | completeness | members named | | messages translated | |
|---|---|---:|---:|---:|---:|
| `en-Latn` | strict | 672 of 672 | 100.0% | 536 of 536 | 100.0% |
| `en-Latn-NP` | base | 12 of 672 | 1.8% | 0 of 536 | 0.0% |
| `hi-Deva-IN` | base | 662 of 672 | 98.5% | 439 of 536 | 81.9% |
| `ne-Deva-NP` | strict | 672 of 672 | 100.0% | 536 of 536 | 100.0% |
| `sa-Deva` | base | 661 of 672 | 98.4% | 439 of 536 | 81.9% |
| `sa-Latn` | base | 661 of 672 | 98.4% | 361 of 536 | 67.4% |

672 members of 45 kinds, 536 messages.

## By kind

| kind | members | `en-Latn` | `en-Latn-NP` | `hi-Deva-IN` | `ne-Deva-NP` | `sa-Deva` | `sa-Latn` |
|---|---:|---:|---:|---:|---:|---:|---:|
| `avastha_baladi` | 5 | 5 | 0 | 5 | 5 | 5 | 5 |
| `avastha_cheshta` | 3 | 3 | 0 | 3 | 3 | 3 | 3 |
| `avastha_deeptadi` | 9 | 9 | 0 | 9 | 9 | 9 | 9 |
| `avastha_jagradadi` | 3 | 3 | 0 | 3 | 3 | 3 | 3 |
| `avastha_lajjitadi` | 6 | 6 | 0 | 6 | 6 | 6 | 6 |
| `avastha_sayanadi` | 12 | 12 | 0 | 12 | 12 | 12 | 12 |
| `ayana` | 2 | 2 | 0 | 2 | 2 | 2 | 2 |
| `ayanamsha` | 48 | 48 | 0 | 48 | 48 | 48 | 48 |
| `calendar` | 17 | 17 | 0 | 17 | 17 | 17 | 17 |
| `chart_frame` | 4 | 4 | 0 | 4 | 4 | 4 | 4 |
| `chart_kind` | 8 | 8 | 0 | 8 | 8 | 8 | 8 |
| `choghadiya` | 7 | 7 | 0 | 7 | 7 | 7 | 7 |
| `dasha_system` | 45 | 45 | 0 | 45 | 45 | 45 | 45 |
| `day_marker` | 3 | 3 | 0 | 3 | 3 | 3 | 3 |
| `dignity` | 11 | 11 | 0 | 11 | 11 | 11 | 11 |
| `direction` | 8 | 8 | 0 | 8 | 8 | 8 | 8 |
| `era` | 13 | 13 | 0 | 3 | 13 | 2 | 2 |
| `gana` | 3 | 3 | 0 | 3 | 3 | 3 | 3 |
| `gowri` | 8 | 8 | 0 | 8 | 8 | 8 | 8 |
| `graha` | 12 | 12 | 0 | 12 | 12 | 12 | 12 |
| `house_system` | 22 | 22 | 0 | 22 | 22 | 22 | 22 |
| `kaala` | 3 | 3 | 0 | 3 | 3 | 3 | 3 |
| `karana` | 11 | 11 | 0 | 11 | 11 | 11 | 11 |
| `masa` | 12 | 12 | 0 | 12 | 12 | 12 | 12 |
| `muhurta_yoga` | 7 | 7 | 0 | 7 | 7 | 7 | 7 |
| `nadi` | 3 | 3 | 0 | 3 | 3 | 3 | 3 |
| `nakshatra` | 27 | 27 | 0 | 27 | 27 | 27 | 27 |
| `nature` | 3 | 3 | 0 | 3 | 3 | 3 | 3 |
| `paksha` | 2 | 2 | 0 | 2 | 2 | 2 | 2 |
| `panchaka` | 5 | 5 | 0 | 5 | 5 | 5 | 5 |
| `paya` | 4 | 4 | 0 | 4 | 4 | 4 | 4 |
| `point` | 49 | 49 | 0 | 49 | 49 | 49 | 49 |
| `rashi` | 12 | 12 | 12 | 12 | 12 | 12 | 12 |
| `relationship` | 5 | 5 | 0 | 5 | 5 | 5 | 5 |
| `ritu` | 6 | 6 | 0 | 6 | 6 | 6 | 6 |
| `shiva_vasa` | 7 | 7 | 0 | 7 | 7 | 7 | 7 |
| `star` | 128 | 128 | 0 | 128 | 128 | 128 | 128 |
| `tithi` | 30 | 30 | 0 | 30 | 30 | 30 | 30 |
| `tithi_class` | 5 | 5 | 0 | 5 | 5 | 5 | 5 |
| `vaiseshikamsa` | 30 | 30 | 0 | 30 | 30 | 30 | 30 |
| `vara` | 7 | 7 | 0 | 7 | 7 | 7 | 7 |
| `varga` | 21 | 21 | 0 | 21 | 21 | 21 | 21 |
| `varna` | 5 | 5 | 0 | 5 | 5 | 5 | 5 |
| `yoga` | 27 | 27 | 0 | 27 | 27 | 27 | 27 |
| `yoni` | 14 | 14 | 0 | 14 | 14 | 14 | 14 |

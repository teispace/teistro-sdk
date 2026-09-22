# The rule readings, measured

Status: `generated` by `cargo xtask interpretations` over
`packs/readings` and the shipped rule packs, 2026-09-21. Do not edit:
`check-interpretations` regenerates this page and fails on any
difference. The design it measures is
[`interpretation-records.md`](interpretation-records.md).

## What the packs and the readings say of each other

639 of the 657 rules the shipped packs hold carry a reading, in 5 locales. The two lists below are exhaustive rather than counted, because a rule that gains a reading and a reading that loses its rule both have to change this page.

| | |
|---|---:|
| rules with a reading | 639 |
| rules without | 18 |
| readings naming no shipped rule | 10 |

**The rules with no reading**, by pack:

- `doshas`: `ANGARAK_DOSHA`, `CHANDAL_DOSHA`, `CHANDRA_DOSHA`, `KALA_AMRITA_YOGA`, `KALSARPA_ANANT`, `KALSARPA_GHATAK`, `KALSARPA_KARKOTAK`, `KALSARPA_KULIK`, `KALSARPA_MAHAPADMA`, `KALSARPA_PADMA`, `KALSARPA_SHANKHACHUD`, `KALSARPA_SHANKHPAL`, `KALSARPA_SHESHNAG`, `KALSARPA_TAKSHAK`, `KALSARPA_VASUKI`, `KALSARPA_VISHDHAR`, `SHRAPIT_DOSHA`, `VISH_YOGA_DOSHA`

**The readings naming no shipped rule** are `ANSHIK_MANGAL`,
`BHAKOOT_DOSHA`, `GANA_DOSHA`, `KUJA_BHANGA_ABSENT`, `MAHENDRA_ABSENCE`,
`NADI_DOSHA`, `PURNA_MANGAL`, `RAJJU_DOSHA`, `STREE_DEERGH_ABSENCE`,
`VEDHA_DOSHA`. They are not errors: each is a reading waiting for a
module the SDK has not built, and a pass that deleted them would lose
work already done.

## The facets a reading carries

A record's `name` is its summary and its `prose` is the passage; the
rest are the reading's named facets, and they are an **open** set rather
than a fixed record — a reading carries the facets it has. Over 3245
language-records:

| facet | records |
|---|---:|
| `body` | 36 |
| `career` | 778 |
| `general` | 1640 |
| `longevity` | 34 |
| `material` | 433 |
| `mind` | 1289 |
| `relationships` | 506 |
| `spirituality` | 707 |
| `wealth` | 155 |

**A facet the base locale has is not always a facet another has.** 609
facets in all, counted rather than hidden: where a locale lacks a facet
its record's summary stands in, which is a visible shortfall and not a
wrong answer.

| locale | facets the base has and it lacks |
|---|---:|
| `hi-Deva-IN` | 154 |
| `ne-Deva-NP` | 11 |
| `sa-Deva` | 222 |
| `sa-Latn` | 222 |

## What the readings cost

They are **loaded, not embedded**: `crates/sdk`'s build script compiles
`i18n/` into every artefact the SDK produces, and this corpus is several
times that root's size, so a consumer computing a Julian day would carry
every Nepali yoga reading to do it. The numbers are why the decision is
a decision — and the pack is not much smaller than its source, so
nothing is being deferred to compression.

| locale | source | pack |
|---|---:|---:|
| `en-Latn` | 465 KB | 443 KB |
| `hi-Deva-IN` | 802 KB | 782 KB |
| `ne-Deva-NP` | 758 KB | 736 KB |
| `sa-Deva` | 638 KB | 618 KB |
| `sa-Latn` | 358 KB | 338 KB |
| **all** | **3023 KB** | **2920 KB** |

One pack a locale, because a Nepali application wants Nepali and its
fallback rather than every language's worth of prose: 736 KB of the 2920
KB, and the consumer chooses.

Against the root the build script *does* embed: `i18n/` is **311 KB** of
messages across every locale and namespace, so the corpus is about **10
times** it. Both halves of that comparison are measured here, because
the figure was written into three documents as three different numbers
and none of them was current.

## What the packs decide

| proposed rule | verdict | measured |
|---|---|---|
| every reading names a key a rule pack could name | **holds** | 0 of 649 disagree |
| every locale carries a reading for every rule the base locale reads | **holds** | 0 of 3245 disagree |
| every reading carries a summary to be named by | **holds** | 0 of 3245 disagree |
| every locale's readings build into a pack an engine can load | **holds** | 0 of 5 disagree |
| every reading answers from the loaded pack, with no source tree behind it | **holds** | 0 of 3245 disagree |


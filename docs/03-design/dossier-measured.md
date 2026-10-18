# The dossier, measured

Status: `generated` by `cargo xtask dossier` and held by
`check-dossier`; do not edit. What a chart's dossier (`dossier.md`)
spends against the canonical JSON of the same chart, over the
conformance corpus's births founded by the SDK under the
`conformance-baseline` profile on the built-in ephemeris, each with
every section a dossier writes: the state, the panchanga, the houses,
the Ashtakavarga, Shadbala, Bhava bala, Vimshopaka, Jaimini, the points,
the aspects, the Avakahada, the D9 and the D10, and the Vimshottari to
the depth the request computes. The dossier is date-free and on each
birth's own clock. A birth whose day has no sunrise (a polar summer)
cannot carry the points, which count from it, and is founded without
them.

Of the corpus's 55 births, 2 are set aside, a section reading a day the
built-in ephemeris does not reach: `c047-london-1800-01-02`,
`c048-kathmandu-2399-12-30`.

## The claims

| proposed rule | verdict | measured |
|---|---|---|
| every chart's whole dossier is fewer bytes than its document's canonical JSON | **holds** | 0 of 53 disagree |
| every section's dossier is fewer bytes than the JSON of what it writes, chart by chart | **holds** | 0 of 687 disagree |

## A chart whole

Over 53 births the dossiers come to 2 677 362 bytes and the documents'
canonical JSON to 6 850 082: 0.39 of it, a chart's own ratio between
0.26 and 0.42.

## Section by section

A section's bytes are what adding it to a dossier costs — its legend
lines, banner, header and rows — and its JSON is the canonical JSON of
the document members it is written from (`CHART` and `GRAHAS` together,
against the foundation and the state), summed over the births.

| sections | charts carrying them | dossier bytes | JSON bytes | dossier / JSON | charts where the dossier is not smaller |
|---|---|---|---|---|---|
| `CHART`, `GRAHAS` | 53 | 102 520 | 658 172 | 0.16 | 0 |
| `HOUSES` | 53 | 32 648 | 90 456 | 0.36 | 0 |
| `VARGAS` | 53 | 27 943 | 114 819 | 0.24 | 0 |
| `DASHAS` | 53 | 2 187 257 | 4 083 902 | 0.54 | 0 |
| `PANCHANGA` | 53 | 17 036 | 590 015 | 0.03 | 0 |
| `ASHTAKAVARGA` | 53 | 42 188 | 51 858 | 0.81 | 0 |
| `SHADBALA` | 53 | 55 508 | 228 801 | 0.24 | 0 |
| `BHAVA_BALA` | 53 | 43 571 | 82 771 | 0.53 | 0 |
| `VIMSHOPAKA` | 53 | 24 866 | 37 072 | 0.67 | 0 |
| `JAIMINI` | 53 | 18 324 | 95 569 | 0.19 | 0 |
| `POINTS` | 51 | 44 727 | 176 019 | 0.25 | 0 |
| `ASPECTS` | 53 | 57 384 | 628 139 | 0.09 | 0 |
| `AVAKAHADA` | 53 | 12 313 | 13 123 | 0.94 | 0 |

Bytes are not tokens: a tokenizer splits a catalogue key
(`PURVA_PHALGUNI`) and a number differently from a JSON key, so this
page counts what is stored and sent, and a token count is the model's
own.

# Licensing

Status: `research`, 2026-09-04. Feeds Q5 and Q6. This is not legal advice;
it is the map of the questions.

## The separation that makes the SDK licence-clean

The SDK links no ephemeris. Swiss Ephemeris is AGPL-3.0 or a Professional
Licence; Teimeris exercises the AGPL and holds a Professional Licence for
private use; both obligations attach to whoever ships them, and the
consumer chooses which provider to ship. The SDK's ephemeris port, the
adapters (published separately, each under the licence its ephemeris
requires) and the SDK core are three different licensing surfaces by
design. A consumer using a public-domain provider carries no ephemeris
obligation at all.

## Components and their licence questions

| component | question |
|---|---|
| SDK core and bindings | Q5: Apache-2.0 (patent grant, adoption), MIT, BSL or proprietary with a commercial tier, dual |
| Teimeris adapter | must follow Teimeris's terms (AGPL or the private arrangement); published separately |
| Swiss Ephemeris adapters | AGPL or Professional; separate packages |
| CLDR-derived data (plural rules, numbering systems, calendars via ICU4X) | Unicode licence, permissive |
| tzdb | public domain |
| BS calendar table | derived from Nepal government publications; confirm terms |
| The baseline engine rule corpus, interpretation texts, entity names, corpora | Q6 decided 2026-09-04: Teispace owns the SDK and all of the baseline engine's content moves into it |
| VSOP87 and ELP series, JPL kernels (built-in ephemeris) | published scientific series used with citation; JPL data public domain; Q17 confirms |
| classical text citations and translations | citations are facts; translated passages may have translator copyright; original prose is ours |
| name corpus (namakarana) | source and terms to confirm |
| interpretation content packs | can be commercial and separately licensed even if the core is open |
| trademarks | "Teistro" and "Teispace" as marks; third-party names (Swiss Ephemeris, Jagannatha Hora) only descriptively |

## Dependency policy (decided as ADR-0019)

An allow list in `deny.toml`: MIT, Apache-2.0 (with the LLVM exception),
BSD-2 and BSD-3, ISC, Zlib, 0BSD, Unicode, CC0, Unlicense. Denied
everywhere in the workspace, dev-dependencies included: GPL, LGPL, AGPL,
SSPL and MPL. The MPL denial is what makes the SDK port ERFA from its
BSD-3 C source rather than depend on the MPL-licensed Rust port, and what
keeps ANISE an oracle. Oracles under other terms live in unpublished
crates or recorded fixtures; the clean-room rules for reading copyleft
implementations are in `CLEAN_ROOM.md`.

## Distribution model options

| model | implication |
|---|---|
| open core (Apache-2.0) with commercial content packs and support | maximum adoption; the value is in packs, hosting and Teimeris |
| source-available (BSL) converting to open after a term | protects against competitors shipping it as a service; slows adoption |
| proprietary SDK with a free tier | conventional for astrology software; contradicts "anyone can build on it" |
| private for Teispace products first, public later | lowest risk; the Teimeris model; delays the ecosystem |

The architecture is the same under every model; only packaging and the
pack signing feature change. Decide before the repository is created
because the licence file and headers go in the first commit.

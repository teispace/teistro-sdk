# The dossier's tokens, measured

Status: `generated` by `cargo xtask dossier-tokens` and held by
`check-dossier-tokens`; do not edit. Each birth of the conformance
corpus founded with every section, as the dossier's own page founds it
(`dossier-measured.md`), written as a whole date-free dossier on its own
clock: once in catalogue keys, the language-neutral mode, and once in
each shipped locale's names (`dossier.md`, "Names"), beside the
canonical JSON of the same document.

2 of the corpus's 55 births are set aside, a section reading a day the built-in ephemeris does not reach: `c047-london-1800-01-02`, `c048-kathmandu-2399-12-30`.

## The tokeniser

Tokens are counted by an approximation with no dependency
(`xtask/src/tokens.rs`, C1165): a run of ASCII letters is a token for
each 4 letters or part of 4, a run of digits a token for each 3, a
newline one, other whitespace nothing, any other ASCII character one,
and any other scalar value (Devanagari, IAST's marked letters) one. No
model's tokenizer ships with the SDK and each is its own trained
vocabulary, so a count by one is that model's alone; this one prices the
pieces a byte-pair tokenizer cannot merge across, the same ruler for
every mode, and its prices are held by the module's own tests. Read the
columns against each other, not as a model's bill.

## The claims

| proposed rule | verdict | measured |
|---|---|---|
| every dossier written in a locale's names keeps the keyed one's lines, banners, column headers and field counts | **holds** | 0 of 265 disagree |
| the keyed dossier is fewer tokens than the document's canonical JSON, over the corpus | **holds** | 0 of 1 disagree |

## Over 53 charts

| written in | bytes | scalar values | tokens | tokens a chart | tokens against keys |
|---|---|---|---|---|---|
| keys (language-neutral) | 2 681 190 | 2 681 190 | 1 547 721 | 29 202 | 1.00 |
| `en-Latn` names | 2 953 396 | 2 953 396 | 1 636 749 | 30 882 | 1.06 |
| `hi-Deva-IN` names | 4 291 044 | 2 751 386 | 2 105 869 | 39 733 | 1.36 |
| `ne-Deva-NP` names | 4 293 996 | 2 749 372 | 2 107 097 | 39 756 | 1.36 |
| `sa-Deva` names | 4 293 436 | 2 751 960 | 2 106 795 | 39 750 | 1.36 |
| `sa-Latn` names | 3 144 303 | 2 838 010 | 1 838 110 | 34 681 | 1.19 |
| the document's canonical JSON | 7 149 254 | 7 148 971 | 4 006 309 | 75 590 | 2.59 |

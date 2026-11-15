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
| keys (language-neutral) | 2 681 541 | 2 681 541 | 1 547 883 | 29 205 | 1.00 |
| `en-Latn` names | 2 953 737 | 2 953 737 | 1 636 896 | 30 884 | 1.06 |
| `hi-Deva-IN` names | 4 291 529 | 2 751 659 | 2 106 071 | 39 737 | 1.36 |
| `ne-Deva-NP` names | 4 294 573 | 2 749 621 | 2 107 329 | 39 760 | 1.36 |
| `sa-Deva` names | 4 293 921 | 2 752 233 | 2 106 997 | 39 754 | 1.36 |
| `sa-Latn` names | 3 144 646 | 2 838 316 | 1 838 273 | 34 684 | 1.19 |
| the document's canonical JSON | 7 026 770 | 7 026 487 | 3 937 620 | 74 294 | 2.54 |

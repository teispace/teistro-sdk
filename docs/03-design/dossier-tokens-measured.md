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
| keys (language-neutral) | 2 678 792 | 2 678 792 | 1 546 627 | 29 181 | 1.00 |
| `en-Latn` names | 2 941 417 | 2 941 417 | 1 632 981 | 30 810 | 1.06 |
| `hi-Deva-IN` names | 4 400 098 | 2 761 664 | 2 144 135 | 40 455 | 1.39 |
| `ne-Deva-NP` names | 4 404 404 | 2 760 504 | 2 145 677 | 40 484 | 1.39 |
| `sa-Deva` names | 4 399 091 | 2 761 187 | 2 143 764 | 40 448 | 1.39 |
| `sa-Latn` names | 3 152 152 | 2 846 747 | 1 846 261 | 34 835 | 1.19 |
| the document's canonical JSON | 7 146 644 | 7 146 361 | 4 005 109 | 75 568 | 2.59 |

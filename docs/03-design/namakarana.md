# Namakarana: the four names, the pada's syllables and a consumer's name book (the `naming` area)

Status: `decided`, 2026-10-28, from the sources before any code. Work
items 1 to 4 and 6 to 10 are built (`crates/matching/src/naming`, the
`naming` area, the boundary and every binding, the messages, the guide
and the example, 2026-10-28; §10 says where the build reads the page);
item 5 is not. Part of it was already built inside naam milan and the
avakahada (`matching.md`, C291 to C299 and C334), and this page builds
on that cakra rather than beside it.

Nāmakaraṇa is the naming of a child. The baseline engine offers a
phonetic matcher, a name validator and an offline corpus of 25,096
lines (`01-research/baseline-engine/00-inventory.md`). The migration
guide lists all three as having no SDK equivalent (§10). This page
decides what the SDK offers in their place and what it declines.

## 1. What is built, and what is not

Built in `crates/matching` (`matching.md`, "The name syllable and the
avakahada, decided"):

- **The cakra.** *Svarodaya*'s śatapada cakra generates the 112 cells.
  The table that *Muhurta Chintamani*'s commentary prints is held to the
  rule (C291, C299).
- **Reading a name.** `name_syllable(name, NameRules)` reads a name's
  first akshara, in Devanagari or declared IAST, into its cell, star,
  quarter and varga. It follows C293 and C294: a conjunct by its first
  consonant, vowel length folded, ṛ read as i, b read as v and ś as s.
- **The birth syllable.** `birth_syllable(nakshatra, pada)` gives the
  syllable for one of the 108 padas (C297, C298).
- **Checking a name.** `name_check(name, rules, nakshatra, pada)`
  reports facts and no score (C334).
- **On a chart.** `chart.avakahada` carries the Moon's syllable in
  every binding.
- **Naam milan.** `sdk.matching().naam(bride, groom, rules)` reads two
  names with this cakra (C296).

Not built:

- The month name and the star's taddhita, which wait on the page
  images (item 5).
- No name corpus exists, by decision (§3): the consumer's book is
  searched.

## 2. What the sources decide

### 2.1 The syllables of a pada: one table, three witnesses

| witness | where | what it gives |
|---|---|---|
| *Muhurta Chintamani* with *Pīyūṣadhārā*, 1954 print, saṃskāra-prakaraṇa v. 11 commentary | p. 173 (leaf n185) | Jyotirarka's verse "cū ce co lā pade 'śvibhe …", four syllables to each of 28 stars with Abhijit; the name to begin with the letter of the birth pada, after the *Gṛhyapariśiṣṭa*'s "tad-akṣarādikaṃ nāma" (C297) |
| *Narapati Jayacharya Svarodaya*, śatapada cakra vv. 1–8, 1946 Khemraj print | pp. 73–74 (leaf n83) | the rule the table follows: twenty row letters, five vowels and four pillars, 112 cells (C291) |
| Himal Panchanga VS 2082 (Nepal), "avakahaḍā-cakra-sahitaṃ vara-vadhū-melāpaka-cakram" | printed p. 9 | the row "caraṇāgata nāmādyakṣara": 28 columns with Abhijit (ju je jo kha) placed in Makara |

The patro's row agrees with the cakra cell for cell once vowel length
is folded. It prints कु, रु, गु and जु where *Muhurta Chintamani* prints
long vowels, which C299 already treats as spelling. So for the
Devanagari syllable of a pada, the classical rule and a printed Nepali
patro **do not disagree**, and the built table stands. The patro is cited
here, not copied: the table is generated from *Svarodaya*'s rule.

The patro's yearly rashifal (printed pp. 7–8) heads each sign with its
syllables. The headings show three things the table above does not:

- **Three glosses (C770).** Mithuna "(ङ=ग)", Kanya "(ण=ड)" and Meena
  "(ञ=ज)". ङ, ण and ञ are cells of the cakra (Ardra's 3rd pada, Hasta's
  3rd and Uttara Bhadrapada's 4th), and no name begins with any of them.
  The patro replaces each nasal with the third letter of its own varga.
  *Muhurta Chintamani*'s commentary is silent on this. *Svarodaya*'s
  sarvatobhadra v. 22 pairs ṅa with ña, which leaves both unusable.
- **Makara has thirteen syllables (C771).** Its heading includes
  Abhijit's ju, je, jo and kha. The verse instead gives each sign "nine
  padas", 108 in all.
- **Bare consonants.** The headings print क, ह, च and ख for kā, hā, cā
  and khā, so a reader's eye takes the inherent a. Vowel length is
  folded anyway, so this changes no cell.

**A Hindi-region printed table was not read.** The variants commonly
reported (ब written for व in Rohini and Mrigashira, श beside स) do not
change a cell under C294, which reads b as v and ś as s. They change
only how the prescribed syllable is spelled, and §4.1 answers every
initial that reaches a cell. The table is still to be obtained, cited
only (§9).

### 2.2 A child has four names, not one (C773)

*Dharmasindhu*, third pariccheda, pp. 163–164, under nāmakaraṇa, names
**four kinds of name**:

1. **Devatā-nāma.** The family deity's name. This is the family's own,
   so the SDK has nothing to compute.
2. **Māsa-nāma.** A name of Viṣṇu for the lunar month of birth (§2.4).
3. **Nākṣatra-nāma.** *Dharmasindhu* gives three readings:
   - its own: a taddhita formed from the star's name, which it lists
     for all 28 (Āśvayuja, Āpabharaṇa, Kārttika, Rauhiṇa, … Ābhijita,
     … Raivata);
   - "some" (*kecit*): the avakahada syllable, by "cū ce co lā", giving
     Cūḍāmaṇi, Cedīśa, Coleśa and Lakṣmaṇa for Ashvini's four padas;
   - the Śāṅkhāyanas': a name after the star's deity (Āgniśarman for a
     Krittika birth).

   It calls the śrauta and gṛhya form, the taddhita, the more widely
   accepted (*bahusaṃmatam*). It also says the star name is the one used
   to greet a teacher and is known only to the parents until the sacred
   thread. Āśvalāyana GS I.15.8 has the same secrecy (Oldenberg, SBE 29,
   p. 183). *Muhurta Chintamani*'s commentary (p. 173) reports "janma-nāma
   tu gopayet": some keep the star name secret and give another name for
   use.
4. **Vyāvahārika-nāma.** The name in daily use, judged by the clauses of
   §2.3, not by a star.

So the syllable the SDK already answers is one reading of one of the
four names. It is the reading *Muhurta Chintamani*'s commentary
prescribes and the one a patro printed in Nepal prints, so it stays the default.
The taddhita and the deity are reported beside it, never instead of it.
A name a parent gives for use is checked against the clauses of §2.3.
Checking it against the star is still possible, and the answer says that
the texts do not ask for it.

### 2.3 The name in use: clauses, not a verdict (C774)

The gṛhyasūtras give the shape of the name in use. The SDK reports each
clause with its source and gives no verdict
(as `matching.md` does for the kootas).

| clause | source |
|---|---|
| begins with a sonant (*ghoṣavat*): the 3rd, 4th or 5th letter of a varga, or h | Pāraskara GS I.17.2, quoted in *Muhurta Chintamani*'s commentary p. 173; Āśvalāyana GS I.15.4 (SBE 29 p. 182); *Dharmasindhu* p. 164 |
| a semivowel (y r l v) within | the same three |
| ends in a visarga (*abhiniṣṭhāna*) | the same three |
| a kṛt, not a taddhita (for a boy) | PGS I.17.2; *Dharmasindhu* p. 164; **not read**: a fact of grammar the letters do not show |
| two or four syllables (two for steadiness, four for holy lustre) | PGS I.17.2; AGS I.15.5–6; *Dharmasindhu* p. 164 |
| an even count for a man, odd for a woman, "in every case" | AGS I.15.7; *Dharmasindhu* p. 164, who keeps this one when the others cannot all be met |
| a girl's: odd, ending in ā, a taddhita | PGS I.17.3 (SBE 29 p. 297); *Muhurta Chintamani*'s commentary p. 174; the count and the ending read, the taddhita not |
| not ending in l or r | Āpastamba and Hiraṇyakeśin, reported by *Dharmasindhu* p. 164 |
| ends in śarman or deva, varman or rāja, gupta or datta, or dāsa, by varṇa | PGS I.17.4 (SBE 29 p. 298); *Dharmasindhu* p. 164 |

**A syllable is a vowel.** *Dharmasindhu* says *akṣaram aj svaraḥ;
vyañjaneṣu na saṅkhyāniyamaḥ*: a syllable is counted by its vowel, and
consonants are not counted. So राम is two (rā-ma). Hindi and Nepali
speech drop the final inherent a ("rām"), and a consumer may want the
spoken count. The knob `final_a` is `SOUNDED` by default (the text) or
`SILENT` (speech), and the answer says which it counted.

**Where the sources part.**

- Only Pāraskara gives a girl's name the ā ending and the taddhita.
- Only Āpastamba and Hiraṇyakeśin forbid a final l or r.
- The varṇa endings are a rule about the family, not the name.

So each clause carries its source, and the knob `sutra` (`PARASKARA` by
default, since *Muhurta Chintamani*'s commentary quotes it, or
`ASHVALAYANA`) picks the set reported. The varṇa endings are reported
only when the request names a varṇa. One clause is garbled in the
*Dharmasindhu* OCR ("ṛṭavarṇarahitam") and is not built until it is
read on the page image.

### 2.4 The month name (C775)

*Muhurta Chintamani*'s commentary (p. 174) quotes Vasiṣṭha.
*Dharmasindhu* (p. 163) gives the same list from Caitra on, and states
the reckoning: the months are lunar and *amānta*, running from the
bright fortnight to the end of the dark (*śuklādi-kṛṣṇāntāḥ*).

| month | name (Dharmasindhu) |
|---|---|
| Chaitra | Vaikuṇṭha |
| Vaishakha | Janārdana |
| Jyeshtha | Upendra |
| Ashadha | Yajñapuruṣa |
| Shravana | Vāsudeva |
| Bhadrapada | Hari |
| Ashvina | Yogīśa |
| Kartika | Puṇḍarīkākṣa |
| Margashirsha | Kṛṣṇa |
| Pausha | Ananta |
| Magha | Acyuta |
| Phalguna | Cakrī |

The two prints differ in two places:

- **The sixth name.** The 1954 *Muhurta Chintamani* OCR reads the sixth
  as "-स्िविक्रमः", most likely Trivikrama where *Dharmasindhu* has Hari.
- **The twelfth name.** *Muhurta Chintamani* has Cakradhārī and
  *Dharmasindhu* Cakrī. The meaning is the same and the spelling
  differs.

**The default is *Dharmasindhu*'s list.** It is the nibandha on the
saṃskāras, it reads cleanly, and it states the reckoning. The knob `masa_names`
(`DHARMASINDHU` by default, or `MUHURTA_CHINTAMANI`) gives the other.
*Muhurta Chintamani*'s value is not built until p. 174 is read on the
page image (leaf n186).

**The adhika month.** Neither text names a month name for an adhika
month (in Nepal, *malamāsa*). The default answers the name of the month
it is adhika to, flagged `adhika`. The knob `adhika` is `SAME_NAME` by
default or `REFUSE`.

The names are a fixed table of twelve over an existing kind, so they
become a `masa` attribute in the catalogue rather than a list in code
(one table per fact).

### 2.5 Reading a Latin name (C772)

C293 refuses to guess an English spelling, because English "ch" is च
while IAST's ch is छ. That stays the default. A consumer with names
typed in Latin then has two choices: refuse them, or fall back to the
baseline engine's fuzzy matcher, which picks one reading silently. Both
are dead ends, so the SDK adds two more ways in.

**1. Declared schemes.** These are read exactly:

- `IAST`, built;
- `ISO_15919`, which is IAST with ṁ, r̥, ē and ō;
- `HARVARD_KYOTO`;
- `ITRANS`.

Each is a transliteration table. The tables live in one place, beside
`teistro_intl::translit`, and the reader does not keep a second copy:
`name.rs`'s `IAST_CONSONANTS` is folded into that table. A Latin name is
transliterated to Devanagari and then read by the one Devanagari reader.
Combining marks are composed by a short table of the scheme's own pairs
(a and U+0304 to ā, and so on), so decomposed input reads the same as
precomposed input, with no Unicode dependency.

**2. `CASUAL`, for everyday spellings.** It reads **every Devanagari
spelling the letters could stand for** under one declared table, and
answers the distinct cells with the spelling that reaches each. It never
chooses between them. The table is a list (a list, not an inference):

| Latin | read as |
|---|---|
| ch, chh | च, छ |
| t, th | त, थ, ट, ठ |
| d, dh | द, ध, ड, ढ |
| n | न, ण |
| sh, shh | श, ष |
| s | स, श |
| kh | ख, ष (Nepali speech says ष as kha) |
| ri, ru | रि, ऋ |
| gy, gya | ग्य, ज्ञ (Nepali says ज्ञ as gya) |
| ksh, x | क्ष |
| v, w, b | व, ब |
| z | ज़ |
| f | फ़ |
| q | क़ |
| aa, ee, oo, ii, uu | ā, ī, ū, ī, ū |
| ai, au | ऐ, औ (refused by C293) |

Vowel length folds and the cakra is coarse, so most names reach one
cell in spite of the ambiguity:

- Krishna is कृ or क्रि, both of which read as कि (Mrigashira's 4th).
- Shiva is शि or षि: the first reads as सि (Shatabhisha's 3rd), the
  second as ष (Hasta's 2nd). Two cells, both answered.

`CASUAL` reports `ambiguous` whenever the cells differ.

**Naam milan** with an ambiguous name is refused under
`naam.bride` or `naam.groom`, and the hint lists the spellings, since a
match needs one star. The same name is answered whenever every reading
falls in one pada.

The default stays `REFUSE`, so no answer that is built today changes.

## 3. The name book: what ships, and what the consumer brings

**The SDK ships no name corpus,** in the core or in the wasm module, and
the consumer brings their own list.

- **The baseline engine's corpus.** It has 25,096 lines, and its
  provenance is not recorded in this repository, so it is not
  relicensed by being copied here (an open question, §9). A migrating
  consumer loads it as their own book, and the suggestions then come
  from the names they already had.
- **Wikidata (CC0).** Measured 2026-10-28 (recorded, not gated): the
  given names that carry a Devanagari native label are **350**. Of
  these, 336 read in the cakra and 14 are refused (ai or au initials).
  They fill **56 of the 108 birth padas**, and 48 sit in the first cell
  (a). That is too thin to suggest from: half the births would get
  nothing.
- **Public-domain texts the SDK could transcribe itself.** The
  candidates are the sahasranāma epithets (for example Viṣṇu's, in the
  Mahābhārata's Anuśāsana Parva) from a public-domain print. Online
  editions are excluded: Wikisource is CC BY-SA (copyleft), and the
  terms of GRETIL and sanskritdocuments are unclear. An epithet is not
  always a given name, so every entry would need a vetter.
- **Modern name books** (in Nepali, Hindi or English) are in copyright.
  They are cited only, never used.

**Decided:** the SDK offers the search and the consumer supplies the
names. A CC0 book may be published later as its own artefact, never
inside the module. It would be built from Wikidata and the SDK's own
transcriptions, and **only vetted entries** go in: a reviewer for the
tradition (GOVERNANCE.md) confirms each spelling and sex. Meanings are
the book's own, never machine-written. The month and star names of
§2.2 and §2.4 are rule data, not a corpus, and live in the catalogue.

A book is JSON: `{"names": [{"devanagari", "latin"?, "sex"?, "tags"?}]}`.

- `devanagari` is required, because the cakra reads Devanagari.
- `latin` is a display spelling, never read.
- `sex` is a catalogue `sex` key.
- `tags` is an open string map, for example a meaning in a language,
  which the SDK carries and never reads.

A book is loaded once into the context, `sdk.naming().load(book) ->
BookId`, and indexed by cell at load, because a 25,000-name book read
on every call is the setup a call repeats (`batch-and-parallelism-measured.md`). A request may instead carry a book inline, for a book of one
use.

## 4. The surface

`naming` is a new area of the surface (`surface-areas.md`). It reads no
sky, like `matching` and `numerology`. The kernel stays in
`crates/matching` beside the cakra, in a `naming` module. The pieces
built there move into it unchanged.

### 4.1 Types

```rust
/// How a Latin name is read (C293, C772). Extends the built enum.
pub enum LatinName { Refuse /* default */, Iast, Iso15919, HarvardKyoto, Itrans, Casual }

/// A prescribed letter no name begins with (C770).
pub enum NasalInitial { VargaThird /* default: the patro's ङ=ग, ण=ड, ञ=ज */, None }

/// The rules a name is read and judged under; each default is the one no source read contradicts.
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct NamingRules {
    pub name: NameRules,          // latin, abhijit (built)
    pub nasal: NasalInitial,
    pub sutra: GrhyaSutra,        // PARASKARA | ASHVALAYANA (C774)
    pub final_a: FinalA,          // SOUNDED | SILENT (C774)
    pub masa_names: MasaNames,    // DHARMASINDHU | MUHURTA_CHINTAMANI (C775)
    pub adhika: AdhikaName,       // SAME_NAME | REFUSE (C775)
}

/// What syllables are asked for.
#[serde(tag = "of", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SyllablesOf { Pada { nakshatra: Nakshatra, pada: u8 }, Nakshatra { nakshatra: Nakshatra }, Rashi { rashi: Rashi } }

/// One cell of the cakra as a name may begin with it. Generalises `BirthSyllable`.
pub struct PadaSyllable {
    pub cell: u8,
    pub nakshatra: Option<Nakshatra>,    // None for Abhijit
    pub pada: Option<u8>,                // None for Abhijit, which is no pada (C298)
    pub rashi: Rashi,                    // Makara for Abhijit, whichever placement (C771)
    pub devanagari: String,              // the print's spelling (C299)
    pub iast: String,
    pub varga: NameVarga,
    pub initials: Vec<Initial>,          // every letter that reaches this cell
}
pub struct Initial { pub devanagari: String, pub iast: String, pub by: InitialBy }
pub enum InitialBy { Cell, Paired /* C294: ब for व, श for स */, Substitute /* C770 */ }

/// A name read: one entry per cell its spelling can reach (one, unless CASUAL is ambiguous).
pub struct NameReading { pub readings: Vec<Reading>, pub ambiguous: bool }
pub struct Reading {
    pub spelled: String,                 // the Devanagari it was read as
    pub syllable: NameSyllable,          // built: cell, nakshatra, quarter, varga
    pub rashi: Rashi,
    pub clauses: NameClauses,
}
pub struct NameClauses {
    pub syllables: u8,
    pub counted: FinalA,
    pub rows: Vec<ClauseRow>,            // { clause: Clause, holds: bool, source: Sutra }
}

/// A name against a birth: the built NameCheck plus the substitute and the clauses.
pub struct NameCheck { pub read: NameSyllable, pub prescribed: PadaSyllable,
    pub same_star: bool, pub same_pada: bool, pub by_substitute: bool, pub clauses: NameClauses }

/// The four names of Dharmasindhu (C773, C775), for one birth.
pub struct Names {
    pub syllable: PadaSyllable,                  // the avakahada reading (the default)
    pub nakshatra_name: StarName,                // the taddhita (Dharmasindhu's list), Devanagari and IAST
    pub deity: Deity,                            // the Śāṅkhāyana reading: the star's deity key
    pub masa_name: Option<MasaName>,             // when the birth's month is known
}

pub struct Suggested { pub index: u32, pub reading: Reading, pub by: InitialBy, pub same_pada: bool }
```

All the enums are closed and cross the boundary as numbers, with their
keys in the IDL, as `NameVarga` already does. No new catalogue kind is
spent, because a name is the consumer's, not something a consumer names
from the SDK (a kind supplies a key space, `core-types-and-catalogue.md`). The new catalogue facts are two
attributes on existing kinds:

- `masa.vishnu_name`, two rows a member, one for each source;
- `nakshatra.nakshatra_name`.

Both are transcribed from the page images (§8), with `sources` on each
member.

### 4.2 Operations

| Rust | binding | answers |
|---|---|---|
| `sdk.naming().syllables(of, rules)` | `naming.syllables` | 1, 4 or 9 `PadaSyllable`s in cakra order; Makara's 9 plus Abhijit's 4, marked by `pada: None` (C771) |
| `sdk.naming().read(&[names], rules)` | `naming.read` | a `NameReading` or a refusal a name, the batch never failing as a whole |
| `sdk.naming().check(&[names], birth, rules)` | `naming.check` | a `NameCheck` or a refusal a name |
| `sdk.naming().of(birth, rules)` | `naming.of` | `Names` |
| `sdk.naming().load(book)` / `suggest(book, birth, filter, rules)` | `naming.load`, `naming.suggest` | the book's names whose cell is the birth pada's or reaches it by an initial, in cakra order then book order, each with its clauses; the names the book holds that were refused, counted and listed |

`birth` is `{nakshatra, pada, masa?, adhika?}`. A chart gives it with
`ChartRequest::with_naming`, which fills `Document::naming` from the
Moon's pada, and from the birth's lunar month when the chart's
panchanga is asked. The fill reads the avakahada's own native, so the
two cannot disagree. `filter` holds `sex`, `syllables`, and `clauses`
that must hold. A filter never ranks, and the order is the cakra's.

At the boundary, each request is read strictly (`deny_unknown_fields`,
with refusals named under `naming.*`). It is answered as a blob in the
shape `naam` set:

- `naming_syllables`, one row per cell, with the initials ragged under
  it;
- `naming_reads`, a count per name and the readings ragged under it;
  a refused name is a row holding its status and field;
- `naming_checks`;
- `naming_names`;
- `naming_suggestions`.

The text crosses as canonical JSON, as `avakahada_syllables` does.
Every binding gets the area by generation from the IDL, with no
hand-written copy (no hand-written copy of what the IDL describes). The MCP server gets
`naming_*` tools from the same description. `naam_milan` calls the
same reader, so a name match and a name check read one cell.

### 4.3 Refusals

Each refusal is `INVALID_ARG` with a field and a hint:

| field | when | hint |
|---|---|---|
| `name` | the name has no first letter, begins with a letter the cakra does not read (ai, au, ḷ, the borrowed vowels), or is Latin while `latin` refuses | the built hints |
| `name` | a Latin name under a scheme its letters do not fit | names the letter and the scheme |
| `pada` | a pada outside 1 to 4 | — |
| `abhijit` | a pada asked of Abhijit's row in `check` | built |
| `masa`, `adhika` | an adhika month while `adhika` refuses | names the knob |
| `book.names[i].devanagari` | a book name with no Devanagari | — |
| `book.names[i].sex` | a sex that is not a key | — |
| `naam.bride`, `naam.groom` | a CASUAL name ambiguous across padas | lists the spellings |

### 4.4 Intl

- **The syllables cross in the print's Devanagari and in IAST.** The
  `en-Latn` locale reads the IAST. A casual Latin display ("chu, che,
  cho, la") is not generated. If a locale wants one, it is a vetted
  form in that locale's files, never machine-stubbed.
- **The star and month names** are `sa-Deva` forms. `sa-Latn` is
  derived from them by `teistro-intl derive`, as every entity's is.
  `ne-Deva-NP` and `hi-Deva-IN` carry them only once vetted.
- **The new messages** (each clause's name and the refusals) go into
  `en-Latn` and `ne-Deva-NP` together, because a strict locale binds
  the base. The Nepali is handed to a speaker, not guessed.
- **A book's meanings and display spellings are the consumer's.** The
  SDK renders none of them.

### 4.5 Determinism and size

- **Determinism.** Every answer is a pure function of the request: no
  float, no clock, no locale-dependent folding. Case folding is
  Unicode's own `to_lowercase`. The order is the cakra's, then the
  book's. A book is hashed into its `BookId` canonically, so the same
  book loaded twice is one book.
- **Size.** No corpus enters the module. The new data is the scheme
  tables (a few hundred entries), the casual table (about 20 rows) and
  two catalogue attributes (40 short strings in two scripts, under
  2 KB raw). The code is the area's five functions and their blob
  writers. The build measures the module gzipped against
  `bindings/wasm/size.json` and records raw and compressed bytes
  (`wasm-binding.md` §7).
- **For scale.** The Wikidata list above is 1.7 KB gzipped. The
  baseline engine's corpus, at its line count, would be several
  hundred KB, which is why no corpus goes in the module.

## 5. Tests the build must have

- **Every cell, both ways.** Each of the 112 cells' `PadaSyllable`
  reads back to its cell through `naming.read`, and every one of its
  `initials` reads back to it too (C294 pairs). A substitute reaches it
  only through `by_substitute`.
- **The sign's syllables.** Every rashi's `syllables` is exactly its
  nine padas (108 in all, no cell twice), plus Abhijit's four for
  Makara alone. The union over the twelve is the 112.
- **The worked examples of the texts:**
  - *Dharmasindhu*'s Cūḍāmaṇi, Cedīśa, Coleśa and Lakṣmaṇa read to
    Ashvini's padas 1 to 4;
  - its Deva and Hari meet every vyāvahārika clause;
  - Rudra fails the visarga and meets the parity, which it gives as the
    clause to keep;
  - Āgniśarman stands for Krittika's deity;
  - Vasiṣṭha's twelve month names in order.
- **The patro's glosses.** A birth in Ardra 3, Hasta 3 or Uttara
  Bhadrapada 4 lists ग, ड or ज as a `Substitute` initial under the
  default, and none under `NONE`. A name beginning with ग still reads
  as Dhanishtha's or Shatabhisha's, never Ardra's.
- **The schemes.** One name in IAST, ISO 15919, Harvard-Kyoto, ITRANS,
  precomposed and decomposed, reads to the cell its Devanagari reads
  to.
- **CASUAL.** Its table is held to the design's list. Krishna is
  unambiguous. Shiva is ambiguous between Shatabhisha 3 and Hasta 2.
  "Gyan" is ambiguous between Dhanishtha 1 and Uttarashadha 3. Naam
  milan refuses an ambiguous name and answers an unambiguous one.
- **Counting.** राम is 2 `SOUNDED` and 1 `SILENT`. A conjunct counts
  once. A final virama counts nothing.
- **Unchanged answers.** Every default reproduces the built answers:
  `name_check` and naam milan answer as before on their existing tests.
- **The book.** A book of names in every cell suggests each in its own
  cell's request and nowhere else. A refused entry is listed, not
  dropped. Loading the same book twice gives one `BookId`. A book of
  25,000 synthetic names loads and suggests within the time budget the
  build measures.
- **Every binding.** Each binding runs the same requests and gets the
  same answers (parity across the five runners), including one refusal.
  The MCP tool answers the same blob.

## 6. Cruxes

C770 to C775 in `01-research/feature-universe/19-verification-cruxes.md`.
C291 to C299 and C334 stand as decided.

## 7. What the migration guide maps

| baseline | SDK |
|---|---|
| the phonetic matcher | `naming.read` with a declared scheme or `CASUAL`, which reports every reading instead of choosing one |
| the name validation | `naming.check`: the cell, `same_star`, `same_pada`, `by_substitute`, the clauses; facts, not a score (C334) |
| the offline corpus | the consumer's own book, `naming.load` and `naming.suggest` (§3) |

## 8. The build, in work items

1. **One transliteration table** for IAST, ISO 15919, Harvard-Kyoto and
   ITRANS, with the composition pairs. The Latin reader becomes
   "transliterate, then read Devanagari", and `name.rs`'s own IAST table
   goes.
2. **`CASUAL`**: the list, the candidate reader and `ambiguous`. Naam
   milan refuses an ambiguous name.
3. **`PadaSyllable`, `initials` and `syllables(of)`**, with C770's
   substitute and C771's Makara. `birth_syllable` becomes a view of it.
4. **The clauses and the syllable count**: C774's table as rows, each
   naming its source.
5. **The catalogue.** Read *Dharmasindhu* pp. 163–164 and *Muhurta
   Chintamani* p. 174 on the page images, then add `masa.vishnu_name`
   and `nakshatra.nakshatra_name` with their sources (C773, C775).
6. **`check` and `of`**, extended, and `ChartRequest::with_naming`.
7. **The book**: the format, `load` with the cell index and the
   `BookId` hash, and `suggest`.
8. **The boundary**: the strict request types, the IDL, the five blobs,
   the FFI and the MCP tools.
9. **The bindings** generated, with parity tests in the five runners.
10. **The messages** in `en-Latn`, and in `ne-Deva-NP` once vetted.
    Also the docs, an example, the migration guide, the CHANGELOG, and
    the wasm size measured.

Items 1 to 4 are independent of the sources still to be read. Item 5
waits on two page images. Items 8 and 9 are the usual mechanical half.

## 9. Open questions

- **The page images.**
  - *Dharmasindhu* pp. 163–164: the 28 taddhita names and the garbled
    clause.
  - *Muhurta Chintamani* 1954 p. 174 (leaf n186): the sixth month name.
- **A Hindi-region printed table**, cited only, to confirm that no cell
  differs from the patro.
- **Gobhila GS II.8.** A secondary report gives girls' names ending in
  -dā there, which would be a third reading of the girl's clause. Not
  read.
- **The baseline engine's corpus.** Where its 25,096 lines came from,
  and whether they may be released under the SDK's licence. Until that
  is known, it migrates only as a consumer's own book.
- **A published CC0 book.** Whether to publish one at all. It needs a
  vetter, and the measured CC0 source covers half the padas.

## 10. Build state

Built 2026-10-28, in Rust only (no boundary, IDL, binding or MCP tool
yet):

- **Item 1.** `teistro_intl::translit` holds one table with a column for
  each of IAST, ISO 15919, Harvard-Kyoto and ITRANS, and
  `to_devanagari(text, Scheme)` reads any of them back, composing the
  scheme's decomposed marks. `devanagari_to_iast` writes from the same
  table, and `name.rs`'s IAST table is gone. A spelling two rows share
  reads as the earlier row (ṛ as the vowel, not ड़), and IAST does not
  read back ढ़'s `ṛh`, which would take the `ṛh` of *bṛhaspati*. ITRANS's
  `x`, `GY` and `dny` are listed conjuncts. ISO 15919 writes ए as ē, so
  a plain `e` is refused under it, naming the letter.
- **Item 2.** `LatinName::{Iso15919, HarvardKyoto, Itrans, Casual}` and
  the `CASUAL` table (§2.5's rows, plus each plain letter listed). Only
  the first akshara is read every way; the rest of the name takes each
  spelling's first reading, and a word's last consonant keeps its `a`
  as Nepali and Hindi write it. `read_name` answers a `NameReading`.
- **Item 3.** `PadaSyllable`, `Initial`, `InitialBy`, `NasalInitial`,
  `SyllablesOf` and `syllables(of, rules)`; `birth_syllable` is a view of
  a `PadaSyllable`, and its answer is unchanged.
- **Item 4.** `name_clauses(spelled, rules)` answers `NameClauses`, with
  `GrhyaSutra` and `FinalA` on `NamingRules`.
- **Item 6.** `name_check(name, Birth, NamingRules)` adds `prescribed`
  as a `PadaSyllable`, `by_substitute` and `clauses`; `names_of(Birth,
  NamingRules)` answers `Names`.
- **Item 7.** `Book` and `BookName` are §3's JSON, read strictly.
  `NameBook::load(Book)` reads every name once into its cell, indexes the
  names by cell in the book's order, and keeps a `BookId`: the SHA-256 of
  the book's canonical JSON (`teistro_core::envelope::content_hash`).
  `NameBook::suggest(Birth, &SuggestFilter, NamingRules)` answers
  `Suggestions`: the `BookId`, the birth pada's `PadaSyllable`, the
  `Suggested` names in cakra order and then the book's, and every refused
  name.

Where the build reads the page, it says so here:

- **`ambiguous` counts readings, not only cells.** `CASUAL` reads b, v
  and w as व and as ब, one cell and two vargas, and the varga koota of
  naam milan reads the varga. So a reading is a distinct `NameSyllable`
  (cell and varga), `ambiguous` is more than one, and naam milan and
  `name_check` refuse any name with more than one reading (under
  `bride.name`, `groom.name` or `name`), the hint listing the spellings.
- **`ru` also reads as रु**, its plain reading, which the table's row
  ("ri, ru | रि, ऋ") leaves unsaid; without it "Rupa" could only be ऋपा.
- **The final l or r is the name's last sound**: a ल or र under a virama,
  or bare when `final_a` is `SILENT`. Read as the last akshara's
  consonant, *Dharmasindhu*'s own Hari would fail it.
- **Āpastamba's and Hiraṇyakeśin's clause is reported under both
  sutras**, sourced to them, since it is neither sutra's. The sonant is
  read as the table gives it (a varga's 3rd, 4th or 5th letter, or h),
  so a vowel or a semivowel first does not meet it.
- **Not built with item 4: the varṇa endings.** A request names a varṇa
  only at the boundary (item 8), and matching an ending (śarman, śarmā)
  is a rule to decide with it.
- **Not built with item 6.** `Names` has no `nakshatra_name` or
  `masa_name` and `NamingRules` no `masa_names` or `adhika`, and `Birth`
  has no month: all wait on item 5's page images. `ChartRequest::with_naming`
  and the `sdk.naming()` area are the boundary's (item 8).
- **Names.** The Rust functions are `read_name`, `name_check`,
  `names_of` and `syllables`, so a crate-root name says what it reads;
  the area's operations keep §4.2's names.
- **The book's two operations are methods.** `load` and `suggest` are
  `NameBook::load` and `NameBook::suggest`, so the crate root gains the
  type and not two bare verbs. The context that holds books by `BookId`,
  and a book carried inline in a request, belong to the boundary
  (item 8).
- **A book's names are read as Devanagari under every rule.** Its
  `devanagari` is read with `latin: REFUSE` at load, so the index does
  not depend on a request's rules. A suggestion recomputes each name's
  clauses under the request's `sutra` and `final_a`, and the request's
  `nasal` decides the substitutes.
- **"No Devanagari" means no letter of the Devanagari block.** An
  empty, blank or Latin-only `devanagari` refuses the whole book under
  `book.names[i].devanagari`. A name that holds Devanagari but that the
  cakra refuses (an ai or au initial, or a Latin letter first) is kept:
  it is listed in `refused` with the reader's refusal, renamed to
  `book.names[i].devanagari`, and every `suggest` answer carries the
  list.
- **`sex` is typed.** `BookName::sex` is an `Option<Sex>`, so a key that
  is not one fails when the book is read. At the boundary the strict
  reader names it `book.names[i].sex` (item 8). Rust never holds such a
  key.
- **A `BookId` hashes the book as given.** Nothing is trimmed and no
  Unicode normalisation is applied, since the area has no Unicode
  dependency (§4.5). The JSON's spacing and key order do not move it.
  Any change to a name, the order, a `latin`, a `sex` or a tag does. A
  name the book lists twice is kept twice, and each copy is suggested at
  its own index.
- **The filter.** `SuggestFilter { sex, syllables, clauses }`, read
  strictly.
  - `sex` keeps names of that sex and names the book gives no sex: a
    name the book leaves open is anyone's.
  - `syllables` is a list of counts, any of which passes, counted as
    `final_a` says. Empty keeps every count. A list, because §2.3's own
    clause is "two or four".
  - `clauses` must all hold. A clause the rules' sutra does not report
    (`EVEN_FOR_A_BOY` under Pāraskara, `LONG_A_FOR_A_GIRL` under
    Āśvalāyana) is refused under `filter.clauses` rather than matching
    nothing.
- **What a suggestion searches.** It reads the pada's own cell, then
  every cell whose letter is a substitute the pada lists. ग, ड and ज are
  row heads only, so a name beginning with one reads in its row's cells,
  whatever its vowel. ज's row runs into Abhijit's, so Uttara Bhadrapada's
  4th pada also reaches names in जू, जे and जो. `by` is read from the
  pada's initials by the name's first consonant, and a vowel's cell is
  reached by `CELL`. `name_check`'s `by_substitute` now asks the same
  question. `same_pada` is false exactly for a substitute.
- **`Suggestions` carries `prescribed`.** The answer names the syllable
  it searched for, with its initials.
- **Scale.** A synthetic book of 25,000 names (each cell's syllable with
  a two-letter tail) loads, and all 108 padas are suggested from it, in
  one unit test in 1.8 s, unoptimised, under nextest. That is the
  measurement §5 asks for, recorded and not gated.
- **The bindings' `LatinName`.** Naam milan already crosses the
  boundary with `NameRules`, so its generated request types gain the
  four new keys; nothing else crosses yet.

Built 2026-10-28, items 8 and 9: the boundary and the bindings.

- **The area.** `sdk.naming()` answers `syllables`, `read`, `check`,
  `of`, `load` and `suggest`, the six operations of §4.2. `read` and
  `check` answer one `Result` a name, so a refused name never refuses
  the batch; `check` refuses the whole batch only for its birth's pada,
  which every name is read against. Every binding's area is the same
  six (`naming.syllables` … `naming.suggest` in the parity runners'
  shape lines).
- **The requests.** `NamingSyllablesRequest`, `NamingReadRequest`,
  `NamingCheckRequest`, `NamingOfRequest`, `NamingLoadRequest` and
  `NamingSuggestRequest`, each read strictly (`deny_unknown_fields` and
  `teistro_core::strict`) under the root `naming`. A field of the page's
  refusal table is named under it: `naming.birth.pada`,
  `naming.names[2]`, `naming.book.names[0].sex`,
  `naming.book.names[0].devanagari`, `naming.filter.clauses`. The root is
  the area's, as `naam` is naam milan's, so the page's bare `pada` and
  `book.names[i].sex` are these with `naming.` before them. A book's own
  refused names keep the kernel's `book.names[i].devanagari`, since they
  are rows of an answer, not a refusal of the request.
- **`syllables` takes `of` as a record:** `{"of": {"of": "PADA",
  "nakshatra", "pada"}, "rules"}`. Flattening the tagged `SyllablesOf`
  into the request would name a refusal inside it by the request's root
  (serde's `flatten`), and the strict reader could not say which field.
- **A book is named or carried.** `NamingSuggestRequest` has `bookId`
  and `book`, exactly one given: neither is refused under `naming.book`,
  both under `naming.bookId`, and an id the context has not loaded under
  `naming.bookId`, the hint saying to load it or carry it. Two fields
  rather than a union picked by its keys, so every binding's generated
  request reads it with no new shape.
- **The context keeps books.** A `Context` holds its loaded books by
  `BookId`, each kept once (the same book loaded again is the book
  already kept). It is the one thing a context gains after it is built,
  and only by `load`. Nothing unloads a book; a context's books go with
  it.
- **`naming.load` is not a record of the agent server.** The server's
  contexts are a cache that keeps no state between calls
  (`mcp-server.md`), so a tool cannot load a book for a later one. The
  server has the other five as tools (`naming.syllables`, `naming.read`,
  `naming.check`, `naming.of`, `naming.suggest`), and its suggestion
  carries its book. Every binding has `load`.
- **The answers cross as canonical JSON, not as blobs.** §4.2 named five
  blob sections. Every answer is nested records of text: a pada's
  spellings and its ragged initials, a reading's clause rows, a refusal a
  name, a book's own tags. A blob would carry each as a numeric column
  beside a section of JSON text, which every binding would then parse
  as well as decode. So each crosses as the JSON the façade's own type
  serialises to, as numerology's profile does, through one `ts_naming_*`
  entry point an operation (`crates/ffi/src/naming.rs`). The agent
  server answers the same bytes, which is what §5 asks of it.
- **A refusal inside an answer is the SDK's error as a record:**
  `{status, detail, message, field, hint, key}`, the one serde writes.
  Each binding reads it into a `NameRefusal`.
- **The answer types are hand-written in each binding**, as numerology's
  are: the generator's answer records (`idl::records`) do not render a
  catalogue member yet. The request types are generated
  (`requests.d.ts`, `_requests.py`, `requests.dart`, Java's `request`
  package). In Node a member is its full key (`nakshatra.ASHWINI`), as
  every Node answer gives it. `Reading` is `SpelledName` in the bindings,
  since Python's catalogue has a kind named `Reading`.
- **`ChartRequest::with_naming` is not built.** A chart's `avakahada`
  already holds the Moon's nakshatra and pada, which is the whole of
  `Birth` until the month (item 5) adds what the avakahada lacks. A
  `Document::naming` section now would copy the avakahada's two fields.
  So `Birth::from(&Avakahada)` reads the chart's own Moon, and the two
  cannot disagree. A binding passes `avakahada`'s `nakshatra` and `pada`
  as the birth. The section is built with the month.
- **A request's `bookId` is text.** The id is a SHA-256, whose schema
  pattern is sixty-four hex digits, and the generated deep examples write
  any text field as `x`. So the request holds it as text and reads it
  into a `BookId` when it answers: text that is no id is refused like an
  id this context never loaded, as `naming.bookId`, with a hint saying
  what an id is. The answer's `book` stays a `BookId`.
- **One schema name a type across the agent server's tools**
  (`crates/mcp/tests/facade.rs`). Three names collided once naming had
  tools: `Clause` (prashna's), `Reading` (Pancha Pakshi's) and the
  catalogue's `Sex` and `Deity` beside rectification's and the ishta
  devata's own. The naming types are named at their source:
  `NameClause` and `SpelledName`. The ishta devata's deity is
  `IshtaDeity`, an answer's name only. Rectification's `Sex` was the
  catalogue's two keys under a second type, so it is now the catalogue's
  own (`pub use`): the same `MALE` and `FEMALE` on the wire, a request
  may also send the full key, and Rust code that matched on it needs a
  wildcard arm, since the catalogue's enum is `non_exhaustive`.

Built 2026-10-28, item 10: the messages, the guide and the example.

- **Seven messages, `sdk.naming`, in `en-Latn` and `ne-Deva-NP`.** They
  give words to the keys an answer carries, which a consumer otherwise
  prints raw: `clause` (each `NameClause`), `holds` (met or not),
  `source` (each `ClauseSource`), `by` (each `InitialBy`), `counted`
  (each `FinalA`), `syllables` (a count) and `pada` (the syllable a
  pada names a child by, the star an entity slot). Each selects on the
  key as the wire spells it, the way `sdk.reading.lifeClass` does, since
  none of these is a catalogue kind a locale names (§4.4: the clause
  names and the vocabulary are messages, not entities). Every member has
  an arm of its own and the catch-all says "another clause", so a member
  added later reads as nothing in particular rather than as the last
  one. `crates/sdk/tests/naming.rs` reads the keys from the answers
  (every clause both sutras report, every `by` over the twelve signs,
  both counts) and fails on one that falls to the catch-all, that two
  keys share words, or that a strict locale falls back.
- **The Nepali** keeps the corpus's own terms: पाद for a pada (as
  `sdk.reason.panchanga.pada` has it), अक्षर for a syllable, the
  ordinals पहिलो to चौथो, the participle phrasing of `sdk.condition`.
  The new terms are the grammar's and the patro's: नामाक्षर for the
  syllable a name begins with, घोष वर्ण for a sonant, अन्तःस्थ वर्ण for a
  semivowel, विसर्ग, जोर and बिजोर for even and odd, and the sutras'
  Sanskrit names. They are flagged for the native review every Nepali
  message is (§4.4: "handed to a speaker, not guessed").
- **A refusal is not a message.** §4.4 named "the refusals" among the
  messages. No area's refusal carries a message key: `Error::with_key`
  has no caller outside its own tests, no binding's error reads a key,
  and every refusal's `message` and `hint` are English. Keying naming's
  alone would make it the one area whose refusals a binding could say
  in Nepali, through a record no binding reads yet. So the refusals stay
  as every area's are, and a localised refusal is a decision for every
  area at once (open, and not this page's).
- **The example is `naming`, in all five sets** (`crates/sdk/examples`,
  and Node, Python, Dart and Java), and prints the same lines in each:
  Ardra's third pada and its substitute, three names checked, `CASUAL`'s
  readings of Krishna and Shiva, a book loaded and suggested from, and
  one check said in both locales. It names no ephemeris, since it reads
  no sky; every other example names the built-in one. The guide is
  `site/content/docs/naming.mdx`, its programs cut from the Node example
  and its outputs what the examples print (`check-guides`,
  `check-parity`).
- **The migration guide** already maps the baseline's three to
  `ctx.naming` (items 8 and 9); nothing in it moved.
- **The wasm size, measured.** `check-wasm` on this Mac (rustc 1.99.1),
  with items 1 to 10 in: the full module is 12,794,384 bytes raw and
  2,988,137 gzipped, 0.7% under its 3,010,000 budget; the `panchanga`
  module is 1,484,646 gzipped, 1.7% under. Against the record
  (`docs/05-testing/sizes.json`, run 40643786235 on CI's Linux, before
  the area crossed the boundary) the full module grew 149,872 bytes raw
  and 24,083 gzipped (0.8%), inside verify's 2% drift. No budget moved:
  a budget moves only from a CI run's artefact, so the next verify run
  records these figures.

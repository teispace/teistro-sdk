# The lexicon: names to keys, dates in text, and claims checked (A2-3, SR-61, A2-4)

Status: `draft`, 2026-11-01: designed before code. A2-3 (the lexicon and
its reverse), SR-61 (dates found in free text) and A2-4 (claims checked
against the SDK) are one stack: the second and third read text through
the first.

The application reads text a person typed or an agent wrote, in Nepali,
Hindi or English, in Devanagari or in Latin letters, and wants to know
what in it the SDK can name. Three requests ask for three answers:

| request | entry point | answer |
|---|---|---|
| A2-3 | `intl.lexicon`, `intl.lookup` | every catalogue member's names in a locale, romanised forms too; and for a text, every span that may name a member, with every member it may name |
| SR-61 | `intl.dates` | every Bikram Sambat or Gregorian date in a text, as a span with each reading of it |
| A2-4 | `verify.claims` | each claim a text makes about an instant and a place, held, contradicted with the computed value, or unread with the reason |

None of them guesses. A span that may name two members answers both; a
date that reads two ways answers both readings; a claim the grammar
cannot settle is unread and says why.

## 1. The lexicon (A2-3)

### 1.1 What it lists

`intl.lexicon({locale, kinds})` lists, for every key of `sdk.entity` in
the base locale (the catalogue being the authority for every key,
`intl-engine-and-packs.md` §7), the forms the locale's chain gives it:

- the locale's own forms, `name`, `prose`, `short` and `iast`, each with
  the locale that answered (the locale itself or a fallback);
- `romanised`, derived: the `iast` form, or the name transliterated by
  `translit::devanagari_to_iast` where the locale gives none, written in
  the unaccented ASCII a person types (§1.2);
- for a Devanagari locale whose `termStyle` is `vernacular` (`ne-Deva-NP`,
  `hi-Deva-IN`), the romanised form again with its final inherent `a`
  dropped, since Nepali and Hindi do not say it (`मंसिर` is *Mangsir*, not
  *Mangsira*).

`kinds` narrows the list to some kinds (`["tithi", "nakshatra"]`); left
out, every kind is listed. A key no locale in the chain names is not
listed: the lexicon lists names, and *catalogued is not named*.

### 1.2 The romanised spelling

A person typing a Sanskrit or Nepali name in Latin letters uses none of
the schemes `translit` reads exactly (IAST, ISO 15919, Harvard-Kyoto,
ITRANS): they type *Ekadashi*, *Rohini*, *Shrawan*. The common spelling
writes IAST's letters as English spelling suggests them: ā ī ū as a i u,
ś and ṣ as *sh*, c as *ch*, ṛ as *ri*, the nasals ṅ ñ ṇ ṃ as *n*, ṭ ḍ as
t d, ḥ as *h*. It is the convention of the Library of Congress's
romanisation tables with their diacritics dropped, and of the Government
of Nepal's English spellings of the Bikram Sambat months (§2.2).

### 1.3 The reverse: text to keys

`intl.lookup({text, locale, kinds})` finds every span of `text` that may
name a member and answers each span with **every** member it may name.

Matching is on a **fold**, one function applied to the lexicon's forms
and to the text alike, so the two meet whatever the spelling:

1. Devanagari is transliterated to IAST (`devanagari_to_iast`), nukta
   letters folded onto their base and the chandrabindu dropped.
2. Letters lose their diacritics; ṛ is *ri*; every nasal is *n*; ś ṣ are
   *s*; the visarga, the avagraha, apostrophes and hyphens go.
3. The spellings a typist varies are made one: an `h` after a consonant
   goes (aspiration: *kh*, *sh*, *ch*, *th*, *dh*, *bh*), `w` and `v` are
   `b` (Nepali writes ब for व: बैशाख, वैशाख), `f` is `p` (फ as *ph* or
   *f*: *Falgun*, *Phalgun*), `z` is `j`, `y` after a consonant goes
   (*Jestha*, *Jyeshtha*), *ai* is *e* and *au* and *ou* are *o*, an `n`
   or `m` before a consonant is `n`, and a doubled letter is one
   (*Kattik*, *Ekaadashi*).
4. A final inherent `a` after a consonant goes.

So *Ekadashi*, *ekādaśī*, *Ekadasi* and एकादशी all fold to `ekadasi`. A
fold makes names collide that differ: that is the point, and the answer
holds every member a folded name meets. A form that folds to fewer than
two letters, and every `short` form, is left out of the reverse index:
`short` is an abbreviation (`सू`, `S11`) that would meet ordinary words.

The text is cut into words (runs of letters and Devanagari signs); a
name may span up to four words (*Purva Phalguni*, *Shukla Ekadashi*),
joined without spaces before folding, and the longest span that names
something wins at each place. A word that names nothing whole is tried
again without a postposition fused to it: Nepali writes `रोहिणीमा`,
`एकादशीको`, `सोमबारदेखि`; the suffixes tried are Nepali's `मा`, `को`,
`का`, `की`, `ले`, `लाई`, `बाट`, `देखि`, `सम्म`, `हरू` and Hindi's `में`, `का`,
`की`, `के`, `से`, `ने`, and their romanised spellings after a Latin word.

`locale` narrows the forms searched to that locale's chain, each member
by the first locale in the chain that names it (as `intl.lexicon` lists
it): under `ne`, *Monday* is not searched, since Nepali names Monday
`सोमबार`. Left out, every loaded locale's forms are searched, so a Hindi
word in a Nepali sentence, or an English one, is found.

Among the core kinds (graha, rashi, nakshatra, masa, vara, paksha) in
English, the fold meets three collisions, each a real homonym rather than
a fold's slip: *Jyeshtha*, *Magha* and *Shravana* each name a lunar
month and a nakshatra. A test lists them, so a new one is a decision.

A span's `start` and `end` count **Unicode scalar values** (code points),
not bytes and not UTF-16 units, and the span's own `text` is answered
beside them (C1092).

### 1.4 Built once

The lexicon and its reverse index are built the first time a locale (or
"every locale") is asked for, and kept on the engine, so one context
builds each once; loading a pack or setting an override forgets them, as
it forgets parsed messages. §5 has the measured cost.

## 2. Dates in text (SR-61)

### 2.1 What is read

`intl.dates({text, calendars})` reads dates written with a **day of the
month**:

| shape | examples |
|---|---|
| year, month name, day | `२०८२ कात्तिक १५ गते`, `2082 Kartik 15`, `2026 October 31` |
| day, month name, year | `15 Kartik 2082`, `31 October 2026`, `३१ अक्टोबर २०२६` |
| month name, day, year | `Kartik 15, 2082`, `October 31st, 2026` |
| year first, in figures | `2082/07/15`, `२०८२-०७-१५`, `2026.10.31` |
| year last, in figures | `15/07/2082`, `10/31/2026` |

Digits are Latin or Devanagari (U+0966 to U+096F); a year has four of
them; an English ordinal suffix (*15th*) and Nepali's `गते` after the
day are read. Between the parts stand spaces, commas, or one separator
used throughout (`/`, `-`, `.`); Nepali's `साल` may follow the year.

An era marker before or after the date is part of the span and settles
its calendar: `बि.सं.`, `वि.सं.`, `विक्रम संवत`, `BS`, `B.S.`, `VS`,
*Bikram Sambat*, *Vikram Samvat* for the Bikram Sambat; `ई.सं.`, `ई.`,
`ईस्वी`, `सन्`, `AD`, `A.D.`, `CE` for the Gregorian. `गते` is the
Bikram Sambat's: a patro printed in Nepal writes a Bikram Sambat day
with it and a Gregorian one without.

### 2.2 The month names

The Bikram Sambat's months are solar, Baishakh to Chaitra
(`calendar-bikram-sambat.md`). Each locale's `sdk.calendar.BIKRAM_SAMBAT.
monthName` and `sdk.calendar.GREGORIAN.monthName` (and `monthShort`, for
the Gregorian's three-letter forms) are rendered for months 1 to 12 in
every loaded locale, and the spellings a pack does not carry are listed
in `dates.rs`, each from a source:

| month | romanised (the Government of Nepal's first) | Devanagari |
|---|---|---|
| 1 | Baishakh, Baisakh, Vaishakh, Vaisakh | बैशाख, वैशाख, बैसाख |
| 2 | Jestha, Jeth, Jyeshtha | जेठ, जेष्ठ, ज्येष्ठ |
| 3 | Asar, Ashar, Asadh, Ashadh, Ashad | असार, आषाढ, आषाढ़, अषाढ |
| 4 | Shrawan, Shravan, Srawan, Saun, Sawan | साउन, श्रावण, सावन |
| 5 | Bhadra, Bhadau, Bhado | भदौ, भाद्र, भाद्रपद |
| 6 | Ashwin, Asoj, Ashoj, Aswin | असोज, आश्विन, असौज |
| 7 | Kartik, Kattik | कात्तिक, कार्तिक |
| 8 | Mangsir, Mansir, Marga, Margashirsha | मंसिर, मङ्सिर, मार्ग, मार्गशीर्ष |
| 9 | Poush, Paush, Push, Pus | पुष, पौष, पुस, पूस |
| 10 | Magh | माघ |
| 11 | Falgun, Phalgun, Fagun | फागुन, फाल्गुन |
| 12 | Chaitra, Chait | चैत, चैत्र |

Sources: the Nepal Rastra Bank's monthly *Current Macroeconomic and
Financial Situation* (nrb.org.np) names its months Shrawan, Bhadra,
Ashwin, Kartik, Mangsir, Poush, Magh, Falgun, Chaitra, Baisakh, Jestha
and Asar, the English spellings the `en-Latn` pack already carries;
Wikipedia's *Vikram Samvat* lists the variants (Baisakh, Jeth, Asar and
Asadh, Saaun and Shrawan, Bhadau, Aasoj, Kattik, Mangsir and Aghan, Paush
and Poush and Push, Chait); the Devanagari spellings are those of the
`ne-Deva-NP` pack and of patros printed in Nepal (vernacular) and the
Sanskrit forms the `hi-Deva-IN` and `sa-Deva` packs carry (tatsama).

**Hindi's months are not the Bikram Sambat's.** In India the Vikram
Samvat is lunisolar, its months new moon to new moon or full moon to full
moon, and a date in it is a tithi (`कार्तिक कृष्ण अमावस्या`), not a day of
a month (Wikipedia, *Hindu calendar*). The `hi-Deva-IN` pack names the
Bikram Sambat's solar months with the same Sanskrit words (`वैशाख` for
Baishakh). So a Hindi `कार्तिक १५, २०८२` is read as the Bikram Sambat's
solar day, which is what the request asks; a span with a paksha word
after the month or the day (`शुक्ल`, `कृष्ण`, `सुदी`, `बदी`, `वदी`, and
their romanised spellings) is a lunar date and is **not read** (C1094).
Hindi's colloquial names of the lunar months (`अगहन`, `क्वार`, `भादों`)
are not listed for the same reason.

### 2.3 Readings, and what settles them

A span is answered with **every** reading that is a day its calendar
has:

- a month name says its calendar; a name both calendars spell would give
  both (none does today);
- figures alone read in both calendars, each kept if the day is one its
  calendar has: `2082/07/15` is a Bikram Sambat day and a Gregorian one
  in 2082 (C1096);
- figures with the year last read day first and month first, both kept
  where both are days (`05/06/2025`), one where the other is not
  (`31/10/2026`);
- an era marker or `गते` keeps its calendar's readings alone;
- `calendars` in the request keeps those calendars' readings alone, for
  an application that knows what its text is;
- a span with no reading left is not a date and is not answered.

Each reading is `{date, order, gregorian}`: the date in its calendar as
`CalendarDay`, the order its parts were read in (`YMD`, `DMY`, `MDY`) and,
for a Bikram Sambat reading, the Gregorian day it is.

Not read: a two-digit year, a date without a year (`Kartik 15`), a
weekday as evidence (it is not checked against the date), a range
(`15–17 Kartik`), the Nepal Sambat and other calendars, and a lunar date.

## 3. Claims checked (A2-4)

### 3.1 The time a claim is judged at

`verify.claims({text, at, latitudeDeg, longitudeDeg, altitudeM,
utcOffsetSeconds, locale})` judges every claim at the instant `at` and the
place, against what `almanac.sankalpa` computes there
(`sankalpa.md`): the vara, the lunar month, the samvatsara, the ritu and
the ayana of the civil day whose sunrise the instant follows; the tithi,
its paksha, the nakshatra, the yoga and the karana, and the Sun's, the
Moon's and Jupiter's sidereal signs, at the instant. A sankalpa is what a
priest recites as *the time now*: the claim "today is Ekadashi" is the
same statement, and a caller who means the tithi at sunrise asks at
sunrise (C1095). The profile settles every convention, as for the
sankalpa: the zodiac, the month's convention, the day's boundary.

### 3.2 The grammar

Narrow on purpose. The text is cut into sentences at `.`, `?`, `!`, `;`,
`।`, `॥` and line breaks, and each sentence is looked up (§1.3) for the
kinds a claim can name: `tithi`, `paksha`, `nakshatra`, `yoga`, `karana`,
`vara`, `masa`, `samvatsara`, `ritu`, `ayana`, `rashi` and `graha`.

| a sentence holding | is read as | judged by |
|---|---|---|
| a member of a day's kind, no graha | the day's (or the instant's) member is it | the sankalpa's member of that kind |
| one graha and a rashi | the graha is in that sign | the Sun's, the Moon's or Jupiter's sign |
| the Moon and a nakshatra | the Moon is in that nakshatra | the sankalpa's nakshatra |

A span naming several members of **one** kind is a claim that any of
them holds: *Ekadashi* names the bright and the dark fortnight's, and
either makes "today is Ekadashi" held. A fortnight's word just before a
tithi (`Shukla`, `शुक्ल`, `sudi`; `Krishna`, `कृष्ण`, `badi`) joins the
tithi's span and keeps that fortnight's member only: "Shukla Ekadashi"
claims `tithi.SHUKLA_EKADASHI`, and `Shukla` is not read apart as the
samvatsara of that name. A paksha named whole ("Shukla Paksha") is its
own claim. A span naming members of several kinds is narrowed by the
sentence: a graha leaves the rashi and the nakshatra; no graha drops the
rashi; a kind's own word (`tithi`, `तिथि`, `paksha`, `पक्ष`, `nakshatra`,
`नक्षत्र`, `star`, `month`, `महिना`, `मास`, `vara`, `बार`, `वार`, `rashi`,
`राशि`, `sign`, `yoga`, `योग`, `karana`, `करण`, `ritu`, `ऋतु`, `season`,
`samvatsara`, `संवत्सर`, `ayana`, `अयन`), matched by its fold, leaves
that kind. What is still several kinds is unread, `AMBIGUOUS`, with every
candidate.

A month claim is judged against the **lunar** month under the profile's
convention, which is what a sankalpa recites. A patro printed in Nepal
heads its pages with the solar Bikram Sambat month, and `Shrawan` names
both; the lexicon holds the lunar months (`masa`), so "this is Shrawan"
is read as the lunar Shravana (C1097). A caller who means the solar
month reads it with `intl.dates`.

A claim is unread, with its reason, when its sentence:

- is a question (it ends in `?`): `QUESTION`;
- holds a negation (`not`, `no`, `isn't`, `never`, `नहीं`, `होइन`, `छैन`,
  `हैन`, `थिएन`, `hoina`, `chaina`, `nahi`): `NEGATED`;
- names another time (`tomorrow`, `yesterday`, `भोलि`, `हिजो`, `परसि`,
  `कल`, `परसों`, `bholi`, `hijo`, `kal`; Hindi's `कल` is either):
  `OTHER_TIME`;
- puts a graha the sankalpa does not place in a sign or a star (Mars in
  Aries; the Sun in a nakshatra): `NOT_COMPUTED`;
- names two grahas, or a sign with no graha ("today is Capricorn":
  whose?): `AMBIGUOUS`.

The three sentence-wide reasons are read in that order, and before a
claim's own. The words are compared whole, lowercased, an apostrophe
dropped: `kal` is not `kaal`, so "Rahu kaal" is not another day.

Not read at all: festivals and observances ("today is Dashain"), dates
(SR-61 reads those), times of day, retrogression, aspects, houses, and
anything a sentence says beyond the members it names. A sentence that
names a graha alone makes no claim.

### 3.3 The answer

Each claim is `{start, end, text, kind, graha, claimed, verdict,
computed, reason}`: the span naming the member (in code points from the
text's start, as §1.3 counts) and its text, the kind claimed (`null` when
several kinds are left), the graha when one is the subject, every key the
span may name, `HELD`, `CONTRADICTED` or `UNREAD`, the computed key (on
held and contradicted claims) and the reason (on unread ones), each
absent value `null`. The answer is sealed with the sankalpa's provenance.
A locale no loaded one answers is refused as `claims.locale`; every other
refusal is the sankalpa's, named under `claims`.

## 4. The boundary

| record | boundary | Rust |
|---|---|---|
| `intl.lexicon` | `ts_intl_lexicon` | `sdk.intl().lexicon(&LexiconRequest)` |
| `intl.lookup` | `ts_intl_lookup` | `sdk.intl().lookup(&LookupRequest)` |
| `intl.dates` | `ts_intl_dates` | `sdk.intl().dates(&DatesRequest)` |
| `verify.claims` | `ts_verify_claims` | `sdk.verify().claims(&ClaimsRequest)` |

Each takes one JSON record and answers canonical JSON; each is an agent
tool with an input example, in every binding, and in the fuzz harness's
slots.

## 5. Cost

Measured on the maintainer's Mac (M-series, release build), by the
`intl` bench's `lexicon` group: see §5 of the measured numbers once
recorded.

## 6. Cruxes

C1090 to C1097 in `01-research/feature-universe/19-verification-cruxes.md`.

## Sources

- IAST: the *Transliteration* convention of the Tenth International
  Congress of Orientalists (Geneva, 1894); ISO 15919:2001,
  *Transliteration of Devanagari and related Indic scripts into Latin
  characters*.
- The Library of Congress, *ALA-LC Romanization Tables*, Nepali and Hindi
  (loc.gov/catdir/cpso/roman.html): the letters, which the common
  spelling writes without diacritics.
- Nepal Rastra Bank, *Current Macroeconomic and Financial Situation*,
  monthly (nrb.org.np): the English spellings of the Bikram Sambat
  months.
- Wikipedia, *Vikram Samvat* (the months' table and their variants) and
  *Hindu calendar* (the lunisolar months and their Hindi names).
- The Unicode Standard 16.0, ch. 12.1, *Devanagari*: the digits
  U+0966–U+096F, the nukta letters U+0958–U+095F.
- `sankalpa.md` and *Dharmasindhu*'s sankalpa form, through it: what a
  claim about *now* is judged against.

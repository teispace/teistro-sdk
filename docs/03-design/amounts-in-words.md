# Amounts in words: numbers and currency spelt out (SR-49)

Status: `built`, 2026-10-30: the rules generated from CLDR, the engine in
`crates/intl` (`teistro_intl::words`), `sdk.intl().in_words`,
`ts_intl_in_words`, the `intl.inWords` agent tool and every binding.
Nepali, Hindi and English are spelt by CLDR. Sanskrit is spelt by the
SDK's own drafted rules, built 2026-11-09 and shipped marked unreviewed
(`isDraft`) until the consumer's reviewer signs the forms off (§5, C803,
C1310 to C1319).

A cheque, an invoice and a receipt in Nepal and India carry an amount
twice: in figures and in words. The words are the legal amount: Nepal
Rastra Bank's *Cheque Standards and Specifications* (draft, May 2011)
§6.3 calls them so, makes them mandatory, and has a machine-written
amount preceded by asterisks; §6.4 puts the figures, under the
pre-printed `NPR`, in their own box. The application asks for both, in
Nepali, Hindi, English and Sanskrit, with the Indian grouping.

## 1. The source

The words come from CLDR's rule-based number formats (RBNF), the
`SpelloutRules` grouping of `common/rbnf/<language>.xml`, rank 1 (a
primary standard). CLDR is under the Unicode licence, which permits
copying with the notice kept; the licence is in
`crates/intl/cldr/LICENSE` and `NOTICE` names the data.

| file | what it gives | range spelt |
|---|---|---|
| `ne.xml` | every word 0 to 99 (they are irregular, and listed one by one), सय, हजार, लाख, करोड, अरब, खरब, शंख; the negative ऋणात्मक | below 10¹⁵, then digits in the Indian grouping |
| `hi.xml` | every word 0 to 99, सौ, हज़ार, लाख, करोड़, अरब, खरब; the negative ऋण | below 10¹⁸ (100 खरब and up are counted in खरब), then digits |
| `en_IN.xml` | the English words with lakh and crore; crores count on to 10¹² (one lakh crore), then trillion and quadrillion | below 10¹⁸, then digits |
| `en.xml` | the English words with million, billion, trillion, quadrillion | below 10¹⁸, then digits |

CLDR 48 has no `sa.xml` (nor `mr.xml`); the release is pinned at 48,
the release ICU4X 2.1's compiled data is built from, so the plural rules
and the words come from one CLDR. Sanskrit's rules are the SDK's own
draft in the same syntax (§5).

**Generated, never typed.** The four XML files sit unedited in
`crates/intl/cldr/rbnf/` (their SHA-256 in the generated file's header).
`cargo xtask gen intl` takes from each the rule sets a cardinal reaches,
`spellout-cardinal` and, where the file has one,
`spellout-cardinal-verbose`, and every rule set those name, parses every
rule with the engine's own parser, and writes them to
`crates/intl/src/words/cldr.rs`. A rule the engine cannot interpret is
a generator failure, not a silent skip, so a CLDR bump that brings new
syntax fails loudly. `check-intl` regenerates in memory and fails on any
difference, so the tables cannot drift from the XML.

**Drafted, never typed either.** A language CLDR has no rules for is
drafted by hand in the same syntax, in `crates/intl/drafts/rbnf/<file>.txt`
(only `sa.txt`). The file is the source: `#` lines carry its citations and
its reasoning, and `gen intl` drops them, keeps the sets
`spellout-cardinal` reaches, parses them with the same parser and writes
`crates/intl/src/words/drafted.rs` (`DRAFTED`, the file's SHA-256 in each
entry), which `check-intl` holds to the file as it holds `cldr.rs` to the
XML. Words spelt from a drafted file say so: the answer's `rules` is
`draft/<file>/<rule set>` and `isDraft` is true.

**The interpreter** is a subset of ICU's `RuleBasedNumberFormat` for
integers, exactly as TR35 §5 (Rule-Based Number Formatting) states it:
the rule with the largest base value not above the number; its divisor
the highest power of the radix (10 unless `base/radix`) not above the
base; `<<` the quotient and `>>` the remainder, in the rule's own rule
set or the one named; `=…=` the number itself in a named rule set or a
decimal pattern; `[…]` dropped when the remainder is zero; the `-x` rule
for a negative; a leading `'` keeping the space after it; and ICU's
roll-back to the previous rule where a rule's base is not a multiple of
its divisor and the number is. Fractions (`x.x`), `Inf` and `NaN` are not
read: an amount's minor units are a second integer (§3).

## 2. The grouping

The Indian grouping names every two digits past the thousands: hazar
(हजार, हज़ार) 10³, lakh 10⁵, crore 10⁷, arab 10⁹, kharab 10¹¹, and in
Nepali shankh 10¹³. It is the grouping of every Nepali and Hindi rule set
in CLDR; neither file has another. English has both: `INDIAN` reads
`en_IN.xml` (lakh, crore) and `INTERNATIONAL` reads `en.xml` (million,
billion). `INTERNATIONAL` in Nepali, Hindi or Sanskrit is refused `UNSUPPORTED`,
naming the field, since CLDR gives no such words and the SDK does not
make them up.

CLDR's English has no arab or kharab: it counts in crores up to one lakh
crore and then says trillion. That is the source's reading and ships as
it is (C804); an application that wants "one hundred crore" past
10⁹ has it already, since 10⁹ is spelt "one hundred crore".

The figures follow the same grouping: `12,34,567` for `INDIAN`,
`1,234,567` for `INTERNATIONAL`, in the digits asked for. Left out, the
grouping is the locale's own (`_meta.json`'s `grouping`: `[3, 2]` is
Indian, `[3]` international), so `ne-Deva-NP` and `hi-Deva-IN` write
Indian and `en-Latn` international unless asked; a language whose words
count in the Indian grouping alone (Nepali, Hindi, Sanskrit) takes it
whatever its locale's figures use, so `sa-Latn`, whose numbers are grouped
by three, writes `pañcāśatsahasrādhikalakṣam` over `1,50,000`.

**Digits** are `LATN` or `DEVA`, the locale's numbering system when left
out. They write the figure, and the digits a rule set falls back to past
its last word (a Nepali amount of 10¹⁵ or more).

## 3. Amounts and rounding

An amount is never a float. It crosses as one of:

- `{"minor": 123456}`: an unsigned count of the currency's minor units
  (123456 paisa is NPR 1,234.56), exact, up to 2⁶⁴ − 1;
- `{"decimal": "1234.56"}`: a decimal string of ASCII digits with at
  most one `.`, no sign and no exponent, rounded to the currency's minor
  digits by `rounding`.

A negative amount cannot be written in either form (the minor count is
unsigned, and a decimal's `-` is refused `INVALID_ARG`): a cheque or an
invoice line is not negative, and a credit note writes the amount and
says "credit" in its own words. A **number** (`{"number": -42}`) is an
`i64` and its negative is spelt by the rule set's own `-x` rule:
"minus forty-two", "ऋणात्मक बयालिस", "ऋण बयालीस".

**Rounding** is `HALF_UP` unless asked: a fraction of half a minor unit
or more goes up, less is dropped. That is the rule India's banks are
held to: RBI's circular DBOD.Dir.BC.70/13.01.01/2006-07 of
30 March 2007 restates that "all transactions … should be rounded off
to the nearest rupee, i.e. fraction of 50 paise and above shall be
rounded off to the next higher rupee and fraction of less than 50 paise
shall be ignored" and tells banks not to refuse a cheque bearing paise.
That is rounding to the rupee, a step an application takes or not; the
SDK rounds only below the minor unit, and by the same half-up rule. No
Nepal Rastra Bank text found states a rounding rule (the cheque standard
above is silent on it), so for NPR half-up is the convention, not a
rule (C805). `HALF_EVEN` (banker's rounding) and `EXACT` (refuse
`INVALID_ARG` when a digit past the minor unit is not zero) are offered.
The answer says whether the decimal was rounded.

**The currency is a parameter.** A request names an ISO 4217 code, three
capital letters, and the currency's minor digits (2 when left out; 0
writes no minor part). The unit words are the locale's message
`sdk.amount.<CODE>.major` and `.minor`, each selecting on `$count` by the
locale's plural rules, so the SDK's NPR and INR are two entries in
`i18n/`, and any other currency is one more entry in a consumer's pack or
an override, with no code. A code the locale does not name, or names
only through a fallback in another language, is refused `UNSUPPORTED`,
naming the key to add: English unit names inside Hindi words would be
wrong silently.

How the parts join is the locale's message too: `sdk.amount.words`
selects on `$shape` (`MAJOR`, `MINOR` alone when the major part is zero
and the minor is not, `BOTH`), and `sdk.amount.only` adds the closing
word cheque guides ask for ("only", मात्र) when `only` is set, so the
amount cannot be extended after it.

| locale | 1234.56 NPR, `only` |
|---|---|
| `en-Latn` (`INDIAN`) | one thousand two hundred thirty-four rupees and fifty-six paisa only |
| `ne-Deva-NP` | एक हजार दुई सय चौँतिस रुपैयाँ र छपन्न पैसा मात्र |
| `hi-Deva-IN` (INR) | एक हज़ार दो सौ चौंतीस रुपये और छप्पन पैसे मात्र |
| `sa-Deva` (INR, draft) | चतुस्त्रिंशदधिकद्विशताधिकसहस्रम् रूप्यकाणि षट्पञ्चाशत् पैसा च मात्रम् |
| `sa-Latn` (INR, draft) | catustriṃśadadhikadviśatādhikasahasram rūpyakāṇi ṣaṭpañcāśat paisā ca mātram |

## 4. The API

`teistro_intl::words` holds the engine; `Intl::in_words` renders a
request in a locale; the façade's `sdk.intl().in_words(&InWordsRequest)`
reads the context's locale when the request names none.

```json
{"amount": {"currency": "NPR", "value": {"minor": 123456}, "only": true},
 "locale": "ne-Deva-NP", "grouping": "INDIAN", "digits": "DEVA", "style": "PLAIN"}
```

- `number` (an `i64`) or `amount` (the currency, the value, `minorDigits`,
  `rounding`, `only`), exactly one;
- `style`: `PLAIN`, CLDR's `spellout-cardinal`, or `VERBOSE`, its
  `spellout-cardinal-verbose` ("one hundred and five"), which only English
  has; asked of another language it is refused `UNSUPPORTED`.

The answer: `text`, the whole; `figure`, the figures; `rules`, the CLDR
file and rule set the words came from (`cldr-48/en_IN/spellout-cardinal`),
or the drafted one (`draft/sa/spellout-cardinal`, with `/iast` when
transliterated for `sa-Latn`); `isDraft`, whether any of the text is
unreviewed, the drafted words or a unit word from a namespace its locale
marks draft; and for an amount its parts, the major and minor counts, each
count's words and unit word, and `rounded`. An application that wants another
order, a capital initial or a currency symbol composes the parts itself.

It crosses as `ts_intl_in_words(context, request_json, out_json)`, the
record `intl.inWords`, and each binding's `intl.inWords` (Node, Python,
Dart, Java) with its generated request type.

## 5. What a reviewer has to read (Q42)

The consumer supplies the Nepali, Hindi and Sanskrit reviewers (Q42,
decided). The CLDR words are rank 1 and ship as CLDR spells them,
`शुन्य` and `चौँतिस` included; a reviewer's correction goes to CLDR and
arrives with the next pin. The words below are the SDK's draft, not
CLDR's, and wait for review (Sanskrit's, `sdk.amount` in `sa-Deva`, are
marked `draft` in `_meta.json` and say `isDraft`):

| key | `ne-Deva-NP` | `hi-Deva-IN` | `sa-Deva` |
|---|---|---|---|
| `sdk.amount.NPR.major` | रुपैयाँ | नेपाली रुपया, नेपाली रुपये | नेपालरूप्यकम्, ०रूप्यके, ०रूप्यकाणि |
| `sdk.amount.NPR.minor` | पैसा | पैसा, पैसे | पैसा |
| `sdk.amount.INR.major` | भारतीय रुपैयाँ | रुपया, रुपये | रूप्यकम्, रूप्यके, रूप्यकाणि |
| `sdk.amount.INR.minor` | पैसा | पैसा, पैसे | पैसा |
| `sdk.amount.words` | … र … | … और … | … … च |
| `sdk.amount.only` | मात्र | मात्र | मात्रम् |

### Sanskrit, drafted (C803, C1310 to C1319)

CLDR has no Sanskrit rules, so the SDK drafts them, in RBNF's own syntax,
in `crates/intl/drafts/rbnf/sa.txt` (§1), and every answer spelt from
them is marked `isDraft` until the consumer's reviewer confirms the forms
below. The consumer's SR-75 answer, to ship Sanskrit text marked rather
than wait on Q42, is read to cover these words too.

**Sources.** The words and how they combine are the grammars':
M. R. Kale, *A Higher Sanskrit Grammar* (1894; 7th ed. 1931, Internet
Archive) §159 (the cardinals 1 to 99 with every optional form, 100, 200
as *dviśata* or *dve śate*, 300 as *triśata* or *trīṇi śatāni*, 1000,
and the scale words with their genders, *lakṣa* neuter, *koṭi*
feminine), §160 (Pāṇini VI.3.47–49, quoted: *dvi*, *tri* and *aṣṭan* are
*dvā*, *trayas*, *aṣṭā* before *-viṃśati* and *-triṃśat*, optionally from
*-catvāriṃśat*, never before *aśīti*) and §161 (the numbers between the
hundreds "generally expressed by means of *adhika*", with the worked
values 101, 112, 150, 1892 and 177639); W. D. Whitney, *A Sanskrit
Grammar* (1879; 1896 issue, Internet Archive) §475 (the decimal names,
and d: "in modern time the only numbers in practical use above thousand
are *lakṣa* and *koṭi*", an Indian sum read as "1-23 crores, 45 lakhs,
67 thousand, eight hundred and ninety"), §476 (11 to 99 and their
sandhi), §§477–478 (*ūna* and *adhika*), §481 (a multiplier compounded
with *śata* or *sahasra*, "in later usage more generally"), §§482–486
(inflection and construction). The rank-1 texts behind them are Pāṇini
VI.3.47–49 and Bhāskara's *Līlāvatī* vv. 10–11 (the place names *eka*,
*daśa*, *śata*, *sahasra*, *ayuta*, *lakṣa*, *prayuta*, *koṭi*, …, as
Kale's §159 footnote prints them).

**How a number reads.** Kale §161's method: the part below the largest
scale, itself so read, compounded with *adhika* ("exceeded by"), then the
scale word with its multiplier compounded (Whitney §481: *dviśatam*, not
*dve śate*), the whole one compound in the nominative singular, neuter
(*-śatam*, *-sahasram*, *-lakṣam*) or, for *koṭi*, feminine (*-koṭiḥ*).
Kale's 1892 comes out letter for letter: द्वानवत्यधिकाष्टशताधिकसहस्रम्.
One word suits a cheque's legal amount: nothing can be added inside it.

| number | `sa-Deva` | `sa-Latn` | from |
|---|---|---|---|
| 0 | शून्यम् | śūnyam | C1311 |
| 1 | एकम् | ekam | K §159, neuter (C1310) |
| 2 | द्वे | dve | K §159 |
| 11 | एकादश | ekādaśa | K §159, W §476 |
| 19 | नवदश | navadaśa | K §159's first form (C1312) |
| 21 | एकविंशतिः | ekaviṃśatiḥ | K §159 |
| 42 | द्वाचत्वारिंशत् | dvācatvāriṃśat | K §160 (C1313) |
| 96 | षण्णवतिः | ṣaṇṇavatiḥ | W §476 d |
| 99 | नवनवतिः | navanavatiḥ | K §159 (C1312) |
| 100 | शतम् | śatam | K §159 |
| 101 | एकाधिकशतम् | ekādhikaśatam | K §161 |
| 200 | द्विशतम् | dviśatam | K §159, W §481 |
| 1000 | सहस्रम् | sahasram | K §159 |
| 1892 | द्वानवत्यधिकाष्टशताधिकसहस्रम् | dvānavatyadhikāṣṭaśatādhikasahasram | K §161 |
| 1,00,000 | लक्षम् | lakṣam | K §159 |
| 1,77,639 | नवत्रिंशदधिकषट्शताधिकसप्तसप्ततिसहस्राधिकलक्षम् | … | K §161, with C1312 and C1315 |
| 1,00,00,000 | कोटिः | koṭiḥ | K §159 |
| 1,23,45,67,890 | नवत्यधिकाष्टशताधिकसप्तषष्टिसहस्राधिकपञ्चचत्वारिंशल्लक्षाधिकत्रयोविंशत्यधिकशतकोटिः | … | W §475 d (C1314) |
| −5 | ऋणम् पञ्च | ṛṇam pañca | C1318 |

**The scales** are the Indian grouping's alone: *śata*, *sahasra*,
*lakṣa*, *koṭi* (Whitney §475 d), and crores are counted with a whole
number, as Whitney's "1-23 crores", so every `i64` is spelt (10⁹ is
शतकोटिः, 10¹⁴ कोटिकोटिः). *Līlāvatī*'s further names (*arbuda*, *abja*,
*kharva*, *nikharva*, *mahāpadma*, *śaṅku*, …) count every power of ten,
not every second one, and the grammars give no grouping in them (C1314).

**Sandhi.** The rules spell each joint out, as CLDR's Nepali spells each
number below a hundred: a separate set for a multiplier before *lakṣa*
(चतुर्, षड्, त्रिंशल्) and before *koṭi* (चतुष्), and the units before each
ten. One joint cannot be spelt in a rule: *adhika*'s final *a* meeting the
vowel a following term begins with (अधिक + अष्टशत = अधिकाष्टशत, K §161's
own 1892). The rules mark it `+`, and the speller closes it by the vowel
sandhi of a final *a* (a + a = ā, a + e = ai, and so on); a `+` anywhere
else is dropped. That one step is the only thing the drafted rules ask of
the speller that CLDR's do not.

**`sa-Latn`** reads the same words transliterated into IAST by the table
that derives `sa-Latn` from `sa-Deva`, and its `sdk.amount` is derived
the same way (`DERIVED_MESSAGES`, `intl-engine-and-packs.md`), still
marked. `VERBOSE` and `INTERNATIONAL` are refused for Sanskrit as for
Hindi.

## 6. Tests

- **Group boundaries, read back.** For each rule set, 0, 1, 9, 10, 11,
  19, 20, 99, 100, 101, 999, 1000, 99 999, 1 00 000, 99 99 999,
  1 00 00 000, every scale word's first value and the value before it,
  and the last spelt value and the first in digits, are spelt and read
  back to the number by a reader built from the generated tables alone
  (each word's value, a scale word multiplying the run before it), so the
  test does not trust the speller to check the speller.
- **Every value 0 to 99** of Nepali and Hindi is CLDR's own word.
- **Worked examples:** 10,000.50 as "ten thousand rupees and fifty paisa
  only", the cheque guide's (Vedantu, *How to write rupees and paise in
  words*); 1234.56 NPR in Nepali; the tables in §3.
- **Rounding:** 0.005 and 0.015 under both rules, 0.004, an `EXACT`
  refusal, a decimal with too many digits for `u64`.
- **Sanskrit** (`crates/intl/tests/sanskrit_words.rs`): every worked
  value of §5 against the section it is read from, Kale's 1892 letter for
  letter and his 177639 with the draft's two choices substituted, Whitney's
  Indian sum, `i64::MAX` and `MIN + 1`, `sa-Latn`, an INR amount in both
  scripts, `isDraft` on every Sanskrit answer and on no other, and the
  generated table held to the hand-kept file's SHA-256.
- **Refusals:** `INTERNATIONAL` in Nepali or Sanskrit, `VERBOSE` in Hindi
  or Sanskrit, an unknown currency, a fallback unit, a malformed decimal,
  both or neither of `number` and `amount`.

## 7. Cruxes

- **C803**: Sanskrit's numerals have optional forms; which ship waits on
  the consumer's reviewer. Drafted 2026-11-09 and shipped marked; the
  choices are C1310 to C1319.
- **C1310 to C1319**: the drafted Sanskrit's forms, for the reviewer to
  confirm or change: the citation form, zero and eight, the ninth of each
  ten, *dvā*/*dvi*, the scales, how the parts join, optional sandhi, the
  currency words, the closing and negative words, and `sa-Latn`.
- **C804**: CLDR's Indian English has no arab or kharab, and says
  trillion from 10¹². Shipped as CLDR has it.
- **C805**: no Nepali text found states a rounding rule below the paisa;
  half-up, India's banking rule, is the convention, `HALF_EVEN` and
  `EXACT` the alternatives.

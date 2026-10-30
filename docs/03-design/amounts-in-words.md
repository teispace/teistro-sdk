# Amounts in words: numbers and currency spelt out (SR-49)

Status: `built`, 2026-10-30: the rules generated from CLDR, the engine in
`crates/intl` (`teistro_intl::words`), `sdk.intl().in_words`,
`ts_intl_in_words`, the `intl.inWords` agent tool and every binding.
Nepali, Hindi and English are spelt; Sanskrit is refused as not yet
reviewed (C803).

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
and the words come from one CLDR.

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
billion). `INTERNATIONAL` in Nepali or Hindi is refused `UNSUPPORTED`,
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
Indian and `en-Latn` international unless asked.

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
file and rule set the words came from (`cldr-48/en_IN/spellout-cardinal`);
and for an amount its parts, the major and minor counts, each count's
words and unit word, and `rounded`. An application that wants another
order, a capital initial or a currency symbol composes the parts itself.

It crosses as `ts_intl_in_words(context, request_json, out_json)`, the
record `intl.inWords`, and each binding's `intl.inWords` (Node, Python,
Dart, Java) with its generated request type.

## 5. What a reviewer has to read (Q42)

The consumer supplies the Nepali, Hindi and Sanskrit reviewers (Q42,
decided). The CLDR words are rank 1 and ship as CLDR spells them,
`शुन्य` and `चौँतिस` included; a reviewer's correction goes to CLDR and
arrives with the next pin. The words below are the SDK's draft, not
CLDR's, and wait for review:

| key | `ne-Deva-NP` | `hi-Deva-IN` |
|---|---|---|
| `sdk.amount.NPR.major` | रुपैयाँ | नेपाली रुपया, नेपाली रुपये |
| `sdk.amount.NPR.minor` | पैसा | पैसा, पैसे |
| `sdk.amount.INR.major` | भारतीय रुपैयाँ | रुपया, रुपये |
| `sdk.amount.INR.minor` | पैसा | पैसा, पैसे |
| `sdk.amount.words` | … र … | … और … |
| `sdk.amount.only` | मात्र | मात्र |

**Sanskrit** (C803). CLDR has no Sanskrit rules. Whitney's *Sanskrit
Grammar* §§475–489 (rank 2) gives the cardinals, and with them the
optional forms the grammar allows for one number (22 as *dvāviṃśati*,
42 as *dvācatvāriṃśat* or *dvicatvāriṃśat*, 19 as *navadaśa* or
*ūnaviṃśati*), and a number's case agreement with what it counts. A
rule set chosen among those without a reviewer would be a guess, so
`sa` is refused `UNSUPPORTED` with the reason `unreviewed` and the hint
naming this page, until the consumer's reviewer picks the forms and they
ship as a rule set of the same shape.

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
- **Refusals:** `sa`, `INTERNATIONAL` in Nepali, `VERBOSE` in Hindi, an
  unknown currency, a fallback unit, a malformed decimal, both or neither
  of `number` and `amount`.

## 7. Cruxes

- **C803**: Sanskrit's numerals have optional forms; which ship waits on
  the consumer's reviewer. Refused as unreviewed.
- **C804**: CLDR's Indian English has no arab or kharab, and says
  trillion from 10¹². Shipped as CLDR has it.
- **C805**: no Nepali text found states a rounding rule below the paisa;
  half-up, India's banking rule, is the convention, `HALF_EVEN` and
  `EXACT` the alternatives.

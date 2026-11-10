# Entity names for what a document carries

Status: `built`, written 2026-09-14; every catalogue member named
2026-11-10 (§4e), under the policy that replaced §4's. It derives from
[`intl-engine-and-packs.md`](intl-engine-and-packs.md) (the packs and
their completeness) and from
[`document-schema.md`](document-schema.md), whose schema is what makes
the scope below computable. The rule it keeps is
`02-architecture/03-localization-architecture.md`'s: *no
machine-translated stubs, because a plausible wrong astrological term is
worse than a visible fallback*. Until 2026-11-10 it also kept a member
unnamed until a vetted source or a reviewer named it; that policy is
replaced by §4e's, under which the SDK owns its text and names every
member from the best-supported source it finds.

## 1. The gap, measured

A chart document carries catalogue keys: a graha, a dignity, a point, a
house system, an ayanamsha. A reader renders each through
`intl.entity(key).name`. The locale engine reports catalogue coverage but
gates only a locale's completeness against the **base locale's** keys,
so a kind nobody had named was complete by definition.

Measured over the 28 catalogue kinds the document schema names, and the
five locales the SDK ships:

| kind | members | named before |
|---|---:|---:|
| `ayanamsha` | 47 | 0 |
| `point` | 49 | 1 (the lagna) |
| `house_system` | 22 | 0 |
| `varga` | 21 | 0 |
| `masa` | 12 | 0 |
| `avastha_deeptadi` | 9 | 0 |
| `direction` | 8 | 0 |
| `chart_kind` | 7 | 0 |
| `avastha_lajjitadi` | 6 | 0 |
| `calendar` | 6 | 0 |
| `avastha_jagradadi` | 3 | 0 |
| `graha` | 12 | 9 (the outer three missing) |

That is 192 members with no name in either strict locale (`en-Latn`,
`ne-Deva-NP`), and nine more (`era`) missing only from the two
base-completeness Devanagari locales. The `chart_reading` example could print a Gulika's sign
and not the word *Gulika*.

## 2. Where names may come from

The clean-room policy ranks sources, and a name is data like any other:

- **Rank 2, the baseline engine**, keeps four-language name tables that
  a production Nepali app has shown to readers: English, Nepali, Hindi,
  and Sanskrit with its IAST. They come with a stated naming policy:
  - Indian systems keep their traditional name (श्रीपति, राशि-भाव);
  - personal names are transliterated, not translated (प्लासिडस);
  - descriptive systems use the established term (सम भाव);
  - alphanumeric designators stay Latin (MC, APC).

  The policy allows rule data from it to be taken.
- **Not a translation of mine.** A Devanagari spelling of a Western
  proper name, or a Nepali gloss for a chart kind, is exactly the
  plausible stub the policy refuses.

So the import takes **every name the baseline engine vets, and nothing
else**. It is read by loading the source modules, not by transcribing
them, so no Devanagari passes through a keyboard:

| kind | source table | members |
|---|---|---:|
| `ayanamsha` | `AYANAMSA_DEFINITIONS[].names` | 47 |
| `house_system` | `HOUSE_SYSTEM_NAMES` | 22 |
| `masa` | `HINDU_MONTH_NAMES` | 12 |
| `avastha_deeptadi` | `DEEPTADI_NAMES` | 9 |
| `direction` | `DISHA_NAMES` | 8 |
| `point` | the upagraha service's table | 7 |
| `avastha_lajjitadi` | `LAJJITADI_NAMES` | 6 |
| `avastha_jagradadi` | `JAGRADADI_NAMES` | 3 |

That is 114 of the 192. Keys match the catalogue's exactly except for three
spellings, and each is a stated rename rather than a fuzzy match:

- `ASHVIN` is `masa.ASHWINA` (the same difference the conformance corpus
  has);
- `PARIVESH` is `point.PARIVESHA`;
- the directions' kebab case (`south-east`) is the catalogue's
  `SOUTHEAST`.

## 3. The record each name becomes

The forms follow the kinds already named, so a consumer cannot tell an
imported record from an authored one:

- **`name` and `prose`** are the language's primary name.
- **`iast`** is the Sanskrit transliteration **where the source gives
  one**, in every Devanagari locale and in English. It is never computed:
  the transliterator writes a Western name in Devanagari as nonsense, and
  a form that is absent is better than a wrong one.
- **No `short`, `glyph` or `gender`.** The source vets none of them, and
  `name` stands in for a missing form, as the engine already allows.
- **`sa-Latn` is not written by hand.** `gen intl` derives it from
  `sa-Deva`, as it does for every other kind, and building showed where
  that is wrong. Transliterating a Western name that was itself
  transliterated into Devanagari yields nonsense (`प्लासिडस` becomes
  `Plāsiḍasa`, `फेगन-ब्रैडली` becomes `Phegana-braiḍalī`), while the
  descriptive Sanskrit names come out right (`Sampūrṇa Rāśi`,
  `Ākāśagaṅgā Kendra 0° Dhanu`). So the 35 names that contain a
  non-Sanskrit proper noun (14 house systems, 21 ayanamshas) are listed
  in `i18n/sa-Latn/_overrides.json`, the mechanism the localisation
  standard keeps for "hand overrides where the mechanical result is
  wrong". Each override is the **source's own English** for that key, so
  the list is a classification and not a translation.

**A form's absence must not change a type.** The binding gates found
this the first time they ran. The generated accessors typed a form as
always present when *every* base-locale record happened to carry it. Before
the import that was true of `iast`, so Dart's `entity.iast` was a
`String` and Node's `entityForms` gave it `''`. The Western ayanamshas and
house systems rightly carry no `iast`, so the inferred set shrank, Dart's
form became nullable and an example that pads it stopped compiling. The
data was right and the inference was the defect. So
`generate::GUARANTEED_FORMS` now declares `name`, `prose` and `iast` as
the forms every binding types as present (empty when a record lacks one),
which is what the bindings' own tests already asserted.

Where the source's English is a gloss (`Awake` for जाग्रत्), the record
keeps the gloss as `name` and the Sanskrit as `iast`, which is what
`avastha_baladi` already does (`Infant`, `Bāla`).

## 4. The gate: a kind a document carries is named, or listed

`check-intl` gains a rule. Each **strict** locale names every member of
every catalogue kind the document schema names, **except the members
listed** in `xtask/src/intl.rs` as unnamed, each kind with the reason
it has no vetted source.

The list fails both ways, as the project's other lists do:

- an unnamed member not on the list fails, so a new kind a document
  starts carrying cannot arrive silently unnamed;
- a named member still on the list fails, so the list only ever shrinks.

The kinds come from the schema, not a second list. Each catalogue key's
schema carries `x-teistro-kind`, which is also what lets a consumer's
tooling map a string back to its catalogue kind.

Left on the list after this step, 78 members, each for want of a vetted
source rather than for want of trying: the other 41 points (the special
lagnas, the arudhas, the sphutas and the yogi points), the 21 divisional
charts, the 7 chart kinds, the 6 calendars and the 3 outer planets.
`era` in the two base-completeness locales (`hi-Deva-IN`, `sa-Deva`) is
outside the rule, which covers strict locales.

**Every kind since 2026-10-21 (C682).** A document is not the only
reader of a key: a day page shows a tithi's class and a muhurta's
nature, a match its kootas, a star list its stars. So the rule now
holds every catalogue kind (`Kind::ALL`), not the schema's. The
widening found 259 members of 26 kinds unnamed and unlisted, and each
kind went on the list with its reason, so the list holds 335:

- the day page's kinds, whose names are proposed in
  [`amrit-kalam-and-lagnas.md`](amrit-kalam-and-lagnas.md) §3.2 and
  wait there for a reviewer of each language;
- the kinds a reading or a match shows (`koota`, `state`, `deity`,
  `auspiciousness`, `sex`, `parity`), for want of a Devanagari table
  from a ranked source;
- the SDK's own classifications (`dasha_family`, `point_family`,
  `chart_layout` and five more), where an English name would be the
  SDK's coinage and would bind a Nepali one;
- the 128 stars, whose IAU names would bind a Nepali name each;
- the Tibetan calendar's elements and animals.

The schema's kinds still scope the coverage page
(`locale-coverage-measured.md`), which measures what a document carries.

## 4a. The dasha systems, when a document first carried them

The dasha section (2026-09-15) put `dasha_system` into the document schema,
and the gate refused all 40 members at once. The baseline engine vets a
four-language name for each of the 18 systems it implements, on each
engine's own `displayNames`, with the Sanskrit transliteration: English
`Vimshottari Dasha`, Nepali and Hindi `विंशोत्तरी दशा`, Sanskrit the same
with `Viṁśottarī Daśā`. They were loaded the way §2's tables were, by
parsing the modules with the TypeScript compiler and reading the literals,
so no Devanagari passed through a keyboard, and all 18 keys match the
catalogue's with no rename. Each record is `name` and `prose` in the
language, with the IAST in every locale. The other 22 systems, which the
engine does not implement, are listed, bringing the list to 100.

## 4b. The blackouts and the eclipses, named where the Nepali is vetted

`blackout_kind`, `lunar_eclipse_kind` and `solar_eclipse_kind` joined the
catalogue on 2026-10-01 so a consumer can name a closed day's reasons and
an eclipse's kind. They are not in the document schema, so §4's list does
not reach them. One rule does: a strict locale carries every key the base
locale carries. So a member is named in English only when its Nepali is
vetted too, and an English name alone would be refused.

The Nepali is the Nepal Panchanga Decision Committee's own usage where it
prints the thing, read from the 2083 national panchanga's page images
(the PDF has no text layer). Otherwise it is the rank-1 text's own word:

| member | Nepali | where |
|---|---|---|
| `CHATURMAS` | चतुर्मास | the committee, "चतुर्मासव्रत सुरु" on Harishayani Ekadashi |
| `ADHIKA_MASA` | मलमास | the committee's usage (C177) |
| `SAMSARPA` | संसर्प | *Dharmasindhu* p. 3, "संसर्पसंज्ञः" |
| `KSHAYA_MASA` | क्षयमास | *Dharmasindhu* p. 3 |
| `PITRU_PAKSHA` | पितृपक्ष | the committee, Ashwin's dark half headed "(पितृ) पक्ष" |
| `SANKRANTI` | सङ्क्रान्ति | the committee, "फाल्गुनसङ्क्रान्ति" |
| `HOLASHTAKA` | होलाष्टक | *Shighrabodha* I.137 (the committee prints "होलिकारम्भ" for its first day) |
| `GURU_ASTA`, `SHUKRA_ASTA` | गुरु अस्त, शुक्र अस्त | the committee, "गुरु पश्चिममा अस्त", "शुक्र पश्चिममा अस्त" |
| `PARTIAL`, `TOTAL` (both eclipse kinds) | खण्डग्रास, खग्रास | the committee, naming an eclipse as Nepal sees it |

Six members stay unnamed in both locales: `KHARMAS`, `ECLIPSE_STAR`,
`ECLIPSE_VEDHA`, `PENUMBRAL`, `ANNULAR` and `HYBRID`. The committee's
2083 almanac prints none of them, since it shows no eclipse Nepal does
not see, and no text read so far gives a Nepali word. Each record is
`name` and `prose`, with no `iast` because no source gives one. The
eclipse kinds' English `prose` is lower case (`total`), because it
qualifies a noun.

## 4c. The terms BPHS itself prints (2026-10-17)

A test app drew `D9` and `KIMSHUKA` where a reader expected a word, so the
list was read again against the rank-1 text. The source is the Devanagari
e-text of *Bṛhat Pārāśara Horāśāstra* at sanskritdocuments.org
(`doc_z_misc_sociology_astrology/par0110.html` to `par9197.html`). Its
chapter numbers are the catalogue's (the vargas are ch. 6, the Sayanadi
ch. 45, the dashas ch. 46). The Nepali record is the text's own
Sanskrit term, as §4b took *Dharmasindhu*'s. The English record is the
catalogue's established spelling. A script refused every name whose
witness string does not occur in the downloaded text, so no Devanagari
was typed from memory.

| kind | named | where |
|---|---:|---|
| `varga` | 15 of 21 | 6.3-4, 6.6-33 (D24 as सिद्धांश, 6.22; D27 as भांश, 6.24; D4 as तुर्यांश, 6.9) |
| `vaiseshikamsa` | 27 of 30 | 6.43-51 |
| `avastha_sayanadi` | 12 of 12 | 45.30-31, the stems as the effect verses inflect them |
| `avastha_cheshta` | 3 of 3 | 45.37-38 |
| `point` | 11 more | the Bhava, Hora and Ghati lagnas (5.2-8), Varnada (5.14), Pranapada (3.71-72), Kala, Mrityu, Ardhaprahara (3.68-69), Karakamsha (ch. 33), Lagna Pada and Upapada for A1 and A12 (29.2-3, 30.2-3) |
| `dasha_system` | 11 more | 46.3-11 and the systems' own verses; Shashtihayani is the text's षष्टिसमा (46.40), Ashtakavarga its अष्टवर्गदशा (46.203), Sudarshana Chakra ch. 74 |
| `nature` | 2 of 3 | 3.11, क्रूर and सौम्य |

Each record is `name` and `prose` alone. The e-text gives no IAST, and
§3 refuses a computed one.

Four rules kept members off:

- **A misprint is not a spelling.** The e-text prints कुमुम (6.48),
  शक्रसिंहासन and स्रीवल्लभ (6.51), and यमघण्ट्कः (3.69). A second print
  must decide each one. The obvious correction is still a guess.
- **Only a name the text prints counts.** BPHS calls every arudha the
  pada of its bhava, so a "धनपद" for A2 would be a coinage. It lists the
  36-year dasha only by its length, and it gives D1 as क्षेत्र where a
  chart prints *rashi*.
- **Not every outer planet's name is vetted.** Nepali Wikipedia titles
  them अरुणग्रह, वरुणग्रह and यमग्रह, against the transliterations
  common in print. No committee, Nepal Academy or curriculum source was
  found to decide between them, so Uranus, Neptune and Pluto stay
  unnamed. The bindings' tests still use `graha.PLUTO` as the entity a
  locale does not carry.
- **The texts not read stay unread.** The sphutas (Prasna Marga), the
  yogi points, the Sree, Indu and Vighati lagnas, the Tajika annual
  dashas and the tithi, yoga and karana dashas are not in BPHS.

`chart_layout` is a catalogue kind, so `intl.entity("chart_layout.NEPALI_LOTUS")`
resolves once a locale names it. No source names the layouts in Nepali,
so §4's list carries all six.

## 4d. The year's rulers (2026-11-01)

The year's offices, the texts that name them and the rules each office's
day is read under (`year-rulers.md`) became catalogue kinds 82 to 84,
`year_office`, `ruler_text` and `ruler_day_rule`, so a reader can print
*Rasesha* where the answer carries `year_office.RASADHIPATI`. The
catalogue keys keep the texts' own titles of office (`RASADHIPATI`); the
names are the shorter forms the printed almanacs use.

**Nepali.** The offices are spelled as a patro printed in Nepal prints
them: the *Himal Panchanga* for VS 2082, p. 2, "श्रीसम्वत् २०८२ सालको
वर्षव्यवस्था", read off its page images (the scan has no text layer),
which seats the year's ten office-bearers (*daśādhikārī*) as "राजापदमा
सूर्य, मन्त्रीपदमा चन्द्र, सस्येशपदमा गुरु, रसेशपदमा शुक्र, नीरसेशपदमा
बुध, मेघेशपदमा सूर्य, धान्येशपदमा मङ्गल", then the lords of fruit, wealth
and forts, which the texts read here do not make. Its raja and mantri for
VS 2082, the Sun and the Moon, are the ones the southern nine print for
that year (`year-rulers.md` §3). The texts and rules carry the texts' own words, read off
the pages:

| member | Nepali | where |
|---|---|---|
| `RAJA`, `MANTRI`, `SASYADHIPATI`, `DHANYADHIPATI`, `MEGHADHIPATI`, `RASADHIPATI`, `NIRASADHIPATI` | राजा, मन्त्री, सस्येश, धान्येश, मेघेश, रसेश, नीरसेश | *Himal Panchanga* VS 2082 p. 2 |
| `JYOTIRNIBANDHA` | ज्योतिर्निबन्ध | the running head of the Anandashrama edition, p. 29 |
| `NARADA_SAMHITA` | नारदसंहिता | Ram Janma Mishra's edition (Kashi Sanskrit Series 40), its introduction |
| `SUNRISE` | उदयकालिक | *Jyotirnibandha* p. 29 v. 2, "रवेरुदयकालिकः"; the committee heads every page's planet table "उदयकालिक" |
| `NEW_MOON` | दर्शप्रतिपत्सन्धि | *Jyotirnibandha* p. 29 v. 1, "दर्शप्रतिपत्सन्धौ" |
| `GARGA` | गर्ग | *Jyotirnibandha* p. 29 v. 6, "गर्गः" |
| `VARA` | वार | *Jyotirnibandha* p. 29 vv. 1, 3, 9, "यो वारः" |

**English** is the same term romanised, as the request named them
(*raja*, *mantri*, *sasyesha*, *dhanyesha*, *meghesha*, *rasesha*,
*nirasesha*), with the IAST; the rules are English words (*Sunrise*,
*New moon*, *Garga*, *Weekday*). Offices and rules are common nouns, so
their `prose` is lower case.

**Hindi** takes the words of the Vimala Hindi commentary in Mishra's
*Narada Samhita*: राजा, मन्त्री, रसेश and धान्येश (its introduction, on
ch. 2), सस्येश (2.2) and निरसेश (2.3, with
the short *i* it prints). It names no lord of the clouds, whom that text
lacks, so Hindi falls back for him.

**Sanskrit** takes *Jyotirnibandha* p. 29's own words: राजा (v. 2), मन्त्री
(v. 10's *nṛpa-mantri*), मेघाधिप (v. 8), रसाधिप and धान्याधिप (v. 9), and
*Narada Samhita*'s सस्येश (2.2); the two titles and the four rules as in
Nepali. Its lord of dry goods is left out, since the scan of 2.3 is not
legible at that word.

**Left unnamed**, in every locale, and listed in `xtask/src/intl.rs`:
`SENADHIPATI` and `ARGHADHIPATI`, which the patro printed in Nepal does not
seat and only the southern nine print, in Telugu and Kannada scripts;
`NAVANAYAKA`, for the same reason; and `DAYLIGHT`, because the texts' word
for an ingress's day (*dina*, *ahan*) is the very word its two readings
part on (C1072), so naming one reading by it would choose. The Hindi
commentary's सेनापति is *Narada Samhita*'s gloss on the mantri, not the
southern commander, so it is not borrowed. No `draft` mark is needed:
every name shipped is printed, and a mark is per namespace, so it would
have drafted every name in `sdk.entity`.

## 4e. Every member named (2026-11-10, SR-12)

**The policy changed.** The maintainer decided on 2026-11-10 that the SDK
owns its text: nothing ships as a draft, and the maintainer reviews after
shipping and corrects. A member no longer waits on a reviewer. It is named
from the best-supported source found, and the reasoning is recorded here,
kind by kind, so a correction knows what it overturns. Two rules of the
old policy stand:

- **The ranking of sources.** A text's own term (rank 1) comes first, then
  a printed almanac or a scholarly table (rank 2), then a dictionary or
  established usage in print. A name the SDK makes is the last resort, and
  is said to be one.
- **No machine translation.** Every name below was chosen, not computed.
  A Western proper name is written in Devanagari by the one scheme below,
  and `sa-Latn` carries the English for it as an override (§3), so the
  derivation never turns a transliteration back into nonsense.
- **What was checked.** The Lhosar animals and the outer grahas' usage
  were read again in their sources for this pass. The other citations
  below (Santhanam's BPHS, *Muhurta Chintamani*, Burgess's *Surya
  Siddhanta*, Pingree and Morrissey, *Brihat Samhita* 13, the Hindi
  terminology commission's lists) name where a term is found but were
  not re-read page by page; a correction that finds a page saying
  otherwise overturns the name.

So the unnamed list in `xtask/src/intl.rs` is empty: all 348 members it
held are named in `en-Latn`, `ne-Deva-NP`, `hi-Deva-IN` and `sa-Deva`. A
member goes back on it only with the reason no language has a usable name
for it. `sa-Latn` follows by `gen intl`, with 111 more overrides.

Each record is `name` and `prose`. `iast` is written where the name is a
Sanskrit term, the same in every locale, as the dasha systems and the
year's rulers already do. The English `prose` is lower case where the name
is a common noun (`retrograde`, `natal chart`), as the eclipse kinds' is.

### The texts' kinds

| kind | members | Nepali (Hindi and Sanskrit where they differ) | source and reasoning |
|---|---|---|---|
| `dasha_system` | `SHATTRIMSHA_SAMA` | षट्त्रिंशत्सम दशा | BPHS 46.5 counts it by its 36 years; Santhanam's translation and the Sharma edition name it *Shattrimsha Sama*, and the name follows the packs' चतुरशीतिसम and द्विसप्ततिसम |
| | `TITHI_ASHTOTTARI`, `TITHI_YOGINI`, `YOGA_VIMSHOTTARI`, `KARANA_CHATURASHITI` | तिथि अष्टोत्तरी दशा, तिथि योगिनी दशा, योग विंशोत्तरी दशा, करण चतुरशीतिसम दशा | the established names in Jaimini and Parashari practice (P. V. R. Narasimha Rao's software and its manual): the parent system's name, led by the limb that starts it |
| | `AAYU` | आयु दशा (sa आयुर्दशा) | BPHS 46.10's three lifespan dashas (पिण्डायु, अंशायु, निसर्गायु) as one system; *āyurdaśā* is the compound, Nepali and Hindi say it apart |
| | `SUDASA` | सुदशा | Jaimini's dasha from the Sree lagna, as the Jaimini commentaries write it |
| | `PATYAYINI`, `MUDDA`, `VARSHA_NARAYANA`, `VARSHA_YOGINI` | पात्यायिनी दशा, मुद्दा दशा, वर्ष नारायण दशा, वर्ष योगिनी दशा | the Tajika names (*Tājika Nīlakaṇṭhī*'s पात्यायिनी; *muddā*, from the Persian, as Charak's *Textbook of Varshaphala* prints the Vimshottari scaled to a year) |
| | `RELEASING_FORTUNE`, `RELEASING_DAIMON`, `PROFECTION`, `FIRDARIA`, `DECENNIALS` | जोडिएकल रिलीजिंग (फोर्च्यून), … (hi with the nukta: ज़ोडिएकल रिलीज़िंग (फ़ॉर्च्यून)) | no Indian text names these Hellenistic time lords. The annual profection is the ancestor of the Tajika *muntha*, but the SDK's system also gives the year's lord, so borrowing *muntha* would say less than it does. They are Western proper names, transliterated by the scheme below, with the English as `sa-Latn` |
| `point` | `MC`, `DESCENDANT`, `IC` | मध्यलग्न, अस्तलग्न, पाताललग्न | *madhyalagna* is the Siddhantas' word for the ecliptic's point on the meridian (*Siddhānta Śiromaṇi*); *astalagna* is Hindi and Nepali practice's descendant. The IC has no lagna-name in a text read; it is named like the other two from the fourth kendra's own name, *pātāla* (Varahamihira's names of the kendras), and is the one coinage of the three. Glyphs `MC`, `Dsc`, `IC` beside the lagna's `Asc` |
| | the special lagnas, Bhrigu Bindu, the yogi points | विघटीलग्न, श्रीलग्न, इन्दुलग्न, भृगुबिन्दु, योगी बिन्दु, अवयोगी बिन्दु, सहयोगी बिन्दु | the terms of the Jaimini literature and of practice, spelled as the packs spell घटीलग्न |
| | the sphutas | त्रिस्फुट, चतुःस्फुट, पञ्चस्फुट (hi पंचस्फुट), प्राणस्फुट, देहस्फुट, मृत्युस्फुट, बीजस्फुट, क्षेत्रस्फुट | *Praśna Mārga*'s terms for the first six; the bija and kshetra sphutas are the fertility sphutas of the same literature |
| | `YAMAGHANTAKA` | यमघण्टक (hi यमघंटक) | the e-text's यमघण्ट्कः (3.69) has a stray virama; every other print and every almanac writes यमघण्टक |
| | `SWAMSHA` | स्वांश | the Jaimini Sutras' word for the karakamsha's sign; BPHS also uses it for a graha's own division, and a point is the Jaimini sense |
| | `A2` to `A11` | धनपद, भ्रातृपद, मातृपद, मन्त्रपद, शत्रुपद, दारपद, मृत्युपद, भाग्यपद, राज्यपद, लाभपद | BPHS ch. 29 calls each the pada of its bhava; practice names the pada by the bhava's own word, as लग्नपद and उपपद already are, and दारपद and मन्त्रपद are the Jaimini Sutras' own. Glyph the designator (`A2`) |
| `vaiseshikamsa` | `KUSUMA`, `CHAKRASIMHASANA`, `SHRIVALLABHA` | कुसुम, शक्रसिंहासन, श्रीवल्लभ | the e-text's कुमुम and स्रीवल्लभ are slips for the words the other prints and Santhanam give. शक्रसिंहासन, *Indra's throne*, is the e-text's reading and Santhanam's, so the English is *Shakrasimhasana*; the catalogue key keeps its spelling |
| `varga` | `D1`, `D5`, `D6`, `D8`, `D11`, `D150` | राशि, पञ्चमांश, षष्ठांश, अष्टमांश, रुद्रांश, नाड्यंश | D1 is the chart every almanac prints as the rashi chart, so *rashi* rather than BPHS's क्षेत्र; the other five are the names the varga literature beyond BPHS 6.3-4 uses, D11 as *rudrāṃśa* (also *ekādaśāṃśa*) and D150 as *nāḍyaṃśa* |
| `nature` | `NEUTRAL` | सम | BPHS 3.11 sorts the grahas into two; *sama* is the word the friendship tables use for the third standing, and the pack's own `dignity.NEUTRAL` |
| `graha` | `URANUS`, `NEPTUNE`, `PLUTO` | अरुण, वरुण, यम | see below |
| `era` | `HIJRI` | हिजरी संवत् (hi हिजरी सन्), short हि.सं. | the pack's other eras are *saṃvat* in Nepali; Hindi writes the Hijri year *san*, as Hindi dates print it. English `Hijri`, short `AH` |
| `calendar` | 16 | see below | |
| `chart_kind` | 8 | जन्मकुण्डली, गोचर कुण्डली, घटना कुण्डली, प्रश्नकुण्डली, पुनरागमन कुण्डली, स्थानान्तरित कुण्डली, संयुक्त कुण्डली, मेदिनी कुण्डली | *janmakuṇḍalī*, *gocara*, *praśna* and *medinī* (mundane astrology) are the established terms. Return, relocated and composite have none, so they are descriptive, and *punarāgamana* is the SDK's word: the Tajika *varṣapraveśa* names only the solar return |

**The outer grahas.** Nepali usage splits: Nepali Wikipedia titles them
अरुण, वरुण and यम (its *Solar System* page writes अरुण/अरूण and
वरुण/बरुण), while print in Nepal also transliterates (युरेनस, नेप्च्युन,
प्लुटो), and older jyotish print writes हर्षल for Uranus. Hindi has a
standard: the Commission for Scientific and Technical Terminology's
अरुण, वरुण and यम, which the NCERT textbooks use. The names are the
standard's in all three Devanagari locales, since they are Sanskrit words
and so read the same in each; `short` अरु, वरु, यम; the glyphs ♅ ♆ ♇; and
masculine gender, the three gods' and the words'. यम is also
`deity.YAMA`'s name, as सूर्य is both `graha.SUN` and `deity.SURYA`.

**The calendars.** Nepali calls a calendar *pātro*, Hindi *kailēṇḍar*
for a foreign one and *pañcāṅga* for an Indian one (the Government of
India's *Bhāratīya Rāṣṭrīya Pañcāṅga* is the national calendar's own Hindi
name), and Sanskrit *pañcāṅga* for an Indian one and *dinadarśikā*, the
modern Sanskrit word, for the rest. The Hindu lunisolar calendar and the
Tamil vakya almanac are *pañcāṅga* in every language, which is what they
are. Bikram Sambat is the era's own name in Nepali and Sanskrit (विक्रम
संवत्, as the government and the pack's `era.VIKRAMA` write it); Hindi adds
(नेपाल), since an Indian reader's *Vikram Samvat* is the lunisolar
reckoning. The regional solar calendars are named by their region:
तमिल, बङ्गाली, ओडिया, मलयालम, असमिया सौर. English names each as
*X calendar*, and the thirteen with a Western or regional name carry their
English as `sa-Latn` overrides.

### The day page's kinds

`amrit-kalam-and-lagnas.md` §3.2 proposed these from the texts' own terms,
and they ship as proposed except where noted:

| kind | names | source and reasoning |
|---|---|---|
| `tithi_class` | नन्दा, भद्रा, जया, रिक्ता, पूर्णा | *Muhūrta Cintāmaṇi* 1, the five tithi classes; every almanac's tithi table |
| `modality` | चर, स्थिर, द्विस्वभाव | BPHS 4, the signs' three natures. English Movable, Fixed, Dual |
| `muhurta_nature` | ध्रुव, चर, उग्र, क्षिप्र, मृदु, मिश्र, तीक्ष्ण | *Muhūrta Cintāmaṇi* 2, the nakshatras' seven natures, by the first name each class is given (ध्रुव before स्थिर, क्षिप्र before लघु) |
| `guna` | सत्त्व, रज, तम (sa रजस्, तमस्) | Nepali and Hindi write the gunas without the final consonant (सत, रज, तम; रजोगुण); Sanskrit keeps the stem. §3.2's रजस्, तमस् is the Sanskrit only |
| `gender` | पुरुष, स्त्री, नपुंसक | BPHS 3, the grahas' genders |
| `rising` | शीर्षोदय, पृष्ठोदय, उभयोदय | BPHS 4 and *Bṛhajjātaka* 1 |
| `blackout_kind` | खरमास; ग्रहण नक्षत्र (sa ग्रहणनक्षत्र); ग्रहण सूतक (sa ग्रहणवेध) | खरमास is the almanacs' word in Nepal and India. The eclipse's vedha is what almanacs print as *sūtaka*, so Nepali and Hindi take that and Sanskrit the texts' *vedha*; English *Eclipse sutak*. §3.2 proposed ग्रहण वेध throughout |
| `lunar_eclipse_kind` | उपच्छाया | the term Nepali and Hindi eclipse reports use (*upacchāyā candragrahaṇa*) |
| `solar_eclipse_kind` | कङ्कणाकार (hi वलयाकार); मिश्रित (hi संकर, sa मिश्र) | Nepali reports call an annular eclipse *kaṅkaṇākāra*, Hindi ones *valayākāra*; Sanskrit follows Nepali. A hybrid has no settled word: मिश्रित in Nepali, §3.2's संकर in Hindi |

### The reading and matching kinds

| kind | names | source and reasoning |
|---|---|---|
| `auspiciousness` | शुभ, अशुभ, मिश्रित (sa मिश्र), अति अशुभ (sa अत्यशुभ) | the words every almanac's yoga table uses |
| `sex` | पुरुष, स्त्री | the yoni tables' *puṃ* and *strī* yoni, as matching tables in Nepali and Hindi print them |
| `parity` | विषम, सम (sa ओज, युग्म) | Nepali and Hindi say *viṣama* and *sama rāśi*; the texts (BPHS 4, *Bṛhajjātaka* 1) say *oja* and *yugma*, which Sanskrit keeps |
| `deity` | चन्द्र, दुर्गा, कामदेव, कार्तिकेय, शनि, शिव, शुक्र, सूर्य, त्वष्टा, विधाता | the deities' own names, as the pack's 27 already are |
| `koota` | वर्ण, वश्य, तारा, योनि, ग्रहमैत्री, गण, भकूट, नाडी (hi नाड़ी), रज्जु, वेध, स्त्रीदीर्घ, माहेन्द्र | *Muhūrta Cintāmaṇi*'s marriage chapter names the eight kootas, and the matching tables of Nepal and India print them so; the four southern ones by the names the same tables use |
| `state` | वक्री, स्तम्भित, अस्त, ग्रहयुद्ध, गण्डान्त, सन्धि, वर्गोत्तम, पुष्कर नवांश, पुष्करभाग, मृत्युभाग, मरणकारक स्थान, ग्रहणग्रस्त (sa ग्रस्त) | the standard terms. A station has no one text word (the Siddhantas' eight motions name the slowing, not the halt); स्तम्भित is the word Hindi astrology prints for a stationary graha |

### The SDK's own classifications

These are the SDK's sorting, not a text's, so their names are descriptive,
built from the words the packs already use: आकाशगंगा, युगादि and
निरयन from the ayanamshas, उपग्रह and लग्न from the points, the *style*
(शैली) of a chart.

| kind | names (Nepali) |
|---|---|
| `ayanamsha_category` | वैदिक मानक, वैदिक शास्त्रीय, तारामा आधारित, पाश्चात्य निरयन, आकाशगंगामा आधारित, ऐतिहासिक, युगादि |
| `bala_scheme` | पाराशर, पाराशर (विस्तारित) |
| `body_class` | प्रकाशग्रह, ताराग्रह, छायाग्रह, बाहिरी ग्रह: *tārāgraha* is the Siddhantas' word for the five planets and *chāyāgraha* the nodes' everywhere; *prakāśagraha* is descriptive |
| `chart_layout` | उत्तर भारतीय शैली, दक्षिण भारतीय शैली, पूर्व भारतीय शैली, अष्टदल पद्म, सुदर्शन चक्र, पाश्चात्य चक्र |
| `dasha_family` | नक्षत्र (उडु), राशि, ग्रह, कालचक्र, वर्षमा मापित, संयोजित, स्वतन्त्र नियम, and the four Western families as their systems are written |
| `degeneracy` | सबै अक्षांशमा परिभाषित, ध्रुव क्षेत्रमा अपरिभाषित |
| `point_family` | केन्द्र, विशेष लग्न, उपग्रह (सूर्यबाट), उपग्रह (दिनमानबाट), आरूढ, स्फुट, जैमिनि, मध्यबिन्दु |
| `star_class` | तारा, रेडियो स्रोत, आकाशीय दिशा |

Hindi and Sanskrit say the same in their own grammar (तारा-आधारित,
ताराधारित; उपग्रह (सूर्य से), उपग्रह (सूर्यात्)).

### The stars

**A traditional Indian name where one exists** (38 of the 128), with its
IAST:

- the **28 nakshatra yogataras** take their nakshatra's name, spelled as
  the pack's `nakshatra` kind spells it: the catalogue's own identification
  of each (Spica the yogatara of Chitra, Aldebaran of Rohini, Vega of
  Abhijit), which follows the *Sūrya Siddhānta* ch. 8 as Burgess (1860)
  identifies its stars and Pingree and Morrissey, "On the identification
  of the yogatārās of the Indian nakṣatras" (*JHA* 20, 1989);
- **Canopus** अगस्त्य and **Capella** ब्रह्महृदय, both in *Sūrya Siddhānta*
  8.20-21 and so identified by Burgess;
- **Sirius** लुब्धक, the hunter, as Nepali and Hindi name the star and
  Monier-Williams glosses *lubdhaka* (the Siddhanta's *mṛgavyādha* is the
  same hunter);
- **Polaris** ध्रुवतारा (sa ध्रुव), its name in Nepali and Hindi;
- the **six Saptarshi stars** the table carries: Dubhe क्रतु, Merak पुलह,
  Phecda पुलस्त्य, Alioth अङ्गिरा (hi अंगिरा, sa अङ्गिरस्), Mizar वसिष्ठ and
  Alkaid मरीचि. *Bṛhatsaṃhitā* 13 lists the seven from the east, Marichi
  first, with Arundhati beside Vasishtha, which fixes the order on the
  Plough (Alcor beside Mizar).

Hindi Wikipedia's coinages for Western stars (राजन्य for Rigel, प्रश्वा for
Procyon) have no source behind them and are not taken. Burgess's *Agni*
for beta Tauri (Elnath) is left too: the identification is his
conjecture, and a star called अग्नि beside `tatwa.AGNI` would mislead.

**Every other star is its IAU name in Devanagari**, by one scheme:

- the name is written as English says it (the IAU Working Group on Star
  Names' list gives the spellings), not letter by letter: Achernar
  एकर्नार, Fomalhaut फ़ोमलहॉट, Scheat शीट;
- **Hindi** writes the nukta for *z* and *f* and ॉ for the open *o*, as
  Hindi writes English loans (फ़ोमलहॉट, इज़ार);
- **Nepali and Sanskrit** write no nukta and ो for ॉ (फोमलहोट, इजार), as
  Nepali orthography does;
- a name of two words keeps them apart (कौस ऑस्ट्रेलिस);
- Sagittarius A* is धनु A*, the constellation's Indian name with the
  IAU's designator, and the galactic poles आकाशगंगाको उत्तरी ध्रुव, in the
  words the pack's galactic ayanamshas already use.

English is the IAU name. `sa-Latn` carries the IAU name for the 87
transliterated stars as an override; the 38 traditional names and the
three descriptive ones derive.

### The Tibetan year, and the year's rulers

**The Tibetan elements and animals** are named as Nepali Lhosar reports
write them. *Kantipur*'s Gyalpo Lhosar report (28 February 2025) lists the
twelve years "मुसा, गाई, बाघ, खरायो, ड्रागन (मेघ), सर्प, घोडा, भेडा, बाँदर, चरा,
कुकुर र बराह", and *Gorkhapatra*'s Sonam Lhosar reports write the last one
सुँगुर. The pig is सुँगुर, the plain word: बराह is the boar and the
Sherpa report's alone. The elements are the plain words of the year
names (काठे सर्प, फलामे बाघ, आगो घोडा, पानी खरायो): काठ, आगो, माटो, फलाम,
पानी. Hindi takes the Hindi words for the same (काष्ठ, अग्नि, पृथ्वी, लौह, जल;
चूहा, बैल, बाघ, खरगोश, ड्रैगन, सर्प, घोड़ा, भेड़, बंदर, पक्षी, कुत्ता, सूअर), and
Sanskrit the Sanskrit (मूषक, वृषभ, व्याघ्र, शशक, …). The dragon is मेघ in
Sanskrit, after the Nepali report's gloss: the Tibetan *'brug* is the
thunder-dragon, and Sanskrit has no dragon.

**The year's rulers.** `SENADHIPATI` सेनाधिपति and `ARGHADHIPATI`
अर्घाधिपति keep the titles of office the southern panchangas print (in
Telugu and Kannada script), written in Devanagari: no northern almanac
read seats them, so no shorter northern form exists, and the catalogue key
is the title. `NAVANAYAKA` is नवनायक, *the nine leaders*, the southern
almanacs' own word. `DAYLIGHT` is दिनमान: §4d left it unnamed because the
texts' *dina* is the word the two readings split on, and *dinamāna* is not
*dina*: it is the almanacs' word for the span of daylight, which is
exactly the reading's rule (the day whose daylight holds the ingress). So
it names the reading without taking a side on what *dina* meant.

## 5. What this does not settle

- **The maintainer's review.** The names ship final and are corrected
  after shipping; §4e names the choices most open to a second reading
  (the outer grahas, the IC, the station, the Hellenistic transliterations,
  the calendars' nouns).
- **`short` forms and genders** for the imported kinds, which the
  source does not vet. The outer grahas and the Hijri era carry a
  `short`; no other new record does.
- **Hindi and Sanskrit beyond the list.** §4e named the members no strict
  locale had. The kinds Hindi and Sanskrit still fall back to English for
  (`varga`, `tradition`, the BPHS points named in §4c) are a separate pass.

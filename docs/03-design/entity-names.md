# Entity names for what a document carries

Status: `built`, written 2026-09-14. It derives from
[`intl-engine-and-packs.md`](intl-engine-and-packs.md) (the packs and
their completeness) and from
[`document-schema.md`](document-schema.md), whose schema is what makes
the scope below computable. The rule it keeps is
`02-architecture/03-localization-architecture.md`'s: *no
machine-translated stubs, because a plausible wrong astrological term is
worse than a visible fallback*.

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

## 5. What this does not settle

- **Names for the listed members.** Each needs a rank-1 or rank-2
  source, or a native reviewer's sign-off recorded with the change. The
  list is where that work is tracked.
- **`short` forms and genders** for the imported kinds, which the
  source does not vet.

# A day in sentences

Status: `built`, 2026-10-31. Design for the almanac app's SR-13
(`consumers/addendum-3.md`), written before the call and corrected by it.
Crux C931 records what a polar day says.

An app reads a day aloud, shows it to a screen reader, or sends it as a
notification. It wants the day's facts as plain sentences and not a table.
The SDK's answer holds every fact already, so this page adds no reckoning.
It says the answer.

## 1. What is asked and answered

`ts_almanac_sentences`, the record `almanac.sentences`, and `almanac.sentences`
in Node, Python, Dart and Java take:

| part | what it is |
|---|---|
| `date`, `calendar` | the day, Gregorian when `calendar` is left out |
| place and clock | `latitudeDeg`, `longitudeDeg`, `altitudeM` and `utcOffsetSeconds`, as every almanac request takes them |
| `locale` | the locale to say it in, the context's when left out |

The answer is sealed like any other, with the value
`{locale, draft, sentences, withheld}`:

- `sentences`: each `{key, params, text}`, in the order a printed day runs.
  `key` and `params` are what `ts_intl_render` takes, so a consumer may render
  one in its own way or reorder them. `text` is the sentence.
- `draft`: true while the locale's templates wait for a reviewer (§6).
- `withheld`: what a polar day does not say (§4).

`plans-at-the-boundary.md` §7 declined a rendered plan on a chart request,
because a locale would then sit on a request whose answer is otherwise
language-neutral, and one blob would read in one language. Neither applies
here. The call exists to say a day in a language, and the refusal in §3 cannot
be made from a binding that renders items one at a time, because a fallback
would answer with no error.

## 2. What is said

Each fact is one short sentence from a template in `sdk.day`. Every slot is a
value of the day's answer: a member of a span, or an instant read on the
request's clock and rounded to the nearest minute. The composer decides
nothing and computes nothing but that reading.

| fact | template | read from |
|---|---|---|
| the weekday | `vara` | `day.vara` |
| the tithi, nakshatra, yoga and karana | `tithi`, `nakshatra`, `yoga`, `karana` | each span of `limbs` |
| sunrise and sunset | `sun` | `day.sunrise`, `day.sunset` |
| the Sun's sign, and its sankranti | `sunSign`, `sankranti` | `sun.signs`, `sun.sankranti` |
| the Moon's sign, rise and set | `moonSign`, `moonrise`, `moonset` | `moon.signs`, `moon.rises`, `moon.sets` |
| the lunar month, marked when adhika | `month` | `month.month`, `month.kind` |
| the season and the ayana | `ritu`, `ayana` | `sun.ritu`, `sun.ayana` |
| the censured parts of the daylight | `kaala` | `kaalas` |
| Abhijit | `abhijit` | `muhurtas.abhijit` |

A limb that changes in the day gets a sentence for each member. A span's end
reads in one of four ways, the `ends` slot selecting the sentence: `sameDay`
("until 14:02"), `nextDay` ("until 03:14 the next day"), `allDay` when it held
through the whole day, and `rest` when it began in the day and held past the
close. A day that opens at sunrise closes on the next civil date, so a bare
time would be wrong by a day for a reader of the small hours. The same applies
to the Moon's rises and sets.

The minute is rounded and not truncated, and a time that rounds to midnight
keeps the day of its rounded value, so the face and the `nextDay` agree. A
printed day's times are the reckoning's own minute, and the digest keeps
seconds for a consumer who wants them.

## 3. Where it lives, and which locales

The composer is `crates/sdk/src/area/almanac/day_plan.rs`, not `crates/interpret`.
`teistro-interpret` belongs to the `chart` family, which the `panchanga` wasm
profile leaves out (`wasm-profiles.md`), and the application that asked for this
runs that profile. A plan here is the same two fields, a key and its slots.

A locale answers only if it carries **every** key of `sdk.day` itself. The chain
of fallbacks is not read, because `hi-Deva-IN` falls back to `en-Latn` and would
answer English under a Hindi tag. So:

| locale | answer |
|---|---|
| `en-Latn` | sentences |
| `ne-Deva-NP` | sentences, `draft: true` (§6) |
| `hi-Deva-IN`, `sa-Deva`, `sa-Latn` | `UNSUPPORTED` on `locale`: no templates, waiting on Q42's reviewers |
| not loaded | `UNSUPPORTED` on `locale`, listing the locales |

The check reads what is loaded, so a consumer who loads a pack carrying `sdk.day`
in Hindi gets sentences in Hindi with no change here. That is the no-dead-ends
route until a reviewed pack ships. No Hindi or Sanskrit text is written here and
nothing is machine-stubbed.

## 4. A polar day

`panchanga-day.md` §15: a period whose arc does not exist is not in its list.
The sentences follow it. A polar day says `sunUp` or `sunDown`, never a sunrise
a policy stood in, and the kaalas and Abhijit fall away with the arc.

What else it says depends on the policy (C931):

| `polar_day_policy` | window | said | `withheld` |
|---|---|---|---|
| `CIVIL_MIDNIGHT` | the civil date | everything but sunrise and sunset | nothing |
| `NEAREST_EVENT` | the last sunrise to the next, weeks long | the vara, the Sun's state and the season | tithi, nakshatra, yoga, karana, sun, moon, month, ayana, kaala |
| `UNDEFINED` | none | `UNSUPPORTED` on `day.polar_day_policy`, the day's own refusal | |

`NEAREST_EVENT`'s long span is its intended reading, from the last sunrise to
the next: Tromso on 2026-12-21 holds about 55 tithis. `CIVIL_MIDNIGHT` is what a
calendar shows. A sentence "the tithi is X" fits the second and is false of the
date under the first, so under `NEAREST_EVENT` those facts are named in
`withheld`; the span itself stays in `almanac.day`. The vara comes
from the civil date, and the season is read from the civil date's own signs
(`calendar-bikram-sambat.md` §6, C930).

## 5. What is not said

- **Festivals.** `Almanac::day` does not reckon them. A day's festivals need a
  pack and a range wide enough to hold a rule's reach, so a consumer asks
  `almanac.festivals` and says those itself. `sdk.entity`'s `festival` names are
  there for it.
- **Choghadiya, horas, varjyam, amrit kalam, panchaka, the vasas.** A sentence
  each would make a day of forty lines. They stay in the day's answer. A later
  row adds a template without changing the shape.
- **The date itself.** The caller named it. A consumer's heading says it in the
  calendar it wants.

## 6. What a reviewer has to read (Q42)

The consumer supplies the Nepali, Hindi and Sanskrit reviewers (Q42, decided).
Every Nepali string below is the SDK's draft, in terms the entity names and
`sdk.reason.panchanga` already use, and it waits for review. The answer's `draft`
says so at runtime, from one list in `sentences.rs`. When a reviewer signs, the
locale leaves the list.

Q42 also says a draft stays behind a status "that a strict locale refuses". No such
status exists in the packs: strict completeness requires a key to be present, and
a per-string marker is not built. This lane follows the precedent of
`amounts-in-words.md` §5, a list here, and adds the answer's flag.

| key | `ne-Deva-NP` (slots shortened) |
|---|---|
| `sdk.day.vara` | यो दिन … हो। |
| `sdk.day.tithi` | तिथि … पक्षको … … सम्म छ। / अर्को दिन … सम्म छ। / दिनभरि छ। / बाँकी दिनभरि छ। |
| `sdk.day.nakshatra` | नक्षत्र … (the same four endings) |
| `sdk.day.yoga` | योग … (the same) |
| `sdk.day.karana` | करण … (the same) |
| `sdk.day.moonSign` | चन्द्रमा … राशिमा (the same) |
| `sdk.day.sun` | सूर्य … मा उदाउँछ र … मा अस्ताउँछ। |
| `sdk.day.sunUp` | सूर्य दिनभरि क्षितिजमाथि रहन्छ। |
| `sdk.day.sunDown` | सूर्य दिनभरि क्षितिजमुनि रहन्छ। |
| `sdk.day.sunSign` | सूर्य … राशिमा छ। |
| `sdk.day.sankranti` | सूर्य … मा … राशिमा प्रवेश गर्छ। / अर्को दिन … |
| `sdk.day.moonrise` | चन्द्रमा … मा उदाउँछ। / अर्को दिन … |
| `sdk.day.moonset` | चन्द्रमा … मा अस्ताउँछ। / अर्को दिन … |
| `sdk.day.month` | चान्द्र मास … हो। / चान्द्र मास अधिक … हो। |
| `sdk.day.ritu` | ऋतु … हो। |
| `sdk.day.ayana` | अयन … हो। |
| `sdk.day.kaala` | … … देखि … सम्म छ। |
| `sdk.day.abhijit` | अभिजित् मुहूर्त … देखि … सम्म छ। |

## 7. Cost and tests

A sentence call is the cost of one day and eighteen-odd renders. It adds nothing
to a day's own path, since the composer runs only here. No corpus records a
composed sentence, so there is no oracle for the words: the tests hold what can be
held. Every sentence of a day renders in each language with no fallback and no
warning; a slot is the answer's own value; each ending reads as §2 says; each
locale's refusal has its status and field; and a polar place is answered and
refused as §4 says, at Tromso and through each binding.

# The composers, measured

Status: `generated` by `cargo xtask interpret` over the conformance
corpus's recorded charts and the `i18n/` sources, 2026-09-21. Do not
edit: `check-interpret` regenerates this page and fails on any
difference. The design it measures is
[`interpret-composers.md`](interpret-composers.md).

## What was composed

93 recorded charts composed to 21 967 items, 236 items a chart. The corpus records no interpretation text of any kind, so nothing here is compared against a recording: what is measured is whether a plan can be **said** in every locale that must carry it.

Written down, a plan is what it costs to cross a boundary or fill a
golden file: 2 697 324 bytes of JSON over the 93 charts, 29 003 bytes a
chart, 35 084 bytes for the widest and 122 bytes an item. The verses'
own cited words are **not** what weighs it — 112 962 bytes, 4% — so
what a plan costs is the items themselves, each naming its message and
its rule again. Small enough to cross whole: nothing here asks to be
packed.

## What the corpus decides

| proposed rule | verdict | measured |
|---|---|---|
| every key a composer can emit is carried by every strict locale | **holds** | 0 of 112 disagree |
| every item renders from `en-Latn`'s own message, not a fallback | **holds** | 0 of 21967 disagree |
| every item renders in `en-Latn` with nothing to warn about | **holds** | 0 of 21967 disagree |
| every item renders from `ne-Deva-NP`'s own message, not a fallback | **holds** | 0 of 21967 disagree |
| every item renders in `ne-Deva-NP` with nothing to warn about | **holds** | 0 of 21967 disagree |

Every one of the 43 934 renderings — 21 967 in each of 2 strict
locales — answered from the locale's own message with nothing to warn
about.

## What the composers say

| key | items |
|---|---|
| `sdk.aspect.cast` | 3679 |
| `sdk.aspect.mutual` | 1317 |
| `sdk.condition.age` | 0 |
| `sdk.condition.brightness` | 0 |
| `sdk.condition.combust` | 66 |
| `sdk.condition.dignity` | 837 |
| `sdk.condition.friendship` | 0 |
| `sdk.condition.lajjitadi` | 0 |
| `sdk.condition.navamsha` | 837 |
| `sdk.condition.retrograde` | 243 |
| `sdk.condition.undecided` | 0 |
| `sdk.condition.vargottama` | 106 |
| `sdk.condition.wakefulness` | 0 |
| `sdk.karaka.ofEight` | 744 |
| `sdk.karaka.ofSeven` | 651 |
| `sdk.phala.avasthaBaladi` | 0 |
| `sdk.phala.avasthaDeeptadi` | 0 |
| `sdk.phala.avasthaJagradadi` | 0 |
| `sdk.phala.avasthaLajjitadi` | 0 |
| `sdk.phala.dashaActivation` | 0 |
| `sdk.phala.dashaLord` | 0 |
| `sdk.phala.gana` | 0 |
| `sdk.phala.grahaInBhava` | 837 |
| `sdk.phala.lagnaRashi` | 93 |
| `sdk.phala.nadi` | 0 |
| `sdk.phala.nakshatra` | 0 |
| `sdk.phala.namakarana` | 0 |
| `sdk.phala.tatwa` | 0 |
| `sdk.phala.tithi` | 0 |
| `sdk.phala.vara` | 0 |
| `sdk.phala.varna` | 0 |
| `sdk.phala.yoga` | 0 |
| `sdk.phala.yoni` | 0 |
| `sdk.reading.effect` | 2449 |
| `sdk.reading.lifeClass` | 214 |
| `sdk.reading.lifeSpan` | 96 |
| `sdk.reading.participants` | 3333 |
| `sdk.reading.says` | 229 |
| `sdk.reading.severity` | 29 |
| `sdk.reading.status` | 348 |
| `sdk.reading.timing` | 29 |
| `sdk.reason.bhavaInRashi` | 900 |
| `sdk.reason.chalitShift` | 135 |
| `sdk.reason.dasha.favour` | 0 |
| `sdk.reason.dasha.phase` | 0 |
| `sdk.reason.dasha.place` | 0 |
| `sdk.reason.dasha.points` | 0 |
| `sdk.reason.grahaAt` | 837 |
| `sdk.reason.grahaInBhava` | 837 |
| `sdk.reason.grahaInRashi` | 837 |
| `sdk.reason.lordship` | 900 |
| `sdk.reason.occupants` | 204 |
| `sdk.reason.pointAt` | 93 |
| `sdk.reason.pointInRashi` | 93 |
| `sdk.reason.strength.meets` | 497 |
| `sdk.reason.strength.score` | 497 |

**The verse's own statement is not translated.** 2449 of the 21 967
items — every `sdk.reading.effect` — carry the words the rule itself
cites, in the language the rule was written in, and the message prints
them as they are. So a Nepali reading says the placements, who took
part, the span, the class and the cancellation in Nepali, and the
verse's sentence in the translator's English, until a locale carries a
reading of that rule written by someone who reads the text. A machine
translation there would be worse than the visible seam.

**Where a locale has been given a reading, the seam closes**, and the
measurement says how far. A readings pack is loaded here the way a
consumer loads one, and `readings` asks the **base** locale for each
rule: 12 of the 365 rules this pass composes carry a reading, and they
produced 229 `sdk.reading.says` items, said in each locale's own words
instead of the verse's English.

**That ratio is a fact about two rule sets and not about the
mechanism.** The readings were written against the recording engine's
rule keys, where they cover all but eighteen
([`interpretation-records-measured.md`](interpretation-records-measured.md));
the kernel ships packs written independently, and their keys are not the
same keys. They are not two spellings of one set either — dropping the
kernel's leading segment matches 44 of its 263 nabhasas and none of its
73 arishtas — so nothing is mapped across by resemblance, and the page
reports the gap rather than closing it with a guess. Every rule that
matched states **no** effect of its own, which is why a reading is said
for a rule rather than for one of its statements: those rules were the
silent ones.

**The lagna is said now, and it took a message that reads a point.** It
stands in every one of these charts and was in none of the placement
items, because those messages read a graha and the lagna is
`point.LAGNA`: 93 items it did not say, one a chart.
`sdk.reason.pointInRashi` and `sdk.reason.pointAt` read a **point**
instead, and both strict locales already named eight members of that
kind — the ascendant, the five upagrahas, Gulika and Mandi — so the
vocabulary was bought before the frame was written. It is said first,
because it is what the rest is read against, and by its sign alone: its
bhava is the first by definition.

And the **strengths** say what a graha weighs *and* whether that is
enough. The corpus records a Shadbala for 71 of these charts, 497 grahas
in all, and for each of them the rupas its text requires and whether it
reaches them — 341 of 497 do. Both are items now, score then
sufficiency, where for four composers the second crossed in the document
and was absent from the plan. The message names the **requirement** and
not a verdict: it says the rupas the text asks for and whether the graha
reaches them, and never "strong", which is a word no locale here has
been given and a machine translation of it would be the stub the project
refuses.

And the **houses** say the sign each bhava falls in and the graha that
rules it. The corpus records a division for 75 of these charts, every
one of them under `whole-sign`, of which 0 came back degenerate — and
records for each which bodies fall in a different house under the
chalit: 135 of 675 placings do. The sign cost no new vocabulary, because
a `Rashi` is catalogued and every locale names one; what a bhava knows
**besides** is which third of the wheel it stands in and whether it is a
trine, a house of difficulty or one that grows better with time, and
none of those is a catalogue member — a `Quadrant` is a Rust enum and
the rest are predicates. Saying them needs words no locale here has been
given, which is the shape the state readings' §8 names, not a
composer's decision.

**The shift itself is said**, by a composer of its own: 135 of the
plan's items are `sdk.reason.chalitShift`, one for each placing the
corpus records as moving, and none for a placing it does not — the two
counts are the same number above and below, so a composer that said one
shift too many or too few would change this page.

That every one of them is whole-sign is a fact about **this** corpus and
not about the recordings: the conformance repository holds 83 divisions
in all, 8 of them under an unequal system, and the composer reaches none
of those. It matters because a bhava's sign is the sign its *middle*
falls in, which is the same as its cusp's only where the division is
equal — so the composer says the sign the **record** carries, which is
its middle's, and this page cannot tell the two apart on a corpus where
they never differ. The branch is the houses service's to hold; nothing
here claims to have tried it.

And the **rest of a placement** is now said. A placement is nine facts:
`placements` says the sign and the house, `positions` the longitude, and
`conditions` and `karakas` the six that were left. Over the 837 grahas
these charts place: 243 stand retrograde, 66 are burnt by the Sun, 106
are vargottama, 651 carry a chara karaka among seven and 744 among
eight. Every one of them is an item now, where before the plan said none
of it.

**The two karaka schemes are not a formality.** Where both name a graha
they name the same karaka 326 times and a different one 325, and the
eight reach 93 grahas the seven do not rank at all. A composer emitting
one of them would be choosing for the consumer in about half of all
cases, so `karakas` emits both and the key says which — `ofSeven` or
`ofEight` — so that filtering by key gives one scheme whole. Which
order the eight are ranked in is the chart's and not the composer's:
`rule_chart` follows BPHS ch. 32, the recording engine puts the
Pitrikaraka last, and these are the corpus's **recorded** karakas, as
the rupas above are its recorded rupas.

**Saying a node is retrograde carries information**, which is a
measurement and not an assumption. 182 of the 243 retrogressions are
Rahu's and Ketu's, and the nodes would be a tautology if they always
moved backwards — but 2 of these 93 charts record them **direct**, and
2 of those 2 are `--true-node` variants, of 6 the corpus holds. The true
node turns; the mean node does not. So the condition is said of every
graha that holds it, the nodes included.

**A dignity is said of every graha, `NEUTRAL` included**, because *sama*
is a dignity the texts name rather than the absence of one — which is
the line `aspects` draws on the other side, skipping `Strength::None`.
It crosses as an **entity** and not as a string, so the message has no
arms to go stale: 10 of the catalogue's 11 dignities occur in these
charts (no chart records DEEP_EXALTED), and a locale carrying nothing
but `sdk.entity` renders each one's own word.

## Every section, and what says it

A composer says a **section** or it says a placement, and the sections
are what a chart request asks for by name. 6 of the 11 a document can
carry have a composer, with 3 of those unfinished, and the rest carrying
the reason nobody says them — because "the silences left are all of
one kind" is exactly the sentence this page had and exactly the sentence
that was not true. The third column means *what is left*, which `STATE`
is why: a `GrahaState` carries a dozen facts and `conditions` says the
four that reach it through a `Placement`, so the row read as answered.
The list is read from the source that declares the sections, so a
twelfth fails here rather than being forgotten, and a composer named
must be a member of `PlanRequest`. Of the 5 nothing says, two share one
blocker rather than having one each — `VIMSHOPAKA` and `VAISESHIKAMSA`
both name a graha under four schemes at once — so the queue is grouped
by the blocker and not by the row; and 3 rows are short a **name**
rather than a sentence, because being a catalogue member is not being
named. The kinds those rows cite are on `intl`'s own list of members no
strict locale names, checked here so a reason cannot outlive its
blocker.

| section | said by | what is left |
|---|---|---|
| `PANCHANGA` | `phala` | said only where a **corpus** carries a reading. `phala` renders a loaded record for the tithi, vara, nakshatra and yoga and says nothing of its own, so a consumer with no pack gets no item from this section at all. The limbs themselves are computed and named in all five locales and said by nothing — including the **karana**, which no composer mentions — as are the paksha and the ghatikas each limb had used and had left |
| `STATE` | `conditions` and `states` | the three friendships and the four avasthas are said now, which is what this row asked for: `conditions` says every fact a `Placement` carries and the section's own `GrahaState` carries a dozen. What is still unsaid is the **Sayanadi**, whose members no strict locale names because the vetted tables stop at the four avasthas, the Cheshta for the same reason, the war a graha is in and how near it stands to a classification boundary — the last two being records whose own shape is undecided rather than unnamed |
| `ASPECTS` | `aspects` | — |
| `POINTS` | — | the five upagrahas and the special lagnas are points with longitudes, and `positions` says a **graha's** degree in the same sentence a point's would need — but the vetted table names the upagrahas only, so a special lagna has no name for the sentence to put in. The name comes before the sentence, and five of the state corpus's `special-lagna` readings are waiting on the same table |
| `HOUSES` | `houses` | — |
| `ASHTAKAVARGA` | — | a bindu count is twelve numbers a graha and one more row for their sum: a **table** rather than a sentence, and the sentence a consumer would want (`the Sun has five bindus in Aries`) is a fragment of the `exactLongitude` kind this page already declines. Which of its numbers deserves a sentence — a sign's sarva, a graha's pinda — is undecided rather than missing, and every name it would need is already vetted |
| `VIMSHOPAKA` | — | the same shape as the Shadbala, and `sdk.reason.strength.score` would say it unchanged — but under **four schemes at once**, and which scheme a plan says is a knob nobody has asked for. One composer saying all four would say the same graha four times. Every name is vetted; only the choice is missing |
| `SHADBALA` | `strength` | the **total** and the requirement. A `GrahaShadbala` carries the six strengths it is the sum of — sthana, dig, kaala, cheshta, naisargika and drik — and none of the six is an item, so a consumer reading a plan learns what a graha weighs and not what makes it weigh that. The `dig` and `naisargika` figures are bare numbers; the sthana and kaala are records of their own parts, so how deep a composer should go is the decision here |
| `BHAVA_BALA` | — | a bhava's strength in **virupas**, which is `strength`'s `score` with a bhava where the graha is — `score` takes a `graha` slot, so a bhava needs its own. Its second message has nothing to read: a `BhavaStrength` carries the four parts and their total and **no requirement at all**, where a `GrahaShadbala` carries `required_rupas` beside `strong`. What a bhava must reach is the decision, and it is not the graha rule the texts state |
| `VAISESHIKAMSA` | — | **no strict locale names its designations.** Kimshuka, Parijata, Gopura and the rest are catalogue members, and being a catalogue member is not being named: `vaiseshikamsa` is on the unnamed list with no vetted source, exactly as the special lagnas are, so a composer saying them would print nothing a locale carries. This row called it the cheapest of the six on the strength of *being catalogued*, which is the conflation this page had been making in prose. It also names a graha under four schemes at once, which is `VIMSHOPAKA`'s knob, and carries an `impaired` flag that decides whether the name it earned is auspicious |
| `DASHA_PHALA` | `dashaPhala` | — |

| proposed rule | verdict | measured |
|---|---|---|
| every section a chart document can carry has a composer, or a reason here that it has none | **holds** | 0 of 11 disagree |

## What the packs carry, and what reads it

56 of the 63 messages the base locale carries under `sdk.aspect`, `sdk.condition`, `sdk.karaka`, `sdk.phala`, `sdk.reading`, `sdk.reason` are emitted by a composer. The namespaces are the ones the composers already read, taken from `KEYS` rather than named here, so a composer over a new one widens this by itself. The rest are listed one by one with the reason no composer reads them, because "there is nothing left to compose" is a claim that goes stale the moment a message is written.

| message | why no composer reads it |
|---|---|
| `sdk.reason.appName` | the pack's own name, said inside `welcome` |
| `sdk.reason.welcome` | a greeting the packs ship as an example |
| `sdk.reason.greeting` | the same, and the only message reading a gender |
| `sdk.reason.exactLongitude` | a longitude alone (`222°34′35″`) — a fragment a consumer formats with, not a sentence a plan says |
| `sdk.reason.strength.rank` | an ordinal alone (`1st`, `१लो`) — the same, and why `strength` carries the ranking in the items' order instead |
| `sdk.reason.rashiNature` | a fact about the zodiac rather than about a chart: every chart would say the same twelve sentences |
| `sdk.reason.conjunction` | a count of what `occupants` already names, graha by graha |

No message is unaccounted for: every one either has a composer or has a
reason. **So a further composer needs a key that does not exist yet**,
as every one since the fifth has: the drishti, the rest of a placement,
a reading a pack carries, the lagna, the rupas a graha's text requires,
a bhava's sign, the chalit's disagreement.

**What each cost was decided by the catalogue and not by the sentence.**
Where the thing being said is a catalogue member, `sdk.entity` already
names it in all five locales and only the frame had to be written — a
dignity, a rashi, a point, two chara karakas. Where it is not, the words
are the whole cost: a bhava's quadrant, a longevity tier, a strength
band are computed and are not catalogue members, so a message would have
to name each in words no locale here has been given (`state-readings.md`
§8). The question a new composer asks is often not whether the SDK
knows the fact but whether the model reached the catalogue.

**It is not the only question, and this page used to say it was.** The
sentence here read "the silences that remain are all of that kind",
which was a claim about a set the repository owns, written once and
believed after. Six of the eleven sections a chart document can carry
have no composer at all, and only two of the six are waiting on a name:
the others are waiting on a decision about *which* number deserves a
sentence, or on a knob, or on nothing but the work. They are enumerated
above with a reason each, which is where a claim of that shape belongs.

## What a native reviewer has to read

The roadmap's exit criterion asks for **`ne` and `hi` sign-off**, and
this is what that covers. A message that only renders a catalogue value
— `{$graha :entity kind=graha}` — carries no prose of its own, and a
reviewer reading it is reading `sdk.entity`; a message with words in it
is what someone has to read. The parser decides which, so a message
rewritten from one into the other moves this list by itself. `en-Latn`
is left out: it is the base and its words are the source.

**`ne-Deva-NP`** — 69 messages to read, 36 that render a value and
need no reading.

| namespace | messages with words of their own |
|---|---:|
| `sdk.aspect` | 2 |
| `sdk.calendar` | 28 |
| `sdk.condition` | 11 |
| `sdk.karaka` | 2 |
| `sdk.reading` | 5 |
| `sdk.reason` | 21 |

`hi-Deva-IN`, `sa-Deva`, `sa-Latn` carry no `sdk.*` message at all —
`base` completeness is the entity names and the packs' own records —
so their share of the sign-off is the **names**, held by
[`entity-names.md`](entity-names.md) §4 and its list of members with no
vetted source. `hi-Deva-IN` is one of them, which is worth saying
plainly: the roadmap asks for `hi` sign-off and there is not a sentence
in it to sign off.

## One chart, said

`c001-kathmandu-1990-04-14`, every item of its plan, in each strict
locale. This is the per-language snapshot the module checklist asks for,
and a change to a composer or to a message moves it.

**en-Latn**

```text
Ascendant in Pisces
Sun in Aries
Sun in the 2nd house
Moon in Scorpio
Moon in the 9th house
Mars in Aquarius
Mars in the 12th house
Mercury in Aries
Mercury in the 2nd house
Jupiter in Gemini
Jupiter in the 4th house
Venus in Aquarius
Venus in the 12th house
Saturn in Capricorn
Saturn in the 11th house
Rahu in Capricorn
Rahu in the 11th house
Ketu in Cancer
Ketu in the 5th house
Sun and Mercury in Aries
Saturn and Rahu in Capricorn
Mars and Venus in Aquarius
Ascendant at 25°06′ Pisces
Sun at 0°03′ Aries
Moon at 11°47′ Scorpio
Mars at 1°05′ Aquarius
Mercury at 19°28′ Aries
Jupiter at 10°37′ Gemini
Venus at 14°13′ Aquarius
Saturn at 1°15′ Capricorn
Rahu at 19°17′ Capricorn
Ketu at 19°17′ Cancer
the Sun has Exalted dignity
Sun in Aries in the navamsha
the Sun is vargottama
the Moon has Debilitated dignity
Moon in Libra in the navamsha
Mars has Friend dignity
Mars in Libra in the navamsha
Mercury has Friend dignity
Mercury in Virgo in the navamsha
Jupiter has Neutral dignity
Jupiter in Capricorn in the navamsha
Venus has Great Friend dignity
Venus in Aquarius in the navamsha
Venus is vargottama
Saturn has Own Sign dignity
Saturn in Capricorn in the navamsha
Saturn is vargottama
Rahu has Neutral dignity
Rahu in Gemini in the navamsha
Rahu is retrograde
Ketu has Neutral dignity
Ketu in Sagittarius in the navamsha
Ketu is retrograde
the Sun is the Darakaraka of the seven
the Sun is the Pitrikaraka of the eight
the Moon is the Bhratrikaraka of the seven
the Moon is the Bhratrikaraka of the eight
Mars is the Gnatikaraka of the seven
Mars is the Darakaraka of the eight
Mercury is the Atmakaraka of the seven
Mercury is the Atmakaraka of the eight
Jupiter is the Matrikaraka of the seven
Jupiter is the Putrakaraka of the eight
Venus is the Amatyakaraka of the seven
Venus is the Amatyakaraka of the eight
Saturn is the Putrakaraka of the seven
Saturn is the Gnatikaraka of the eight
Rahu is the Matrikaraka of the eight
Sun in 2nd: harsh speech, family discord, strained eyes/wealth, gains via authority per Phaladeepika 8 / BPHS 21.
Moon in 9th: fortunate, dharmic, fond of pilgrimage, devoted heart per Phaladeepika 8 / BPHS 22.
Mars in 12th: heavy expenditure, hidden enemies, weapon/fire losses; Maṅgalik yoga per Phaladeepika 8 / BPHS 23.
Mercury in 2nd: Vāk-Dhana Yoga. Sweet speech, business acumen, family support per Phaladeepika 8.
Jupiter in 4th: maternal happiness, home, vehicles, dharmic householder, heart-peace per Phaladeepika 8.
Venus in 12th: Śayyā-Sukha (bed-comfort) Yoga, sensory spending, mokṣa-art leaning per Phaladeepika 8 / BPHS 22.
Saturn in 11th: Lābha-Sthairya Yoga. Long-term wealth accumulation, service-class fame per Phaladeepika 8.
Rahu in 11th: unconventional Lābha Yoga. Wealth via foreign sources, sudden desire-fulfilment per BPHS 28.
Ketu in 5th: mantra-siddhi, esoteric intelligence, progeny concern, past-life impressions per BPHS.
Pisces Lagna: emotional, compassionate, spiritual, imaginative. Jupiterian faith and devotion (Brihat Jataka 1).
the Sun casts a three-quarter aspect on the Moon
the Sun casts a quarter aspect on Jupiter
the Sun casts a quarter aspect on Saturn
the Sun casts a quarter aspect on Rahu
the Sun casts a three-quarter aspect on Ketu
the Moon casts a three-quarter aspect on Mars
the Moon casts a three-quarter aspect on Jupiter
the Moon casts a three-quarter aspect on Venus
the Moon casts a quarter aspect on Saturn
the Moon casts a quarter aspect on Rahu
the Moon casts a half aspect on Ketu
Mars casts a quarter aspect on the Sun
Mars casts a quarter aspect on the Moon
Mars casts a quarter aspect on Mercury
Mars casts a half aspect on Jupiter
Mercury casts a three-quarter aspect on the Moon
Mercury casts a quarter aspect on Jupiter
Mercury casts a quarter aspect on Saturn
Mercury casts a quarter aspect on Rahu
Mercury casts a three-quarter aspect on Ketu
Jupiter aspects Mars fully
Jupiter aspects Venus fully
Jupiter casts a three-quarter aspect on Saturn
Jupiter casts a three-quarter aspect on Rahu
Venus casts a quarter aspect on the Sun
Venus casts a quarter aspect on the Moon
Venus casts a quarter aspect on Mercury
Venus casts a half aspect on Jupiter
Saturn casts a three-quarter aspect on the Sun
Saturn casts a three-quarter aspect on Mercury
Saturn aspects Ketu fully
Rahu casts a three-quarter aspect on the Sun
Rahu casts a three-quarter aspect on Mercury
Rahu aspects Ketu fully
Ketu casts a quarter aspect on the Sun
Ketu casts a half aspect on the Moon
Ketu casts a three-quarter aspect on Mars
Ketu casts a quarter aspect on Mercury
Ketu casts a three-quarter aspect on Venus
Ketu aspects Saturn fully
Ketu aspects Rahu fully
the Sun and Saturn aspect each other
the Sun and Rahu aspect each other
the Sun and Ketu aspect each other
the Moon and Mars aspect each other
the Moon and Venus aspect each other
the Moon and Ketu aspect each other
Mars and Jupiter aspect each other
Mercury and Saturn aspect each other
Mercury and Rahu aspect each other
Mercury and Ketu aspect each other
Jupiter and Venus aspect each other
Saturn and Ketu aspect each other
Rahu and Ketu aspect each other
NABHASA_PASA: liable to imprisonment, skilful in work, deceiving of disposition, talkative, bereft of good qualities, and attended by many servants
NABHASA_PASA: Sun, Moon, Mars, Mercury, Jupiter, Venus and Saturn take part
LUNAR_MOON_IN_PANAPHARA_FROM_THE_SUN: of middling wealth, intelligence and skill
LUNAR_MOON_IN_PANAPHARA_FROM_THE_SUN: Moon takes part
PARASHARA_ASHUBHA: sensuous, a doer of sinful acts, and a swallower of other men's wealth
PARASHARA_ASHUBHA: Mars, Sun and Mercury take part
PARASHARA_PARVATA: wealthy, eloquent, charitable, learned in the shastras, fond of mirth, famous, splendid, and the leader of a city
PARASHARA_PARVATA: Jupiter takes part
SARAVALI_THE_SECOND_FROM_THE_MOON_ASPECTED_BY_A_BENEFIC: earning plenty of wealth
SARAVALI_THE_SECOND_FROM_THE_MOON_ASPECTED_BY_A_BENEFIC: Jupiter and Moon take part
BPHS_TWENTY_SECOND_DECANATE_OF_A_MIXED_PLANET: his body only drying up
BPHS_TWENTY_SECOND_DECANATE_OF_A_MIXED_PLANET: Ascendant takes part
BPHS_NINTH_AND_FIFTH_LORDS_AND_THEIR_COMPANIONS_GIVE_WEALTH: wealth in the periods of these grahas
BPHS_NINTH_AND_FIFTH_LORDS_AND_THEIR_COMPANIONS_GIVE_WEALTH: Moon, Mars and Venus take part
BPHS_MOON_NAVAMSHA_LORD_WITH_A_MARAKA_OR_IN_A_MARAKA_HOUSE: penniless
BPHS_MOON_NAVAMSHA_LORD_WITH_A_MARAKA_OR_IN_A_MARAKA_HOUSE: Mars takes part
BPHS_EIGHTH_OR_TWELFTH_ASPECTED_BY_KARAKAMSHA_LORD_AND_LAGNA_LORD: bereft of wealth
BPHS_EIGHTH_OR_TWELFTH_ASPECTED_BY_KARAKAMSHA_LORD_AND_LAGNA_LORD: Mercury, Ascendant and Jupiter take part
BPHS_TWELFTH_FROM_ATMAKARAKA_OR_LAGNA_ASPECTED_BY_ITS_LORD: a spendthrift
BPHS_TWELFTH_FROM_ATMAKARAKA_OR_LAGNA_ASPECTED_BY_ITS_LORD: Jupiter takes part
BPHS_SUN_IN_SECOND_UNASPECTED_BY_SATURN: riches and fame
BPHS_SUN_IN_SECOND_UNASPECTED_BY_SATURN: Sun takes part
BPHS_GRAHA_WITH_A_DUSTHANA_LORD_UNASPECTED_BY_TRINE_LORDS_HARMS_WEALTH: harm to finances in the periods of these grahas
BPHS_GRAHA_WITH_A_DUSTHANA_LORD_UNASPECTED_BY_TRINE_LORDS_HARMS_WEALTH: Mercury and Rahu take part
BPHS_ATMAKARAKA_IN_A_BENEFICS_SIGN_OR_NAVAMSHA: wealthy
BPHS_ATMAKARAKA_IN_A_BENEFICS_SIGN_OR_NAVAMSHA: Mercury takes part
BPHS_ONE_TO_THREE_GRAHAS_EXALTED: one of royal birth becomes a king, another equal to a king or wealthy
BPHS_ONE_TO_THREE_GRAHAS_EXALTED: Sun takes part
BPHS_AMATYAKARAKA_WITH_ATMAKARAKAS_DISPOSITOR: great intelligence, a king's minister
BPHS_DUAL_LAGNA_WITH_ITS_LORD_WELL_PLACED_LONG: a long life
BPHS_DUAL_LAGNA_WITH_ITS_LORD_WELL_PLACED_LONG: Jupiter takes part
BPHS_MARS_AND_THIRD_LORD_OR_EIGHTH_LORD_AND_SATURN_AFFLICTED_SHORT: a short life
BPHS_MARS_AND_THIRD_LORD_OR_EIGHTH_LORD_AND_SATURN_AFFLICTED_SHORT: Mars and Rahu take part
BPHS_LAGNA_LORD_FRIEND_OF_THE_SUN_LONG: a long life
BPHS_LAGNA_LORD_FRIEND_OF_THE_SUN_LONG: Jupiter and Sun take part
BPHS_FIXED_THIRD_PLACE_OF_DEATH: death in one's own house
BPHS_BENEFIC_ON_EIGHTH_AND_NINTH_LORD_WITH_A_BENEFIC_DEATH_IN_A_SHRINE: death in a shrine
BPHS_BENEFIC_ON_EIGHTH_AND_NINTH_LORD_WITH_A_BENEFIC_DEATH_IN_A_SHRINE: Jupiter and Venus take part
BPHS_ASCENT_TO_THE_MANES: goes to the manes after death
BPHS_ASCENT_TO_THE_MANES: Venus takes part
BPHS_ASCENT_TO_EARTH: is reborn on earth
BPHS_ASCENT_TO_EARTH: Mars takes part
ARISHTA_LUMINARY_VENUS_OR_RAHU_IN_TWELFTH: Venus takes part
ARISHTA_LUMINARY_VENUS_OR_RAHU_IN_TWELFTH: cancelled
ARISHTA_MALEFICS_FROM_THE_SUN: Moon and Ketu take part
ARISHTA_MALEFICS_FROM_THE_SUN: cancelled
ARISHTA_MALEFICS_FROM_THE_MOON: Sun, Mars and Mercury take part
ARISHTA_MALEFICS_FROM_THE_MOON: cancelled
ARISHTA_BHANGA_BENEFIC_IN_KENDRA: Jupiter takes part
ARISHTA_BHANGA_MARS_WITH_JUPITER: Jupiter and Mars take part
BJ_MALEFICS_IN_TWELFTH_AND_SECOND: Mars, Sun and Mercury take part
BJ_MALEFICS_IN_FIFTH_AND_NINTH: Ketu and Moon take part
SARAVALI_BHANGA_BENEFIC_IN_SIXTH_SEVENTH_OR_EIGHTH_FROM_MOON: Jupiter takes part
NEECHA_BHANGA_RAJA: Neecha-Bhanga Raja-Yoga — aggregate. Debility cancelled, raja-yoga effect.
NEECHA_BHANGA_RAJA: Moon takes part
NEECHA_BHANGA_DEBIL_LORD_KENDRA: Neecha-Bhanga — lord of debilitation sign in kendra. Defect-source neutralized.
NEECHA_BHANGA_DEBIL_LORD_KENDRA: Moon takes part
NEECHA_BHANGA_EXALT_LORD_KENDRA: Neecha-Bhanga — lord of exaltation sign in kendra. Lift-protection.
NEECHA_BHANGA_EXALT_LORD_KENDRA: Moon takes part
Sun scores 8.846 rupas
Sun reaches the 5.00 rupas its text requires
Jupiter scores 7.887 rupas
Jupiter reaches the 6.50 rupas its text requires
Venus scores 7.067 rupas
Venus reaches the 5.50 rupas its text requires
Mars scores 6.848 rupas
Mars reaches the 5.00 rupas its text requires
Moon scores 6.779 rupas
Moon reaches the 6.00 rupas its text requires
Mercury scores 6.379 rupas
Mercury falls short of the 7.00 rupas its text requires
Saturn scores 6.267 rupas
Saturn reaches the 5.00 rupas its text requires
the 1st house in Pisces
Jupiter rules house 1
the 2nd house in Aries
Mars rules house 2
the 3rd house in Taurus
Venus rules house 3
the 4th house in Gemini
Mercury rules house 4
the 5th house in Cancer
Moon rules house 5
the 6th house in Leo
Sun rules house 6
the 7th house in Virgo
Mercury rules house 7
the 8th house in Libra
Venus rules house 8
the 9th house in Scorpio
Mars rules house 9
the 10th house in Sagittarius
Jupiter rules house 10
the 11th house in Capricorn
Saturn rules house 11
the 12th house in Aquarius
Saturn rules house 12
Sun in house 2 by sign and house 1 by chalit
Mars in house 12 by sign and house 11 by chalit
Saturn in house 11 by sign and house 10 by chalit
```

**ne-Deva-NP**

```text
लग्न मीनमा
सूर्य मेषमा
सूर्य दोस्रो भावमा
चन्द्र वृश्चिकमा
चन्द्र ९औं भावमा
मंगल कुम्भमा
मंगल १२औं भावमा
बुध मेषमा
बुध दोस्रो भावमा
गुरु मिथुनमा
गुरु चौथो भावमा
शुक्र कुम्भमा
शुक्र १२औं भावमा
शनि मकरमा
शनि ११औं भावमा
राहु मकरमा
राहु ११औं भावमा
केतु कर्कटमा
केतु ५औं भावमा
मेषमा सूर्य र बुध
मकरमा शनि र राहु
कुम्भमा मंगल र शुक्र
लग्न २५°०६′ मीनमा
सूर्य ०°०३′ मेषमा
चन्द्र ११°४७′ वृश्चिकमा
मंगल १°०५′ कुम्भमा
बुध १९°२८′ मेषमा
गुरु १०°३७′ मिथुनमा
शुक्र १४°१३′ कुम्भमा
शनि १°१५′ मकरमा
राहु १९°१७′ मकरमा
केतु १९°१७′ कर्कटमा
सूर्य उच्चमा छ
नवांशमा सूर्य मेषमा
सूर्य वर्गोत्तम छ
चन्द्र नीचमा छ
नवांशमा चन्द्र तुलामा
मंगल मित्रमा छ
नवांशमा मंगल तुलामा
बुध मित्रमा छ
नवांशमा बुध कन्यामा
गुरु सममा छ
नवांशमा गुरु मकरमा
शुक्र अधिमित्रमा छ
नवांशमा शुक्र कुम्भमा
शुक्र वर्गोत्तम छ
शनि स्वक्षेत्रमा छ
नवांशमा शनि मकरमा
शनि वर्गोत्तम छ
राहु सममा छ
नवांशमा राहु मिथुनमा
राहु वक्री छ
केतु सममा छ
नवांशमा केतु धनुमा
केतु वक्री छ
सात कारकमध्ये सूर्य दारकारक हो
आठ कारकमध्ये सूर्य पितृकारक हो
सात कारकमध्ये चन्द्र भ्रातृकारक हो
आठ कारकमध्ये चन्द्र भ्रातृकारक हो
सात कारकमध्ये मंगल ज्ञातिकारक हो
आठ कारकमध्ये मंगल दारकारक हो
सात कारकमध्ये बुध आत्मकारक हो
आठ कारकमध्ये बुध आत्मकारक हो
सात कारकमध्ये गुरु मातृकारक हो
आठ कारकमध्ये गुरु पुत्रकारक हो
सात कारकमध्ये शुक्र अमात्यकारक हो
आठ कारकमध्ये शुक्र अमात्यकारक हो
सात कारकमध्ये शनि पुत्रकारक हो
आठ कारकमध्ये शनि ज्ञातिकारक हो
आठ कारकमध्ये राहु मातृकारक हो
सूर्य दोस्रोमा कठोर वाणी, कुटुम्ब-कलह, नेत्र-धनमा प्रयत्न।
चन्द्र नवौंमा भाग्यवान्, धार्मिक, तीर्थ-यात्रा-प्रिय, मातृ-धर्म-निष्ठ।
मंगल बाह्रौंमा व्यय-बाहुल्य, गुप्त-शत्रु, शस्त्र-अग्नि-व्यय, मांगलिक-योग।
बुध दोस्रोमा वाक्-धन-योग। मधुर वचन, व्यापार-कौशल, कुटुम्ब-पोषण।
गुरु चौथोमा मातृ-गृह-वाहन-योग। हृदय-सुख, धार्मिक गृहस्थ।
शुक्र बाह्रौंमा शय्या-सुख-योग, भोग-व्यय, मोक्ष-कला-प्रवृत्ति।
शनि एघारौंमा लाभ-स्थैर्य-योग। दीर्घकालिक धन-सञ्चय, सेवा-वर्ग-कीर्ति।
राहु एघारौंमा अकल्पित लाभ-योग। विदेश-स्रोतले धन, अकल्पित इष्ट-सिद्धि।
केतु पाँचौंमा मन्त्र-सिद्धि, गूढ-प्रज्ञा, सन्तति-चिन्ता, पूर्व-जन्म-संस्कार।
मीन लग्न: भावुक, दयालु, आध्यात्मिक, कल्पनाशील। बृहस्पतिको प्रभावले श्रद्धावान्।
सूर्यले चन्द्रलाई त्रिपाद दृष्टि दिन्छ
सूर्यले गुरुलाई पाद दृष्टि दिन्छ
सूर्यले शनिलाई पाद दृष्टि दिन्छ
सूर्यले राहुलाई पाद दृष्टि दिन्छ
सूर्यले केतुलाई त्रिपाद दृष्टि दिन्छ
चन्द्रले मंगललाई त्रिपाद दृष्टि दिन्छ
चन्द्रले गुरुलाई त्रिपाद दृष्टि दिन्छ
चन्द्रले शुक्रलाई त्रिपाद दृष्टि दिन्छ
चन्द्रले शनिलाई पाद दृष्टि दिन्छ
चन्द्रले राहुलाई पाद दृष्टि दिन्छ
चन्द्रले केतुलाई अर्ध दृष्टि दिन्छ
मंगलले सूर्यलाई पाद दृष्टि दिन्छ
मंगलले चन्द्रलाई पाद दृष्टि दिन्छ
मंगलले बुधलाई पाद दृष्टि दिन्छ
मंगलले गुरुलाई अर्ध दृष्टि दिन्छ
बुधले चन्द्रलाई त्रिपाद दृष्टि दिन्छ
बुधले गुरुलाई पाद दृष्टि दिन्छ
बुधले शनिलाई पाद दृष्टि दिन्छ
बुधले राहुलाई पाद दृष्टि दिन्छ
बुधले केतुलाई त्रिपाद दृष्टि दिन्छ
गुरुले मंगललाई पूर्ण दृष्टि दिन्छ
गुरुले शुक्रलाई पूर्ण दृष्टि दिन्छ
गुरुले शनिलाई त्रिपाद दृष्टि दिन्छ
गुरुले राहुलाई त्रिपाद दृष्टि दिन्छ
शुक्रले सूर्यलाई पाद दृष्टि दिन्छ
शुक्रले चन्द्रलाई पाद दृष्टि दिन्छ
शुक्रले बुधलाई पाद दृष्टि दिन्छ
शुक्रले गुरुलाई अर्ध दृष्टि दिन्छ
शनिले सूर्यलाई त्रिपाद दृष्टि दिन्छ
शनिले बुधलाई त्रिपाद दृष्टि दिन्छ
शनिले केतुलाई पूर्ण दृष्टि दिन्छ
राहुले सूर्यलाई त्रिपाद दृष्टि दिन्छ
राहुले बुधलाई त्रिपाद दृष्टि दिन्छ
राहुले केतुलाई पूर्ण दृष्टि दिन्छ
केतुले सूर्यलाई पाद दृष्टि दिन्छ
केतुले चन्द्रलाई अर्ध दृष्टि दिन्छ
केतुले मंगललाई त्रिपाद दृष्टि दिन्छ
केतुले बुधलाई पाद दृष्टि दिन्छ
केतुले शुक्रलाई त्रिपाद दृष्टि दिन्छ
केतुले शनिलाई पूर्ण दृष्टि दिन्छ
केतुले राहुलाई पूर्ण दृष्टि दिन्छ
सूर्य र शनिबीच परस्पर दृष्टि छ
सूर्य र राहुबीच परस्पर दृष्टि छ
सूर्य र केतुबीच परस्पर दृष्टि छ
चन्द्र र मंगलबीच परस्पर दृष्टि छ
चन्द्र र शुक्रबीच परस्पर दृष्टि छ
चन्द्र र केतुबीच परस्पर दृष्टि छ
मंगल र गुरुबीच परस्पर दृष्टि छ
बुध र शनिबीच परस्पर दृष्टि छ
बुध र राहुबीच परस्पर दृष्टि छ
बुध र केतुबीच परस्पर दृष्टि छ
गुरु र शुक्रबीच परस्पर दृष्टि छ
शनि र केतुबीच परस्पर दृष्टि छ
राहु र केतुबीच परस्पर दृष्टि छ
NABHASA_PASA: liable to imprisonment, skilful in work, deceiving of disposition, talkative, bereft of good qualities, and attended by many servants
NABHASA_PASA: सूर्य, चन्द्र, मंगल, बुध, गुरु, शुक्र र शनि संलग्न छन्
LUNAR_MOON_IN_PANAPHARA_FROM_THE_SUN: of middling wealth, intelligence and skill
LUNAR_MOON_IN_PANAPHARA_FROM_THE_SUN: चन्द्र संलग्न छ
PARASHARA_ASHUBHA: sensuous, a doer of sinful acts, and a swallower of other men's wealth
PARASHARA_ASHUBHA: मंगल, सूर्य र बुध संलग्न छन्
PARASHARA_PARVATA: wealthy, eloquent, charitable, learned in the shastras, fond of mirth, famous, splendid, and the leader of a city
PARASHARA_PARVATA: गुरु संलग्न छ
SARAVALI_THE_SECOND_FROM_THE_MOON_ASPECTED_BY_A_BENEFIC: earning plenty of wealth
SARAVALI_THE_SECOND_FROM_THE_MOON_ASPECTED_BY_A_BENEFIC: गुरु र चन्द्र संलग्न छन्
BPHS_TWENTY_SECOND_DECANATE_OF_A_MIXED_PLANET: his body only drying up
BPHS_TWENTY_SECOND_DECANATE_OF_A_MIXED_PLANET: लग्न संलग्न छ
BPHS_NINTH_AND_FIFTH_LORDS_AND_THEIR_COMPANIONS_GIVE_WEALTH: wealth in the periods of these grahas
BPHS_NINTH_AND_FIFTH_LORDS_AND_THEIR_COMPANIONS_GIVE_WEALTH: चन्द्र, मंगल र शुक्र संलग्न छन्
BPHS_MOON_NAVAMSHA_LORD_WITH_A_MARAKA_OR_IN_A_MARAKA_HOUSE: penniless
BPHS_MOON_NAVAMSHA_LORD_WITH_A_MARAKA_OR_IN_A_MARAKA_HOUSE: मंगल संलग्न छ
BPHS_EIGHTH_OR_TWELFTH_ASPECTED_BY_KARAKAMSHA_LORD_AND_LAGNA_LORD: bereft of wealth
BPHS_EIGHTH_OR_TWELFTH_ASPECTED_BY_KARAKAMSHA_LORD_AND_LAGNA_LORD: बुध, लग्न र गुरु संलग्न छन्
BPHS_TWELFTH_FROM_ATMAKARAKA_OR_LAGNA_ASPECTED_BY_ITS_LORD: a spendthrift
BPHS_TWELFTH_FROM_ATMAKARAKA_OR_LAGNA_ASPECTED_BY_ITS_LORD: गुरु संलग्न छ
BPHS_SUN_IN_SECOND_UNASPECTED_BY_SATURN: riches and fame
BPHS_SUN_IN_SECOND_UNASPECTED_BY_SATURN: सूर्य संलग्न छ
BPHS_GRAHA_WITH_A_DUSTHANA_LORD_UNASPECTED_BY_TRINE_LORDS_HARMS_WEALTH: harm to finances in the periods of these grahas
BPHS_GRAHA_WITH_A_DUSTHANA_LORD_UNASPECTED_BY_TRINE_LORDS_HARMS_WEALTH: बुध र राहु संलग्न छन्
BPHS_ATMAKARAKA_IN_A_BENEFICS_SIGN_OR_NAVAMSHA: wealthy
BPHS_ATMAKARAKA_IN_A_BENEFICS_SIGN_OR_NAVAMSHA: बुध संलग्न छ
BPHS_ONE_TO_THREE_GRAHAS_EXALTED: one of royal birth becomes a king, another equal to a king or wealthy
BPHS_ONE_TO_THREE_GRAHAS_EXALTED: सूर्य संलग्न छ
BPHS_AMATYAKARAKA_WITH_ATMAKARAKAS_DISPOSITOR: great intelligence, a king's minister
BPHS_DUAL_LAGNA_WITH_ITS_LORD_WELL_PLACED_LONG: पूर्णायु
BPHS_DUAL_LAGNA_WITH_ITS_LORD_WELL_PLACED_LONG: गुरु संलग्न छ
BPHS_MARS_AND_THIRD_LORD_OR_EIGHTH_LORD_AND_SATURN_AFFLICTED_SHORT: अल्पायु
BPHS_MARS_AND_THIRD_LORD_OR_EIGHTH_LORD_AND_SATURN_AFFLICTED_SHORT: मंगल र राहु संलग्न छन्
BPHS_LAGNA_LORD_FRIEND_OF_THE_SUN_LONG: पूर्णायु
BPHS_LAGNA_LORD_FRIEND_OF_THE_SUN_LONG: गुरु र सूर्य संलग्न छन्
BPHS_FIXED_THIRD_PLACE_OF_DEATH: death in one's own house
BPHS_BENEFIC_ON_EIGHTH_AND_NINTH_LORD_WITH_A_BENEFIC_DEATH_IN_A_SHRINE: death in a shrine
BPHS_BENEFIC_ON_EIGHTH_AND_NINTH_LORD_WITH_A_BENEFIC_DEATH_IN_A_SHRINE: गुरु र शुक्र संलग्न छन्
BPHS_ASCENT_TO_THE_MANES: goes to the manes after death
BPHS_ASCENT_TO_THE_MANES: शुक्र संलग्न छ
BPHS_ASCENT_TO_EARTH: is reborn on earth
BPHS_ASCENT_TO_EARTH: मंगल संलग्न छ
ARISHTA_LUMINARY_VENUS_OR_RAHU_IN_TWELFTH: शुक्र संलग्न छ
ARISHTA_LUMINARY_VENUS_OR_RAHU_IN_TWELFTH: रद्द
ARISHTA_MALEFICS_FROM_THE_SUN: चन्द्र र केतु संलग्न छन्
ARISHTA_MALEFICS_FROM_THE_SUN: रद्द
ARISHTA_MALEFICS_FROM_THE_MOON: सूर्य, मंगल र बुध संलग्न छन्
ARISHTA_MALEFICS_FROM_THE_MOON: रद्द
ARISHTA_BHANGA_BENEFIC_IN_KENDRA: गुरु संलग्न छ
ARISHTA_BHANGA_MARS_WITH_JUPITER: गुरु र मंगल संलग्न छन्
BJ_MALEFICS_IN_TWELFTH_AND_SECOND: मंगल, सूर्य र बुध संलग्न छन्
BJ_MALEFICS_IN_FIFTH_AND_NINTH: केतु र चन्द्र संलग्न छन्
SARAVALI_BHANGA_BENEFIC_IN_SIXTH_SEVENTH_OR_EIGHTH_FROM_MOON: गुरु संलग्न छ
NEECHA_BHANGA_RAJA: नीचभङ्ग राजयोग — समग्र — नीच-दोष निवारण राजयोग।
NEECHA_BHANGA_RAJA: चन्द्र संलग्न छ
NEECHA_BHANGA_DEBIL_LORD_KENDRA: नीचभङ्ग — नीच-राशीश केन्द्रमा — दोष-कारक नियन्त्रण।
NEECHA_BHANGA_DEBIL_LORD_KENDRA: चन्द्र संलग्न छ
NEECHA_BHANGA_EXALT_LORD_KENDRA: नीचभङ्ग — उच्च-राशीश केन्द्रमा — उत्थान संरक्षण।
NEECHA_BHANGA_EXALT_LORD_KENDRA: चन्द्र संलग्न छ
सूर्यले ८.८४६ रूपा पाउँछ
सूर्यले आवश्यक ५.०० रूपा पुग्छ
गुरुले ७.८८७ रूपा पाउँछ
गुरुले आवश्यक ६.५० रूपा पुग्छ
शुक्रले ७.०६७ रूपा पाउँछ
शुक्रले आवश्यक ५.५० रूपा पुग्छ
मंगलले ६.८४८ रूपा पाउँछ
मंगलले आवश्यक ५.०० रूपा पुग्छ
चन्द्रले ६.७७९ रूपा पाउँछ
चन्द्रले आवश्यक ६.०० रूपा पुग्छ
बुधले ६.३७९ रूपा पाउँछ
बुधले आवश्यक ७.०० रूपा पुग्दैन
शनिले ६.२६७ रूपा पाउँछ
शनिले आवश्यक ५.०० रूपा पुग्छ
पहिलो भाव मीनमा
गुरु १ भावको स्वामी हो
दोस्रो भाव मेषमा
मंगल २ भावको स्वामी हो
तेस्रो भाव वृषभमा
शुक्र ३ भावको स्वामी हो
चौथो भाव मिथुनमा
बुध ४ भावको स्वामी हो
५औं भाव कर्कटमा
चन्द्र ५ भावको स्वामी हो
६औं भाव सिंहमा
सूर्य ६ भावको स्वामी हो
७औं भाव कन्यामा
बुध ७ भावको स्वामी हो
८औं भाव तुलामा
शुक्र ८ भावको स्वामी हो
९औं भाव वृश्चिकमा
मंगल ९ भावको स्वामी हो
१०औं भाव धनुमा
गुरु १० भावको स्वामी हो
११औं भाव मकरमा
शनि ११ भावको स्वामी हो
१२औं भाव कुम्भमा
शनि १२ भावको स्वामी हो
सूर्य राशिले २ भावमा, चलितले १ भावमा
मंगल राशिले १२ भावमा, चलितले ११ भावमा
शनि राशिले ११ भावमा, चलितले १० भावमा
```

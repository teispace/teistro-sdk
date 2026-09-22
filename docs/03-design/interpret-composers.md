# Composers: a chart's answers as a narrative plan

Status: `built`, designed and built 2026-09-21; measured in
[`interpret-measured.md`](interpret-measured.md). It builds the layer
[`../02-architecture/03-localization-architecture.md`](../02-architecture/03-localization-architecture.md)
fixes under "Composers" — *a composer returns a narrative plan: an ordered
list of (message key, slots); plans are language-neutral, testable and
serialisable, and `intl` renders them* — and it is Phase 6's `interpret`
([`../07-roadmap/00-roadmap.md`](../07-roadmap/00-roadmap.md)), the step after
a rule rendered to prose ([`rule-doc.md`](rule-doc.md)).

## 1. Purpose and scope

The SDK computes a chart and answers it by rule. Nothing turns those answers
into sentences a reader sees, in the reader's language. This page decides the
plan — what a composer returns — the first composers, what decides a message
key, and the gates that keep a plan renderable in every shipped locale.

In scope: the `Plan` and `Item` types, which composers come first and what
they take, how a key is chosen, serialisation, determinism, and the
measurement.

Out of scope: the **interpretation records** — the baseline engine's 108
graha-in-bhava cells and its yoga and dosha readings in four languages —
which reach `i18n/` through `teistro-intl migrate baseline` and are a step of
their own; the **report section catalogue**, which groups plans into a
document; and **crossing the C boundary**, which rides on a chart reading the
way rules do ([`rules-at-the-boundary.md`](rules-at-the-boundary.md)) and is
worth deciding once there is more than one composer to carry.

## 2. What the research found

- **The shape is already decided, and so is the runtime.** `Params` is
  `BTreeMap<String, Value>` — ordered, so a plan is deterministic by
  construction — and `Value` already carries text, integers, numbers, a
  catalogue key, a list, a date, a time and a ghati count, which is every
  slot the messages below need. `sdk.intl.render(key, &params)` answers with
  a `Rendered` that says which locale resolved it, whether it fell back and
  what warned. Nothing new is needed to render a plan.
- **`Value` is not `Serialize`, and a plan must be.** The derive belongs on
  `Value` in `crates/intl`, where the type lives, rather than on a mirror
  type in `interpret`: two value enums would be two lists to keep in step,
  which is the defect this project has hit before.
- **The first composer's whole vocabulary already exists, in both strict
  locales.** `i18n/*/sdk.reason.json` carries `grahaInBhava`, `grahaInRashi`,
  `grahaAt`, `conjunction`, `lordship`, `occupants`, `rashiNature` and
  `strength` in `en-Latn` and in `ne-Deva-NP`, translated by hand. That
  decides which composer is first: `ne-Deva-NP` is a **strict** locale, so a
  new key is a key that must be translated before it can ship, and the
  project forbids machine-translated stubs ("a plausible wrong astrological
  term is worse than a visible fallback"). A first composer that adds no key
  proves the whole path — chart to plan to text in two languages — with no
  translation debt at all.
- **The corpus records no interpretation text.** Checked over every section
  on 2026-09-20: it holds numbers, keys and flags, and not one composed
  sentence. So a composer has **no oracle** here — the corpus can neither
  derive a plan nor falsify one — and the roadmap's byte-for-byte comparison
  against the baseline engine's text waits on `migrate baseline`. What can be
  measured over the corpus is **coverage**: run the composers over every
  recorded chart and count what they say, what they cannot say, and what
  would fall back.
- **And what nothing says at all.** A composer says a *section* or it says
  a placement, and the sections are what a chart request asks for by name.
  Six of the eleven a document can carry have **no composer** —
  `POINTS`, `ASHTAKAVARGA`, `VIMSHOPAKA`, `BHAVA_BALA`, `VAISESHIKAMSA`
  and `DASHA_PHALA` — and the measured page enumerates them with the
  reason for each, read from the source that declares the sections so a
  twelfth cannot be forgotten. That table is this design's remaining
  queue, and it replaces the sentence the page used to carry: *"the
  silences that remain are all of that kind"*, which was true of the
  messages it was counting and not of the sections nobody had counted.
  `DASHA_PHALA` was the cheapest of them and is built: `dasha_phala`, the
  eleventh composer. `STATE`, which the table had read as answered, is
  what the twelfth was for. **Three** of the six were waiting on a **name**, not
  two, and that is the
  correction worth keeping: *being a catalogue member is not being named.*
  `vaiseshikamsa` and `nature` are both catalogued and neither is named by
  any strict locale — both sit on `intl.rs`'s list of members with no
  vetted source, exactly where the special lagnas sit — so a composer
  saying them would print nothing a locale carries. The measured page
  checks each kind a row cites against that list, so a reason cannot
  outlive its blocker. The others wait on a decision about which number
  deserves a sentence, or on a knob, and two share that knob rather than
  having one each: `VIMSHOPAKA` and `VAISESHIKAMSA` both name or score a
  graha under four schemes at once, so the queue is grouped by the blocker
  and not by the row. The row for `VAISESHIKAMSA` called itself the
  cheapest of the six until its sources were read, which is the pattern
  this project keeps earning — a design page's own account of an unbuilt
  step is a hypothesis, and this one was falsified twice before a line of
  the composer was written. `DASHA_PHALA` was the cheapest — one fact a
  graha, no scheme to choose — and it cost one workaround: its unnamed
  `nature` is said the way `sdk.reading.lifeClass` says a class of life,
  matched on its key with the words written in each locale. Five are left.
- **The façade's reading types are the façade's.** `RulesReading` and
  `Present` live in `crates/sdk`, so a composer taking them would put
  `interpret` above the façade and invert the crate graph. Composers take the
  kernel's own types — a placement, a `&Rule` with its `RuleResult`, a
  `HouseReading` — and the façade adapts what it holds into them.

## 3. The plan

```rust
/// One thing to say: a message and its slots.
pub struct Item {
    pub key: String,
    pub params: Params,
}

/// What a composer says, in the order it says it.
pub struct Plan {
    pub items: Vec<Item>,
}
```

- **Flat and ordered.** A composer is about one subject and says its piece in
  order; a report that wants several concatenates their plans. Grouping is
  the report catalogue's job, not the plan's.
- **No text.** An item carries keys and values, never a rendered word, so one
  plan renders in every locale and a snapshot of it is a test of the composer
  rather than of the translator.
- **Serialisable**, both ways, so a plan crosses a boundary and a golden file
  holds it.
- **Deterministic**: the slots are a `BTreeMap` and the items are a `Vec` the
  composer fills in a fixed order, so the same chart gives the same bytes.

## 4. The first composers

**`placements`** — what the chart is, from the kernel's `RuleChart`: for each
of the nine grahas, where it stands (`grahaInRashi`, `grahaInBhava`), and
for each sign holding two or more, who shares it (`occupants`).

**The lagna took a message of its own, and it reads a *point*.** It is
placed in the chart like a graha, but it is `point.LAGNA` in the catalogue
and those three messages read a `Graha` — so for six composers it stood in
every chart and was in no item, which the measured page counted at 93, one
a chart. `sdk.reason.pointInRashi` and `sdk.reason.pointAt` read a point
instead. Two things made that cheap rather than a translation project:
both strict locales already name eight members of the `point` kind — the
ascendant, the five upagrahas, Gulika and Mandi — so the vocabulary was
bought before the frame was written; and the message is general, so a
composer that one day places Gulika says it through the same key.

It is said **first**, because it is what the rest of the chart is read
against, and by its **sign alone**: the lagna's bhava is the first by
definition, and an item saying so would say nothing.

**`readings`** — what the rules answered: for each rule a chart held, what
its verse says, when it acts, who took part, whether a cancellation moved
it and how grave it is. **Built** over a namespace of its own,
`sdk.reading`, in both strict locales.

**When it acts is a third claim, and neither of the other two.** A corpus
of state readings carries a `timing` for a rule — when the dosha bites,
not what it means — and `sdk.reading.timing` says it the way
`sdk.reading.says` says the reading: as a form on the same `rule.<KEY>`
record, rendered by the locale from its own words. The composer asks for
the two **independently**, because 34 of the shipped rules carry both and
18 carry a timing and no reading at all. Those 18 are the computed
doshas, whose verses state no effect: they said nothing in words at all
before this. The message has **no prose of its own in any locale** — it
is the entity render and nothing else — so unlike the six below it adds
nothing to the native review.

Its other messages are deliberately **mechanical** — a span, a class of
life, a list of grahas with a verb that agrees with it, a status, a
severity —
because those are the parts a locale can say for itself. **The verse's own
statement is not translated**: it crosses as a `text` slot in the words the
rule cites, and the message prints them as they are, so a Nepali reading says
everything but the verse's sentence in Nepali. A machine translation there
would be worse than the visible seam, and the seam closes when a locale
carries a reading of that rule written by someone who reads the text.

**`strength`** — what the chart weighs: each graha's Shadbala in rupas,
the strongest first. **Built**, and it adds no message either:
`sdk.reason.strength.score` was already carried by both strict locales and
used by nothing, so the third composer ships with no translation debt for
the same reason the first did.

It is the first composer over a **section** rather than over the chart's
placements or a rule's answer — it reads `Document.shadbala`, which a
request asks for by name and which the boundary now computes for it, as it
computes what a rule set reads. That is the shape the remaining composers
take, so it is worth having one of them built.

Two things it deliberately does not do. It does not say whether a graha
**reaches** the rupas its text requires: the Shadbala carries
`required_rupas` and `strong`, and no locale carries a message for either,
so the plan is silent and the measured page counts the silence. And it does
not emit `sdk.reason.strength.rank`, which renders an ordinal alone — `1st`,
`१लो`. That is a **fragment a consumer formats with, not a sentence a plan
says**, and the distinction is worth stating: a message in the pack is not
automatically a plan item, and `KEYS` lists what the composers emit rather
than everything the locale has.

**`houses`** — whose each of the twelve bhavas is: for each house, the lord
of the sign its **middle** falls in (`sdk.reason.lordship`). **Built**, the
second composer over a section — it reads `Document.houses`, which
`ChartRequest::with_houses` asks for — and the third in a row that adds no
message: `sdk.reason.lordship` was already carried by both strict locales,
hand-translated, and read by nothing.

Lordship is the relation the rest of the tradition is written in — a house is
read through its lord, and no other composer says who that is. `placements`
says where a graha *stands*; nothing until now said what it *rules*, and the
two are different facts about the same graha.

It says the lord and nothing else, which is a larger silence than the other
composers carry and is counted rather than hidden. The bhava's own sign has
no message (`grahaInRashi` takes a graha, not a house); its class — kendra,
trikona, dusthana, upachaya — has none; the **chalit shift**, a body that
falls in one house under the placement system and another under the chalit,
has none, though `Houses` records it body by body; and neither the house
system nor a degenerate chart's `Outcome` has one. Each is a sentence a
locale would have to be given before a composer could say it.

One thing the corpus cannot check here, and the checking of it was a guess
the corpus corrected. `Bhava::sign` is the sign the house's **middle** falls
in, because under an unequal division a house may begin in one sign and be
centred in another. This page first claimed the corpus had no unequal
division to try that on; it has twenty systems' cusps for every chart and
**eight charts selected under Placidus**, two of them degenerate. What is
true is narrower and worth stating as such: none of those eight is in the
yogas corpus this composer is measured over, so all 75 charts the pass
reaches are whole-sign, where cusp and middle coincide. The distinction is
the houses service's to hold — deriving a madhya from a recorded cusp would
be this pass re-implementing the rule it is measuring — and the measured
page says how many charts it was measured on rather than implying more.

**`positions`** — where each graha stands **to the degree**:
`sdk.reason.grahaAt`, which renders "Mars at 12°35′ Scorpio" and
"मंगल १२°३५′ वृश्चिकमा". **Built**, and the fourth in a row that adds no
message: `grahaAt` was carried by both strict locales, hand-translated and
tested, and read by nothing. It reads the same `RuleChart` `placements`
reads, so it needs no section a request does not already compute.

**Why it is a composer of its own rather than a line inside `placements`.**
`grahaInRashi` says the sign; `grahaAt` says the sign *and* the degree, so
one subsumes the other and a composer emitting both would repeat itself once
a graha. Splitting them makes the precision a **knob**: a narrative report
asks for `placements` and a position table asks for `positions`, and a
consumer that wants both is asking for the sign twice and can see that it
is. This is the first place a composer's *option* would have done instead —
`PlanRequest` is a record of named members rather than a bit set precisely
so a composer can grow one — and it is deliberately not taken: an option
that changes which key a composer emits changes the plan's shape with a
member's value, and a second composer costs a consumer nothing.

It does **not** emit `sdk.reason.exactLongitude`, which renders an absolute
longitude alone — "222°34′35″". That is a fragment a consumer formats with
and not a sentence a plan says, which is the same line `strength` drew at
`strength.rank`. The rule has now held twice, so it is worth stating as one:
**a message in the pack is a plan item when it says something on its own.**

One honest caveat, and it belongs to `intl` rather than to the composer.
`grahaAt` rounds to arc-minutes for display, so a body at 29.9999° Aries
renders as "0°00′ Taurus" — `crates/intl` tests exactly that case. A plan
carrying both composers therefore says "Sun in Aries" and "Sun at 0°00′
Taurus" of the same body. Neither is wrong: one reads the longitude, the
other reads it rounded. A composer cannot fix it without rounding the sign
too, which would make the plan disagree with the chart.

**`aspects`** — which graha looks at which, and how strongly: a `cast` item
for every drishti the chart holds and a `mutual` item for every pair that
looks at each other. **Built**, and it is the **first composer that spends
translation debt**: `sdk.aspect` is a namespace of two messages written for
it, in English and in Nepali, because no locale carried a word for a drishti
— not "looks at", not a grading, not the relation.

**That is a decision this page records rather than a task it performed.**
Four composers shipped free because the packs held a message nobody read,
and the fifth exhausted them. The drishti was the largest thing the SDK
computes and cannot say, so it is where the debt is first spent. The
precedent is `sdk.reading`'s six: written from the texts' own vocabulary
rather than invented prose, and **flagged for the native review the roadmap
already requires for `ne` and `hi`**. The terms are the tradition's own —
पाद, अर्ध, त्रिपाद and पूर्ण दृष्टि for the quarters a drishti is counted
in, and परस्पर दृष्टि for a mutual one — so a reviewer is checking grammar
and register, not a translator's guess at a technical term.

The grading is the message's, not the composer's. `Strength` is not a
catalogued entity, so `cast` selects on the slot the way `sdk.reading`'s
`lifeClass` and `status` do, with the arms spelled exactly as
`Strength::key()` writes them — `QUARTER`, `HALF`, `THREE_QUARTERS` and `*`
for `FULL`, the house style of putting the last member on the catch-all.
`Strength::None` is never stored by `Aspects`, so it can never reach the
catch-all and be said as "full"; the composer skips it anyway and a test
holds that, because a catch-all that would lie is worth one branch.

**`mutual` is not a restatement.** A pair that looks at each other already
has two `cast` items, but *parasparadrishti* is a named condition the texts
read as one thing, and a consumer would otherwise derive it by scanning. It
is the same argument that puts `occupants` beside `grahaInRashi` in
`placements`.

Its subject is a **slice of relations** rather than the `Aspects` container,
which is what let the pairing stay in one place. `Aspects::mutual()` took
each pair once by the foundation's order of bodies; that rule is a property
of a set of relations and not of the container, so it moved to
`aspect::mutual_pairs` over a slice and `Aspects::mutual()` delegates to it,
so the composer and the container answer from one implementation rather than
two. `plans.rs` holds them against each other — the plan's `mutual` items
counted against `Aspects::mutual()` over the same document — and
`crates/aspect`'s own test still holds the rule over a founded chart.

What it does not say: how near either end stands to a sign edge
(`from_edge`, `to_edge`), which is a statement about how much an ayanamsha
that moved would change the reading rather than about the native; and which
drishti table the settings named, which is a setting. Neither has a message,
and neither is a sentence a reading wants.

One thing the corpus cannot decide. **It records no aspect at all**, so
unlike `strength` and `houses` there are no recorded values to compose from.
The pass computes the relations from the corpus's recorded **signs** with
`aspect::drishti::between`, and the page says so: what is measured is that a
plan of them can be said in both strict locales, which is this pass's
question anyway. Whether the drishtis themselves are right is
`aspect-drishti-measured.md`'s business.

Every item names its rule in a `rule` slot the base messages declare and do
not print, so a consumer can group a plan by rule and a locale that wants the
key in its prose has it. The measured page's snapshot prints it as a prefix,
because a reviewer of a rendering needs to know which rule said what.

**`conditions`** — what a graha **is** where it stands, as against where it
stands: its dignity, its navamsha sign and whether that sign makes it
vargottama, whether it is retrograde, and whether the Sun burns it. **Built**
over `sdk.condition`, five messages in English and Nepali, and it reads the
same `RuleChart` `placements` reads, so it costs a request no section.

**It closes the silence `positions` left, and the measured page had counted
it.** A `Placement` is nine facts; `placements` said the sign and the house
and `positions` the longitude, and the other six had no message in any
locale. `conditions` says four of them and `karakas` the last two, so every
fact a placement carries is now an item.

**The second composer to spend translation debt, and much the cheaper.**
Two of its five messages say a value the **entity** namespace already names
in all five locales — a dignity and a rashi — so the terms that had to be
written are the three conditions that have no value to name: वक्री,
अस्तंगत and वर्गोत्तम. They are the tradition's own, flagged for the same
native review `sdk.aspect` and `sdk.reading` wait on. This is worth stating
as a rule rather than an observation: **where a composer says a catalogued
value, the vocabulary is already bought.** `sdk.entity` carries 34 kinds in
five locales, and a message with an entity slot reaches all of them.

**A dignity crosses as an entity and not as a string**, which is where this
composer differs from `aspects` and deliberately. `cast` selects on a slot
with the arms spelled as `Strength::key()` writes them, because `Strength`
is not catalogued. `Dignity` is, and it is `#[non_exhaustive]` with members
appended, so eleven arms and a catch-all would read a twelfth as the
eleventh — a message that goes wrong by standing still. An entity slot
renders each member's own word and cannot.

**A dignity is said of every graha, `NEUTRAL` included**, where `aspects`
skips `Strength::None`. The two look alike and are not: no aspect is an
absence, and the catch-all would have said "fully" of it, while *sama* is a
dignity the texts name. The three conditions that **are** absences — not
retrograde, not burnt, not vargottama — are said only where they hold, and
a test holds that a quiet chart says none of them.

**Vargottama stands beside the navamsha rather than instead of it**, which
is the argument that put `mutual` beside the two casts it is made of: the
fact is the sign, the name is what the texts read, and a consumer would
otherwise derive the name by comparing two items.

One thing the corpus decided rather than the page. Three quarters of the
retrogressions in these charts are the nodes', which would make "Rahu is
retrograde" a tautology worth skipping — except that the corpus records the
nodes **direct** in two of its six `--true-node` variants, and never in a
mean-node chart. The true node turns; the mean node does not. So the
condition carries information and is said of every graha that holds it, and
the measured page counts it rather than this page assuming it.

**`chalit`** — where the two house readings put a graha in different
bhavas. **Built** over `sdk.reason.chalitShift`, one message, and it is
the tenth composer.

A chart places every graha **twice** — under the placement system, which
is what most of the tradition means by "in the seventh", and under the
chalit, which reads the cusps — and keeps both without recomputing either
(`chart-bhava-chalit.md`). Over the recorded corpus they differ for 135 of
675 placings, and the measured page holds the composer's item count
against the corpus's own `shifted` list: one item a recorded shift, none
for a placing the corpus does not record as moving, so a composer that
said one too many or too few changes that page.

**It says only the grahas that differ**, because agreement is the ordinary
case and an item a graha would bury the disagreement in eight repetitions
of it. A chart whose readings agree throughout composes to nothing here,
which is the answer and not a failure.

**Why it is not a line inside `placements`.** That composer reads a
`RuleChart`, which carries one house a graha, and the façade's
`placements(document)` is held by a test to be exactly
`interpret::placements(&RuleInputs::of(document)?.chart)` — the façade
adapts and does not re-derive. Both readings live on the chart foundation,
so this composer reads that, and takes the narrowest type carrying them as
every composer here does. The message needs no `.match`: `lordship` had
already settled that a house can be said by its number.

**`houses`** — the sign each of the twelve bhavas falls in, and the graha
that rules that sign. Two items a bhava, because they are two facts; the
sign was unsaid for three composers and cost **no new vocabulary**, since a
`Rashi` is catalogued and named in every locale and the ordinal shape was
the one `grahaInBhava` already had translated.

It is the sign the house's **middle** falls in, which is what `Bhava`
carries and what the tradition means: under an unequal division a house can
begin in one sign and be centred in another. The composer repeats the
record rather than choosing between them, and the measured page cannot tell
the two apart, because every division the corpus records is whole-sign —
which is a gap in the corpus, said as one, rather than a branch claimed
tested.

What a bhava knows besides — its quadrant, and whether it is a trine, a
house of difficulty or one that grows better with time — is the shape
[`state-readings.md`](state-readings.md) §8 names: computed, and not a
catalogue member. `Quadrant` is a Rust enum and the rest are predicates, so
a message would have to name each in words no locale here has been given.

**`strength`** — what the chart weighs, and whether that is enough. It says
**two** items of each graha because the Shadbala carries two facts: the
rupas it scores (`score`) and the rupas its text requires beside whether it
reaches them (`meets`). For four composers the second crossed in the
document and was absent from the plan, counted at 341 of 497 grahas.

**It names the requirement and never a verdict.** "Strong" is a word the
tradition spends carefully and no locale here has been given it; what the
Shadbala computes is a number and a comparison, so that is what the message
says — *Mercury falls short of the 7.00 rupas its text requires*, *मंगलले
आवश्यक ५.०० रूपा पुग्छ*. The selector is a word and not a boolean, as every
other selector in the packs is, and the composer names both arms as
constants so the two cannot drift apart.

`sdk.reason.strength.rank` is still not emitted: an ordinal alone is a
fragment a consumer formats with, and the ranking is carried by the items'
order.

**`karakas`** — which chara karaka each graha holds, under the seven-karaka
scheme and the eight-karaka one. **Built** over `sdk.karaka`, two messages,
and it reads the same `RuleChart` too.

**Both schemes, because they disagree.** Over the corpus's 93 charts the two
give a graha the same karaka 326 times and a different one 325, and the
eight rank one graha a chart the seven do not rank at all. A composer
emitting one of them would be choosing for the consumer, silently, in about
half of all cases — the dead end the no-dead-ends mandate forbids. It emits
both, and **the key says which** rather than a slot: `ofSeven` and
`ofEight`, so a consumer filtering by key gets one scheme whole instead of
reading a value to find out what it has.

**Which order the eight are ranked in is the chart's and not the
composer's.** `rule_chart` computes them as BPHS ch. 32 orders them
(`EightKarakas::Parashara`); the recording engine puts the Pitrikaraka last,
and `RuleChart::with_chara_karakas` switches. The composer repeats whatever
the chart carries. That is worth knowing when reading the measured page: it
composes the corpus's **recorded** karakas, as `strength` composes recorded
rupas and `houses` recorded cusp signs, so its numbers are the recording
engine's ranking and the SDK's own answer for the same chart can differ.

**`states`** — the other half of a graha's state: how it stands to its
dispositor under all three friendships, and the four avasthas — the fifth
of its sign, its wakefulness, its brightness where the chart decides one,
and the lajjitadi that hold beside the ones nothing decides. **Built** over
six more of `sdk.condition` and four of `sdk.phala`, the twelfth composer.

**The section table asked for it, which is what the table is for.**
`STATE` had read as answered because `conditions` says every fact a
`Placement` carries — nine of them, all said — while the section's own
`GrahaState` carries a dozen, and the other half reached no reader at all.
That is why the table's third column means *what is left* rather than *why
nobody says it*.

**Almost every word of it was already bought**, which is the opposite of
`dasha_phala` and why it follows rather than leads. `Relationship` and the
four avastha kinds are catalogue members that all five locales name, so the
frame is the whole cost: six messages and not one new term, with nothing
for the native review that it has not already seen. What is *not* said is
not said by name — the Sayanadi and the Cheshta are catalogued and
**unnamed**, the vetted tables stopping at the four — and the row cites
them rather than leaving them to be noticed.

**The undecided lajjitadi are said, and that is the point of them.**
`Lajjitadi` names three lists — holding, ruled out, and the ones nothing in
the chart decides, because the tradition's necessary condition holds and
what narrows it further is not on the page. A plan that printed only what
held would turn *we cannot tell* into *no*, which is the silent default
this project refuses; so the undecided are an item of their own, with a
message that says so.

**`dasha_phala`** — what each graha's placement says of its dasha: when
in the dasha its effects come, whether its place is auspicious, the points
its dignity earns and whether the placement makes the dasha favourable
(BPHS ch. 28 vv. 7 to 10, ch. 47 vv. 3 to 6) — with the reading a loaded
corpus carries of that graha as a dasha lord. **Built** over
`sdk.reason.dasha` and two of `sdk.phala`, the second composer over a
**section**.

**It was the queue's own cheapest item, and it was still words.** The
measured page's table of sections named it so: of the six a composer could
not say, three were short a *name* rather than a sentence, and this one
was short exactly one — `nature`, a catalogue member no strict locale
names. So its place is said the way `sdk.reading.lifeClass` says a class
of life: **matched on the key**, with the words written in each locale,
rather than through an entity slot that would print nothing. Being a
catalogue member is not being named, and this is where that distinction
was first spent rather than merely recorded.

**The tilt is one item and not two.** A placement can make a dasha
favourable and unfavourable at once — vv. 5 and 6 allow it, and both flags
then stand — so the message has a third arm for both rather than the
composer saying it twice. A placement that tilts it neither way says
nothing of the tilt, as a rule nothing cancelled says nothing of its
cancellation.

**And the corpus's own two are asked for separately.**
`dasha-lord-effect` and `dasha-lord-activation` key onto a graha under the
forms `dashaPhala` and `dashaActivation` — 18 readings that had no
composer to attach to, which is why this section was the one the state
corpus was waiting on.

**`phala`** — what a loaded corpus of **state readings** says of this
chart's subjects: a graha in a bhava, the lagna's sign, each limb of the
panchanga, and the six things the birth nakshatra **is**. **Built** over
`sdk.phala` (`03-design/state-readings.md`).

**It is the first composer that says what the SDK did not compute.** Every
other one turns an answer into an item; this one turns a *record* into one,
and is silent unless a pack carrying those records has been loaded. A chart
composes to exactly the plan it did before until a consumer asks for the
words, which is why it is a member of `PlanRequest` and off by default: a
plan that grew by a hundred items the moment a pack was loaded would change
every consumer's page without being asked.

**What the birth nakshatra is costs nothing to say, and that is the
point.** Every nakshatra in the catalogue already carries its gana, nadi,
yoni, varna and element as attributes, and the corpus keys its readings
onto those kinds' own records — `gana.DEVA`, `yoni.ELEPHANT`,
`tatwa.PRITHVI`. So the subject of each of those readings is *settled* by
the nakshatra the panchanga limb above already names, which is the Moon's
and therefore the janma nakshatra every text reads them from. Nothing is
computed, nothing is chosen, and no catalogue kind had to be decided:
`namakarana` is a second form on the nakshatra itself, exactly as
`lagnaPhala` is a second form on a rashi. Fifty-seven readings reached a
reader this way, more than any other step, and the reason they were cheap
is the test for the ones that are left — a category whose subject the SDK
already computes *and already names* can be said without a decision.

**It asks the same trait `readings` asks**, of the same base locale and for
the same reason. `Vocabulary` was spelled for one subject
(`has_reading(rule)`); a second subject made it the general question it
always was — `has_form(key, form)` over a catalogue key — and a reading of a
rule is now the `name` form of a `rule` record, which is what `has_reading`
spells.

**What it cannot say yet is counted rather than hidden**, and the count
is on `state-readings-measured.md` rather than in this sentence, which
carried `211 of 425` through two steps that changed it. That page lists
what is left by the category it came from, renders every reading a
composer *can* say in each strict locale, and holds that none falls back
or warns. Most of the panchanga messages emit nothing over the yoga
corpus, because those charts record no panchanga — the dosha corpus does,
and the reachability claim is what exercises them meanwhile.

**Why it is not four more lines inside `conditions`.** A chara karaka is
Jaimini's reading of a placement, not a Parashari condition of it, and a
report that wants the dignity does not automatically want the Atmakaraka.
Apart, the tradition is a knob — the same argument that separated
`positions` from `placements`, made over a tradition rather than over a
precision.

## 5. What decides a key

A composer that emits a key the locale does not carry produces a visible
fallback, which is a defect and not a feature. Two rules keep that from
happening:

1. **Every key a composer can emit is listed** in the module, `KEYS`, and a
   gate holds the list against the base locale and against every strict
   locale, failing both ways — the pattern the rules kernel's `KINDS` list
   follows.
2. **Where a composer chooses between a general key and a specific one** — a
   reading of "any rule" against a reading written for `MAHAPURUSHA_HAMSA` —
   it asks the **base locale**, not the reader's. The base locale is a fact
   of the build; the reader's locale is not, and a plan that changed shape
   with the reader would not be language-neutral. The trait that asks arrives
   with the composer that chooses (step 4): `placements` has one key for each
   thing it says, and an interface with no implementor is an interface
   designed from a guess.

## 6. The measurement, and the gates

`cargo xtask interpret` writes `interpret-measured.md` and `check-interpret`
holds it. Over every readable chart of the corpus:

- how many items a chart's plan carries, and of which keys;
- what the composer **cannot** say — a body the chart does not place, a rule
  whose verse states no effect — counted rather than hidden;
- that every item renders in **both strict locales** with no fallback and no
  warning, which is the honest end-to-end check available without an oracle:
  `Rendered` reports both.

Beside the pass, in the crate: a golden plan for one chart, and its rendered
text in English and Nepali (the checklist's "composer plans have snapshots
per language"); a test that the key list matches what the composers emit; and
a test that a plan round-trips through JSON.

## 7. Order of work

1. `crates/interpret` with `Plan`, `Item` and the `placements` composer;
   `Value` given serde derives in `crates/intl`; the key list and its gate;
   snapshots in English and Nepali. **Built.**
2. The pass and `interpret-measured.md`, `check-interpret` in `fast-check`.
   **Built**: 93 recorded charts, 1878 items, 3756 renderings with no
   fallback and no warning, and one chart said whole in both languages.
3. The façade: `sdk.chart().plan(…)` returning a plan, and `sdk.intl` already
   renders it; the Rust example.
4. `readings`, with the `sdk.reading` namespace written in English and
   Nepali, and the rule's own cited effect as a slot where no locale carries
   a reading of its own. **Built**: 93 charts against five shipped packs
   compose to 8347 items, 16 694 renderings, no fallback and no warning.
   The Nepali of `sdk.reading` awaits the native review the roadmap's exit
   criterion already requires for `ne` and `hi`; the terms are the texts'
   own (अल्पायु, मध्यायु, पूर्णायु), not invented prose.
5. The plan at the boundary, so a composer added reaches four languages
   rather than one. **Built**: `03-design/plans-at-the-boundary.md`.
6. `strength`, the first composer over a section. **Built**: it adds no
   message, reads `Document.shadbala`, and is measured over the corpus's
   **recorded** Shadbalas rather than the SDK's computed ones — the numbers
   themselves are `check-shadbala`'s business, and what this pass decides is
   whether a plan made of them can be said.
7. `houses`, the second composer over a section and the first to say what a
   graha **rules** rather than where it stands. **Built**: it adds no message
   either, and is measured over the corpus's **recorded** cusp signs
   (`baseline/charts`' `houses.selected.cusp_sign_index`) for the same reason
   `strength` is measured over its recorded rupas — whether those signs are
   right is the houses service's business, and what this pass decides is
   whether a plan made of them can be said.
8. `positions`, the degree `placements` rounds away. **Built**, and the
   last composer the shipped packs could carry for free. The measured page
   settles what is left rather than this one claiming it: it takes the
   namespaces from `KEYS`, lists every message the base locale carries
   under them, says which composer emits it, and prints any that is
   neither emitted nor given a reason as **unaccounted**. The seven that
   are spare are two fragments (`exactLongitude`, `strength.rank`), a fact
   about the zodiac rather than a chart (`rashiNature`), a count of what
   `occupants` already names (`conjunction`), and the packs' three example
   messages. The counts live on that page, which is generated and gated, so
   they cannot go stale here.
9. `aspects`, the drishti, and the **first composer written a message**.
   **Built**: `sdk.aspect` in English and Nepali, the relations derived from
   the corpus's recorded signs because it records no aspect at all, and the
   pairing rule moved onto a slice so the composer and `Aspects` share one
   implementation.
10. `conditions` and `karakas`, the six facts of a placement that
    `placements` and `positions` leave unsaid. **Built**: `sdk.condition`
    and `sdk.karaka`, seven messages, four of which say a value the entity
    namespace already names in all five locales. With them every fact a
    `Placement` carries is an item, and what the composers still cannot say
    belongs to the sections rather than to the placements — a bhava's sign
    and class, the chalit shift, whether a graha reaches its required rupas.

11. `phala`, the ninth, and the first that says what a **corpus** carries
    rather than what the SDK computed. **Built**: `sdk.phala`, each
    message saying a record a loaded pack brings, and `Vocabulary`
    generalised from one subject to any.
12. The **chalit shift**, the last silence the measured page counted.
    **Built**: `chalit`, the tenth composer, over one message and the
    chart's own grahas.
13. The **bhava's sign**, the third silence in as many steps and the one
    that cost nothing: `sdk.reason.bhavaInRashi`, over a kind already
    catalogued and an ordinal shape already translated. **Built**.
14. The **strengths'** second fact. **Built**:
    `sdk.reason.strength.meets`, one message, two sentences a locale — the
    rupas a graha's text requires and whether it reaches them. The
    Shadbala had carried it since `strength` was written; no locale had a
    message for it.
15. The **lagna**, said at last. **Built**: `sdk.reason.pointInRashi` and
    `sdk.reason.pointAt`, two messages that read a point rather than a
    graha, said by `placements` and `positions`. It was the oldest silence
    in this page — counted at 93 items, one a chart, for six composers —
    and it cost two messages because the `point` kind was already named.

## 8. What this design does not settle



- **The interpretation records.** 108 graha-in-bhava cells with condition
  modifiers and conjunction synthesis, and a reading for each yoga and dosha,
  in four languages, are data and not code; they arrive through `migrate
  baseline` and turn `placements` from a description into an interpretation
  without changing its shape.
- **A further composer needs a new translated key** — answered twice now,
  and the second answer is cheaper than the first. Five composers shipped
  for free because the packs carried a message nobody read; `positions`
  emptied that pool. `aspects` spent the debt on the **drishti**, a computed
  section with no word in any locale, writing `sdk.aspect` from the
  tradition's own terms — पाद, अर्ध, त्रिपाद and पूर्ण दृष्टि, परस्पर
  दृष्टि — and flagging it for the native review the roadmap already
  requires for `ne` and `hi`, exactly as `sdk.reading`'s six were.
  `conditions` and `karakas` then spent much less for more, because **where
  a composer says a catalogued value the vocabulary is already bought**:
  four of their seven messages carry a dignity, a rashi or a chara karaka
  as an entity slot, and `sdk.entity` names those in all five locales. Only
  a condition with no value to name — वक्री, अस्तंगत, वर्गोत्तम — had to
  be written. What is left is the same shape and belongs to the sections
  rather than to the placements: a bhava's sign and class, the chalit
  shift, whether a graha reaches its required rupas, and a nakshatra with
  its pada. Each is a message to write and a review to get, not a design
  question — and `interpret-measured.md` counts them so the size of the gap
  is a measurement rather than a memory.

- **A consumer's own composer** — **settled for v1.0, and not by building a
  registry.** This page said a registry cost nothing to add once a second
  composer existed to prove the interface. Eight now exist, and what they
  proved is that the registry was the wrong thing to reach for.

  The promise in the extensibility table is *a narrative plan function
  (Rust)*, and that is **already kept**: `Plan`, `Item`, `TypedMessage`,
  `params` and the generated `messages` tree are all published through
  `teistro`, `Plan::items` is public, and a composer is a function returning
  a `Plan`. A consumer writes one and concatenates it, exactly as the
  shipped composers are concatenated. `plans.rs` holds an acceptance test
  that does it with the published surface alone, so the table's row is a
  gated fact rather than a claim. The table's validation column — *keys
  exist* — is the consumer's to check and needs no machinery either, but it
  takes the right field, and writing this page got it wrong. The claim was
  that `Rendered::is_fallback` reported an unknown key; the acceptance test
  failed and said otherwise. `is_fallback` means a **fallback locale**
  answered. A key no locale carries at all is not a fallback, because
  nothing fell back: `resolved_from` is `None`, and `Intl::has` answers
  before rendering at all. The measurement pass has always checked both
  (`is_fallback || resolved_from.is_none()`); only the prose was wrong, and
  a test written against the promise is what caught it.

  So a registry buys nothing in Rust. It would buy one thing only: letting a
  consumer's composer be **named at the boundary**, so it runs inside the
  SDK's crossing for Node, Dart and Python. That is the v1.x *declarative
  plan*, and it is a boundary decision rather than a Rust one, because
  `interpret_json` refuses an unknown member by design — a typo composing
  nothing silently is the dead end the no-dead-ends mandate forbids. A
  registry makes unknown names something to look up instead of something to
  refuse, and that trade wants deciding on its own rather than as a
  side-effect of adding a map.

  And if one is built, **the subject is not `&Document`**, which is what
  several composers showed and one could not have. All but one take a
  document at the façade; `readings` takes a `RulesReading`, because a
  rule's answers are not in the document and never will be. A registry keyed
  on `&Document` would therefore exclude the composer a consumer most wants
  to extend — the one that says what its own rule pack found. The subject is
  a record of what the crossing answered, the document and the rules'
  reading together.
- **The report.** Sections, their order, and which composers fill them.

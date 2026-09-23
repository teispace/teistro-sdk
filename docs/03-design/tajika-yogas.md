# The sixteen Tajika yogas

Status: built but for what `YearYoga::awaiting` names —
2026-09-23; that is designed and *named*, which is not the same as
absent. The count is deliberately not written here: it is generated in
`muntha-measured.md` §10 from the type, where it cannot go stale. All
five steps of the order of work landed the same day, and the sixteen
**cross** every binding for the matters a caller names (see *Crossing the
boundary*); what remains waits on a source (C117), not on building.
Written after reading the source's Table X-3 off the page and before
writing any of it, as every
module since Phase 4 has been. The definitions are in
[`01-research/feature-universe/07-tajika-varshaphala.md`](../01-research/feature-universe/07-tajika-varshaphala.md),
"The sixteen yogas, read".

It is built on the aspects ([`tajika-aspects.md`](tajika-aspects.md)) and
the five-fold strength ([`panchavargiya.md`](panchavargiya.md)), both of
which exist.

## They are answers to a question, not facts about a chart

This is the thing the reading settled, and it changes the shape of the
module. A Parashari yoga is a fact: Gaja Kesari either holds in a chart or
does not. **Fourteen of these sixteen are not.** They are judgements about
a *pair*:

- the **lagnesha**, the lord of the annual lagna — fixed by the chart;
- the **karyesha**, the significator of *the matter asked about* — the
  lord of the house that matter belongs to, which the chart cannot know.

So "what yogas does this year have?" is not a well-formed question. "Is
the marriage promised this year?" is: it fixes the karyesha as the seventh
lord, and the sixteen then say whether the promise is fulfilled, delayed,
helped by someone, or negated. Only **Ikabala** and **Induvara** are chart
facts, and they are the two that say nothing about a matter.

The API therefore takes a matter and answers for it:

```rust
sdk.chart().tajika_yogas(&annual, House::try_new(7)?) -> YearYogas
```

with the two chart-level yogas answered whatever is asked. A caller who
wants "every yoga for every house" asks twelve times, which is honest
about what it is doing.

**The design said `Bhava::Seventh` and the building said `House`**, which
is the page being corrected by the code as this repository expects. There
is no `Bhava` in the catalogue and there should not be: a house is named
by its number, and `House` — a newtype over 1 to 12 with its refusal in
one place — already existed in `teistro_rules::language`, where three
other crates were already reaching for it. It moved to `teistro_core`
rather than being copied, because a second twelve-valued primitive is a
second place to get the counting wrong. It gained `sign_from`, the
inverse of its `between`, held by a test over all 144 pairs.

### What the answer carries

`YearYogas` carries the **question** as well as the answer — the house,
its sign, the lagnesha and the karyesha — because a list of yogas whose
pair a reader cannot see is not checkable. And it carries
`unanswered`: the twelve this build cannot yet judge, listed at **every
call**. `holds` returns `Option<bool>`, so *did not hold* and *cannot be
told* are different values and not the same silence.

### The first house is never a pair

The building found this and the design had not: the first house's lord
**is** the lagnesha, by definition, so a question about the native's own
self can never be one of the fourteen pair judgements. One further house
is the same under a lagna ruled by one of the five that rule two signs,
and none is under Cancer or Leo. Over the corpus that is 3 899 of 25 908
matters — 15.0% — reported as `same_lord` rather than returned as an
empty list that would read as *nothing holds*. Whether the tradition
reads one planet for both lords as the matter being promised outright is
a question no text in reach answers, so it is reported and not decided.

## What each of the sixteen needs

Everything below is already built unless marked.

| need | where it is |
|---|---|
| the Tajika aspect and the deeptamsha orbs | `drishti` |
| Ithasala in all three kinds, and Ishrafa | `drishti`, since 2026-09-23; `Yoga::is_ithasala` is the question the fourteen actually ask |
| "strong" and "weak" | the Vishwa bala, `bala` — the source's own floor of **five units** is stated for the year lord and **not** for the yogas (crux) |
| own Hudda, Drekkana and Navamsha | `bala`'s lords, which "unqualified" needs |
| exalted, debilitated, retrograde, combust | the founded chart's own graha rows |
| kendra, panaphara, apoklima, trika | whole-sign houses from the annual lagna |
| benefic and malefic | Tajika's own reckoning: **Mars and Saturn** are the malefics Manau names |

Nothing here is missing. The module is a composition, which is why it was
worth building the aspects and the strength first and separately.

## Two corrections, made — and a third question they opened

**Done, 2026-09-23.** The reading found the book disagreeing with itself
twice; `crates/tajika` shipped the prose's reading of both, and now ships
the table's.

1. **Ishrafa's degree** (crux C110). The table requires the faster planet
   **one degree or more** ahead; the prose says only "ahead", within the
   orb. The table's degree is in. The prose's orb stayed, because a
   summary table omitting the orb the whole system rests on is an
   omission and not a rule.
2. **The sign's-end Ithasala's name** (crux C111). `Yoga::Bhavishyat`
   ships under the table's name, since the table is what enumerates the
   kinds. The prose's *Rashyanta* survives in `RASHYANTA_DEG`, which
   names the **position** at a sign's end rather than the yoga.

And the kind the table adds that was not built at all: **Poorna**, the
Ithasala within a single degree, which the source marks as immediate
fulfilment. It is in, and it opened the question the corrections could
not close.

### The band between them (crux C112)

Table X-3 begins Ishrafa a degree past and gives Poorna as a narrowing of
*Vartamana*, which it states only for a faster planet **behind**. Between
them sits a band the table bounds twice and places once: the faster less
than a degree **past**. Three readings are each defensible, and they are
not near each other — **Poorna**, the most fulfilled thing a pair can be;
**Ishrafa**, generally unfavourable; or nothing at all.

The sweep in [`muntha-measured.md`](muntha-measured.md) §9 says how much
turns on it: **934 of the 29 166 aspecting pairs** over the recorded
births' first forty years, a twelfth of every Ishrafa. That is too large
to settle by taste, so all three ship as `SubDegree` in a `DrishtiRules`,
and `Between::disputed` marks the pairs so a reader can show a judgement
as contested.

The **default is Poorna**, on an argument from the table rather than from
astrology: it is the only reading under which the degree the table prints
on the Ishrafa row does any work at all. The measured sizes agree — 940
pairs in the Poorna the table states outright against 934 in the band,
sitting symmetrically either side of an exact aspect. A reading on which
one side of exactness is immediate fulfilment and the other side is
nothing would have to explain that asymmetry, and the book does not.

## Three judgements upon an Ithasala

Manau, Kamboola and Khallasara are not alternatives to the Ithasala —
they are things said **about** one, so all three are asked only where a
pair already makes it, and all three are reported **beside** it rather
than instead of it. Manau says the Ithasala was destroyed; the Ithasala
is still what was destroyed, and an answer showing only the verdict
could not say what happened.

Two readings the table's wording forces, both recorded rather than
assumed:

- **"Conjunct or inimically aspecting" is one condition, not two.** A
  planet sharing a sign stands at house 1, and house 1 is
  `Drishti::Inimical` in Tajika. The same collapse applies to
  Khallasara's "neither conjunct with nor aspecting either".
- **The third planet must be a third planet.** Mars is a malefic and is
  also the lagnesha under Scorpio or Aries; a malefic that is itself one
  of the pair does not destroy its own Ithasala. A test asserts it,
  because the rule as printed does not say so and the arithmetic would
  happily say yes.

### Tajika's malefics are two (C114)

`MALEFICS` is `[Mars, Saturn]`, named in this module and deliberately
**not** read from the catalogue's `Nature::Malefic`, which follows the
Parashari reckoning and carries the Sun, Rahu and Ketu as well. Reading
the catalogue would have silently added the Sun to every Manau. A test
asserts the Sun is absent so the two lists cannot quietly converge.

### "Unqualified", and what measuring it found (C115)

The source defines the word outright, and every clause is computable:
neither exalted nor debilitated, nor aspected or associated, nor in its
own Hudda, Drekkana or Navamsha. `Qualification` carries **all six
clauses separately** rather than collapsing them, so a reader asking why
a Khallasara did not hold gets the clause — and so a pass can count
which one does the disqualifying.

Counting it was worth doing. Over 2 159 annual charts the Moon was
unqualified **once**, and Khallasara held 2 times in 25 908 matters.
The reason is structural: Tajika counts **eight of the twelve** sign
relations as an aspect, so a planet nothing aspects needs all six others
inside the four neutral houses at once. The source's own worked chart
cannot manage it at any degree of the Moon's circle.

A yoga a text bothers to name and define is unlikely to be that rare, so
the literal reading is probably too wide — but *probably* is not a
citation, so it ships literal and C115 records the doubt with the number
attached.

And one clause is **vacuous**: the Hudda is the Egyptian terms, which
divide every sign among Mars, Mercury, Jupiter, Venus and Saturn and
give the luminaries nothing. For the one planet this definition is ever
applied to, `own_hudda` can never be true. It stays in the code because
the source states it and a reader comparing the two should find all six;
the measured zero is explained on the page rather than left looking like
missing data.

## What the six strength yogas will need, and why they are not here

Rudda, Duhphali-kuttha, Dutthottha-Davira, Tambira, Kuttha and Durapha
were surveyed before this unit closed rather than after, because two of
their needs are **structural** and one is a crux — and finding that out
by starting to write them would have been the expensive way.

| they need | where it is |
|---|---|
| exalted, debilitated, own sign | the catalogue, already read by `Qualification` |
| the trika houses; kendras and panapharas | `House::is_trika`, `is_kendra`, `is_panaphara` — put there when `House` moved to `core` |
| malefic influence | `MALEFICS` and the sign aspect, built |
| **retrograde** | the founded chart's graha rows — **not** in `AnnualSky`, which carries longitudes and nothing else |
| **combust** | `teistro_state::burn::combustion`, which `teistro-tajika` does not depend on and should not |
| **"strong" and "weak"** | `Strength`, **built** (C116): graded on the Vishwa bala, with the dignities as alternatives |
| benefic influence | Tajika's own benefics, which the table does not enumerate as it enumerates the malefics |

### The input shape this decides

`AnnualSky` must **not** grow retrograde and combustion. It is the input
to `panchavargiya` and `drishtis` as well, and neither needs them;
widening it would make every caller supply data for a question they are
not asking.

Instead a second input, filled by the façade from the annual `Document`
and the state crate, exactly as `YearCharts` is filled from two
documents for the office-bearers:

```rust
pub struct AnnualStates { retrograde: Vec<Graha>, combust: Vec<Graha> }
```

**Built as sets, not the `[bool; 7]` first written here.** Seven flags
in some order can be read against the wrong planet and say nothing when
they are; a set names each planet, and `AnnualStates::check` refuses the
entries no chart could hold — a luminary retrograde, the Sun combust, a
node — naming the field. Combustion already takes retrograde as an
argument, so the two belong in one value rather than two. `teistro-tajika` stays free of a
`teistro-state` dependency; the SDK depends on both already.

**And the honesty mechanism extends for free** — built as designed,
with one addition. A caller with only longitudes genuinely cannot answer
Rudda, so `year_yogas` and `year_yogas_with_rules` list the three that
need states under `unanswered`, and `year_yogas_with_states` answers
them. `unanswered` now varies with what the caller supplied, so
`YearYogas::holds` asks the call's own list rather than the build, and
the addition is **`YearYogas::why`**: the build's reason and "this call
was not given the states" have different remedies, so they are
different answers. The façade always supplies the states, read from the
founded chart under the context's combustion table.

### The crux that had to be settled first — and what settling it found

"Strong" and "weak" appear in five of the six, and the source gives a
floor — five Vishwa units — **only** for the year lord. It was
registered as **C116** and measured before any of the five shipped
(`muntha-measured.md` §11), and the measurement overturned the shape
this page had assumed.

**There is a middle.** This page took strong and weak to be one floor's
two sides. A second book grades the Vishwa scale into four — under five
*Nirbali*, five to ten *Madhya*, ten to fifteen *Poorna*, above fifteen
*Parakrami* — so the yogas' *weak* is read as the bottom grade and their
*strong* as *Poorna* or better, with a **middling** band that is
neither. The two books meet at five, which is the one number with two
sources behind it. With one floor at five, 98.9% of the corpus's
readings would have been *strong*; graded, 63.9% are, 34.9% are
middling, and 1.1% are weak.

**A weak planet is rare on this scale whatever the floor.** Weak means a
total under twenty of eighty, and four hostile divisional lords already
cost fifteen, so it needs its sign's lord hostile, the others nearly so,
and a place close to its own debilitation, all at once. Both lords are
weak in **6** of 22 009 judged matters at the default, and 2 871 at a
floor of ten. The yogas that need a weak pair — Dutthottha-Davira and
Durapha — are rare for the reason Khallasara is: the scale, not the
corpus. The pass holds that ceiling as a check rather than a sentence.

**The building found a defect the one-floor design hid.** Written for a
single floor, Dutthottha-Davira asked `!is_strong` where it meant
*weak*; under the graded reading a middling pair walked through. Asked
positively now, and tested with a middling pair.

## The order of work

1. ~~The two corrections above, and `Poorna`, in `drishti`~~ — **done**,
   2026-09-23, alone as planned, because they moved a shipped enum. They
   opened C112, which ships as a reading rather than a decision.
2. ~~The **lagnesha and karyesha** pair, and the four yogas that need
   only the aspects~~ — **done**, 2026-09-23. Ithasala (in all three of
   its kinds), Ishrafa, Nakta and Yamaya. Nakta and Yamaya wanted one
   thing the aspects did not have: the source measures a third planet's
   reach by **its own** deeptamsha and not by the mean it would share
   with each of the pair, so the orb was lifted out of `between` into a
   crate-private `between_within` rather than the four bands being
   written a second time.
3. The ones that need the strength and the chart's dignities. **Manau,
   Kamboola and Khallasara are done** (2026-09-23) — they need a third
   planet's aspects on the pair rather than a strength, so they came
   first and brought `unqualified` with them. **The floors are settled
   and Dutthottha-Davira is done** (2026-09-23, C116): it needed
   nothing but *strong* and *weak*. The rest, **regrouped by their
   hardest blocker** — the survey above had grouped them by the first
   one found, and two were in the wrong group:
   - **Rudda, Duhphali-kuttha and Durapha** needed `AnnualStates`
     (retrograde and combust) — **done** (2026-09-23), with `Affliction`
     beside `Qualification` and `Strength` as the third clause-carrying
     verdict. **Read literally, Rudda spoils 94.4% of the corpus's
     Ithasalas**; the measurement showed the one open clause (C118)
     cannot change that, since the breadth is the five-way list over two
     planets. Recorded, not corrected, as C115 was. Durapha's parse (C119)
     is bounded to six matters by the weak-pair ceiling.
   - **Kuttha** needs Tajika's benefics (C117), and nothing else now.
   - **Tambira** needed a **projection**: the karyesha forming an
     Ithasala from the *next* sign with a lagnesha its present sign does
     not aspect. That is not the Bhavishyat Ithasala, which requires the
     signs to aspect already, so it moves to step 4.
4. ~~**The two that ask what happens next**~~ — **done** (2026-09-23):
   Tambira, and Gairi-Kamboola, which needs the unqualified Moon *and*
   where it will stand in the next sign. One projection serves both:
   the one planet moved to the next sign's first degree, the other six
   held, and the same Ithasala question asked of that sky (C120), so
   no second copy of the four bands was written. The source's worked
   Gairi-Kamboola, Chart X-17, is the acceptance test and comes out as
   printed. It also found **C121**: read literally, X-17 is a
   Khallasara as well, and the source's own comment on Khallasara
   excludes the case. Measured (`muntha-measured.md` §13), Gairi-Kamboola
   holds nowhere in the corpus, and the step that empties it is C115's
   *unqualified* Moon and not the projection. Tambira holds in a handful
   of matters, and more under the source's "some authorities" reading,
   which ships as `YogaRules::tambira`. Tambira needs `AnnualStates`,
   because a retrograde karyesha at a sign's end is going back rather
   than on. `YearYoga::needs_no_aspect` joins the groupings the pass
   holds every count to.
5. ~~Ikabala and Induvara~~ — **done** (2026-09-23), with the three above,
   since the *trika* clause needed the same house-of-a-planet reckoning.
   They are facts about the chart, so they answer every matter alike —
   the first house, with no pair, included — and the pass checks that
   each holds in all twelve of a chart's matters or in none.

## Crossing the boundary

The sixteen answer a **matter**, so a binding cannot simply receive
"the yogas" the way it receives a year's lord: someone has to say which
matters. That is the same shape as the residence question
`varsha_json.place` already answers — the SDK does not decide it for the
caller, and absent, nothing is computed.

### What a caller asks

```json
{ "through": 40, "place": "birth", "matters": [7, 10], "yogas": { "tambira": "either_lord" } }
```

- **`matters`** is `"all"` or a list of house numbers, 1 to 12, in the
  caller's own order. A house named twice is refused by `matters`,
  because it is a mistake and never a request. An empty list asks for
  nothing and is answered with nothing. `"all"` is the word rather
  than the default because twelve judgements a year for two hundred
  years is a real cost, and one a caller asking about marriage did not
  ask to pay.
- **`matters` needs `place`**: the sixteen are read from a year's own
  chart, and without a place no chart is founded. Refused by
  `varsha_json.matters` with that remedy, rather than answered empty.
- **`yogas`** carries `YogaRules` whole — the aspects' `subDegree`, the
  two strength floors and Tambira's mover — as `varshesha` carries the
  year lord's readings. The floors cross as `Bala` does everywhere at
  the boundary: an integer count of sub-sub units, 3600 to a unit.

**One casing on the wire.** Building this found that the boundary read
`VarsheshaRules` in snake case while the request around it, the
residence and `DrishtiRules` are camel case, and that Node's declaration
promised `noneAspects` — so a Node caller who followed their own types
was refused. Both rule records are now camel case on the wire, like
everything around them; Python keeps its own snake-case keys and
translates them, as it does for the place.

### Batch in the signature

`sdk.chart().tajika_yogas_many(&annual, &houses, rules)` answers several
matters of one chart in one call: the sky, the retrograde and combust
planets, the rules' own check and the seven strengths are the same for
every matter, so they are read **once** rather than once a matter. The
single-house entry points are that call with one house.

### What crosses

Each year's own chart gains its **retrograde** and **combust** planets
(bit sets over the graha ids, as the lajjitadi are), because the answers
were judged on them and an answer should be readable without the call
that made it. Then three sections, each ragged by the one before, in
the shape `year_claims` set:

| section | one row per | carries |
|---|---|---|
| `year_matters` | matter asked, per year | the house, its sign, the lagnesha and karyesha, `same_lord`, the pair's own relation (present unless `same_lord`), and `unanswered` as a bit set over `TsYearYoga` |
| `matter_yogas` | yoga that holds, per matter | which yoga; whether it is the pair's relation that made it; the third planet and the planet entering the next sign, each with a presence flag; the two lords' afflictions as bit sets over `TsAffliction` |
| `matter_legs` | leg, per held yoga (none or two) | a pair relation in `year_yogas`' own columns, with a presence flag on the yoga because a leg may make none |

A yoga's `between` is always the matter's own pair or nothing — it is
how the source's judgement *upon an Ithasala* reads — so it crosses as a
flag and not a second copy of the pair. The encoder **refuses** a row
where that stops being true rather than writing a flag that lies.

`unanswered` crosses and `why` does not: the reason is a sentence the
build writes about itself, and a binding reads `unanswered` to tell *did
not hold* from *cannot be told*, which is the half a program acts on.
Each binding's answer has `holds(yoga)`, returning `null` / `None` for an
unanswered yoga exactly as the Rust `Option<bool>` does.

## What is decided and what is not

| | |
|---|---|
| **decided** | that the module takes a matter and answers for it, because the sources define fourteen of sixteen against a karyesha; that the two chart-level yogas answer regardless |
| **decided by measurement** | what "strong" and "weak" mean for a yoga (C116): graded, weak below five and strong from ten, a middling band between, both `YogaRules` fields |
| **decided by the source** | that a Moon completing a Gairi-Kamboola is not also a Khallasara (C121, the source's comment against its own table) |
| **not decided** | whether "benefic influence" means Tajika's own benefics or the chart's (C117), which is all that stands between Kuttha and shipping; how *under malefic influence* and Durapha's list read (C118, C119), each shipped as one reading with every clause carried; what *on entering the next sign* means (C120), shipped as the instant projection with a retrograde lord entering nothing; which lord Tambira moves, shipped as the definition's karyesha with the source's "some authorities" a `YogaRules::tambira` away |
| **built** | every one `YearYoga::awaiting` does not name, through `sdk.chart().tajika_yogas(&annual, house)` and its `_with_rules` twin, which need **no ephemeris**, and `tajika_yogas_many` for several matters at once, which judges the chart's rules, states and strengths once; the crossing, `varsha_json.matters` and `varsha_json.yogas`, in all four bindings; `sdk.chart().qualification`, `strength` and `affliction` for the three clause-carrying verdicts, and `annual_states` for what the longitudes cannot say; `YearYoga::ALL` names all sixteen, `awaiting` says what each unbuilt one needs, matched exhaustively so a yoga cannot be added without a decision, and `judges_an_ithasala`, `needs_a_weak_pair`, `needs_no_aspect` and `is_chart_fact` describe the structure the measured page holds every count to |
| **not built** | Kuttha (C117), naming its blocker at every call — across the boundary too, as a bit in every matter's `unanswered` |

## Why this page exists before the code

The pattern this repository has used since Phase 4: where a source can be
read, read it and write the design first, and expect the building to
correct the page. Here the reading has already corrected two things that
were **shipped**, which is the argument for having read it before writing
sixteen rules on top of them.

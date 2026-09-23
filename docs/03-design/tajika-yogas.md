# The sixteen Tajika yogas

Status: **four of the sixteen built**, 2026-09-23; the other twelve
designed and *named*, which is not the same as absent. Steps 1 and 2 of
the order of work below landed the same day. Written after reading the
source's Table X-3 off the page and before writing any of it, as every
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
3. The ones that need the strength and the chart's dignities: Manau,
   Kamboola, Khallasara, Rudda, Duhphali-kuttha, Dutthottha-Davira,
   Tambira, Kuttha, Durapha.
4. **Gairi-Kamboola** last: it needs the unqualified Moon *and* a
   projection of where the Moon will be in the next sign, which is the
   only one of the sixteen that asks what happens next rather than what
   is.
5. Ikabala and Induvara, which are two lines and belong with the rest for
   the sake of the set being whole.

## What is decided and what is not

| | |
|---|---|
| **decided** | that the module takes a matter and answers for it, because the sources define fourteen of sixteen against a karyesha; that the two chart-level yogas answer regardless |
| **not decided** | what "strong" and "weak" mean for a yoga, where the source gives a floor only for the year lord; whether "benefic influence" means Tajika's own benefics or the chart's readings; the two cruxes above |
| **built** | Ithasala, Ishrafa, Nakta and Yamaya, through `sdk.chart().tajika_yogas(&annual, house)` and its `_with_rules` twin, which need **no ephemeris**; `YearYoga::ALL` names all sixteen and `YearYoga::awaiting` says what each of the other twelve still needs, matched exhaustively so a yoga cannot be added without a decision |
| **not built** | twelve of the sixteen, each naming its blocker at every call. They do **not** cross the boundary yet: the sixteen answer a *matter*, so crossing them means deciding which matters a caller asks for — the same shape as the residence decision `varsha_json.place`, and its own unit |

## Why this page exists before the code

The pattern this repository has used since Phase 4: where a source can be
read, read it and write the design first, and expect the building to
correct the page. Here the reading has already corrected two things that
were **shipped**, which is the argument for having read it before writing
sixteen rules on top of them.

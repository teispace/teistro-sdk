# The sixteen Tajika yogas

Status: `designed`, 2026-09-23 — **not built**. Written after reading the
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

```
sdk.chart().tajika_yogas(&annual, Bhava::Seventh) -> Vec<Yoga>
```

with the two chart-level yogas answered whatever is asked. A caller who
wants "every yoga for every house" asks twelve times, which is honest
about what it is doing.

## What each of the sixteen needs

Everything below is already built unless marked.

| need | where it is |
|---|---|
| the Tajika aspect and the deeptamsha orbs | `drishti` |
| Ithasala, Ishrafa and the sign's-end kind | `drishti` |
| "strong" and "weak" | the Vishwa bala, `bala` — the source's own floor of **five units** is stated for the year lord and **not** for the yogas (crux) |
| own Hudda, Drekkana and Navamsha | `bala`'s lords, which "unqualified" needs |
| exalted, debilitated, retrograde, combust | the founded chart's own graha rows |
| kendra, panaphara, apoklima, trika | whole-sign houses from the annual lagna |
| benefic and malefic | Tajika's own reckoning: **Mars and Saturn** are the malefics Manau names |

Nothing here is missing. The module is a composition, which is why it was
worth building the aspects and the strength first and separately.

## Two corrections to make first

The reading found the book disagreeing with itself twice, and
`crates/tajika` currently ships the prose's reading of both:

1. **Ishrafa's degree** (crux C110). The table requires the faster planet
   to be **one degree or more** ahead; the prose says only "ahead", within
   the orb. They differ for a pair less than a degree past. The finer test
   goes in, with the pass counting how many pairs over the recorded years
   the two readings disagree on — a number, not a guess.
2. **The sign's-end Ithasala's name** (crux C111). The table calls it
   **Bhavishyat** and lists it as one of three kinds beside Vartamana and
   Poorna; the prose calls the same configuration a *Rashyanta* Ithasala.
   The table's name ships, because the table is what enumerates the kinds.

And one thing the table adds that is not built at all: **Poorna**, the
Ithasala within a single degree, which the source marks as immediate
fulfilment.

## The order of work

1. The two corrections above, and `Poorna`, in `drishti` — small, and they
   move a shipped enum, so they go first and alone.
2. The **lagnesha and karyesha** pair, and the four yogas that need only
   the aspects: Ithasala's three kinds, Ishrafa, Nakta, Yamaya.
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
| **not built** | all of it |

## Why this page exists before the code

The pattern this repository has used since Phase 4: where a source can be
read, read it and write the design first, and expect the building to
correct the page. Here the reading has already corrected two things that
were **shipped**, which is the argument for having read it before writing
sixteen rules on top of them.

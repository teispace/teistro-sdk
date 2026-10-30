# Printed conventions: readings an almanac prints without a text

Status: `built`, 2026-10-30. Written before the code, from the prints and
the texts searched (the application's SR-20, addendum 3).

An almanac prints readings that no text read gives. Shiva vasa is one:
a patro prints where Shiva dwells on each tithi, and the SDK found no
verse for it (C750). Leaving such a reading out sends an application to
compute it by hand. Shipping it as a rule would claim a text that is not
there. This page decides a third way: the reading ships as a named
**convention**, which says it has no text behind it, which a setting
chooses, and whose default never refuses.

## 1. What counts as a convention

A reading is a convention when all three hold:

1. an almanac in print gives it, cited by edition and page;
2. the texts searched for it do not give it, each search recorded in its
   crux with the editions read;
3. it is a fixed function of what a day already carries (a tithi, a
   vara, a span), so the SDK can compute it for any day and place.

A reading a text does give is a **rule**, cited by verse, and none of
this page applies to it. Bhadra's loka is a rule (Muhurta Chintamani,
Shubhashubha-prakarana v. 45, C820), and so is the agni vasa (C748).
When a text is found for a convention, the reading becomes a rule: its
knob keeps its members, the provenance entry of §3 goes, and its
catalogue members' mark becomes `V`.

## 2. The two shipped

| reading | the printed rule | printed in | texts searched | crux |
|---|---|---|---|---|
| Shiva vasa | the tithi *t*, 1 from Shukla Pratipada to 30 at Amavasya; (2*t* + 5) mod 7: 1 Kailasa, 2 beside Gauri, 3 on the bull, 4 the assembly, 5 the meal, 6 at play, 0 the cremation ground | Surya Panchanga VS 2082, p. 60, "शिववासफलसारिणी, मेरुतन्त्रोक्तम्", a table of all thirty tithis that the rule reproduces in every cell | the *Meru Tantra* the print names (Venkateshwar Press, 1908, three scans' OCR); *Muhurta Chintamani* (1906 and the 1954 *Piyushadhara*); the *Narada Samhita* (Hindi edition) | C750 |
| the panchaka's name | the vara the panchaka begins on: Sunday Roga, Monday Raja, Tuesday Agni, Friday Chora, Saturday Mrityu; Wednesday and Thursday give none | Indian almanacs and their web editions, which name each month's panchaka so | *Muhurta Chintamani* (both prints), the *Muhurta Martanda* (Khemraj, 1910), the *Narada Samhita*; Muhurta Chintamani's *Vivaha-prakarana* v. 72 names the five by weekday for the **bana** (the remainder panchaka, `omen::panchaka_remainder`), a different table: Sunday roga, Tuesday agni and chora, Saturday nripa, Wednesday mrityu | C821 |

The prints in Nepal read do not name the panchaka by weekday: they
print it as धनिष्ठापञ्चक with its hours. The weekday names are the
Indian almanacs'.

The Shiva vasa's fruits are the print's too: it gives Shiva at the meal
as food gained, favourable, where web almanacs give pain (C822), and
`ShivaVasa`'s `favourable` attribute follows the print.

## 3. The shape

Each convention is three things the SDK already has, and nothing new
beside them.

1. **A knob** in the `panchanga` group, one per reading, whose members are
   the printed rule (or rules, when prints differ) and `NONE`:
   `panchanga.shiva_vasa` (`PRINTED`, `NONE`) and `panchanga.panchaka_name`
   (`WEEKDAY`, `NONE`). The root profile takes the printed rule, so the
   default answers for every day and refuses none; `NONE` writes nothing.
   A knob's members are the place a second print's rule goes, as a
   second member, when one is found.
2. **The provenance's `applied_conventions`.** Every almanac answer whose
   settings name a convention other than `NONE` carries an entry for it:
   the knob, the value, and a reason saying the reading is printed and no
   text read gives it, with its crux. This is the field an answer already
   uses for a choice the SDK made to terminate (the festival days' reach,
   the eclipse window), so every binding, the C boundary and MCP carry it
   now. The list comes from one table, `omen::PRINTED_CONVENTIONS`, which
   names each knob, its crux and the print, so a reading added there is
   flagged in every answer without another line.
3. **Catalogue marks.** A convention's members are catalogued with mark
   `T` (traditional: in use, awaiting a citation) and the print as their
   source, where a rule's are `V` with the verse.

What it is not: a second provenance channel, a per-value flag, or a
separate request. The value crosses as any span does; what says it is a
convention is the stamp on the answer and the mark on the member.

The `confidence` field is left as it is. It speaks of catalogue rows that
took part in a computation (ADR-0018); a convention is named by its own
entry, which is the more specific statement, and an application that sets
the knob to `NONE` gets an answer with no convention in it.

## 4. Where each value goes

| reading | Rust | C boundary | bindings |
|---|---|---|---|
| Shiva vasa | `omen::shiva_vasa(tithi)`, `omen::shiva_vasas(tithis, reading)`; `Panchanga::shiva_vasa` reads the knob the day was computed under | span section `shiva_vasa` | `shiva_vasa`/`shivaVasa` on the day |
| the panchaka's name | `omen::panchaka_by_vara(vara)`; the day's `named_panchaka`, one span or none: `whole` the panchaka from its first instant to the Moon leaving Revati, `inside` its part in the day, the member the name | span section `named_panchaka` | `named_panchaka`/`namedPanchaka` on the day |

The Shiva vasa is a method over the stored tithis, like the agni vasa.
The panchaka's name needs the panchaka's first instant, which may lie
days before the day asked; the day's own panchaka spans are extended,
not recomputed: the day's first span gives it when the panchaka begins
on the day, and otherwise one search of the Moon's signs (or its
nakshatras, when `panchanga.panchaka_start` is `DHANISHTHA`) before that
span finds it, read through `omen::panchaka` itself. The weekday is the
vara of the day that instant falls in, sunrise to sunrise, the vara the
SDK gives every day. Only a day holding a panchaka searches, five or six
days a month.

## 5. The ones that fit later

Each of these is a printed reading the SDK has declined so far, and each
fits §3 unchanged: a knob with the print's rule and `NONE`, an entry in
`PRINTED_CONVENTIONS`, and catalogue members marked `T`.

- Rahu's weekday direction, one for the whole day (C749 builds the
  verse's walk over the daylight).
- The maha punya kala inside a sankranti's window (C754).
- A sait's lagna preference where the committee's print is the only
  witness (C745 already ships as a reading of a sait rule; it would move
  here if it became a knob).

## 6. The falsification

There is no corpus of Shiva vasa or weekday names to measure. What is
measured instead is the rule against the print it is taken from: the
Shiva vasa rule is held to every one of the thirty tithis of the Surya
Panchanga's table by a test, so a slip in the arithmetic, or a print
that does not follow the rule, fails it. The weekday names have no table
to hold, so the test holds the seven varas to the list in §2.

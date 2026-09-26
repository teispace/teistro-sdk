# Jaimini significators: the karakamsha, the Brahma graha and the Sthira dasa

Status: `draft`, 2026-09-26; §5 steps 1 to 4 **built** the same day.
Written from the texts and a measurement before any code; the building is
expected to correct it.

Derives from `dasha-coverage-measured.md`, which lists `STHIRA` under
"the text is not settled" with a stated row — "from the lagna,
consecutive, seven, eight or nine years by modality" — and `SUDASA`
under "it waits on a module", the karakamsha; and from the roadmap's
Phase 7 list of what is left of `jaimini`: the karakamsha and swamsa,
the graha arudhas, the sthira karakas, and Brahma, Rudra and Maheshwara.
This page takes the two the dashas wait on, the karakamsha and Brahma,
and the dasha Brahma starts.

## 1. What the texts say

**The karakamsha** is "the Navamsa occupied by the Atma Karaka planet"
(BPHS ch. 33 v. 1, Santhanam's note; ch. 32 for the Atmakaraka). The
definition is not in dispute. What is, is the chart its houses are
counted in (§3, C130).

**The Sthira dasa** (BPHS ch. 46 vv. 168 to 169): seven, eight and nine
years for a movable, a fixed and a dual sign — चरे सप्त स्थिरे
[अष्टौ द्व्यङ्गे नव] समाः स्मृताः — and ब्रह्मखेटाश्रितर्क्षादि
दशेयं परिवर्तते, **the dasha revolves from the sign Brahma occupies**,
not from the lagna. The translation adds that the dasas "are counted
onwards from the odd signs and in the reverse order from the even
signs"; the Sanskrit says nothing of order (C129).

**The Brahma graha** (vv. 170 to 173), read in the Sanskrit. The text
here is the archive.org OCR of the Sanskrit printed beside the
translation, and where the OCR garbles a word the reading in brackets is
this page's: v. 168's middle (the OCR's "स्थिरे sect gg qa", which the
printed page reads स्थिरे चाऽष्टौ द्वन्द्वे नव, eight years for a fixed sign
and nine for a dual, as the translation gives) and v. 172's "अष्टमेशो"
(the OCR's "अष्टसेशो"), which is still to be read against the printed
page before a crux is closed on it.

- v. 171, षष्ठाष्टव्ययनाथेषु यो बली विषमर्क्षगः पृष्ठस्थितो भवेद्
  ब्रह्मा बलिनो लग्नजाययोः: of the lords of the 6th, the 8th and the
  12th, the strong one, **in an odd sign** and **standing behind** the
  stronger of the lagna and the 7th, is Brahma.
- v. 172, कारकाद[ष्टमे]शो वा ब्रह्माऽप्यष्टभावगः । शनौ पाते च ब्रह्मत्वे
  ब्रह्मा तत्षष्ठखेचरः: an alternative from the karaka's 8th lord in
  the 8th house, which the translation does not render; and **when
  Saturn or the node gains Brahma-hood, Brahma is the planet in the 6th
  from it**. The translation reads the second half the other way, that
  Saturn or the nodes "become Brahma Graha".
- v. 173, बहवो लक्षणाक्रान्ता ज्ञेयस्तेष्वधिकांशकः । अंशसाम्ये बलाधिक्याद्
  विज्ञेयो ब्रह्मखेचरः: when several carry the marks, the one of **more
  degrees**; with degrees equal, the stronger.

The translator's note gives a different rule altogether: the 8th lord
in the 8th, else any planet in the 8th, else "the planet in an odd sign
within 6 signs from the Ascendant or the 7th", the most degrees among
several, Rahu's degrees taken from the end of his sign.

**The worked example** (after v. 173) is the acceptance case: lagna
Aquarius 16°28′, Mercury in Aquarius 13°06′. There is no planet in the
8th; the lagna, with three benefics, is stronger than the 7th; counting
back six signs from it, the 8th lord Mercury stands in the odd sign
Aquarius — the lagna itself — so Mercury is Brahma and the Sthira dasa
runs Aquarius 8, Pisces 9, Aries 7, Taurus 8 and on.

## 2. Measured

PyJHora 4.8.7 was run as a black box (CLEAN_ROOM rule 3: its values,
never its code) over 600 charts drawn at random from 1900 to 2050 and
55° S to 60° N, and its Brahma recorded beside the positions it was
given:

- on the worked example it answers **Mercury**, as the text does;
- its answer is always a lord of the 6th, 8th or 12th from the lagna or
  the 7th (600 of 600), and never Rahu or Ketu;
- only 338 of its 600 stand in an odd sign, so it has a fallback the
  verses do not state;
- its Sthira dasa starts at its Brahma's sign on all 200 of 200 charts
  and runs **forward from an even sign too** (87 of 87), which is the
  Sanskrit's silence on order read as consecutive, and not the
  translation's.

The verses read strictly (the default of §4) **find no Brahma on 272 of
the 600 charts**, 45%: no lord of the 6th, 8th or 12th stands in an odd
sign behind the stronger sign. Where they do find one they agree with
PyJHora on 119 to 136 of 328, by which way a tie is broken. The best
fitting reading of the ones tried agreed on 300 of 600, so PyJHora's
rule is not any reading of these verses this page could make, and it is
not shipped: a black box cannot be cited.

## 3. The forks, registered

| crux | question | readings | default | why |
|---|---|---|---|---|
| C124 | what "behind" (पृष्ठस्थित) the stronger sign covers | the sign itself and the five before it, counting back six, as the worked example counts; or the 7th to the 12th from it | the six counted back | the only count the source demonstrates |
| C125 | how several qualifying lords are ordered | more degrees, then strength (v. 173); or strength first, reading यो बली as a ranking | degrees, then strength | v. 173 states it; यो बली is read as a mark the three share, the stronger sign they are counted from |
| C126 | whether Scorpio's and Aquarius's second lords are lords here | the Jaimini co-lords count; or the one lord | the co-lords count | the chapter's own lordship (vv. 158 to 166, `stronger_lord`) |
| C127 | what Saturn or the node qualifying means | Brahma passes to the planet in the 6th from it (the Sanskrit); or it is Brahma (the translation) | the Sanskrit | the verse; the translation contradicts its own text |
| C128 | what a chart with no qualifying lord has | no Brahma (the verses); or the translator's note's rule | no Brahma, **refused by name**, the note a knob | the verses are silent and a silent fallback is a guess; the note is cited and selectable |
| C129 | the order of the Sthira dasa's signs | consecutive from Brahma's sign; or forward from odd and back from even (the translation) | consecutive | the Sanskrit states no order; PyJHora runs forward on every chart, and its antardashas forward from their own sign on 480 of 480 |
| C130 | the chart the karakamsha's houses are counted in | the rasi chart, from the sign the Atmakaraka's navamsha names (the recording engine); or the navamsha chart itself | both, reported | the schools divide on it and a consumer reading one wants the other named |
| C131 | the Yogardha dasa's order | by the start's parity (the translation, its worked example and PyJHora); or forward | by parity | v. 174 in Sanskrit states the span and the start and no order; PyJHora's antardashas, one fixed sequence for every mahadasha, are declined for the kernel's (C50) |

v. 172's first half, the karaka's 8th lord in the 8th, is **not built**:
the verse is not rendered in the translation and "karaka" is not said
to be the Atmakaraka. It is recorded under C127's row as unread.

## 4. Decided

**The karakamsha** is the Atmakaraka's navamsha sign, the Atmakaraka
the one the profile's chara karaka scheme (`jaimini.chara_karakas`)
ranks first. A chart reports it with every graha's house counted from
it in the rasi chart and in the navamsha, so both readings of C130 are
answers rather than a choice.

**The Brahma graha** is the verses' rule under C124 to C127's defaults:
the lords (with co-lords) of the 6th, 8th and 12th from the stronger of
the lagna and the 7th by the chapter's own sign strength
(`stronger_sign`, vv. 158 to 166, the lagna where they tie), those in
an odd sign among the six signs counted back from it, the most degrees
among them — Rahu's and Ketu's taken from the end of the sign — and the
stronger sign on equal degrees; and where the one chosen is Saturn, Rahu
or Ketu, the planet in the 6th sign from it with the most degrees. A
chart with none has **no Brahma**, and says why.

`jaimini.brahma`, a knob: `VERSES` (the default) or `TRANSLATORS_NOTE`,
the note's rule in full, cited as the translation's commentary.

**The Sthira dasa** is a row of the sign-based kernel: it starts from
Brahma's sign, runs consecutively forward, and gives a sign seven,
eight or nine years by its modality. A chart with no Brahma refuses it
by name, the refusal saying which rule found none and which knob
supplies one. `Start` gains `Brahma`, and the kernel's chart gains the
sign, which is a start and not a period: ADR-0017's kill criterion is
about a third chart-query field for **periods**, and this is not one.

## 5. The order of work

1. **The karakamsha** (`dasha::jaimini::karakamsha`): **done**, both
   readings of its houses, through `sdk.chart().jaimini`. At the
   boundary it waits on step 6.
2. **The Brahma graha** under both rules: **done**, measured over the
   corpus's births in `jaimini-measured.md` (the verses find none on 32
   of 55).
3. **The Sthira dasa**: **done**. `Start::Brahma` and `Order::Forward`
   in the sign-based kernel; `start_sign` is fallible, so a Sthira dasa
   over a chart with no Brahma is refused on `jaimini.brahma` with the
   note's rule named, never started from the lagna. The coverage page's
   stated row, and its registrable entry, are gone; this build computes
   23 of the catalogue's 40. The corpus does not record Sthira, and the
   dasha tests list it as the one shipped row unrecorded, both ways.
   Its antardashas follow the row's order: forward from their own sign,
   equal, as the mahadashas are, where every other sign-based row runs
   them by the sign's parity. The verses are as silent on the sub-periods
   as on the periods, and PyJHora runs them forward from their own sign
   on 480 of 480 (240 from an even sign), so C129's reading holds at
   both levels. **Building it found a request for everything refused**
   on nearly half of charts: `with_everything` asked for every computed
   system, and a Sthira over a chart with no Brahma refuses the whole
   reading. It now asks for `dasha::systems_every_chart_gives()`, the
   systems whose start every chart gives (`Start::every_chart_gives`),
   and the Sthira dasa is asked for by name, where its refusal names the
   setting that supplies a Brahma.
4. **Yogardha** (v. 174): **done**. Read on the printed page, the
   Sanskrit states the span, half the sum of the Chara and Sthira years,
   and the start, the stronger of the lagna and the 7th — and, as for
   the Sthira dasa, **no order**: this page had said v. 174 "does" state
   one, which was the translation's. The translation's order, forward
   from an odd sign and back from an even one, is also its worked
   example's and PyJHora's on 200 of 200 charts, so it is the default
   (C131). It is **not** a composition over kernels, as the coverage page
   supposed: it takes the Sthira dasa's years and not its start, so it
   needs no Brahma, and it is one `Length`, `MeanOfCountAndModality`,
   inside the sign-based kernel, whose `years` now answers half-years.
   Its start is Mandooka's, `stronger_of: [1, 7]` under
   `dasha.rashi_start`. **The printed table is reproduced but two named
   cells**: Leo, where the book's 8 contradicts its own Chara table's 7
   (7½ is computed), and Scorpio, where both of the book's tables count
   to Mars against C51's modality step, which reopens C51. Its Capricorn
   4½ counts two signs back to Saturn, as computed, where its Chara
   table's 5 counts nowhere. PyJHora gives every mahadasha the same
   antardashas, from the dasha's second sign, on 720 of 720; no text
   states that, and the kernel's own (C50) stand.
5. **Sudasa**, which the coverage page says waits on the karakamsha:
   confirm from the source which point it starts from before building,
   because the Sree lagna, which is built, is the other candidate.
6. **The reading at the boundary**: the karakamsha and Brahma as a
   document section every binding reads, after the pattern of the
   Vaiseshikamsa.

## 6. Not decided here

Rudra and Maheshwara, the sthira karakas and the graha arudhas are the
rest of the roadmap's Jaimini list. PyJHora answers each, and each is
its own reading of BPHS or the Jaimini Sutras; they are taken after
these, with the same method.

# Gochar: the transits read from the natal Moon

Status: `built`, 2026-09-26: §6 steps 1 to 3 built here, and step 4's
parts built on their own pages ([`sade-sati.md`](sade-sati.md),
[`transit-hit-list.md`](transit-hit-list.md),
[`gochar-ashtakavarga.md`](gochar-ashtakavarga.md)). Written
from the text before any code; the building is expected to correct it.

Derives from `01-research/feature-universe/11-transits-gochar.md` (P0:
"classical gochar results per planet from the Moon sign with vedha
points and exemptions"; its closing checklist asks for the vedha table
with citations and the exemptions confirmed) and the roadmap's Phase 7
`gochar`. This page takes the snapshot, the one step every later part
reads: the hit list, the transit calendar, Sade Sati's phases and the
Ashtakavarga's transit scoring all ask of an instant what this answers.

## 1. What the text says

Phaladeepika ch. 26 (Mantreswara; V. Subrahmanya Sastri's 1950 edition,
pp. 286 to 288), read in the Sanskrit on the printed page. The archive's
OCR of the Sanskrit is noise and one of its English numerals reads
"Bill", so every number below is the page's.

- **v. 1**, सर्वेषु लग्नेष्वपि सत्सु चन्द्रलग्नं प्रधानं खलु गोचरेषु: of all
  the lagnas, **the Moon's is the one for transits**; count every transit
  from the natal Moon's sign.
- **v. 2**, the houses from it where each transit is good: the Sun
  षट्-त्रि-दश (6, 3, 10); the Moon त्रिदश-षट्-सप्त-आद्य (3, 10, 6, 7, 1);
  Jupiter अस्त-तप-द्वि-पञ्चम (7, 9, 2, 5); Mars and Saturn षट्-त्रि (6, 3);
  Mercury षट्-स्व-चतुर्-दश-अष्टम (6, 2, 4, 10, 8); **every graha in the
  11th** (उपान्त); Venus in all but ख-अस्त-रिपु (10, 7, 6); and the nodes
  **like the Sun** (तिग्मांशुवत्).
- **vv. 3 to 8**, each good house's **vedha**: the good result holds only
  if no other graha then transits the paired house.

| graha | good house → its vedha house | not obstructed by |
|---|---|---|
| Sun (v. 3) | 11→5, 3→9, 10→4, 6→12 | Saturn (व्यार्किभिः; the note: no vedha between father and son) |
| Moon (v. 4) | 7→2, 1→5, 6→12, 11→8, 10→4, 3→9 | Mercury (विबुधैः) |
| Mars (v. 5) | 3→12, 11→5, 6→9 | — |
| Saturn (v. 5) | as Mars | the Sun (घर्मघृणिना न विध्यते) |
| Mercury (v. 6) | 2→5, 4→3, 6→9, 8→1, 10→8, 11→12 | the Moon (विविधुभिः) |
| Jupiter (v. 7) | 2→12, 11→8, 9→10, 5→4, 7→3 | — |
| Venus (v. 8) | 1→8, 2→7, 3→1, 4→10, 5→9, 8→5, 9→11, 12→6, 11→3 | — |

The exemptions are mutual as stated: the Sun and Saturn do not obstruct
each other (vv. 3, 5), nor the Moon and Mercury (vv. 4, 6). The print of
Mercury's 11th pair is unclear (प्राप्ति or प्रान्त्य); the translation
reads the 12th, which is the table's usual pair, and this page takes it.

- **v. 25**, when a transit bears fruit within a sign: the Sun and Mars in
  its **first** decanate, Jupiter and Venus in the **middle** one, the Moon
  and Saturn in the **last**, Mercury and Rahu **throughout**.

vv. 9 to 24 give each graha's effects in the twelve houses; they are
readings, carried by the interpretation corpus when a vetted rendering
exists, and not this module's. vv. 26 to 28 are the nakshatra transits:
the seven-line *saptashalaka* of 28 stars (v. 26, the Sun's vedha of the
janma, adhana and karma stars; v. 27, the other grahas') and the taras
(v. 28). The Sarvatobhadra chakra is v. 48, which reads its vedhas into
gochar; it is built as the second layer (§8). The saptashalaka and the
taras are not built (C428).

## 2. What was measured

No source to measure against: the corpus records no transit, and PyJHora
4.8.7 has no gochar or vedha function (a signature search finds none). So
the acceptance values are the table above, asserted cell by cell, and
the pass (`gochar-measured.md`, `cargo xtask gochar`) measures what the
table does over the real sky rather than against a recording:

- **Who obstructs whom**, over every day from 1960 to 2020 read from all
  twelve reference signs: each of the 72 ordered pairs obstructs every
  time it stands in the other's vedha house, or is spared by exactly the
  verses' exemptions and C140, or never stands there because Mercury and
  Venus never stand opposite the Sun. Each list is written in the pass and
  held both ways.
- **The literal reading of the nodes**, measured: in 175 320 node readings
  in a good house, the other node stood in the vedha house every time
  (C140).
- **The forks, over the recorded births**: counting from the lagna rather
  than the Moon moves 53% of the verdicts (C139); the nodes' readings move
  2% to 5%.

## 3. The forks (cruxes)

| crux | question | readings | default | why |
|---|---|---|---|---|
| C136 | the nodes' vedha | the Sun's pairs, "like the Sun" read whole; none, since v. 2 compares only the good houses | **the Sun's pairs**, a knob for none | तिग्मांशुवत् qualifies the nodes without restriction; the Sun's exemption of Saturn is father and son (v. 3's note) and is **not** carried to them |
| C137 | whether the nodes obstruct others | yes, as grahas (ग्रहैः, खेचरैः); no | **yes**, a knob for no | the verses say "planets" and name the exceptions; the nodes are not among them |
| C138 | whether Ketu bears fruit throughout, as Rahu | as Rahu; by no decanate | **as Rahu** | v. 25 names Rahu alone; the nodes are paired everywhere else in the chapter |
| C139 | what gochar counts from | the natal Moon's sign (v. 1); the lagna (JHora's second reference) | **the Moon**, the request naming another | v. 1 is explicit; the lagna is a consumer's choice, not the text's |
| C140 | whether the nodes obstruct each other | no; yes, the verses read literally | **no**, a knob value for yes | found while planning the pass: the nodes always stand opposite and the Sun's pairs are opposite houses, so under C136's default each node in a good house has the other in its vedha house; read literally, v. 2's good houses for the nodes are never good, and the text is not read as voiding its own clause |

## 4. The design

A new crate, `teistro-gochar`, depending on `teistro-core` alone: the
table as data and one function,
`gochar(reference: Rashi, transits: &[Transit; 9], rules: GocharRules) -> [GrahaGochar; 9]`,
where a `Transit` is a graha's sign and its degrees within it. Each
`GrahaGochar` carries the house from the reference, whether that house is
good (v. 2), its vedha house when it has one, **which grahas obstruct it**
(named, not only counted, so a reader can check the verdict), the verdict
— `Good`, `Obstructed` or `NotGood` — and whether the transit is in its
fruitful decanate (v. 25).

Two knobs in a new `gochar` settings group carry C136 and C137. The
façade reads the natal Moon's sign from the natal chart and founds the
transit chart at each instant at the natal place:
`sdk.chart().gochar(&natal, at)` and a batch form over many instants
that reads the reference once.

## 5. Tests

- Every cell of the table, both ways: each good house is good, each other
  house is `NotGood`, each vedha pair obstructs, and each exemption does
  not.
- The mutual exemptions: the Sun unobstructed by Saturn and Saturn by the
  Sun; the Moon by Mercury and Mercury by the Moon.
- Venus's nine houses, since v. 8's translation phrases them as bad
  effects when marred, which is the same rule.
- The nodes under both knobs, and the decanate of v. 25 at its edges.
- Through the façade: the reference is the natal Moon's sign, and the
  batch form agrees with one call per instant.

## 6. Order of work

1. The crate, the knobs and the façade, with the tests above: **done**.
   Planning step 2 then found C140: the unit tests placed grahas where no
   sky puts them (both nodes in one sign), which hid that on the real sky
   the literal reading never lets a node be good. `node_obstruction`
   gained the third value before the knob shipped, and a façade test holds
   it over three years of the real sky.
   `teistro-gochar` depends on `teistro-core` alone; the façade founds the
   transit charts at the natal place in one batch and seals them with the
   founder's provenance. Adding the `gochar` settings group found
   `SettingsPatch::is_empty` leaving out the `panchanga` group, so a patch
   setting only a panchanga knob called itself empty; it now compares with
   the empty patch, which cannot miss a group, and a test sets each knob of
   every group alone.
2. The measured pass, held by `check-gochar`: **done**, and §2 says what
   it found. Pricing the batch first found the façade founding a whole
   chart per instant — its day, houses and two lagnas — to read nine
   longitudes: `Founder::longitudes` places the grahas at every instant in
   one request, without the speeds' second request either side, each place
   equal to the founded chart's to the bit (a façade test holds it under
   the default, the topocentric conformance profile and the true node,
   proved red by a 1e-9° nudge). 240 instants: 110 ms founded, 46 ms
   placed, warm, under the gates' build profile. Assembling a document's
   sections was measured separately and costs nothing visible; the
   founding is the cost.
3. The boundary and the bindings: **done**. A request's `gochar_json`
   (`{instants, from}`, read by `GocharRequest::from_json`, which refuses
   no instant as `gochar.instants`) answers sections 53 and 54: `gochar`,
   a row a chart an instant, and `gochar_grahas`, nine under each, with
   `obstructed_by` a nine-graha bit set and the vedha house 0 for none.
   **Fixed, not ragged**, since the request settles how many instants
   every chart gets, so each binding checks the rows divide by the chart
   count rather than trusting it. Every binding reads it as
   `chart.gochar`, the parity gate compares every graha of two instants
   across every runner, and an ABI test holds every cell to the façade's own
   reading. Crossing it found a reading saying its reference sign but not
   **what** it was counted from: `Reference { from, sign }` now says both,
   and `GocharFrom` moved into the crate. The node knobs cross by `of`,
   which answers `None` for a member the boundary does not know rather
   than guessing, and a test holds every member to its own code; the
   Brahma rule's version of that test deduplicated without sorting, which
   passes two equal codes that are not neighbours, and now sorts first.
4. Sade Sati (its phases need a source: Phaladeepika v. 22 reads Saturn
   over the Janma rasi and does not name the seven and a half years;
   **built** at rank 4 as practice defines it, `sade-sati.md`), the
   hit list over the crossing search, and the Ashtakavarga's transit
   scoring.

## 7. Said in words

A reading crossed every binding as numbers and codes, and an application
holding one could show only those: building a test app on the SDK, the
gochar screen printed the reading's JSON (2026-10-14), because nothing
turned a verdict into a sentence. `teistro_interpret::gochar` does, and
`sdk.interpret().gochar(&reading)` reaches it.

- **What it says**, in the order a reader takes it in: the sign the houses
  are counted from and whose it is (`sdk.reason.gochar.from`); each graha
  the Sun to Ketu in its house with its verdict (`transit`); for an
  obstructed one, who obstructs it and from which house (`vedha`); where in
  its sign it bears fruit and whether it stands there now (`fruition`, v.
  25); and, when the request asked for the Ashtakavarga, each of the
  seven's bindus with the reading's own good-or-not (`ashtakavarga`). Five
  messages in English and Nepali, the Nepali in the verses' terms (गोचर,
  वेध, शुभ, अशुभ, बिन्दु).
- **It judges nothing.** The verdict and the threshold are the kernel's
  under the settings' `gochar` group, and a sentence neither softens nor
  sharpens one: "favourable, but obstructed" is `OBSTRUCTED`, not a third
  opinion. It needs no corpus, unlike Sade Sati's readings, because the
  rule is the SDK's and so are its words.
- **One plan an instant.** A plan is a list of things to say about one
  sky; a batch of instants is a batch of plans, so `Plans.gochar` is a
  list of plans in the request's order rather than one plan that would
  have to date each sentence.
- **Left unsaid**, on purpose: the kakshya and the sign's sarva (v. 16's
  "others say"), the vedha house when nothing stands in it (that is what
  `GOOD` means), and the fruition of the three that bear fruit throughout
  their signs (C138), which tells a reader nothing.

**In a plan request.** `gochar` is a member of `PlanRequest` like
`sadeSati`: it reads no section and **needs a gochar request beside it**
(`PlanInputs::with_gochar`; at the boundary, the chart request's
`gochar`), refused as `interpret.gochar` without one. The transits are
read **once**: `ChartArea::interpreted` reads them beside each chart and
hands them back as `Interpreted::gochar` as well as saying them, and
`compose` takes its `gochar` table from there instead of reading it a
second time.

**Measured.** `interpret-measured.md`'s founded chart says its transits a
month apart over two years with the Ashtakavarga, which emits all five
keys and renders each in both strict locales with no fallback and no
warning; a façade test (`crates/sdk/tests/plans.rs`) holds the plans to
the composer over the same call's readings, one an instant, and the Node,
Python and Dart suites hold the refusal and the shape across the boundary.

## 8. The Sarvatobhadra chakra: the second layer

Status: `built` 2026-10-17 in the kernel (`teistro_gochar::sarvatobhadra`),
the façade 2026-10-18 (§8.5). Cruxes C428 to C432.

### 8.1 What the texts say

**Where it is.** Phaladeepika ch. 26 v. 48 (Sastri's 1950 edition, p.
301, read on the page image): सर्वतोभद्रचक्रोक्तशुभवेधाः शुभावहाः ।
पापवेधा दुःखतरा गोचरे तांश्च चिन्तयेत्, the chakra's benefic vedhas bring
good and its malefic ones sorrow, to be weighed in gochar. That is all
Mantreswara says of it. vv. 26 to 28, which §1 once named as this chakra,
are the seven-line saptashalaka and the taras (C428). The chakra itself
is *Narapati Jayacharya Svarodaya*'s, the text C291 to C294 already read
for naam milan: the 1946 Khemraj print with Harivansha's Jayalakshmi
tika, pp. 55 to 62 (Internet Archive
`QwjT_narapati-jayacharya-svarodaya-of-shri-narapati-with-jaya-lakshmi-tika-of-harivansha-1946-khemar`,
leaves n64 to n71), every verse read on the page image. Sastri's
edition prints under v. 48, pp. 305 to 307, the same verses from
*Jatakabharana* and *Horaratna*, with Rajavijaya's and *Svara
Chintamani*'s, which agree with *Svarodaya* except where §8.3 says.

**The grid** (vv. 2 to 9). Ten lines each way make 81 cells (v. 2). The
sixteen vowels a to the visarga go to the corners, the outermost first,
north-east, south-east, south-west, north-west, then round again one ring
in (v. 3). The 28 stars from Krittika fill the outer ring, seven a side
from the east (v. 4). The next ring takes twenty letters, a va ka ha ḍa
in the east, ma ṭa pa ra ta in the south, na ya bha ja kha in the west and
ga sa da ca la in the north (v. 5). The next takes the twelve signs from
Taurus (v. 6). The last five take the tithi groups, Nanda in the east,
Bhadra, Jaya and Rikta round, Purna at the centre (the tika on v. 7), and
the weekdays with them: Tuesday and Sunday in Nanda, Monday and
Wednesday in Bhadra, Thursday in Jaya, Friday in Rikta, Saturday in Purna
(vv. 8, 9). Every ring fills clockwise from the north-east corner, so the
grid is **one rule**: each ring's corners take the next four vowels and
its sides the next members of one sequence. The kernel encodes the rule;
a test holds it to the printed chakra (p. 58, leaf n67) cell for cell.

```
 अ  कृ  रो  मृ  आ  पु  पु  श्ले  आ        east at the top,
 भ  उ   अ   व  क   ह   ड   ऊ   म        north at the left,
 अ  ल   ऌ   वृ  मि  क   ॡ   म   पू       as the print draws it
 रे  च   मे   ओ  नं  औ   सिं  ट   उ
 उ  द   मी  रि  पू  भ   क   प   ह        नं भ ज रि पू: the tithi
 पू  स   कुं  अः  ज  अं   तु   र   चि       groups Nanda to Purna
 श  ग   ऐ   म  ध   वृ   ए   त   स्वा
 ध  ॠ   ख   ज  भ   य   न   ऋ   वि
 ई  श्र   अ   उ  पू   मू   ज्ये  अ   इ
```

**The vedha** (vv. 12 to 30).

- Saturn, the Sun, Rahu, Ketu and Mars are malefic, the rest benefic;
  Mercury joined with a malefic is malefic, and so is the waning Moon (v.
  12). Benefics joined with malefics give a malefic's results (v. 30).
- From the star it stands in a graha strikes along three lines, by its
  sight: left, front and right (v. 13).
- The five from Mars: the retrograde look right, the fast left, the mean
  ahead (v. 15); vv. 16 to 18 read the motion off the Sun's sign counted
  from the graha's, "without computing the grahas". Rahu and Ketu are
  always retrograde, the Sun and the Moon always fast, and "because their
  motion is of one nature, these always have the three sights" (v. 19).
- The sixteen vowels pair (v. 21), and b with v, ś with s, ṣ with kh, ṅ
  with ñ (v. 22): one struck, both are. The letters the grid lacks are
  struck with ka, pa, bha and da (v. 23), and by a graha standing in Ardra,
  Hasta, Purvashadha and Uttarabhadrapada, the stars whose padas they are
  (v. 24).
- A graha in the last pada of the star before a corner or the first of
  the one after it strikes the corner's vowels and Purna (v. 26).
- From one to five malefics striking: agitation, fear, loss, disease,
  death (v. 27). Struck, the star brings confusion, the letter loss, the
  vowel sickness, the tithi fear, the sign a great obstacle; struck in all
  five, the native does not live (v. 28). The birth weekday struck by a
  malefic takes away peace of mind (*Svara Chintamani*, quoted p. 307).

### 8.2 What was checked

The rule against the print, every cell. Three worked examples, a graha in
Krittika, Rohini and Mrigashira and the nine cells each strikes
(*Jatakabharana* vv. 4 to 6; Rajavijaya, p. 306; Ranahasti in the tika on
vv. 15 to 18), reproduced cell for cell by the three lines, the front
striking only the star at its far end. On a sky that can happen: Mercury
within 28° of the Sun and Venus within 47° are never retrograde by vv. 16
to 18 (Mercury always fast or unseen, Venus at most mean), though both go
retrograde in the sky; the nodes, always opposite, never share a star and
always look along all three lines.

No reference to measure against: neither the corpus nor PyJHora records
a Sarvatobhadra vedha, so the acceptance values are the printed grid and
the three examples.

### 8.3 The forks

| crux | question | default | the rival, kept as a knob |
|---|---|---|---|
| C428 | which text, and where Abhijit stands | *Svarodaya*'s chakra, the one Phaladeepika v. 48 names; a graha over 276°40′ to 280°53′20″ stands in Abhijit (C292's span) | none: vv. 26 to 28 are another chakra, recorded and not built |
| C429 | what the front strikes; which diagonal is "left" | the star at its far end only, as all three worked examples list it; left and right as the graha sees them facing the centre, so a retrograde graha looks back to the stars behind it on every side | `FrontVedha::EveryCell`, v. 13 alone |
| C430 | which lines a graha looks along, and its motion | the five by motion, one line; the Sun, the Moon and the nodes all three (v. 19 read whole); the caller gives the five's motion, `Motion::by_the_sun` gives vv. 16 to 18's | `Sight::OneEach` (the tika's "some": Sun and Moon left, nodes right), `Sight::AllThree` |
| C431 | the native's points and letters | star, letter, vowel, tithi and sign (v. 28) and the birth weekday; v. 22's four pairs, v. 23's pillars and v. 24's stars all strike; a name beginning with a vowel at the letter a; the corner's four vowels | `CornerVedha::OuterVowel` (the tika's other reading) |
| C432 | malefic and benefic | v. 12 and v. 30 over the sky: a benefic in a malefic's sign is malefic, the Moon waning past 180° of elongation | none |

### 8.4 The design

`sarvatobhadra(transits, motions, native, rules) -> Result<SarvatobhadraReading, Error>`
in `teistro-gochar`, beside `gochar()` and leaving its verdicts as they
were. `transits` are the nine's places as `gochar()` takes them;
`motions` the five's from Mars; `native` the star, sign, tithi, weekday
and, when given, the name's first letter and vowel (a letter the chakra
does not read is refused as `native.name.letter`). Each graha's
`GrahaVedha` says where it stands (`StarPlace`: the chakra's star and the
27's star and pada), its motion, the lines it looks along, its nature and
why (`waning`, `joined_by`), every cell it strikes and how (`Reach`: a
line, a corner, or standing in a letter's star), and the native's points
among them. Each `PointVedha` names the malefics and the benefics that
strike the point; `malefic_points` counts v. 28's five a malefic strikes.
**It reports clauses and scores nothing**: v. 27's "one to five" and v.
28's "all five" are read off the lists, not folded into a number.

`chakra()` is the grid, `sight(star, line)` a line's cells, `varas(class)`
a tithi cell's weekdays.

### 8.5 The façade

Built 2026-10-18. `sdk.chart().sarvatobhadra(&natal, &request)` beside
`gochar`, one reading an instant, every transit chart founded at the
natal place in one batch. The native's star and sign are the natal
Moon's; the tithi and weekday the birth's, from the document's panchanga,
which the rules read the same limbs from (`tithi_at`, `vara`), so a
document without it is refused on `panchanga`; the name's letter and
vowel are the request's (`SarvatobhadraRequest::named`). The three knobs
are the request's (`with_rules`), not the settings': they shape this one
reading, and the kernel's `SarvatobhadraRules` already reads and writes
them, so no profile or generated settings page moves.

**The five's motion** (C430), read on the page image (p. 57): the tika
introduces vv. 16–18 as the motion "without computing the grahas"
(ग्रहगणितं विना), and defines the slow as "less than the even and the
mean". So the default, `SarvatobhadraMotion::Sky`, computes it: each
transit chart's speed against the graha's mean daily motion
(`Motion::by_speed`), retrograde below zero, fast above the mean, mean
otherwise. The mean is the *Surya Siddhanta*'s, its revolutions over an
age's civil days (`teistro_siddhanta::Parameters::TEXT`, I.29 to 37), and
Mercury's and Venus's the Sun's, as I.29 makes them. `Sun` is the verses'
rule (`Motion::by_the_sun`); a graha in the Sun's own sign is unseen (v.
18) and the verses give it no motion, so it takes the sky's, which is
where an inferior planet's retrograde falls. v. 20's "fast" says the fast
keep their own nature; it is not read as making the unseen fast. `Given`
is the caller's five.

The tests (`crates/sdk/tests/sarvatobhadra.rs`) read every answer back
elsewhere, over every fourth day of 2025 and 2026: the native's points
from the natal Moon and the birth's almanac; each reading equal to the
kernel's over a chart founded at its instant; the sky's motion equal to
`by_speed` on each chart's speed, with all five retrograde somewhere and
Mercury and Venus seen in all three motions; the Sun's equal to the
verses wherever they speak, Mercury and Venus never retrograde there, and
the sky's wherever they do not.

**The record.** A chart request's `sarvatobhadra` record
(`SarvatobhadraRequest::from_json`: `instants`, and optionally `name`,
`motion`, `rules`) reads it for every chart in a batch through
`ChartArea::sarvatobhadra_many`, which founds the transit charts once
for every chart at one place and of one kind; the record widens the
request with the panchanga it reads. Section 135, `sarvatobhadra`,
carries the readings across the C boundary as canonical JSON, a list of
readings a chart (`tests/abi.rs` holds it to the façade's own).

### 8.6 Left unbuilt

The saptashalaka and the taras (Phaladeepika vv. 26 to 28); v. 20's
retrograde malefic "very cruel" and benefic "very good", vv. 31 to 38's
effects by graha, strength by sign and the doubling for a retrograde or
exalted striker (readings, the corpus's when vetted); vv. 10 and 11's
upward, sideways, level and downward sights; the directions
"set" while the Sun crosses their signs (the extracts as Sastri renders them, p. 315); and the chakra's use in
muhurta (vv. 39 to 42).

# Gochar: the transits read from the natal Moon

Status: `draft`, 2026-09-26; §6 step 1 **built** the same day. Written
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
exists, and not this module's. vv. 26 to 28 (the 28-star Sarvatobhadra
vedha and the tara transits) are a second layer, after this one.

## 2. What was measured

No source to measure against: the corpus records no transit, and PyJHora
4.8.7 has no gochar or vedha function (a signature search finds none). So
the acceptance values are the table above, asserted cell by cell, and the
pass that will hold the module measures it over the recorded births
rather than against a recording, as the annual chart's did, saying so on
its first line.

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
2. The measured pass over the recorded births, held by a gate.
3. The boundary and the bindings, after the pattern of the Jaimini reading.
4. Sade Sati (its phases need a source: Phaladeepika v. 22 reads Saturn
   over the Janma rasi and does not name the seven and a half years), the
   hit list over the crossing search, and the Ashtakavarga's transit
   scoring.

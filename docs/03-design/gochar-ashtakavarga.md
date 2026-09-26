# Gochar by the Ashtakavarga: the bindus a transit stands on

Status: `draft`, 2026-09-27; §6 steps 1 and 2 **built** the same day.
Written from the text before any code; the building is expected to
correct it.

Derives from `gochar.md` §6 step 4 and the research page's P0 row
"Ashtakavarga transit scoring with kakshya"
(`01-research/feature-universe/11-transits-gochar.md`). Gochar judges a
transit by its house from the natal Moon and the vedha; the Ashtakavarga
judges the same transit by the **bindus** the natal chart put in the sign
it crosses. The two are separate readings of one snapshot, reported side
by side and never folded into one score.

## 1. What the text says

Phaladeepika ch. 23 (V. Subrahmanya Sastri's 1950 edition, pp. 262 to
265), read in the Sanskrit on the printed page; the archive's OCR turns
the verse numbers into 70 and 77, so every number here is the page's.

- **v. 11**, कृत्वाष्टवर्गं द्युसदां … क्रमशः फलानि: with each graha's
  Ashtakavarga prepared, a graha transiting a sign with **no** bindu in its
  own brings death (मृति); one to eight bring, in order, loss (नाश),
  expense (व्यय), dread (भीति), fear (भय), wealth (अर्थ), a bride (नारी),
  prosperity (श्री) and sovereignty (राज्यसिद्धि). The translation renders
  अर्थ as "accomplishment of the desired object". **Four is fear**: by the
  verse a transit is good from five bindus.
- **v. 14**: a transit through the house holding the most bindus advances
  that bhava; through one with few or none, the reverse.
- **vv. 16 to 19, the kakshya.** To time a bindu's fruit a sign is divided
  into **eight equal parts**, 3°45′ each, lorded in the order of the orbits
  (कक्षाक्रमेण): **Saturn, Jupiter, Mars, the Sun, Venus, Mercury, the Moon
  and the lagna** (v. 19: शनिर्द्वितीये तु गुरुः … अन्त्यभागकाले विलग्नं; v.
  18 lists the same eight from the south row up, the lagna first). A bindu
  bears fruit while the graha transits the part whose lord gave it. v. 16
  introduces the division as what **others** say (फलमाहुरन्ये), so the text
  reports it rather than adopting it; it is built because v. 19 then states
  it without qualification.
- **v. 20**, सर्वग्रहाणां … अष्टाक्षसंख्याधिकबिन्दवश्चेच्छुभं तदूने व्यसनं:
  when a sign's sarvashtakavarga exceeds **28** (अष्टाक्ष, eight and two read
  right to left), transits over it are auspicious; below it, distress.

## 2. What can be measured

Nothing records a transit's bindus: the corpus has no transit and PyJHora
4.8.7 has no kakshya or transit scoring (a signature search finds
`get_ashtaka_varga` and the Ashtakavarga dasha, nothing else). But
`get_ashtaka_varga` answers the **prastara** — for each graha and sign,
which of the eight contributors gave a bindu — and the kakshya reads
exactly that table. So the table the kakshya stands on is measured cell by
cell against PyJHora as a black box over drawn charts (rank 3), beside the
existing bindu totals the corpus already holds; the rule on top is the
text's and is asserted verse by verse.

**Measured 2026-09-27**, 3 000 drawn charts, 2 016 000 cells: the SDK's
prastara and PyJHora's agree on every cell but three a chart, each in
every chart — so the contributor each bindu is credited to is confirmed,
and the three are table readings, read on Phaladeepika's printed pages
(ch. 23 vv. 4 and 8, whose places are katapayadi syllables):

| cell | the SDK (BPHS ch. 66, as the corpus records) | Phaladeepika | PyJHora |
|---|---|---|---|
| the Moon's, from the Moon, the 9th | no | no | **yes** |
| the Moon's, from Mars, the 9th | yes | yes | **no** |
| the Moon's, from Jupiter | the 12th | **the 2nd** (कौरवसज्जनस्य: 1, 2, 4, 7, 8, 10, 11; the footnote gives Varahamihira's 12th) | the 2nd |
| Venus's, from Mars | the 5th | the 5th (लोमस्ताळिपरे: 3, 5, 6, 9, 11, 12) | **the 4th**, the footnote's rival reading |

So the SDK and the text this page reads part on one cell (C144), and
PyJHora follows a reading of its own on two.

## 3. The forks (cruxes)

| crux | question | readings | default | why |
|---|---|---|---|---|
| C141 | how many bindus make a transit good | five (v. 11: four is भय, fear); four (much modern practice reads four as middling or good) | **five**, `gochar.ashtakavarga_good_from = FOUR` for the other | the verse names four a fear |
| C142 | a sign whose sarvashtakavarga is exactly 28 | good; bad; neither | **neither**, reported as `EVEN` | v. 20 says more than 28 and less than it, and nothing of 28; a third state says so rather than choosing |
| C144 | the Moon's bindu from Jupiter | the 12th (BPHS ch. 66, Varahamihira; the corpus's engine); the 2nd (Phaladeepika ch. 23 v. 4) | **the 12th**, the table the natal Ashtakavarga already answers with and the corpus holds | one chart has one Ashtakavarga; a reading of the text's own table is a knob to add, not a second table beside the first |
| C143 | raw bindus or reduced | the prepared Ashtakavarga (v. 11's कृत्वाष्टवर्गं); after the trine and Ekadhipatya reductions | **raw** | v. 11 reads the prepared table; the reductions are for the pindas (ch. 23 vv. 3 to 9) |

## 4. The design

- **`teistro-strength`** gains `prastara(chart) -> [[u8; 12]; 7]`: for each
  graha, Sun to Saturn, and each sign, a bit set of the contributors that
  gave it a bindu, bit `n` the graha with id `n` and bit 7 the lagna.
  `bindus` becomes its population count, so the two cannot disagree.
- **`teistro-gochar`** gains `ashtakavarga`, depending on nothing new: it
  takes the natal prastara as data. `Kakshya::at(degrees)` answers the part
  (1 to 8) and its `KakshyaLord`. `transits(prastara, transits, rules)`
  answers, for each of the seven: its bindus in the sign it transits,
  whether they reach `good_from` (C141), its kakshya and whether that
  kakshya's lord gave a bindu there (v. 16), the sign's sarvashtakavarga
  and where it stands against 28 (`ABOVE`, `EVEN`, `BELOW`; C142). The
  nodes have no Ashtakavarga and are not in it.
- **The façade**: `GocharRequest::with_ashtakavarga()` asks for it; each
  `GocharReading` then carries it, the natal prastara computed once for
  the batch. A reading not asked for carries none, so the default request
  costs nothing new.

## 5. Tests

- `prastara` against `bindus` over every lagna and placement the crate's
  tests draw, and against PyJHora's prastara cell by cell over drawn
  charts (a generated page, held by a gate).
- The kakshya's eight parts at their edges, and each lord in v. 19's order.
- v. 11's threshold under both readings of C141; v. 20's three states,
  28 itself `EVEN`.
- Through the façade: the reading's bindus are the natal Ashtakavarga's
  for the sign transited, and a request that did not ask carries none.

## 6. Order of work

1. `prastara`, with `bindus` derived from it, and its measurement against
   PyJHora: **done**, §2 records the three cells where the tables part.
   The measurement is a one-off black-box run recorded here rather than a
   gate, since the SDK's bindus are already held to the corpus and what
   was new — which contributor each is credited to — agreed everywhere
   the texts agree.
2. The crate module, the knob and the façade, with the tests above:
   **done**. `GocharRequest::with_ashtakavarga()` (JSON `ashtakavarga:
   true`) fills each reading's `ashtakavarga`, the natal prastara
   computed once for the batch and the transits placed once for both
   readings; the façade's Ashtakavarga shares the sign-gathering with it
   (`ashtakavarga_chart_of`) rather than keeping a second copy.
3. The boundary and the bindings: **done**. `gochar_json.ashtakavarga`
   fills section 55, `gochar_ashtakavarga`, seven rows under each `gochar`
   row or none, and each binding refuses any other count rather than
   misreading it; `gochar` gains `ashtakavarga_good_from` beside the node
   readings. Building it merged the threshold into `GocharRules`, so one
   rules record mirrors the settings' `gochar` group and every reading
   says all three of its readings, rather than a second record the
   encoder could not see.

# Varga schemes and karaka namings a consumer chooses (SR-114, SR-115)

Status: built. Crates: `core` (`varga_rows`, the knobs), `vargas`
(`Scheme::named`, `Scheme::from_rows`, `Scheme::chosen`), `rules`
(`SevenKarakas`, `RuleChart::holder`), `sdk` (every reader of a chart
under a context). Cruxes C2060 to C2069.

## What a consumer asked for

- **SR-114.** The alternative varga schemes by name, one chosen per varga in
  the settings, and a consumer's own rows accepted beside them, checked
  like the shipped ones, with a refusal that names the field.
- **SR-115.** The sthira karakas and the named chara karaka schemes beside
  `chara_karakas` and `eight_karaka_order`.

## One evaluator, more rows (crux C2060)

`varga-kernel.md` derived one evaluator, `sign = (a·rashi + step·part +
offset) mod 12` over a span rule, or a listed table where no line fits. A
variant scheme is **rows for that evaluator, never a new code path**:

- A shipped variant is a `Scheme` constant in `VARIANTS` beside the
  twenty-one of `SCHEMES`, carrying its `VargaVariant` name.
- A consumer's rows are `varga_rows::Rows`: a classifier and one group per
  class, each a span rule (`EQUAL` or `DEGREES` widths) and a map (`STEP`
  with base, step and offset, or `LISTED` signs). They are the same rows a
  shipped scheme writes (`Scheme::rows`), so a consumer can start from a
  shipped scheme, change a row and hand it back.
- The classifiers are the four the twenty-one need (`ALL`, `PARITY`,
  `MODALITY`, `ELEMENT`) and `SIGN`, twelve groups, one per sign: what a
  table that follows no rule (Kashinatha's hora, Somanatha's drekkana)
  needs. It is the general case of the other four.

A consumer's rows are checked before a chart is cast (`Rows::check`), and
every fault names its field: `vargas.schemes.D9.groups` when the count of
groups is not the classifier's, `vargas.schemes.D30.groups[0].spans.widths`
when widths do not fill thirty degrees, `…map.step` or `…map.offset` past
11, `…map.signs` of the wrong length and `…map.signs[3]` past 11. A
named variant chosen for a chart it does not serve is refused at
`vargas.schemes.<V>.which`, listing the variants that serve it; D1 is
refused anything but `PARASHARA`. The settings' coherence check runs the
same checks, so a context is never built over rows a chart would refuse
(rule `varga-scheme`).

Checked rows are interned (leaked once, deduplicated by content), so
`Scheme` stays `Copy` and a chart reader holds no allocation. The table is
capped at 1,024 distinct row sets (`MOST_RULES`); past it a new set is
refused with `LIMIT`. A process that builds contexts from untrusted rows
in a loop meets the cap rather than growing without bound.

## The variants shipped, by source (cruxes C2063 to C2066)

| varga | variant | rows | source | rank |
|---|---|---|---|---|
| D2 | `PARIVRITTI` | cyclic, step 1 | P.V.R. Narasimha Rao, *Personal Finances from Hora Chart* (2003), parivritti-dwaya hora: Ar → Ar, Ta; Ta → Ge, Cn | 2 |
| D2 | `KASHINATHA` | `SIGN`, listed | the same handout, Kashinatha's hora: each half to a sign the lord owns, by day- and night-strength; worked: Jupiter 25°16′ Ta → Li, Mars 18°53′ Sg → Sg | 2 |
| D2 | `UMA_SHAMBHU` | `PARITY`, cyclic forward / backward | Narasimha Rao, *Parasara's Hora Chart Decoded: Uma-Shambhu Hora* (2012): Ar → Ar, Ta; Ta → Cn, Ge; Cn's second half → Li | 2 |
| D3 | `PARIVRITTI` | cyclic, step 1 | Iranganti Rangacharya, after the Vriddha Karika (Saptarishis Astrology, 2023): the parivritti-traya table | 2 |
| D3 | `SOMANATHA` | `SIGN`, fixed step ±1 | the same article, Somanatha's Kalpalata on the Phalaratnamala: Ar → Ar, Ta, Ge; Ta → Pi, Aq, Cp; Cn → Sg, Sc, Li | 2 |
| D4 | `PARIVRITTI` | cyclic, step 1 | the 2003 handout's Table 1 | 2 |
| D5 | `PARIVRITTI` | cyclic, step 1 | the 2003 handout's Table 1 | 2 |
| D9 | `KRISHNA_MISHRA` | `PARITY`, cyclic forward / backward | the 2012 paper: Ta → Vi, Le, Cn, Ge, Ta, Ar, Pi, Aq, Cp | 2 |

Every acceptance value above is a test (`crates/vargas/tests/variants.rs`),
taken from the source's own worked example or table. Not shipped, for want
of a source read: a Jagannatha hora or drekkana, a Raman hora as distinct
from Parashara's, and any D10, D8, D11 or D30 variant (JHora lists variants
of these; its manual does not define them). A consumer who has a source
writes the rows (crux C2068).

## One reader: `Scheme::chosen`

`vargas.schemes` is a map from a varga to a choice; a varga left out is
BPHS ch. 6's. **Every chart read under a context asks `Scheme::chosen`**:
a document's varga sections, the navamsa lagna, a drawing
(`geometry::draw_chosen`), a change hit's varga, the rules' navamsha and
`rule_vargas`, the Jaimini karakamsha and the ishta devata, and the
prashna sky's navamsha and shadvarga. Three readers keep their text's own
scheme and ask `Scheme::of` (crux C2067): Tajika's pancha-vargiya bala,
which Neelakantha defines on the Parashari divisions; the rectification
purifier and sensitivity, which measure distance to a division's edges;
and the corpus tools in `xtask`.

A scheme writes its `variant` into a document only when it is not
`PARASHARA`, so a document cast under the shipped rows keeps its bytes and
its hash. A stored document read back under other settings keeps the
scheme it was cast with: the variant and the rows are in it.

## The chara karakas

Already present: the seven and eight karakas (`chara_karakas`), the eight's
naming (`eight_karaka_order`, C101), and the **sthira karakas** with their
karaka bhavas (BPHS ch. 32 vv. 18 to 24, `JaiminiReading.sthira_karakas`,
`jaimini-significators.md`). New:

- **`jaimini.seven_karaka_order`** (crux C2061). `JAIMINI`, the shipped
  naming and the recording engine's: the Jaimini Sutras 1.1.13 to 18 name
  seven, Atma, Amatya, Bhratri, Matri, Putra, Gnati, Dara, with no
  Pitrikaraka. `MATRI_IS_PUTRA`: BPHS ch. 32 v. 16, "others say the
  Matrikaraka is the Putrakaraka" (also Jaimini 1.1.19), read as the fifth
  rank being the Pitrikaraka and the children read from the Matrikaraka's
  graha. Under it `RuleChart::holder(PUTRAKARAKA)` answers the
  Matrikaraka's graha, so every rule and remedy that asks for the
  Putrakaraka reads it.
- **`jaimini.chara_karakas = EIGHT_ON_A_TIE`** (crux C2062): BPHS ch. 32
  v. 2 counts eight when two of the seven stand equally far into their
  signs. Equal is to the arc-second, the resolution the karakas are ranked
  at (`RuleChart::seven_tie`). On a sky with no tie it is `SEVEN`.

Not shipped: Brihat Jataka 4.5's day and night parent karakas (the Sun
and Venus for the father and mother by day, Saturn and the Moon by
night), whose reading of `tadviparyayāt` the translators dispute (crux
C2069, open).

## Bindings

The settings are one JSON patch on every binding, so a scheme reaches C,
Node, wasm, Python, Dart, Java and MCP with no API of its own: each
binding's test patches `vargas.schemes`, reads a D2 under Kashinatha
against its table and has a broken row refused with its field in the
message. The settings schema, which every binding's types are generated
from, carries the new knobs with their descriptions.

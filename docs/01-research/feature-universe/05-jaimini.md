# Jaimini

Status: `research`, 2026-09-04. Checked against the baseline engine's Jaimini slice
(chara karakas with 7 and 8 schemes, arudhas, three pairs, Chara dasha,
Narayana dasha), JHora, PyJHora and Kala.

| feature | inputs | variants | baseline | field | tier |
|---|---|---|---|---|---|
| chara karakas (Atma, Amatya, Bhratru, Matru, Putra, Gnati, Dara; Pitru in the 8-karaka scheme) | degrees within sign of 7 or 8 bodies | 7-karaka (Rahu excluded) and 8-karaka (Rahu by 30° minus degree); tie rules; PyJHora lists four methods | yes (7, 8) | all | P0 |
| sthira karakas and naisargika karakas | | | partial | JHora | P0 |
| arudha padas A1–A12, Upapada (A12), Darapada (A7), Arudha Lagna (A1) | lord positions | exception rule variants; co-lord rules | yes | all | P0 |
| graha arudhas | | | no | JHora | P1 |
| rashi drishti | signs | | yes | all | P0 |
| argala and virodha argala with strength comparison | positions | | partial | JHora, Kala, PyJHora | P1 |
| karakamsa (navamsa of the Atmakaraka) and swamsa analysis | D9 | | partial | all | P0 |
| Varnada lagna and Varnada dasha | lagna and hora lagna | three methods | partial | JHora, PyJHora | P1 |
| Sree lagna and Sudasa | | | no | JHora, PyJHora | P1 |
| Brahma, Rudra and Maheshwara grahas | | | no | JHora | P1 |
| Chara dasha (Parashara, K.N. Rao, Raghava Bhatta, Rangacharya variants) | | | yes (one variant) | all | P0 |
| Narayana dasha of any varga | | | yes | JHora | P0 |
| Sthira, Shoola, Niryana Shoola, Brahma, Mandooka, Navamsa, Trikona, Yogardha, Paryaya dashas | | | Sthira and Shoola yes | JHora, PyJHora | P0 for two, P1 for the rest |
| the three pairs of longevity (lagna lord and 8th lord, lagna and Moon, lagna and hora lagna) with movable, fixed, dual combinations | | | yes | Kala, JHora | P0 |
| Jaimini yogas (Raja yogas from karakas, arudha relationships) | | | partial | Kala, JHora | P1 |
| Jaimini aspects on arudhas for marriage (Upapada analysis), Darapada, A10 for career | | | partial | Kala | P1 |
| Kala's "Jaimini screen" and JHora's argala highlighting as presentation | | | no | Kala, JHora | P1 |

## Closing checklist

- Choose the default Chara dasha variant per profile and record the
  differences on five charts.
- Confirm the arudha exception rule variants and the co-lord strength rules
  (Sanjay Rath's eight rules) with citations.
- Argala strength comparison rules need a reference implementation to test
  against.

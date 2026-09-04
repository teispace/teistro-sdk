# Chakras, special charts and mundane charts

Status: `research`, 2026-09-04. Checked against JHora (eight chakras, mundane
charts, dasha pravesh charts), PyJHora (13 chakras) and the baseline engine (Sudarshana
chakra endpoint, tara chakra, ghata chakra).

## Chakras (diagram-based techniques)

A chakra is a placement of bodies (natal and transit) onto a fixed diagram
keyed by nakshatra, sign or degree, plus rules about which cells are sensitive.
For the SDK a chakra is a data description (cells, mapping, sensitive cells,
rules) rendered by the consumer; the engine outputs cell occupancy and rule
results.

| chakra | key | inputs | baseline | field | tier |
|---|---|---|---|---|---|
| Sudarshana chakra (lagna, Moon, Sun rings) with its dasha | houses | natal | yes | JHora, Kala, PL | P0 |
| Sarvatobhadra chakra (nakshatra, tithi, vara, akshara cells; vedha lines) | 9×9 grid | natal and transit | no | JHora, PyJHora | P1 |
| Kota chakra (Durga chakra) with stambha, madhya, prakara, bahya and the Kota pala and swami | nakshatra from natal Moon | transit | no | JHora, PyJHora | P1 |
| Kalachakra (of the dasha and of the day) | | | dasha yes | JHora | P1 |
| Surya and Chandra Kalanala chakras | | | no | JHora, PyJHora | P2 |
| Shoola chakra, Tripataki chakra (Moon transit from the lagna with drishti lines) | | | no | JHora, PyJHora | P2 |
| Sapta Shalaka, Pancha Shalaka, Sapta Nadi chakras (rain and weather prashna) | | | no | PyJHora | P2 |
| Ghata chakra (inauspicious month, tithi, vara, nakshatra, lagna per Moon sign) | Moon sign table | | yes | many | P0 |
| Tara chakra (navatara) | | | yes | many | P0 |
| Chandra kundali, Surya kundali, Bhava chalit chart, Karakamsa chart, Arudha chart, Drekkana charts (Ayudha, Sarpa, Pakshi) | natal | | partial (Moon chart, chalit) | all | P0 for the first three |

## Mundane and event charts (JHora)

| chart | inputs | tier |
|---|---|---|
| Aries and Capricorn ingress (solar new year), solar new month charts | crossing search | P1 |
| full Moon and new Moon charts (annual, monthly), lunar new year and month | crossing search | P1 |
| planetary conjunction and opposition charts | crossing search | P1 |
| rasi, nakshatra, navamsa and varga change charts | crossing search | P1 |
| solar and lunar eclipse charts | eclipse search | P1 |
| dasha pravesh charts (chart at a period start) | dasha tree | P1 |
| financial new year charts | | P2 |
| swearing-in charts with dasha compression | | P2 |

All of these are one abstraction: a chart cast at an instant found by a
search. The SDK provides the search results and the chart caster; the chart
kinds are a catalogue.

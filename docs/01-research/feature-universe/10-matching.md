# Compatibility and matching

Status: `research`, 2026-09-04. Checked against the baseline engine's milan package (Ashta
Koot with cancellations, extended checks, Mangal matching, marriage doshas,
quick milan from name or nakshatra, synastry and composites in research),
PyJHora (Ashta Koota and South Indian 10 porutham), Astro-Vision (Gun Milan,
Kerala and Tamil methods), Kala's compatibility module and Solar Fire's
synastry and composite tools.

| feature | inputs | variants | baseline | field | tier |
|---|---|---|---|---|---|
| Ashta Koota (Varna, Vashya, Tara, Yoni, Graha Maitri, Gana, Bhakoot, Nadi) out of 36 | both Moons | scoring matrices differ by source (Vashya asymmetric 5-group matrix, Gana table variants, Tara both-directions); Bhakoot and Nadi cancellations (exceptions lists differ) | yes | all Vedic | P0 |
| Dasha Koota (South Indian 10 porutham: Dina, Gana, Mahendra, Stree Deergha, Yoni, Rasi, Rasi Adhipati, Vasya, Rajju, Vedha) | nakshatras and signs | Tamil and Kerala practice differ on Rajju sub-types and on which are mandatory | extended checks yes (Rajju, Vedha, Mahendra, Stree Deergha) | PyJHora, Astro-Vision | P0 |
| Mangal (Kuja) dosha matching with severity and cancellations | both charts | | yes | all | P0 |
| marriage doshas as a unified list with severity (Nadi, Bhakoot, Gana, Vedha, Rajju by body part, absence checks) | | | yes | | P0 |
| recommendation bands | totals | | yes | all | P0 |
| quick match from name or nakshatra and pada only (naam milan) | akshar resolver | | yes | many consumer apps | P0 |
| Dashamsha and D9 comparison, longevity comparison, dasha overlap (marriage timing for both) | both charts | | partial (analysis endpoint) | Kala | P1 |
| Papasamya (malefic balance around the 7th house and Venus) | | Kerala practice | no | Astro-Vision | P1 |
| Western synastry: inter-aspects with orbs, house overlays, scores | tropical or sidereal | | yes (research module) | Solar Fire, Maitreya | P1 (`western`) |
| composite (midpoint) and Davison relationship charts; multi-person composites | | | yes (midpoint composite, Davison reference) | Solar Fire (up to 15 people) | P1 |
| interpretation of kootas, matched values, verdict prose | locale packs | | yes | all | P0 (`interpret`) |

## Closing checklist

- Both Ashta Koota and Dasha Koota scoring tables must be data with
  citations so regional variants (Nepali practice versus Tamil) are profiles.
- Confirm the Rajju sub-types (Pada, Kati, Udara, Kantha, Shira) and the
  Vedha pairs list.

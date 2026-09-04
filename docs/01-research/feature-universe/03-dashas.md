# Dasha systems (time lords)

Status: `research`, 2026-09-04. Checked against the baseline engine's `dasha-pipeline`
doc and 18-system registry, the JHora features page (around 45 named
systems) and PyJHora's list (60+). The SDK's dasha registry is designed to
hold all of them as plug-ins over four seed kinds.

## The four seed kinds

Every system in the field is seeded by one of four things, and the registry
models the seed explicitly so that a new system is a new record, not a new
engine.

| seed | how the first period is found | balance at birth | examples |
|---|---|---|---|
| nakshatra of a body (usually the Moon) | lord sequence indexed by nakshatra; elapsed fraction of the nakshatra gives the balance | spatial (fraction of arc) or temporal (fraction of the nakshatra's time span, the baseline engine's default) | Vimshottari, Ashtottari, Yogini, Shodashottari, Dwadashottari, Panchottari, Shatabdika, Chaturashiti-sama, Dwisaptati-sama, Shashtihayani, Shat-trimsha-sama, Kalachakra |
| a rashi (lagna, karaka, arudha, Sree lagna, or a varga lagna) | sign sequences with direction rules and period lengths from lord placement | none, or fractional by degree in some | Chara, Narayana, Sudasa, Drig, Shoola, Sthira, Brahma, Mandooka, Navamsa, Lagnamsaka, Padanathamsa, Paryaya, Trikona, Yogardha, Varnada, Kendradi rashi |
| a body order (graha) | fixed or strength-ordered planet sequence | | Naisargika, Kendradi graha, Atmakaraka kendradi graha, Karaka dasha, Aayu, Kaala |
| an event or a chart | annual or monthly chart, tithi, yoga, chakra position | | Mudda, Patyayini, Varsha Vimshottari, Varsha Narayana, Tithi Ashtottari, Tithi Yogini, Sudarshana Chakra, Panchasvara |

## Catalogue

Tier: P0 = the baseline engine has it (18), P1 = in JHora or PyJHora, P2 = rare or
experimental. "Cond." marks conditional systems whose applicability is a
chart rule.

| system | seed | levels | baseline | cond. | notes and variants | tier |
|---|---|---|---|---|---|---|
| Vimshottari | Moon nakshatra, 120 years | up to 5 (deha) | yes | no | seed from lagna, Sun, Gulika, Maandi, kshema/utpanna/adhana taras (variants); tribhagi (1/3 lengths); rashi-bhukta variant; year length 365.25 or 360 or sidereal; footedness triads for AD ordering (BPHS 46.156) | P0 |
| Ashtottari | Moon nakshatra, 108 years | 4 | yes | yes (Rahu in kendra or trikona from lagna lord, day/night birth variants) | 8 lords; starting nakshatra Ardra; tithi variant | P0 |
| Yogini | Moon nakshatra, 36 years | 4 | yes | no | 8 yoginis; tithi variant | P0 |
| Kalachakra | Moon nakshatra pada | 3 | yes | no | savya and apasavya groups, deha and jeeva rashis, 100 or 144 year cycle, Simha–Karka jumps and Vrischika–Meena jumps (variants in JHora and PyJHora) | P0 |
| Shodashottari | 116 years | 3 | yes | yes (lagna in Moon's hora, or Moon in Sun's hora, by day/night) | | P0 |
| Dwadashottari | 112 years | 3 | yes | yes (lagna in Venus navamsa) | | P0 |
| Panchottari | 105 years | 3 | yes | yes (lagna in Cancer with Cancer dwadashamsa) | | P0 |
| Shatabdika | 100 years | 3 | yes | yes (lagna vargottama) | | P0 |
| Chaturashiti-sama | 84 years | 3 | yes | yes (10th lord in 10th) | | P0 |
| Dwisaptati-sama | 72 years | 3 | yes | yes (lagna lord in 1st or 7th) | | P0 |
| Shashtihayani | 60 years | 3 | yes | yes (Sun in lagna) | | P0 |
| Shat-trimsha-sama | 36 years | 3 | yes | yes (day birth with lagna in Sun's hora, night with Moon's) | | P0 |
| Chara (Jaimini) | lagna sign | 3 | yes | no | Parashara, K.N. Rao, Raghava Bhatta and Iranganti Rangacharya variants; period length by lord's distance with exaltation and debilitation adjustments; direction by odd/even and Vishama padas **verify** | P0 |
| Narayana | lagna or 7th (stronger) in any varga | 3 | yes | no | all divisional charts; Sanjay Rath rules | P0 |
| Sthira | Brahma graha | 3 | yes | no | fixed lengths 7, 8, 9 by sign type | P0 |
| Shoola | | 3 | yes | no | 12 house variants, Niryana Shoola for longevity | P0 |
| Yogini tithi | | | no | | | P1 |
| Sudarshana Chakra | lagna, Moon, Sun | 3 | yes | no | one sign per year, three reference points | P0 |
| Moola | | | no | | multiple configurations | P1 |
| Tara | Moon nakshatra | | no | | | P1 |
| Sudasa (Rasi dasha from Sree Lagna) | | | no | | | P1 |
| Drig | 9th and 5th house rules | | no | | two methods | P1 |
| Lagna Kendradi rashi, Atmakaraka Kendradi rashi and graha | | | no | | | P1 |
| Trikona | | | no | | | P1 |
| Yogardha | | | no | | | P1 |
| Paryaya (sthira, chara, ubhaya) | | | no | | all vargas | P1 |
| Brahma | Brahma graha | | no | | | P1 |
| Mandooka (Rudramsha) | | | no | | | P1 |
| Navamsa | | | no | | | P1 |
| Lagnamsaka, Padanathamsa | | | no | | three methods for Padanathamsa | P1 |
| Varnada | Varnada lagna | | no | | | P1 |
| Chakra | | | no | | | P2 |
| Sandhya | | | no | | | P2 |
| Tara Lagna | | | no | | | P2 |
| Nirayana Shoola | | | partial | | | P1 |
| Naisargika | fixed ages | | no | | | P1 |
| Kaala, Aayu, Buddhi-gati, Karaka (7 and 8), Rashmi, Saptarishi nakshatra | | | no | | rare | P2 |
| Karana Chaturashiti-sama | | | no | | | P2 |
| Yoga Vimshottari | | | no | | | P2 |
| Patyayini, Mudda | Tajika annual chart | | yes (partial) | | | P0 |
| Varsha Vimshottari, Varsha Narayana | | | no | | | P1 |
| Tithi Ashtottari, Tithi Yogini | tithi | | no | | | P1 |
| Panchasvara | | | no | | | P2 |
| Ashtakavarga dasha | | | no | | experimental | P2 |

Count: 18 P0, about 25 P1, about 12 P2. JHora offers roughly 45; PyJHora 60+
including experimental ones; Parashara's Light around 10; Kala the Parashari
and Jaimini core; Solar Fire only Vimshottari.

## Common machinery the registry must provide

| capability | detail | baseline |
|---|---|---|
| depth control | per call and per system (`dashaMaxLevel`, `dashaDepthBySystem`) | yes |
| balance method | spatial versus temporal, observer-frame invariant | yes |
| year length | 365.25 (default), 360 savana, sidereal 365.2564, lunar, Gregorian | partial |
| sub-period ordering | starts from the major lord (default) or from the next lord (footedness rules for Vimshottari AD; some systems start from the lord's own position) | yes |
| period tree with exact instants | JD start and end per node, walkable to any level | yes |
| active chain at an instant | MD, AD, PD ... at a reference JD | yes |
| dasha pravesh charts | chart cast at a period's start | no (JHora has) | 
| dasha sandhi detection | boundaries within a window | partial |
| compression (mundane, swearing-in) | JHora | no |
| interpretation hooks | lord placement, lord-house relationship, dasha lord effects text | yes (composer) |

## Closing checklist

- Reconcile the baseline engine's 18 implementations against JHora outputs for five
  charts each and record deliberate differences with citations.
- Confirm the conditional-dasha applicability rules per BPHS 46.
- Decide P1 ordering: Chara variants, Narayana over all vargas and Tithi
  dashas first because PL, JHora and PyJHora all have them.
- Define the plug-in interface so a consumer can register a dasha system
  from outside the SDK (a seed kind, a sequence, lengths, ordering rules).

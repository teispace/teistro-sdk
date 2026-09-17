# The yogas, measured

Status: `generated` by `cargo xtask yogas` over the conformance corpus's
`baseline/yogas`, 2026-09-15. Do not edit: `check-yogas` regenerates
this page and fails on any difference. The design it measures is
[`rules-engine.md`](rules-engine.md). The pass evaluates every rule of
`rules.json` over every recorded chart under a reading and counts what
disagrees with what the engine recorded.

## What the corpus holds

605 rules over 93 charts: 55 521 decisions of the 597 rules written in the condition language, 5350 of them present. The other 8 rules carry no conditions because the engine computes them in code: NEECHA_BHANGA_RAJA, NEECHA_BHANGA_DEBIL_LORD_KENDRA, NEECHA_BHANGA_EXALT_PLANET_KENDRA, NEECHA_BHANGA_D9_EXALT, NEECHA_BHANGA_BENEFIC_ASPECT, NEECHA_BHANGA_OWN_NAVAMSA, NEECHA_BHANGA_EXALT_LORD_KENDRA, NEECHA_BHANGA_RETROGRADE. The rules use 22 condition types; by use, `lord-of-house-in-house` 448, `and` 346, `planet-in-house` 261, `planet-dignity` 149, `planet-in-house-from` 104, `or` 91, `planet-in-kendra` 74, `planet-conjunct` 70, `lord-of-house-in-kendra` 54, `mutual-exchange` 54, `chara-karaka-in-house` 34, `lord-conjunct-lord` 31, `planet-in-sign` 28, `all-classical-grahas-in-houses` 22, `no-planet-in-houses-from` 16, `not` 16, `planet-retrograde` 16, `planet-in-trikona` 13, `planet-in-kendra-from` 10, `n-grahas-conjunct-with` 7, `occupied-sign-count` 7, `planet-combust` 6.

## What the corpus decides

| proposed rule | verdict | measured |
|---|---|---|
| the engine's reading, every choice below as the engine makes it | **holds** | 0 of 55521 disagree; 0 decisions wrong, 0 of 5350 presences' planets, 0 cancellations |
| benefics by the natural lists alone, the Moon and Mercury never turned malefic | falsified | 382 of 55521 disagree; 281 decisions wrong, 101 of 5280 presences' planets, 0 cancellations; 46 rules move (ALPAYU_BALARISHTA_TRIPLE, AMALA, AMALA_KIRTI_BENEFIC_10_FROM_MOON, BALARISHTA_LAGNA_7TH_MALEFICS, …) |
| a rule asking for exaltation or debilitation met by that dignity alone, not its deep form | untested | moves none of the 55 521 decisions or 5350 presences |
| all seven between the nodes counted from Rahu to Ketu only | untested | moves none of the 55 521 decisions or 5350 presences |
| houses counted whole-sign from the lagna rather than recorded | untested | moves none of the 55 521 decisions or 5350 presences |
| Rahu and Ketu counted retrograde where a rule asks | untested | moves none of the 55 521 decisions or 5350 presences |
| an involved planet only from the branch that decided | untested | moves none of the 55 521 decisions or 5350 presences |
| an unqualified conjunction within 10° rather than in one sign | falsified | 276 of 55521 disagree; 274 decisions wrong, 0 of 5115 presences' planets, 2 cancellations; 55 rules move (ADITYA_YOGA, BUDHADITYA, CHANDRA_MANGAL, CHANDRA_MANGAL_DHANA, …) |

## Coverage

116 of the 597 rules are never present on any recorded chart, so the corpus holds no positive case for them; 0 are present on every one, so it holds no negative case. By category, rules and presences: arishta 20 (164), ayur 15 (128), bhagya 10 (178), chandra 39 (386), daridra 12 (196), dhana 42 (222), dur 18 (209), jaimini 25 (282), karma 12 (184), mahapurusha 21 (99), miscellaneous 106 (729), nabhasha 34 (143), neecha-bhanga 8 (202), pravrajya 8 (34), raja 146 (965), sahaja 8 (180), santana 12 (217), surya 22 (285), tajika 19 (247), vidya 10 (259), viparita-raja 18 (243).

Never present: SHASHA_EXALTED, RUCHAKA_BHANGA, MALAVYA_BHANGA,
ADHI_YOGA, VASUMATI, KAHALA_MUKHYA, BRAHMA_RAJA, BHERI, PARIJATA,
KALPADRUMA, ADHI_RAJA, VASUMATI_RAJA, PUSHPA_MALA, RAJA_1_5, RAJA_1_9,
RAJA_4_5, RAJA_7_5, RAJA_7_9, RAJA_10_5, RAJA_10_9, ADHI,
SURYA_UCHCHA_10, DHANA_PARIVARTANA_2_11, DHANA_PARIVARTANA_1_2, GOLA,
YUGA, RAJJU, MUSALA, NALA, MAALA, GADA, GADA_4_7, SAKATA, VIHAGA,
SHRINGATAKA, HALA_AKRITI, HALA_AKRITI_3_7_11, HALA_AKRITI_4_8_12, YUPA,
SAKTI, DANDA_AKRITI, KUTA, DAMINI, PADMA, VAPI, MAHA_PARIVARTANA_1_4,
MAHA_PARIVARTANA_1_7, MAHA_PARIVARTANA_1_10, MAHA_PARIVARTANA_4_7,
MAHA_PARIVARTANA_4_10, MAHA_PARIVARTANA_5_7, MAHA_PARIVARTANA_5_10,
MAHA_PARIVARTANA_7_10, MAHA_PARIVARTANA_9_10, DAINYA_PARIVARTANA_6_12,
DAINYA_PARIVARTANA_8_12, BALARISHTA_MOON_LAGNA_MALEFIC,
BALARISHTA_MOON_SATURN_DUSTHANA, BALARISHTA_SUN_MARS_LAGNA,
YOGARISHTA_SUN_SATURN_RAHU, YOGARISHTA_SATURN_LAGNA_DEBIL,
MADHYARISHTA_MOON_HEMMED_4, ANGA_ARISHTA, BANDHA_ARISHTA, JUP_IN_11,
MOON_IN_10, MOON_EXALTED_KENDRA, VIPARITA_6_IN_12, KHALA_1_6, KHALA_1_8,
KHALA_1_12, KHALA_4_8, KHALA_4_12, KHALA_5_6, KHALA_5_8, KHALA_5_12,
KHALA_7_6, KHALA_7_8, KHALA_7_12, KHALA_9_8, KHALA_9_12, KHALA_10_6,
KHALA_10_8, KHALA_10_12, KHALA_11_6, KHALA_11_8, BENEFIC_ALL_KENDRAS,
MATSYA, CHAPA_PHALADEEPIKA, CHATUSSAGARA, LAGNADHI, PUSHYA,
RAJA_LAKSHANA, JUP_IN_9_EXALTED, PRAVRAJYA_TRIDANDI,
DARIDRA_MOON_SATURN_12, DARIDRA_TRIKA_LUMINARY,
ALPAYU_MOON_SATURN_AFFLICTED, CHIRANJIVI_KENDRA_BENEFICS,
VIPAREETA_RAJA_TRIPLE, DUR_BHAGYA_SUN_KETU_9,
DUR_SAMBANDHA_MARS_VENUS_AXIS, DUR_KARMA_SATURN_RAHU_10,
DUR_KARMA_SUN_DEBIL_10, DUR_SAHAJA_MARS_DEBIL_3, PITRA_BHAGYA_SUN_9,
BHAGYA_LORDS_TRIKONA_EXCHANGE, SAHAJA_LORDS_PARIVARTANA_3_11,
KARMA_LORDS_EXCHANGE_1_10, RAVI_BHASHKARA_SUN_MERCURY_10,
VIDYA_4_5_LORDS_PARIVARTANA, VAGISH_MERCURY_2_5_9_OWN_EXALT,
SANTANA_LORDS_PARIVARTANA_5_9, JAIMINI_AK_AmK_PARIVARTANA_PROXY,
TAJIK_YAMAYYA_DUAL_BENEFIC_LAGNA, TAJIK_RUD_DHAR_LORDS_PARIVARTANA_1_10.

## The eight, written as rules

`crates/rules/rules/computed-yogas.json` says in the language what the
engine computes in code: the Neecha Bhanga aggregate and its seven
cancellations, each over any debilitated graha. Measured against what
its code recorded:

| rule | decisions | presences | what parts |
|---|---|---|---|
| `NEECHA_BHANGA_RAJA` | 0 of 93 | 54 | the planets' order on 3 presences |
| `NEECHA_BHANGA_DEBIL_LORD_KENDRA` | 0 of 93 | 43 | nothing |
| `NEECHA_BHANGA_EXALT_PLANET_KENDRA` | 0 of 93 | 30 | nothing |
| `NEECHA_BHANGA_D9_EXALT` | 0 of 93 | 9 | nothing |
| `NEECHA_BHANGA_BENEFIC_ASPECT` | 0 of 93 | 14 | nothing |
| `NEECHA_BHANGA_OWN_NAVAMSA` | 0 of 93 | 8 | nothing |
| `NEECHA_BHANGA_EXALT_LORD_KENDRA` | 0 of 93 | 40 | nothing |
| `NEECHA_BHANGA_RETROGRADE` | 0 of 93 | 4 | nothing |

The engine's own "cancellations" for these eight are the prose traces
its code writes, which no rule carries, and are left out. Where the
aggregate's planets part, the same grahas are listed in another order:
these rules list them as the chart does, the Sun to Ketu, and the engine
lists them in the order its seven conditions hit.

## What it means for the kernel

**The engine's semantics are settled over the corpus**, each a choice
the rows above measure: conjunction in one sign unless a rule gives an
orb, the Moon malefic when waning and Mercury when only malefics share
its sign, a deep dignity meeting its plain form, all seven between the
nodes on either side, the nodes never retrograde, houses as recorded,
and a rule's planets gathered from every condition that held, including
inside a branch that went on to fail. Two of those choices the corpus
decides — the Moon's and Mercury's natures, and the sign against an
orb — and the rest it cannot see, since flipping them moves nothing;
the kernel still makes each explicit, and a fixture that separates them
is worth adding.

**The involved planets are part of the answer**, so the kernel's trace
has to reproduce the engine's accumulation where the conformance profile
asks for it, and may report the deciding branch alone where it does not.

**Coverage is the kernel's first test debt.** A rule never present here
has only negative cases, and the rules-engine page requires a positive
and a negative fixture before a rule is marked stable; the Neecha Bhanga
family needs the kernel's table lookups or its divisional-chart
predicate before it can be written as rules at all.
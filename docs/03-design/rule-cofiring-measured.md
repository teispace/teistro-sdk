# Rules that hold together, measured

Status: `generated` by `cargo xtask cofiring`; `check-cofiring` regenerates it in verify and fails on any difference.

Two rules that hold on exactly the same charts are a duplicate unless something says why ([`01-quality-bar.md`](../05-testing/01-quality-bar.md), "rule fixtures"). The pass evaluates every shipped rule, 4819 of them, on 6000 charts the SDK computes: 1000 at each of 6 places from Quito to Sydney, from 1900 to 2100, no two at one instant. 4513 rules hold at least once and 306 never do.

Rules holding on the same 5 or more charts form 628 groups, and each is explained. Below that, 38 groups are nested conjunctions of five and six grahas, each held on a chart or two, which is chance.

- **509 groups: two texts state one condition.** The rules cite different texts, so the group is agreement between sources, and each rule carries its own reading.
- **36 groups: one aspect read from both grahas.** Saravali reads a graha in each sign under each aspect, a chapter to a graha, and two grahas that cast only the seventh aspect each other exactly when each stands seventh from the other.
- **83 groups: one text says it twice**, where the table below says.

A group of one text that is none of these fails the gate, and so does a listed group that no longer holds together.

## One text, twice

| charts | rules | where |
|---:|---|---|
| 38 | `PARIJATA_TOGETHER_SUN_MOON_SATURN`<br>`SARAVALI_SATURN_WITH_BOTH_LUMINARIES`<br>`SARAVALI_TOGETHER_SUN_MOON_SATURN` | Saravali reads Saturn with both luminaries twice: ch. 10 vv. 6 and 7 among the deaths of children, ch. 16 v. 5 among the sets of three |
| 515 | `BPHS_ARMY_CHIEF_BY_MALEFICS_THIRD_AND_SIXTH_FROM_ATMAKARAKA`<br>`BPHS_KING_BY_MALEFICS_THIRD_AND_SIXTH_FROM_ATMAKARAKA` | BPHS gives malefics third and sixth from the Atmakaraka twice: ch. 39 v. 10 makes the native a king, ch. 40 v. 7 an army chief |
| 38 | `SARAVALI_MERCURY_AND_JUPITER_IN_THE_SEVENTH`<br>`SARAVALI_MERCURY_JUPITER_IN_BHAVA_7` | Saravali reads Mercury and Jupiter in the seventh in ch. 34 v. 47 (one wife), and again among two grahas in an angle |
| 113 | `SARAVALI_MERCURY_AND_VENUS_IN_THE_SEVENTH`<br>`SARAVALI_MERCURY_VENUS_IN_BHAVA_7` | Saravali reads Mercury and Venus in the seventh in ch. 34, and again among two grahas in an angle |
| 1008 | `PHALADEEPIKA_KETU_IN_OR_ASPECTING_THE_EIGHTH_CAUSE_OF_DEATH`<br>`PHALADEEPIKA_RAHU_IN_OR_ASPECTING_THE_EIGHTH_CAUSE_OF_DEATH` | Phaladeepika ch. 14 vv. 13 and 15 give each node its own death; the nodes stand opposite and aspect by the seventh alone, so one in the eighth or the second puts both on it |
| 512 | `BJ_MOON_IN_TAURUS`<br>`SARAVALI_MOON_IN_TAURUS`<br>`SARAVALI_RAJA_MOON_EXALTED` | Saravali reads the Moon in Taurus in ch. 23 among her signs, and again in ch. 35 v. 106 as the exalted Moon that makes a king even when waning |
| 217 | `PARIJATA_DARIDRA_SATURN_MOON_SUN_IN_KENDRAS`<br>`PARIJATA_ROGA_SATURN_MOON_SUN_IN_KENDRAS_IDIOT` | Jataka Parijata gives Saturn, the Moon and the Sun in kendras twice: ch. 6 v. 4 among the poverty yogas, ch. 6 v. 81 among the diseases of the mind |
| 39 | `PARIJATA_JANMA_JARAJA_LORDS_OF_ONE_AND_FOUR_EXCHANGED`<br>`PARIJATA_VIYONI_LORDS_OF_ONE_AND_FOUR_EXCHANGED` | Jataka Parijata ch. 3 gives the first and fourth lords exchanged twice: v. 5 a birth of another kind, v. 54 the child of another man, the same figure read of a human chart |
| 488 | `DWIGRAHA_SUN_MOON`<br>`PARIJATA_JANMA_KANINA_LUMINARIES_TOGETHER`<br>`PARIJATA_TOGETHER_SUN_MOON`<br>`SARAVALI_TOGETHER_SUN_MOON` | Jataka Parijata gives the Sun and the Moon together twice: ch. 3 v. 51 among the parentages, the child born before marriage, and ch. 8 among the conjunctions |
| 512 | `BJ_MARS_IN_BHAVA_3`<br>`BPHS_MARS_IN_THIRD_DEATH_BY_WOUNDS`<br>`BPHS_SAHAJA_MARS_IN_THE_THIRD`<br>`SARAVALI_MARS_IN_BHAVA_3` | BPHS gives the figure twice: ch. 44 v. 25-31 (marana) and ch. 14 v. 14 (house-based) |
| 497 | `BJ_SATURN_IN_BHAVA_1`<br>`PARIJATA_JANMA_FATHER_AWAY_SATURN_RISING`<br>`SARAVALI_BHANGA_SATURN_RISING`<br>`SARAVALI_SATURN_IN_BHAVA_1` | Saravali gives Saturn rising twice: ch. 38 v. 12 among what cancels a raja yoga, and among the grahas in the houses |
| 94 | `PARIJATA_TOGETHER_SUN_MARS_MERCURY_VENUS`<br>`SARAVALI_PRAVRAJYA_SUN_MARS_MERCURY_VENUS`<br>`SARAVALI_TOGETHER_SUN_MARS_MERCURY_VENUS` | Saravali gives the Sun, Mars, Mercury and Venus in one sign twice: ch. 20 v. 13 among the ascetics, and among its sets of grahas together |
| 49 | `PARIJATA_TOGETHER_SUN_MARS_MERCURY_JUPITER`<br>`SARAVALI_PRAVRAJYA_SUN_MARS_MERCURY_JUPITER`<br>`SARAVALI_TOGETHER_SUN_MARS_MERCURY_JUPITER` | Saravali gives the Sun, Mars, Mercury and Jupiter in one sign twice: ch. 20 v. 4, 12 among the ascetics, and among its sets of grahas together |
| 41 | `BJ_MARANA_SATURN_IN_CANCER_MOON_IN_CAPRICORN_DEATH_BY_DROPSY`<br>`PARIJATA_MARANA_SATURN_IN_CANCER_MOON_IN_CAPRICORN_DEATH_BY_DROPSY`<br>`PARIJATA_ROGA_SATURN_IN_CANCER_MOON_IN_CAPRICORN_DROPSY`<br>`SARAVALI_MARANA_SATURN_IN_CANCER_MOON_IN_CAPRICORN_DROPSY`<br>`SARAVALI_SATURN_IN_CANCER_ASPECTED_BY_MOON` | Saravali gives Saturn in Cancer with the Moon in Capricorn twice: ch. 46 v. 4 as the manner of death, and among the grahas aspected |
| 35 | `PARIJATA_TOGETHER_SUN_MOON_MARS_MERCURY`<br>`SARAVALI_PRAVRAJYA_SUN_MOON_MARS_MERCURY`<br>`SARAVALI_TOGETHER_SUN_MOON_MARS_MERCURY` | Saravali gives the Sun, the Moon, Mars and Mercury in one sign twice: ch. 20 v. 2, 16 among the ascetics, and among its sets of grahas together |
| 32 | `BPHS_BHAGYA_SATURN_SECOND_SUN_SEVENTH_FROM_SATURN`<br>`BPHS_SATURN_IN_SECOND_ASPECTED_BY_SUN` | BPHS gives the figure twice: ch. 20 v. 23-24 (arishta-father) and ch. 42 v. 18 (daridra) |
| 30 | `BPHS_KALATRA_SECOND_AND_ELEVENTH_LORDS_EXCHANGED`<br>`BPHS_LABHA_SECOND_AND_ELEVENTH_LORDS_EXCHANGED` | BPHS gives the figure twice: ch. 18 v. 29 (kalatra) and ch. 22 v. 9 (dhana) |
| 22 | `PARIJATA_TOGETHER_SUN_MERCURY_JUPITER_SATURN`<br>`SARAVALI_PRAVRAJYA_SUN_MERCURY_JUPITER_SATURN`<br>`SARAVALI_TOGETHER_SUN_MERCURY_JUPITER_SATURN` | Saravali gives the Sun, Mercury, Jupiter and Saturn in one sign twice: ch. 20 v. 4 among the ascetics, and among its sets of grahas together |
| 22 | `PARIJATA_TOGETHER_SUN_MOON_MERCURY_SATURN`<br>`SARAVALI_PRAVRAJYA_SUN_MOON_MERCURY_SATURN`<br>`SARAVALI_TOGETHER_SUN_MOON_MERCURY_SATURN` | Saravali gives the Sun, the Moon, Mercury and Saturn in one sign twice: ch. 20 v. 3 among the ascetics, and among its sets of grahas together |
| 21 | `PARIJATA_TOGETHER_MOON_MARS_MERCURY_VENUS`<br>`SARAVALI_PRAVRAJYA_MOON_MARS_MERCURY_VENUS`<br>`SARAVALI_TOGETHER_MOON_MARS_MERCURY_VENUS` | Saravali gives the Moon, Mars, Mercury and Venus in one sign twice: ch. 20 v. 16 among the ascetics, and among its sets of grahas together |
| 18 | `PARIJATA_TOGETHER_SUN_JUPITER_VENUS_SATURN`<br>`SARAVALI_PRAVRAJYA_SUN_JUPITER_VENUS_SATURN`<br>`SARAVALI_TOGETHER_SUN_JUPITER_VENUS_SATURN` | Saravali gives the Sun, Jupiter, Venus and Saturn in one sign twice: ch. 20 v. 16 among the ascetics, and among its sets of grahas together |
| 17 | `PARIJATA_TOGETHER_SUN_MOON_MARS_VENUS`<br>`SARAVALI_PRAVRAJYA_SUN_MOON_MARS_VENUS`<br>`SARAVALI_TOGETHER_SUN_MOON_MARS_VENUS` | Saravali gives the Sun, the Moon, Mars and Venus in one sign twice: ch. 20 v. 16 among the ascetics, and among its sets of grahas together |
| 11 | `PARIJATA_TOGETHER_SUN_MARS_MERCURY_JUPITER_VENUS`<br>`SARAVALI_PRAVRAJYA_SUN_MARS_MERCURY_JUPITER_VENUS`<br>`SARAVALI_TOGETHER_SUN_MARS_MERCURY_JUPITER_VENUS` | Saravali gives the Sun, Mars, Mercury, Jupiter and Venus in one sign twice: ch. 20 v. 9 among the ascetics, and among its sets of grahas together |
| 11 | `PARIJATA_TOGETHER_SUN_MOON_MARS_MERCURY_VENUS`<br>`SARAVALI_PRAVRAJYA_SUN_MOON_MARS_MERCURY_VENUS`<br>`SARAVALI_TOGETHER_SUN_MOON_MARS_MERCURY_VENUS` | Saravali gives the Sun, the Moon, Mars, Mercury and Venus in one sign twice: ch. 20 v. 8 among the ascetics, and among its sets of grahas together |
| 9 | `PARIJATA_TOGETHER_SUN_MARS_MERCURY_VENUS_SATURN`<br>`SARAVALI_PRAVRAJYA_SUN_MARS_MERCURY_VENUS_SATURN`<br>`SARAVALI_TOGETHER_SUN_MARS_MERCURY_VENUS_SATURN` | Saravali gives the Sun, Mars, Mercury, Venus and Saturn in one sign twice: ch. 20 v. 2, 6 among the ascetics, and among its sets of grahas together |
| 9 | `PARIJATA_TOGETHER_SUN_MOON_VENUS_SATURN`<br>`SARAVALI_PRAVRAJYA_SUN_MOON_VENUS_SATURN`<br>`SARAVALI_TOGETHER_SUN_MOON_VENUS_SATURN` | Saravali gives the Sun, the Moon, Venus and Saturn in one sign twice: ch. 20 v. 12 among the ascetics, and among its sets of grahas together |
| 6 | `PARIJATA_TOGETHER_MOON_MARS_JUPITER_VENUS`<br>`SARAVALI_PRAVRAJYA_MOON_MARS_JUPITER_VENUS`<br>`SARAVALI_TOGETHER_MOON_MARS_JUPITER_VENUS` | Saravali gives the Moon, Mars, Jupiter and Venus in one sign twice: ch. 20 v. 16 among the ascetics, and among its sets of grahas together |
| 6 | `PARIJATA_TOGETHER_MOON_MARS_MERCURY_JUPITER`<br>`SARAVALI_PRAVRAJYA_MOON_MARS_MERCURY_JUPITER`<br>`SARAVALI_TOGETHER_MOON_MARS_MERCURY_JUPITER` | Saravali gives the Moon, Mars, Mercury and Jupiter in one sign twice: ch. 20 v. 13 among the ascetics, and among its sets of grahas together |
| 6 | `PARIJATA_TOGETHER_MOON_MARS_MERCURY_SATURN`<br>`SARAVALI_PRAVRAJYA_MOON_MARS_MERCURY_SATURN`<br>`SARAVALI_TOGETHER_MOON_MARS_MERCURY_SATURN` | Saravali gives the Moon, Mars, Mercury and Saturn in one sign twice: ch. 20 v. 14 among the ascetics, and among its sets of grahas together |
| 5 | `PARIJATA_TOGETHER_MARS_MERCURY_JUPITER_SATURN`<br>`SARAVALI_PRAVRAJYA_MARS_MERCURY_JUPITER_SATURN`<br>`SARAVALI_TOGETHER_MARS_MERCURY_JUPITER_SATURN` | Saravali gives Mars, Mercury, Jupiter and Saturn in one sign twice: ch. 20 v. 6, 14 among the ascetics, and among its sets of grahas together |
| 5 | `PARIJATA_TOGETHER_SUN_MARS_JUPITER_VENUS_SATURN`<br>`SARAVALI_PRAVRAJYA_SUN_MARS_JUPITER_VENUS_SATURN`<br>`SARAVALI_TOGETHER_SUN_MARS_JUPITER_VENUS_SATURN` | Saravali gives the Sun, Mars, Jupiter, Venus and Saturn in one sign twice: ch. 20 v. 7 among the ascetics, and among its sets of grahas together |
| 1979 | `BJ_MARANA_FIXED_EIGHTH_PLACE_OF_DEATH`<br>`BPHS_DUAL_THIRD_PLACE_OF_DEATH`<br>`BPHS_STRI_SEVENTH_MOVABLE`<br>`PARIJATA_MARANA_FIXED_EIGHTH_PLACE_OF_DEATH`<br>`SARAVALI_MARANA_FIXED_EIGHTH_PLACE_OF_DEATH` | BPHS reads a movable lagna twice: ch. 44 v. 34 by its dual third, the place of death, and ch. 80 v. 17 by its movable seventh, in a woman's chart |
| 1480 | `PHALADEEPIKA_AVA`<br>`PHALADEEPIKA_BHAVA_LAGNA_LORD_IN_A_DUSTHANA` | Phaladeepika gives the lagna lord in a dusthana twice: ch. 6 vv. 57 to 69's Ava yoga, and ch. 16 v. 4 among the effects of the houses |
| 1359 | `PARIJATA_A_BENEFIC_FALLEN`<br>`PARIJATA_JANMA_A_BENEFIC_DEBILITATED` | Jataka Parijata gives a benefic debilitated twice: ch. 6 v. 35, and ch. 3 v. 69 among the circumstances of birth |
| 562 | `BPHS_KARAKAMSHA_KETU_ASPECTING_THE_FIFTH_FROM_IT`<br>`BPHS_KARAKAMSHA_RAHU_FIFTH_FROM_IT` | BPHS ch. 33 gives one figure two ways: Ketu aspecting the fifth from the Karakamsha (v. 37) is Rahu in it (v. 39), Ketu's seventh being Rahu's sign |
| 483 | `BJ_MARS_IN_BHAVA_7`<br>`BPHS_STRI_MARS_SEVENTH`<br>`PARIJATA_JANMA_FATHER_AWAY_MARS_SEVENTH`<br>`SARAVALI_MARS_IN_BHAVA_7`<br>`SARAVALI_STRI_MARS_SEVENTH_WIDOWED_A_BRIDE` | Saravali gives Mars in the seventh twice: ch. 45 v. 16 in a woman's chart, and among the grahas in the houses |
| 470 | `BPHS_KARAKAMSHA_KETU_IN_IT`<br>`BPHS_KARAKAMSHA_RAHU_SEVENTH_FROM_IT` | BPHS ch. 33 gives one figure two ways, the nodes standing opposite: Ketu in the Karakamsha (v. 18) is Rahu in its seventh (v. 48) |
| 219 | `SARAVALI_DECANATE_1_OF_CANCER`<br>`SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_AQUARIUS_FIRST` | Saravali reads the first decanate of Cancer rising twice: among the rising decanates, and in ch. 46 vv. 22, 42 by the death the 22nd drekkana from it, Aquarius's first, gives |
| 206 | `SARAVALI_DECANATE_2_OF_SCORPIO`<br>`SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_GEMINI_SECOND` | Saravali reads the second decanate of Scorpio rising twice: among the rising decanates, and in ch. 46 vv. 22, 27 by the death the 22nd drekkana from it, Gemini's second, gives |
| 205 | `SARAVALI_DECANATE_1_OF_LEO`<br>`SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_PISCES_FIRST` | Saravali reads the first decanate of Leo rising twice: among the rising decanates, and in ch. 46 vv. 22, 44 by the death the 22nd drekkana from it, Pisces's first, gives |
| 194 | `SARAVALI_DECANATE_2_OF_LIBRA`<br>`SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_TAURUS_SECOND` | Saravali reads the second decanate of Libra rising twice: among the rising decanates, and in ch. 46 vv. 22, 25 by the death the 22nd drekkana from it, Taurus's second, gives |
| 193 | `SARAVALI_DECANATE_1_OF_GEMINI`<br>`SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_CAPRICORN_FIRST` | Saravali reads the first decanate of Gemini rising twice: among the rising decanates, and in ch. 46 vv. 22, 40 by the death the 22nd drekkana from it, Capricorn's first, gives |
| 192 | `SARAVALI_DECANATE_2_OF_SAGITTARIUS`<br>`SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_CANCER_SECOND` | Saravali reads the second decanate of Sagittarius rising twice: among the rising decanates, and in ch. 46 vv. 22, 30 by the death the 22nd drekkana from it, Cancer's second, gives |
| 190 | `SARAVALI_DECANATE_1_OF_LIBRA`<br>`SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_TAURUS_FIRST` | Saravali reads the first decanate of Libra rising twice: among the rising decanates, and in ch. 46 vv. 22, 25 by the death the 22nd drekkana from it, Taurus's first, gives |
| 184 | `SARAVALI_DECANATE_3_OF_SCORPIO`<br>`SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_GEMINI_THIRD` | Saravali reads the third decanate of Scorpio rising twice: among the rising decanates, and in ch. 46 vv. 22, 28 by the death the 22nd drekkana from it, Gemini's third, gives |
| 183 | `SARAVALI_DECANATE_1_OF_SCORPIO`<br>`SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_GEMINI_FIRST` | Saravali reads the first decanate of Scorpio rising twice: among the rising decanates, and in ch. 46 vv. 22, 27 by the death the 22nd drekkana from it, Gemini's first, gives |
| 183 | `SARAVALI_DECANATE_2_OF_GEMINI`<br>`SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_CAPRICORN_SECOND` | Saravali reads the second decanate of Gemini rising twice: among the rising decanates, and in ch. 46 vv. 22, 41 by the death the 22nd drekkana from it, Capricorn's second, gives |
| 182 | `SARAVALI_DECANATE_2_OF_VIRGO`<br>`SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_ARIES_SECOND` | Saravali reads the second decanate of Virgo rising twice: among the rising decanates, and in ch. 46 vv. 22, 24 by the death the 22nd drekkana from it, Aries's second, gives |
| 177 | `SARAVALI_DECANATE_3_OF_LEO`<br>`SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_PISCES_THIRD` | Saravali reads the third decanate of Leo rising twice: among the rising decanates, and in ch. 46 vv. 22, 45 by the death the 22nd drekkana from it, Pisces's third, gives |
| 176 | `SARAVALI_DECANATE_1_OF_TAURUS`<br>`SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_SAGITTARIUS_FIRST` | Saravali reads the first decanate of Taurus rising twice: among the rising decanates, and in ch. 46 vv. 22, 38 by the death the 22nd drekkana from it, Sagittarius's first, gives |
| 176 | `SARAVALI_DECANATE_3_OF_CANCER`<br>`SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_AQUARIUS_THIRD` | Saravali reads the third decanate of Cancer rising twice: among the rising decanates, and in ch. 46 vv. 22, 43 by the death the 22nd drekkana from it, Aquarius's third, gives |
| 176 | `SARAVALI_DECANATE_3_OF_VIRGO`<br>`SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_ARIES_THIRD` | Saravali reads the third decanate of Virgo rising twice: among the rising decanates, and in ch. 46 vv. 22, 24 by the death the 22nd drekkana from it, Aries's third, gives |
| 174 | `SARAVALI_DECANATE_3_OF_GEMINI`<br>`SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_CAPRICORN_THIRD` | Saravali reads the third decanate of Gemini rising twice: among the rising decanates, and in ch. 46 vv. 22, 41 by the death the 22nd drekkana from it, Capricorn's third, gives |
| 174 | `SARAVALI_DECANATE_3_OF_LIBRA`<br>`SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_TAURUS_THIRD` | Saravali reads the third decanate of Libra rising twice: among the rising decanates, and in ch. 46 vv. 22, 26 by the death the 22nd drekkana from it, Taurus's third, gives |
| 171 | `SARAVALI_DECANATE_2_OF_CANCER`<br>`SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_AQUARIUS_SECOND` | Saravali reads the second decanate of Cancer rising twice: among the rising decanates, and in ch. 46 vv. 22, 43 by the death the 22nd drekkana from it, Aquarius's second, gives |
| 170 | `SARAVALI_DECANATE_2_OF_LEO`<br>`SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_PISCES_SECOND` | Saravali reads the second decanate of Leo rising twice: among the rising decanates, and in ch. 46 vv. 22, 45 by the death the 22nd drekkana from it, Pisces's second, gives |
| 167 | `SARAVALI_DECANATE_2_OF_CAPRICORN`<br>`SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_LEO_SECOND` | Saravali reads the second decanate of Capricorn rising twice: among the rising decanates, and in ch. 46 vv. 22, 31 by the death the 22nd drekkana from it, Leo's second, gives |
| 166 | `SARAVALI_DECANATE_1_OF_SAGITTARIUS`<br>`SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_CANCER_FIRST` | Saravali reads the first decanate of Sagittarius rising twice: among the rising decanates, and in ch. 46 vv. 22, 29 by the death the 22nd drekkana from it, Cancer's first, gives |
| 165 | `SARAVALI_DECANATE_3_OF_SAGITTARIUS`<br>`SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_CANCER_THIRD` | Saravali reads the third decanate of Sagittarius rising twice: among the rising decanates, and in ch. 46 vv. 22, 30 by the death the 22nd drekkana from it, Cancer's third, gives |
| 164 | `SARAVALI_DECANATE_3_OF_TAURUS`<br>`SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_SAGITTARIUS_THIRD` | Saravali reads the third decanate of Taurus rising twice: among the rising decanates, and in ch. 46 vv. 22, 39 by the death the 22nd drekkana from it, Sagittarius's third, gives |
| 157 | `SARAVALI_DECANATE_2_OF_TAURUS`<br>`SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_SAGITTARIUS_SECOND` | Saravali reads the second decanate of Taurus rising twice: among the rising decanates, and in ch. 46 vv. 22, 38 by the death the 22nd drekkana from it, Sagittarius's second, gives |
| 154 | `SARAVALI_DECANATE_3_OF_ARIES`<br>`SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_SCORPIO_THIRD` | Saravali reads the third decanate of Aries rising twice: among the rising decanates, and in ch. 46 vv. 22, 37 by the death the 22nd drekkana from it, Scorpio's third, gives |
| 148 | `SARAVALI_DECANATE_1_OF_CAPRICORN`<br>`SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_LEO_FIRST` | Saravali reads the first decanate of Capricorn rising twice: among the rising decanates, and in ch. 46 vv. 22, 31 by the death the 22nd drekkana from it, Leo's first, gives |
| 144 | `SARAVALI_DECANATE_3_OF_PISCES`<br>`SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_LIBRA_THIRD` | Saravali reads the third decanate of Pisces rising twice: among the rising decanates, and in ch. 46 vv. 22, 35 by the death the 22nd drekkana from it, Libra's third, gives |
| 140 | `SARAVALI_DECANATE_2_OF_AQUARIUS`<br>`SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_VIRGO_SECOND` | Saravali reads the second decanate of Aquarius rising twice: among the rising decanates, and in ch. 46 vv. 22, 33 by the death the 22nd drekkana from it, Virgo's second, gives |
| 135 | `SARAVALI_DECANATE_3_OF_CAPRICORN`<br>`SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_LEO_THIRD` | Saravali reads the third decanate of Capricorn rising twice: among the rising decanates, and in ch. 46 vv. 22, 32 by the death the 22nd drekkana from it, Leo's third, gives |
| 134 | `SARAVALI_DECANATE_2_OF_PISCES`<br>`SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_LIBRA_SECOND` | Saravali reads the second decanate of Pisces rising twice: among the rising decanates, and in ch. 46 vv. 22, 35 by the death the 22nd drekkana from it, Libra's second, gives |
| 130 | `SARAVALI_DECANATE_1_OF_AQUARIUS`<br>`SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_VIRGO_FIRST` | Saravali reads the first decanate of Aquarius rising twice: among the rising decanates, and in ch. 46 vv. 22, 33 by the death the 22nd drekkana from it, Virgo's first, gives |
| 130 | `SARAVALI_DECANATE_1_OF_ARIES`<br>`SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_SCORPIO_FIRST` | Saravali reads the first decanate of Aries rising twice: among the rising decanates, and in ch. 46 vv. 22, 36 by the death the 22nd drekkana from it, Scorpio's first, gives |
| 126 | `SARAVALI_DECANATE_3_OF_AQUARIUS`<br>`SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_VIRGO_THIRD` | Saravali reads the third decanate of Aquarius rising twice: among the rising decanates, and in ch. 46 vv. 22, 34 by the death the 22nd drekkana from it, Virgo's third, gives |
| 125 | `SARAVALI_DECANATE_1_OF_PISCES`<br>`SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_LIBRA_FIRST` | Saravali reads the first decanate of Pisces rising twice: among the rising decanates, and in ch. 46 vv. 22, 35 by the death the 22nd drekkana from it, Libra's first, gives |
| 124 | `BJ_STRI_VENUS_AND_MERCURY_RISING`<br>`SARAVALI_MERCURY_VENUS_IN_BHAVA_1`<br>`SARAVALI_STRI_VENUS_AND_MERCURY_RISING` | Saravali gives Mercury and Venus in the first twice: ch. 45 v. 26 in a woman's chart, and among the grahas in the houses |
| 121 | `SARAVALI_DECANATE_2_OF_ARIES`<br>`SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_SCORPIO_SECOND` | Saravali reads the second decanate of Aries rising twice: among the rising decanates, and in ch. 46 vv. 22, 36 by the death the 22nd drekkana from it, Scorpio's second, gives |
| 98 | `BJ_JANMA_THREE_DEBILITATED_BORN_ON_THE_GROUND`<br>`PARIJATA_JANMA_THREE_DEBILITATED`<br>`PARIJATA_YOGA_THREE_DEBILITATED`<br>`SARAVALI_BHANGA_THREE_DEBILITATED` | Jataka Parijata gives three grahas debilitated twice: ch. 8 v. 111 among the counts of grahas, and ch. 3 v. 70 among the circumstances of birth |
| 43 | `BJ_STRI_MOON_AND_VENUS_RISING`<br>`SARAVALI_MOON_VENUS_IN_BHAVA_1`<br>`SARAVALI_STRI_MOON_AND_VENUS_RISING` | Saravali gives the Moon and Venus in the first twice: ch. 45 v. 25 in a woman's chart, and among the grahas in the houses |
| 31 | `BJ_STRI_MOON_AND_MERCURY_RISING`<br>`SARAVALI_MOON_MERCURY_IN_BHAVA_1`<br>`SARAVALI_STRI_MOON_AND_MERCURY_RISING` | Saravali gives the Moon and Mercury in the first twice: ch. 45 v. 25 in a woman's chart, and among the grahas in the houses |
| 506 | `BJ_MARANA_SUN_IN_THE_EIGHTH_DEATH_BY_FIRE`<br>`BJ_SUN_IN_BHAVA_8`<br>`BPHS_STRI_SUN_EIGHTH`<br>`BPHS_SUN_IN_EIGHTH_DEATH_BY_FIRE`<br>`PARIJATA_MARANA_SUN_IN_THE_EIGHTH_DEATH_BY_FIRE`<br>`SARAVALI_MARANA_SUN_IN_THE_EIGHTH_DEATH_BY_FIRE`<br>`SARAVALI_SUN_IN_BHAVA_8` | Brihat Jataka gives the Sun in the eighth twice: ch. 25 v. 1 as the manner of death, and ch. 20 v. 3 among the grahas in the bhavas |
| 485 | `BJ_MARANA_MOON_IN_THE_EIGHTH_DEATH_BY_WATER`<br>`BJ_MOON_IN_BHAVA_8`<br>`BPHS_MOON_IN_EIGHTH_DEATH_BY_WATER`<br>`BPHS_STRI_MOON_EIGHTH`<br>`PARIJATA_MARANA_MOON_IN_THE_EIGHTH_DEATH_BY_WATER`<br>`SARAVALI_MARANA_MOON_IN_THE_EIGHTH_DEATH_BY_WATER`<br>`SARAVALI_MOON_IN_BHAVA_8` | Brihat Jataka gives the Moon in the eighth twice: ch. 25 v. 1 as the manner of death, and ch. 20 v. 5 among the grahas in the bhavas |
| 501 | `BJ_MARANA_MARS_IN_THE_EIGHTH_DEATH_BY_A_WEAPON`<br>`BJ_MARS_IN_BHAVA_8`<br>`BPHS_MARS_IN_EIGHTH_DEATH_BY_WEAPONS`<br>`BPHS_STRI_MARS_EIGHTH`<br>`PARIJATA_MARANA_MARS_IN_THE_EIGHTH_DEATH_BY_A_WEAPON`<br>`SARAVALI_MARANA_MARS_IN_THE_EIGHTH_DEATH_BY_A_WEAPON`<br>`SARAVALI_MARS_IN_BHAVA_8` | Brihat Jataka gives Mars in the eighth twice: ch. 25 v. 1 as the manner of death, and ch. 20 v. 6 among the grahas in the bhavas |
| 499 | `BJ_MARANA_MERCURY_IN_THE_EIGHTH_DEATH_BY_FEVER`<br>`BJ_MERCURY_IN_BHAVA_8`<br>`BPHS_MERCURY_IN_EIGHTH_DEATH_BY_FEVER`<br>`BPHS_STRI_MERCURY_EIGHTH`<br>`PARIJATA_MARANA_MERCURY_IN_THE_EIGHTH_DEATH_BY_FEVER`<br>`SARAVALI_MARANA_MERCURY_IN_THE_EIGHTH_DEATH_BY_FEVER`<br>`SARAVALI_MERCURY_IN_BHAVA_8` | Brihat Jataka gives Mercury in the eighth twice: ch. 25 v. 1 as the manner of death, and ch. 20 v. 6 among the grahas in the bhavas |
| 505 | `BJ_JUPITER_IN_BHAVA_8`<br>`BJ_MARANA_JUPITER_IN_THE_EIGHTH_DEATH_BY_AN_UNKNOWN_ILLNESS`<br>`BPHS_JUPITER_IN_EIGHTH_DEATH_BY_DISEASE`<br>`BPHS_STRI_JUPITER_EIGHTH`<br>`PARIJATA_MARANA_JUPITER_IN_THE_EIGHTH_DEATH_BY_AN_UNKNOWN_ILLNESS`<br>`SARAVALI_JUPITER_IN_BHAVA_8`<br>`SARAVALI_MARANA_JUPITER_IN_THE_EIGHTH_DEATH_BY_DISEASE` | Brihat Jataka gives Jupiter in the eighth twice: ch. 25 v. 1 as the manner of death, and ch. 20 v. 7 among the grahas in the bhavas |
| 513 | `BJ_MARANA_VENUS_IN_THE_EIGHTH_DEATH_BY_THIRST`<br>`BJ_VENUS_IN_BHAVA_8`<br>`BPHS_STRI_VENUS_EIGHTH`<br>`BPHS_VENUS_IN_EIGHTH_DEATH_BY_HUNGER`<br>`PARIJATA_MARANA_VENUS_IN_THE_EIGHTH_DEATH_BY_THIRST`<br>`SARAVALI_MARANA_VENUS_IN_THE_EIGHTH_DEATH_BY_THIRST`<br>`SARAVALI_VENUS_IN_BHAVA_8` | Brihat Jataka gives Venus in the eighth twice: ch. 25 v. 1 as the manner of death, and ch. 20 v. 8 among the grahas in the bhavas |
| 504 | `BJ_MARANA_SATURN_IN_THE_EIGHTH_DEATH_BY_HUNGER`<br>`BJ_SATURN_IN_BHAVA_8`<br>`BPHS_SATURN_IN_EIGHTH_DEATH_BY_THIRST`<br>`BPHS_STRI_SATURN_EIGHTH`<br>`PARIJATA_MARANA_SATURN_IN_THE_EIGHTH_DEATH_BY_HUNGER`<br>`SARAVALI_MARANA_SATURN_IN_THE_EIGHTH_DEATH_BY_HUNGER`<br>`SARAVALI_SATURN_IN_BHAVA_8` | Brihat Jataka gives Saturn in the eighth twice: ch. 25 v. 1 as the manner of death, and ch. 20 v. 9 among the grahas in the bhavas |

## One aspect from both grahas

| charts | rules |
|---:|---|
| 53 | `BJ_MOON_IN_ARIES_ASPECTED_BY_SUN`<br>`CHANDRA_IN_ARIES_ASPECTED_BY_SUN`<br>`SARAVALI_MOON_IN_ARIES_ASPECTED_BY_SUN`<br>`SARAVALI_SUN_IN_LIBRA_ASPECTED_BY_MOON` |
| 50 | `BJ_MOON_IN_SAGITTARIUS_ASPECTED_BY_SUN`<br>`CHANDRA_IN_SAGITTARIUS_ASPECTED_BY_SUN`<br>`SARAVALI_MOON_IN_SAGITTARIUS_ASPECTED_BY_SUN`<br>`SARAVALI_SUN_IN_GEMINI_ASPECTED_BY_MOON` |
| 49 | `BJ_MOON_IN_ARIES_ASPECTED_BY_MERCURY`<br>`CHANDRA_IN_ARIES_ASPECTED_BY_MERCURY`<br>`SARAVALI_MERCURY_IN_LIBRA_ASPECTED_BY_MOON`<br>`SARAVALI_MOON_IN_ARIES_ASPECTED_BY_MERCURY` |
| 47 | `BJ_MOON_IN_AQUARIUS_ASPECTED_BY_SUN`<br>`CHANDRA_IN_AQUARIUS_ASPECTED_BY_SUN`<br>`SARAVALI_MOON_IN_AQUARIUS_ASPECTED_BY_SUN`<br>`SARAVALI_SUN_IN_LEO_ASPECTED_BY_MOON` |
| 47 | `BJ_MOON_IN_GEMINI_ASPECTED_BY_SUN`<br>`CHANDRA_IN_GEMINI_ASPECTED_BY_SUN`<br>`SARAVALI_MOON_IN_GEMINI_ASPECTED_BY_SUN`<br>`SARAVALI_SUN_IN_SAGITTARIUS_ASPECTED_BY_MOON` |
| 45 | `BJ_MOON_IN_CAPRICORN_ASPECTED_BY_VENUS`<br>`CHANDRA_IN_CAPRICORN_ASPECTED_BY_VENUS`<br>`SARAVALI_MOON_IN_CAPRICORN_ASPECTED_BY_VENUS`<br>`SARAVALI_VENUS_IN_CANCER_ASPECTED_BY_MOON` |
| 45 | `BJ_MOON_IN_GEMINI_ASPECTED_BY_MERCURY`<br>`CHANDRA_IN_GEMINI_ASPECTED_BY_MERCURY`<br>`SARAVALI_MERCURY_IN_SAGITTARIUS_ASPECTED_BY_MOON`<br>`SARAVALI_MOON_IN_GEMINI_ASPECTED_BY_MERCURY` |
| 45 | `BJ_MOON_IN_SCORPIO_ASPECTED_BY_VENUS`<br>`CHANDRA_IN_SCORPIO_ASPECTED_BY_VENUS`<br>`SARAVALI_MOON_IN_SCORPIO_ASPECTED_BY_VENUS`<br>`SARAVALI_VENUS_IN_TAURUS_ASPECTED_BY_MOON` |
| 44 | `BJ_MOON_IN_CANCER_ASPECTED_BY_VENUS`<br>`CHANDRA_IN_CANCER_ASPECTED_BY_VENUS`<br>`SARAVALI_MOON_IN_CANCER_ASPECTED_BY_VENUS`<br>`SARAVALI_VENUS_IN_CAPRICORN_ASPECTED_BY_MOON` |
| 44 | `BJ_MOON_IN_TAURUS_ASPECTED_BY_MERCURY`<br>`CHANDRA_IN_TAURUS_ASPECTED_BY_MERCURY`<br>`SARAVALI_MERCURY_IN_SCORPIO_ASPECTED_BY_MOON`<br>`SARAVALI_MOON_IN_TAURUS_ASPECTED_BY_MERCURY` |
| 43 | `BJ_MOON_IN_LEO_ASPECTED_BY_MERCURY`<br>`CHANDRA_IN_LEO_ASPECTED_BY_MERCURY`<br>`SARAVALI_MERCURY_IN_AQUARIUS_ASPECTED_BY_MOON`<br>`SARAVALI_MOON_IN_LEO_ASPECTED_BY_MERCURY` |
| 42 | `BJ_MOON_IN_GEMINI_ASPECTED_BY_VENUS`<br>`CHANDRA_IN_GEMINI_ASPECTED_BY_VENUS`<br>`SARAVALI_MOON_IN_GEMINI_ASPECTED_BY_VENUS`<br>`SARAVALI_VENUS_IN_SAGITTARIUS_ASPECTED_BY_MOON` |
| 42 | `BJ_MOON_IN_LIBRA_ASPECTED_BY_VENUS`<br>`CHANDRA_IN_LIBRA_ASPECTED_BY_VENUS`<br>`SARAVALI_MOON_IN_LIBRA_ASPECTED_BY_VENUS`<br>`SARAVALI_VENUS_IN_ARIES_ASPECTED_BY_MOON` |
| 42 | `BJ_MOON_IN_PISCES_ASPECTED_BY_VENUS`<br>`CHANDRA_IN_PISCES_ASPECTED_BY_VENUS`<br>`SARAVALI_MOON_IN_PISCES_ASPECTED_BY_VENUS`<br>`SARAVALI_VENUS_IN_VIRGO_ASPECTED_BY_MOON` |
| 42 | `BJ_MOON_IN_SAGITTARIUS_ASPECTED_BY_VENUS`<br>`CHANDRA_IN_SAGITTARIUS_ASPECTED_BY_VENUS`<br>`SARAVALI_MOON_IN_SAGITTARIUS_ASPECTED_BY_VENUS`<br>`SARAVALI_VENUS_IN_GEMINI_ASPECTED_BY_MOON` |
| 42 | `BJ_MOON_IN_SCORPIO_ASPECTED_BY_SUN`<br>`CHANDRA_IN_SCORPIO_ASPECTED_BY_SUN`<br>`SARAVALI_MOON_IN_SCORPIO_ASPECTED_BY_SUN`<br>`SARAVALI_SUN_IN_TAURUS_ASPECTED_BY_MOON` |
| 42 | `BJ_MOON_IN_VIRGO_ASPECTED_BY_SUN`<br>`CHANDRA_IN_VIRGO_ASPECTED_BY_SUN`<br>`SARAVALI_MOON_IN_VIRGO_ASPECTED_BY_SUN`<br>`SARAVALI_SUN_IN_PISCES_ASPECTED_BY_MOON` |
| 41 | `BJ_MOON_IN_CAPRICORN_ASPECTED_BY_MERCURY`<br>`CHANDRA_IN_CAPRICORN_ASPECTED_BY_MERCURY`<br>`SARAVALI_MERCURY_IN_CANCER_ASPECTED_BY_MOON`<br>`SARAVALI_MOON_IN_CAPRICORN_ASPECTED_BY_MERCURY` |
| 41 | `BJ_MOON_IN_LIBRA_ASPECTED_BY_SUN`<br>`CHANDRA_IN_LIBRA_ASPECTED_BY_SUN`<br>`SARAVALI_MOON_IN_LIBRA_ASPECTED_BY_SUN`<br>`SARAVALI_SUN_IN_ARIES_ASPECTED_BY_MOON` |
| 41 | `BJ_MOON_IN_SCORPIO_ASPECTED_BY_MERCURY`<br>`CHANDRA_IN_SCORPIO_ASPECTED_BY_MERCURY`<br>`SARAVALI_MERCURY_IN_TAURUS_ASPECTED_BY_MOON`<br>`SARAVALI_MOON_IN_SCORPIO_ASPECTED_BY_MERCURY` |
| 39 | `BJ_MOON_IN_AQUARIUS_ASPECTED_BY_VENUS`<br>`CHANDRA_IN_AQUARIUS_ASPECTED_BY_VENUS`<br>`SARAVALI_MOON_IN_AQUARIUS_ASPECTED_BY_VENUS`<br>`SARAVALI_VENUS_IN_LEO_ASPECTED_BY_MOON` |
| 39 | `BJ_MOON_IN_CANCER_ASPECTED_BY_MERCURY`<br>`CHANDRA_IN_CANCER_ASPECTED_BY_MERCURY`<br>`SARAVALI_MERCURY_IN_CAPRICORN_ASPECTED_BY_MOON`<br>`SARAVALI_MOON_IN_CANCER_ASPECTED_BY_MERCURY` |
| 39 | `BJ_MOON_IN_VIRGO_ASPECTED_BY_VENUS`<br>`CHANDRA_IN_VIRGO_ASPECTED_BY_VENUS`<br>`SARAVALI_MOON_IN_VIRGO_ASPECTED_BY_VENUS`<br>`SARAVALI_VENUS_IN_PISCES_ASPECTED_BY_MOON` |
| 38 | `BJ_MOON_IN_ARIES_ASPECTED_BY_VENUS`<br>`CHANDRA_IN_ARIES_ASPECTED_BY_VENUS`<br>`SARAVALI_MOON_IN_ARIES_ASPECTED_BY_VENUS`<br>`SARAVALI_VENUS_IN_LIBRA_ASPECTED_BY_MOON` |
| 38 | `BJ_MOON_IN_LEO_ASPECTED_BY_SUN`<br>`CHANDRA_IN_LEO_ASPECTED_BY_SUN`<br>`SARAVALI_MOON_IN_LEO_ASPECTED_BY_SUN`<br>`SARAVALI_SUN_IN_AQUARIUS_ASPECTED_BY_MOON` |
| 38 | `BJ_MOON_IN_LEO_ASPECTED_BY_VENUS`<br>`CHANDRA_IN_LEO_ASPECTED_BY_VENUS`<br>`SARAVALI_MOON_IN_LEO_ASPECTED_BY_VENUS`<br>`SARAVALI_VENUS_IN_AQUARIUS_ASPECTED_BY_MOON` |
| 38 | `BJ_MOON_IN_PISCES_ASPECTED_BY_MERCURY`<br>`CHANDRA_IN_PISCES_ASPECTED_BY_MERCURY`<br>`SARAVALI_MERCURY_IN_VIRGO_ASPECTED_BY_MOON`<br>`SARAVALI_MOON_IN_PISCES_ASPECTED_BY_MERCURY` |
| 38 | `BJ_MOON_IN_TAURUS_ASPECTED_BY_VENUS`<br>`CHANDRA_IN_TAURUS_ASPECTED_BY_VENUS`<br>`SARAVALI_MOON_IN_TAURUS_ASPECTED_BY_VENUS`<br>`SARAVALI_VENUS_IN_SCORPIO_ASPECTED_BY_MOON` |
| 38 | `BJ_MOON_IN_VIRGO_ASPECTED_BY_MERCURY`<br>`CHANDRA_IN_VIRGO_ASPECTED_BY_MERCURY`<br>`SARAVALI_MERCURY_IN_PISCES_ASPECTED_BY_MOON`<br>`SARAVALI_MOON_IN_VIRGO_ASPECTED_BY_MERCURY` |
| 36 | `BJ_MOON_IN_AQUARIUS_ASPECTED_BY_MERCURY`<br>`CHANDRA_IN_AQUARIUS_ASPECTED_BY_MERCURY`<br>`SARAVALI_MERCURY_IN_LEO_ASPECTED_BY_MOON`<br>`SARAVALI_MOON_IN_AQUARIUS_ASPECTED_BY_MERCURY` |
| 36 | `BJ_MOON_IN_TAURUS_ASPECTED_BY_SUN`<br>`CHANDRA_IN_TAURUS_ASPECTED_BY_SUN`<br>`SARAVALI_MOON_IN_TAURUS_ASPECTED_BY_SUN`<br>`SARAVALI_SUN_IN_SCORPIO_ASPECTED_BY_MOON` |
| 35 | `BJ_MOON_IN_CAPRICORN_ASPECTED_BY_SUN`<br>`CHANDRA_IN_CAPRICORN_ASPECTED_BY_SUN`<br>`SARAVALI_MOON_IN_CAPRICORN_ASPECTED_BY_SUN`<br>`SARAVALI_SUN_IN_CANCER_ASPECTED_BY_MOON` |
| 35 | `BJ_MOON_IN_LIBRA_ASPECTED_BY_MERCURY`<br>`CHANDRA_IN_LIBRA_ASPECTED_BY_MERCURY`<br>`SARAVALI_MERCURY_IN_ARIES_ASPECTED_BY_MOON`<br>`SARAVALI_MOON_IN_LIBRA_ASPECTED_BY_MERCURY` |
| 35 | `BJ_MOON_IN_SAGITTARIUS_ASPECTED_BY_MERCURY`<br>`CHANDRA_IN_SAGITTARIUS_ASPECTED_BY_MERCURY`<br>`SARAVALI_MERCURY_IN_GEMINI_ASPECTED_BY_MOON`<br>`SARAVALI_MOON_IN_SAGITTARIUS_ASPECTED_BY_MERCURY` |
| 32 | `BJ_MOON_IN_CANCER_ASPECTED_BY_SUN`<br>`CHANDRA_IN_CANCER_ASPECTED_BY_SUN`<br>`SARAVALI_MOON_IN_CANCER_ASPECTED_BY_SUN`<br>`SARAVALI_SUN_IN_CAPRICORN_ASPECTED_BY_MOON` |
| 30 | `BJ_MOON_IN_PISCES_ASPECTED_BY_SUN`<br>`CHANDRA_IN_PISCES_ASPECTED_BY_SUN`<br>`SARAVALI_MOON_IN_PISCES_ASPECTED_BY_SUN`<br>`SARAVALI_SUN_IN_VIRGO_ASPECTED_BY_MOON` |

## Two texts, one condition

| charts | rules |
|---:|---|
| 4942 | `BJ_LOKA_PAST_BIRTH_DREKKANA_LORD_NEITHER_EXALTED_NOR_DEBILITATED`<br>`PARIJATA_LOKA_PAST_BIRTH_DREKKANA_LORD_NEITHER_EXALTED_NOR_DEBILITATED` |
| 3591 | `BJ_JANMA_MOON_DARKENED_BY_SATURN_BORN_IN_THE_DARK`<br>`PARIJATA_JANMA_MOON_DARKENED_BY_SATURN` |
| 3565 | `BJ_MARANA_UNCONSCIOUS_LAGNA_UNASPECTED_BY_ITS_LORD_OR_A_BENEFIC`<br>`PARIJATA_MARANA_UNCONSCIOUS_LAGNA_UNASPECTED_BY_ITS_LORD_OR_A_BENEFIC`<br>`SARAVALI_MARANA_UNCONSCIOUS_LAGNA_UNASPECTED_BY_ITS_LORD_OR_A_BENEFIC` |
| 3331 | `BJ_STRI_MOVABLE_SEVENTH_HUSBAND_ABROAD`<br>`SARAVALI_STRI_MOVABLE_SEVENTH_HUSBAND_ABROAD` |
| 3156 | `BJ_JANMA_SIGN_RISING_HEAD_FIRST`<br>`PARIJATA_JANMA_SIGN_RISING_HEAD_FIRST` |
| 3053 | `DWIGRAHA_SUN_MERCURY`<br>`PARIJATA_TOGETHER_SUN_MERCURY`<br>`SARAVALI_TOGETHER_SUN_MERCURY` |
| 3030 | `BPHS_TANU_LAGNA_JOINED_OR_ASPECTED_BY_A_BENEFIC`<br>`PARIJATA_GARBHA_BENEFIC_IN_OR_ASPECTING_THE_LAGNA_SON` |
| 2785 | `BJ_STRI_SETTING_NAVAMSHA_OF_A_BENEFIC`<br>`BPHS_STRI_SEVENTH_NAVAMSHA_OF_A_BENEFIC`<br>`SARAVALI_STRI_SETTING_NAVAMSHA_OF_A_BENEFIC` |
| 2681 | `BJ_CHANDRA_KEMADRUMA`<br>`PHALADEEPIKA_KEMADRUMA` |
| 2556 | `BJ_NABHASA_PASHA`<br>`NABHASA_PASA` |
| 2441 | `BJ_JANMA_SIGN_RISING_FEET_FIRST`<br>`PARIJATA_JANMA_SIGN_RISING_FEET_FIRST` |
| 2434 | `PARIJATA_RAJA_DEBILITATED_GRAHAS_SIGN_OR_EXALTATION_LORD_IN_A_KENDRA`<br>`PHALADEEPIKA_MAHARAJA_NEECHA_BHANGA_LORD_IN_A_KENDRA` |
| 2310 | `BJ_CHANDRA_ONE_BENEFIC_IN_UPACHAYA_FROM_THE_MOON`<br>`LUNAR_DHANA_ONE_BENEFIC_IN_UPACHAYA` |
| 2291 | `PARASHARA_SANKHA`<br>`PARIJATA_SANKHA` |
| 2286 | `BJ_STRI_MALEFIC_EIGHTH_WIDOWED`<br>`SARAVALI_STRI_MALEFIC_EIGHTH_WIDOWED` |
| 2249 | `BPHS_TANU_MALEFIC_IN_THE_LAGNA`<br>`PARIJATA_ASHUBHA` |
| 2079 | `SARAVALI_UBHAYACHARI`<br>`SOLAR_UBHAYACHARI` |
| 2060 | `BJ_MARANA_UNCONSCIOUS_LAGNA_ASPECTED_BY_A_BENEFIC`<br>`PARIJATA_MARANA_UNCONSCIOUS_LAGNA_ASPECTED_BY_A_BENEFIC`<br>`SARAVALI_MARANA_UNCONSCIOUS_LAGNA_ASPECTED_BY_A_BENEFIC` |
| 2031 | `PARIJATA_AMALA`<br>`PHALADEEPIKA_AMALA` |
| 2018 | `BJ_MARANA_DUAL_EIGHTH_PLACE_OF_DEATH`<br>`BPHS_MOVABLE_THIRD_PLACE_OF_DEATH`<br>`PARIJATA_MARANA_DUAL_EIGHTH_PLACE_OF_DEATH`<br>`SARAVALI_MARANA_DUAL_EIGHTH_PLACE_OF_DEATH` |
| 2013 | `BJ_CHANDRA_MOON_IN_APOKLIMA_FROM_THE_SUN`<br>`LUNAR_MOON_IN_APOKLIMA_FROM_THE_SUN`<br>`PHALADEEPIKA_VARISHTHA` |
| 2013 | `BJ_CHANDRA_MOON_IN_KENDRA_FROM_THE_SUN`<br>`LUNAR_MOON_IN_KENDRA_FROM_THE_SUN`<br>`PHALADEEPIKA_ADHAMA` |
| 2007 | `BJ_LOKA_PAST_BIRTH_FROM_THE_NETHER_WORLD`<br>`BPHS_DESCENT_FROM_HELL`<br>`PARIJATA_LOKA_PAST_BIRTH_FROM_THE_NETHER_WORLD` |
| 2003 | `BJ_MARANA_MOVABLE_EIGHTH_PLACE_OF_DEATH`<br>`BPHS_FIXED_THIRD_PLACE_OF_DEATH`<br>`PARIJATA_MARANA_MOVABLE_EIGHTH_PLACE_OF_DEATH`<br>`SARAVALI_MARANA_MOVABLE_EIGHTH_PLACE_OF_DEATH` |
| 1988 | `BJ_STRI_MOON_IN_VIRGO_SCORPIO_TAURUS_OR_LEO_FEW_SONS`<br>`SARAVALI_STRI_MOON_IN_VIRGO_SCORPIO_TAURUS_OR_LEO_FEW_SONS` |
| 1974 | `BJ_CHANDRA_MOON_IN_PANAPHARA_FROM_THE_SUN`<br>`LUNAR_MOON_IN_PANAPHARA_FROM_THE_SUN`<br>`PHALADEEPIKA_SAMA` |
| 1898 | `BJ_STRI_SETTING_SIGN_OR_NAVAMSHA_OF_JUPITER`<br>`BPHS_STRI_SEVENTH_SIGN_OR_NAVAMSHA_OF_JUPITER`<br>`SARAVALI_STRI_SETTING_SIGN_OR_NAVAMSHA_OF_JUPITER` |
| 1892 | `BJ_STRI_SETTING_SIGN_OR_NAVAMSHA_OF_MARS`<br>`BPHS_STRI_SEVENTH_SIGN_OR_NAVAMSHA_OF_MARS`<br>`SARAVALI_STRI_SETTING_SIGN_OR_NAVAMSHA_OF_MARS` |
| 1856 | `BJ_STRI_SETTING_SIGN_OR_NAVAMSHA_OF_SATURN`<br>`BPHS_STRI_SEVENTH_SIGN_OR_NAVAMSHA_OF_SATURN`<br>`SARAVALI_STRI_SETTING_SIGN_OR_NAVAMSHA_OF_SATURN` |
| 1846 | `BJ_LOKA_AFTER_DEATH_TO_THE_NETHER_WORLD`<br>`PARIJATA_LOKA_AFTER_DEATH_TO_THE_NETHER_WORLD` |
| 1813 | `BJ_STRI_SETTING_SIGN_OR_NAVAMSHA_OF_VENUS`<br>`BPHS_STRI_SEVENTH_SIGN_OR_NAVAMSHA_OF_VENUS`<br>`SARAVALI_STRI_SETTING_SIGN_OR_NAVAMSHA_OF_VENUS` |
| 1798 | `BJ_STRI_SETTING_SIGN_OR_NAVAMSHA_OF_MERCURY`<br>`BPHS_STRI_SEVENTH_SIGN_OR_NAVAMSHA_OF_MERCURY`<br>`SARAVALI_STRI_SETTING_SIGN_OR_NAVAMSHA_OF_MERCURY` |
| 1793 | `ARISHTA_BHANGA_STRONG_LAGNA_LORD_IN_KENDRA`<br>`PHALADEEPIKA_MAHARAJA_STRONG_LAGNA_LORD_IN_A_KENDRA` |
| 1667 | `BJ_LOKA_PAST_BIRTH_FROM_THE_BEASTS`<br>`BPHS_DESCENT_FROM_YAMA`<br>`PARIJATA_LOKA_PAST_BIRTH_FROM_MORTALS` |
| 1622 | `BJ_NABHASA_KEDARA`<br>`NABHASA_KEDARA` |
| 1597 | `BJ_VARGA_ONE_IN_A_FRIENDS_SIGN`<br>`PARIJATA_DARIDRA_ONE_IN_FRIENDS_SIGNS` |
| 1542 | `BPHS_DHANA_SECOND_LORD_IN_A_DUSTHANA`<br>`PHALADEEPIKA_NISSVA` |
| 1518 | `BJ_ANISHTA_VENUS_HEMMED_BY_MALEFICS`<br>`PHALADEEPIKA_KALATRA_VENUS_HEMMED_BY_MALEFICS` |
| 1515 | `DWIGRAHA_MERCURY_VENUS`<br>`PARIJATA_TOGETHER_MERCURY_VENUS`<br>`SARAVALI_TOGETHER_MERCURY_VENUS` |
| 1503 | `BPHS_PUTRA_FIFTH_LORD_IN_A_DUSTHANA`<br>`PHALADEEPIKA_PAMARA` |
| 1492 | `BJ_STRI_LAGNA_AND_MOON_IN_ODD_SIGNS`<br>`BPHS_STRI_LAGNA_AND_MOON_IN_ODD_SIGNS`<br>`SARAVALI_STRI_LAGNA_AND_MOON_IN_ODD_SIGNS` |
| 1490 | `BJ_STRI_LAGNA_AND_MOON_IN_EVEN_SIGNS`<br>`BPHS_STRI_LAGNA_AND_MOON_IN_EVEN_SIGNS`<br>`SARAVALI_STRI_LAGNA_AND_MOON_IN_EVEN_SIGNS` |
| 1486 | `BJ_VARGA_TWO_IN_FRIENDS_SIGNS`<br>`PARIJATA_DHANA_TWO_IN_FRIENDS_SIGNS`<br>`SARAVALI_TWO_IN_FRIENDS_SIGNS` |
| 1462 | `BJ_LOKA_AFTER_DEATH_TO_THE_MANES`<br>`PARIJATA_LOKA_AFTER_DEATH_TO_THE_MANES` |
| 1397 | `BJ_LOKA_PAST_BIRTH_FROM_THE_MANES`<br>`BPHS_DESCENT_FROM_THE_MANES`<br>`PARIJATA_LOKA_PAST_BIRTH_FROM_THE_MANES` |
| 1380 | `DWIGRAHA_SUN_VENUS`<br>`PARIJATA_TOGETHER_SUN_VENUS`<br>`SARAVALI_TOGETHER_SUN_VENUS` |
| 1376 | `BJ_NABHASA_DAMINI`<br>`NABHASA_DAMA` |
| 1355 | `BJ_LOKA_AFTER_DEATH_TO_THE_GODS`<br>`PARIJATA_LOKA_AFTER_DEATH_TO_THE_GODS` |
| 1337 | `BJ_LOKA_AFTER_DEATH_TO_THE_BEASTS`<br>`PARIJATA_LOKA_AFTER_DEATH_TO_MORTALS` |
| 1334 | `BJ_CHANDRA_SUNAPHA`<br>`PHALADEEPIKA_SUNAPHA`<br>`SARAVALI_SUNAPHA` |
| 1331 | `BJ_CHANDRA_ANAPHA`<br>`PHALADEEPIKA_ANAPHA`<br>`SARAVALI_ANAPHA` |
| 1266 | `BPHS_TANU_BENEFIC_IN_THE_LAGNA`<br>`PARIJATA_SHUBHA` |
| 1240 | `BPHS_DHANA_BENEFIC_IN_THE_SECOND`<br>`SARAVALI_BENEFICS_IN_THE_SECOND` |
| 1118 | `BJ_VARGA_THREE_IN_FRIENDS_SIGNS`<br>`PARIJATA_DHANA_THREE_IN_FRIENDS_SIGNS`<br>`SARAVALI_THREE_IN_FRIENDS_SIGNS` |
| 1059 | `BJ_CHANDRA_TWO_BENEFICS_IN_UPACHAYA_FROM_THE_MOON`<br>`LUNAR_DHANA_TWO_BENEFICS_IN_UPACHAYA` |
| 1012 | `BJ_STRI_MALEFICS_SEVENTH_WIDOWED`<br>`SARAVALI_STRI_MALEFICS_SEVENTH_WIDOWED` |
| 994 | `BJ_STRI_JUPITER_MARS_VENUS_MERCURY_STRONG_EVEN_LAGNA`<br>`BPHS_STRI_MARS_MERCURY_JUPITER_AND_VENUS_STRONG_EVEN_LAGNA` |
| 983 | `PARASHARA_CHAMARA`<br>`PARIJATA_CHAMARA` |
| 945 | `BJ_STRI_SUN_SEVENTH_ABANDONED`<br>`SARAVALI_STRI_SUN_SEVENTH_ABANDONED` |
| 929 | `BJ_LOKA_PAST_BIRTH_FROM_THE_GODS`<br>`BPHS_DESCENT_FROM_THE_GODS`<br>`PARIJATA_LOKA_PAST_BIRTH_FROM_THE_GODS` |
| 925 | `BJ_MARANA_UNCONSCIOUS_LAGNA_ASPECTED_BY_ITS_LORD`<br>`PARIJATA_MARANA_UNCONSCIOUS_LAGNA_ASPECTED_BY_ITS_LORD`<br>`SARAVALI_MARANA_UNCONSCIOUS_LAGNA_ASPECTED_BY_ITS_LORD` |
| 907 | `BJ_STRI_SETTING_SIGN_OR_NAVAMSHA_OF_THE_MOON`<br>`BPHS_STRI_SEVENTH_SIGN_OR_NAVAMSHA_OF_THE_MOON`<br>`SARAVALI_STRI_SETTING_SIGN_OR_NAVAMSHA_OF_THE_MOON` |
| 857 | `BJ_STRI_SETTING_SIGN_OR_NAVAMSHA_OF_THE_SUN`<br>`BPHS_STRI_SEVENTH_SIGN_OR_NAVAMSHA_OF_THE_SUN`<br>`SARAVALI_STRI_SETTING_SIGN_OR_NAVAMSHA_OF_THE_SUN` |
| 831 | `BJ_ANISHTA_TAURUS_ARIES_OR_SAGITTARIUS_RISING_ASPECTED_BY_A_MALEFIC`<br>`PARIJATA_ROGA_ARIES_TAURUS_OR_SAGITTARIUS_RISING_ASPECTED_BY_A_MALEFIC` |
| 830 | `BJ_MARANA_EMPTY_EIGHTH_ASPECTED_BY_A_STRONG_MARS`<br>`PARIJATA_MARANA_EMPTY_EIGHTH_ASPECTED_BY_A_STRONG_MARS` |
| 826 | `DWIGRAHA_MARS_VENUS`<br>`PARIJATA_TOGETHER_MARS_VENUS`<br>`SARAVALI_TOGETHER_MARS_VENUS` |
| 826 | `DWIGRAHA_SUN_MARS`<br>`PARIJATA_TOGETHER_SUN_MARS`<br>`SARAVALI_TOGETHER_SUN_MARS` |
| 823 | `BPHS_BIRTH_IN_BHADRA`<br>`PARIJATA_BIRTH_IN_VISHTI` |
| 821 | `BJ_STRI_BENEFICS_AND_MALEFICS_SEVENTH_REMARRIES`<br>`SARAVALI_STRI_BENEFICS_AND_MALEFICS_SEVENTH_REMARRIES` |
| 812 | `DWIGRAHA_MARS_MERCURY`<br>`PARIJATA_TOGETHER_MARS_MERCURY`<br>`SARAVALI_TOGETHER_MARS_MERCURY` |
| 787 | `BJ_MARANA_TWENTY_SECOND_DREKKANA_A_SERPENT_OR_A_NOOSE_DEATH_IN_CAPTIVITY`<br>`SARAVALI_MARANA_TWENTY_SECOND_DREKKANA_A_SERPENT_NOOSE_OR_FETTER_CAPTIVITY` |
| 777 | `BJ_STRI_LAGNA_AND_MOON_IN_ODD_SIGNS_WITH_MALEFICS`<br>`SARAVALI_STRI_LAGNA_AND_MOON_IN_ODD_SIGNS_WITH_MALEFICS` |
| 777 | `PHALADEEPIKA_PRAVRAJYA_MOON_SIGN_LORD_ASPECTED_BY_SATURN_ALONE`<br>`SARAVALI_PRAVRAJYA_MOON_SIGN_LORD_ASPECTED_BY_SATURN_ALONE` |
| 753 | `BJ_MALEFICS_IN_TWELFTH_AND_SECOND`<br>`PARIJATA_PAPA_KARTARI`<br>`PHALADEEPIKA_PAPA_KARTARI` |
| 690 | `PARIJATA_TOGETHER_SUN_MERCURY_VENUS`<br>`SARAVALI_TOGETHER_SUN_MERCURY_VENUS` |
| 654 | `BJ_CHANDRA_DURUDHARA`<br>`LUNAR_DURADHARA`<br>`PHALADEEPIKA_DURUDHARA`<br>`SARAVALI_DURUDHARA` |
| 654 | `BJ_MARANA_EMPTY_EIGHTH_ASPECTED_BY_A_STRONG_JUPITER`<br>`PARIJATA_MARANA_EMPTY_EIGHTH_ASPECTED_BY_A_STRONG_JUPITER` |
| 642 | `BJ_JANMA_VARGOTTAMA_LAGNA_IN_THE_MOTHERS_OWN_HOUSE`<br>`PARIJATA_JANMA_VARGOTTAMA_LAGNA` |
| 634 | `BJ_VARGA_FOUR_IN_FRIENDS_SIGNS`<br>`PARIJATA_DHANA_FOUR_IN_FRIENDS_SIGNS`<br>`SARAVALI_FOUR_IN_FRIENDS_SIGNS` |
| 613 | `BJ_MARS_IN_LEO`<br>`SARAVALI_MARS_IN_LEO` |
| 601 | `DWIGRAHA_SUN_JUPITER`<br>`PARIJATA_TOGETHER_SUN_JUPITER`<br>`SARAVALI_TOGETHER_SUN_JUPITER` |
| 600 | `BJ_SUN_IN_GEMINI`<br>`SARAVALI_SUN_IN_GEMINI` |
| 600 | `BJ_SUN_IN_LEO`<br>`SARAVALI_SUN_IN_LEO` |
| 600 | `BJ_SUN_IN_LIBRA`<br>`SARAVALI_SUN_IN_LIBRA` |
| 596 | `DWIGRAHA_JUPITER_VENUS`<br>`PARIJATA_TOGETHER_JUPITER_VENUS`<br>`SARAVALI_TOGETHER_JUPITER_VENUS` |
| 592 | `DWIGRAHA_MERCURY_JUPITER`<br>`PARIJATA_TOGETHER_MERCURY_JUPITER`<br>`SARAVALI_TOGETHER_MERCURY_JUPITER` |
| 587 | `BJ_MARS_IN_CANCER`<br>`SARAVALI_MARS_IN_CANCER` |
| 581 | `BJ_MARANA_EMPTY_EIGHTH_ASPECTED_BY_A_STRONG_SATURN`<br>`PARIJATA_MARANA_EMPTY_EIGHTH_ASPECTED_BY_A_STRONG_SATURN` |
| 581 | `BJ_SUN_IN_SAGITTARIUS`<br>`SARAVALI_SUN_IN_SAGITTARIUS` |
| 580 | `BJ_MARS_IN_VIRGO`<br>`SARAVALI_MARS_IN_VIRGO` |
| 565 | `BJ_SATURN_IN_CAPRICORN`<br>`SARAVALI_SATURN_IN_CAPRICORN` |
| 558 | `BJ_SATURN_IN_SAGITTARIUS`<br>`SARAVALI_SATURN_IN_SAGITTARIUS` |
| 557 | `BJ_MERCURY_IN_SCORPIO`<br>`SARAVALI_MERCURY_IN_SCORPIO` |
| 555 | `DWIGRAHA_SUN_SATURN`<br>`PARIJATA_TOGETHER_SUN_SATURN`<br>`SARAVALI_TOGETHER_SUN_SATURN` |
| 554 | `BJ_STRI_SATURN_SEVENTH_ASPECTED_BY_A_MALEFIC_UNMARRIED`<br>`SARAVALI_STRI_SATURN_SEVENTH_ASPECTED_BY_A_MALEFIC_UNMARRIED` |
| 548 | `BJ_MARS_IN_GEMINI`<br>`SARAVALI_MARS_IN_GEMINI` |
| 547 | `BJ_JUPITER_IN_LEO`<br>`SARAVALI_JUPITER_IN_LEO` |
| 545 | `DWIGRAHA_MERCURY_SATURN`<br>`PARIJATA_TOGETHER_MERCURY_SATURN`<br>`SARAVALI_TOGETHER_MERCURY_SATURN` |
| 540 | `BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_SATURN_AND_A_TRIMSHAMSHA_OF_JUPITER`<br>`BPHS_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_SATURN_AND_A_TRIMSHAMSHA_OF_JUPITER`<br>`SARAVALI_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_SATURN_AND_A_TRIMSHAMSHA_OF_JUPITER` |
| 539 | `BJ_SUN_IN_PISCES`<br>`SARAVALI_SUN_IN_PISCES` |
| 538 | `BJ_MERCURY_IN_SAGITTARIUS`<br>`SARAVALI_MERCURY_IN_SAGITTARIUS` |
| 537 | `BJ_MERCURY_IN_LIBRA`<br>`SARAVALI_MERCURY_IN_LIBRA` |
| 536 | `BJ_MARS_IN_LIBRA`<br>`SARAVALI_MARS_IN_LIBRA` |
| 532 | `BJ_JUPITER_IN_CANCER`<br>`SARAVALI_JUPITER_IN_CANCER` |
| 531 | `BJ_LOKA_PAST_BIRTH_DREKKANA_LORD_EXALTED`<br>`PARIJATA_LOKA_PAST_BIRTH_DREKKANA_LORD_EXALTED` |
| 530 | `BJ_SATURN_IN_AQUARIUS`<br>`SARAVALI_SATURN_IN_AQUARIUS` |
| 530 | `DWIGRAHA_VENUS_SATURN`<br>`PARIJATA_TOGETHER_VENUS_SATURN`<br>`SARAVALI_TOGETHER_VENUS_SATURN` |
| 529 | `BJ_JUPITER_IN_VIRGO`<br>`SARAVALI_JUPITER_IN_VIRGO` |
| 529 | `BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MERCURY_AND_A_TRIMSHAMSHA_OF_JUPITER`<br>`BPHS_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MERCURY_AND_A_TRIMSHAMSHA_OF_JUPITER`<br>`SARAVALI_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MERCURY_AND_A_TRIMSHAMSHA_OF_JUPITER` |
| 527 | `BJ_LOKA_PAST_BIRTH_DREKKANA_LORD_DEBILITATED`<br>`PARIJATA_LOKA_PAST_BIRTH_DREKKANA_LORD_DEBILITATED` |
| 525 | `BJ_JUPITER_IN_SCORPIO`<br>`SARAVALI_JUPITER_IN_SCORPIO` |
| 525 | `BJ_VENUS_IN_TAURUS`<br>`SARAVALI_VENUS_IN_TAURUS` |
| 524 | `BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MERCURY_AND_A_TRIMSHAMSHA_OF_MERCURY`<br>`BPHS_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MERCURY_AND_A_TRIMSHAMSHA_OF_MERCURY`<br>`SARAVALI_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MERCURY_AND_A_TRIMSHAMSHA_OF_MERCURY` |
| 523 | `PHALADEEPIKA_PAST_BIRTH_LORD_SUN_LAND`<br>`SARAVALI_LAGNA_SAGITTARIUS` |
| 521 | `BJ_MERCURY_IN_VIRGO`<br>`SARAVALI_MERCURY_IN_VIRGO` |
| 521 | `BJ_VENUS_IN_CANCER`<br>`SARAVALI_VENUS_IN_CANCER` |
| 519 | `BJ_JUPITER_IN_LIBRA`<br>`SARAVALI_JUPITER_IN_LIBRA` |
| 519 | `BJ_VENUS_IN_LEO`<br>`SARAVALI_VENUS_IN_LEO` |
| 518 | `BJ_MARS_IN_BHAVA_10`<br>`SARAVALI_MARS_IN_BHAVA_10` |
| 518 | `BJ_MOON_IN_LIBRA`<br>`SARAVALI_MOON_IN_LIBRA` |
| 517 | `BJ_MERCURY_IN_BHAVA_10`<br>`SARAVALI_MERCURY_IN_BHAVA_10` |
| 517 | `BJ_MERCURY_IN_CAPRICORN`<br>`SARAVALI_MERCURY_IN_CAPRICORN` |
| 517 | `BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_VENUS_AND_A_TRIMSHAMSHA_OF_JUPITER`<br>`BPHS_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_VENUS_AND_A_TRIMSHAMSHA_OF_JUPITER`<br>`SARAVALI_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_VENUS_AND_A_TRIMSHAMSHA_OF_JUPITER` |
| 516 | `BJ_MOON_IN_ARIES`<br>`SARAVALI_MOON_IN_ARIES` |
| 516 | `BJ_MOON_IN_GEMINI`<br>`SARAVALI_MOON_IN_GEMINI` |
| 516 | `BJ_VENUS_IN_GEMINI`<br>`SARAVALI_VENUS_IN_GEMINI` |
| 515 | `BJ_JUPITER_IN_SAGITTARIUS`<br>`SARAVALI_JUPITER_IN_SAGITTARIUS` |
| 514 | `BJ_MARS_IN_BHAVA_1`<br>`SARAVALI_MARS_IN_BHAVA_1` |
| 513 | `BJ_VENUS_IN_PISCES`<br>`SARAVALI_VENUS_IN_PISCES` |
| 512 | `BJ_MARS_IN_BHAVA_11`<br>`SARAVALI_MARS_IN_BHAVA_11` |
| 512 | `BJ_MERCURY_IN_BHAVA_12`<br>`SARAVALI_MERCURY_IN_BHAVA_12` |
| 512 | `BJ_MOON_IN_SCORPIO`<br>`SARAVALI_MOON_IN_SCORPIO` |
| 512 | `DWIGRAHA_MARS_SATURN`<br>`PARIJATA_TOGETHER_MARS_SATURN`<br>`SARAVALI_TOGETHER_MARS_SATURN` |
| 511 | `BJ_MOON_IN_BHAVA_12`<br>`SARAVALI_MOON_IN_BHAVA_12` |
| 511 | `BJ_MOON_IN_BHAVA_9`<br>`SARAVALI_MOON_IN_BHAVA_9` |
| 511 | `BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_JUPITER_AND_A_TRIMSHAMSHA_OF_MERCURY`<br>`BPHS_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_JUPITER_AND_A_TRIMSHAMSHA_OF_MERCURY`<br>`SARAVALI_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_JUPITER_AND_A_TRIMSHAMSHA_OF_MERCURY` |
| 511 | `BJ_VENUS_IN_BHAVA_1`<br>`SARAVALI_VENUS_IN_BHAVA_1` |
| 510 | `BJ_MERCURY_IN_BHAVA_3`<br>`SARAVALI_MERCURY_IN_BHAVA_3` |
| 510 | `BJ_MOON_IN_SAGITTARIUS`<br>`SARAVALI_MOON_IN_SAGITTARIUS` |
| 509 | `BJ_MERCURY_IN_BHAVA_6`<br>`SARAVALI_MERCURY_IN_BHAVA_6` |
| 509 | `BJ_SATURN_IN_PISCES`<br>`SARAVALI_SATURN_IN_PISCES` |
| 508 | `BJ_VENUS_IN_BHAVA_9`<br>`SARAVALI_VENUS_IN_BHAVA_9` |
| 507 | `BJ_JUPITER_IN_BHAVA_10`<br>`SARAVALI_JUPITER_IN_BHAVA_10` |
| 507 | `BJ_JUPITER_IN_BHAVA_4`<br>`SARAVALI_JUPITER_IN_BHAVA_4` |
| 507 | `BJ_MARS_IN_BHAVA_12`<br>`SARAVALI_MARS_IN_BHAVA_12` |
| 507 | `BJ_MOON_IN_BHAVA_6`<br>`SARAVALI_MOON_IN_BHAVA_6` |
| 507 | `BJ_SATURN_IN_BHAVA_10`<br>`SARAVALI_SATURN_IN_BHAVA_10` |
| 507 | `BJ_SATURN_IN_BHAVA_5`<br>`SARAVALI_SATURN_IN_BHAVA_5` |
| 506 | `BJ_JUPITER_IN_BHAVA_11`<br>`SARAVALI_JUPITER_IN_BHAVA_11` |
| 506 | `BJ_JUPITER_IN_GEMINI`<br>`SARAVALI_JUPITER_IN_GEMINI` |
| 506 | `BJ_MARS_IN_BHAVA_2`<br>`SARAVALI_MARS_IN_BHAVA_2` |
| 506 | `BJ_MOON_IN_BHAVA_10`<br>`SARAVALI_MOON_IN_BHAVA_10` |
| 506 | `BJ_SATURN_IN_BHAVA_12`<br>`SARAVALI_SATURN_IN_BHAVA_12` |
| 506 | `BJ_SATURN_IN_VIRGO`<br>`SARAVALI_SATURN_IN_VIRGO` |
| 506 | `BJ_SUN_IN_BHAVA_10`<br>`SARAVALI_SUN_IN_BHAVA_10` |
| 506 | `BJ_VENUS_IN_BHAVA_11`<br>`SARAVALI_VENUS_IN_BHAVA_11` |
| 506 | `DWIGRAHA_MARS_JUPITER`<br>`PARIJATA_TOGETHER_MARS_JUPITER`<br>`SARAVALI_TOGETHER_MARS_JUPITER` |
| 505 | `BJ_JUPITER_IN_BHAVA_3`<br>`SARAVALI_JUPITER_IN_BHAVA_3` |
| 505 | `BJ_MERCURY_IN_AQUARIUS`<br>`SARAVALI_MERCURY_IN_AQUARIUS` |
| 505 | `BJ_MOON_IN_BHAVA_4`<br>`SARAVALI_MOON_IN_BHAVA_4` |
| 505 | `BJ_SATURN_IN_BHAVA_4`<br>`SARAVALI_SATURN_IN_BHAVA_4` |
| 505 | `BJ_SUN_IN_BHAVA_4`<br>`SARAVALI_SUN_IN_BHAVA_4` |
| 505 | `BJ_VENUS_IN_BHAVA_6`<br>`SARAVALI_VENUS_IN_BHAVA_6` |
| 505 | `BPHS_BHAVESHA_LAGNA_LORD_IN_THE_SIXTH`<br>`PHALADEEPIKA_BHAVA_LAGNA_LORD_IN_THE_SIXTH` |
| 505 | `BPHS_BHAVESHA_LAGNA_LORD_IN_THE_TENTH`<br>`PHALADEEPIKA_BHAVA_LAGNA_LORD_IN_THE_TENTH` |
| 504 | `BJ_MOON_IN_BHAVA_2`<br>`SARAVALI_MOON_IN_BHAVA_2` |
| 504 | `BJ_SATURN_IN_BHAVA_6`<br>`SARAVALI_SATURN_IN_BHAVA_6` |
| 504 | `BJ_SATURN_IN_BHAVA_7`<br>`SARAVALI_SATURN_IN_BHAVA_7` |
| 504 | `BJ_SUN_IN_BHAVA_9`<br>`SARAVALI_SUN_IN_BHAVA_9` |
| 504 | `BJ_VENUS_IN_LIBRA`<br>`SARAVALI_VENUS_IN_LIBRA` |
| 503 | `BJ_SUN_IN_BHAVA_2`<br>`SARAVALI_SUN_IN_BHAVA_2` |
| 503 | `BJ_SUN_IN_TAURUS`<br>`SARAVALI_SUN_IN_TAURUS` |
| 502 | `BJ_MERCURY_IN_BHAVA_2`<br>`SARAVALI_MERCURY_IN_BHAVA_2` |
| 502 | `BJ_SUN_IN_BHAVA_7`<br>`BPHS_STRI_SUN_SEVENTH`<br>`SARAVALI_SUN_IN_BHAVA_7` |
| 502 | `BJ_VENUS_IN_BHAVA_5`<br>`SARAVALI_VENUS_IN_BHAVA_5` |
| 502 | `DWIGRAHA_JUPITER_SATURN`<br>`PARIJATA_TOGETHER_JUPITER_SATURN`<br>`SARAVALI_TOGETHER_JUPITER_SATURN`<br>`UTTARA_PUTRA_SATURN_AND_JUPITER_TOGETHER` |
| 501 | `BJ_JUPITER_IN_BHAVA_12`<br>`SARAVALI_JUPITER_IN_BHAVA_12` |
| 501 | `BJ_JUPITER_IN_BHAVA_2`<br>`SARAVALI_JUPITER_IN_BHAVA_2` |
| 501 | `BJ_MOON_IN_BHAVA_7`<br>`SARAVALI_MOON_IN_BHAVA_7` |
| 501 | `BJ_SATURN_IN_BHAVA_2`<br>`SARAVALI_SATURN_IN_BHAVA_2` |
| 501 | `BJ_SUN_IN_BHAVA_5`<br>`SARAVALI_SUN_IN_BHAVA_5` |
| 501 | `BJ_VENUS_IN_BHAVA_4`<br>`SARAVALI_VENUS_IN_BHAVA_4` |
| 501 | `DWIGRAHA_MOON_MARS`<br>`PARIJATA_TOGETHER_MOON_MARS`<br>`SARAVALI_TOGETHER_MOON_MARS` |
| 501 | `DWIGRAHA_MOON_VENUS`<br>`PARIJATA_TOGETHER_MOON_VENUS`<br>`SARAVALI_TOGETHER_MOON_VENUS` |
| 500 | `BJ_JUPITER_IN_BHAVA_5`<br>`SARAVALI_JUPITER_IN_BHAVA_5` |
| 500 | `BJ_VENUS_IN_BHAVA_2`<br>`SARAVALI_VENUS_IN_BHAVA_2` |
| 500 | `BJ_VENUS_IN_CAPRICORN`<br>`SARAVALI_VENUS_IN_CAPRICORN` |
| 499 | `BJ_JUPITER_IN_BHAVA_6`<br>`SARAVALI_JUPITER_IN_BHAVA_6` |
| 498 | `BJ_MERCURY_IN_BHAVA_1`<br>`SARAVALI_MERCURY_IN_BHAVA_1` |
| 498 | `BJ_MOON_IN_BHAVA_3`<br>`BPHS_MOON_IN_THIRD_DEATH_BY_CONSUMPTION`<br>`SARAVALI_MOON_IN_BHAVA_3` |
| 498 | `DWIGRAHA_MOON_MERCURY`<br>`PARIJATA_TOGETHER_MOON_MERCURY`<br>`SARAVALI_TOGETHER_MOON_MERCURY` |
| 497 | `BJ_MERCURY_IN_BHAVA_5`<br>`SARAVALI_MERCURY_IN_BHAVA_5` |
| 497 | `BJ_MOON_IN_CAPRICORN`<br>`SARAVALI_MOON_IN_CAPRICORN` |
| 497 | `BJ_SATURN_IN_BHAVA_3`<br>`BPHS_SAHAJA_SATURN_IN_THE_THIRD`<br>`SARAVALI_SATURN_IN_BHAVA_3` |
| 497 | `BJ_SUN_IN_ARIES`<br>`SARAVALI_SUN_IN_ARIES` |
| 497 | `BJ_SUN_IN_BHAVA_1`<br>`SARAVALI_SUN_IN_BHAVA_1` |
| 497 | `BJ_SUN_IN_BHAVA_11`<br>`SARAVALI_SUN_IN_BHAVA_11` |
| 497 | `BJ_SUN_IN_BHAVA_12`<br>`SARAVALI_SUN_IN_BHAVA_12` |
| 496 | `BJ_JUPITER_IN_BHAVA_1`<br>`SARAVALI_JUPITER_IN_BHAVA_1` |
| 496 | `BJ_SATURN_IN_LEO`<br>`SARAVALI_SATURN_IN_LEO` |
| 496 | `BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_JUPITER_AND_A_TRIMSHAMSHA_OF_JUPITER`<br>`BPHS_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_JUPITER_AND_A_TRIMSHAMSHA_OF_JUPITER`<br>`SARAVALI_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_JUPITER_AND_A_TRIMSHAMSHA_OF_JUPITER` |
| 495 | `BJ_MERCURY_IN_LEO`<br>`SARAVALI_MERCURY_IN_LEO` |
| 495 | `BJ_VENUS_IN_BHAVA_10`<br>`SARAVALI_VENUS_IN_BHAVA_10` |
| 494 | `BJ_MARS_IN_TAURUS`<br>`SARAVALI_MARS_IN_TAURUS` |
| 494 | `BJ_MOON_IN_BHAVA_5`<br>`SARAVALI_MOON_IN_BHAVA_5` |
| 494 | `BJ_SUN_IN_BHAVA_6`<br>`PARIJATA_JANMA_KANINA_SUN_ASPECTS_THE_TWELFTH`<br>`SARAVALI_SUN_IN_BHAVA_6` |
| 493 | `BJ_MERCURY_IN_BHAVA_4`<br>`SARAVALI_MERCURY_IN_BHAVA_4` |
| 493 | `BJ_MOON_IN_BHAVA_11`<br>`SARAVALI_MOON_IN_BHAVA_11` |
| 492 | `BJ_MARS_IN_BHAVA_4`<br>`SARAVALI_MARS_IN_BHAVA_4` |
| 492 | `BJ_MARS_IN_BHAVA_6`<br>`SARAVALI_MARS_IN_BHAVA_6` |
| 492 | `BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MARS_AND_A_TRIMSHAMSHA_OF_JUPITER`<br>`BPHS_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MARS_AND_A_TRIMSHAMSHA_OF_JUPITER`<br>`SARAVALI_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MARS_AND_A_TRIMSHAMSHA_OF_JUPITER` |
| 492 | `BJ_VENUS_IN_ARIES`<br>`SARAVALI_VENUS_IN_ARIES` |
| 492 | `BPHS_BHAVESHA_SEVENTH_LORD_IN_THE_FIFTH`<br>`PHALADEEPIKA_KALATRA_SEVENTH_LORD_FIFTH` |
| 492 | `DWIGRAHA_MOON_SATURN`<br>`PARIJATA_TOGETHER_MOON_SATURN`<br>`SARAVALI_TOGETHER_MOON_SATURN` |
| 491 | `BJ_JUPITER_IN_BHAVA_9`<br>`SARAVALI_JUPITER_IN_BHAVA_9` |
| 491 | `BJ_MERCURY_IN_BHAVA_9`<br>`SARAVALI_MERCURY_IN_BHAVA_9` |
| 491 | `BJ_VENUS_IN_BHAVA_3`<br>`SARAVALI_VENUS_IN_BHAVA_3` |
| 490 | `BJ_JANMA_WATERY_SIGN_RISING_BY_THE_WATERS`<br>`PARIJATA_JANMA_WATERY_SIGN_RISING_BY_THE_WATERS` |
| 490 | `BJ_MOON_IN_CANCER`<br>`SARAVALI_MOON_IN_CANCER` |
| 489 | `BJ_MERCURY_IN_BHAVA_7`<br>`SARAVALI_MERCURY_IN_BHAVA_7` |
| 489 | `BJ_VENUS_IN_SAGITTARIUS`<br>`SARAVALI_VENUS_IN_SAGITTARIUS` |
| 488 | `BJ_SUN_IN_BHAVA_3`<br>`BPHS_SAHAJA_SUN_IN_THE_THIRD`<br>`SARAVALI_SUN_IN_BHAVA_3` |
| 488 | `BJ_VENUS_IN_BHAVA_12`<br>`SARAVALI_VENUS_IN_BHAVA_12` |
| 486 | `BJ_MOON_IN_VIRGO`<br>`SARAVALI_MOON_IN_VIRGO` |
| 485 | `BJ_MARS_IN_BHAVA_5`<br>`SARAVALI_MARS_IN_BHAVA_5` |
| 485 | `BJ_MOON_IN_BHAVA_1`<br>`SARAVALI_MOON_IN_BHAVA_1` |
| 485 | `BJ_MOON_IN_PISCES`<br>`SARAVALI_MOON_IN_PISCES` |
| 485 | `BJ_SATURN_IN_BHAVA_9`<br>`SARAVALI_SATURN_IN_BHAVA_9` |
| 484 | `BJ_JUPITER_IN_TAURUS`<br>`SARAVALI_JUPITER_IN_TAURUS` |
| 484 | `BJ_MARS_IN_SCORPIO`<br>`SARAVALI_MARS_IN_SCORPIO` |
| 484 | `BJ_VENUS_IN_AQUARIUS`<br>`SARAVALI_VENUS_IN_AQUARIUS` |
| 483 | `BJ_MERCURY_IN_BHAVA_11`<br>`SARAVALI_MERCURY_IN_BHAVA_11` |
| 483 | `BJ_SATURN_IN_BHAVA_11`<br>`SARAVALI_SATURN_IN_BHAVA_11` |
| 482 | `BJ_JUPITER_IN_BHAVA_7`<br>`SARAVALI_JUPITER_IN_BHAVA_7` |
| 482 | `BJ_MERCURY_IN_CANCER`<br>`SARAVALI_MERCURY_IN_CANCER` |
| 482 | `BJ_SATURN_IN_ARIES`<br>`SARAVALI_SATURN_IN_ARIES` |
| 482 | `DWIGRAHA_MOON_JUPITER`<br>`PARIJATA_TOGETHER_MOON_JUPITER`<br>`SARAVALI_TOGETHER_MOON_JUPITER` |
| 481 | `BJ_MERCURY_IN_PISCES`<br>`SARAVALI_MERCURY_IN_PISCES` |
| 480 | `BJ_JUPITER_IN_CAPRICORN`<br>`SARAVALI_JUPITER_IN_CAPRICORN` |
| 480 | `BJ_MOON_IN_AQUARIUS`<br>`SARAVALI_MOON_IN_AQUARIUS` |
| 480 | `BJ_SATURN_IN_SCORPIO`<br>`SARAVALI_SATURN_IN_SCORPIO` |
| 480 | `BJ_VENUS_IN_BHAVA_7`<br>`BPHS_KALATRA_VENUS_SEVENTH`<br>`SARAVALI_VENUS_IN_BHAVA_7` |
| 478 | `BJ_MARS_IN_BHAVA_9`<br>`SARAVALI_MARS_IN_BHAVA_9` |
| 478 | `BJ_MOON_IN_LEO`<br>`SARAVALI_MOON_IN_LEO` |
| 473 | `BJ_SATURN_IN_CANCER`<br>`SARAVALI_SATURN_IN_CANCER` |
| 471 | `BJ_SATURN_IN_LIBRA`<br>`SARAVALI_SATURN_IN_LIBRA` |
| 471 | `BJ_VENUS_IN_VIRGO`<br>`SARAVALI_VENUS_IN_VIRGO` |
| 470 | `BJ_SATURN_IN_TAURUS`<br>`SARAVALI_SATURN_IN_TAURUS` |
| 466 | `BJ_VENUS_IN_SCORPIO`<br>`SARAVALI_VENUS_IN_SCORPIO` |
| 465 | `BJ_MERCURY_IN_GEMINI`<br>`SARAVALI_MERCURY_IN_GEMINI` |
| 462 | `BJ_JUPITER_IN_ARIES`<br>`SARAVALI_JUPITER_IN_ARIES` |
| 461 | `BJ_SUN_IN_AQUARIUS`<br>`SARAVALI_SUN_IN_AQUARIUS` |
| 460 | `BJ_SATURN_IN_GEMINI`<br>`SARAVALI_SATURN_IN_GEMINI` |
| 457 | `BJ_JUPITER_IN_AQUARIUS`<br>`SARAVALI_JUPITER_IN_AQUARIUS` |
| 457 | `BJ_MARS_IN_ARIES`<br>`SARAVALI_MARS_IN_ARIES` |
| 452 | `BJ_MERCURY_IN_TAURUS`<br>`SARAVALI_MERCURY_IN_TAURUS` |
| 450 | `BJ_MERCURY_IN_ARIES`<br>`SARAVALI_MERCURY_IN_ARIES` |
| 444 | `BJ_JUPITER_IN_PISCES`<br>`SARAVALI_JUPITER_IN_PISCES` |
| 442 | `BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MARS_AND_A_TRIMSHAMSHA_OF_MERCURY`<br>`BPHS_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MARS_AND_A_TRIMSHAMSHA_OF_MERCURY`<br>`SARAVALI_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MARS_AND_A_TRIMSHAMSHA_OF_MERCURY` |
| 441 | `BJ_MARS_IN_SAGITTARIUS`<br>`SARAVALI_MARS_IN_SAGITTARIUS` |
| 433 | `BJ_MARS_IN_PISCES`<br>`SARAVALI_MARS_IN_PISCES` |
| 433 | `BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_VENUS_AND_A_TRIMSHAMSHA_OF_MERCURY`<br>`BPHS_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_VENUS_AND_A_TRIMSHAMSHA_OF_MERCURY`<br>`SARAVALI_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_VENUS_AND_A_TRIMSHAMSHA_OF_MERCURY` |
| 433 | `PARIJATA_TOGETHER_SUN_MARS_MERCURY`<br>`SARAVALI_TOGETHER_SUN_MARS_MERCURY` |
| 425 | `PARIJATA_CHANDRADHI`<br>`UTTARA_CHANDRADHI` |
| 422 | `BJ_MARS_IN_CAPRICORN`<br>`SARAVALI_MARS_IN_CAPRICORN` |
| 419 | `BJ_SUN_IN_CAPRICORN`<br>`SARAVALI_SUN_IN_CAPRICORN` |
| 405 | `BJ_MARS_IN_AQUARIUS`<br>`SARAVALI_MARS_IN_AQUARIUS` |
| 405 | `PHALADEEPIKA_NEXT_BIRTH_LORD_SUN_LAND`<br>`SARAVALI_LAGNA_ARIES` |
| 403 | `BJ_JANMA_SIGN_RISING_BOTH_WAYS`<br>`PARIJATA_JANMA_SIGN_RISING_BOTH_WAYS`<br>`SARAVALI_LAGNA_PISCES` |
| 400 | `BJ_SUN_IN_CANCER`<br>`SARAVALI_SUN_IN_CANCER` |
| 400 | `BJ_SUN_IN_SCORPIO`<br>`SARAVALI_SUN_IN_SCORPIO` |
| 400 | `BJ_SUN_IN_VIRGO`<br>`SARAVALI_SUN_IN_VIRGO` |
| 397 | `BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_SATURN_AND_A_TRIMSHAMSHA_OF_MERCURY`<br>`BPHS_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_SATURN_AND_A_TRIMSHAMSHA_OF_MERCURY`<br>`SARAVALI_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_SATURN_AND_A_TRIMSHAMSHA_OF_MERCURY` |
| 396 | `BJ_VARGA_AQUARIUS_RISING`<br>`SARAVALI_LAGNA_AQUARIUS` |
| 386 | `BJ_STRI_MALEFIC_EIGHTH_BENEFIC_SECOND_DIES_FIRST`<br>`SARAVALI_STRI_MALEFIC_EIGHTH_BENEFIC_SECOND_DIES_FIRST` |
| 379 | `BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_VENUS_AND_A_TRIMSHAMSHA_OF_MARS`<br>`BPHS_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_VENUS_AND_A_TRIMSHAMSHA_OF_MARS`<br>`SARAVALI_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_VENUS_AND_A_TRIMSHAMSHA_OF_MARS` |
| 378 | `BPHS_AYU_EIGHTH_LORD_IN_A_KENDRA_LAGNA_LORD_WEAK`<br>`PARIJATA_EIGHTH_LORD_IN_A_KENDRA_LAGNA_LORD_WEAK` |
| 367 | `BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_VENUS_AND_A_TRIMSHAMSHA_OF_VENUS`<br>`BPHS_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_VENUS_AND_A_TRIMSHAMSHA_OF_VENUS`<br>`SARAVALI_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_VENUS_AND_A_TRIMSHAMSHA_OF_VENUS` |
| 355 | `BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MARS_AND_A_TRIMSHAMSHA_OF_MARS`<br>`BPHS_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MARS_AND_A_TRIMSHAMSHA_OF_MARS`<br>`SARAVALI_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MARS_AND_A_TRIMSHAMSHA_OF_MARS` |
| 354 | `BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MARS_AND_A_TRIMSHAMSHA_OF_VENUS`<br>`BPHS_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MARS_AND_A_TRIMSHAMSHA_OF_VENUS`<br>`SARAVALI_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MARS_AND_A_TRIMSHAMSHA_OF_VENUS` |
| 353 | `BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_VENUS_AND_A_TRIMSHAMSHA_OF_SATURN`<br>`BPHS_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_VENUS_AND_A_TRIMSHAMSHA_OF_SATURN`<br>`SARAVALI_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_VENUS_AND_A_TRIMSHAMSHA_OF_SATURN` |
| 338 | `BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MERCURY_AND_A_TRIMSHAMSHA_OF_VENUS`<br>`BPHS_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MERCURY_AND_A_TRIMSHAMSHA_OF_VENUS`<br>`SARAVALI_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MERCURY_AND_A_TRIMSHAMSHA_OF_VENUS` |
| 337 | `BJ_ANISHTA_VENUS_AND_SUN_IN_A_TRINE_OR_THE_SEVENTH`<br>`PARIJATA_ANGA_VENUS_AND_SUN_IN_A_TRIKONA_OR_THE_SEVENTH` |
| 331 | `BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MARS_AND_A_TRIMSHAMSHA_OF_SATURN`<br>`BPHS_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MARS_AND_A_TRIMSHAMSHA_OF_SATURN`<br>`SARAVALI_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MARS_AND_A_TRIMSHAMSHA_OF_SATURN` |
| 328 | `BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MERCURY_AND_A_TRIMSHAMSHA_OF_MARS`<br>`BPHS_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MERCURY_AND_A_TRIMSHAMSHA_OF_MARS`<br>`SARAVALI_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MERCURY_AND_A_TRIMSHAMSHA_OF_MARS` |
| 320 | `BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MERCURY_AND_A_TRIMSHAMSHA_OF_SATURN`<br>`BPHS_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MERCURY_AND_A_TRIMSHAMSHA_OF_SATURN`<br>`SARAVALI_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MERCURY_AND_A_TRIMSHAMSHA_OF_SATURN` |
| 314 | `BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_JUPITER_AND_A_TRIMSHAMSHA_OF_MARS`<br>`BPHS_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_JUPITER_AND_A_TRIMSHAMSHA_OF_MARS`<br>`SARAVALI_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_JUPITER_AND_A_TRIMSHAMSHA_OF_MARS` |
| 313 | `BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_SATURN_AND_A_TRIMSHAMSHA_OF_SATURN`<br>`BPHS_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_SATURN_AND_A_TRIMSHAMSHA_OF_SATURN`<br>`SARAVALI_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_SATURN_AND_A_TRIMSHAMSHA_OF_SATURN` |
| 306 | `BJ_JANMA_FATHER_ABROAD_SUN_EIGHTH_OR_NINTH_IN_A_MOVABLE_SIGN`<br>`PARIJATA_JANMA_FATHER_AWAY_SUN_EIGHTH_OR_NINTH_IN_A_MOVABLE_SIGN` |
| 297 | `PARIJATA_TOGETHER_SUN_MERCURY_JUPITER`<br>`SARAVALI_TOGETHER_SUN_MERCURY_JUPITER` |
| 296 | `BJ_MARANA_EMPTY_EIGHTH_ASPECTED_BY_A_STRONG_SUN`<br>`PARIJATA_MARANA_EMPTY_EIGHTH_ASPECTED_BY_A_STRONG_SUN` |
| 292 | `BJ_STRI_LAGNA_OR_MOON_IN_LEO_AND_A_TRIMSHAMSHA_OF_JUPITER`<br>`BPHS_STRI_LAGNA_OR_MOON_IN_LEO_AND_A_TRIMSHAMSHA_OF_JUPITER`<br>`SARAVALI_STRI_LAGNA_OR_MOON_IN_LEO_AND_A_TRIMSHAMSHA_OF_JUPITER` |
| 289 | `BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_JUPITER_AND_A_TRIMSHAMSHA_OF_VENUS`<br>`BPHS_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_JUPITER_AND_A_TRIMSHAMSHA_OF_VENUS`<br>`SARAVALI_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_JUPITER_AND_A_TRIMSHAMSHA_OF_VENUS` |
| 288 | `BJ_STRI_LAGNA_OR_MOON_IN_CANCER_AND_A_TRIMSHAMSHA_OF_JUPITER`<br>`BPHS_STRI_LAGNA_OR_MOON_IN_CANCER_AND_A_TRIMSHAMSHA_OF_JUPITER`<br>`SARAVALI_STRI_LAGNA_OR_MOON_IN_CANCER_AND_A_TRIMSHAMSHA_OF_JUPITER` |
| 284 | `PARIJATA_TOGETHER_SUN_MERCURY_SATURN`<br>`SARAVALI_TOGETHER_SUN_MERCURY_SATURN` |
| 281 | `BJ_NABHASA_SHULA`<br>`NABHASA_SOOLA` |
| 279 | `BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_JUPITER_AND_A_TRIMSHAMSHA_OF_SATURN`<br>`BPHS_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_JUPITER_AND_A_TRIMSHAMSHA_OF_SATURN`<br>`SARAVALI_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_JUPITER_AND_A_TRIMSHAMSHA_OF_SATURN` |
| 278 | `BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_SATURN_AND_A_TRIMSHAMSHA_OF_VENUS`<br>`BPHS_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_SATURN_AND_A_TRIMSHAMSHA_OF_VENUS`<br>`SARAVALI_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_SATURN_AND_A_TRIMSHAMSHA_OF_VENUS` |
| 272 | `PARIJATA_BHANGA_STRONG_LAGNA_LORD_IN_A_KENDRA_BENEFICS_ONLY`<br>`PHALADEEPIKA_BHANGA_STRONG_LAGNA_LORD_IN_A_KENDRA_ASPECTED_BY_BENEFICS` |
| 264 | `ARISHTA_SUN_ASPECTED_BY_SATURN_IN_MARS_NAVAMSHA`<br>`BJ_SUN_IN_MARS_NAVAMSA_ASPECTED_BY_SATURN` |
| 264 | `BJ_MARANA_EMPTY_EIGHTH_ASPECTED_BY_A_STRONG_VENUS`<br>`PARIJATA_MARANA_EMPTY_EIGHTH_ASPECTED_BY_A_STRONG_VENUS` |
| 263 | `BJ_MOON_IN_MARS_NAVAMSA_ASPECTED_BY_SATURN`<br>`PHALADEEPIKA_PRAVRAJYA_MOON_IN_A_NAVAMSHA_OF_MARS_ASPECTED_BY_SATURN`<br>`SARAVALI_PRAVRAJYA_MOON_IN_A_NAVAMSHA_OF_MARS_ASPECTED_BY_SATURN` |
| 262 | `BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_SATURN_AND_A_TRIMSHAMSHA_OF_MARS`<br>`BPHS_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_SATURN_AND_A_TRIMSHAMSHA_OF_MARS`<br>`SARAVALI_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_SATURN_AND_A_TRIMSHAMSHA_OF_MARS` |
| 261 | `BJ_LOKA_MOKSHA_EXALTED_JUPITER_SIXTH_EIGHTH_OR_A_KENDRA`<br>`PARIJATA_LOKA_MOKSHA_EXALTED_JUPITER_SIXTH_EIGHTH_OR_A_KENDRA` |
| 259 | `BJ_STRI_LAGNA_AND_MOON_IN_EVEN_SIGNS_ASPECTED_BY_BENEFICS`<br>`BPHS_STRI_LAGNA_AND_MOON_IN_EVEN_SIGNS_ASPECTED_BY_BENEFICS`<br>`SARAVALI_STRI_LAGNA_AND_MOON_IN_EVEN_SIGNS_ASPECTED_BY_BENEFICS` |
| 258 | `PARIJATA_TOGETHER_SUN_MOON_MERCURY`<br>`SARAVALI_TOGETHER_SUN_MOON_MERCURY` |
| 256 | `BJ_MARANA_EMPTY_EIGHTH_ASPECTED_BY_A_STRONG_MOON`<br>`PARIJATA_MARANA_EMPTY_EIGHTH_ASPECTED_BY_A_STRONG_MOON` |
| 252 | `BJ_VARGA_FIVE_IN_FRIENDS_SIGNS`<br>`PARIJATA_RAJA_FIVE_IN_FRIENDS_SIGNS`<br>`SARAVALI_FIVE_IN_FRIENDS_SIGNS` |
| 249 | `BJ_STRI_WEAK_MALEFIC_SEVENTH_ASPECTED_BY_A_BENEFIC_REJECTED`<br>`SARAVALI_STRI_WEAK_MALEFIC_SEVENTH_ASPECTED_BY_A_BENEFIC_REJECTED` |
| 239 | `BJ_STRI_LAGNA_OR_MOON_IN_CANCER_AND_A_TRIMSHAMSHA_OF_MERCURY`<br>`BPHS_STRI_LAGNA_OR_MOON_IN_CANCER_AND_A_TRIMSHAMSHA_OF_MERCURY`<br>`SARAVALI_STRI_LAGNA_OR_MOON_IN_CANCER_AND_A_TRIMSHAMSHA_OF_MERCURY` |
| 238 | `BJ_MOON_IN_JYESHTHA`<br>`BPHS_GIRL_BORN_IN_JYESHTHA` |
| 236 | `BJ_JANMA_ARIES_LEO_OR_TAURUS_RISING_WITH_SATURN_OR_MARS_THE_CORD`<br>`BPHS_TANU_ARIES_TAURUS_OR_LEO_RISING_WITH_SATURN_OR_MARS_THE_CORD`<br>`PARIJATA_JANMA_CORD_ARIES_LEO_OR_TAURUS_RISING_WITH_SATURN_OR_MARS` |
| 232 | `BJ_STRI_LAGNA_OR_MOON_IN_LEO_AND_A_TRIMSHAMSHA_OF_MERCURY`<br>`BPHS_STRI_LAGNA_OR_MOON_IN_LEO_AND_A_TRIMSHAMSHA_OF_MERCURY`<br>`SARAVALI_STRI_LAGNA_OR_MOON_IN_LEO_AND_A_TRIMSHAMSHA_OF_MERCURY` |
| 232 | `BJ_STRI_SETTING_NAVAMSHA_OF_MARS_ASPECTED_BY_SATURN`<br>`BPHS_STRI_SEVENTH_NAVAMSHA_OF_MARS_ASPECTED_BY_SATURN` |
| 224 | `BJ_MARANA_EMPTY_EIGHTH_ASPECTED_BY_A_STRONG_MERCURY`<br>`PARIJATA_MARANA_EMPTY_EIGHTH_ASPECTED_BY_A_STRONG_MERCURY` |
| 220 | `BJ_CHANDRA_ALL_BENEFICS_IN_UPACHAYA_FROM_THE_MOON`<br>`LUNAR_DHANA_THREE_BENEFICS_IN_UPACHAYA` |
| 213 | `PARIJATA_SHUBHA_KARTARI`<br>`PHALADEEPIKA_SHUBHA_KARTARI` |
| 203 | `PARIJATA_TOGETHER_MARS_MERCURY_VENUS`<br>`SARAVALI_TOGETHER_MARS_MERCURY_VENUS` |
| 197 | `BPHS_BIRTH_ON_AMAVASYA`<br>`PARIJATA_BIRTH_ON_AMAVASYA` |
| 191 | `BJ_MOON_IN_PUSHYA`<br>`SARAVALI_RAJA_MOON_IN_PUSHYA` |
| 179 | `BJ_STRI_LAGNA_OR_MOON_IN_CANCER_AND_A_TRIMSHAMSHA_OF_MARS`<br>`BPHS_STRI_LAGNA_OR_MOON_IN_CANCER_AND_A_TRIMSHAMSHA_OF_MARS`<br>`SARAVALI_STRI_LAGNA_OR_MOON_IN_CANCER_AND_A_TRIMSHAMSHA_OF_MARS` |
| 176 | `PARIJATA_TOGETHER_SUN_MARS_VENUS`<br>`SARAVALI_TOGETHER_SUN_MARS_VENUS` |
| 175 | `BJ_CHANDRA_MOON_IN_OWN_OR_A_GREAT_FRIENDS_NAVAMSHA_ASPECTED_BY_JUPITER_BY_DAY_VENUS_BY_NIGHT`<br>`PARIJATA_MOON_IN_OWN_OR_A_GREAT_FRIENDS_NAVAMSHA_ASPECTED_BY_JUPITER_BY_DAY_VENUS_BY_NIGHT` |
| 174 | `BJ_STRI_LAGNA_OR_MOON_IN_LEO_AND_A_TRIMSHAMSHA_OF_SATURN`<br>`BPHS_STRI_LAGNA_OR_MOON_IN_LEO_AND_A_TRIMSHAMSHA_OF_SATURN`<br>`SARAVALI_STRI_LAGNA_OR_MOON_IN_LEO_AND_A_TRIMSHAMSHA_OF_SATURN` |
| 173 | `BJ_STRI_LAGNA_OR_MOON_IN_CANCER_AND_A_TRIMSHAMSHA_OF_VENUS`<br>`BPHS_STRI_LAGNA_OR_MOON_IN_CANCER_AND_A_TRIMSHAMSHA_OF_VENUS`<br>`SARAVALI_STRI_LAGNA_OR_MOON_IN_CANCER_AND_A_TRIMSHAMSHA_OF_VENUS` |
| 170 | `BJ_STRI_LAGNA_OR_MOON_IN_LEO_AND_A_TRIMSHAMSHA_OF_MARS`<br>`BPHS_STRI_LAGNA_OR_MOON_IN_LEO_AND_A_TRIMSHAMSHA_OF_MARS`<br>`SARAVALI_STRI_LAGNA_OR_MOON_IN_LEO_AND_A_TRIMSHAMSHA_OF_MARS` |
| 168 | `BJ_STRI_LAGNA_OR_MOON_IN_CANCER_AND_A_TRIMSHAMSHA_OF_SATURN`<br>`BPHS_STRI_LAGNA_OR_MOON_IN_CANCER_AND_A_TRIMSHAMSHA_OF_SATURN`<br>`SARAVALI_STRI_LAGNA_OR_MOON_IN_CANCER_AND_A_TRIMSHAMSHA_OF_SATURN` |
| 160 | `BJ_MOON_IN_ARIES_ASPECTED_BY_JUPITER`<br>`CHANDRA_IN_ARIES_ASPECTED_BY_JUPITER`<br>`SARAVALI_MOON_IN_ARIES_ASPECTED_BY_JUPITER` |
| 160 | `BJ_STRI_VENUS_AND_MARS_IN_EACH_OTHERS_NAVAMSHA_ADULTERY`<br>`BPHS_STRI_VENUS_AND_MARS_IN_EACH_OTHERS_NAVAMSHA`<br>`SARAVALI_STRI_VENUS_AND_MARS_IN_EACH_OTHERS_NAVAMSHA_ADULTERY` |
| 154 | `BJ_STRI_LAGNA_OR_MOON_IN_LEO_AND_A_TRIMSHAMSHA_OF_VENUS`<br>`BPHS_STRI_LAGNA_OR_MOON_IN_LEO_AND_A_TRIMSHAMSHA_OF_VENUS`<br>`SARAVALI_STRI_LAGNA_OR_MOON_IN_LEO_AND_A_TRIMSHAMSHA_OF_VENUS` |
| 154 | `PARIJATA_TOGETHER_MERCURY_JUPITER_VENUS`<br>`SARAVALI_TOGETHER_MERCURY_JUPITER_VENUS` |
| 150 | `BJ_NABHASA_VALLAKI`<br>`NABHASA_VEENA` |
| 147 | `BJ_MARANA_MALEFICS_FIFTH_AND_NINTH_UNASPECTED_DEATH_IN_CAPTIVITY`<br>`PARIJATA_MARANA_MALEFICS_FIFTH_AND_NINTH_UNASPECTED_DEATH_IN_CAPTIVITY` |
| 145 | `BJ_MOON_IN_CAPRICORN_ASPECTED_BY_JUPITER`<br>`CHANDRA_IN_CAPRICORN_ASPECTED_BY_JUPITER`<br>`SARAVALI_MOON_IN_CAPRICORN_ASPECTED_BY_JUPITER` |
| 145 | `PARIJATA_TOGETHER_MERCURY_VENUS_SATURN`<br>`SARAVALI_TOGETHER_MERCURY_VENUS_SATURN` |
| 143 | `BJ_MOON_IN_GEMINI_ASPECTED_BY_SATURN`<br>`CHANDRA_IN_GEMINI_ASPECTED_BY_SATURN`<br>`SARAVALI_MOON_IN_GEMINI_ASPECTED_BY_SATURN` |
| 142 | `PARIJATA_TOGETHER_SUN_JUPITER_VENUS`<br>`SARAVALI_TOGETHER_SUN_JUPITER_VENUS` |
| 141 | `BJ_MOON_IN_LIBRA_ASPECTED_BY_SATURN`<br>`CHANDRA_IN_LIBRA_ASPECTED_BY_SATURN`<br>`SARAVALI_MOON_IN_LIBRA_ASPECTED_BY_SATURN` |
| 139 | `BJ_MOON_IN_SAGITTARIUS_ASPECTED_BY_MARS`<br>`CHANDRA_IN_SAGITTARIUS_ASPECTED_BY_MARS`<br>`SARAVALI_MOON_IN_SAGITTARIUS_ASPECTED_BY_MARS` |
| 136 | `BJ_MOON_IN_GEMINI_ASPECTED_BY_JUPITER`<br>`CHANDRA_IN_GEMINI_ASPECTED_BY_JUPITER`<br>`SARAVALI_MOON_IN_GEMINI_ASPECTED_BY_JUPITER` |
| 135 | `BJ_MOON_IN_TAURUS_ASPECTED_BY_SATURN`<br>`CHANDRA_IN_TAURUS_ASPECTED_BY_SATURN`<br>`SARAVALI_MOON_IN_TAURUS_ASPECTED_BY_SATURN` |
| 134 | `BJ_MOON_IN_SCORPIO_ASPECTED_BY_MARS`<br>`CHANDRA_IN_SCORPIO_ASPECTED_BY_MARS`<br>`SARAVALI_MOON_IN_SCORPIO_ASPECTED_BY_MARS` |
| 133 | `BJ_MOON_IN_SCORPIO_ASPECTED_BY_SATURN`<br>`CHANDRA_IN_SCORPIO_ASPECTED_BY_SATURN`<br>`SARAVALI_MOON_IN_SCORPIO_ASPECTED_BY_SATURN` |
| 132 | `BJ_MOON_IN_CAPRICORN_ASPECTED_BY_MARS`<br>`CHANDRA_IN_CAPRICORN_ASPECTED_BY_MARS`<br>`SARAVALI_MOON_IN_CAPRICORN_ASPECTED_BY_MARS` |
| 131 | `BJ_MOON_IN_AQUARIUS_ASPECTED_BY_MARS`<br>`CHANDRA_IN_AQUARIUS_ASPECTED_BY_MARS`<br>`SARAVALI_MOON_IN_AQUARIUS_ASPECTED_BY_MARS` |
| 130 | `BJ_STRI_TAURUS_OR_LIBRA_RISING_IN_AN_AQUARIUS_NAVAMSHA`<br>`BPHS_STRI_SIGN_OF_VENUS_RISING_IN_AN_AQUARIUS_NAVAMSHA`<br>`SARAVALI_STRI_TAURUS_OR_LIBRA_RISING_IN_AN_AQUARIUS_NAVAMSHA` |
| 128 | `BJ_MOON_IN_ARIES_ASPECTED_BY_MARS`<br>`CHANDRA_IN_ARIES_ASPECTED_BY_MARS`<br>`SARAVALI_MOON_IN_ARIES_ASPECTED_BY_MARS` |
| 128 | `BJ_MOON_IN_ARIES_ASPECTED_BY_SATURN`<br>`CHANDRA_IN_ARIES_ASPECTED_BY_SATURN`<br>`SARAVALI_MOON_IN_ARIES_ASPECTED_BY_SATURN` |
| 127 | `BJ_MOON_IN_PISCES_ASPECTED_BY_JUPITER`<br>`CHANDRA_IN_PISCES_ASPECTED_BY_JUPITER`<br>`SARAVALI_MOON_IN_PISCES_ASPECTED_BY_JUPITER` |
| 125 | `BJ_MOON_IN_GEMINI_ASPECTED_BY_MARS`<br>`CHANDRA_IN_GEMINI_ASPECTED_BY_MARS`<br>`SARAVALI_MOON_IN_GEMINI_ASPECTED_BY_MARS` |
| 125 | `BJ_MOON_IN_LIBRA_ASPECTED_BY_JUPITER`<br>`CHANDRA_IN_LIBRA_ASPECTED_BY_JUPITER`<br>`SARAVALI_MOON_IN_LIBRA_ASPECTED_BY_JUPITER` |
| 125 | `BJ_MOON_IN_PISCES_ASPECTED_BY_MARS`<br>`CHANDRA_IN_PISCES_ASPECTED_BY_MARS`<br>`SARAVALI_MOON_IN_PISCES_ASPECTED_BY_MARS` |
| 123 | `BJ_MOON_IN_SAGITTARIUS_ASPECTED_BY_JUPITER`<br>`CHANDRA_IN_SAGITTARIUS_ASPECTED_BY_JUPITER`<br>`SARAVALI_MOON_IN_SAGITTARIUS_ASPECTED_BY_JUPITER` |
| 123 | `BJ_MOON_IN_SCORPIO_ASPECTED_BY_JUPITER`<br>`CHANDRA_IN_SCORPIO_ASPECTED_BY_JUPITER`<br>`SARAVALI_MOON_IN_SCORPIO_ASPECTED_BY_JUPITER` |
| 122 | `BJ_MOON_IN_AQUARIUS_ASPECTED_BY_SATURN`<br>`CHANDRA_IN_AQUARIUS_ASPECTED_BY_SATURN`<br>`SARAVALI_MOON_IN_AQUARIUS_ASPECTED_BY_SATURN` |
| 122 | `BJ_MOON_IN_CAPRICORN_ASPECTED_BY_SATURN`<br>`CHANDRA_IN_CAPRICORN_ASPECTED_BY_SATURN`<br>`SARAVALI_MOON_IN_CAPRICORN_ASPECTED_BY_SATURN` |
| 122 | `BJ_MOON_IN_LEO_ASPECTED_BY_SATURN`<br>`CHANDRA_IN_LEO_ASPECTED_BY_SATURN`<br>`SARAVALI_MOON_IN_LEO_ASPECTED_BY_SATURN` |
| 122 | `BJ_MOON_IN_PISCES_ASPECTED_BY_SATURN`<br>`CHANDRA_IN_PISCES_ASPECTED_BY_SATURN`<br>`SARAVALI_MOON_IN_PISCES_ASPECTED_BY_SATURN` |
| 122 | `BJ_MOON_IN_VIRGO_ASPECTED_BY_JUPITER`<br>`CHANDRA_IN_VIRGO_ASPECTED_BY_JUPITER`<br>`SARAVALI_MOON_IN_VIRGO_ASPECTED_BY_JUPITER` |
| 122 | `PARIJATA_TOGETHER_MOON_MERCURY_VENUS`<br>`SARAVALI_TOGETHER_MOON_MERCURY_VENUS` |
| 122 | `PARIJATA_TOGETHER_SUN_VENUS_SATURN`<br>`SARAVALI_TOGETHER_SUN_VENUS_SATURN` |
| 121 | `BJ_MOON_IN_LIBRA_ASPECTED_BY_MARS`<br>`CHANDRA_IN_LIBRA_ASPECTED_BY_MARS`<br>`SARAVALI_MOON_IN_LIBRA_ASPECTED_BY_MARS` |
| 120 | `BJ_MOON_IN_CANCER_ASPECTED_BY_SATURN`<br>`CHANDRA_IN_CANCER_ASPECTED_BY_SATURN`<br>`SARAVALI_MOON_IN_CANCER_ASPECTED_BY_SATURN` |
| 119 | `BJ_MOON_IN_VIRGO_ASPECTED_BY_MARS`<br>`CHANDRA_IN_VIRGO_ASPECTED_BY_MARS`<br>`SARAVALI_MOON_IN_VIRGO_ASPECTED_BY_MARS` |
| 119 | `BJ_MOON_IN_VIRGO_ASPECTED_BY_SATURN`<br>`CHANDRA_IN_VIRGO_ASPECTED_BY_SATURN`<br>`SARAVALI_MOON_IN_VIRGO_ASPECTED_BY_SATURN` |
| 118 | `BJ_MOON_IN_CANCER_ASPECTED_BY_JUPITER`<br>`CHANDRA_IN_CANCER_ASPECTED_BY_JUPITER`<br>`SARAVALI_MOON_IN_CANCER_ASPECTED_BY_JUPITER` |
| 118 | `BJ_MOON_IN_SAGITTARIUS_ASPECTED_BY_SATURN`<br>`CHANDRA_IN_SAGITTARIUS_ASPECTED_BY_SATURN`<br>`SARAVALI_MOON_IN_SAGITTARIUS_ASPECTED_BY_SATURN` |
| 115 | `BJ_MOON_IN_TAURUS_ASPECTED_BY_MARS`<br>`CHANDRA_IN_TAURUS_ASPECTED_BY_MARS`<br>`SARAVALI_MOON_IN_TAURUS_ASPECTED_BY_MARS` |
| 115 | `PHALADEEPIKA_MAHARAJA_SUN_AND_VENUS_FOURTH`<br>`SARAVALI_SUN_VENUS_IN_BHAVA_4` |
| 114 | `BJ_ANISHTA_SATURN_RISING_MARS_IN_A_TRINE_OR_THE_SEVENTH`<br>`PARIJATA_ROGA_SATURN_RISING_MARS_SEVENTH_OR_IN_A_TRIKONA` |
| 114 | `BJ_MOON_IN_TAURUS_ASPECTED_BY_JUPITER`<br>`CHANDRA_IN_TAURUS_ASPECTED_BY_JUPITER`<br>`SARAVALI_MOON_IN_TAURUS_ASPECTED_BY_JUPITER` |
| 113 | `BJ_PRAVRAJYA_MOON_LORD_ASPECTS_SATURN_AND_IS_STRONGER`<br>`PRAVRAJYA_MOON_LORD_ASPECTS_SATURN_AND_IS_STRONGER` |
| 113 | `PARIJATA_TOGETHER_SUN_MOON_VENUS`<br>`SARAVALI_TOGETHER_SUN_MOON_VENUS` |
| 112 | `BJ_MOON_IN_LEO_ASPECTED_BY_JUPITER`<br>`CHANDRA_IN_LEO_ASPECTED_BY_JUPITER`<br>`SARAVALI_MOON_IN_LEO_ASPECTED_BY_JUPITER` |
| 111 | `BJ_MARANA_MOON_IN_A_SIGN_OF_MARS_BETWEEN_MALEFICS`<br>`PARIJATA_MARANA_MOON_IN_A_SIGN_OF_MARS_BETWEEN_MALEFICS` |
| 110 | `BJ_MOON_IN_CANCER_ASPECTED_BY_MARS`<br>`CHANDRA_IN_CANCER_ASPECTED_BY_MARS`<br>`SARAVALI_MOON_IN_CANCER_ASPECTED_BY_MARS` |
| 104 | `BJ_MOON_IN_AQUARIUS_ASPECTED_BY_JUPITER`<br>`CHANDRA_IN_AQUARIUS_ASPECTED_BY_JUPITER`<br>`SARAVALI_MOON_IN_AQUARIUS_ASPECTED_BY_JUPITER` |
| 104 | `BJ_MOON_IN_LEO_ASPECTED_BY_MARS`<br>`CHANDRA_IN_LEO_ASPECTED_BY_MARS`<br>`SARAVALI_MOON_IN_LEO_ASPECTED_BY_MARS` |
| 101 | `PARIJATA_JANMA_FATHER_AWAY_MOON_BETWEEN_MERCURY_AND_VENUS`<br>`SARAVALI_DURUDHARA_MERCURY_VENUS` |
| 100 | `BJ_MARANA_MOON_IN_A_SIGN_OF_SATURN_BETWEEN_MALEFICS`<br>`PARIJATA_MARANA_MOON_IN_A_SIGN_OF_SATURN_BETWEEN_MALEFICS`<br>`SARAVALI_MARANA_MOON_IN_A_SIGN_OF_SATURN_BETWEEN_MALEFICS` |
| 96 | `PARIJATA_TOGETHER_MARS_MERCURY_JUPITER`<br>`SARAVALI_TOGETHER_MARS_MERCURY_JUPITER` |
| 91 | `BPHS_LAGNA_LORD_ANGULAR_WITH_JUPITER_AND_VENUS_LONG`<br>`PARIJATA_LAGNA_LORD_IN_A_KENDRA_WITH_JUPITER_AND_VENUS` |
| 90 | `BJ_JANMA_WATERY_SIGN_RISING_MOON_SEVENTH_IN_A_BOAT`<br>`PARIJATA_JANMA_BOAT_WATERY_SIGN_RISING_MOON_SEVENTH` |
| 84 | `PARIJATA_TOGETHER_SUN_MARS_JUPITER`<br>`SARAVALI_TOGETHER_SUN_MARS_JUPITER` |
| 83 | `BJ_STRI_MERCURY_AND_SATURN_SEVENTH_HUSBAND_IMPOTENT`<br>`SARAVALI_STRI_MERCURY_AND_SATURN_SEVENTH_HUSBAND_IMPOTENT` |
| 83 | `PARIJATA_TOGETHER_MARS_JUPITER_VENUS`<br>`SARAVALI_TOGETHER_MARS_JUPITER_VENUS` |
| 82 | `PARIJATA_TOGETHER_MOON_MARS_VENUS`<br>`SARAVALI_TOGETHER_MOON_MARS_VENUS` |
| 79 | `BJ_MARANA_SUN_OR_MARS_FOURTH_SATURN_TENTH_IMPALED`<br>`PARIJATA_MARANA_SUN_OR_MARS_FOURTH_SATURN_TENTH_IMPALED`<br>`SARAVALI_MARANA_SUN_OR_MARS_FOURTH_SATURN_TENTH_IMPALED` |
| 77 | `BJ_ANISHTA_LUMINARIES_SIXTH_AND_TWELFTH`<br>`PARIJATA_ANGA_LUMINARIES_SIXTH_AND_TWELFTH` |
| 77 | `BJ_ANISHTA_LUMINARIES_TOGETHER_IN_CANCER_OR_LEO`<br>`PARIJATA_ROGA_LUMINARIES_IN_CANCER_OR_LEO` |
| 75 | `PARIJATA_TOGETHER_MARS_VENUS_SATURN`<br>`SARAVALI_TOGETHER_MARS_VENUS_SATURN` |
| 73 | `PARIJATA_TOGETHER_MARS_MERCURY_SATURN`<br>`SARAVALI_TOGETHER_MARS_MERCURY_SATURN` |
| 72 | `BJ_VARGA_CANCER_NAVAMSHA_RISING_VARGOTTAMA`<br>`SARAVALI_NAVAMSA_1_OF_CANCER` |
| 72 | `PARIJATA_TOGETHER_SUN_MOON_MARS`<br>`SARAVALI_TOGETHER_SUN_MOON_MARS` |
| 70 | `PARIJATA_TOGETHER_SUN_MARS_SATURN`<br>`SARAVALI_TOGETHER_SUN_MARS_SATURN` |
| 69 | `PARIJATA_TOGETHER_SUN_MERCURY_JUPITER_VENUS`<br>`SARAVALI_TOGETHER_SUN_MERCURY_JUPITER_VENUS` |
| 67 | `PHALADEEPIKA_PRAVRAJYA_MOON_IN_SATURNS_DREKKANA_ASPECTED_BY_MARS_AND_SATURN`<br>`SARAVALI_PRAVRAJYA_MOON_IN_SATURNS_DREKKANA_ASPECTED_BY_MARS_AND_SATURN` |
| 66 | `BJ_VARGA_SCORPIO_NAVAMSHA_RISING_VARGOTTAMA`<br>`SARAVALI_NAVAMSA_5_OF_SCORPIO` |
| 64 | `PARIJATA_TOGETHER_SUN_MERCURY_VENUS_SATURN`<br>`SARAVALI_TOGETHER_SUN_MERCURY_VENUS_SATURN` |
| 63 | `PARIJATA_TOGETHER_MOON_MARS_MERCURY`<br>`SARAVALI_TOGETHER_MOON_MARS_MERCURY` |
| 62 | `BJ_MARANA_MOON_IN_VIRGO_BETWEEN_MALEFICS`<br>`PARIJATA_MARANA_MOON_IN_VIRGO_BETWEEN_MALEFICS`<br>`SARAVALI_MARANA_MOON_IN_VIRGO_BETWEEN_MALEFICS` |
| 62 | `BJ_VARGA_LIBRA_NAVAMSHA_RISING_VARGOTTAMA`<br>`SARAVALI_NAVAMSA_1_OF_LIBRA` |
| 62 | `PARIJATA_TOGETHER_JUPITER_VENUS_SATURN`<br>`SARAVALI_TOGETHER_JUPITER_VENUS_SATURN` |
| 59 | `PARIJATA_RAJA_JUPITER_SECOND_WITH_VENUS`<br>`SARAVALI_RAJA_JUPITER_SECOND_WITH_VENUS` |
| 58 | `BJ_VARGA_LEO_NAVAMSHA_RISING_VARGOTTAMA`<br>`SARAVALI_NAVAMSA_5_OF_LEO` |
| 56 | `BJ_SUN_IN_BHAVA_1_LIBRA_RISING`<br>`PARIJATA_ANGA_SUN_RISING_IN_LIBRA` |
| 56 | `PARIJATA_TOGETHER_SUN_JUPITER_SATURN`<br>`SARAVALI_TOGETHER_SUN_JUPITER_SATURN` |
| 55 | `PARIJATA_TOGETHER_SUN_MOON_MERCURY_VENUS`<br>`SARAVALI_TOGETHER_SUN_MOON_MERCURY_VENUS` |
| 54 | `BJ_JANMA_BIPED_SIGN_RISING_WITH_SATURN_ASPECTED_BY_JUPITER`<br>`PARIJATA_JANMA_HUMAN_SIGN_RISING_WITH_SATURN_ASPECTED_BY_JUPITER` |
| 54 | `BJ_JANMA_BIPED_SIGN_RISING_WITH_SATURN_ASPECTED_BY_MARS`<br>`PARIJATA_JANMA_HUMAN_SIGN_RISING_WITH_SATURN_ASPECTED_BY_MARS` |
| 54 | `BJ_SUN_IN_BHAVA_1_LEO_RISING`<br>`PARIJATA_ANGA_SUN_RISING_IN_LEO` |
| 54 | `BJ_VARGA_GEMINI_NAVAMSHA_RISING_VARGOTTAMA`<br>`SARAVALI_NAVAMSA_9_OF_GEMINI` |
| 53 | `BJ_VARGA_VIRGO_NAVAMSHA_RISING_VARGOTTAMA`<br>`SARAVALI_NAVAMSA_9_OF_VIRGO` |
| 52 | `BJ_MARANA_WANING_MOON_ASPECTED_BY_STRONG_MARS_SATURN_EIGHTH_PILES`<br>`PARIJATA_MARANA_WANING_MOON_ASPECTED_BY_STRONG_MARS_SATURN_EIGHTH_PILES`<br>`SARAVALI_MARANA_WANING_MOON_ASPECTED_BY_STRONG_MARS_SATURN_EIGHTH` |
| 50 | `BJ_VARGA_TAURUS_NAVAMSHA_RISING_VARGOTTAMA`<br>`SARAVALI_NAVAMSA_5_OF_TAURUS` |
| 50 | `PHALADEEPIKA_MAHARAJA_NINTH_AND_TENTH_LORDS_EXCHANGED`<br>`UTTARA_RAJA_NINTH_AND_TENTH_LORDS_EXCHANGED` |
| 49 | `BJ_VARGA_CAPRICORN_NAVAMSHA_RISING_VARGOTTAMA`<br>`SARAVALI_NAVAMSA_1_OF_CAPRICORN` |
| 49 | `PARIJATA_TOGETHER_MOON_JUPITER_VENUS`<br>`SARAVALI_TOGETHER_MOON_JUPITER_VENUS` |
| 48 | `PARIJATA_TOGETHER_MERCURY_JUPITER_SATURN`<br>`SARAVALI_TOGETHER_MERCURY_JUPITER_SATURN` |
| 47 | `BJ_VARGA_PISCES_NAVAMSHA_RISING_VARGOTTAMA`<br>`SARAVALI_NAVAMSA_9_OF_PISCES` |
| 45 | `PARIJATA_TOGETHER_SUN_MOON_JUPITER`<br>`SARAVALI_TOGETHER_SUN_MOON_JUPITER` |
| 45 | `PHALADEEPIKA_KALATRA_MOON_AND_SATURN_SEVENTH`<br>`SARAVALI_MOON_SATURN_IN_BHAVA_7` |
| 44 | `BJ_VARGA_AQUARIUS_NAVAMSHA_RISING_VARGOTTAMA`<br>`SARAVALI_NAVAMSA_5_OF_AQUARIUS` |
| 44 | `BJ_VARGA_ARIES_NAVAMSHA_RISING_VARGOTTAMA`<br>`SARAVALI_NAVAMSA_1_OF_ARIES` |
| 44 | `BJ_VARGA_SIX_IN_FRIENDS_SIGNS`<br>`PARIJATA_RAJA_SIX_IN_FRIENDS_SIGNS`<br>`SARAVALI_SIX_IN_FRIENDS_SIGNS` |
| 43 | `BJ_VARGA_SAGITTARIUS_NAVAMSHA_RISING_VARGOTTAMA`<br>`SARAVALI_NAVAMSA_9_OF_SAGITTARIUS` |
| 43 | `PARIJATA_RAJA_MARS_RISING_IN_FIRE_ASPECTED_BY_A_FRIEND`<br>`PHALADEEPIKA_MAHARAJA_MARS_RISING_IN_FIRE` |
| 43 | `PARIJATA_TOGETHER_MARS_JUPITER_SATURN`<br>`SARAVALI_TOGETHER_MARS_JUPITER_SATURN` |
| 43 | `PARIJATA_TOGETHER_MOON_MARS_JUPITER`<br>`SARAVALI_TOGETHER_MOON_MARS_JUPITER` |
| 42 | `BJ_JANMA_MOON_RISING_ASPECTED_BY_A_MALEFIC_MARS_SEVENTH_DESERTED_AND_DIES`<br>`PARIJATA_JANMA_MOON_RISING_ASPECTED_BY_A_MALEFIC_MARS_SEVENTH` |
| 42 | `BJ_PRAVRAJYA_MERCURY`<br>`PRAVRAJYA_MERCURY`<br>`SARAVALI_PRAVRAJYA_MERCURY_STRONGEST` |
| 42 | `BPHS_STRI_MERCURY_AND_SATURN_SEVENTH`<br>`SARAVALI_MERCURY_SATURN_IN_BHAVA_7` |
| 42 | `PARIJATA_TOGETHER_MOON_VENUS_SATURN`<br>`SARAVALI_TOGETHER_MOON_VENUS_SATURN` |
| 41 | `PARIJATA_TOGETHER_MOON_MERCURY_SATURN`<br>`SARAVALI_TOGETHER_MOON_MERCURY_SATURN` |
| 40 | `PARIJATA_TOGETHER_MOON_MERCURY_JUPITER`<br>`SARAVALI_TOGETHER_MOON_MERCURY_JUPITER` |
| 39 | `PARIJATA_TOGETHER_MOON_MARS_SATURN`<br>`SARAVALI_TOGETHER_MOON_MARS_SATURN` |
| 37 | `BJ_PRAVRAJYA_MOON_LORD_ASPECTS_SATURN_AND_SATURN_IS_STRONGER`<br>`PRAVRAJYA_MOON_LORD_ASPECTS_SATURN_AND_SATURN_IS_STRONGER` |
| 37 | `BPHS_KRISHNA_CHATURDASHI_PART_6`<br>`PARIJATA_KRISHNA_CHATURDASHI_PART_6` |
| 36 | `PARIJATA_TOGETHER_MOON_JUPITER_SATURN`<br>`SARAVALI_TOGETHER_MOON_JUPITER_SATURN` |
| 36 | `PARIJATA_TOGETHER_SUN_MARS_MERCURY_SATURN`<br>`SARAVALI_TOGETHER_SUN_MARS_MERCURY_SATURN` |
| 35 | `BJ_PRAVRAJYA_SUN`<br>`PRAVRAJYA_SUN`<br>`SARAVALI_PRAVRAJYA_SUN_STRONGEST` |
| 35 | `BPHS_KRISHNA_CHATURDASHI_PART_2`<br>`PARIJATA_KRISHNA_CHATURDASHI_PART_2` |
| 33 | `BPHS_KRISHNA_CHATURDASHI_PART_1`<br>`PARIJATA_KRISHNA_CHATURDASHI_PART_1` |
| 33 | `PARIJATA_YOGA_FIVE_IN_ENEMIES_SIGNS`<br>`SARAVALI_FIVE_IN_ENEMIES_SIGNS` |
| 32 | `BJ_SUN_IN_BHAVA_1_CANCER_RISING`<br>`PARIJATA_ANGA_SUN_RISING_IN_CANCER` |
| 31 | `BPHS_KRISHNA_CHATURDASHI_PART_3`<br>`PARIJATA_KRISHNA_CHATURDASHI_PART_3` |
| 30 | `BPHS_KRISHNA_CHATURDASHI_PART_4`<br>`PARIJATA_KRISHNA_CHATURDASHI_PART_4` |
| 29 | `BJ_ANISHTA_LUMINARIES_MARS_SATURN_IN_THE_NINTH_ELEVENTH_THIRD_FIFTH`<br>`PARIJATA_ROGA_MALEFICS_NINTH_ELEVENTH_THIRD_FIFTH_UNASPECTED` |
| 29 | `BJ_MARANA_SUN_AND_MARS_TENTH_AND_FOURTH_DEATH_BY_A_FALLING_STONE`<br>`PARIJATA_MARANA_SUN_AND_MARS_TENTH_AND_FOURTH_DEATH_BY_A_FALLING_STONE`<br>`SARAVALI_MARANA_SUN_AND_MARS_TENTH_AND_FOURTH_FALL_FROM_A_HILL` |
| 28 | `BPHS_KRISHNA_CHATURDASHI_PART_5`<br>`PARIJATA_KRISHNA_CHATURDASHI_PART_5` |
| 27 | `BJ_STRI_THREE_BENEFICS_RISING`<br>`SARAVALI_STRI_THREE_BENEFICS_RISING` |
| 27 | `BJ_SUN_IN_BHAVA_1_ARIES_RISING`<br>`PARIJATA_ANGA_SUN_RISING_IN_ARIES` |
| 23 | `BJ_PRAVRAJYA_MOON`<br>`PRAVRAJYA_MOON`<br>`SARAVALI_PRAVRAJYA_MOON_STRONGEST` |
| 22 | `PARASHARA_SRINATHA`<br>`PARIJATA_SRINATHA` |
| 22 | `PARIJATA_TOGETHER_MARS_MERCURY_JUPITER_VENUS`<br>`SARAVALI_TOGETHER_MARS_MERCURY_JUPITER_VENUS` |
| 20 | `PARIJATA_TOGETHER_MARS_MERCURY_VENUS_SATURN`<br>`SARAVALI_TOGETHER_MARS_MERCURY_VENUS_SATURN` |
| 20 | `PARIJATA_TOGETHER_MERCURY_JUPITER_VENUS_SATURN`<br>`SARAVALI_TOGETHER_MERCURY_JUPITER_VENUS_SATURN` |
| 20 | `PARIJATA_TOGETHER_SUN_MARS_JUPITER_VENUS`<br>`SARAVALI_TOGETHER_SUN_MARS_JUPITER_VENUS` |
| 19 | `PARIJATA_TOGETHER_SUN_MARS_VENUS_SATURN`<br>`SARAVALI_TOGETHER_SUN_MARS_VENUS_SATURN` |
| 19 | `PARIJATA_TOGETHER_SUN_MOON_MERCURY_JUPITER`<br>`SARAVALI_TOGETHER_SUN_MOON_MERCURY_JUPITER` |
| 18 | `BJ_PRAVRAJYA_MERCURY_COMBUST`<br>`SARAVALI_PRAVRAJYA_MERCURY_STRONGEST_COMBUST` |
| 18 | `PARASHARA_KHADGA`<br>`PARIJATA_KHADGA` |
| 17 | `BJ_JANMA_DESERTED_MOON_ASPECTED_BY_A_BENEFIC_TAKEN_IN`<br>`PARIJATA_JANMA_DESERTED_MOON_ASPECTED_BY_A_BENEFIC` |
| 17 | `BJ_NABHASA_SHARA`<br>`NABHASA_SARA` |
| 16 | `BJ_JANMA_BIPED_SIGN_RISING_WITH_SATURN_ASPECTED_BY_MERCURY`<br>`PARIJATA_JANMA_HUMAN_SIGN_RISING_WITH_SATURN_ASPECTED_BY_MERCURY` |
| 16 | `BJ_JANMA_BIPED_SIGN_RISING_WITH_SATURN_ASPECTED_BY_SUN`<br>`PARIJATA_JANMA_HUMAN_SIGN_RISING_WITH_SATURN_ASPECTED_BY_SUN` |
| 16 | `BJ_RAJA_VARGOTTAMA_MOON_ASPECTED_BY_FOUR`<br>`PARIJATA_RAJA_VARGOTTAMA_MOON_ASPECTED_BY_FOUR` |
| 15 | `BJ_NABHASA_DANDA`<br>`NABHASA_DANDA` |
| 15 | `BJ_NABHASA_YUGA`<br>`NABHASA_YUGA` |
| 14 | `BJ_ANISHTA_MOON_TENTH_VENUS_SEVENTH_MALEFIC_FOURTH`<br>`PHALADEEPIKA_PUTRA_MALEFIC_FOURTH_VENUS_SEVENTH_MOON_TENTH` |
| 14 | `BJ_CHANDRA_ADHI`<br>`LUNAR_ADHI_YOGA` |
| 14 | `BJ_MARANA_SUN_TENTH_MARS_FOURTH_FALL_FROM_A_VEHICLE`<br>`PARIJATA_MARANA_SUN_TENTH_MARS_FOURTH_FALL_FROM_A_VEHICLE` |
| 14 | `BJ_STRI_SATURN_AND_VENUS_IN_EACH_OTHERS_NAVAMSHA_ASPECTING_EACH_OTHER`<br>`BPHS_STRI_SATURN_AND_VENUS_IN_EACH_OTHERS_NAVAMSHA_ASPECTING_EACH_OTHER`<br>`SARAVALI_STRI_SATURN_AND_VENUS_IN_EACH_OTHERS_NAVAMSHA_ASPECTING_EACH_OTHER` |
| 13 | `BJ_NABHASA_SHAKTI`<br>`NABHASA_SAKTHI` |
| 13 | `BJ_NABHASA_YUPA`<br>`NABHASA_YUPA` |
| 13 | `PARIJATA_TOGETHER_MARS_JUPITER_VENUS_SATURN`<br>`SARAVALI_TOGETHER_MARS_JUPITER_VENUS_SATURN` |
| 12 | `BJ_JANMA_WATERY_SIGN_RISING_WITH_SATURN_ASPECTED_BY_MERCURY`<br>`PARIJATA_JANMA_WATERY_SIGN_RISING_WITH_SATURN_ASPECTED_BY_MERCURY` |
| 12 | `BJ_MARANA_LUMINARIES_IN_VIRGO_ASPECTED_BY_A_MALEFIC_DEATH_THROUGH_KIN`<br>`PARIJATA_MARANA_LUMINARIES_IN_VIRGO_ASPECTED_BY_A_MALEFIC_DEATH_THROUGH_KIN`<br>`SARAVALI_MARANA_LUMINARIES_IN_VIRGO_ASPECTED_BY_A_MALEFIC_BONDS` |
| 12 | `BJ_RAJA_VARGOTTAMA_LAGNA_ASPECTED_BY_FOUR`<br>`PARIJATA_RAJA_VARGOTTAMA_LAGNA_ASPECTED_BY_FOUR` |
| 12 | `BJ_STRI_MOON_AND_VENUS_RISING_IN_A_SIGN_OF_SATURN_OR_MARS_ASPECTED_BY_A_MALEFIC`<br>`BPHS_STRI_MOON_AND_VENUS_RISING_IN_A_SIGN_OF_SATURN_OR_MARS_ASPECTED_BY_A_MALEFIC`<br>`SARAVALI_STRI_MOON_AND_VENUS_RISING_IN_A_SIGN_OF_SATURN_OR_MARS_ASPECTED_BY_A_MALEFIC` |
| 11 | `BJ_JANMA_MOON_RISING_SATURN_TWELFTH_ASPECTED_BY_A_MALEFIC_IN_PRISON`<br>`PARIJATA_JANMA_MOON_RISING_SATURN_TWELFTH_ASPECTED_BY_A_MALEFIC` |
| 11 | `PARIJATA_TOGETHER_MOON_MERCURY_JUPITER_VENUS`<br>`SARAVALI_TOGETHER_MOON_MERCURY_JUPITER_VENUS` |
| 10 | `BJ_MARANA_LUMINARIES_IN_A_DUAL_LAGNA_DEATH_BY_DROWNING`<br>`PARIJATA_MARANA_LUMINARIES_IN_A_DUAL_LAGNA_DEATH_BY_DROWNING`<br>`SARAVALI_MARANA_LUMINARIES_IN_A_DUAL_LAGNA_DROWNING` |
| 10 | `PARASHARA_TRIMURTHI_HARI`<br>`PARIJATA_HARIHARABRAHMA_FROM_THE_SECOND_LORD` |
| 10 | `PARIJATA_TOGETHER_MOON_MARS_VENUS_SATURN`<br>`SARAVALI_TOGETHER_MOON_MARS_VENUS_SATURN` |
| 10 | `PARIJATA_TOGETHER_MOON_MERCURY_VENUS_SATURN`<br>`SARAVALI_TOGETHER_MOON_MERCURY_VENUS_SATURN` |
| 9 | `ARISHTA_SATURN_MARS_SUN_IN_FIFTH`<br>`PARIJATA_SATURN_MARS_SUN_IN_FIFTH` |
| 9 | `BJ_MARANA_MARS_LIBRA_SATURN_ARIES_MOON_IN_A_SIGN_OF_SATURN_DEATH_IN_FILTH`<br>`PARIJATA_MARANA_MARS_LIBRA_SATURN_ARIES_MOON_IN_A_SIGN_OF_SATURN_DEATH_IN_FILTH` |
| 9 | `BJ_PRAVRAJYA_MOON_COMBUST`<br>`SARAVALI_PRAVRAJYA_MOON_STRONGEST_COMBUST` |
| 9 | `PARIJATA_TOGETHER_SUN_MERCURY_JUPITER_VENUS_SATURN`<br>`SARAVALI_TOGETHER_SUN_MERCURY_JUPITER_VENUS_SATURN` |
| 9 | `PARIJATA_TOGETHER_SUN_MOON_JUPITER_VENUS`<br>`SARAVALI_TOGETHER_SUN_MOON_JUPITER_VENUS` |
| 8 | `BJ_ANISHTA_MALEFICS_TWELFTH_SEVENTH_AND_FIRST_WANING_MOON_FIFTH`<br>`PHALADEEPIKA_KALATRA_WANING_MOON_FIFTH_MALEFICS_TWELFTH_SEVENTH_FIRST` |
| 8 | `BJ_JANMA_WATERY_SIGN_RISING_WITH_SATURN_ASPECTED_BY_SUN`<br>`PARIJATA_JANMA_WATERY_SIGN_RISING_WITH_SATURN_ASPECTED_BY_SUN` |
| 7 | `BJ_ANISHTA_LUMINARIES_SEVENTH_ASPECTED_BY_SATURN`<br>`PARIJATA_LUMINARIES_SEVENTH_ASPECTED_BY_SATURN` |
| 7 | `BJ_JANMA_MOON_RISING_MARS_AND_SATURN_ELEVENTH_DESERTED_AND_DIES`<br>`PARIJATA_JANMA_MOON_RISING_MARS_AND_SATURN_ELEVENTH` |
| 7 | `BJ_JANMA_SCORPIO_OR_CANCER_RISING_WITH_SATURN_ASPECTED_BY_THE_MOON_IN_A_PIT`<br>`PARIJATA_JANMA_SCORPIO_OR_CANCER_RISING_WITH_SATURN_ASPECTED_BY_THE_MOON` |
| 7 | `BJ_MARANA_SATURN_MOON_MARS_FOURTH_SEVENTH_TENTH_DEATH_IN_A_WELL`<br>`PARIJATA_MARANA_SATURN_MOON_MARS_FOURTH_SEVENTH_TENTH_DEATH_IN_A_WELL`<br>`SARAVALI_MARANA_SATURN_MOON_MARS_FOURTH_SEVENTH_TENTH_IN_A_WELL` |
| 7 | `PARIJATA_TOGETHER_SUN_MARS_JUPITER_SATURN`<br>`SARAVALI_TOGETHER_SUN_MARS_JUPITER_SATURN` |
| 6 | `BJ_JANMA_WATERY_SIGN_RISING_WITH_SATURN_ASPECTED_BY_MOON`<br>`PARIJATA_JANMA_WATERY_SIGN_RISING_WITH_SATURN_ASPECTED_BY_MOON` |
| 6 | `BJ_PRAVRAJYA_JUPITER`<br>`PRAVRAJYA_JUPITER`<br>`SARAVALI_PRAVRAJYA_JUPITER_STRONGEST` |
| 6 | `BJ_STRI_MOON_VENUS_AND_MARS_SEVENTH_ADULTERY_BY_LEAVE`<br>`SARAVALI_STRI_MOON_VENUS_AND_MARS_SEVENTH_ADULTERY_BY_LEAVE` |
| 6 | `PARIJATA_DARIDRA_SIX_IN_ENEMIES_SIGNS`<br>`SARAVALI_SIX_IN_ENEMIES_SIGNS` |
| 6 | `PARIJATA_MERCURY_IN_CANCER_SIXTH_OR_EIGHTH_ASPECTED_BY_THE_MOON`<br>`SARAVALI_MERCURY_IN_CANCER_AS_SIXTH_OR_EIGHTH` |
| 6 | `PARIJATA_TOGETHER_SUN_MOON_MERCURY_JUPITER_VENUS`<br>`SARAVALI_TOGETHER_SUN_MOON_MERCURY_JUPITER_VENUS` |
| 6 | `PARIJATA_TOGETHER_SUN_MOON_MERCURY_VENUS_SATURN`<br>`SARAVALI_TOGETHER_SUN_MOON_MERCURY_VENUS_SATURN` |
| 5 | `BJ_MARANA_MARS_FOURTH_SUN_SEVENTH_SATURN_TENTH_WEAPON_FIRE_OR_KING`<br>`PARIJATA_MARANA_MARS_FOURTH_SUN_SEVENTH_SATURN_TENTH_WEAPON_FIRE_OR_KING`<br>`SARAVALI_MARANA_MARS_FOURTH_SUN_SEVENTH_SATURN_TENTH_WEAPON_FIRE_OR_KING` |
| 5 | `BJ_NABHASA_ARDHA_CHANDRA`<br>`NABHASA_ARDHA_CHANDRA` |
| 5 | `BJ_PRAVRAJYA_SATURN`<br>`PRAVRAJYA_SATURN`<br>`SARAVALI_PRAVRAJYA_SATURN_STRONGEST` |
| 5 | `PARIJATA_TOGETHER_MOON_JUPITER_VENUS_SATURN`<br>`SARAVALI_TOGETHER_MOON_JUPITER_VENUS_SATURN` |

## Never held

A rule that holds on none of the charts is not a duplicate of anything, but it is either rare or dead, and every one is accounted for: it is listed with an instant and a place where it holds, which the gate evaluates, or with why none can. The instants were found by searching every two hours from 1900 to 2100 at Kathmandu's latitude, on the meridian that puts the rule's lagna on the horizon, and from 1800 to 2400, the span the built-in ephemeris covers, for a rule the two centuries lacked. 0 rules the sky never holds are computed in code rather than written as conditions, which this evaluation does not read, and `check-doshas` measures them.

### Held at a found instant

| rule | instant (JD, UT) | latitude, longitude |
|---|---:|---|
| `BJ_ANISHTA_ECLIPSED_MOON_RISING_SATURN_AND_MARS_IN_THE_TRINES` | 2392651.0593 | -60, 78 |
| `BJ_ANISHTA_ECLIPSED_SUN_RISING_SATURN_AND_MARS_IN_THE_TRINES` | 2440476.2218 | 25, -171 |
| `BJ_ANISHTA_MOON_RISING_SATURN_AND_MARS_ABOUT_SUN_SEVENTH` | 2379626.9306 | 27.7172, 85.324 |
| `BJ_ANISHTA_MOON_TENTH_MARS_SEVENTH_SATURN_SECOND_FROM_THE_SUN` | 2379033.125 | 27.7172, 85.324 |
| `BJ_RAJA_VENUS_AT_HOME_IN_THE_FOURTH` | 2409098.2361 | 27.7172, 85.324 |
| `BJ_RAJA_ARIES_SUN_AND_MOON_RISING_FIGURE` | 2406723.5139 | 27.7172, 85.324 |
| `BJ_RAJA_ARIES_SUN_RISING_FIGURE` | 2380808.4722 | 27.7172, 85.324 |
| `BJ_RAJA_CANCER_JUPITER_RISING_FIGURE` | 2387738.7361 | 27.7172, 85.324 |
| `BJ_RAJA_CAPRICORN_MARS_AND_MOON_FIGURE` | 2381415.6527 | 27.7172, 85.324 |
| `BJ_RAJA_CAPRICORN_MARS_AND_SATURN_FIGURE` | 2405512.6389 | 27.7172, 85.324 |
| `BJ_RAJA_SAGITTARIUS_MOON_AND_JUPITER_FIGURE` | 2406714.5069 | 27.7172, 85.324 |
| `BJ_RAJA_TAURUS_MOON_RISING_FIGURE` | 2384950.1944 | 27.7172, 85.324 |
| `ARISHTA_JUPITER_LAGNA_FOUR_IN_SECOND` | 2415023.16667 | 27.7172, 172.13 |
| `ARISHTA_NODES_WITH_LUMINARIES_LAGNA_AFFLICTED` | 2415345.25 | 27.7172, -86.883 |
| `ARISHTA_SATURN_LAGNA_MOON_EIGHTH_JUPITER_THIRD` | 2416294.08333 | 27.7172, 105.938 |
| `BJ_ECLIPSED_MOON_IN_LAGNA_MARS_EIGHTH` | 2425046.83333 | 27.7172, 167.283 |
| `BJ_SATURN_SUN_MOON_MARS_IN_ORDER` | 2416033.75 | 27.7172, 121.864 |
| `BPHS_BRAHMIN_CURSE_2` | 2431221 | 27.7172, 153.649 |
| `BPHS_BRAHMIN_CURSE_6` | 2386282.5 | 27.7172, 59.955 |
| `BPHS_BROTHER_CURSE_1` | 2382500.5 | 27.7172, -124.453 |
| `BPHS_BROTHER_CURSE_10` | 2500859 | 27.7172, 40.202 |
| `BPHS_BROTHER_CURSE_2` | 2403064 | 27.7172, 13.969 |
| `BPHS_BROTHER_CURSE_3` | 2385689.5 | 27.7172, 176.183 |
| `BPHS_BROTHER_CURSE_4` | 2381655.5 | 27.7172, -59.659 |
| `BPHS_BROTHER_CURSE_7` | 2431492 | 27.7172, -60.936 |
| `BPHS_BROTHER_CURSE_9` | 2399106.75 | 27.7172, -166.358 |
| `BPHS_CANCER_RISING_WITH_JUPITER_AND_MOON_LIMITLESS` | 2404926.5 | 27.7172, 152.741 |
| `BJ_AYUR_CANCER_RISING_WITH_JUPITER_AND_MOON_BEYOND_RECKONING` | 2404926.5 | 27.7172, 152.741 |
| `BPHS_MATERNAL_UNCLE_CURSE_2` | 2380555 | 27.7172, 173.046 |
| `BPHS_MOON_AND_A_BENEFIC_IN_ARUDHA_LAGNA_AND_JUPITER_SECOND` | 2436428.66667 | 27.7172, 79.059 |
| `BPHS_MOTHER_CURSE_12` | 2434156 | 27.7172, 72.443 |
| `BPHS_MOTHER_CURSE_13` | 2584735.5 | 27.7172, -9.26 |
| `BPHS_MOTHER_CURSE_5` | 2410500.5 | 27.7172, -49.856 |
| `BPHS_MOTHER_CURSE_6` | 2385569 | 27.7172, 177.583 |
| `BPHS_MOTHER_CURSE_8` | 2477457 | 27.7172, -90.153 |
| `BPHS_NO_SON_STRONG_MALEFICS_IN_THE_FIFTH` | 2514807.5 | 27.7172, -2.923 |
| `BPHS_SATURN_IN_A_FIFTH_OF_HIS_OWN_WITH_THE_LUMINARIES_ELEVENTH` | 2415964.16667 | 27.7172, -95.118 |
| `BPHS_SERPENT_CURSE_5` | 2379840.5 | 27.7172, -84.943 |
| `BPHS_SPIRIT_CURSE_1` | 2383249.5 | 27.7172, -42.061 |
| `BPHS_SPIRIT_CURSE_8` | 2417856.5 | 27.7172, -121.56 |
| `BPHS_SPIRIT_CURSE_9` | 2380791 | 27.7172, 109.151 |
| `BPHS_THE_SUN_IN_LEO_AS_THE_FIFTH_WITH_THREE_ELEVENTH` | 2524474.33333 | 27.7172, 17.863 |
| `BPHS_WIFE_CURSE_10` | 2451130.5 | 27.7172, -113.242 |
| `BPHS_WIFE_CURSE_11` | 2420394.5 | 27.7172, -103.057 |
| `BPHS_WIFE_CURSE_7` | 2384731.5 | 27.7172, 103.036 |
| `BPHS_WIFE_CURSE_8` | 2400560 | 27.7172, 162.235 |
| `NABHASA_CHAKRA` | 2412258.5 | 27.7172, 65.74 |
| `NABHASA_CHATRA` | 2378584.5 | 27.7172, -158.98 |
| `NABHASA_GADA` | 2430110 | 27.7172, -93.752 |
| `NABHASA_GOLA` | 2437699.08333 | 27.7172, -37.399 |
| `NABHASA_KAMALA` | 2420757.16667 | 27.7172, 119.593 |
| `NABHASA_NALA` | 2420757.16667 | 27.7172, 59.593 |
| `NABHASA_SAKATA` | 2418976.25 | 27.7172, -8.53 |
| `NABHASA_SAMUDRA` | 2412258.5 | 27.7172, 97.14 |
| `NABHASA_VIHAGA` | 2418976.25 | 27.7172, 81.47 |
| `PARASHARA_MATSYA` | 2418336 | 27.7172, 112.354 |
| `PARIJATA_JUPITER_EIGHTH_IN_A_SIGN_OF_MARS_ASPECTED_BY_FOUR` | 2428308.91667 | 27.7172, -89.235 |
| `PARIJATA_ARIES_END_RISING_WITH_JUPITER_OR_VENUS_MOON_MID_TAURUS_MARS_SIMHASANA` | 2379731.99182 | 27.7172, -107.86 |
| `PARIJATA_JUPITER_SATURN_ONE_NAVAMSHA_IN_THE_NINTH_OR_TENTH_SUN_RISING` | 2429878.5 | 27.7172, 82.5 |
| `PARIJATA_JUPITER_GOPURA_IN_A_KENDRA_VENUS_PARAVATA_IN_A_TRINE_CANCER_RISING` | 2473202.25 | 27.7172, -68.985 |
| `PARIJATA_SAGITTARIUS_RISING_IN_ARIES_NAVAMSHA_WITH_JUPITER_VENUS_SEVENTH_MOON_IN_VIRGO` | 2380915 | 27.7172, 41.087 |
| `PARIJATA_MALIKA_FROM_THE_SECOND` | 2379544.9722 | 27.7172, 85.324 |
| `PARIJATA_SARADA` | 2390777.2222 | 27.7172, 85.324 |
| `PARIJATA_KURMA` | 2547710.3125 | 27.7172, 85.324 |
| `PARIJATA_MALIKA_FROM_THE_FIFTH` | 2379544.7847 | 27.7172, 85.324 |
| `PARIJATA_MALIKA_FROM_THE_SEVENTH` | 2379544.7847 | 27.7172, 45 |
| `PARIJATA_MALIKA_FROM_THE_EIGHTH` | 2379544.7847 | 27.7172, 12 |
| `PARIJATA_MALIKA_FROM_THE_NINTH` | 2379544.7847 | 27.7172, -22 |
| `PARIJATA_MALIKA_FROM_THE_ELEVENTH` | 2379544.7847 | 27.7172, -92 |
| `PHALADEEPIKA_MAHAPURUSHA_FIVE` | 2545713 | 27.7172, 85.324 |
| `PHALADEEPIKA_KING_BY_THE_LORDS` | 2457986.9168 | 27.7172, 85.324 |
| `BJ_JANMA_SUN_IN_A_QUADRUPED_SIGN_THE_REST_STRONG_IN_DUAL_SIGNS_TWINS` | 2386271.2083 | 27.7172, 85.324 |
| `BPHS_TANU_SUN_IN_A_QUADRUPED_SIGN_THE_REST_STRONG_IN_DUAL_SIGNS_TWINS` | 2386271.2083 | 27.7172, 85.324 |
| `PARIJATA_JANMA_TWINS_IN_ONE_SAC_SUN_IN_A_QUADRUPED_SIGN` | 2386271.2083 | 27.7172, 85.324 |
| `PARIJATA_RAJA_FIVE_EXALTED_AND_STRONG` | 2413666.7083 | 27.7172, 85.324 |
| `SARAVALI_PRAVRAJYA_STRONG_SUN_MOON_MARS_MERCURY_SATURN` | 2379849.5417 | 27.7172, 85.324 |
| `PHALADEEPIKA_MAHARAJA_FIVE_DIGNIFIED_IN_KENDRAS` | 2426839.0139 | 27.7172, 85.324 |
| `PHALADEEPIKA_MAHARAJA_FULL_MOON_THREE_IN_OWN_NAVAMSHAS` | 2379392.2917 | 27.7172, 85.324 |
| `PHALADEEPIKA_MAHARAJA_MOON_IN_A_GREAT_FRIENDS_NAVAMSHA_ASPECTED_BY_VENUS` | 2381876.4167 | 27.7172, 85.324 |
| `PHALADEEPIKA_MAHARAJA_NIGHT_MOON_IN_CANCER_TENTH` | 2383287.25 | 27.7172, 85.324 |
| `PHALADEEPIKA_MAHARAJA_SAGITTARIUS_FIGURE` | 2381414.3333 | 27.7172, 85.324 |
| `PHALADEEPIKA_MAHARAJA_VENUS_IN_ASHVINI_RISING` | 2380836.4028 | 27.7172, 85.324 |
| `PARIJATA_SATURN_IN_A_JUPITER_NAVAMSHA_LAGNA_LORD_EXALTED` | 2416949 | 27.7172, 48.39 |
| `PARIJATA_SUN_SATURN_MARS_IN_LAGNA_WANING_MOON_IN_A_VENUS_SEVENTH` | 2425223.25 | 27.7172, 169.281 |
| `PARIJATA_TOGETHER_MOON_MARS_MERCURY_JUPITER_SATURN` | 2437699.08333 | 27.7172, -37.399 |
| `PARIJATA_TOGETHER_MOON_MARS_MERCURY_JUPITER_VENUS_SATURN` | 2437699.08333 | 27.7172, -37.399 |
| `PARIJATA_TOGETHER_SUN_MOON_MARS_JUPITER_SATURN` | 2437699.08333 | 27.7172, -37.399 |
| `PARIJATA_TOGETHER_SUN_MOON_MARS_JUPITER_VENUS_SATURN` | 2437699.08333 | 27.7172, -37.399 |
| `PARIJATA_TOGETHER_SUN_MOON_MARS_MERCURY_JUPITER_SATURN` | 2437699.08333 | 27.7172, -37.399 |
| `SARAVALI_JUPITER_IN_EIGHTH_IN_A_SIGN_OF_MARS` | 2428308.91667 | 27.7172, -89.235 |
| `SARAVALI_MARS_SUN_SATURN_IN_TAURUS_AS_EIGHTH` | 2451702.41667 | 27.7172, -97.355 |
| `SARAVALI_RAJA_ALL_VARGOTTAMA_OR_AT_HOME` | 2414577.9722 | 27.7172, 85.324 |
| `SARAVALI_RAJA_ARIES_RISING_WITH_MOON_MARS_AND_JUPITER` | 2382242.5347 | 27.7172, 85.324 |
| `SARAVALI_RAJA_BENEFICS_IN_THE_FIRST_FOURTH_SEVENTH_MALEFICS_THIRD_NINTH_ELEVENTH` | 2383809.3125 | 27.7172, 85.324 |
| `SARAVALI_RAJA_EXALTED_SUN_TENTH_WITH_JUPITER_VENUS_NINTH` | 2386658.7153 | 27.7172, 85.324 |
| `SARAVALI_RAJA_FULL_MOON_IN_A_TAURUS_NAVAMSHA_ASPECTED_BY_ALL` | 2409719.1528 | 27.7172, 85.324 |
| `SARAVALI_RAJA_JUPITER_AND_MOON_FIFTH_THREE_IN_OWN_NAVAMSHAS` | 2385461.1389 | 27.7172, 85.324 |
| `SARAVALI_RAJA_JUPITER_FOURTH_AT_HOME_FULL_MOON_NINTH` | 2554453.4931 | 27.7172, 85.324 |
| `SARAVALI_RAJA_KALASHA` | 2387002.7431 | 27.7172, 85.324 |
| `SARAVALI_RAJA_LUMINARIES_EARLY_SAGITTARIUS_SATURN_RISING` | 2387615.8889 | 27.7172, 85.324 |
| `SARAVALI_RAJA_MERCURY_RISING_JUPITER_SEVENTH_MOON_FOURTH_IN_CANCER` | 2436292.5556 | 27.7172, 85.324 |
| `SARAVALI_RAJA_MOON_AT_HIGHEST_EXALTATION_ASPECTED_BY_VENUS` | 2384649.5556 | 27.7172, 85.324 |
| `SARAVALI_RAJA_SUN_AT_HIGHEST_EXALTATION_RISING` | 2378971.5 | 27.7172, 85.324 |
| `SARAVALI_RAJA_TAURUS_RISING_JUPITER_AND_MOON` | 2391343.6944 | 27.7172, 85.324 |
| `SARAVALI_RAJA_VENUS_MERCURY_MARS_RISING_MOON_AND_JUPITER_SEVENTH` | 2392442.4931 | 27.7172, 85.324 |
| `SARAVALI_RAJA_VENUS_MOON_SUN_THIRD_MARS_SEVENTH_JUPITER_NINTH` | 2413189.3194 | 27.7172, 85.324 |
| `SARAVALI_RAJA_VENUS_RISING_IN_A_SIGN_OF_JUPITER_MERCURY_EXALTED` | 2404003.6875 | 27.7172, 85.324 |
| `SARAVALI_RAJA_BENEFICS_SECOND_MOON_AND_MARS_SEVENTH` | 2400648.3194 | 27.7172, 85.324 |
| `SARAVALI_RAJA_CAPRICORN_MARS_RISING_SATURN_NINTH_LUMINARIES_SEVENTH` | 2412303.0694 | 27.7172, 85.324 |
| `SARAVALI_RAJA_CAPRICORN_RISING_MARS_RISING_BENEFICS_FOURTH` | 2395064.2222 | 27.7172, 85.324 |
| `SARAVALI_RAJA_CAPRICORN_SATURN_RISING_SUN_SEVENTH_VENUS_EIGHTH` | 2480990.0208 | 27.7172, 85.324 |
| `SARAVALI_RAJA_MARS_TENTH_MOON_AND_VENUS_NINTH_EXALTED_SUN_ELEVENTH` | 2525076.6597 | 27.7172, 85.324 |
| `SARAVALI_RAJA_MOON_AND_JUPITER_IN_THE_FIRST_FIVE_DEGREES_OF_CANCER` | 2378916.2569 | 27.7172, 85.324 |
| `SARAVALI_RAJA_SATURN_AND_MOON_RISING_JUPITER_SEVENTH_MERCURY_EXALTED` | 2411977.4931 | 27.7172, 85.324 |
| `SARAVALI_RAJA_BRIGHT_LAGNA_LORD_MALEFICS_ELEVENTH_BENEFICS_IN_KENDRAS` | 2387779.6458 | 27.7172, 85.324 |
| `SARAVALI_RAJA_MERCURY_AND_SATURN_IN_KENDRAS_FROM_VENUS_THE_REST_THIRD_OR_ELEVENTH` | 2403424.3611 | 27.7172, 85.324 |
| `SARAVALI_TOGETHER_MOON_MARS_MERCURY_JUPITER_SATURN` | 2437699.08333 | 27.7172, -37.399 |
| `SARAVALI_TOGETHER_MOON_MARS_MERCURY_JUPITER_VENUS_SATURN` | 2437699.08333 | 27.7172, -37.399 |
| `SARAVALI_TOGETHER_SUN_MOON_MARS_JUPITER_SATURN` | 2437699.08333 | 27.7172, -37.399 |
| `SARAVALI_TOGETHER_SUN_MOON_MARS_JUPITER_VENUS_SATURN` | 2437699.08333 | 27.7172, -37.399 |
| `SARAVALI_TOGETHER_SUN_MOON_MARS_MERCURY_JUPITER_SATURN` | 2437699.08333 | 27.7172, -37.399 |
| `PARIJATA_RAJA_SUN_MOON_MERCURY_VENUS_TENTH_IN_FRIENDS_NAVAMSHAS` | 2527530.6932 | 27.7172, 85.324 |
| `PARIJATA_BUDHA` | 2393193.0833 | 27.7172, 85.324 |
| `PARIJATA_RAJA_FIVE_EXALTED_JUPITER_RISING` | 2413666.7431 | 27.7172, 85.324 |
| `PARIJATA_RAJA_FULL_MOON_NINTH_SATURN_TENTH_MARS_SECOND` | 2379244.3681 | 27.7172, 85.324 |
| `PARIJATA_RAJA_FULL_MOON_RISING_IN_PISCES_MARS_EXALTED_SATURN_AQUARIUS` | 2417158.8819 | 27.7172, 85.324 |
| `PARIJATA_RAJA_LUMINARIES_EARLY_SAGITTARIUS_STRONG_SATURN_RISING_MARS_EXALTED` | 2387615.8889 | 27.7172, 85.324 |
| `PARIJATA_RAJA_MARS_CAPRICORN_SUN_ARIES_JUPITER_AQUARIUS` | 2390285.2639 | 27.7172, 85.324 |
| `PARIJATA_RAJA_MERCURY_RISING_JUPITER_SEVENTH_BRIGHT_MOON_FOURTH_IN_CANCER_VENUS_TENTH` | 2436292.5556 | 27.7172, 85.324 |
| `PARIJATA_RAJA_MOON_RISING_JUPITER_FOURTH_VENUS_TENTH_SATURN_DIGNIFIED` | 2380915.5972 | 27.7172, 85.324 |
| `PARIJATA_RAJA_ONE_EACH_IN_THE_FIRST_SECOND_THIRD_FOURTH_FIFTH_SEVENTH_NINTH` | 2381967.2292 | 27.7172, 85.324 |
| `PARIJATA_RAJA_SATURN_RISING_IN_AQUARIUS_FOUR_EXALTED` | 2406725.3889 | 27.7172, 85.324 |
| `PARIJATA_RAJA_STRONG_MARS_RISING_IN_CAPRICORN_SATURN_NINTH_OR_TWELFTH_LUMINARIES_SEVENTH` | 2383452.0278 | 27.7172, 85.324 |
| `PARIJATA_RAJA_SUN_AND_MERCURY_FOURTH_SATURN_AND_MOON_TENTH_MARS_RISING` | 2385710.2639 | 27.7172, 85.324 |
| `PARIJATA_RAJA_SUN_IN_A_TRIKONA_IN_PISCES_MOON_AND_JUPITER_IN_CANCER` | 2378944.2639 | 27.7172, 85.324 |
| `PARIJATA_RAJA_SUN_RISING_IN_SAGITTARIUS_MARS_AND_MOON_TENTH_VENUS_ELEVENTH_OR_TWELFTH` | 2387253.5347 | 27.7172, 85.324 |
| `PARIJATA_RAJA_TAURUS_MOON_RISING_ASPECTED_BY_ALL` | 2454074.9375 | 27.7172, 85.324 |
| `PARIJATA_RAJA_VENUS_IN_ASHWINI_RISING` | 2380836.3889 | 27.7172, 85.324 |
| `BJ_NABHASA_NALA` | 2420757.16667 | 27.7172, 59.593 |
| `BJ_NABHASA_GADA` | 2430110 | 27.7172, -93.752 |
| `BJ_NABHASA_SAKATA` | 2418976.25 | 27.7172, -8.53 |
| `BJ_NABHASA_VIHAGA` | 2418976.25 | 27.7172, 81.47 |
| `BJ_NABHASA_KAMALA` | 2420757.16667 | 27.7172, 119.593 |
| `BJ_NABHASA_CHHATRA` | 2378584.5 | 27.7172, -158.98 |
| `BJ_NABHASA_SAMUDRA` | 2412258.5 | 27.7172, 97.14 |
| `BJ_NABHASA_CHAKRA` | 2412258.5 | 27.7172, 65.74 |
| `BJ_NABHASA_GOLA` | 2437699.08333 | 27.7172, -37.399 |
| `PARIJATA_ANGA_LAST_NAVAMSHA_OF_CAPRICORN_RISING_ASPECTED_BY_SATURN_MOON_SUN` | 2378699.0972 | 27.7172, 85.324 |
| `PARIJATA_ANGA_LAST_NAVAMSHA_OF_CAPRICORN_RISING_MALEFICS_IN_ITS_THREE_DREKKANAS` | 2426740.5694 | 27.7172, 85.324 |
| `PARIJATA_ANGA_LEO_RISING_WITH_THE_LUMINARIES_ASPECTED_BY_SATURN_AND_MARS` | 2384931.5 | 27.7172, 85.324 |
| `PARIJATA_ANGA_LUMINARIES_TWELFTH_MALEFICS_SIXTH_AND_EIGHTH_BOTH_EYES` | 2379584.6042 | 27.7172, 85.324 |
| `PARIJATA_ANGA_MARS_IN_THE_FIRST_DREKKANA_OF_THE_FIFTH_ASPECTED_BY_SATURN_SUN_MOON` | 2403740.6667 | 27.7172, 85.324 |
| `PARIJATA_ANGA_MARS_IN_THE_FIRST_DREKKANA_OF_THE_LAGNA_ASPECTED_BY_SATURN_SUN_MOON` | 2403741 | 27.7172, 85.324 |
| `PARIJATA_ANGA_MOON_SIXTH_SUN_EIGHTH_SATURN_TWELFTH_MARS_SECOND` | 2395967.8889 | 27.7172, 85.324 |
| `PARIJATA_ANGA_MOON_TENTH_MARS_SEVENTH_SUN_SECOND` | 2378835.375 | 27.7172, 85.324 |
| `PARIJATA_DARIDRA_LAGNA_LORD_TWELFTH_MARS_WITH_THE_MOON_IN_A_CRUEL_TENTH` | 2378631.5 | 27.7172, 85.324 |
| `PARIJATA_DARIDRA_NIGHT_MOVABLE_LAGNA_WEAK_BENEFICS_IN_KENDRAS_AND_TRIKONAS` | 2452872.1626 | 27.7172, 85.324 |
| `PARIJATA_DARIDRA_SUN_AT_HIS_DEEPEST_FALL` | 2415318.5 | 27.7172, 85.324 |
| `PARIJATA_DARIDRA_SUN_IN_ARIES_IN_A_LIBRA_NAVAMSHA_ASPECTED_BY_A_MALEFIC` | 2378617.5 | 27.7172, 85.324 |
| `PARIJATA_DARIDRA_VENUS_FIRST_JUPITER_FIFTH_MARS_ELEVENTH_MOON_THIRD_ALL_FALLEN` | 2416028.5 | 27.7172, 85.324 |
| `PARIJATA_DARIDRA_VENUS_JUPITER_MOON_MARS_FALLEN_IN_FOUR_GOOD_HOUSES` | 2416028.8264 | 27.7172, 85.324 |
| `PARIJATA_MARANA_MOON_IN_VIRGO_SEVENTH_WITH_A_MALEFIC_VENUS_IN_ARIES_SUN_RISING` | 2380041.5 | 27.7172, 85.324 |
| `BJ_MARANA_MOON_IN_VIRGO_SEVENTH_WITH_A_MALEFIC_VENUS_IN_ARIES_SUN_RISING` | 2380041.5 | 27.7172, 85.324 |
| `PARIJATA_MARANA_SATURN_SECOND_MOON_FOURTH_MARS_TENTH_WORMS_IN_A_WOUND` | 2378648.5069 | 27.7172, 85.324 |
| `BJ_MARANA_SATURN_SECOND_MOON_FOURTH_MARS_TENTH_WORMS_IN_A_WOUND` | 2378648.5069 | 27.7172, 85.324 |
| `PARIJATA_MARANA_SUN_AND_MARS_SEVENTH_SATURN_EIGHTH_WANING_MOON_FOURTH_BIRDS` | 2386221.9653 | 27.7172, 85.324 |
| `BJ_MARANA_SUN_AND_MARS_SEVENTH_SATURN_EIGHTH_WANING_MOON_FOURTH_BIRDS` | 2386221.9653 | 27.7172, 85.324 |
| `PARIJATA_MARANA_SUN_FIRST_MARS_FIFTH_SATURN_EIGHTH_MOON_NINTH_A_FALL` | 2378894.5 | 27.7172, 85.324 |
| `BJ_MARANA_SUN_FIRST_MARS_FIFTH_SATURN_EIGHTH_MOON_NINTH_A_FALL` | 2378894.5 | 27.7172, 85.324 |
| `PARIJATA_MARANA_SUN_FOURTH_MARS_TENTH_ASPECTED_BY_THE_WANING_MOON_IMPALED` | 2380352.2222 | 27.7172, 85.324 |
| `BJ_MARANA_SUN_FOURTH_MARS_TENTH_ASPECTED_BY_THE_WANING_MOON_IMPALED` | 2380352.2222 | 27.7172, 85.324 |
| `PARIJATA_PRESHYA_VENUS_NINTH_MOON_SEVENTH_JUPITER_SECOND_OR_FIRST_MARS_EIGHTH_FIXED_LAGNA` | 2380934.7222 | 27.7172, 85.324 |
| `PARIJATA_VENUS_FALLEN_OR_SATURNIAN_IN_A_DUSTHANA_ASPECTED_BY_SATURN_LUMINARIES_SEVENTH` | 2378875.9792 | 27.7172, 85.324 |
| `BJ_JANMA_BIPED_SIGN_RISING_WITH_SATURN_ASPECTED_BY_VENUS_AND_THE_MOON` | 2380000.25 | 27.7172, 29.306 |
| `PHALADEEPIKA_KALATRA_VENUS_AND_MOON_SEVENTH_FROM_MARS_AND_SATURN` | 2406841.5 | 27.7172, -21.251 |
| `PHALADEEPIKA_PUTRA_SUN_FIFTH_IN_A_SIGN_OF_FEW_CHILDREN_SATURN_EIGHTH_MARS_RISING` | 2436432.5 | 27.7172, -26.638 |
| `PHALADEEPIKA_PUTRA_VENUS_AND_MERCURY_SEVENTH_JUPITER_FIFTH_MALEFIC_FOURTH` | 2379520.5 | 27.7172, -67.337 |
| `PARIJATA_GARBHA_MALE_HERMAPHRODITE_SATURN_WITH_MERCURY_SIXTH` | 2413883.5 | 27.7172, -64.469 |
| `PARIJATA_GARBHA_MANY_LAGNA_IN_SAGITTARIUS_LAST_NAVAMSHA` | 2380863.5 | 27.7172, -59.061 |
| `PARIJATA_GARBHA_TRIPLETS_TWO_BOYS_AND_A_GIRL` | 2404407.5 | 27.7172, -147.429 |
| `PARIJATA_GARBHA_TRIPLETS_TWO_GIRLS_AND_A_BOY` | 2382381.5 | 27.7172, -38.206 |
| `PARIJATA_GARBHA_TWIN_BOYS_FIVE_IN_GEMINI_OR_SAGITTARIUS_NAVAMSHAS` | 2411498.5 | 27.7172, 68.733 |
| `PARIJATA_GARBHA_TWIN_GIRLS_FIVE_IN_VIRGO_OR_PISCES_NAVAMSHAS` | 2407511.5 | 27.7172, 38.386 |
| `PARIJATA_RAJA_FIVE_IN_OWN_SIGNS` | 2384900.5 | 27.7172, 4.173 |
| `BPHS_BHAGYA_SUN_DEEP_EXALTED_NINTH_LORD_ELEVENTH` | 2378971.5 | 27.7172, -145.651 |
| `BPHS_BHAGYA_VENUS_DEEP_EXALTED_WITH_THE_NINTH_LORD_SATURN_THIRD` | 2385162.5 | 27.7172, 9.244 |
| `BPHS_DHANA_JUPITER_ELEVENTH_VENUS_SECOND_BENEFICS_WITH_THE_SECOND_LORD_AND_IN_THE_TWELFTH` | 2387114.5 | 27.7172, 108.579 |
| `BPHS_DHANA_SECOND_AND_ELEVENTH_LORDS_IN_DUSTHANAS_MARS_ELEVENTH_RAHU_SECOND` | 2381651.5 | 27.7172, 126.853 |
| `BPHS_KARMA_SUN_ELEVENTH_WITH_RAHU_MARS_AND_SATURN` | 2390045.5 | 27.7172, 167.89 |
| `BPHS_PUTRA_JUPITER_DEEP_EXALTED_SECOND_LORD_WITH_RAHU_NINTH_LORD_NINTH` | 2400571.5 | 27.7172, -126.567 |
| `BPHS_PUTRA_MALEFICS_FOURTH_AND_SIXTH_FIFTH_LORD_DEEP_EXALTED_WITH_THE_LAGNA_LORD` | 2388627.5 | 27.7172, -43.29 |
| `BPHS_PUTRA_NINTH_LORD_RISING_FIFTH_LORD_DEBILITATED_KETU_AND_MERCURY_FIFTH` | 2398851.5 | 27.7172, -8.867 |
| `BPHS_PUTRA_SIX_GRAHAS_FIFTH_ITS_LORD_TWELFTH_LAGNA_LORD_AND_MOON_STRONG` | 2540998.5 | 27.7172, -34.632 |
| `BPHS_ROGA_FIFTH_LORD_SIXTH_SIXTH_LORD_WITH_JUPITER_TWELFTH_LORD_RISING` | 2384552.5 | 27.7172, -56.53 |
| `BPHS_VYAYA_RAHU_TWELFTH_WITH_MARS_SATURN_AND_THE_SUN_ITS_LORD_WITH_THE_SUN` | 2414974.5 | 27.7172, 146.319 |
| `SARAVALI_AYUR_CANCER_RISING_WITH_JUPITER_AND_MOON_UNLIMITED` | 2404926.5 | 27.7172, 158.281 |
| `SARAVALI_BHANGA_AQUARIUS_RISING_THREE_DEBILITATED_JUPITER_WITH_THE_SUN_NONE_EXALTED` | 2385812.5 | 27.7172, 141.59 |
| `SARAVALI_BHANGA_LAGNA_LORD_HOSTILE_TO_THE_MOON_SUN_THIRD_MARS_SIXTH_SATURN_SEVENTH` | 2393713.5 | 27.7172, 15.17 |
| `SARAVALI_BHANGA_MALEFICS_IN_KENDRAS_FALLEN_OR_HOSTILE_BENEFICS_HIDDEN` | 2431765.5 | 27.7172, -106.764 |
| `SARAVALI_BHANGA_SUN_DEEPLY_DEBILITATED_OTHERS_DIGNIFIED` | 2380254.5 | 27.7172, -96.651 |
| `SARAVALI_BHANGA_SUN_IN_THE_TENTH_DEGREE_OF_LIBRA` | 2378792.5 | 27.7172, -95.678 |
| `SARAVALI_FIVE_DEBILITATED` | 2455125.5 | 27.7172, -90.773 |
| `SARAVALI_FIVE_IN_MOOLATRIKONA` | 2562831.5 | 27.7172, -47.561 |
| `SARAVALI_FIVE_IN_OWN_SIGNS` | 2394381.5 | 27.7172, 19.537 |
| `SARAVALI_FOUR_IN_MOOLATRIKONA` | 2430250.5 | 27.7172, -53.554 |
| `SARAVALI_MARANA_MARS_TENTH_MOON_FOURTH_SATURN_SECOND_WORMS_IN_A_WOUND` | 2378648.5 | 27.7172, 103.298 |
| `SARAVALI_MARANA_PISCES_RISING_SUN_MOON_SEVENTH_WITH_A_MALEFIC_VENUS_IN_ARIES` | 2380041.5 | 27.7172, 91.114 |
| `SARAVALI_MARANA_SUN_AND_MARS_SEVENTH_SATURN_EIGHTH_MOON_FOURTH_BIRDS` | 2386221.5 | 27.7172, -91.278 |
| `SARAVALI_MARANA_SUN_FOURTH_MARS_TENTH_WITH_THE_WANING_MOON_ASPECTED_BY_SATURN_WOOD` | 2391264.5 | 27.7172, 12.237 |
| `SARAVALI_MARANA_SUN_MARS_SATURN_MOON_IN_THE_LAGNA_EIGHTH_AND_TRINES_A_FALL` | 2378894.5 | 27.7172, 96.063 |
| `SARAVALI_MARANA_WANING_MOON_EIGHTH_MARS_TENTH_SUN_FIRST_SATURN_FOURTH_CLUBBED` | 2378597.5 | 27.7172, 96.517 |
| `SARAVALI_MARANA_WANING_MOON_TENTH_MARS_NINTH_SUN_FIRST_SATURN_FIFTH_SMOKE_OR_FIRE` | 2385009.5 | 27.7172, 91.388 |
| `SARAVALI_PRAVRAJYA_MOON_MARS_MERCURY_JUPITER_SATURN` | 2379849.5 | 27.7172, -57.476 |
| `SARAVALI_PRAVRAJYA_STRONG_MOON_TENTH_IN_A_BENEFICS_NAVAMSHA_ASPECTED_BY_STRONG_SATURN` | 2389559.5 | 27.7172, 92.185 |
| `SARAVALI_PRAVRAJYA_SUN_MOON_MARS_JUPITER_VENUS_SATURN` | 2386257.5 | 27.7172, 106.689 |
| `SARAVALI_PRAVRAJYA_SUN_MOON_MARS_MERCURY_JUPITER_SATURN` | 2379849.5 | 27.7172, -57.476 |
| `SARAVALI_RAJA_SUN_VIRGO_MARS_JUPITER_SCORPIO_MOON_ARIES_UNASPECTED` | 2480300.5 | 27.7172, -63.669 |
| `SARAVALI_SIX_DEBILITATED` | 2558865.5 | 27.7172, -98.605 |
| `SARAVALI_PRAVRAJYA_STRONG_SUN_MARS_JUPITER_SATURN` | 2451653.0995 | 27.7172, 85.324 |
| `SARAVALI_PRAVRAJYA_STRONG_MOON_MARS_MERCURY_JUPITER_VENUS_SATURN` | 2386256.75 | 27.7172, 85.324 |
| `SARAVALI_PRAVRAJYA_STRONG_SUN_MARS_MERCURY_JUPITER_VENUS_SATURN` | 2386254 | 27.7172, 85.324 |
| `BPHS_STRI_MOON_VENUS_SATURN_AND_MARS_RISING_FIFTH_TOUCHED_BY_A_MALEFIC` | 2381300.5 | 27.7172, 146.992 |
| `PARIJATA_JANMA_HUMAN_SIGN_RISING_WITH_SATURN_ASPECTED_BY_VENUS_AND_THE_MOON` | 2380000.5 | 27.7172, -60.941 |
| `SARAVALI_STRI_STRONG_MERCURY_JUPITER_VENUS_WITH_THE_MOON_IN_AN_EVEN_LAGNA` | 2382682.5 | 27.7172, 56.166 |

### Held nowhere searched

| rule | why |
|---|---|
| `ARISHTA_ALL_IN_APOKLIMAS_WITHOUT_STRENGTH` | It asks all seven below their required Shadbala at once, the Moon among them, and her Shadbala rarely falls below her six rupas: no instant from 1900 to 2100 with the seven in apoklimas has every one weak. |
| `BJ_RAJA_AQUARIUS_ARIES_TAURUS_FIGURE` | It puts the Sun in Aries and Mercury in Gemini, two signs on, more than 30 degrees from the Sun, and Mercury never stands 28 degrees from him. |
| `BJ_RAJA_CAPRICORN_SATURN_RISING_FIGURE` | No sky from 1801 to 2398 has it; the nearest a search from 1000 BC found is in 483 BC. |
| `BJ_RAJA_CAPRICORN_SATURN_RISING_LORDS_AT_HOME_FIGURE` | It puts the Sun in Leo and Mercury in Gemini, two signs back, more than 30 degrees from the Sun, and Mercury never stands 28 degrees from him. |
| `BJ_RAJA_JUPITER_RISING_MOON_TENTH` | It puts Mercury in the second and the Sun in the fourth, more than 30 degrees apart, and Mercury never stands 28 degrees from the Sun. |
| `BJ_RAJA_LIBRA_TAURUS_FIGURE` | No sky from 1000 BC to AD 2400 has its seven grahas so with Libra or Taurus rising, searched day by day and every ten minutes of a day its slow grahas allow. |
| `BJ_RAJA_MARS_AND_SATURN_RISING` | No sky from 1000 BC to AD 2400 has its seven grahas in those houses, searched day by day and every ten minutes of a day its slow grahas allow. |
| `BJ_RAJA_PISCES_MOON_RISING_FIGURE` | No sky from 1801 to 2398 has it; the nearest a search from 1000 BC found is in 980 BC. |
| `BJ_RAJA_SATURN_RISING_LUMINARIES_TENTH` | No sky from 1801 to 2398 has it; the nearest a search from 1000 BC found is in AD 1250. |
| `BJ_RAJA_TAURUS_MOON_RISING_OTHERS_ELEVENTH` | No sky from 1000 BC to AD 2400 has its seven grahas in those houses, searched day by day and every ten minutes of a day its slow grahas allow. |
| `BJ_RAJA_VIRGO_MERCURY_RISING_FIFTH_AND_FOURTH_FIGURE` | Venus three signs from Mercury needs both near their greatest elongations on either side of the Sun, and no sky from 1000 BC to AD 2400 has it with the other five so. |
| `BJ_RAJA_VIRGO_MERCURY_RISING_PISCES_FIGURE` | Venus in Gemini three signs from Mercury in Virgo needs both near their greatest elongations on either side of the Sun, and no sky from 1000 BC to AD 2400 has it with the other five so. |
| `BJ_VARGA_SEVEN_IN_OWN_HOUSES` | It asks the Sun in Leo, Mercury in Virgo and Venus in Libra together, which happens on 868 days from 1800 to 2398; Jupiter and Saturn are at home on 41 of those and Mars on none, and from 1001 BCE to 1800 no day holds all seven. |
| `BPHS_BRAHMIN_CURSE_1` | Of the 529 half-days from 1800 to 2398 with Jupiter, Mars and Saturn together and Rahu in a sign of Jupiter, the ninth lord, counted from a lagna four signs behind them, stands in the fifth itself or in the third or fourth, never the eighth. |
| `BPHS_DEBILITATED_GRAHAS_SIXTH_EIGHTH_THIRD_AND_LAGNA_LORD_DIGNIFIED` | Only Leo and Aquarius rising have a sign of debilitation in each of the third, sixth and eighth, and in both the graha debilitated in the third is the lagna lord, the Sun in Libra or Saturn in Aries, whom the verse wants exalted or in his own sign. |
| `BPHS_LAGNA_LORD_NEUTRAL_OF_THE_SUN_MEDIUM` | No lagna lord regards the Sun as a neutral: the Moon, Mars, Mercury and Jupiter count him a friend, Venus and Saturn an enemy, and for Leo rising the lord is the Sun himself. |
| `BPHS_MATERNAL_UNCLE_CURSE_1` | It asks Mercury, Jupiter, Mars and Rahu in one sign with Saturn four signs behind them: of the 361 half-days from 1800 to 2398 with the four together, none has Saturn so. |
| `BPHS_MATERNAL_UNCLE_CURSE_4` | With the Moon, Mercury and Mars in the fifth, the sixth or the twelfth lord is one of them under ten lagnas, and under Virgo and Pisces the Sun in the lagna would stand four signs from Mercury; the translation's fourth lord fails the same way (C390). |
| `BPHS_MERCURY_IN_A_FIFTH_OF_HIS_OWN_WITH_THREE_ELEVENTH` | It asks Mars and Jupiter together in the sign opposite Mercury in his own, so both near opposition to the Sun, with the Moon there too: no day from 1800 to 2400, the span the built-in ephemeris covers, has the slower grahas so. |
| `BPHS_MOTHER_CURSE_7` | Only Cancer and Scorpio rising leave its five lords' places consistent: of the 78 half-days from 1800 to 2398 with Jupiter and Saturn in Cancer and Mars in Sagittarius, none has Venus in Aquarius, and of the 54 with Mars and Jupiter in Aries and Saturn in Gemini, none has Mercury in Scorpio. |
| `BPHS_MOTHER_CURSE_9` | The thin Moon in the seventh keeps the Sun and Mercury between the fourth and the tenth, and Saturn, Rahu and Ketu stand in the fifth, the fourth and the tenth, so Mars alone is left to hem the lagna, which needs a malefic on each side (C388). |
| `BPHS_SERPENT_CURSE_7` | It asks six grahas in the fifth, Rahu, Jupiter, Saturn and Mars among them: no half-day from 1800 to 2398, the span the built-in ephemeris covers, has even the Sun, Mars, Jupiter, Saturn and Rahu in one sign. |
| `BPHS_SIX_GRAHAS_EXALTED` | Mercury exalted in Virgo and Venus in Pisces would stand opposite, which the two never do, so the six are the Sun in Aries, the Moon in Taurus, Mars in Capricorn, Jupiter in Cancer, Venus in Pisces and Saturn in Libra: no day from 1800 to 2400 has the slower grahas so. |
| `BPHS_SPIRIT_CURSE_3` | It asks the Sun in the twelfth and Mercury in the fifth, five signs apart, and Mercury keeps within a sign of the Sun (C390). |
| `BPHS_SPIRIT_CURSE_5` | The lagna lord in the eighth is none of the five in the lagna, and the Sun or Mercury there would stand five signs from Venus, so it is Mars, with Aries or Scorpio rising: no half-day from 1800 to 2398 has Jupiter, Saturn and Rahu together in either. |
| `PARASHARA_KOORMA` | It asks the Sun, Mars and Saturn each in its own or exaltation sign in the lagna, third and eleventh, with a benefic in each of the fifth, sixth and seventh: no day from 1800 to 2400 has the grahas other than the Moon so. |
| `PARIJATA_MOON_IN_LEO_SUN_AND_SATURN_EIGHTH_VENUS_ASPECTING` | Venus casts only the seventh aspect and never stands seventh from the Sun. The verse holds under the partial aspects Jataka Parijata ch. 2 counts, which the condition language does not read. |
| `PARIJATA_SATURN_DEVALOKA_MARS_PARAVATA_JUPITER_SIMHASANA_RISING` | Saturn in seven or eight of his own vargas of the ten and Mars in six meet on about thirty days from 1800 to 2398 (the half-daily sky), and Jupiter then holds at most four of his, never the five the verse asks. |
| `PARIJATA_WANING_MOON_EIGHTH_LORD_WITH_A_MALEFIC_WEAK_LAGNA` | A lagna's strength is its bhava bala, which has no required figure for it to fall short of, so no chart carries a weak lagna. |
| `PHALADEEPIKA_MAHARAJA_PISCES_FIGURE` | Of the 68 days from 1801 to 2398 with Venus, Jupiter and Saturn in Pisces, the Moon stands in Taurus for 164 hours, and in none of them does Mars aspect the Sun. |
| `SARAVALI_BHANGA_JUPITER_AND_MOON_IN_CANCER_MERCURY_AND_SATURN_IN_LIBRA` | Four days from 1800 to 2400, all in 1895, have Jupiter in Cancer, Mercury and Saturn in Libra and the Sun, Mars and Venus as the rule asks, and on them the Moon is in Libra to Sagittarius, never Cancer. |
| `SARAVALI_MARS_SATURN_AND_SUN_IN_THE_SECOND_UNDER_A_WEAK_MOON` | A Moon aspecting the second while the Sun stands in it is opposite the Sun, near full, and no instant from 1900 to 2100 has her Shadbala below her six rupas there. |
| `SARAVALI_RAJA_ARIES_RISING_JUPITER_AND_SUN_IN_LEO_FIGURE` | No sky from 1801 to 2398 has its seven grahas in those signs, searched four times a day. |
| `SARAVALI_RAJA_CANCER_MOON_RISING_FIGURE` | No sky from 1801 to 2398 has its seven grahas in those houses with Cancer rising, searched four times a day. |
| `SARAVALI_RAJA_EXALTED_MOON_RISING_FIGURE` | No sky from 1801 to 2398 has its six grahas in those signs, searched four times a day. |
| `SARAVALI_RAJA_LEO_SUN_RISING_FIGURE` | No sky from 1801 to 2398 has its five grahas in those signs, searched four times a day. |
| `SARAVALI_RAJA_SATURN_LATE_CAPRICORN_FIGURE` | No sky from 1801 to 2398 has its seven grahas in those signs with Saturn in the latter half of Capricorn, searched four times a day. |
| `SARAVALI_RAJA_VIRGO_MERCURY_RISING_MOON_ELEVENTH` | No sky from 1801 to 2398 has its six grahas in those signs, searched four times a day. |
| `SARAVALI_RAJA_ARIES_FIGURE_JUPITER_NINTH_IN_OWN_NAVAMSHA` | No sky from 1801 to 2398 has its seven grahas in those signs, searched every ten minutes of a day they allow. |
| `SARAVALI_RAJA_BENEFICS_FIFTH_SUN_SIXTH_MARS_EXALTED_SATURN_NINTH` | No sky from 1801 to 2398 has Venus, Mercury and Jupiter together with the Sun a sign on, Mars exalted and Saturn four signs on from them, searched every ten minutes of a day they allow. |
| `SARAVALI_RAJA_VIRGO_MERCURY_RISING_JUPITER_PISCES_SATURN_SIXTH` | Venus in Sagittarius three signs from Mercury in Virgo needs both near their greatest elongations on either side of the Sun, and no sky from 1801 to 2398 has it with Jupiter in Pisces and Saturn in Aquarius. |
| `SARAVALI_RAJA_MOON_TENTH_VENUS_NINTH_OTHERS_ELEVENTH` | No sky from 1801 to 2398 has Mars, Mercury, Jupiter and Saturn in the Sun's sign with Venus two signs and the Moon one sign behind it, searched every ten minutes of a day they allow. |
| `SARAVALI_RAJA_JUPITER_SECOND_MARS_FOURTH_SUN_TENTH_MOON_RISING_THE_REST_IN_ENEMY_SIGNS` | No sky from 1801 to 2398 holds it: the 164 instants at Kathmandu that place Jupiter, Mars, the Sun and the Moon as it asks, searched every ten minutes of a day they allow, never also put Mercury, Venus and Saturn all in an enemy's sign. |
| `SARAVALI_RAJA_VENUS_AND_MERCURY_IN_KENDRAS_FROM_SATURN_THE_REST_EXALTED` | No sky from 1801 to 2398 has the Sun, the Moon, Mars, Jupiter and Saturn exalted at once, searched every ten minutes of a day they allow. |
| `PARIJATA_RAJA_SIX_EXALTED` | As BPHS_SIX_GRAHAS_EXALTED: Mercury exalted in Virgo and Venus in Pisces would stand opposite, which the two never do, so the six are the Sun, the Moon, Mars, Jupiter, Venus and Saturn, and no day from 1800 to 2398 has the slower of them exalted together. |
| `PARIJATA_RAJA_LIBRA_FIGURE` | It fixes six grahas' signs and the lagna: the Moon, Jupiter and Venus in Sagittarius, Mercury in Libra rising, Mars in Virgo and Saturn in Capricorn. No day from 1800 to 2398 has the grahas in those signs, searched day by day. |
| `PARIJATA_RAJA_MERCURY_VIRGO_MOON_PISCES_MARS_GEMINI_JUPITER_SAGITTARIUS_SATURN_CAPRICORN` | No day from 1800 to 2398 has Mercury, the Moon, Mars, Jupiter and Saturn in Virgo, Pisces, Gemini, Sagittarius and Capricorn at once, searched day by day by sign. |
| `BJ_NABHASA_VAJRA` | Benefics in the 1st and 7th and malefics in the 4th and 10th, all seven in the kendras: the Sun, a malefic, would stand three or nine signs from Venus, a benefic, and Venus is never more than two signs from the Sun (her greatest elongation 47.28 degrees, day by day 1800 to 2398). Ch. 12 v. 6 says so itself. |
| `BJ_NABHASA_YAVA` | Vajra's figure turned about, malefics in the 1st and 7th and benefics in the 4th and 10th, all seven in the kendras: the Sun and Venus would again stand three or nine signs apart, which her greatest elongation of 47.28 degrees, day by day 1800 to 2398, never allows. |
| `PARIJATA_MARANA_WANING_MOON_EIGHTH_MARS_TENTH_SATURN_FIRST_SUN_FOURTH_CLUBBED` | The Moon in the 8th and the Sun in the 4th stand five signs apart, so the Moon is 120 to 180 degrees past the Sun and waxing; the verse asks her waning. |
| `BJ_MARANA_WANING_MOON_EIGHTH_MARS_TENTH_SATURN_FIRST_SUN_FOURTH_CLUBBED` | The Moon in the 8th and the Sun in the 4th stand five signs apart, so the Moon is 120 to 180 degrees past the Sun and waxing; the verse asks her waning. |
| `PARIJATA_MARANA_WANING_MOON_TENTH_MARS_NINTH_SATURN_FIRST_SUN_FIFTH_SMOKE_OR_FIRE` | The Moon in the 10th and the Sun in the 5th stand six signs apart, 120 to 180 degrees past the Sun, so she is waxing; of 83 days 1800 to 2398 holding everything else, none has her waning. |
| `BJ_MARANA_WANING_MOON_TENTH_MARS_NINTH_SATURN_FIRST_SUN_FIFTH_SMOKE_OR_FIRE` | The Moon in the 10th and the Sun in the 5th stand six signs apart, 120 to 180 degrees past the Sun, so she is waxing; of 83 days 1800 to 2398 holding everything else, none has her waning. |
| `PARIJATA_MARANA_WANING_MOON_TENTH_SUN_SEVENTH_MARS_FOURTH_DEATH_IN_FILTH` | The Moon in the 10th and the Sun in the 7th stand four signs apart, so the Moon is past the Sun by under 180 degrees and waxing; the verse asks her waning. |
| `BJ_MARANA_WANING_MOON_TENTH_SUN_SEVENTH_MARS_FOURTH_DEATH_IN_FILTH` | The Moon in the 10th and the Sun in the 7th stand four signs apart, so the Moon is past the Sun by under 180 degrees and waxing; the verse asks her waning. |
| `PARIJATA_PRESHYA_NIGHT_SUN_TENTH_MOON_SEVENTH_SATURN_FOURTH_MARS_THIRD_JUPITER_SECOND` | The verse asks a birth at night (the Sanskrit's nisayam) with the Sun in the 10th, which is above the horizon: no night birth has the Sun there. |
| `PARIJATA_ROGA_MARS_WITH_MERCURY_SIXTH_IN_A_MALEFICS_NAVAMSHA_ASPECTED_BY_MOON_AND_VENUS` | Venus aspects only the 7th from her, so she would stand opposite Mars and so opposite Mercury, with whom he stands; Mercury and Venus are never more than about 76 degrees apart. |
| `PARIJATA_JUPITER_WITH_SATURN_MARS_RAHU_ASPECTED_BY_VENUS_AND_MERCURY` | No day 1800 to 2398 has Jupiter with Saturn, Mars and Rahu aspected by Venus and Mercury, searched by sign day by day; leaving out any one condition leaves at most 26 days. |
| `PARIJATA_DARIDRA_FIVE_IN_THE_EIGHTH_SIXTH_TWELFTH_FIFTH_TENTH_TWELFTH_LORD_FALLEN_YET_STRONGER` | Of the days 1800 to 2398 placing the five grahas so, the twelfth lord is debilitated on 66 and combust as well on none, searched by sign day by day; the translator's own example figure cannot occur. |
| `PARIJATA_ROGA_SATURN_WITH_MARS_SIXTH_ASPECTED_BY_SUN_AND_RAHU_LAGNA_LORD_WEAK` | Of the days 1800 to 2398 holding everything else, 383 by sign, none has Rahu aspecting Saturn and Mars in the 6th. |
| `PARIJATA_ANGA_MERCURY_IN_A_TRIKONA_THE_REST_WEAK` | It asks every graha but Mercury weak, and the Sun, Mars and Saturn are never weak together: of 2,000 instants drawn over 1900 to 2100 the Sun is weak on 482, Mars on 54, both on 12, and the three on none. |
| `PARIJATA_LOKA_MOKSHA_PISCES_RISING_IN_A_BENEFICS_NAVAMSHA_WITH_JUPITER_THE_REST_WEAK` | It asks every graha but Jupiter weak, and the Sun, Mars and Saturn are never weak together: of 2,000 instants drawn over 1900 to 2100 the Sun is weak on 482, Mars on 54, both on 12, and the three on none. |
| `BJ_LOKA_MOKSHA_PISCES_RISING_IN_A_BENEFICS_NAVAMSHA_WITH_JUPITER_THE_REST_WEAK` | It asks every graha but Jupiter weak, and the Sun, Mars and Saturn are never weak together: of 2,000 instants drawn over 1900 to 2100 the Sun is weak on 482, Mars on 54, both on 12, and the three on none. |
| `PARIJATA_MOON_AND_MERCURY_TENTH_AFFLICTED_IN_FALLEN_NAVAMSHAS` | It holds on none of 24,848 instants drawn over 1900 to 2100 at Kathmandu, nor on any ten minutes of the days a sign-level search of 1800 to 2398 found for it: the Moon and Mercury in the 10th, each in its fallen navamsha, with a malefic, aspected by one, together. |
| `PARIJATA_ROGA_SATURN_SIXTH_WITH_GULIKA_ASPECTED_BY_SUN_MARS_RAHU` | It holds on none of 24,848 instants drawn over 1900 to 2100 at Kathmandu, nor on any ten minutes of the days a sign-level search of 1800 to 2398 found with Saturn so aspected: Gulika, which moves through the day, must stand in the 6th with him as well. |
| `BJ_STRI_SATURN_MIDDLING_THE_SOFT_WEAK_THE_HARD_STRONG_ODD_LAGNA` | It asks the Moon, Venus and Mercury below their required Shadbala together while the Sun, Mars and Jupiter pass theirs, and the Moon rarely falls below her six rupas: of 9,000 instants drawn over 1900 to 2100, none holds it. |
| `SARAVALI_STRI_SATURN_MIDDLING_THE_SOFT_WEAK_THE_HARD_STRONG_ODD_LAGNA` | Brihat Jataka ch. 24's figure, its conditions copied: of the same 9,000 instants, none holds it. |
| `PARIJATA_VIYONI` | It asks every malefic strong in its own dwadashamsha and every benefic weak and in none of its own, the Moon among them, and her Shadbala rarely falls below her six rupas: of 9,000 instants drawn over 1900 to 2100, none holds it. |
| `PARIJATA_VIYONI_LAGNA_ASPECTED_BY_VENUS_COW` | It is a figure within the viyoni birth of v. 2, which asks every malefic strong in its own dwadashamsha and every benefic weak and in none of its own, the Moon among them and rarely below her six rupas: of 9,000 instants drawn over 1900 to 2100, none holds v. 2. |
| `PARIJATA_VIYONI_LAGNA_WITH_OR_ASPECTED_BY_SATURN_BUFFALO` | It is a figure within the viyoni birth of v. 2, which asks every malefic strong in its own dwadashamsha and every benefic weak and in none of its own, the Moon among them and rarely below her six rupas: of 9,000 instants drawn over 1900 to 2100, none holds v. 2. |
| `PARIJATA_VIYONI_NODE_RISING_SHEEP_OR_GOAT` | It is a figure within the viyoni birth of v. 2, which asks every malefic strong in its own dwadashamsha and every benefic weak and in none of its own, the Moon among them and rarely below her six rupas: of 9,000 instants drawn over 1900 to 2100, none holds v. 2. |
| `PARIJATA_VIYONI_MALEFICS_RISING_OTHER_KIND` | It is a figure within the viyoni birth of v. 2, which asks every malefic strong in its own dwadashamsha and every benefic weak and in none of its own, the Moon among them and rarely below her six rupas: of 9,000 instants drawn over 1900 to 2100, none holds v. 2. |
| `PARIJATA_GARBHA_TRIPLETS_THREE_BOYS` | The lagna and all seven must stand in two navamsha signs with Mercury in one of them: no day from 1800 to 2398 holds it at the level of signs, and none of 9,000 instants drawn over 1900 to 2100. |
| `PARIJATA_GARBHA_TRIPLETS_THREE_GIRLS` | The lagna and all seven must stand in two navamsha signs with Mercury in one of them: no day from 1800 to 2398 holds it at the level of signs, and none of 9,000 instants drawn over 1900 to 2100. |
| `PARIJATA_RAJA_FOUR_EXALTED_AND_STRONG_IN_KENDRAS` | Only the Sun, Mars, Jupiter and Saturn are exalted in signs square to one another, Aries, Capricorn, Cancer and Libra, and no day from 1800 to 2398 has the four exalted together. |
| `SARAVALI_AYUR_PISCES_RISING_LAST_NAVAMSHA_BENEFICS_IN_TAURUS_THE_REST_EXALTED` | It asks Jupiter in Taurus, Saturn in Libra, the Sun in Aries, Mars in Capricorn and Mercury and Venus in Taurus: the slow pair stands so on 1,808 days from 1800 to 2398, and on none of them are the other four placed so. |
| `SARAVALI_SIX_EXALTED` | Six exalted asks Jupiter in Cancer with Saturn in Libra, which holds on 1,339 days from 1800 to 2398, and no day of them has four more exalted. |
| `SARAVALI_SIX_IN_MOOLATRIKONA` | No day from 1800 to 2398 has six of the seven in their moolatrikona spans. |
| `SARAVALI_SEVEN_IN_MOOLATRIKONA` | Jupiter and Saturn are both in moolatrikona on 102 days from 1800 to 2398, and on none of them are the other five. |
| `SARAVALI_SIX_IN_OWN_SIGNS` | No day from 1800 to 2398 has six of the seven in their own signs outside the moolatrikona span. |
| `SARAVALI_SEVEN_IN_OWN_SIGNS` | No day from 1800 to 2398 has the seven in their own signs outside the moolatrikona span. |
| `SARAVALI_MARANA_WANING_MOON_TENTH_SUN_SEVENTH_MARS_FOURTH_FILTH` | It asks a waning Moon in the tenth with the Sun in the seventh, which puts her three signs ahead of him and so always waxing. |

## Held on every chart

A rule that holds on every chart of the sky fails the gate, unless it names grahas rather than asking whether, and these are the ones that do.

| rule | why |
|---|---|
| `BPHS_NINTH_AND_FIFTH_LORDS_AND_THEIR_COMPANIONS_GIVE_WEALTH` | BPHS names every chart's wealth-givers, the lords of the ninth and fifth and the grahas joining either, and the rule says which they are in its participants. |

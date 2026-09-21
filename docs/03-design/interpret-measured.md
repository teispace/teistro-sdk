# The composers, measured

Status: `generated` by `cargo xtask interpret` over the conformance
corpus's recorded charts and the `i18n/` sources, 2026-09-21. Do not
edit: `check-interpret` regenerates this page and fails on any
difference. The design it measures is
[`interpret-composers.md`](interpret-composers.md).

## What was composed

93 recorded charts composed to 8347 items, 89 items a chart. The corpus records no interpretation text of any kind, so nothing here is compared against a recording: what is measured is whether a plan can be **said** in every locale that must carry it.

Written down, a plan is what it costs to cross a boundary or fill a
golden file: 1 169 365 bytes of JSON over the 93 charts, 12 573 bytes a
chart, 17 425 bytes for the widest and 140 bytes an item. The verses'
own cited words are **not** what weighs it — 112 962 bytes, 9% — so
what a plan costs is the items themselves, each naming its message and
its rule again. Small enough to cross whole: nothing here asks to be
packed.

## What the corpus decides

| proposed rule | verdict | measured |
|---|---|---|
| every key a composer can emit is carried by every strict locale | **holds** | 0 of 18 disagree |
| every item renders from `en-Latn`'s own message, not a fallback | **holds** | 0 of 8347 disagree |
| every item renders in `en-Latn` with nothing to warn about | **holds** | 0 of 8347 disagree |
| every item renders from `ne-Deva-NP`'s own message, not a fallback | **holds** | 0 of 8347 disagree |
| every item renders in `ne-Deva-NP` with nothing to warn about | **holds** | 0 of 8347 disagree |

Every one of the 16 694 renderings — 8347 in each of 2 strict locales
— answered from the locale's own message with nothing to warn about.

## What the composers say

| key | items |
|---|---|
| `sdk.reading.effect` | 2449 |
| `sdk.reading.lifeClass` | 214 |
| `sdk.reading.lifeSpan` | 96 |
| `sdk.reading.participants` | 3333 |
| `sdk.reading.severity` | 29 |
| `sdk.reading.status` | 348 |
| `sdk.reason.grahaInBhava` | 837 |
| `sdk.reason.grahaInRashi` | 837 |
| `sdk.reason.occupants` | 204 |

**The verse's own statement is not translated.** 2449 of the 8347 items
— every `sdk.reading.effect` — carry the words the rule itself
cites, in the language the rule was written in, and the message prints
them as they are. So a Nepali reading says the placements, who took
part, the span, the class and the cancellation in Nepali, and the
verse's sentence in the translator's English, until a locale carries a
reading of that rule written by someone who reads the text. A machine
translation there would be worse than the visible seam.

What they cannot say is counted too: the **lagna** stands in every one
of these charts and is in none of the placement items, because those
messages read a graha and the lagna is `point.LAGNA` — 93 items it
does not say, one a chart. It does take part in a reading, where the
message names no kind and the lagna is the point it is.

## One chart, said

`c001-kathmandu-1990-04-14`, every item of its plan, in each strict
locale. This is the per-language snapshot the module checklist asks for,
and a change to a composer or to a message moves it.

**en-Latn**

```text
Sun in Aries
Sun in the 2nd house
Moon in Scorpio
Moon in the 9th house
Mars in Aquarius
Mars in the 12th house
Mercury in Aries
Mercury in the 2nd house
Jupiter in Gemini
Jupiter in the 4th house
Venus in Aquarius
Venus in the 12th house
Saturn in Capricorn
Saturn in the 11th house
Rahu in Capricorn
Rahu in the 11th house
Ketu in Cancer
Ketu in the 5th house
Sun and Mercury in Aries
Saturn and Rahu in Capricorn
Mars and Venus in Aquarius
NABHASA_PASA: liable to imprisonment, skilful in work, deceiving of disposition, talkative, bereft of good qualities, and attended by many servants
NABHASA_PASA: Sun, Moon, Mars, Mercury, Jupiter, Venus and Saturn take part
LUNAR_MOON_IN_PANAPHARA_FROM_THE_SUN: of middling wealth, intelligence and skill
LUNAR_MOON_IN_PANAPHARA_FROM_THE_SUN: Moon takes part
PARASHARA_ASHUBHA: sensuous, a doer of sinful acts, and a swallower of other men's wealth
PARASHARA_ASHUBHA: Mars, Sun and Mercury take part
PARASHARA_PARVATA: wealthy, eloquent, charitable, learned in the shastras, fond of mirth, famous, splendid, and the leader of a city
PARASHARA_PARVATA: Jupiter takes part
SARAVALI_THE_SECOND_FROM_THE_MOON_ASPECTED_BY_A_BENEFIC: earning plenty of wealth
SARAVALI_THE_SECOND_FROM_THE_MOON_ASPECTED_BY_A_BENEFIC: Jupiter and Moon take part
BPHS_TWENTY_SECOND_DECANATE_OF_A_MIXED_PLANET: his body only drying up
BPHS_TWENTY_SECOND_DECANATE_OF_A_MIXED_PLANET: Ascendant takes part
BPHS_NINTH_AND_FIFTH_LORDS_AND_THEIR_COMPANIONS_GIVE_WEALTH: wealth in the periods of these grahas
BPHS_NINTH_AND_FIFTH_LORDS_AND_THEIR_COMPANIONS_GIVE_WEALTH: Moon, Mars and Venus take part
BPHS_MOON_NAVAMSHA_LORD_WITH_A_MARAKA_OR_IN_A_MARAKA_HOUSE: penniless
BPHS_MOON_NAVAMSHA_LORD_WITH_A_MARAKA_OR_IN_A_MARAKA_HOUSE: Mars takes part
BPHS_EIGHTH_OR_TWELFTH_ASPECTED_BY_KARAKAMSHA_LORD_AND_LAGNA_LORD: bereft of wealth
BPHS_EIGHTH_OR_TWELFTH_ASPECTED_BY_KARAKAMSHA_LORD_AND_LAGNA_LORD: Mercury, Ascendant and Jupiter take part
BPHS_TWELFTH_FROM_ATMAKARAKA_OR_LAGNA_ASPECTED_BY_ITS_LORD: a spendthrift
BPHS_TWELFTH_FROM_ATMAKARAKA_OR_LAGNA_ASPECTED_BY_ITS_LORD: Jupiter takes part
BPHS_SUN_IN_SECOND_UNASPECTED_BY_SATURN: riches and fame
BPHS_SUN_IN_SECOND_UNASPECTED_BY_SATURN: Sun takes part
BPHS_GRAHA_WITH_A_DUSTHANA_LORD_UNASPECTED_BY_TRINE_LORDS_HARMS_WEALTH: harm to finances in the periods of these grahas
BPHS_GRAHA_WITH_A_DUSTHANA_LORD_UNASPECTED_BY_TRINE_LORDS_HARMS_WEALTH: Mercury and Rahu take part
BPHS_ATMAKARAKA_IN_A_BENEFICS_SIGN_OR_NAVAMSHA: wealthy
BPHS_ATMAKARAKA_IN_A_BENEFICS_SIGN_OR_NAVAMSHA: Mercury takes part
BPHS_ONE_TO_THREE_GRAHAS_EXALTED: one of royal birth becomes a king, another equal to a king or wealthy
BPHS_ONE_TO_THREE_GRAHAS_EXALTED: Sun takes part
BPHS_AMATYAKARAKA_WITH_ATMAKARAKAS_DISPOSITOR: great intelligence, a king's minister
BPHS_DUAL_LAGNA_WITH_ITS_LORD_WELL_PLACED_LONG: a long life
BPHS_DUAL_LAGNA_WITH_ITS_LORD_WELL_PLACED_LONG: Jupiter takes part
BPHS_MARS_AND_THIRD_LORD_OR_EIGHTH_LORD_AND_SATURN_AFFLICTED_SHORT: a short life
BPHS_MARS_AND_THIRD_LORD_OR_EIGHTH_LORD_AND_SATURN_AFFLICTED_SHORT: Mars and Rahu take part
BPHS_LAGNA_LORD_FRIEND_OF_THE_SUN_LONG: a long life
BPHS_LAGNA_LORD_FRIEND_OF_THE_SUN_LONG: Jupiter and Sun take part
BPHS_FIXED_THIRD_PLACE_OF_DEATH: death in one's own house
BPHS_BENEFIC_ON_EIGHTH_AND_NINTH_LORD_WITH_A_BENEFIC_DEATH_IN_A_SHRINE: death in a shrine
BPHS_BENEFIC_ON_EIGHTH_AND_NINTH_LORD_WITH_A_BENEFIC_DEATH_IN_A_SHRINE: Jupiter and Venus take part
BPHS_ASCENT_TO_THE_MANES: goes to the manes after death
BPHS_ASCENT_TO_THE_MANES: Venus takes part
BPHS_ASCENT_TO_EARTH: is reborn on earth
BPHS_ASCENT_TO_EARTH: Mars takes part
ARISHTA_LUMINARY_VENUS_OR_RAHU_IN_TWELFTH: Venus takes part
ARISHTA_LUMINARY_VENUS_OR_RAHU_IN_TWELFTH: cancelled
ARISHTA_MALEFICS_FROM_THE_SUN: Moon and Ketu take part
ARISHTA_MALEFICS_FROM_THE_SUN: cancelled
ARISHTA_MALEFICS_FROM_THE_MOON: Sun, Mars and Mercury take part
ARISHTA_MALEFICS_FROM_THE_MOON: cancelled
ARISHTA_BHANGA_BENEFIC_IN_KENDRA: Jupiter takes part
ARISHTA_BHANGA_MARS_WITH_JUPITER: Jupiter and Mars take part
BJ_MALEFICS_IN_TWELFTH_AND_SECOND: Mars, Sun and Mercury take part
BJ_MALEFICS_IN_FIFTH_AND_NINTH: Ketu and Moon take part
SARAVALI_BHANGA_BENEFIC_IN_SIXTH_SEVENTH_OR_EIGHTH_FROM_MOON: Jupiter takes part
NEECHA_BHANGA_RAJA: Moon takes part
NEECHA_BHANGA_DEBIL_LORD_KENDRA: Moon takes part
NEECHA_BHANGA_EXALT_LORD_KENDRA: Moon takes part
```

**ne-Deva-NP**

```text
सूर्य मेषमा
सूर्य दोस्रो भावमा
चन्द्र वृश्चिकमा
चन्द्र ९औं भावमा
मंगल कुम्भमा
मंगल १२औं भावमा
बुध मेषमा
बुध दोस्रो भावमा
गुरु मिथुनमा
गुरु चौथो भावमा
शुक्र कुम्भमा
शुक्र १२औं भावमा
शनि मकरमा
शनि ११औं भावमा
राहु मकरमा
राहु ११औं भावमा
केतु कर्कटमा
केतु ५औं भावमा
मेषमा सूर्य र बुध
मकरमा शनि र राहु
कुम्भमा मंगल र शुक्र
NABHASA_PASA: liable to imprisonment, skilful in work, deceiving of disposition, talkative, bereft of good qualities, and attended by many servants
NABHASA_PASA: सूर्य, चन्द्र, मंगल, बुध, गुरु, शुक्र र शनि संलग्न छन्
LUNAR_MOON_IN_PANAPHARA_FROM_THE_SUN: of middling wealth, intelligence and skill
LUNAR_MOON_IN_PANAPHARA_FROM_THE_SUN: चन्द्र संलग्न छ
PARASHARA_ASHUBHA: sensuous, a doer of sinful acts, and a swallower of other men's wealth
PARASHARA_ASHUBHA: मंगल, सूर्य र बुध संलग्न छन्
PARASHARA_PARVATA: wealthy, eloquent, charitable, learned in the shastras, fond of mirth, famous, splendid, and the leader of a city
PARASHARA_PARVATA: गुरु संलग्न छ
SARAVALI_THE_SECOND_FROM_THE_MOON_ASPECTED_BY_A_BENEFIC: earning plenty of wealth
SARAVALI_THE_SECOND_FROM_THE_MOON_ASPECTED_BY_A_BENEFIC: गुरु र चन्द्र संलग्न छन्
BPHS_TWENTY_SECOND_DECANATE_OF_A_MIXED_PLANET: his body only drying up
BPHS_TWENTY_SECOND_DECANATE_OF_A_MIXED_PLANET: लग्न संलग्न छ
BPHS_NINTH_AND_FIFTH_LORDS_AND_THEIR_COMPANIONS_GIVE_WEALTH: wealth in the periods of these grahas
BPHS_NINTH_AND_FIFTH_LORDS_AND_THEIR_COMPANIONS_GIVE_WEALTH: चन्द्र, मंगल र शुक्र संलग्न छन्
BPHS_MOON_NAVAMSHA_LORD_WITH_A_MARAKA_OR_IN_A_MARAKA_HOUSE: penniless
BPHS_MOON_NAVAMSHA_LORD_WITH_A_MARAKA_OR_IN_A_MARAKA_HOUSE: मंगल संलग्न छ
BPHS_EIGHTH_OR_TWELFTH_ASPECTED_BY_KARAKAMSHA_LORD_AND_LAGNA_LORD: bereft of wealth
BPHS_EIGHTH_OR_TWELFTH_ASPECTED_BY_KARAKAMSHA_LORD_AND_LAGNA_LORD: बुध, लग्न र गुरु संलग्न छन्
BPHS_TWELFTH_FROM_ATMAKARAKA_OR_LAGNA_ASPECTED_BY_ITS_LORD: a spendthrift
BPHS_TWELFTH_FROM_ATMAKARAKA_OR_LAGNA_ASPECTED_BY_ITS_LORD: गुरु संलग्न छ
BPHS_SUN_IN_SECOND_UNASPECTED_BY_SATURN: riches and fame
BPHS_SUN_IN_SECOND_UNASPECTED_BY_SATURN: सूर्य संलग्न छ
BPHS_GRAHA_WITH_A_DUSTHANA_LORD_UNASPECTED_BY_TRINE_LORDS_HARMS_WEALTH: harm to finances in the periods of these grahas
BPHS_GRAHA_WITH_A_DUSTHANA_LORD_UNASPECTED_BY_TRINE_LORDS_HARMS_WEALTH: बुध र राहु संलग्न छन्
BPHS_ATMAKARAKA_IN_A_BENEFICS_SIGN_OR_NAVAMSHA: wealthy
BPHS_ATMAKARAKA_IN_A_BENEFICS_SIGN_OR_NAVAMSHA: बुध संलग्न छ
BPHS_ONE_TO_THREE_GRAHAS_EXALTED: one of royal birth becomes a king, another equal to a king or wealthy
BPHS_ONE_TO_THREE_GRAHAS_EXALTED: सूर्य संलग्न छ
BPHS_AMATYAKARAKA_WITH_ATMAKARAKAS_DISPOSITOR: great intelligence, a king's minister
BPHS_DUAL_LAGNA_WITH_ITS_LORD_WELL_PLACED_LONG: पूर्णायु
BPHS_DUAL_LAGNA_WITH_ITS_LORD_WELL_PLACED_LONG: गुरु संलग्न छ
BPHS_MARS_AND_THIRD_LORD_OR_EIGHTH_LORD_AND_SATURN_AFFLICTED_SHORT: अल्पायु
BPHS_MARS_AND_THIRD_LORD_OR_EIGHTH_LORD_AND_SATURN_AFFLICTED_SHORT: मंगल र राहु संलग्न छन्
BPHS_LAGNA_LORD_FRIEND_OF_THE_SUN_LONG: पूर्णायु
BPHS_LAGNA_LORD_FRIEND_OF_THE_SUN_LONG: गुरु र सूर्य संलग्न छन्
BPHS_FIXED_THIRD_PLACE_OF_DEATH: death in one's own house
BPHS_BENEFIC_ON_EIGHTH_AND_NINTH_LORD_WITH_A_BENEFIC_DEATH_IN_A_SHRINE: death in a shrine
BPHS_BENEFIC_ON_EIGHTH_AND_NINTH_LORD_WITH_A_BENEFIC_DEATH_IN_A_SHRINE: गुरु र शुक्र संलग्न छन्
BPHS_ASCENT_TO_THE_MANES: goes to the manes after death
BPHS_ASCENT_TO_THE_MANES: शुक्र संलग्न छ
BPHS_ASCENT_TO_EARTH: is reborn on earth
BPHS_ASCENT_TO_EARTH: मंगल संलग्न छ
ARISHTA_LUMINARY_VENUS_OR_RAHU_IN_TWELFTH: शुक्र संलग्न छ
ARISHTA_LUMINARY_VENUS_OR_RAHU_IN_TWELFTH: रद्द
ARISHTA_MALEFICS_FROM_THE_SUN: चन्द्र र केतु संलग्न छन्
ARISHTA_MALEFICS_FROM_THE_SUN: रद्द
ARISHTA_MALEFICS_FROM_THE_MOON: सूर्य, मंगल र बुध संलग्न छन्
ARISHTA_MALEFICS_FROM_THE_MOON: रद्द
ARISHTA_BHANGA_BENEFIC_IN_KENDRA: गुरु संलग्न छ
ARISHTA_BHANGA_MARS_WITH_JUPITER: गुरु र मंगल संलग्न छन्
BJ_MALEFICS_IN_TWELFTH_AND_SECOND: मंगल, सूर्य र बुध संलग्न छन्
BJ_MALEFICS_IN_FIFTH_AND_NINTH: केतु र चन्द्र संलग्न छन्
SARAVALI_BHANGA_BENEFIC_IN_SIXTH_SEVENTH_OR_EIGHTH_FROM_MOON: गुरु संलग्न छ
NEECHA_BHANGA_RAJA: चन्द्र संलग्न छ
NEECHA_BHANGA_DEBIL_LORD_KENDRA: चन्द्र संलग्न छ
NEECHA_BHANGA_EXALT_LORD_KENDRA: चन्द्र संलग्न छ
```

# The composers, measured

Status: `generated` by `cargo xtask interpret` over the conformance
corpus's recorded charts and the `i18n/` sources, 2026-09-21. Do not
edit: `check-interpret` regenerates this page and fails on any
difference. The design it measures is
[`interpret-composers.md`](interpret-composers.md).

## What was composed

93 recorded charts composed to 1878 items, 20 items a chart. The corpus records no interpretation text of any kind, so nothing here is compared against a recording: what is measured is whether a plan can be **said** in every locale that must carry it.

## What the corpus decides

| proposed rule | verdict | measured |
|---|---|---|
| every key a composer can emit is carried by every strict locale | **holds** | 0 of 6 disagree |
| every item renders from `en-Latn`'s own message, not a fallback | **holds** | 0 of 1878 disagree |
| every item renders in `en-Latn` with nothing to warn about | **holds** | 0 of 1878 disagree |
| every item renders from `ne-Deva-NP`'s own message, not a fallback | **holds** | 0 of 1878 disagree |
| every item renders in `ne-Deva-NP` with nothing to warn about | **holds** | 0 of 1878 disagree |

Every one of the 3756 renderings — 1878 in each of 2 strict locales
— answered from the locale's own message with nothing to warn about.

## What the composers say

| key | items |
|---|---|
| `sdk.reason.grahaInBhava` | 837 |
| `sdk.reason.grahaInRashi` | 837 |
| `sdk.reason.occupants` | 204 |

What they cannot say is counted too: the **lagna** stands in every one
of these charts and is in none of these plans, because the messages read
a graha and the lagna is `point.LAGNA` — 93 items it does not say, one
a chart.

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
```

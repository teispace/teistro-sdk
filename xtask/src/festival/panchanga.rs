//! The fitted rules against Nepal's national panchanga
//! (`03-design/festival-rules.md` §10, C600).
//!
//! The `NEPAL_PANCHANGA` pack adds to `NEPAL` the days the committee prints
//! that no text in hand gives a table for, each fitted to its VS 2082 and
//! 2083 editions. Every printed day below was read off the page images'
//! "vrataparva vivarana" column, from the row's own Gregorian column. The
//! pack is found over both years at Kathmandu on Nepal's clock, over the
//! committee's own sky (`nepali-committee`), which the pass holds to the
//! print, and over the modern sky (`nepali-default`), which it counts beside
//! it. Every parting under the committee's sky carries its cause and the day
//! the pack gives, which the pass checks both ways.

use std::fmt::Write as _;

use teistro::festival::{Observance, Observances};
use teistro::{FestivalPack, FestivalRequest};

use crate::measure::{Claim, table};

use super::nepal::{Day, Sky, gregorian, kathmandu, near, on};
use super::spell;

/// The days founded together, `(from, to)`: the two printed years in
/// quarters, a worker each. An occurrence a span's edge cuts is judged by
/// the span beside it, so a span's unjudged rules are not refused.
const SPANS: [(Day, Day); 8] = [
    ((2025, 4, 10), (2025, 7, 20)),
    ((2025, 7, 10), (2025, 10, 20)),
    ((2025, 10, 10), (2026, 1, 20)),
    ((2026, 1, 10), (2026, 4, 20)),
    ((2026, 4, 10), (2026, 7, 20)),
    ((2026, 7, 10), (2026, 10, 20)),
    ((2026, 10, 10), (2027, 1, 20)),
    ((2027, 1, 10), (2027, 4, 20)),
];

/// What the committee printed for the fitted rules: the Vikram year, the
/// rule, the Gregorian day, and the page and the words it prints.
const PRINTED: [(i32, &str, Day, &str); 132] = [
    (
        2082,
        "PARASHURAMA_JAYANTI",
        (2025, 4, 29),
        "p. 3: परशुरामजयन्ती",
    ),
    (
        2082,
        "SHANKARACHARYA_JAYANTI",
        (2025, 5, 2),
        "p. 3: आद्यजगत्गुरु शङ्कराचार्यजयन्ती",
    ),
    (2082, "GANGA_SAPTAMI", (2025, 5, 4), "p. 3: गंगासप्तमी"),
    (2082, "SITA_NAVAMI", (2025, 5, 6), "p. 3: सीताजयन्ती"),
    (2082, "KURMA_JAYANTI", (2025, 5, 12), "p. 3: कूर्मजयन्ती"),
    (
        2082,
        "DASHAHARA_SNANA_ARAMBHA",
        (2025, 5, 28),
        "p. 5: दशहरास्नानारम्भ",
    ),
    (
        2082,
        "GOSAIKUNDA_SNANA_ARAMBHA",
        (2025, 5, 28),
        "p. 5: गोसाइकुण्डस्नानारम्भ",
    ),
    (2082, "KUMARA_JATRA", (2025, 6, 2), "p. 5: कुमारयात्रा"),
    (2082, "GANGA_DASHAHARA", (2025, 6, 5), "p. 5: गङ्गादशहरा"),
    (
        2082,
        "GOSAIKUNDA_SNANA_SAMAPTI",
        (2025, 6, 5),
        "p. 5: गोसाइँकुण्ड स्नानसमाप्ति",
    ),
    (
        2082,
        "HATYAMOCHANA_SNANA",
        (2025, 6, 5),
        "p. 5: हत्यामोचन तीर्थस्नान",
    ),
    (
        2082,
        "NIRJALA_EKADASHI",
        (2025, 6, 7),
        "p. 5: निर्जला एकादशीव्रत",
    ),
    (2082, "DILA_CHARHE", (2025, 6, 24), "p. 6: (दिल्ला चःह्रे पूजा)"),
    (
        2082,
        "HILE_JATRA",
        (2025, 8, 17),
        "p. 10: रामेछाप चिसापानी हिलेजात्रा",
    ),
    (2082, "AJA_EKADASHI", (2025, 8, 19), "p. 10: अजाएकादशीव्रत"),
    (2082, "CHATHA", (2025, 8, 26), "p. 11: चन्द्रदोषदर्शन (चथा)"),
    (
        2082,
        "GANESHA_CHATURTHI",
        (2025, 8, 27),
        "p. 11: गणेशचतुर्थीव्रत",
    ),
    (2082, "RISHI_PANCHAMI", (2025, 8, 28), "p. 11: ऋषिपञ्चमीव्रत"),
    (2082, "GAURA_SAPTAMI", (2025, 8, 30), "p. 11: गौरासप्तमी"),
    (
        2082,
        "AMUKTABHARANA_SAPTAMI",
        (2025, 8, 30),
        "p. 11: अमुक्ताभरणसप्तमी",
    ),
    (2082, "DURVA_ASHTAMI", (2025, 8, 31), "p. 11: दूर्वाष्टमीव्रत"),
    (2082, "VAMANA_DWADASHI", (2025, 9, 4), "p. 11: वामनद्वादशी"),
    (
        2082,
        "PRATIPADA_SHRADDHA",
        (2025, 9, 8),
        "p. 12: प्रतिपदाश्राद्ध",
    ),
    (
        2082,
        "SORHA_SHRADDHA_ARAMBHA",
        (2025, 9, 8),
        "p. 12: सोह्रश्राद्ध सुरु",
    ),
    (
        2082,
        "DWITIYA_SHRADDHA",
        (2025, 9, 9),
        "p. 12: द्वितीयाश्राद्ध",
    ),
    (
        2082,
        "TRITIYA_SHRADDHA",
        (2025, 9, 10),
        "p. 12: तृतीयाश्राद्ध (गातिला)",
    ),
    (
        2082,
        "CHATURTHI_SHRADDHA",
        (2025, 9, 11),
        "p. 12: चतुर्थीश्राद्ध",
    ),
    (
        2082,
        "PANCHAMI_SHRADDHA",
        (2025, 9, 12),
        "p. 12: पञ्चमीश्राद्ध",
    ),
    (2082, "SHASHTHI_SHRADDHA", (2025, 9, 12), "p. 12: षष्ठीश्राद्ध"),
    (2082, "SAPTAMI_SHRADDHA", (2025, 9, 13), "p. 12: सप्तमीश्राद्ध"),
    (2082, "ASHTAMI_SHRADDHA", (2025, 9, 14), "p. 12: अष्टमीश्राद्ध"),
    (
        2082,
        "NAVAMI_SHRADDHA",
        (2025, 9, 15),
        "p. 12: नवमीश्राद्ध, मातृनवमीश्राद्ध",
    ),
    (2082, "DASHAMI_SHRADDHA", (2025, 9, 16), "p. 12: दशमीश्राद्ध"),
    (
        2082,
        "EKADASHI_SHRADDHA",
        (2025, 9, 17),
        "p. 12: एकादशीश्राद्ध",
    ),
    (
        2082,
        "DWADASHI_SHRADDHA",
        (2025, 9, 18),
        "p. 12: द्वादशीश्राद्ध",
    ),
    (
        2082,
        "TRAYODASHI_SHRADDHA",
        (2025, 9, 19),
        "p. 12: त्रयोदशीश्राद्ध",
    ),
    (2082, "KALI_YUGADI", (2025, 9, 19), "p. 12: कलियुगादि"),
    (
        2082,
        "CHATURDASHI_SHRADDHA",
        (2025, 9, 20),
        "p. 12: शस्त्रादिबाट निधन भएकाहरूको विश्वेदेवासहित एकोदिष्ट विधिले चतुर्दशीश्राद्ध",
    ),
    (
        2082,
        "AMAVASYA_SHRADDHA",
        (2025, 9, 21),
        "p. 12: औंसीश्राद्ध, पितृविसर्जन",
    ),
    (
        2082,
        "UPANGA_LALITA_VRATA",
        (2025, 9, 26),
        "p. 13: उपाङ्गललिताव्रत",
    ),
    (2082, "KUCHHI_BHOYE", (2025, 9, 30), "p. 13: (कूछि भ्वय्)"),
    (
        2082,
        "DHANVANTARI_JAYANTI",
        (2025, 10, 19),
        "p. 14: धन्वन्तरी जयन्ती",
    ),
    (2082, "MHA_PUJA", (2025, 10, 22), "p. 15: म्हपूजा"),
    (
        2082,
        "NEPAL_SAMBAT_NEW_YEAR",
        (2025, 10, 22),
        "p. 15: ने.सं.११४६ प्रारम्भ",
    ),
    (
        2082,
        "HARIBODHINI_EKADASHI",
        (2025, 11, 1),
        "p. 15: हरिबोधिनीएकादशीव्रत",
    ),
    (
        2082,
        "BHISHMA_PANCHAKA_ARAMBHA",
        (2025, 11, 1),
        "p. 15: भीष्मपञ्चकारम्भ",
    ),
    (2082, "TULASI_VIVAHA", (2025, 11, 1), "p. 15: तुलसीविवाह"),
    (
        2082,
        "PASHUPATI_HALESI_MELA",
        (2025, 11, 18),
        "p. 16: हलेसीमहादेवमेला",
    ),
    (
        2082,
        "BALA_CHATURDASHI",
        (2025, 11, 19),
        "p. 16: बालाचतुर्दशी",
    ),
    (
        2082,
        "PISHACHAMOCHANA_SHRADDHA",
        (2025, 12, 3),
        "p. 17: काशीमा पिशाचमोचन तीर्थश्राद्ध",
    ),
    (
        2082,
        "DATTATREYA_JAYANTI",
        (2025, 12, 4),
        "p. 17: दत्तात्रेयजयन्ती",
    ),
    (
        2082,
        "NUWAKOT_DUPCHESHWAR_MELA",
        (2025, 12, 4),
        "p. 17: नुवाकोट दुप्चेश्वरमेला",
    ),
    (2082, "GAIDU_PUJA", (2025, 12, 4), "p. 17: गैडुपूजा"),
    (
        2082,
        "PAUSHA_PUTRADA_EKADASHI",
        (2025, 12, 30),
        "p. 19: पुत्रदाएकादशीव्रत",
    ),
    (2082, "LAI_CHARHE", (2026, 1, 17), "p. 20: (लैचःह्रेपूजा)"),
    (
        2082,
        "TILAKUNDA_CHATURTHI",
        (2026, 1, 22),
        "p. 21: तिलकुन्द चौथी",
    ),
    (
        2082,
        "BHIMA_EKADASHI",
        (2026, 1, 29),
        "p. 21: भीमा एकादशीव्रत",
    ),
    (
        2082,
        "VIJAYA_EKADASHI",
        (2026, 2, 13),
        "p. 22: विजया एकादशीव्रत",
    ),
    (2082, "SHILA_CHARHE", (2026, 2, 15), "p. 22: (शिलाचःह्रेपूजा)"),
    (2082, "DVAPARA_YUGADI", (2026, 2, 17), "p. 22: द्वापरयुगादि"),
    (2082, "PAHAN_CHARHE", (2026, 3, 17), "p. 24: (पाशाचःह्रेपूजा)"),
    (2082, "GHODE_JATRA", (2026, 3, 18), "p. 24: घोडेजात्रा"),
    (
        2082,
        "MATSYANARAYANA_MELA",
        (2026, 3, 21),
        "p. 25: मत्स्यनारायणमेला",
    ),
    (2082, "MATSYA_JAYANTI", (2026, 3, 21), "p. 25: मत्स्यजयन्ती"),
    (2082, "CHAITE_DASHAIN", (2026, 3, 26), "p. 25: चैतेदशैं"),
    (
        2082,
        "SETO_MACHHINDRANATH_JATRA",
        (2026, 3, 26),
        "p. 25: श्रीश्वेतमत्स्येन्द्रनाथ रथयात्रा",
    ),
    (
        2082,
        "VAISHAKHA_SNANA_ARAMBHA",
        (2026, 4, 2),
        "p. 25: वैशाखस्नान सुरु",
    ),
    (
        2083,
        "PARASHURAMA_JAYANTI",
        (2026, 4, 19),
        "p. 3: परशुरामजयन्ती",
    ),
    (
        2083,
        "SHANKARACHARYA_JAYANTI",
        (2026, 4, 21),
        "p. 3: आद्यजगत्गुरु शङ्कराचार्यजयन्ती",
    ),
    (2083, "GANGA_SAPTAMI", (2026, 4, 23), "p. 3: गंगासप्तमी"),
    (2083, "SITA_NAVAMI", (2026, 4, 25), "p. 3: सीताजयन्ती"),
    (2083, "KURMA_JAYANTI", (2026, 5, 1), "p. 3: कूर्मजयन्ती"),
    (
        2083,
        "DASHAHARA_SNANA_ARAMBHA",
        (2026, 5, 17),
        "p. 5: दशहरास्नानारम्भ",
    ),
    (
        2083,
        "GOSAIKUNDA_SNANA_ARAMBHA",
        (2026, 5, 17),
        "p. 5: गोसाइकुण्डस्नानारम्भ",
    ),
    (2083, "GANGA_DASHAHARA", (2026, 5, 26), "p. 5: गङ्गादशहरा"),
    (
        2083,
        "GOSAIKUNDA_SNANA_SAMAPTI",
        (2026, 5, 26),
        "p. 5: गोसाइँकुण्ड स्नानसमाप्ति",
    ),
    (
        2083,
        "HATYAMOCHANA_SNANA",
        (2026, 5, 26),
        "p. 5: हत्यामोचन तीर्थस्नान",
    ),
    (2083, "ANALA_PUNHI", (2026, 5, 31), "p. 5: अनलापुन्हि"),
    (2083, "KUMARA_JATRA", (2026, 6, 21), "p. 7: कुमारयात्रा"),
    (
        2083,
        "NIRJALA_EKADASHI",
        (2026, 6, 25),
        "p. 7: निर्जला एकादशीव्रत",
    ),
    (2083, "DILA_CHARHE", (2026, 7, 13), "p. 8: (दिल्ला चःह्रे पूजा)"),
    (2083, "GAURA_SAPTAMI", (2026, 9, 3), "p. 12: गौरासप्तमी"),
    (
        2083,
        "AMUKTABHARANA_SAPTAMI",
        (2026, 9, 3),
        "p. 12: अमुक्ताभरणसप्तमी",
    ),
    (2083, "DURVA_ASHTAMI", (2026, 9, 4), "p. 12: दूर्वाष्टमीव्रत"),
    (
        2083,
        "HILE_JATRA",
        (2026, 9, 5),
        "p. 12: रामेछाप चिसापानी हिलेजात्रा",
    ),
    (2083, "AJA_EKADASHI", (2026, 9, 7), "p. 12: अजाएकादशीव्रत"),
    (2083, "CHATHA", (2026, 9, 14), "p. 13: चन्द्रदर्शनदोष (चथा)"),
    (
        2083,
        "GANESHA_CHATURTHI",
        (2026, 9, 14),
        "p. 13: गणेशचतुर्थीव्रत",
    ),
    (2083, "RISHI_PANCHAMI", (2026, 9, 15), "p. 13: ऋषिपञ्चमीव्रत"),
    (2083, "VAMANA_DWADASHI", (2026, 9, 23), "p. 13: वामनजयन्ती"),
    (
        2083,
        "PRATIPADA_SHRADDHA",
        (2026, 9, 27),
        "p. 14: प्रतिपदाश्राद्ध",
    ),
    (
        2083,
        "SORHA_SHRADDHA_ARAMBHA",
        (2026, 9, 27),
        "p. 14: सोह्रश्राद्ध सुरु",
    ),
    (
        2083,
        "DWITIYA_SHRADDHA",
        (2026, 9, 28),
        "p. 14: द्वितीयाश्राद्ध",
    ),
    (2083, "TRITIYA_SHRADDHA", (2026, 9, 29), "p. 14: तृतीयाश्राद्ध"),
    (
        2083,
        "CHATURTHI_SHRADDHA",
        (2026, 9, 30),
        "p. 14: चतुर्थीश्राद्ध",
    ),
    (
        2083,
        "PANCHAMI_SHRADDHA",
        (2026, 10, 1),
        "p. 14: पञ्चमीश्राद्ध",
    ),
    (2083, "SHASHTHI_SHRADDHA", (2026, 10, 1), "p. 14: षष्ठीश्राद्ध"),
    (2083, "SAPTAMI_SHRADDHA", (2026, 10, 2), "p. 14: सप्तमीश्राद्ध"),
    (2083, "ASHTAMI_SHRADDHA", (2026, 10, 3), "p. 14: अष्टमीश्राद्ध"),
    (
        2083,
        "NAVAMI_SHRADDHA",
        (2026, 10, 4),
        "p. 14: नवमीश्राद्ध, मातृनवमीश्राद्ध",
    ),
    (2083, "DASHAMI_SHRADDHA", (2026, 10, 5), "p. 14: दशमीश्राद्ध"),
    (
        2083,
        "EKADASHI_SHRADDHA",
        (2026, 10, 6),
        "p. 14: एकादशीश्राद्ध",
    ),
    (
        2083,
        "DWADASHI_SHRADDHA",
        (2026, 10, 7),
        "p. 14: द्वादशीश्राद्ध",
    ),
    (
        2083,
        "TRAYODASHI_SHRADDHA",
        (2026, 10, 8),
        "p. 14: त्रयोदशीश्राद्ध",
    ),
    (2083, "KALI_YUGADI", (2026, 10, 8), "p. 14: कलियुगादि"),
    (
        2083,
        "CHATURDASHI_SHRADDHA",
        (2026, 10, 9),
        "p. 14: शस्त्रादिबाट निधन भएकाहरूको विश्वेदेवासहित एकोदिष्ट विधिले चतुर्दशीश्राद्ध",
    ),
    (
        2083,
        "AMAVASYA_SHRADDHA",
        (2026, 10, 10),
        "p. 14: औंसीश्राद्ध, पितृविसर्जन",
    ),
    (
        2083,
        "UPANGA_LALITA_VRATA",
        (2026, 10, 15),
        "p. 15: उपाङ्गललिताव्रत",
    ),
    (2083, "KUCHHI_BHOYE", (2026, 10, 18), "p. 15: (कूछि भ्वय्)"),
    (
        2083,
        "DHANVANTARI_JAYANTI",
        (2026, 11, 7),
        "p. 16: धन्वन्तरी जयन्ती",
    ),
    (2083, "MHA_PUJA", (2026, 11, 10), "p. 17: म्हपूजा"),
    (
        2083,
        "NEPAL_SAMBAT_NEW_YEAR",
        (2026, 11, 10),
        "p. 17: ने.सं.११४७ प्रारम्भ",
    ),
    (
        2083,
        "HARIBODHINI_EKADASHI",
        (2026, 11, 20),
        "p. 17: हरिबोधिनीएकादशीव्रत",
    ),
    (
        2083,
        "BHISHMA_PANCHAKA_ARAMBHA",
        (2026, 11, 20),
        "p. 17: भीष्मपञ्चकारम्भ",
    ),
    (2083, "TULASI_VIVAHA", (2026, 11, 20), "p. 17: तुलसीविवाह"),
    (
        2083,
        "PASHUPATI_HALESI_MELA",
        (2026, 12, 6),
        "p. 18: हलेसीमहादेव मेला",
    ),
    (2083, "BALA_CHATURDASHI", (2026, 12, 7), "p. 18: बालाचतुर्दशी"),
    (
        2083,
        "PISHACHAMOCHANA_SHRADDHA",
        (2026, 12, 22),
        "p. 19: काशीमा पिशाचमोचन तीर्थश्राद्ध",
    ),
    (
        2083,
        "DATTATREYA_JAYANTI",
        (2026, 12, 23),
        "p. 19: दत्तात्रेयजयन्ती",
    ),
    (
        2083,
        "NUWAKOT_DUPCHESHWAR_MELA",
        (2026, 12, 24),
        "p. 19: नुवाकोट दुप्चेश्वरमेला",
    ),
    (2083, "GAIDU_PUJA", (2026, 12, 24), "p. 19: गैडुपूजा"),
    (
        2083,
        "PAUSHA_PUTRADA_EKADASHI",
        (2027, 1, 18),
        "p. 21: स्मार्तहरुको पुत्रदाएकादशीव्रत",
    ),
    (2083, "LAI_CHARHE", (2027, 2, 5), "p. 22: (लैचःह्रेपूजा)"),
    (
        2083,
        "TILAKUNDA_CHATURTHI",
        (2027, 2, 10),
        "p. 23: तिलकुन्द चौथी",
    ),
    (
        2083,
        "BHIMA_EKADASHI",
        (2027, 2, 17),
        "p. 23: भीमा एकादशीव्रत",
    ),
    (
        2083,
        "VIJAYA_EKADASHI",
        (2027, 3, 4),
        "p. 24: विजया एकादशीव्रत",
    ),
    (2083, "SHILA_CHARHE", (2027, 3, 6), "p. 24: (शिलाचःह्रेपूजा)"),
    (2083, "DVAPARA_YUGADI", (2027, 3, 8), "p. 24: द्वापरयुगादि"),
    (2083, "PAHAN_CHARHE", (2027, 4, 5), "p. 26: (पाशाचःह्रेपूजा)"),
    (2083, "GHODE_JATRA", (2027, 4, 6), "p. 26: घोडेजात्रा"),
    (
        2083,
        "MATSYANARAYANA_MELA",
        (2027, 4, 9),
        "p. 27: मत्स्यनारायणमेला",
    ),
    (2083, "MATSYA_JAYANTI", (2027, 4, 9), "p. 27: मत्स्यजयन्ती"),
];

/// Why the pack over the committee's sky parts from the print, per (year,
/// rule): the day it gives within a fortnight of the printed one, or none,
/// and the cause. The pass fails on a parting with no entry, on an entry
/// whose row agrees, and on an entry whose day is not the one found.
const PARTS: [(i32, &str, Option<Day>, &str); 18] = [
    (
        2082,
        "HILE_JATRA",
        Some((2025, 9, 1)),
        "the print's हिलेजात्रा is Ramechhap's, on Shravana's dark 9th; the rule's Bhadrapada bright 9th prints no Hile jatra (C609)",
    ),
    (
        2082,
        "PISHACHAMOCHANA_SHRADDHA",
        Some((2025, 12, 4)),
        "the 14th begins in the earlier day's second fifth, inside purvahna and after its first fifth, which the rule reads until DAY_MUHURTA (C602)",
    ),
    (
        2082,
        "SITA_NAVAMI",
        Some((2025, 5, 5)),
        "the 9th begins inside the earlier day's madhyahna and ends inside the later's, and the later holds more; the cases cannot weigh two parts, so the earlier stands until GREATER (C601)",
    ),
    (
        2082,
        "UPANGA_LALITA_VRATA",
        Some((2025, 9, 27)),
        "purvahna is the daylight's first two fifths; read as its first fifth, the later day holds it wholly, where the earlier holds more of the two fifths (C602)",
    ),
    (
        2083,
        "AMUKTABHARANA_SAPTAMI",
        Some((2026, 9, 18)),
        "counted from GAURA_SAPTAMI (C608)",
    ),
    (
        2083,
        "ANALA_PUNHI",
        None,
        "the print's अनलापुन्हि is the adhika month's full moon, and the rule keeps the nija Jyeshtha's (C607)",
    ),
    (
        2083,
        "DASHAHARA_SNANA_ARAMBHA",
        None,
        "the committee prints it in the adhika Jyeshtha, and the rule keeps the nija month (C606)",
    ),
    (
        2083,
        "DURVA_ASHTAMI",
        Some((2026, 9, 19)),
        "the committee prints Shravana's dark 8th, inside the solar Bhadra, where Bhadrapada's bright 8th falls in the solar Ashwin (C608)",
    ),
    (
        2083,
        "GANGA_DASHAHARA",
        None,
        "the committee prints it in the adhika Jyeshtha, and the rule keeps the nija month (C606)",
    ),
    (
        2083,
        "GAURA_SAPTAMI",
        Some((2026, 9, 18)),
        "the committee prints Shravana's dark 7th, inside the solar Bhadra, where Bhadrapada's bright 7th falls in the solar Ashwin (C608)",
    ),
    (
        2083,
        "GOSAIKUNDA_SNANA_ARAMBHA",
        None,
        "counted from DASHAHARA_SNANA_ARAMBHA, in the adhika Jyeshtha by the print (C606)",
    ),
    (
        2083,
        "GOSAIKUNDA_SNANA_SAMAPTI",
        None,
        "counted from GANGA_DASHAHARA, in the adhika Jyeshtha by the print (C606)",
    ),
    (
        2083,
        "HATYAMOCHANA_SNANA",
        None,
        "counted from GANGA_DASHAHARA, in the adhika Jyeshtha by the print (C606)",
    ),
    (
        2083,
        "HILE_JATRA",
        Some((2026, 9, 20)),
        "the print's हिलेजात्रा is Ramechhap's, on Shravana's dark 9th; the rule's Bhadrapada bright 9th prints no Hile jatra (C609)",
    ),
    (
        2083,
        "MHA_PUJA",
        Some((2026, 11, 9)),
        "the 1st begins inside the earlier day's madhyahna and ends inside the later's, and the later holds more; the earlier stands until GREATER (C601)",
    ),
    (
        2083,
        "NEPAL_SAMBAT_NEW_YEAR",
        Some((2026, 11, 9)),
        "counted from MHA_PUJA, which is a day early (C601)",
    ),
    (
        2083,
        "NUWAKOT_DUPCHESHWAR_MELA",
        Some((2026, 12, 23)),
        "the committee prints the mela on the full moon, a day after Dattatreya's 14th (C611)",
    ),
    (
        2083,
        "PISHACHAMOCHANA_SHRADDHA",
        Some((2026, 12, 23)),
        "the 14th begins in the earlier day's second fifth, inside purvahna and after its first fifth, which the rule reads until DAY_MUHURTA (C602)",
    ),
];

pub(super) struct Row {
    year: i32,
    rule: &'static str,
    printed: Day,
    page: &'static str,
    /// The pack over the committee's sky.
    committee: Option<Observance>,
    /// The pack over the modern sky.
    modern: Option<Observance>,
}

/// Every span's observances over a sky, a worker per span.
fn found(sky: Sky, request: &FestivalRequest) -> Result<Observances, String> {
    let (place, clock) = kathmandu();
    std::thread::scope(|scope| {
        let workers: Vec<_> = SPANS
            .iter()
            .map(|&(from, to)| {
                scope.spawn(move || {
                    sky.context()?
                        .almanac()
                        .festivals(&gregorian(from), &gregorian(to), &place, clock, request)
                        .map(|found| found.value)
                        .map_err(|e| format!("{from:?}: {e}"))
                })
            })
            .collect();
        workers
            .into_iter()
            .try_fold(Observances::default(), |all, worker| {
                let part = worker
                    .join()
                    .map_err(|_| String::from("a worker finding the fitted rules panicked"))??;
                Ok(all.merged(part))
            })
    })
}

fn found_day(found: Option<&Observance>) -> Option<Day> {
    found.map(|o| (o.day.year, o.day.month, o.day.day))
}

/// Each printed row beside the day the pack gives over each sky, with a
/// problem for every parting under the committee's sky without its cause,
/// and every cause without its parting or naming another day.
pub(super) fn compare(problems: &mut Vec<String>) -> Result<Vec<Row>, String> {
    let request = FestivalRequest::from(FestivalPack::NepalPanchanga);
    let committee = found(Sky::Committee, &request)?;
    let modern = found(Sky::Modern, &request)?;
    let rows: Vec<Row> = PRINTED
        .iter()
        .map(|&(year, rule, printed, page)| Row {
            year,
            rule,
            printed,
            page,
            committee: near(&committee, rule, printed),
            modern: near(&modern, rule, printed),
        })
        .collect();
    for row in &rows {
        let excuse = PARTS.iter().find(|p| (p.0, p.1) == (row.year, row.rule));
        let given = found_day(row.committee.as_ref());
        match (on(row.committee.as_ref(), row.printed), excuse) {
            (false, None) => problems.push(format!(
                "VS {} {} printed {:?}, and the NEPAL_PANCHANGA pack over the committee's sky gives {given:?}: a parting needs its cause in panchanga::PARTS",
                row.year, row.rule, row.printed
            )),
            (true, Some(_)) => problems.push(format!(
                "VS {} {} agrees and is excused in panchanga::PARTS: remove the entry",
                row.year, row.rule
            )),
            (false, Some(&(_, _, excused, _))) if excused != given => problems.push(format!(
                "VS {} {} is excused as giving {excused:?}, and the pack gives {given:?}",
                row.year, row.rule
            )),
            _ => {}
        }
    }
    Ok(rows)
}

fn day_of(found: Option<&Observance>, printed: Day) -> String {
    found.map_or_else(
        || "none ✗".to_owned(),
        |o| {
            format!(
                "{}{}",
                spell((o.day.month, o.day.day)),
                if on(Some(o), printed) { "" } else { " ✗" }
            )
        },
    )
}

/// §6 of the page.
pub(super) fn render(out: &mut String, rows: &[Row]) {
    out.push_str(
        "\n## 6. The fitted rules against the print\n\n\
         The days the committee printed for VS 2082 and 2083 that the\n\
         `NEPAL_PANCHANGA` pack adds to `NEPAL` (`festival-rules.md` §10), at\n\
         Kathmandu on Nepal's clock, beside the day the pack gives over the\n\
         committee's own sky (`nepali-committee`), each parting named with its\n\
         cause, and over the modern sky (`nepali-default`). Each rule is fitted:\n\
         its window and table were chosen to give these days, so the agreement\n\
         measures the fit and not a text (C600).\n\n",
    );
    out.push_str(
        "| VS | rule | printed | where | committee sky | modern sky |\n\
         |---:|---|---|---|---|---|\n",
    );
    for row in rows {
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} | {} |",
            row.year,
            row.rule,
            spell((row.printed.1, row.printed.2)),
            row.page,
            day_of(row.committee.as_ref(), row.printed),
            day_of(row.modern.as_ref(), row.printed),
        );
    }
    let parted = |pick: fn(&Row) -> Option<&Observance>| {
        rows.iter()
            .filter(|row| !on(pick(row), row.printed))
            .count()
    };
    out.push('\n');
    // The pass has refused a parting under the committee's sky without its
    // cause, so every one counted is named below.
    out.push_str(&table(&[
        Claim::counted(
            "the `NEPAL_PANCHANGA` pack over the committee's sky falls on the printed day",
            parted(|row| row.committee.as_ref()),
            rows.len(),
        )
        .with_note("each parting is named below with its cause"),
        Claim::counted(
            "the same over the modern sky falls on the printed day",
            parted(|row| row.modern.as_ref()),
            rows.len(),
        )
        .with_note("the sky decides these: a tithi's end moves the day where the two skies part"),
    ]));
    out.push('\n');
    out.push_str("Where the pack over the committee's sky parts from the print, and why:\n\n");
    for (year, rule, given, why) in PARTS {
        let given = given.map_or_else(
            || "none within a fortnight".to_owned(),
            |d| spell((d.1, d.2)),
        );
        let _ = writeln!(out, "- VS {year} {rule} ({given}): {why}");
    }
}

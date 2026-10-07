//! The antardaśā śāntis of BPHS chs. 37 to 45, as printed
//! (`03-design/remedies.md`, step 2, C348 to C350).
//!
//! Each chapter takes one mahādaśā lord and, for each of its nine
//! antardaśās, names a condition, an evil and the rite that answers it.
//! The rows are read from the 1923 Venkateśvara print on its page images.
//! Its two doubtful readings were checked against the 1899 print.
//!
//! Each condition is reported as the predicate the verse states, not
//! judged against a chart. Judging belongs with the subjects of a
//! remedy (step 3), which reads the chart. Where the verse's grammar
//! leaves the predicate open, the variant says so and quotes it.

use serde::Serialize;
use teistro_core::catalogue::Graha;

/// A condition an antardaśā's śānti is given under. Houses are counted
/// from the lagna unless a variant says otherwise.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Condition {
    /// The antardaśā lord is the lord of the 2nd or the 7th
    /// (*dvitīya-dyūna-nātha*, *dvitīya-saptamādhīśa*): 53 rows. Four
    /// rows print *dvitīye dyūna-nāthe* as two words, which is read as
    /// the same compound (C348).
    LordOfSecondOrSeventh,
    /// The antardaśā lord is the lord of the 7th
    /// (*saptamādhipa-doṣeṇa*).
    LordOfSeventh,
    /// The antardaśā graha stands in the 2nd or the 7th.
    InSecondOrSeventh,
    /// It stands in the 2nd, the 7th or the 8th
    /// (*dvitīya-dyūna-randhra-sthe*, Moon/Saturn).
    InSecondSeventhOrEighth,
    /// The 2nd or 7th lord, "*randhra-riṣpha-samanvite*": with the 8th
    /// or the 12th. The verse does not say whether that means placed
    /// there or joined with their lords.
    WithEighthOrTwelfth,
    /// "*dvitīye dyūna-nāthe tu randhre randhrādhipo yadā*"
    /// (Moon/Mars): the 2nd or 7th lord, when the 8th lord is in the
    /// 8th.
    EighthLordInEighth,
    /// In the 6th or the 8th from the mahādaśā lord, or debilitated, or
    /// joined with a malefic (Sun/Jupiter).
    SixthOrEighthFromDashaLordOrWeak,
    /// In the 6th, 8th or 12th from the mahādaśā lord **and** joined
    /// with a malefic (Mars/Saturn).
    DusthanaFromDashaLordWithMalefic,
    /// The node in the 2nd or the 7th **and** joined with that house's
    /// lord (Sun/Rahu).
    InSecondOrSeventhWithItsLord,
    /// "*dvitīya-dyūna-nāthena saṃbandhe tatra saṃsthite*" (Ketu/Ketu):
    /// related to the 2nd or 7th lord and placed there.
    RelatedToSecondOrSeventhLord,
    /// "*nidhanādhipa-doṣeṇa*", the 8th lord's fault, joined with the
    /// 2nd or 7th lord (Ketu/Moon).
    EighthLordWithSecondOrSeventhLord,
    /// The node joined with the lords of the 2nd and the 7th
    /// (Saturn/Rahu).
    WithSecondOrSeventhLords,
    /// The lord of the 8th or the 7th, or standing in the 2nd
    /// (Saturn/Mars).
    LordOfEighthOrSeventhOrInSecond,
    /// The lord of the 2nd and the 6th, as the 1923 print reads
    /// Jupiter/Moon. The 1899 print reads *ṣaṣṭhamādhīśe*, which is not
    /// a well-formed compound and is one syllable from *saptamādhīśe*,
    /// "the 7th lord" (C349).
    LordOfSecondAndSixth,
    /// "*dvitīya-dyūna-nāthe tu saptama-sthānam āśritaḥ*" (Rahu/Rahu):
    /// a locative, then a nominative, so the verse does not say which
    /// stands in the 7th.
    SecondOrSeventhLordInSeventh,
    /// The 2nd or 7th lord, then "*yutekṣite*", joined or aspected,
    /// without naming by what (Jupiter/Venus).
    JoinedOrAspected,
}

/// A rite or gift an antardaśā's śānti prescribes, keyed by the verse's
/// own words.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Remedy {
    /// *mṛtyuñjaya-japa*: the Mṛtyuñjaya mantra.
    MrityunjayaJapa,
    /// *mahā-mṛtyuñjaya-japa*.
    MahaMrityunjayaJapa,
    /// *rudra-jāpya*.
    RudraJapa,
    /// *durgā-japa*.
    DurgaJapa,
    /// *durgā-devī-japa*.
    DurgaDeviJapa,
    /// *durgā-lakṣmī-japa*.
    DurgaLakshmiJapa,
    /// *durgā-pāṭha*: the Durgā text recited.
    DurgaPatha,
    /// *viṣṇu-sāhasraka*: Viṣṇu's thousand names.
    VishnuSahasranama,
    /// *śiva-sāhasraka*: Śiva's thousand names.
    ShivaSahasranama,
    /// *āditya-hṛdaya*: the hymn to the Sun.
    AdityaHridayaJapa,
    /// *iṣṭa-jāpya*: japa to one's chosen deity.
    IshtaJapa,
    /// "*subrahma-japa-dāna*" as printed (Sun/Mars); the word is unclear
    /// and is kept as it stands (C348).
    SubrahmaJapaDana,
    /// *śānti*, by rule (*yathāvidhi*, *vidhivat*, *vidhānataḥ*).
    Shanti,
    /// A śānti by one's own *gṛhya* rules.
    GrihyaShanti,
    /// *śānti-homa*.
    ShantiHoma,
    /// *ayuta-homa*: ten thousand oblations.
    AyutaHoma,
    /// *tila-homa*: oblations of sesame.
    TilaHoma,
    /// *darśa-śānti*, as both prints read Ketu/Sun: a śānti of the new
    /// moon (C350).
    DarshaShanti,
    /// A rite pleasing the Sun (*sūrya-prīti*).
    SuryaPriti,
    /// *sūrya-pūjā*.
    SuryaPuja,
    /// A rite pleasing the Moon (*candra-prītikara*).
    ChandraPriti,
    /// A gift pleasing Mercury (*budha-prītikara dāna*).
    BudhaPritiDana,
    /// *śiva-pūjā*.
    ShivaPuja,
    /// *brāhmaṇa-arcana*: worship of a Brahmin.
    BrahmanaArcana,
    /// Feeding gods and Brahmins.
    DevaBrahmanaBhojana,
    /// *chāga*: a goat.
    ChagaDana,
    /// *śvetā gauḥ* and *mahiṣī*: a white cow and a she-buffalo, which
    /// the text always gives together.
    ShvetaGoMahishi,
    /// *kṛṣṇā gauḥ* and *mahiṣī*: a black cow and a she-buffalo.
    KrishnaGoMahishi,
    /// *śvetā gauḥ* and *rajata*: a white cow and silver (Moon/Venus).
    ShvetaGoRajata,
    /// *go-dāna*: a cow.
    GoDana,
    /// *dhenu*: a milch cow.
    DhenuDana,
    /// Tawny (*kapila*) cows.
    KapilaGoDana,
    /// *svarṇa-dhenu*: a cow of gold.
    SvarnaDhenu,
    /// Cow, land and gold.
    GoBhuHiranyaDana,
    /// *anaḍvān*: a draught bull.
    AnadvanDana,
    /// *aśva*: a horse.
    AshvaDana,
    /// *nāga-dāna*: an image of a serpent.
    NagaDana,
    /// *svarṇa*, *hiraṇya*: gold.
    SvarnaDana,
    /// An image of gold.
    SvarnaPratimaDana,
    /// An image of silver.
    RajataPratimaDana,
    /// *anna*: food.
    AnnaDana,
    /// *vastra*: cloth.
    VastraDana,
    /// Jaggery, ghee, curd and rice.
    GudaGhritaDadhiTandula,
}

/// One antardaśā's śānti, as printed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct DashaShanti {
    /// The mahādaśā lord.
    pub mahadasha: Graha,
    /// The antardaśā lord.
    pub antardasha: Graha,
    /// The chapter of BPHS (the 1923 print), 37 to 45.
    pub chapter: u8,
    /// The verses that hold the condition and the rite.
    pub verses: &'static str,
    /// The printed page.
    pub page: u16,
    /// The conditions, any of which brings the evil; empty when the
    /// verses name none.
    pub conditions: &'static [Condition],
    /// The rites, all of which are prescribed; empty when none is
    /// printed.
    pub remedies: &'static [Remedy],
}

const fn row(
    (mahadasha, antardasha): (Graha, Graha),
    (chapter, verses, page): (u8, &'static str, u16),
    conditions: &'static [Condition],
    remedies: &'static [Remedy],
) -> DashaShanti {
    DashaShanti {
        mahadasha,
        antardasha,
        chapter,
        verses,
        page,
        conditions,
        remedies,
    }
}

/// Every row, chapter by chapter in the printed order.
const ROWS: [DashaShanti; 81] = {
    use Condition as C;
    use Graha::{
        Jupiter as JU, Ketu as KE, Mars as MA, Mercury as ME, Moon as MO, Rahu as RA, Saturn as SA,
        Sun as SU, Venus as VE,
    };
    use Remedy as R;
    const P1: &[Condition] = &[C::LordOfSecondOrSeventh];
    const P2: &[Condition] = &[C::LordOfSeventh];
    const P3: &[Condition] = &[C::InSecondOrSeventh];
    [
        // Ch. 37, the Sun's mahādaśā.
        row(
            (SU, SU),
            (37, "21–22", 273),
            P1,
            &[R::MrityunjayaJapa, R::SuryaPriti],
        ),
        row(
            (SU, MO),
            (37, "32–33", 273),
            P1,
            &[R::ShvetaGoMahishi, R::Shanti],
        ),
        row(
            (SU, MA),
            (37, "40–41", 274),
            P1,
            &[R::SubrahmaJapaDana, R::AnadvanDana, R::Shanti],
        ),
        row(
            (SU, RA),
            (37, "48–50", 274),
            &[C::InSecondOrSeventhWithItsLord],
            &[R::DurgaJapa, R::ChagaDana, R::KrishnaGoMahishi],
        ),
        row(
            (SU, JU),
            (37, "56–58", 275),
            &[C::SixthOrEighthFromDashaLordOrWeak],
            &[R::SvarnaDana, R::IshtaJapa, R::KapilaGoDana],
        ),
        row(
            (SU, SA),
            (37, "65–66", 275),
            P1,
            &[R::KrishnaGoMahishi, R::MrityunjayaJapa, R::ChagaDana],
        ),
        row(
            (SU, ME),
            (37, "75", 276),
            P1,
            &[R::VishnuSahasranama, R::AnnaDana, R::RajataPratimaDana],
        ),
        row(
            (SU, KE),
            (37, "82–83", 277),
            P1,
            &[R::DurgaJapa, R::ChagaDana, R::MahaMrityunjayaJapa],
        ),
        row(
            (SU, VE),
            (37, "90–92", 277),
            &[C::LordOfSeventh, C::WithEighthOrTwelfth],
            &[R::MrityunjayaJapa, R::ShvetaGoMahishi, R::RudraJapa],
        ),
        // Ch. 38, the Moon's.
        row(
            (MO, MO),
            (38, "5–6", 278),
            &[C::WithEighthOrTwelfth],
            &[R::ShvetaGoMahishi],
        ),
        row(
            (MO, MA),
            (38, "12", 278),
            &[C::EighthLordInEighth],
            &[R::BrahmanaArcana],
        ),
        row((MO, RA), (38, "20–21", 279), P3, &[R::ChagaDana]),
        row(
            (MO, JU),
            (38, "31", 279),
            P1,
            &[R::ShivaSahasranama, R::SvarnaDana],
        ),
        row(
            (MO, SA),
            (38, "37–38", 280),
            &[C::InSecondSeventhOrEighth],
            &[R::MrityunjayaJapa, R::KrishnaGoMahishi],
        ),
        row(
            (MO, ME),
            (38, "45–46", 280),
            P1,
            &[R::ChagaDana, R::VishnuSahasranama],
        ),
        row((MO, KE), (38, "51–52", 281), P3, &[R::MrityunjayaJapa]),
        row(
            (MO, VE),
            (38, "62–64", 281),
            P1,
            &[R::RudraJapa, R::ShvetaGoRajata],
        ),
        row((MO, SU), (38, "69", 282), P1, &[R::ShivaPuja]),
        // Ch. 39, Mars's.
        row(
            (MA, MA),
            (39, "6–8", 282),
            P1,
            &[R::RudraJapa, R::AnadvanDana],
        ),
        row(
            (MA, RA),
            (39, "13–14", 283),
            P3,
            &[R::NagaDana, R::DevaBrahmanaBhojana, R::MrityunjayaJapa],
        ),
        row((MA, JU), (39, "22", 283), P1, &[R::ShivaSahasranama]),
        row(
            (MA, SA),
            (39, "33–35", 284),
            &[C::DusthanaFromDashaLordWithMalefic],
            &[R::MrityunjayaJapa],
        ),
        row(
            (MA, ME),
            (39, "46–47", 285),
            P1,
            &[R::AshvaDana, R::VishnuSahasranama],
        ),
        row((MA, KE), (39, "53–54", 285), P3, &[R::MrityunjayaJapa]),
        row((MA, VE), (39, "62–63", 286), P1, &[R::ShvetaGoMahishi]),
        row((MA, SU), (39, "68–69", 286), P1, &[R::Shanti]),
        row(
            (MA, MO),
            (39, "75–76", 287),
            P1,
            &[R::DurgaLakshmiJapa, R::ShvetaGoMahishi],
        ),
        // Ch. 40, Rahu's.
        row(
            (RA, RA),
            (40, "7", 287),
            &[C::SecondOrSeventhLordInSeventh],
            &[R::Shanti],
        ),
        row(
            (RA, JU),
            (40, "19–20", 288),
            P1,
            &[R::SvarnaPratimaDana, R::ShivaPuja, R::Shanti],
        ),
        row((RA, SA), (40, "29", 289), P1, &[R::KrishnaGoMahishi]),
        row(
            (RA, ME),
            (40, "38–39", 289),
            P1,
            &[R::VishnuSahasranama, R::GrihyaShanti],
        ),
        row((RA, KE), (40, "45", 290), P1, &[R::ChagaDana]),
        row((RA, VE), (40, "58–59", 291), P1, &[R::DurgaLakshmiJapa]),
        row((RA, SU), (40, "66–67", 291), P1, &[R::SuryaPriti]),
        row((RA, MO), (40, "74–75", 292), P1, &[R::ShvetaGoMahishi]),
        row((RA, MA), (40, "83", 292), P1, &[R::AnadvanDana, R::GoDana]),
        // Ch. 41, Jupiter's.
        row(
            (JU, JU),
            (41, "6–7", 293),
            P2,
            &[R::ShivaSahasranama, R::RudraJapa, R::GoDana],
        ),
        row(
            (JU, SA),
            (41, "18–19", 294),
            P1,
            &[R::VishnuSahasranama, R::KrishnaGoMahishi],
        ),
        row(
            (JU, ME),
            (41, "30–31", 294),
            P1,
            &[R::VishnuSahasranama, R::BudhaPritiDana, R::Shanti],
        ),
        row(
            (JU, KE),
            (41, "37–38", 295),
            P1,
            &[R::ChagaDana, R::MrityunjayaJapa, R::Shanti],
        ),
        row(
            (JU, VE),
            (41, "48–50", 296),
            &[C::JoinedOrAspected],
            &[R::Shanti, R::ShvetaGoMahishi],
        ),
        row(
            (JU, SU),
            (41, "56–57", 296),
            P1,
            &[R::AdityaHridayaJapa, R::SuryaPriti],
        ),
        row(
            (JU, MO),
            (41, "64", 297),
            &[C::LordOfSecondAndSixth],
            &[R::DurgaPatha],
        ),
        row((JU, MA), (41, "70–71", 297), P1, &[R::AnadvanDana]),
        row(
            (JU, RA),
            (41, "79–80", 298),
            P3,
            &[R::MrityunjayaJapa, R::ChagaDana],
        ),
        // Ch. 42, Saturn's.
        row((SA, SA), (42, "6–7", 298), P1, &[R::MrityunjayaJapa]),
        row(
            (SA, ME),
            (42, "14–15", 299),
            P1,
            &[R::VishnuSahasranama, R::AnnaDana],
        ),
        row((SA, KE), (42, "22–23", 299), P3, &[R::ChagaDana]),
        row(
            (SA, VE),
            (42, "35–36", 300),
            P1,
            &[R::DurgaDeviJapa, R::ShvetaGoMahishi],
        ),
        row((SA, SU), (42, "42", 300), P1, &[R::SuryaPuja]),
        row(
            (SA, MO),
            (42, "53–54", 301),
            P1,
            &[R::TilaHoma, R::GudaGhritaDadhiTandula, R::ShvetaGoMahishi],
        ),
        row(
            (SA, MA),
            (42, "61–62", 302),
            &[C::LordOfEighthOrSeventhOrInSecond],
            &[R::ShantiHoma, R::AnadvanDana],
        ),
        row(
            (SA, RA),
            (42, "69–70", 302),
            &[C::WithSecondOrSeventhLords],
            &[R::MrityunjayaJapa, R::ChagaDana, R::AnadvanDana],
        ),
        row(
            (SA, JU),
            (42, "81–83", 303),
            P1,
            &[R::ShivaSahasranama, R::SvarnaDana],
        ),
        // Ch. 43, Mercury's.
        row((ME, ME), (43, "4–5", 303), P1, &[R::VishnuSahasranama]),
        row((ME, KE), (43, "11–12", 304), P1, &[R::ChagaDana]),
        row((ME, VE), (43, "18–19", 304), P1, &[R::DurgaDeviJapa]),
        row(
            (ME, SU),
            (43, "24–25", 305),
            P1,
            &[R::Shanti, R::DhenuDana, R::SvarnaDana],
        ),
        row(
            (ME, MO),
            (43, "34–35", 305),
            P1,
            &[R::DurgaDeviJapa, R::VastraDana],
        ),
        row(
            (ME, MA),
            (43, "45–46", 306),
            P1,
            &[R::AnadvanDana, R::MrityunjayaJapa],
        ),
        row(
            (ME, RA),
            (43, "53–55", 306),
            P3,
            &[R::DurgaLakshmiJapa, R::ShvetaGoMahishi],
        ),
        row(
            (ME, JU),
            (43, "65–66", 307),
            P3,
            &[R::ShivaSahasranama, R::GoBhuHiranyaDana],
        ),
        row(
            (ME, SA),
            (43, "71–72", 308),
            P1,
            &[R::MrityunjayaJapa, R::KrishnaGoMahishi],
        ),
        // Ch. 44, Ketu's.
        row(
            (KE, KE),
            (44, "5–6", 308),
            &[C::RelatedToSecondOrSeventhLord],
            &[R::DurgaDeviJapa, R::MrityunjayaJapa],
        ),
        row(
            (KE, VE),
            (44, "14–15", 309),
            P1,
            &[R::DurgaDeviJapa, R::ShvetaGoMahishi],
        ),
        row(
            (KE, SU),
            (44, "23–24", 309),
            P1,
            &[R::DarshaShanti, R::SvarnaDhenu],
        ),
        row(
            (KE, MO),
            (44, "35–36", 310),
            &[C::EighthLordWithSecondOrSeventhLord],
            &[R::Shanti, R::ChandraPriti],
        ),
        row((KE, MA), (44, "42–44", 310), P1, &[R::AnadvanDana]),
        row(
            (KE, RA),
            (44, "49–50", 311),
            P3,
            &[R::DurgaDeviJapa, R::AyutaHoma],
        ),
        row(
            (KE, JU),
            (44, "59–60", 312),
            P1,
            &[R::ShivaSahasranama, R::MahaMrityunjayaJapa],
        ),
        row(
            (KE, SA),
            (44, "67–68", 312),
            P1,
            &[R::TilaHoma, R::KrishnaGoMahishi],
        ),
        row((KE, ME), (44, "78", 313), P1, &[R::VishnuSahasranama]),
        // Ch. 45, Venus's.
        row(
            (VE, VE),
            (45, "10–11", 314),
            P1,
            &[R::DurgaJapa, R::DhenuDana],
        ),
        row((VE, SU), (45, "19", 314), P2, &[R::SuryaPriti]),
        row((VE, MO), (45, "20–29", 314), &[], &[]),
        row((VE, MA), (45, "35", 315), P1, &[]),
        row((VE, RA), (45, "43–44", 316), P3, &[R::MrityunjayaJapa]),
        row((VE, JU), (45, "50–51", 316), P1, &[R::MahaMrityunjayaJapa]),
        row((VE, SA), (45, "57–59", 317), P1, &[R::TilaHoma, R::GoDana]),
        row((VE, ME), (45, "66", 317), P2, &[R::VishnuSahasranama]),
        row(
            (VE, KE),
            (45, "72–74", 318),
            P1,
            &[R::MrityunjayaJapa, R::ChagaDana],
        ),
    ]
};

/// Every antardaśā śānti BPHS chs. 37 to 45 print, chapter by chapter.
#[must_use]
pub fn dasha_shantis() -> &'static [DashaShanti] {
    &ROWS
}

/// The śānti BPHS prints for `antardasha` within `mahadasha`'s
/// mahādaśā, or `None` for a graha the Vimśottarī does not run (Uranus,
/// Neptune, Pluto).
///
/// ```
/// use teistro_core::catalogue::Graha;
/// use teistro_remedies::{Condition, Remedy, dasha_shanti};
///
/// // BPHS 37.82–83: the Sun's mahādaśā, Ketu's antardaśā, Ketu the 2nd or
/// // 7th lord: Durgā japa, a goat and the great Mṛtyuñjaya.
/// let read = dasha_shanti(Graha::Sun, Graha::Ketu).unwrap();
/// assert_eq!(read.conditions, [Condition::LordOfSecondOrSeventh]);
/// assert!(read.remedies.contains(&Remedy::ChagaDana));
/// ```
#[must_use]
pub fn dasha_shanti(mahadasha: Graha, antardasha: Graha) -> Option<&'static DashaShanti> {
    ROWS.iter()
        .find(|row| row.mahadasha == mahadasha && row.antardasha == antardasha)
}

//! What the texts prescribe for a graha: its graha-śānti
//! (`03-design/remedies.md`, step 2, C343 to C347).
//!
//! Four rank 1 texts are read on their pages:
//! - **BPHS ch. 84** (the 1952 print, cited by verse number): the image,
//!   the ṛk, the japa, the samidh, the food and the dakṣiṇā;
//! - **Yājñavalkya Smṛti I.295–307**, which gives the same rite verse
//!   for verse except for Rahu's ṛk and the japa;
//! - **Jataka Parijata II.21**: the gem each graha owns;
//! - **Brihat Jataka II.5 and II.12**: each graha's direction and
//!   substance.
//!
//! Each value is the verse's own word. A commentator's addition (a
//! yellow cloth, a cow with its calf) belongs to the page, not to the
//! value.

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Direction, Graha};

/// What a graha's image is made of (BPHS 84.4 = Yājñavalkya I.297).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ImageMaterial {
    /// *tāmra*, the Sun's.
    Copper,
    /// *sphaṭika*, the Moon's.
    Crystal,
    /// *raktacandana*, Mars's.
    RedSandalwood,
    /// *svarṇa*, Mercury's and Jupiter's ("*svarṇakāt ubhau*").
    Gold,
    /// *rajata*, Venus's.
    Silver,
    /// *ayas*, Saturn's.
    Iron,
    /// *sīsa*, Rahu's.
    Lead,
    /// *kāṃsya*, Ketu's: bell metal.
    Bronze,
}

/// The wood a graha's fire is fed with (BPHS 84.21 = Yājñavalkya I.302).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Samidh {
    /// *arka*, the Sun's.
    Arka,
    /// *palāśa*, the Moon's.
    Palasha,
    /// *khadira*, Mars's.
    Khadira,
    /// *apāmārga*, Mercury's.
    Apamarga,
    /// *pippala* (aśvattha), Jupiter's.
    Pippala,
    /// *udumbara*, Venus's.
    Udumbara,
    /// *śamī*, Saturn's.
    Shami,
    /// *dūrvā*, Rahu's.
    Durva,
    /// *kuśa*, Ketu's.
    Kusha,
}

/// The food Brahmins are fed for a graha (BPHS 84.23 = Yājñavalkya
/// I.304–305).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Food {
    /// *guḍaudana*, rice with jaggery: the Sun's.
    Gudaudana,
    /// *pāyasa*, rice boiled in milk: the Moon's.
    Payasa,
    /// *haviṣya*, food fit for an offering: Mars's.
    Havishya,
    /// *kṣīra-ṣāṣṭika*, sixty-day rice in milk: Mercury's.
    KshiraShashtika,
    /// *dadhyodana*, rice with curd: Jupiter's.
    Dadhyodana,
    /// *haviḥ*, which the 1952 gloss reads as rice with ghee: Venus's.
    Havis,
    /// *cūrṇa*, which the gloss reads as sesame powder with rice:
    /// Saturn's.
    Churna,
    /// *māṃsa*, meat: Rahu's.
    Mamsa,
    /// *citrānna*, mixed rice: Ketu's.
    Chitranna,
}

/// The fee given for a graha (BPHS 84.25 = Yājñavalkya I.306).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Dakshina {
    /// *dhenu*, a milch cow: the Sun's.
    MilchCow,
    /// *śaṅkha*, a conch: the Moon's.
    Conch,
    /// *anaḍvān*, a draught bull: Mars's.
    Bull,
    /// *hema*, gold: Mercury's.
    Gold,
    /// *vāsas*, cloth: Jupiter's.
    Cloth,
    /// *haya*, a horse: Venus's.
    Horse,
    /// *kṛṣṇā gauḥ*, a black cow: Saturn's.
    BlackCow,
    /// *āyasa*, iron, which the Mitākṣarā reads as an iron weapon:
    /// Rahu's.
    Iron,
    /// *chāga*, a goat: Ketu's. The 1918 English "sheep" mistranslates
    /// it (C344).
    Goat,
}

/// The gem a graha owns (Jataka Parijata II.21). The verse states
/// ownership only, and nothing about wearing a gem or giving it away.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Gem {
    /// *māṇikya*, ruby: the Sun's.
    Ruby,
    /// *muktāphala*, pearl: the Moon's.
    Pearl,
    /// *vidruma*, coral: Mars's.
    Coral,
    /// *marakata*, emerald: Mercury's.
    Emerald,
    /// *puṣparāga*, topaz: Jupiter's.
    Topaz,
    /// *vajra*, diamond: Venus's.
    Diamond,
    /// *nīla*, sapphire: Saturn's.
    Sapphire,
    /// *gomeda*, hessonite: Rahu's, the first of "the other two".
    Hessonite,
    /// *vaidūrya*, cat's eye: Ketu's (C347).
    CatsEye,
}

/// The substance a graha rules (Brihat Jataka II.12 = Jataka Parijata
/// II.20). Neither verse gives one to the nodes (C346).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Substance {
    /// *tāmra*: the Sun's.
    Copper,
    /// *maṇi*: the Moon's.
    Gems,
    /// *kāñcana*: Mars's.
    Gold,
    /// *yukti*, which Iyer renders as brass: Mercury's.
    Alloy,
    /// *raupya*: Jupiter's. Iyer's note gives him gold when he is in
    /// his own sign; the verse does not.
    Silver,
    /// *muktā*: Venus's.
    Pearls,
    /// *ayas*: Saturn's.
    Iron,
}

/// Where a graha's image stands in the Matsya maṇḍala, which the
/// Mitākṣarā quotes on Yājñavalkya I.298 for laying out the rite.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MandalaPlace {
    /// The centre: the Sun.
    Centre,
    /// Venus.
    East,
    /// The Moon.
    Southeast,
    /// Mars.
    South,
    /// Rahu.
    Southwest,
    /// Saturn.
    West,
    /// Ketu.
    Northwest,
    /// Jupiter.
    North,
    /// Mercury.
    Northeast,
}

/// Which text's ṛk is given for Rahu. The two texts agree for the other
/// eight grahas (C343).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RikSource {
    /// BPHS 84.18: *kayā naś citra*.
    #[default]
    Bphs,
    /// Yājñavalkya I.301: *kāṇḍāt*.
    Yajnavalkya,
}

/// The readings a graha-śānti is given under.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct ShantiRules {
    /// The text Rahu's ṛk is read from.
    pub rik: RikSource,
}

/// What the texts prescribe for one graha.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Shanti {
    /// The graha.
    pub graha: Graha,
    /// The material its image is made of (BPHS 84.4).
    pub image: ImageMaterial,
    /// The opening words of its Vedic ṛk in IAST, as BPHS 84.17–18
    /// prints them, or Yājñavalkya I.301 for Rahu under
    /// [`RikSource::Yajnavalkya`].
    pub rik: &'static str,
    /// Its japa count in thousands (BPHS 84.19–20). The verse writes
    /// each count as a word-numeral (*bhūta-saṅkhyā*), and digits are
    /// read from right to left: *tri-pakṣāḥ* is 3 then 2, so 23.
    /// Yājñavalkya gives no counts.
    pub japa_thousands: u8,
    /// The wood its fire is fed with (BPHS 84.21).
    pub samidh: Samidh,
    /// The food given (BPHS 84.23).
    pub food: Food,
    /// The fee given (BPHS 84.25).
    pub dakshina: Dakshina,
    /// The gem it owns (Jataka Parijata II.21).
    pub gem: Gem,
    /// The substance it rules (Brihat Jataka II.12), `None` for the
    /// nodes.
    pub substance: Option<Substance>,
    /// Its direction (Brihat Jataka II.5), `None` for Ketu, which the
    /// verse leaves out (C345).
    pub direction: Option<Direction>,
    /// Its place in the Matsya maṇḍala.
    pub mandala: MandalaPlace,
}

/// The number of offerings BPHS 84.22 and Yājñavalkya I.303 give each
/// graha, in the order they give them: 108, or else 28.
pub const OFFERINGS: [u8; 2] = [108, 28];

/// One graha's row as the verses give it, Rahu's ṛk under BPHS.
#[allow(
    clippy::too_many_arguments,
    reason = "one argument per column of the verses' table"
)]
const fn row(
    graha: Graha,
    image: ImageMaterial,
    rik: &'static str,
    japa_thousands: u8,
    (samidh, food, dakshina): (Samidh, Food, Dakshina),
    gem: Gem,
    substance: Option<Substance>,
    direction: Option<Direction>,
    mandala: MandalaPlace,
) -> Shanti {
    Shanti {
        graha,
        image,
        rik,
        japa_thousands,
        samidh,
        food,
        dakshina,
        gem,
        substance,
        direction,
        mandala,
    }
}

/// Rahu's ṛk in Yājñavalkya I.301, the dūrvā verse (VS XIII.20).
const RAHU_RIK_YAJNAVALKYA: &str = "kāṇḍāt";

/// The nine rows in graha order, from the Sun.
const SHANTI: [Shanti; 9] = {
    use Direction as D;
    use MandalaPlace as M;
    [
        row(
            Graha::Sun,
            ImageMaterial::Copper,
            "ā kṛṣṇena",
            7,
            (Samidh::Arka, Food::Gudaudana, Dakshina::MilchCow),
            Gem::Ruby,
            Some(Substance::Copper),
            Some(D::East),
            M::Centre,
        ),
        row(
            Graha::Moon,
            ImageMaterial::Crystal,
            "imaṃ devā",
            11,
            (Samidh::Palasha, Food::Payasa, Dakshina::Conch),
            Gem::Pearl,
            Some(Substance::Gems),
            Some(D::Northwest),
            M::Southeast,
        ),
        row(
            Graha::Mars,
            ImageMaterial::RedSandalwood,
            "agnir mūrdhā divaḥ kakut",
            10,
            (Samidh::Khadira, Food::Havishya, Dakshina::Bull),
            Gem::Coral,
            Some(Substance::Gold),
            Some(D::South),
            M::South,
        ),
        row(
            Graha::Mercury,
            ImageMaterial::Gold,
            "udbudhyasva",
            9,
            (Samidh::Apamarga, Food::KshiraShashtika, Dakshina::Gold),
            Gem::Emerald,
            Some(Substance::Alloy),
            Some(D::North),
            M::Northeast,
        ),
        row(
            Graha::Jupiter,
            ImageMaterial::Gold,
            "bṛhaspate",
            19,
            (Samidh::Pippala, Food::Dadhyodana, Dakshina::Cloth),
            Gem::Topaz,
            Some(Substance::Silver),
            Some(D::Northeast),
            M::North,
        ),
        row(
            Graha::Venus,
            ImageMaterial::Silver,
            "annāt parisrutaḥ",
            16,
            (Samidh::Udumbara, Food::Havis, Dakshina::Horse),
            Gem::Diamond,
            Some(Substance::Pearls),
            Some(D::Southeast),
            M::East,
        ),
        row(
            Graha::Saturn,
            ImageMaterial::Iron,
            "śaṃ no devīr abhiṣṭaye",
            23,
            (Samidh::Shami, Food::Churna, Dakshina::BlackCow),
            Gem::Sapphire,
            Some(Substance::Iron),
            Some(D::West),
            M::West,
        ),
        row(
            Graha::Rahu,
            ImageMaterial::Lead,
            "kayā naś citra",
            18,
            (Samidh::Durva, Food::Mamsa, Dakshina::Iron),
            Gem::Hessonite,
            None,
            Some(D::Southwest),
            M::Southwest,
        ),
        row(
            Graha::Ketu,
            ImageMaterial::Bronze,
            "ketuṃ kṛṇvann",
            17,
            (Samidh::Kusha, Food::Chitranna, Dakshina::Goat),
            Gem::CatsEye,
            None,
            None,
            M::Northwest,
        ),
    ]
};

/// What the texts prescribe for `graha`'s śānti, or `None` for Uranus,
/// Neptune and Pluto, which no text read names.
///
/// ```
/// use teistro_core::catalogue::Graha;
/// use teistro_remedies::{Dakshina, RikSource, ShantiRules, shanti};
///
/// // Saturn's japa is *tri-pakṣāḥ*, 23 thousand, read right to left.
/// let saturn = shanti(Graha::Saturn, ShantiRules::default()).unwrap();
/// assert_eq!(saturn.japa_thousands, 23);
/// assert_eq!(saturn.dakshina, Dakshina::BlackCow);
///
/// // Rahu's ṛk is the one text the two rank 1 sources part on.
/// let rules = ShantiRules { rik: RikSource::Yajnavalkya };
/// assert_eq!(shanti(Graha::Rahu, rules).unwrap().rik, "kāṇḍāt");
/// ```
#[must_use]
pub fn shanti(graha: Graha, rules: ShantiRules) -> Option<Shanti> {
    let read = SHANTI.iter().find(|row| row.graha == graha).copied()?;
    Some(match (graha, rules.rik) {
        (Graha::Rahu, RikSource::Yajnavalkya) => Shanti {
            rik: RAHU_RIK_YAJNAVALKYA,
            ..read
        },
        _ => read,
    })
}

//! What a festival is, beside when it falls (`03-design/festival-metadata.md`,
//! SR-02): its kind, tradition, communities and regions, as fields of its
//! rule, so a consumer filters a pack without a mapping of its own.
//!
//! No value here is invented. Each comes from a source the page names (a
//! government's holiday notice, the classification's community column, the
//! text the rule cites), and a field no source gives stays unset, which
//! [`FestivalMeta::unsourced`] says by name (C850).

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{FestivalKind, Tradition};

/// What a rule's festival is, as far as a source says.
///
/// ```
/// use teistro_panchanga::festival::FestivalMeta;
/// use teistro_core::catalogue::{FestivalKind, Tradition};
///
/// let gai_jatra = FestivalMeta::nepal("GAI_JATRA");
/// assert_eq!(gai_jatra.kind, Some(FestivalKind::Gazetted));
/// assert_eq!(gai_jatra.communities, ["NEWAR"]);
/// assert_eq!(gai_jatra.regions, ["NP", "KATHMANDU_VALLEY"]);
/// assert_eq!(gai_jatra.unsourced(), ["tradition"]);
///
/// // A rule of the consumer's own, with nothing sourced, says so of every field.
/// assert_eq!(FestivalMeta::default().unsourced().len(), 4);
/// assert_eq!(FestivalMeta::nepal("BUDDHA_JAYANTI").tradition, Some(Tradition::Buddhist));
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FestivalMeta {
    /// Its kind; unset when no source classes it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<FestivalKind>,
    /// Its religion; unset when no source places it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tradition: Option<Tradition>,
    /// The communities that keep it, as open keys (`NEWAR`); empty when no
    /// source names one, which does not say it is kept by all.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub communities: Vec<String>,
    /// Where it is kept, as open keys: an ISO 3166 country first (`NP`,
    /// `IN`), then a part of it a notice names (`KATHMANDU_VALLEY`); empty
    /// when no source says.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub regions: Vec<String>,
    /// Whether the notice leaves it to each office's list: one of the
    /// occasions three of which are chosen a year (the Government of
    /// India's para 3.1), so a year's list may not hold it.
    #[serde(default, skip_serializing_if = "core::ops::Not::not")]
    pub optional: bool,
}

/// One set of values [`NEPAL_KEYS`] rows share.
struct Group {
    kind: Option<FestivalKind>,
    tradition: Option<Tradition>,
    communities: &'static [&'static str],
    regions: &'static [&'static str],
}

impl Group {
    fn meta(&self) -> FestivalMeta {
        let owned = |keys: &[&str]| keys.iter().map(|&key| key.to_owned()).collect();
        FestivalMeta {
            kind: self.kind,
            tradition: self.tradition,
            communities: owned(self.communities),
            regions: owned(self.regions),
            optional: false,
        }
    }
}

impl FestivalMeta {
    /// Whether nothing is sourced.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self == &FestivalMeta::default()
    }

    /// The fields no source gives, by their wire names: what a rule says
    /// of itself rather than a blank a reader might take for an answer.
    #[must_use]
    pub fn unsourced(&self) -> Vec<&'static str> {
        let blank = [
            ("kind", self.kind.is_none()),
            ("tradition", self.tradition.is_none()),
            ("communities", self.communities.is_empty()),
            ("regions", self.regions.is_empty()),
        ];
        blank
            .into_iter()
            .filter_map(|(field, unset)| unset.then_some(field))
            .collect()
    }

    /// The values of a key `NEPAL` or `NEPAL_PANCHANGA` ships; empty for
    /// one they do not.
    #[must_use]
    pub fn nepal(key: &str) -> FestivalMeta {
        NEPAL_KEYS
            .binary_search_by_key(&key, |&(held, _)| held)
            .ok()
            .and_then(|at| NEPAL_KEYS.get(at))
            .and_then(|&(_, group)| NEPAL_GROUPS.get(usize::from(group)))
            .map_or_else(FestivalMeta::default, Group::meta)
    }

    /// The values of a rule from a Hindu text and from no country's list:
    /// *Dharmasindhu*'s packs and the Gaudiya pack.
    #[must_use]
    pub fn hindu_text() -> FestivalMeta {
        FestivalMeta {
            kind: Some(FestivalKind::ReligiousCultural),
            tradition: Some(Tradition::Hindu),
            ..FestivalMeta::default()
        }
    }
}

/// What a request keeps of its answer: a rule stands when, of each list
/// given, its meta holds a member. An empty list asks nothing of its
/// field, and a rule with that field unset fails a list that is not.
///
/// ```
/// use teistro_panchanga::festival::{FestivalMeta, MetaFilter};
/// use teistro_core::catalogue::FestivalKind;
///
/// let holidays = MetaFilter { kinds: vec![FestivalKind::Gazetted], ..MetaFilter::default() };
/// assert!(holidays.admits(&FestivalMeta::nepal("GAI_JATRA")));
/// assert!(!holidays.admits(&FestivalMeta::default()));
/// let newar = MetaFilter { communities: vec!["NEWAR".to_owned()], ..holidays };
/// assert!(newar.admits(&FestivalMeta::nepal("GAI_JATRA")));
/// assert!(!newar.admits(&FestivalMeta::nepal("SHIVARATRI")));
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MetaFilter {
    /// The kinds kept.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub kinds: Vec<FestivalKind>,
    /// The traditions kept.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub traditions: Vec<Tradition>,
    /// The communities kept.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub communities: Vec<String>,
    /// The regions kept.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub regions: Vec<String>,
}

impl MetaFilter {
    /// Whether a rule with this meta stands.
    #[must_use]
    pub fn admits(&self, meta: &FestivalMeta) -> bool {
        let one_of = |wanted: &[String], held: &[String]| {
            wanted.is_empty() || wanted.iter().any(|want| held.contains(want))
        };
        (self.kinds.is_empty() || meta.kind.is_some_and(|kind| self.kinds.contains(&kind)))
            && (self.traditions.is_empty()
                || meta
                    .tradition
                    .is_some_and(|tradition| self.traditions.contains(&tradition)))
            && one_of(&self.communities, &meta.communities)
            && one_of(&self.regions, &meta.regions)
    }
}

/// The groups of [`NEPAL_KEYS`], whose rows name them by index: each one set
/// of sourced values, in the order the table reads.
const NEPAL_GROUPS: [Group; 21] = [
    Group {
        kind: Some(FestivalKind::Gazetted),
        tradition: Some(Tradition::Buddhist),
        communities: &[],
        regions: &["NP"],
    },
    Group {
        kind: Some(FestivalKind::Gazetted),
        tradition: Some(Tradition::Hindu),
        communities: &[],
        regions: &["NP"],
    },
    Group {
        kind: Some(FestivalKind::Gazetted),
        tradition: None,
        communities: &[],
        regions: &["NP"],
    },
    Group {
        kind: Some(FestivalKind::Gazetted),
        tradition: None,
        communities: &[],
        regions: &["NP", "HILLS"],
    },
    Group {
        kind: Some(FestivalKind::Gazetted),
        tradition: None,
        communities: &[],
        regions: &["NP", "TERAI"],
    },
    Group {
        kind: Some(FestivalKind::Gazetted),
        tradition: None,
        communities: &["GURUNG"],
        regions: &["NP"],
    },
    Group {
        kind: Some(FestivalKind::Gazetted),
        tradition: None,
        communities: &["KIRAT"],
        regions: &["NP"],
    },
    Group {
        kind: Some(FestivalKind::Gazetted),
        tradition: None,
        communities: &["MAITHIL", "BHOJPURI"],
        regions: &["NP"],
    },
    Group {
        kind: Some(FestivalKind::Gazetted),
        tradition: None,
        communities: &["NEWAR"],
        regions: &["NP"],
    },
    Group {
        kind: Some(FestivalKind::Gazetted),
        tradition: None,
        communities: &["NEWAR"],
        regions: &["NP", "KATHMANDU_VALLEY"],
    },
    Group {
        kind: Some(FestivalKind::Gazetted),
        tradition: None,
        communities: &["SHERPA"],
        regions: &["NP"],
    },
    Group {
        kind: Some(FestivalKind::Gazetted),
        tradition: None,
        communities: &["TAMANG"],
        regions: &["NP"],
    },
    Group {
        kind: Some(FestivalKind::Gazetted),
        tradition: None,
        communities: &["THARU"],
        regions: &["NP"],
    },
    Group {
        kind: Some(FestivalKind::Gazetted),
        tradition: None,
        communities: &["THARU", "MAITHIL"],
        regions: &["NP"],
    },
    Group {
        kind: Some(FestivalKind::Gazetted),
        tradition: Some(Tradition::Sikh),
        communities: &[],
        regions: &["NP"],
    },
    Group {
        kind: None,
        tradition: None,
        communities: &[],
        regions: &["NP"],
    },
    Group {
        kind: Some(FestivalKind::Observance),
        tradition: None,
        communities: &[],
        regions: &["NP"],
    },
    Group {
        kind: Some(FestivalKind::RegionalEthnic),
        tradition: None,
        communities: &["NEWAR"],
        regions: &["NP"],
    },
    Group {
        kind: Some(FestivalKind::ReligiousCultural),
        tradition: Some(Tradition::Buddhist),
        communities: &[],
        regions: &["NP"],
    },
    Group {
        kind: Some(FestivalKind::ReligiousCultural),
        tradition: Some(Tradition::Hindu),
        communities: &[],
        regions: &["NP"],
    },
    Group {
        kind: Some(FestivalKind::ReligiousCultural),
        tradition: Some(Tradition::Jain),
        communities: &[],
        regions: &["NP"],
    },
];

/// Every key `NEPAL` and `NEPAL_PANCHANGA` ship, sorted for the binary
/// search, with its group in [`NEPAL_GROUPS`].
const NEPAL_KEYS: [(&str, u8); 359] = [
    ("ACHALA_SAPTAMI", 15),
    ("ADUKHA_NAVAMI", 17),
    ("AJA_EKADASHI", 15),
    ("AKHILBALI_PURTI", 15),
    ("AKSHAYA_TRITIYA", 15),
    ("AMALAKI_EKADASHI", 15),
    ("AMAVASYA_SHRADDHA", 19),
    ("AMUKTABHARANA_SAPTAMI", 15),
    ("ANALA_PUNHI", 15),
    ("ANANTA_CHATURDASHI", 15),
    ("ANNAPURNA_JATRA", 15),
    ("APARA_EKADASHI", 15),
    ("ARANIKO_MEMORIAL_DAY", 16),
    ("ARGHA_SARAI_PARVA", 15),
    ("ARJUNDHARA_MELA", 15),
    ("ARYAGHAT_MADHAV_NARAYAN_MELA", 15),
    ("ASHTAMI_SHRADDHA", 19),
    ("AVIDHAVA_NAVAMI", 19),
    ("BAHIDYO_BOYEGU", 17),
    ("BAISAKH_SNAN_ENDS", 15),
    ("BAKHUMADA", 15),
    ("BALAJU_BAISDHARA_MELA", 15),
    ("BALAMBU_MAHALAXMI_JATRA", 17),
    ("BALA_CHATURDASHI", 17),
    ("BALA_CHATURDASHI_BALA", 15),
    ("BALI_PRATIPADA", 19),
    ("BASANTA_PANCHAMI", 2),
    ("BHAKTAPUR_BRAHMAYANI_JATRA", 17),
    ("BHAKTAPUR_CHANDI_BHAGAWATI_JATRA", 17),
    ("BHAKTAPUR_CHHUMA_GANESH_JATRA", 17),
    ("BHAKTAPUR_MAHAKALI_MAHALAXMI_JATRA", 17),
    ("BHAKTAPUR_SIDDHA_POKHARI_MELA", 15),
    ("BHAL_BHAL_ASHTAMI", 17),
    ("BHANU_JAYANTI", 16),
    ("BHARANI_SHRADDHA", 19),
    ("BHAUMASHTAMI", 15),
    ("BHIMA_EKADASHI", 15),
    ("BHISHMASHTAMI", 15),
    ("BHISHMA_DWADASHI", 15),
    ("BHISHMA_PANCHAKA_ARAMBHA", 15),
    ("BHUMIRAJ", 15),
    ("BHUMI_PUJA", 15),
    ("BILVA_NIMANTRAN", 15),
    ("BISHALNAGAR_VAISHNAVI_DEVI_JATRA", 17),
    ("BISKET_JATRA_BEGINS", 17),
    ("BISKET_JATRA_ENDS", 17),
    ("BUDDHAVATAR", 15),
    ("BUDDHA_JAYANTI", 0),
    ("BUDHASHTAMI", 15),
    (
        "CASTE_BASED_DISCRIMINATION_AND_UNTOUCHABILITY_ELIMINATION_DAY",
        16,
    ),
    ("CHAALA", 15),
    ("CHAITE_DASHAIN", 15),
    ("CHAMPA_SHASHTHI", 15),
    ("CHANDI_PURNIMA", 2),
    ("CHANDRA_SHASHTHI", 15),
    ("CHANDRODAYA", 15),
    ("CHANGUNARAYAN_AKHANDA_DEEP_DARSHAN", 15),
    ("CHANGUNARAYAN_CHHINNAMASTA_KILESHWOR_MULRATH_JATRA", 17),
    ("CHANGUNARAYAN_KALASH_JATRA", 17),
    ("CHANGU_MADHAV_NARAYAN_MELA", 15),
    ("CHATHA", 17),
    ("CHATURDASHI_SHRADDHA", 19),
    ("CHATURMAS_BEGINS", 15),
    ("CHATURMAS_ENDS", 15),
    ("CHATURTHI_SHRADDHA", 19),
    ("CHHANDA_DIWAS", 16),
    ("CHHATH", 7),
    ("CHIR_SWAYEGU", 17),
    ("CHONAM_PARVA", 15),
    ("CHRISTMAS", 2),
    ("CIVIL_SERVICE_DAY", 16),
    ("CONSTITUTION_DAY", 2),
    ("DAKSHINAMURTI_PUJA", 15),
    ("DAR_KHANE_DIN", 15),
    ("DASHAHARA_SNANA_ARAMBHA", 19),
    ("DASHAMI_SHRADDHA", 19),
    ("DATTATREYA_JAYANTI", 15),
    ("DEMOCRACY_DAY", 2),
    ("DEOPATAN_GANGAMAI_RATH_JATRA", 17),
    ("DEOPATAN_TRISHUL_JATRA", 17),
    ("DEVGHAT_MAKAR_SNAN", 15),
    ("DEVI_VISARJAN", 19),
    ("DHANTERAS", 15),
    ("DHANU_SANKRANTI", 15),
    ("DHANVANTARI_JAYANTI", 15),
    ("DHANYA_PURNIMA", 2),
    ("DHAPAKHEL_NAGDAHA", 15),
    ("DILA_CHARHE", 17),
    ("DILLA_PUNHI", 17),
    ("DRONA_NAVAMI", 15),
    ("DURVA_ASHTAMI", 15),
    ("DVAPARA_YUGADI", 15),
    ("DWADASHI_SHRADDHA", 19),
    ("DWITIYA_SHRADDHA", 19),
    ("EARTHQUAKE_SAFETY_DAY", 16),
    ("EKADASHI_GAUDIYA", 19),
    ("EKADASHI_SHRADDHA", 19),
    ("EKADASHI_SMARTA", 19),
    ("EKADASHI_SMARTA_RENUNCIANT", 19),
    ("EKADASHI_VAISHNAVA", 19),
    ("ELECTION_DAY", 16),
    ("FAGU_PURNIMA", 3),
    ("FALGUNANDA_JAYANTI", 16),
    ("FULKHARKA_GANGAJAMUNA_MELA", 15),
    ("GAIDU_PUJA", 15),
    ("GAI_JATRA", 9),
    ("GANESHA_CHATURTHI", 15),
    ("GANGA_DASHAHARA", 19),
    ("GANGA_PUJA", 15),
    ("GANGA_SAPTAMI", 15),
    ("GATILA", 15),
    ("GAURA_SAPTAMI", 15),
    ("GAURI_VRAT", 15),
    ("GAYA_MAUNI_AUNSI", 15),
    ("GHODE_JATRA", 9),
    ("GITA_JAYANTI", 15),
    ("GOPASHTAMI", 15),
    ("GORAKHNATH_JAYANTI", 15),
    ("GOSAIKUNDA_SNANA_ARAMBHA", 15),
    ("GOSAIKUNDA_SNANA_SAMAPTI", 15),
    ("GOVARDHAN_PUJA", 2),
    ("GOVATSA_DWADASHI", 19),
    ("GOVINDA_DWADASHI", 15),
    ("GUGA_NAVAMI", 15),
    ("GUHYESHWARI_JATRA", 17),
    ("GUNLA_BEGINS", 17),
    ("GUNLA_ENDS", 17),
    ("GURU_NANAK_JAYANTI", 14),
    ("GURU_PADMASAMBHAVA_JAYANTI", 18),
    ("GURU_PURNIMA", 15),
    ("GYALPO_LHOSAR", 10),
    ("HALI_TIHAR", 15),
    ("HANUMAN_JAYANTI", 15),
    ("HARIBODHINI_EKADASHI", 15),
    ("HARIPARIVARTINI_EKADASHI", 15),
    ("HARISHAYANI_EKADASHI", 15),
    ("HARITALIKA", 1),
    ("HATYAMOCHANA_SNANA", 15),
    ("HAYAGRIVA_JAYANTI", 15),
    ("HETAUDA_KUSHMANDA_SAROBAR_MELA", 15),
    ("HILE_JATRA", 15),
    ("HOLIKA", 19),
    ("HOLI_HILLS", 3),
    ("HOLI_TERAI", 4),
    ("HUMAN_RIGHTS_DAY", 16),
    ("INDIRA_EKADASHI", 15),
    ("INDRADAHA_SNAN", 17),
    ("INDRADHWAJA_UTTHAN", 17),
    ("INDRA_JATRA", 9),
    ("INTERNATIONAL_DAY_OF_PEACE", 16),
    ("INTERNATIONAL_DAY_OF_PERSONS_WITH_DISABILITIES", 2),
    ("INTERNATIONAL_DAY_OF_THE_WORLD_S_INDIGENOUS_PEOPLES", 16),
    ("INTERNATIONAL_LABOUR_DAY", 2),
    ("INTERNATIONAL_LITERACY_DAY", 16),
    ("INTERNATIONAL_SPORTS_JOURNALISTS_DAY", 16),
    ("INTERNATIONAL_WOMEN_S_DAY", 2),
    ("JAGANNATH_RATH_JATRA", 15),
    ("JANMASHTAMI", 1),
    ("JITIYA", 13),
    ("JYA_PUNHI", 17),
    ("KABIR_JAYANTI", 15),
    ("KAG_TIHAR", 2),
    ("KALI_YUGADI", 15),
    ("KALKI_JAYANTI", 15),
    ("KAMADA_EKADASHI", 15),
    ("KAMIKA_EKADASHI", 15),
    ("KANDAKARAK_PUJA", 15),
    ("KANYA_SANKRANTI", 15),
    ("KARAKA_CHATURTHI", 19),
    ("KARKATA_SANKRANTI", 15),
    ("KARTIK_SNAN_BEGINS", 15),
    ("KARTIK_SNAN_ENDS", 15),
    ("KASHI_BADA_GANESH_UTSAV", 19),
    ("KATIN_PUNHI", 17),
    ("KAYA_ASHTAMI", 17),
    ("KHADGA_JATRA", 17),
    ("KHANCHI_SARAI_PARVA", 15),
    ("KHEER_KHANE_DIN", 15),
    ("KIRAT_SAMAJ_SUDHAR_DIWAS", 16),
    ("KIRTIPUR_BAGH_BHAIRAV_JATRA", 17),
    ("KIRTIPUR_INDRAYANI_JATRA", 17),
    ("KOJAGRAT_PURNIMA", 19),
    ("KOKA_KAUSHIKI_SANGAM_MELA", 15),
    ("KOKHAJA_BIYE", 15),
    ("KUCHHI_BHOYE", 17),
    ("KUKUR_TIHAR", 2),
    ("KUMARA_JATRA", 17),
    ("KUMAR_SHASHTHI", 15),
    ("KUMBHA_SANKRANTI", 15),
    ("KURMA_JAYANTI", 15),
    ("KUSHE_AUNSI", 15),
    ("KUSHMANDA_NAVAMI", 15),
    ("KWATI_PUNHI", 17),
    ("LAI_CHARHE", 17),
    ("LAKSHMI_PUJA", 1),
    ("LALITPUR_NARSINGH_JATRA", 17),
    ("LAW_DAY", 16),
    ("LAXMI_PRASAD_DEVKOTA_JAYANTI", 15),
    ("LEKHNATH_PAUDYAL_JAYANTI", 16),
    ("LOKTANTRA_DIWAS", 16),
    ("MAGHA_SHRADDHA", 19),
    ("MAGHI_THARU", 12),
    ("MAGH_SNAN_BEGINS", 15),
    ("MAGH_SNAN_ENDS", 15),
    ("MAHAVIR_JAYANTI", 20),
    ("MAHA_ASHTAMI", 2),
    ("MAHA_NAVAMI", 1),
    ("MAKARA_SANKRANTI", 2),
    ("MAKAR_SNAN", 15),
    ("MANGALA_CHATURTHI", 19),
    ("MARTYRS_DAY", 2),
    ("MASTA_PURNIMA", 15),
    ("MATA_TIRTHA_AUNSI", 15),
    ("MATSYANARAYANA_MELA", 15),
    ("MATSYA_JAYANTI", 15),
    ("MESHA_SANKRANTI", 2),
    ("MHA_PUJA", 8),
    ("MINA_SANKRANTI", 15),
    ("MITHUNA_SANKRANTI", 15),
    ("MOHINI_EKADASHI", 15),
    ("MOKSHADA_EKADASHI", 15),
    ("MOTIRAM_JAYANTI", 15),
    ("MUKHA_ASHTAMI", 17),
    ("MUSYADULI", 15),
    ("NAG_PANCHAMI", 15),
    ("NAKWA_DISHI", 17),
    ("NALA_MACHHINDRANATH_RATH_JATRA", 17),
    ("NARAK_CHATURDASHI", 15),
    ("NARASIMHA_JAYANTI", 19),
    ("NATIONAL_CHILDREN_S_DAY", 16),
    ("NATIONAL_DAY_AGAINST_HUMAN_TRAFFICKING", 16),
    ("NATIONAL_PADDY_DAY", 16),
    ("NAVAMI_SHRADDHA", 19),
    ("NAVARATRA_ARAMBHA", 1),
    ("NEPAL_SAMBAT_NEW_YEAR", 8),
    ("NEW_YEAR_S_DAY_AD", 16),
    ("NIRJALA_EKADASHI", 15),
    ("NUWAKOT_DUPCHESHWAR_MELA", 15),
    ("PACHALI_BHAIRAV_JATRA", 17),
    ("PADMINI_EKADASHI", 15),
    ("PAHAN_CHARHE", 17),
    ("PANAUTI_JATRA", 17),
    ("PANAUTI_SNAN_MELA", 15),
    ("PANCHAMI_SHRADDHA", 19),
    ("PANGA_VAISHNAVI_DEVI_JATRA", 17),
    ("PAPANKUSHA_EKADASHI", 15),
    ("PAPMOCHANI_EKADASHI", 15),
    ("PARAMA_EKADASHI", 15),
    ("PARASHURAMA_JAYANTI", 15),
    ("PASHUPATINATH_DAMAN_PUSHPARPAN", 15),
    ("PASHUPATI_HALESI_MELA", 15),
    ("PAUSHA_PUTRADA_EKADASHI", 15),
    ("PHULPATI", 2),
    ("PISHACHAMOCHANA_SHRADDHA", 19),
    ("PRADOSHA_VRATA", 15),
    ("PRATIPADA_SHRADDHA", 19),
    ("PRITHVI_JAYANTI", 2),
    ("PUNCHALI_BHUJA", 15),
    ("PURNIMA_VRATA", 19),
    ("RADHASHTAMI", 15),
    ("RAKSHABANDHAN", 1),
    ("RAMA_EKADASHI", 15),
    ("RAMA_NAVAMI", 1),
    ("RAMESHWAR_PUJA", 15),
    ("RATO_MACHHINDRANATH_RATHAROHAN", 17),
    ("RATO_MACHHINDRANATH_RATH_JATRA_BEGINS", 17),
    ("RAVI_SAPTAMI", 19),
    ("REPUBLIC_DAY", 2),
    ("RISHI_PANCHAMI", 15),
    ("ROPAI_JATRA", 15),
    ("SAFALA_EKADASHI", 15),
    ("SAKWO_CHANGU_WONE", 17),
    ("SAMVATSARA_ARAMBHA", 15),
    ("SANKASHTA_CHATURTHI", 19),
    ("SANSKRIT_DAY", 15),
    ("SANYASI_MAHALAYA", 19),
    ("SAPTAMI_SHRADDHA", 19),
    ("SATYAYUGADI", 15),
    ("SETO_MACHHINDRANATH_JATRA", 17),
    ("SETO_MACHHINDRANATH_SNAN", 17),
    ("SHADANANDA_JAYANTI", 15),
    ("SHALYA_DASHAMI", 15),
    ("SHANI_JAYANTI", 15),
    ("SHANKARACHARYA_JAYANTI", 15),
    ("SHASHTHI_SHRADDHA", 19),
    ("SHATTILA_EKADASHI", 15),
    ("SHILA_CHARHE", 17),
    ("SHITALASHTAMI", 15),
    ("SHIVARATRI", 1),
    ("SHIVA_BAIKUNTHA_CHATURDASHI", 15),
    ("SHIVA_PAVITRAROPAN", 15),
    ("SHRAWAN_PUTRADA_EKADASHI", 15),
    ("SHREEGHA_JATRA", 17),
    ("SHREE_BALLABH_JAYANTI", 15),
    ("SILANGA_MAHARUDRA_KOTIHOM_MELA", 15),
    ("SIMHA_SANKRANTI", 15),
    ("SITA_NAVAMI", 15),
    ("SITHI_NAKHA", 17),
    ("SI_PUNHI", 17),
    ("SKANDA_SHASHTHI", 15),
    ("SOMAVATI_AMAVASYA", 19),
    ("SONAM_LHOSAR", 11),
    ("SORHA_SHRADDHA_ARAMBHA", 15),
    ("SURYA_PUJA", 15),
    ("SURYA_SHASHTHI", 15),
    ("SWASTHANI_VRAT_BEGINS", 15),
    ("SWASTHANI_VRAT_ENDS", 15),
    ("SWONTI_CHARHE", 17),
    ("SYAKWA_TYAKWA", 17),
    ("TAMU_LHOSAR", 5),
    ("THANKOT_MAHALAXMI_JATRA", 17),
    ("THIMI_BALKUMARI_JATRA", 17),
    ("TILAKUNDA_CHATURTHI", 15),
    ("TOKHA_CHANDESHWARI_JATRA", 17),
    ("TOKHA_SAPAN_TIRTHA_SNAN", 15),
    ("TOL_LHOSAR", 15),
    ("TRAYODASHI_SHRADDHA", 19),
    ("TRETAYUGADI", 15),
    ("TRITIYA_SHRADDHA", 19),
    ("TULASI_VIVAHA", 15),
    ("TULA_SANKRANTI", 15),
    ("TULSIDAS_JAYANTI", 15),
    ("UBHAULI", 6),
    ("UDHAULI", 6),
    ("UNITED_NATIONS_DAY", 16),
    ("UPAKARMA_MADHYANDINA", 19),
    ("UPAKARMA_SAMAVEDI", 19),
    ("UPAKARMA_TAITTIRIYA", 19),
    ("UPANGA_LALITA_VRATA", 15),
    ("UTPATTIKA_EKADASHI", 15),
    ("VAISHAKHA_SNANA_ARAMBHA", 15),
    ("VAMANA_DWADASHI", 15),
    ("VARAHA_JAYANTI", 15),
    ("VARUTHINI_EKADASHI", 15),
    ("VASTU_DIWAS", 16),
    ("VAYU_ASHTAMI", 15),
    ("VIJAYA_DASHAMI", 1),
    ("VIJAYA_EKADASHI", 15),
    ("VISHWAKARMA_PUJA", 15),
    ("VIVAH_PANCHAMI", 15),
    ("VRISHABHA_SANKRANTI", 15),
    ("VRISHCHIKA_SANKRANTI", 15),
    ("WALKE_PARVA", 15),
    ("WATU_MAGH_JATRA", 17),
    ("WORLD_DIABETES_DAY", 16),
    ("WORLD_ENVIRONMENT_DAY", 16),
    ("WORLD_NO_TOBACCO_DAY", 16),
    ("WORLD_POPULATION_DAY", 16),
    ("WORLD_PRESS_FREEDOM_DAY", 16),
    ("WORLD_RED_CROSS_DAY", 16),
    ("WORLD_TOURISM_DAY", 16),
    ("WORLD_TUBERCULOSIS_DAY", 16),
    ("WORLD_WATER_DAY", 16),
    ("YALA_PANCHADAN", 17),
    ("YAMAPANCHAKA_ARAMBHA", 15),
    ("YAMA_DWITIYA", 1),
    ("YOGINI_EKADASHI", 15),
    ("YOMARI_PUNHI", 8),
    ("YOSIN_PATAN", 17),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_table_is_sorted_for_its_search_and_names_real_groups() {
        assert!(
            NEPAL_KEYS
                .windows(2)
                .all(|pair| matches!(pair, [a, b] if a.0 < b.0))
        );
        assert!(
            NEPAL_KEYS
                .iter()
                .all(|row| usize::from(row.1) < NEPAL_GROUPS.len())
        );
    }

    #[test]
    fn a_key_the_table_lacks_is_unsourced_in_every_field() {
        assert!(FestivalMeta::nepal("NO_SUCH_RULE").is_empty());
    }

    #[test]
    fn a_filter_reads_each_field_and_an_unset_field_fails_it() {
        let meta = FestivalMeta::nepal("BUDDHA_JAYANTI");
        let by_tradition = |tradition| MetaFilter {
            traditions: vec![tradition],
            ..MetaFilter::default()
        };
        assert!(by_tradition(Tradition::Buddhist).admits(&meta));
        assert!(!by_tradition(Tradition::Jain).admits(&meta));
        assert!(!by_tradition(Tradition::Jain).admits(&FestivalMeta::default()));
        let in_india = MetaFilter {
            regions: vec!["IN".to_owned()],
            ..MetaFilter::default()
        };
        assert!(!in_india.admits(&meta));
        assert!(MetaFilter::default().admits(&FestivalMeta::default()));
        assert!(
            !MetaFilter {
                kinds: vec![FestivalKind::Observance],
                ..MetaFilter::default()
            }
            .admits(&meta)
        );
    }
}

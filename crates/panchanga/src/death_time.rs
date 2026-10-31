//! The death-time check (SR-26, `03-design/death-time.md`): the
//! *dhanishtha panchaka*, the *tripada* and *dvipada* nakshatras and the
//! *pushkara* yogas at a death and a cremation, and the rites the texts
//! tie to them (*Dharmasindhu* pp. 316, 334–335; *Nirṇayasindhu* pp.
//! 434–435; *Muhūrta-cintāmaṇi* II.48–49 with its commentary, pp. 84–87).

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Kind, Nakshatra, Rashi, Tithi, Vara, write_in_full};
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Utc};

/// Whether a nakshatra straddles a sign so that three or two of its feet
/// fall in one.
///
/// ```
/// use teistro_panchanga::death_time::NakshatraFeet;
///
/// assert_eq!(serde_json::to_string(&NakshatraFeet::Tripada)?, r#""TRIPADA""#);
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NakshatraFeet {
    /// A sign ends after the first foot (*viṣamāṅghri*).
    Tripada,
    /// A sign ends after the second foot (*yamalāṅghri*).
    Dvipada,
}

/// The yoga of a tithi, a vara and a nakshatra's feet.
///
/// ```
/// use teistro_panchanga::death_time::Pushkara;
///
/// assert_ne!(Pushkara::Tripushkara, Pushkara::Dvipushkara);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Pushkara {
    /// With a tripada nakshatra: what happens is said to come threefold.
    Tripushkara,
    /// With a dvipada nakshatra, or by the table's other reading: twofold.
    Dvipushkara,
}

/// A rite the texts tie to a reading.
///
/// ```
/// use teistro_panchanga::death_time::DeathRiteKind;
///
/// assert_ne!(DeathRiteKind::PanchakaFigures, DeathRiteKind::ClosingShanti);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DeathRiteKind {
    /// The five figures burnt with the body.
    PanchakaFigures,
    /// The shanti at the period's end.
    ClosingShanti,
    /// Three penances and three figures burnt with the body.
    PushkaraFigures,
}

/// One rite owed, with its clause.
///
/// ```
/// use teistro_panchanga::death_time::{DeathRite, DeathRiteKind};
///
/// let rite = DeathRite { rite: DeathRiteKind::ClosingShanti, clause: String::from("Dharmasindhu p. 335") };
/// assert_eq!(rite.rite, DeathRiteKind::ClosingShanti);
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeathRite {
    /// The rite.
    pub rite: DeathRiteKind,
    /// The text and page.
    pub clause: String,
}

/// A tradition's lists, every one data.
///
/// ```
/// use teistro_core::catalogue::{Nakshatra, Vara};
/// use teistro_panchanga::death_time::DeathTimeTable;
///
/// let table = DeathTimeTable::dharmasindhu();
/// assert!(table.tripada.contains(&Nakshatra::Vishakha));
/// assert!(table.pushkara_varas.contains(&Vara::Shanivara));
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeathTimeTable {
    /// The table's name, repeated in each answer.
    pub name: String,
    /// The Moon's signs of the panchaka: Aquarius and Pisces, which is
    /// the second half of Dhanishtha to the end of Revati.
    pub panchaka_signs: Vec<Rashi>,
    /// The tripada nakshatras.
    pub tripada: Vec<Nakshatra>,
    /// The dvipada nakshatras.
    pub dvipada: Vec<Nakshatra>,
    /// The tithis of the pushkara yogas.
    pub pushkara_tithis: Vec<Tithi>,
    /// The varas of the pushkara yogas.
    pub pushkara_varas: Vec<Vara>,
    /// Whether the tithi and vara alone make a dvipushkara, the other
    /// reading of *Dharmasindhu* p. 316 and Garga in *Nirṇayasindhu* p.
    /// 434 (C953).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub two_make_dvipushkara: bool,
}

impl DeathTimeTable {
    /// The name [`DeathTimeTable::dharmasindhu`] answers under.
    ///
    /// ```
    /// use teistro_panchanga::death_time::DeathTimeTable;
    ///
    /// assert_eq!(DeathTimeTable::dharmasindhu().name, DeathTimeTable::DHARMASINDHU);
    /// ```
    pub const DHARMASINDHU: &'static str = "DHARMASINDHU";

    /// *Dharmasindhu* pp. 334–335, whose lists *Muhūrta-cintāmaṇi*
    /// II.48–49 repeats: the bhadrā tithis (2, 7, 12) on a Sunday,
    /// Tuesday or Saturday.
    ///
    /// ```
    /// use teistro_panchanga::death_time::DeathTimeTable;
    ///
    /// let table = DeathTimeTable::dharmasindhu();
    /// assert_eq!((table.tripada.len(), table.dvipada.len()), (6, 3));
    /// ```
    #[must_use]
    pub fn dharmasindhu() -> DeathTimeTable {
        use Nakshatra as N;
        DeathTimeTable {
            name: DeathTimeTable::DHARMASINDHU.to_owned(),
            panchaka_signs: vec![Rashi::Aquarius, Rashi::Pisces],
            tripada: vec![
                N::Punarvasu,
                N::UttaraAshadha,
                N::Krittika,
                N::UttaraPhalguni,
                N::PurvaBhadrapada,
                N::Vishakha,
            ],
            dvipada: vec![N::Mrigashira, N::Chitra, N::Dhanishtha],
            pushkara_tithis: vec![
                Tithi::ShuklaDvitiya,
                Tithi::ShuklaSaptami,
                Tithi::ShuklaDwadashi,
                Tithi::KrishnaDvitiya,
                Tithi::KrishnaSaptami,
                Tithi::KrishnaDwadashi,
            ],
            pushkara_varas: vec![Vara::Ravivara, Vara::Mangalavara, Vara::Shanivara],
            two_make_dvipushkara: false,
        }
    }

    /// The feet of `nakshatra`.
    ///
    /// ```
    /// use teistro_core::catalogue::Nakshatra;
    /// use teistro_panchanga::death_time::{DeathTimeTable, NakshatraFeet};
    ///
    /// let table = DeathTimeTable::dharmasindhu();
    /// assert_eq!(table.feet(Nakshatra::Chitra), Some(NakshatraFeet::Dvipada));
    /// assert_eq!(table.feet(Nakshatra::Ashwini), None);
    /// ```
    #[must_use]
    pub fn feet(&self, nakshatra: Nakshatra) -> Option<NakshatraFeet> {
        if self.tripada.contains(&nakshatra) {
            Some(NakshatraFeet::Tripada)
        } else if self.dvipada.contains(&nakshatra) {
            Some(NakshatraFeet::Dvipada)
        } else {
            None
        }
    }

    /// The pushkara yoga of a tithi, vara and nakshatra.
    ///
    /// ```
    /// use teistro_core::catalogue::{Nakshatra, Tithi, Vara};
    /// use teistro_panchanga::death_time::{DeathTimeTable, Pushkara};
    ///
    /// let table = DeathTimeTable::dharmasindhu();
    /// let found = table.pushkara(Tithi::ShuklaSaptami, Vara::Shanivara, Nakshatra::Krittika);
    /// assert_eq!(found, Some(Pushkara::Tripushkara));
    /// assert_eq!(table.pushkara(Tithi::ShuklaSaptami, Vara::Budhavara, Nakshatra::Krittika), None);
    /// ```
    #[must_use]
    pub fn pushkara(&self, tithi: Tithi, vara: Vara, nakshatra: Nakshatra) -> Option<Pushkara> {
        if !self.pushkara_tithis.contains(&tithi) || !self.pushkara_varas.contains(&vara) {
            return None;
        }
        match self.feet(nakshatra) {
            Some(NakshatraFeet::Tripada) => Some(Pushkara::Tripushkara),
            Some(NakshatraFeet::Dvipada) => Some(Pushkara::Dvipushkara),
            None => self.two_make_dvipushkara.then_some(Pushkara::Dvipushkara),
        }
    }
}

/// What stood at one instant.
///
/// ```
/// use teistro_core::catalogue::{Nakshatra, Rashi, Tithi, Vara};
/// use teistro_core::quantity::JulianDay;
/// use teistro_panchanga::death_time::{DeathInstant, DeathTimeTable};
///
/// let table = DeathTimeTable::dharmasindhu();
/// let at = JulianDay::literal(2_461_051.5);
/// let read = DeathInstant::read(&table, at, Nakshatra::Shatabhisha, Tithi::ShuklaPanchami, Vara::Somavara, Rashi::Aquarius);
/// assert!(read.panchaka);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeathInstant {
    /// The instant.
    pub at: JulianDay<Utc>,
    /// The nakshatra at it.
    pub nakshatra: Nakshatra,
    /// The tithi at it.
    pub tithi: Tithi,
    /// The vara of the sunrise day holding it.
    pub vara: Vara,
    /// The Moon's sign at it.
    pub moon: Rashi,
    /// Whether it is in the panchaka.
    pub panchaka: bool,
    /// The nakshatra's feet.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub feet: Option<NakshatraFeet>,
    /// The pushkara yoga.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pushkara: Option<Pushkara>,
}

impl DeathInstant {
    /// The reading of one instant under `table`.
    ///
    /// ```
    /// use teistro_core::catalogue::{Nakshatra, Rashi, Tithi, Vara};
    /// use teistro_core::quantity::JulianDay;
    /// use teistro_panchanga::death_time::{DeathInstant, DeathTimeTable, NakshatraFeet};
    ///
    /// // Dhanishtha's first half: dvipada, and short of the panchaka.
    /// let table = DeathTimeTable::dharmasindhu();
    /// let at = JulianDay::literal(2_461_051.5);
    /// let read = DeathInstant::read(&table, at, Nakshatra::Dhanishtha, Tithi::ShuklaPanchami, Vara::Somavara, Rashi::Capricorn);
    /// assert_eq!((read.panchaka, read.feet), (false, Some(NakshatraFeet::Dvipada)));
    /// ```
    #[must_use]
    pub fn read(
        table: &DeathTimeTable,
        at: JulianDay<Utc>,
        nakshatra: Nakshatra,
        tithi: Tithi,
        vara: Vara,
        moon: Rashi,
    ) -> DeathInstant {
        DeathInstant {
            at,
            nakshatra,
            tithi,
            vara,
            moon,
            panchaka: table.panchaka_signs.contains(&moon),
            feet: table.feet(nakshatra),
            pushkara: table.pushkara(tithi, vara, nakshatra),
        }
    }
}

/// The answer.
///
/// ```
/// use teistro_core::catalogue::{Nakshatra, Rashi, Tithi, Vara};
/// use teistro_core::quantity::JulianDay;
/// use teistro_panchanga::death_time::{DeathInstant, DeathTimeTable, owed};
///
/// let table = DeathTimeTable::dharmasindhu();
/// let at = JulianDay::literal(2_461_051.5);
/// let death = DeathInstant::read(&table, at, Nakshatra::Revati, Tithi::ShuklaPanchami, Vara::Somavara, Rashi::Pisces);
/// assert_eq!(owed(&death, None).len(), 2);
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeathTime {
    /// The death's reading.
    pub death: DeathInstant,
    /// The cremation's reading, when its instant was given.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cremation: Option<DeathInstant>,
    /// The rites owed. With no cremation given, the cremation is taken to
    /// fall as the death did.
    pub rites: Vec<DeathRite>,
    /// The table's name.
    pub table: String,
}

impl DeathTime {
    /// Catalogue members a binding reads in full.
    ///
    /// ```
    /// use teistro_panchanga::death_time::DeathTime;
    ///
    /// assert_eq!(DeathTime::MEMBERS.len(), 8);
    /// ```
    pub const MEMBERS: [(&'static str, Kind); 8] = [
        ("death.nakshatra", Kind::Nakshatra),
        ("death.tithi", Kind::Tithi),
        ("death.vara", Kind::Vara),
        ("death.moon", Kind::Rashi),
        ("cremation.nakshatra", Kind::Nakshatra),
        ("cremation.tithi", Kind::Tithi),
        ("cremation.vara", Kind::Vara),
        ("cremation.moon", Kind::Rashi),
    ];

    /// As JSON, every catalogue member written as its full key.
    ///
    /// ```
    /// use teistro_core::catalogue::{Nakshatra, Rashi, Tithi, Vara};
    /// use teistro_core::quantity::JulianDay;
    /// use teistro_panchanga::death_time::{DeathInstant, DeathTime, DeathTimeTable};
    ///
    /// let table = DeathTimeTable::dharmasindhu();
    /// let death = DeathInstant::read(&table, JulianDay::literal(2_461_051.5), Nakshatra::Revati, Tithi::ShuklaPanchami, Vara::Somavara, Rashi::Pisces);
    /// let answer = DeathTime { death, cremation: None, rites: Vec::new(), table: table.name };
    /// assert_eq!(answer.in_full()?["death"]["moon"], "rashi.PISCES");
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INTERNAL` if the answer does not serialise.
    pub fn in_full(&self) -> Result<serde_json::Value, Error> {
        let mut value = serde_json::to_value(self).map_err(|error| {
            Error::internal(format!("the death time did not serialise: {error}"))
        })?;
        for (path, kind) in Self::MEMBERS {
            write_in_full(&mut value, path, kind);
        }
        Ok(value)
    }
}

/// The rites owed for a death and a cremation (*Dharmasindhu* p. 335).
///
/// ```
/// use teistro_core::catalogue::{Nakshatra, Rashi, Tithi, Vara};
/// use teistro_core::quantity::JulianDay;
/// use teistro_panchanga::death_time::{DeathInstant, DeathRiteKind, DeathTimeTable, owed};
///
/// let table = DeathTimeTable::dharmasindhu();
/// let at = JulianDay::literal(2_461_051.5);
/// let read = |n, r| DeathInstant::read(&table, at, n, Tithi::ShuklaPanchami, Vara::Somavara, r);
/// // A death in another nakshatra burnt in the panchaka: the figures, no shanti.
/// let rites = owed(&read(Nakshatra::Shravana, Rashi::Capricorn), Some(&read(Nakshatra::Revati, Rashi::Pisces)));
/// assert_eq!(rites.iter().map(|one| one.rite).collect::<Vec<_>>(), [DeathRiteKind::PanchakaFigures]);
/// // A panchaka death burnt in Ashwini: the shanti, no figures.
/// let rites = owed(&read(Nakshatra::Revati, Rashi::Pisces), Some(&read(Nakshatra::Ashwini, Rashi::Aries)));
/// assert_eq!(rites.iter().map(|one| one.rite).collect::<Vec<_>>(), [DeathRiteKind::ClosingShanti]);
/// ```
#[must_use]
pub fn owed(death: &DeathInstant, cremation: Option<&DeathInstant>) -> Vec<DeathRite> {
    let burnt = cremation.unwrap_or(death);
    let mut rites = Vec::new();
    let mut add = |rite, clause: &str| {
        rites.push(DeathRite {
            rite,
            clause: clause.to_owned(),
        });
    };
    if burnt.panchaka {
        add(
            DeathRiteKind::PanchakaFigures,
            if death.panchaka {
                "Dharmasindhu pp. 334–335; Nirṇayasindhu p. 434: a death in the panchaka, burnt in it, the five figures with the body"
            } else {
                "Dharmasindhu p. 335; Muhūrta-cintāmaṇi II.48 comm.: a death in another nakshatra, burnt in the panchaka, the figures and no shanti"
            },
        );
    }
    let pushkara = death.pushkara.is_some() || burnt.pushkara.is_some();
    if pushkara {
        add(
            DeathRiteKind::PushkaraFigures,
            "Dharmasindhu p. 335: at a death or burning in a tripushkara or dvipushkara, three penances, and three figures of barley flour with the body",
        );
    }
    if death.panchaka {
        add(
            DeathRiteKind::ClosingShanti,
            if burnt.panchaka {
                "Dharmasindhu p. 335; Nirṇayasindhu pp. 434–435: the shanti at the period's end"
            } else {
                "Dharmasindhu p. 335; Muhūrta-cintāmaṇi II.48 comm.: a panchaka death burnt past it, the shanti and no figures"
            },
        );
    } else if death.feet == Some(NakshatraFeet::Tripada) {
        add(
            DeathRiteKind::ClosingShanti,
            "Dharmasindhu p. 335; Nirṇayasindhu p. 435: in a tripada nakshatra, the same shanti",
        );
    } else if pushkara {
        add(
            DeathRiteKind::ClosingShanti,
            "Dharmasindhu p. 335: the shanti at the period's end, after a pushkara",
        );
    }
    rites
}

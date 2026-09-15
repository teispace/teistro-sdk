//! The closed value sets of the knobs. Each is a small enum with a key
//! form (`SCREAMING_SNAKE_CASE`) that is what the canonical document,
//! the C boundary and every binding use.

macro_rules! knob {
    ($(#[$m:meta])* $name:ident { $( $(#[$vm:meta])* $variant:ident = $key:literal ),+ $(,)? }) => {
        $(#[$m])*
        #[non_exhaustive]
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
        #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
        pub enum $name {
            $( $(#[$vm])* #[serde(rename = $key)] $variant ),+
        }

        impl $name {
            /// Every value, in declaration order.
            pub const ALL: &'static [$name] = &[ $( $name::$variant ),+ ];

            /// The key form.
            #[must_use]
            pub const fn key(self) -> &'static str {
                match self { $( $name::$variant => $key ),+ }
            }

            /// The value with a key.
            #[must_use]
            pub fn from_key(key: &str) -> Option<$name> {
                match key { $( $key => Some($name::$variant), )+ _ => None }
            }
        }

        impl core::fmt::Display for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str(self.key())
            }
        }

        impl core::str::FromStr for $name {
            type Err = crate::quantity::InvalidValue;

            fn from_str(key: &str) -> Result<$name, crate::quantity::InvalidValue> {
                $name::from_key(key).ok_or_else(|| crate::quantity::InvalidValue {
                    quantity: stringify!($name),
                    value: key.to_string(),
                    accepted: concat!("one of ", $( $key, " " ),+),
                    field: None,
                })
            }
        }
    };
}

knob!(
    /// Tropical or sidereal longitudes.
    Zodiac { /// Tropical.
        Tropical = "TROPICAL", /// Sidereal, with the ayanamsha knob.
        Sidereal = "SIDEREAL" }
);
knob!(
    /// Whether the ayanamsha carries the nutation in longitude.
    AyanamshaBasis { /// The mean value.
        Mean = "MEAN", /// With nutation.
        True = "TRUE" }
);
knob!(
    /// Which lunar node.
    Node { /// The mean node.
        Mean = "MEAN", /// The true (osculating) node.
        True = "TRUE" }
);
knob!(
    /// Where positions are observed from.
    Centre { /// The Earth's centre.
        Geocentric = "GEOCENTRIC", /// The place of birth.
        Topocentric = "TOPOCENTRIC" }
);
knob!(
    /// Apparent or true positions.
    Positions { /// Light time, aberration and deflection applied.
        Apparent = "APPARENT", /// Geometric positions.
        True = "TRUE" }
);
knob!(
    /// Which nakshatra scheme.
    NakshatraScheme { /// Twenty-seven equal spans.
        TwentySeven = "TWENTY_SEVEN", /// Twenty-eight with Abhijit.
        TwentyEight = "TWENTY_EIGHT" }
);
knob!(
    /// What to do where a house system has no solution.
    PolarPolicy { /// Refuse the chart.
        Error = "ERROR", /// Whole-sign houses.
        FallbackWholeSign = "FALLBACK_WHOLE_SIGN", /// Porphyry houses.
        FallbackPorphyry = "FALLBACK_PORPHYRY", /// The nearest defined latitude.
        Clamp = "CLAMP" }
);
knob!(
    /// Which moment sunrise is.
    Sunrise { /// The centre of the disc on the geometric horizon.
        CentreNoRefraction = "CENTRE_NO_REFRACTION", /// The upper limb with refraction.
        UpperLimbRefraction = "UPPER_LIMB_REFRACTION", /// The lower limb with refraction.
        LowerLimbRefraction = "LOWER_LIMB_REFRACTION" }
);
knob!(
    /// When the day begins.
    DayBoundary { /// Civil midnight.
        Midnight = "MIDNIGHT", /// Sunrise at the place.
        Sunrise = "SUNRISE", /// Sunset at the place.
        Sunset = "SUNSET", /// Local apparent noon.
        Noon = "NOON" }
);
knob!(
    /// What a day without a sunrise is.
    PolarDayPolicy { /// An undefined state.
        Undefined = "UNDEFINED", /// The nearest rise or set.
        NearestEvent = "NEAREST_EVENT", /// Civil midnight.
        CivilMidnight = "CIVIL_MIDNIGHT" }
);
knob!(
    /// How ghatis are counted.
    GhatiReckoning { /// Twenty-four minutes each.
        Civil = "CIVIL", /// Thirty over the actual day and night.
        Proportional = "PROPORTIONAL" }
);
knob!(
    /// Which window a daily panchanga finds the Moon's rise and set in.
    ///
    /// Every other field of an almanac day is bounded by sunrise; the
    /// recording engine's moonrise and moonset are the first at or after
    /// local civil midnight, so a quarter of them fall outside the day
    /// its own limbs occupy (entry 18 of the deliberate-difference
    /// registry). The SDK uses the day's own window and offers the other
    /// reading rather than hiding it.
    MoonEvents { /// The rises and sets inside the panchanga day.
        Window = "WINDOW", /// The first rise and the first set at or after local civil midnight.
        CivilDay = "CIVIL_DAY" }
);
knob!(
    /// How the planetary hours (horas) are counted.
    HoraReckoning { /// Twelve over the daylight and twelve over the night.
        Proportional = "PROPORTIONAL", /// Twenty-four of sixty minutes from sunrise.
        Equal = "EQUAL" }
);
knob!(
    /// A civil time inside a daylight-saving gap.
    DstGap { /// Refuse.
        Error = "ERROR", /// Add the gap.
        ShiftForward = "SHIFT_FORWARD" }
);
knob!(
    /// A civil time repeated by a daylight-saving overlap.
    DstOverlap { /// The earlier offset.
        Earlier = "EARLIER", /// The later offset.
        Later = "LATER", /// Refuse.
        Error = "ERROR" }
);
knob!(
    /// A birth without a time.
    UnknownTime { /// Refuse.
        Refuse = "REFUSE", /// Noon.
        Noon = "NOON", /// Sunrise at the place.
        Sunrise = "SUNRISE", /// Midnight.
        Midnight = "MIDNIGHT" }
);
knob!(
    /// Which Delta T.
    DeltaT { /// The measured table, then the model.
        TableThenModel = "TABLE_THEN_MODEL", /// Espenak and Meeus 2006.
        EspenakMeeus2006 = "ESPENAK_MEEUS_2006", /// Stephenson, Morrison and Hohenkerk 2016.
        StephensonMorrisonHohenkerk2016 = "STEPHENSON_MORRISON_HOHENKERK_2016", /// The provider's own.
        Provider = "PROVIDER" }
);
knob!(
    /// How a dasha's first period is balanced.
    Balance { /// By the elapsed fraction of the seed span.
        Spatial = "SPATIAL", /// By elapsed time.
        Temporal = "TEMPORAL" }
);
knob!(
    /// The length of a dasha year.
    YearLength { /// 365.25 days.
        Julian36525 = "JULIAN_365_25", /// 360 days.
        Savana360 = "SAVANA_360", /// The sidereal year.
        Sidereal = "SIDEREAL", /// The tropical year.
        Tropical = "TROPICAL", /// Twelve lunar months.
        Lunar = "LUNAR", /// 324 days.
        Nakshatra324 = "NAKSHATRA_324" }
);
impl YearLength {
    /// The days in one dasha year of this length.
    ///
    /// The astronomical lengths are the mean values at J2000 the
    /// *Astronomical Almanac* gives in its glossary: the sidereal year
    /// 365.256 363 days, the tropical year 365.242 190 days, and the synodic
    /// month 29.530 589 days, of which the lunar year is twelve. The others
    /// are the counts their names state: the Julian 365.25, the savana 360,
    /// and the nakshatra year of twelve 27-day months, 324.
    #[must_use]
    pub const fn days(self) -> f64 {
        match self {
            YearLength::Julian36525 => 365.25,
            YearLength::Savana360 => 360.0,
            YearLength::Sidereal => 365.256_363,
            YearLength::Tropical => 365.242_190,
            YearLength::Lunar => 12.0 * 29.530_589,
            YearLength::Nakshatra324 => 324.0,
        }
    }
}

knob!(
    /// How a dasha's birth period is divided among its sub-periods.
    BirthPeriod { /// Each sub-period its share of the balance the birth period runs for,
        /// which the recording engine does on every recorded answer.
        Compressed = "COMPRESSED", /// Each sub-period its share of the whole period, which began
        /// before birth; those already over are dropped and the one running
        /// is cut at birth.
        Elapsed = "ELAPSED" }
);
knob!(
    /// What a dasha answers past the end of its cycle.
    AfterCycle { /// No period: the cycle has ended, as the recording engine answers.
        End = "END", /// The cycle begins again, from its first lord and in full.
        Repeat = "REPEAT" }
);
knob!(
    /// Which pada table a nakshatra takes in the Kalachakra dasha (crux C54).
    KalachakraMembership { /// The lists the conformance corpus's recording engine carries,
        /// which differ from the triad rule at Ardra, Uttara Phalguni,
        /// Jyeshtha, Shatabhisha and Revati.
        Listed = "LISTED", /// By the nakshatra's place in its triad, as the published lists
        /// read have it: the middle of each triad takes its chakra's second
        /// table and the outer two the first.
        Triad = "TRIAD" }
);
knob!(
    /// How the Kalachakra balance at birth is taken (crux C55).
    KalachakraBalance { /// The unelapsed part of the pada, of the first sign's years, as the
        /// recording engine takes it.
        FirstSign = "FIRST_SIGN", /// The elapsed part of the pada, of the pada's whole span, the signs
        /// it covers skipped.
        WholePada = "WHOLE_PADA" }
);
knob!(
    /// What follows the ninth Kalachakra mahadasha (crux C56).
    KalachakraAfterNinth { /// The same nine signs reversed, as the recording engine runs them.
        Reverse = "REVERSE", /// The same nine in the same order.
        Repeat = "REPEAT" }
);
knob!(
    /// A seed outside a conditional dasha's cycle.
    SeedOverflow { /// Wrap to the start, flagged.
        WrapToStart = "WRAP_TO_START", /// Refuse.
        Reject = "REJECT" }
);
knob!(
    /// How many chara karakas.
    CharaKarakas { /// Seven.
        Seven = "SEVEN", /// Eight.
        Eight = "EIGHT" }
);
knob!(
    /// The nodes' co-lordship of Aquarius and Scorpio.
    NodeCoLordship { /// None.
        None = "NONE", /// The stronger lord.
        StrongerLord = "STRONGER_LORD", /// Both.
        Both = "BOTH" }
);
knob!(
    /// The aspects of the nodes.
    NodeAspects { /// None beyond the seventh.
        None = "NONE", /// The fifth, seventh and ninth.
        FiveSevenNine = "FIVE_SEVEN_NINE", /// The third, seventh and eleventh.
        ThreeSevenEleven = "THREE_SEVEN_ELEVEN" }
);
knob!(
    /// How the Ashtakavarga's Ekadhipatya reduction treats a co-ruled sign
    /// beside an occupied one (crux C60).
    Ekadhipatya { /// BPHS ch. 68: an empty sign beside an occupied sign with the smaller
        /// number keeps the difference, and otherwise goes to zero.
        Bphs = "BPHS", /// The conformance corpus's recording engine: the empty sign always
        /// goes to zero.
        EmptyToZero = "EMPTY_TO_ZERO" }
);
knob!(
    /// How the Vimshopaka scores a graha in a varga (crux C63).
    Vimshopaka { /// BPHS ch. 7: 20 in exaltation or the own sign, else 18, 15, 10, 7
        /// or 5 by the compound relationship with the sign's lord in the rasi
        /// chart, times the varga's weight over 20.
        Bphs = "BPHS", /// The conformance corpus's recording engine: the Saptavargaja
        /// virupas over 45 by natural friendship, rounded to hundredths.
        SaptavargajaVirupas = "SAPTAVARGAJA_VIRUPAS" }
);
knob!(
    /// How the Shadbala's Saptavargaja scores a graha in each of its seven
    /// vargas (crux C64).
    Saptavargaja { /// B.V. Raman's reading of BPHS ch. 27: moolatrikona 45 (in the rasi
        /// chart by its degrees), the own sign 30, then 22.5, 15, 7.5, 3.75 or
        /// 1.875 by the compound relationship, its temporary half from the rasi
        /// chart; exaltation is Uchcha's.
        Compound = "COMPOUND", /// The conformance corpus's recording engine: exaltation 45,
        /// moolatrikona or the own sign 30, then 15, 7.5 or 3.75 by natural
        /// friendship alone, debilitation nothing.
        Natural = "NATURAL" }
);
knob!(
    /// How the Shadbala's Nathonnatha measures the hour (crux C65).
    Nathonnatha { /// BPHS ch. 27 vv. 8 and 9: from midnight at any hour, the Moon, Mars
        /// and Saturn twice the nata in ghatis, the Sun, Jupiter and Venus 60
        /// less that.
        Midnight = "MIDNIGHT", /// The recording engine: the day grahas rise to 60 at the middle of
        /// the daylight and the night grahas at the middle of the night, each
        /// with nothing outside its own arc.
        Arc = "ARC" }
);
knob!(
    /// Which night a birth before sunrise is measured in, for the Shadbala's
    /// Nathonnatha and Tribhaga (crux C65).
    PreDawnNight { /// The night it falls in, from the previous evening's sunset.
        PreviousEvening = "PREVIOUS_EVENING", /// The recording engine's: from the civil date's own sunset, which
        /// comes after the birth, so no night graha gains.
        SameEvening = "SAME_EVENING" }
);
knob!(
    /// Whether the Sun's Ayana bala is counted, doubled, in the Shadbala's
    /// Kaala (crux C66).
    SunAyana { /// BPHS ch. 27 v. 17 and Sripati: doubled in Kaala.
        Doubled = "DOUBLED", /// The recording engine: not in Kaala at all.
        NotInKaala = "NOT_IN_KAALA" }
);
knob!(
    /// The Cheshta bala of the Sun and the Moon, which never retrogress
    /// (crux C66).
    LuminaryCheshta { /// BPHS ch. 27 v. 18: the Sun's is its Ayana bala and the Moon's her
        /// Paksha bala.
        AyanaAndPaksha = "AYANA_AND_PAKSHA", /// The recording engine: the Sun's Ayana, and a third of the Moon's
        /// elongation.
        AyanaAndElongation = "AYANA_AND_ELONGATION", /// Sripati, as B.V. Raman works it: neither has any.
        None = "NONE" }
);
knob!(
    /// The declination the Shadbala's Ayana bala reads, and the obliquity
    /// its formula scales by (crux C66).
    Kranti { /// The graha's true declination, from its ecliptic latitude and the
        /// date's obliquity, as BPHS's notes take it from an ephemeris.
        True = "TRUE", /// The recording engine: its tropical longitude at zero latitude, on
        /// an obliquity of 23.4393°.
        Ecliptic = "ECLIPTIC", /// Sripati, as B.V. Raman works it: the Hindu table of 362′, 341′,
        /// 299′, 236′, 150′ and 52′ a 15° step of the sayana bhuja, on 24°.
        HinduTable = "HINDU_TABLE" }
);
knob!(
    /// Whose weekdays the Shadbala's Abda and Masa lords are (crux C67).
    KaalaLords { /// BPHS ch. 27 v. 13: the ahargana's 360-day year and 30-day month.
        Ahargana = "AHARGANA", /// The recording engine: the UT weekday of the last Mesha sankranti
        /// for the year, and the lord of the Sun's sign for the month.
        Sankranti = "SANKRANTI" }
);
knob!(
    /// The kendras the Shadbala's Dig bala measures from (crux C68).
    DigKendras { /// BPHS ch. 27 v. 7: the ascendant, the nadir, the descendant and
        /// the midheaven.
        Angles = "ANGLES", /// The recording engine: all four projected from the lagna's degree,
        /// 30° a house.
        LagnaProjection = "LAGNA_PROJECTION" }
);
knob!(
    /// How the Shadbala's Drik bala weighs the drishtis a graha receives
    /// (crux C69).
    Drik { /// BPHS ch. 27 v. 19: a quarter of each benefic's sphuta drishti added
        /// and each malefic's taken away, and Jupiter's and Mercury's added
        /// again in full.
        QuarterWithJupiterMercury = "QUARTER_WITH_JUPITER_MERCURY", /// Sripati, as B.V. Raman works it: a quarter of the net sphuta
        /// drishti pinda.
        Quarter = "QUARTER", /// The recording engine: each whole-sign graded drishti in full,
        /// Mercury's taken away, the sum bounded at ±60.
        Full = "FULL" }
);
knob!(
    /// The Shadbala's natural strengths (crux C71).
    Naisargika { /// BPHS ch. 27 v. 14: exact sevenths of a rupa.
        Exact = "EXACT", /// The recording engine's, rounded to hundredths.
        Hundredths = "HUNDREDTHS" }
);
knob!(
    /// The rupas a graha's Shadbala must reach to be strong (crux C71).
    RequiredRupas { /// BPHS ch. 27 vv. 32 and 33: 6.5, 6, 5, 7, 6.5, 5.5 and 5, Sun to
        /// Saturn.
        Bphs = "BPHS", /// Sripati, as B.V. Raman gives it and the recording engine takes it:
        /// the Sun's 5.
        Sripati = "SRIPATI" }
);
knob!(
    /// Which decanate gives each gender of graha the Shadbala's Drekkana bala
    /// (crux C72).
    Drekkana { /// BPHS ch. 27 v. 6 as translated, and the recording engine: male
        /// grahas the first, female the second, neuter the third.
        MaleFemaleNeuter = "MALE_FEMALE_NEUTER", /// Sripati, as B.V. Raman works it: male the first, neuter the
        /// second, female the third.
        MaleNeuterFemale = "MALE_NEUTER_FEMALE" }
);
knob!(
    /// Which grahas count as benefics for the Shadbala's Paksha and Drik
    /// balas (crux C69).
    Benefics { /// Jupiter and Venus; the Moon from the eighth day of the bright half
        /// to the eighth of the dark; Mercury unless it shares its sign with
        /// the Sun, Mars, Saturn, a node or a malefic Moon (B.V. Raman after
        /// Sripati).
        Conditional = "CONDITIONAL", /// The recording engine: the Moon, Mercury, Jupiter and Venus, always.
        Fixed = "FIXED" }
);
knob!(
    /// The mean elements the Shadbala's Cheshta bala reads for Mars to Saturn
    /// (crux C70).
    Cheshta { /// Kedarnath Dutt's tables as B.V. Raman gives them, the inferior
        /// planets' mean the mean Sun's.
        Sripati = "SRIPATI", /// The recording engine's J2000 elements, each inferior planet's own
        /// seeghrochcha taken as its mean too.
        RecordingEngine = "RECORDING_ENGINE" }
);
knob!(
    /// Whether the Shadbala counts the Yuddha bala of grahas at war (crux C70).
    Yuddha { /// Sripati, as B.V. Raman works it: two of Mars to Saturn within a
        /// degree, the one of lesser longitude the victor, the difference of
        /// their Sthana, Dig and Kaala over the difference of their discs.
        Sripati = "SRIPATI", /// The recording engine: never.
        None = "NONE" }
);
knob!(
    /// How a graha's Ishta and Kashta phalas are read from its Uchcha and
    /// Cheshta balas (crux C76).
    IshtaKashta { /// BPHS ch. 28 vv. 2 to 6: the Uchcha and Cheshta rays, the Ishta their
        /// mean over the balas and the Kashta 60 less it, the Sun's Cheshta
        /// kendra its sayana longitude and three signs, the Moon's her
        /// elongation.
        Rays = "RAYS", /// Sripati, as B.V. Raman works it: the Ishta the square root of the
        /// two balas' product and the Kashta of their complements', the
        /// luminaries' Cheshta from the same kendras.
        SquareRoots = "SQUARE_ROOTS", /// The recording engine: Raman's square roots of the Shadbala's own
        /// Uchcha and Cheshta balas.
        ShadbalaCheshta = "SHADBALA_CHESHTA" }
);
knob!(
    /// How the Bhava bala's Dig bala reads a bhava's sign class (crux C73).
    BhavaDig { /// BPHS ch. 27 vv. 26 to 28 as translated: the arc, over 3, from the
        /// bhava madhya to the madhya of the house its class makes weakest,
        /// Cancer and Scorpio insects, Sagittarius and Capricorn split at their
        /// halves.
        Bphs = "BPHS", /// Sripati, as B.V. Raman works it: ten virupas a house from that
        /// house, Scorpio the only insect and Cancer watery, the halves split.
        Sripati = "SRIPATI", /// The recording engine: ten virupas a house by the whole sign,
        /// Sagittarius human and Capricorn quadruped throughout, Cancer an insect.
        WholeSign = "WHOLE_SIGN" }
);
knob!(
    /// How the Bhava bala weighs the drishtis a bhava receives (crux C74).
    BhavaDrishti { /// BPHS ch. 27 v. 29 as translated, and the recording engine: a quarter
        /// of the Dig bala added for a benefic's full drishti and taken away for
        /// a malefic's, with Jupiter's and Mercury's Drik bala added when they
        /// aspect it.
        QuarterOfDig = "QUARTER_OF_DIG", /// Sripati, as B.V. Raman works it: the sphuta drishti on the bhava
        /// madhya, Jupiter's and Mercury's in full and a quarter of each other
        /// graha's, Mercury always a benefic.
        Sphuta = "SPHUTA" }
);
knob!(
    /// Whether the Bhava bala adds BPHS ch. 27 vv. 30 and 31's special rules
    /// (crux C75).
    BhavaSpecialRules { /// A rupa for each Jupiter and Mercury in the bhava and less one for
        /// each Sun, Mars and Saturn, and 15 virupas for a sirshodaya sign by
        /// day, a prishtodaya sign by night and an ubhayodaya sign in twilight.
        Bphs = "BPHS", /// Sripati, as B.V. Raman works it, and the recording engine: none.
        None = "NONE" }
);
knob!(
    /// Where the Ashtakavarga's reductions and pindas are made (crux C59).
    Shodhana { /// In each graha's own Ashtakavarga, its pindas from its own reduced
        /// bindus and the grahas standing in each sign (BPHS chs. 67 to 69).
        EachGraha = "EACH_GRAHA", /// On the sum of the seven, as the conformance corpus's recording
        /// engine makes them: the rashi pinda of the reduced sum and a graha
        /// pinda of the raw bindus.
        Sarva = "SARVA" }
);
knob!(
    /// The convention for a divisional chart no text attests.
    UnattestedDn { /// Cyclic (parivritti).
        Cyclic = "CYCLIC" }
);
knob!(
    /// The lunar month system.
    LunarMonth { /// New moon to new moon.
        Amanta = "AMANTA", /// Full moon to full moon.
        Purnimanta = "PURNIMANTA" }
);
knob!(
    /// The provider override policy (ADR-0013).
    OverridePolicy { /// A declared native implementation is used.
        PreferNative = "PREFER_NATIVE", /// The SDK's own everywhere.
        SdkOnly = "SDK_ONLY", /// Native or refuse.
        NativeOnly = "NATIVE_ONLY" }
);
knob!(
    /// The built-in ephemeris tier.
    Tier { /// About an arcminute.
        Compact = "COMPACT", /// About an arcsecond.
        Standard = "STANDARD", /// The theories' accuracy.
        Full = "FULL", /// The DE refit.
        Reference = "REFERENCE" }
);

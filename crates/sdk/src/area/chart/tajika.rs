//! Tajika through the façade (`03-design/annual-chart.md`): the years'
//! returns and their charts, the office-bearers, the aspects and yogas,
//! the sahams, the strengths and the annual dashas.

use teistro_astro::completion::Completion;
use teistro_astro::precession::PrecessionModel;
use teistro_chart::foundation::ChartFoundation;
use teistro_core::angle::Nas;
use teistro_core::catalogue::{DashaSystem, Graha, Nakshatra};
use teistro_core::envelope::Envelope;
use teistro_core::error::Error;
use teistro_core::house::House;
use teistro_core::settings::Balance;
use teistro_core::time::UtcOffset;
use teistro_dasha::{Wheel, YearDasha, YearRing};
use teistro_serial::Document;
use teistro_state::state;
use teistro_tajika::{
    Affliction, AnnualDasha, AnnualDashaRules, AnnualSky, AnnualStates, Between, DrishtiRules,
    Favour, Harsha, HarshaRules, MuddaBalance, Muntha, MunthaDegree, Natal, OfficeBearers,
    Panchavargiya, Pravesha, Qualification, Reading, SEVEN, Saham, SahamFormula, SahamPlace,
    SahamReading, SahamRules, SahamSky, SahamStrength, SahamStrengthRules, Strength, Varshesha,
    VarsheshaRules, YearCharts, YearYogas, YogaRules,
};

use super::ChartArea;
use crate::ephemeris::no_ephemeris;
use crate::reading::ChartRequest;
use crate::varsha::{AnnualChart, AnnualPlace, VARSHA, Varsha, VarshaRequest, VarshaYear};

impl ChartArea<'_> {
    /// The annual charts' instants: the Sun's returns to where it stood at
    /// birth, `1` opening the first year of life
    /// (`03-design/annual-chart.md`).
    ///
    /// Read in the chart's **own** zodiac, on the ayanamsha basis the
    /// settings name — not a frame's sidereal reading, which applies the
    /// mean ayanamsha where a founded chart applies the nutated one and
    /// lands about two degrees of lagna away
    /// (`03-design/annual-chart-measured.md`).
    ///
    /// Fewer instants than asked for is the answer rather than an error:
    /// an ephemeris that ends before a birth's fortieth year has said so.
    ///
    /// ```no_run
    /// # use teistro::{ChartRequest, Context, Document, Ephemeris};
    /// # use teistro::tajika::Reading;
    /// # fn main() -> Result<(), teistro::Error> {
    /// # let sdk = Context::builder().ephemeris([Ephemeris::Builtin]).build()?;
    /// # let document: Document = todo!();
    /// let years = sdk.chart().praveshas(&document, Reading::Sidereal, 40)?;
    /// let thirtieth = years.iter().find(|one| one.year == 30);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// No ephemeris; a `through` outside one to two hundred, named
    /// `through`; whatever the provider refuses while searching.
    pub fn praveshas(
        self,
        document: &Document,
        reading: Reading,
        through: u16,
    ) -> Result<Vec<Pravesha>, Error> {
        // The argument is refused before anything is looked up, so a
        // nonsense year is reported as one whichever reading was asked
        // for and whether or not a provider is attached.
        let asked = teistro_tajika::years(through)?;
        let foundation = &document.foundation;
        let natal = Self::natal_of(foundation)?;
        if reading == Reading::Mean {
            return teistro_tajika::mean_praveshas(&natal, asked);
        }
        let provider = self.context.ephemeris().ok_or_else(no_ephemeris)?;
        let settings = self.context.settings();
        let completion = Completion::new(
            provider,
            settings.provider.overrides,
            self.context.delta_t(),
        );
        let searched = foundation.zodiac.search(
            &completion,
            settings.frame.ayanamsha_basis,
            (PrecessionModel::default(), self.context.delta_t()),
        )?;
        let (frame, zodiac) = (searched.frame, searched.zodiac);
        let mut longitudes = completion.longitudes(frame);
        if frame.centre == teistro_port_ephemeris::Centre::Topocentric {
            longitudes = longitudes.with_observer(foundation.place);
        }
        // Asked for more years than the provider covers, answer the ones
        // it does. Without this the search runs off the end and the
        // provider's own `OutOfRange` comes back naming an instant the
        // caller never mentioned — true, and useless to act on. Fewer
        // instants than asked for is the answer this method documents,
        // and none of them is the answer when the coverage reaches none;
        // the refusal for a nonsense year already happened above, before
        // capping could turn it into an empty list.
        let through = asked.min(Self::years_covered(&completion, foundation.instant.get()));
        if through == 0 {
            return Ok(Vec::new());
        }
        teistro_tajika::praveshas(&longitudes, zodiac, &natal, reading, through)
    }

    /// How many whole returns the provider's coverage holds after an
    /// instant.
    ///
    /// A year short rather than a year long: the last return inside the
    /// coverage is the last one that can be *searched for*, and the search
    /// samples a step past where it expects to find it.
    fn years_covered<P: teistro_port_ephemeris::EphemerisProvider + ?Sized>(
        completion: &Completion<'_, P>,
        birth: f64,
    ) -> u16 {
        let covered = completion.capabilities().jd_range.1;
        let years = (covered - birth - teistro_tajika::STEP_DAYS * 2.0)
            / teistro_tajika::SIDEREAL_YEAR_DAYS;
        if years <= 0.0 {
            return 0;
        }
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "a positive count of years is clamped to u16 below"
        )]
        let years = years.floor().min(f64::from(u16::MAX)) as u16;
        years
    }

    /// One annual chart, founded: the document of the year a return opens.
    ///
    /// The place and the zone are the birth's, which is the default the
    /// schools differ on rather than a rule
    /// (`03-design/annual-chart.md`); a consumer casting for a residence
    /// asks for the instant with [`ChartArea::praveshas`] and founds the
    /// chart themselves, which is the same call this makes.
    ///
    /// # Errors
    ///
    /// As [`ChartArea::praveshas`]; a year the ephemeris does not reach is
    /// refused by `year` with the years it does reach.
    pub fn annual(
        self,
        document: &Document,
        reading: Reading,
        year: u16,
        request: &ChartRequest,
    ) -> Result<Envelope<Document>, Error> {
        let found = self.praveshas(document, reading, year)?;
        let at = found
            .iter()
            .find(|one| one.year == year)
            .ok_or_else(|| {
                Error::invalid_arg(format!(
                    "the ephemeris does not reach year {year} of this birth"
                ))
                .with_field("year")
                .with_hint(format!(
                    "it reaches {}",
                    found.last().map_or_else(
                        || String::from("none of its years"),
                        |one| format!("year {}", one.year)
                    )
                ))
            })?
            .at;
        self.reading(at, request)
    }

    /// The **annual charts** a request asks of one birth: each year's
    /// return and Muntha, and — where [`VarshaRequest::place`] asks for the
    /// charts — each year's own chart read down to its office-bearers,
    /// year lord, yogas by matter, sahams, Harsha bala and annual dashas
    /// (`03-design/annual-chart.md`).
    ///
    /// The one call every binding's `varsha` makes, so a year is composed
    /// once, here; the parts are each a method of their own for a caller
    /// who wants one.
    ///
    /// `clock` is the one the birth was cast under, which
    /// [`AnnualPlace::Birth`] casts each year's chart under too.
    ///
    /// ```no_run
    /// # use teistro::{AnnualPlace, Asked, ChartRequest, Context, Document, Ephemeris, VarshaRequest};
    /// # fn main() -> Result<(), teistro::Error> {
    /// # let sdk = Context::builder().ephemeris([Ephemeris::Builtin]).build()?;
    /// # let (birth, request): (Document, ChartRequest) = todo!();
    /// let asked = VarshaRequest::through(40)
    ///     .at(AnnualPlace::Birth)
    ///     .with_sahams(Asked::All);
    /// let varsha = sdk.chart().varsha(&birth, request.offset(), &asked)?;
    /// let fortieth = varsha.years.last().and_then(|year| year.annual.as_ref());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// A request [`VarshaRequest::check`] refuses; no ephemeris; whatever
    /// the provider refuses while searching or founding. Each is named
    /// under `varsha`, the record's name in every binding.
    pub fn varsha(
        self,
        birth: &Document,
        clock: UtcOffset,
        asked: &VarshaRequest,
    ) -> Result<Varsha, Error> {
        asked.check()?;
        self.varsha_of(birth, clock, asked)
            .map_err(|error| error.under(VARSHA))
    }

    /// [`ChartArea::varsha`] for a request already checked, its refusals
    /// named as the parts name them.
    fn varsha_of(
        self,
        birth: &Document,
        clock: UtcOffset,
        asked: &VarshaRequest,
    ) -> Result<Varsha, Error> {
        let years = self
            .praveshas(birth, asked.reading, asked.through)?
            .into_iter()
            .map(|pravesha| {
                // The Muntha is progressed by the years the return
                // *completes*, which is exactly what `year` counts.
                let muntha = self.muntha(birth, pravesha.year, asked.muntha)?;
                let annual = asked
                    .place
                    .map(|place| self.annual_chart(birth, clock, place, asked, pravesha))
                    .transpose()?;
                Ok(VarshaYear {
                    pravesha,
                    muntha,
                    annual,
                })
            })
            .collect::<Result<Vec<VarshaYear>, Error>>()?;
        // The birth's own sahams, which have no year lord.
        let natal_sahams = match &asked.sahams {
            Some(sahams) => self.saham_strength_with_rules(
                birth,
                sahams.members(),
                None,
                asked.strength_rules(),
            )?,
            None => Vec::new(),
        };
        Ok(Varsha {
            years,
            natal_sahams,
        })
    }

    /// A year's own chart, founded where the caller said and read down to
    /// what Tajika reads from it.
    ///
    /// Founded with a bare request — the foundation and nothing else —
    /// because a year's chart asked for the birth's sections too would cost
    /// each year a whole reading nobody requested.
    fn annual_chart(
        self,
        birth: &Document,
        clock: UtcOffset,
        place: AnnualPlace,
        asked: &VarshaRequest,
        pravesha: Pravesha,
    ) -> Result<AnnualChart, Error> {
        let request = match place {
            AnnualPlace::Birth => ChartRequest::at(birth.foundation.place, clock),
            AnnualPlace::At { place, offset } => ChartRequest::at(place, offset),
        };
        let annual = self.reading(pravesha.at, &request)?.value;
        let bearers = self.office_bearers(birth, &annual, pravesha.year)?;
        let year_lord = self.varshesha(birth, &annual, pravesha.year, asked.varshesha)?;
        let yogas = self
            .drishtis(&annual)?
            .into_iter()
            .filter(|pair| pair.yoga.is_some())
            .collect();
        let matters = match &asked.matters {
            Some(matters) => self.tajika_yogas_many(&annual, matters.members(), asked.yogas)?,
            None => Vec::new(),
        };
        let sahams = match &asked.sahams {
            Some(sahams) => self.saham_strength_with_rules(
                &annual,
                sahams.members(),
                Some(year_lord.graha),
                asked.strength_rules(),
            )?,
            None => Vec::new(),
        };
        let harsha = self.harsha_with_rules(&annual, asked.harsha_rules)?;
        // One call for every system asked, so the Sun is read over the year
        // once however many divide it.
        let dashas = match &asked.dashas {
            Some(dashas) => self.annual_dashas(
                birth,
                &annual,
                pravesha.year,
                dashas.members(),
                asked.dasha_rules,
            )?,
            None => Vec::new(),
        };
        Ok(AnnualChart {
            lagna_deg: annual.foundation.lagna_deg,
            bearers,
            year_lord,
            yogas,
            states: self.annual_states(&annual)?,
            matters,
            sahams,
            harsha,
            dashas,
        })
    }

    /// The **Muntha** at one of a birth's returns: the lagna's sign
    /// progressed one sign for each completed year, and that sign's lord.
    ///
    /// `completed_years` is exactly a [`Pravesha::year`], which counts
    /// returns — pass that field in unchanged. Zero is the birth, where
    /// the Muntha sits on the lagna.
    ///
    /// **It needs no ephemeris.** The whole progression is the founded
    /// chart's own lagna and a count of years, so a context with no
    /// provider attached answers it, exactly as [`Reading::Mean`] does
    /// for the returns themselves.
    ///
    /// # Errors
    ///
    /// A `completed_years` past two hundred, named `completed_years`; a
    /// lagna that is not a number, named `lagna`.
    pub fn muntha(
        self,
        document: &Document,
        completed_years: u16,
        degree: MunthaDegree,
    ) -> Result<Muntha, Error> {
        teistro_tajika::muntha(document.foundation.lagna_deg, completed_years, degree)
    }

    /// The annual chart's five **office-bearers**, read from the birth
    /// chart and an annual chart **you founded** — for the birthplace or
    /// for a residence, which is your choice and not the SDK's.
    ///
    /// `completed_years` is the [`Pravesha::year`] the annual chart was
    /// founded for. Three of the five depend on the annual chart's place
    /// (its lagna, and whether the return fell between sunrise and sunset
    /// there), which is why this takes the chart rather than an instant.
    ///
    /// It needs **no ephemeris**: both charts are already founded.
    ///
    /// # Errors
    ///
    /// A `completed_years` past two hundred, named `completed_years`; an
    /// annual chart that places no Sun or no Moon.
    pub fn office_bearers(
        self,
        natal: &Document,
        annual: &Document,
        completed_years: u16,
    ) -> Result<OfficeBearers, Error> {
        let year = &annual.foundation;
        let at = |graha: Graha| {
            year.graha(graha)
                .map(|placed| placed.longitude_deg)
                .ok_or_else(|| Error::internal(format!("a founded chart places {graha:?}")))
        };
        teistro_tajika::office_bearers(&YearCharts {
            natal_lagna_deg: natal.foundation.lagna_deg,
            completed_years,
            annual_lagna_deg: year.lagna_deg,
            annual_sun_deg: at(Graha::Sun)?,
            annual_moon_deg: at(Graha::Moon)?,
            by_day: year.day.part.is_daylight(),
        })
    }

    /// The **Panchavargiya bala** of the seven, read from an annual chart
    /// you founded: the five-fold strength the lord of the year is chosen
    /// by.
    ///
    /// Exact, in sub-sub units — the source's own unit, and its worked
    /// chart's last figure — so nothing here rounds and two answers can be
    /// compared as integers.
    ///
    /// It needs **no ephemeris**: the chart is already founded.
    ///
    /// # Errors
    ///
    /// An annual chart that does not place one of the seven.
    pub fn panchavargiya(self, annual: &Document) -> Result<[Panchavargiya; 7], Error> {
        teistro_tajika::panchavargiya(&Self::sky_of(annual)?)
    }

    /// The **Harsha bala** of the seven, read from an annual chart you
    /// founded under the source's readings: four places a planet is
    /// "happy" in, five units each (`03-design/tajika-harsha.md`).
    ///
    /// The houses are whole signs from the chart's lagna, and the fourth
    /// part is read from whether the year opened by day. It needs **no
    /// ephemeris**: the chart is already founded.
    ///
    /// # Errors
    ///
    /// An annual chart that does not place one of the seven, or whose
    /// lagna is not a number.
    pub fn harsha(self, annual: &Document) -> Result<[Harsha; 7], Error> {
        self.harsha_with_rules(annual, HarshaRules::default())
    }

    /// The **Harsha bala** under the readings you name: Venus's house of
    /// joy the verse's 5th, or the 12th a widely used program reads.
    ///
    /// # Errors
    ///
    /// As [`ChartArea::harsha`].
    pub fn harsha_with_rules(
        self,
        annual: &Document,
        rules: HarshaRules,
    ) -> Result<[Harsha; 7], Error> {
        let foundation = &annual.foundation;
        teistro_tajika::harsha(
            &Self::sky_of(annual)?,
            foundation.lagna_deg,
            foundation.day.part.is_daylight(),
            rules,
        )
    }

    /// The **Varshesha**, the lord of the year, from a birth chart and an
    /// annual chart you founded.
    ///
    /// One call for the whole chain: the five office-bearers, their
    /// five-fold strengths, and the rule that picks among them — the
    /// strongest that **aspects the annual lagna**, with the source's
    /// fallbacks each a named step. The answer carries every claimant and
    /// which step decided it, because a year lord chosen on strength and
    /// one chosen by a fallback are different statements about the year.
    ///
    /// It needs **no ephemeris**: both charts are already founded.
    ///
    /// # Errors
    ///
    /// As [`ChartArea::office_bearers`] and
    /// [`ChartArea::panchavargiya`]; an annual chart whose lagna is not a
    /// number.
    pub fn varshesha(
        self,
        natal: &Document,
        annual: &Document,
        completed_years: u16,
        rules: VarsheshaRules,
    ) -> Result<Varshesha, Error> {
        let bearers = self.office_bearers(natal, annual, completed_years)?;
        let sky = Self::sky_of(annual)?;
        let strengths = teistro_tajika::panchavargiya(&sky)?;
        teistro_tajika::varshesha(
            &bearers,
            &sky,
            &strengths,
            annual.foundation.lagna_deg,
            rules,
        )
    }

    /// The **Tajika aspects** of an annual chart you founded: every pair
    /// of the seven, with the orb that governs it and whether the two are
    /// coming together or drawing apart.
    ///
    /// Twenty-one pairs. Each carries the sign aspect (the kendras and
    /// the 3, 5, 9, 11 houses; the rest is no aspect at all), the mean of
    /// the two deeptamshas, how far apart they stand **within their
    /// signs** — which is how this tradition counts behind from ahead —
    /// and the **Ithasala** — in one of its three kinds — or **Ishrafa**
    /// that makes, if any.
    ///
    /// It needs **no ephemeris**: the chart is already founded.
    ///
    /// [`ChartArea::drishtis_with_rules`] takes the readings the source
    /// leaves open; this is that under [`DrishtiRules::default`].
    ///
    /// # Errors
    ///
    /// An annual chart that does not place one of the seven.
    pub fn drishtis(self, annual: &Document) -> Result<Vec<Between>, Error> {
        self.drishtis_with_rules(annual, DrishtiRules::default())
    }

    /// The Tajika aspects under stated readings of the source.
    ///
    /// The source's chapter and its Table X-3 place a pair less than a
    /// degree past differently, and 934 of the 29 166 aspecting pairs
    /// over the corpus's recorded years fall there. [`crate::SubDegree`]
    /// names
    /// the three readings; the default is the table's.
    ///
    /// ```no_run
    /// # use teistro::{Context, Document, DrishtiRules, SubDegree};
    /// # fn main() -> Result<(), teistro::Error> {
    /// # let sdk = Context::builder().build()?;
    /// # let annual: Document = todo!();
    /// let rules = DrishtiRules { sub_degree: SubDegree::Ishrafa };
    /// let pairs = sdk.chart().drishtis_with_rules(&annual, rules)?;
    /// let contested = pairs.iter().filter(|pair| pair.disputed()).count();
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// As [`ChartArea::drishtis`].
    pub fn drishtis_with_rules(
        self,
        annual: &Document,
        rules: DrishtiRules,
    ) -> Result<Vec<Between>, Error> {
        teistro_tajika::drishtis_with_rules(&Self::sky_of(annual)?, rules)
    }

    /// The **sixteen Tajika yogas** of an annual chart, for one matter.
    ///
    /// Fourteen of the sixteen are judgements about a **pair** — the
    /// *lagnesha*, the lord of the annual lagna, and the *karyesha*, the
    /// lord of the house the matter asked about belongs to — so this
    /// takes the house and not only the chart. "Is the marriage promised
    /// this year?" is `House::try_new(7)`; "what yogas does this year
    /// have?" is not a question these sixteen answer.
    ///
    /// A yoga a call cannot answer for is **listed** in
    /// [`YearYogas::unanswered`], and [`YearYogas::why`] says what it
    /// needs, because "Kuttha did not hold" and "this call cannot tell
    /// you about Kuttha" are different statements; [`YearYogas::holds`]
    /// answers `None` for it rather than `false`.
    ///
    /// Retrograde and combustion — which Rudda, Duhphali-kuttha,
    /// Tambira and Durapha read — are taken from the founded chart's own
    /// graha states, under this context's combustion table, so every one
    /// of the sixteen is answered and the list comes back empty.
    ///
    /// It needs **no ephemeris**: the chart is already founded.
    ///
    /// ```no_run
    /// # use teistro::{Context, Document, House};
    /// # fn main() -> Result<(), teistro::Error> {
    /// # let sdk = Context::builder().build()?;
    /// # let annual: Document = todo!();
    /// let marriage = sdk.chart().tajika_yogas(&annual, House::try_new(7)?)?;
    /// let promised = marriage.holds(teistro::YearYoga::Ithasala);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// An annual chart that does not place one of the seven, or whose
    /// lagna is not a number.
    pub fn tajika_yogas(self, annual: &Document, house: House) -> Result<YearYogas, Error> {
        self.tajika_yogas_with_rules(annual, house, YogaRules::default())
    }

    /// The sixteen Tajika yogas for one matter, under stated readings.
    ///
    /// [`YogaRules`] carries every reading the sixteen leave open: the
    /// aspects' own ([`crate::SubDegree`]), which each pair yoga inherits,
    /// and the two strength floors (crux C116).
    ///
    /// # Errors
    ///
    /// As [`ChartArea::tajika_yogas`]; and floors that would let a planet
    /// be strong and weak at once, named `strong_from`.
    pub fn tajika_yogas_with_rules(
        self,
        annual: &Document,
        house: House,
        rules: YogaRules,
    ) -> Result<YearYogas, Error> {
        teistro_tajika::year_yogas_with_states(
            annual.foundation.lagna_deg,
            house,
            &Self::sky_of(annual)?,
            &self.annual_states(annual)?,
            rules,
        )
    }

    /// The sixteen Tajika yogas for **several matters** of one annual
    /// chart, in the order the houses are given.
    ///
    /// What the matters share — the chart's sky, its retrograde and
    /// combust planets, the rules' check and the seven strengths — is
    /// read once for the chart and not once a matter, so asking for all
    /// twelve costs far less than twelve calls to
    /// [`ChartArea::tajika_yogas_with_rules`], and answers exactly as they
    /// would.
    ///
    /// ```no_run
    /// # use teistro::{Context, Document, House, YogaRules};
    /// # fn main() -> Result<(), teistro::Error> {
    /// # let sdk = Context::builder().build()?;
    /// # let annual: Document = todo!();
    /// let every = sdk.chart().tajika_yogas_many(&annual, &House::ALL, YogaRules::default())?;
    /// for matter in &every {
    ///     println!("house {}: {} held", matter.house.get(), matter.held.len());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// As [`ChartArea::tajika_yogas_with_rules`], refused before any
    /// matter is judged.
    pub fn tajika_yogas_many(
        self,
        annual: &Document,
        houses: &[House],
        rules: YogaRules,
    ) -> Result<Vec<YearYogas>, Error> {
        teistro_tajika::year_yogas_many(
            annual.foundation.lagna_deg,
            houses,
            &Self::sky_of(annual)?,
            Some(&self.annual_states(annual)?),
            rules,
        )
    }

    /// The **sahams** asked for, in a founded chart, under the source's
    /// readings (`03-design/tajika-sahams.md`).
    ///
    /// Any chart: the source reads an annual chart's sahams beside the
    /// birth chart's, and both are a [`Document`]. Day or night is the
    /// chart's own; a house's point is Sripati's mid-point, built from the
    /// chart's lagna and midheaven as the source builds it, whatever chalit
    /// the profile gives the chart. Each saham is computed once however
    /// many read it.
    ///
    /// It needs **no ephemeris**: the chart is already founded.
    ///
    /// ```no_run
    /// # use teistro::{Context, Document, Saham};
    /// # fn main() -> Result<(), teistro::Error> {
    /// # let sdk = Context::builder().build()?;
    /// # let annual: Document = todo!();
    /// let read = sdk.chart().sahams(&annual, &[Saham::Punya, Saham::Vivaha])?;
    /// for asked in &read.points {
    ///     println!("{:?}: {:?}, house {}", asked.saham, asked.point.sign, asked.point.house.get());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// A chart that does not place one of the seven, or whose lagna or a
    /// bhava is not a number.
    pub fn sahams(self, chart: &Document, which: &[Saham]) -> Result<SahamReading, Error> {
        self.sahams_with_rules(chart, which, SahamRules::default())
    }

    /// The sahams asked for, under stated readings: when a sign is added,
    /// where a house stands, and which Roga is meant.
    ///
    /// # Errors
    ///
    /// As [`ChartArea::sahams`].
    pub fn sahams_with_rules(
        self,
        chart: &Document,
        which: &[Saham],
        rules: SahamRules,
    ) -> Result<SahamReading, Error> {
        teistro_tajika::sahams(&self.saham_sky_of(chart)?, which, rules)
    }

    /// Where a saham of the caller's own falls: another authority's, or
    /// one the source does not give, over the same factors and rules.
    ///
    /// # Errors
    ///
    /// As [`ChartArea::sahams`]; and a node or an unreadable fixed degree
    /// as a factor, named by the factor.
    pub fn saham_point(
        self,
        chart: &Document,
        formula: &SahamFormula,
        rules: SahamRules,
    ) -> Result<SahamPlace, Error> {
        teistro_tajika::saham_point(&self.saham_sky_of(chart)?, formula, rules)
    }

    /// What a saham is read from, off a founded chart: its midheaven
    /// ([`ChartArea::angles`]), and its own chalit for a caller who asks for
    /// that.
    fn saham_sky_of(self, chart: &Document) -> Result<SahamSky, Error> {
        let foundation = &chart.foundation;
        let angles = self.angles(chart)?;
        let sky = SahamSky::new(
            Self::sky_of(chart)?,
            foundation.lagna_deg,
            angles.midheaven_deg,
            foundation.day.part.is_daylight(),
        )
        .with_chalit(foundation.chalit.madhya);
        // The nodes only feed a saham's strength, which reads whether a
        // saham stands in their axis; a chart that does not place Rahu
        // leaves that fact unread rather than guessed.
        Ok(match foundation.graha(Graha::Rahu) {
            Some(rahu) => sky.with_rahu(rahu.longitude_deg),
            None => sky,
        })
    }

    /// A saham's **strength**, clause by clause: every clause of the
    /// source's strong and weak lists, named and evaluated, with the facts
    /// they were read from — and **no verdict**, since the source never
    /// scores a saham (`03-design/tajika-saham-strength.md`).
    ///
    /// `year_lord` is the annual chart's lord of the year, from
    /// [`ChartArea::varshesha`], or `None` for a birth chart, which has
    /// none. It needs **no ephemeris**.
    ///
    /// # Errors
    ///
    /// As [`ChartArea::sahams`]; a year lord outside the seven, named
    /// `year_lord`.
    pub fn saham_strength(
        self,
        chart: &Document,
        which: &[Saham],
        year_lord: Option<Graha>,
    ) -> Result<Vec<SahamStrength>, Error> {
        self.saham_strength_with_rules(chart, which, year_lord, SahamStrengthRules::default())
    }

    /// A saham's strength under the readings you name: the chapter's or
    /// the catalogue's natures, positional or natural friendship, the
    /// Vishwa floor, and the sahams' and the Harsha bala's own rules.
    ///
    /// # Errors
    ///
    /// As [`ChartArea::saham_strength`].
    pub fn saham_strength_with_rules(
        self,
        chart: &Document,
        which: &[Saham],
        year_lord: Option<Graha>,
        rules: SahamStrengthRules,
    ) -> Result<Vec<SahamStrength>, Error> {
        teistro_tajika::saham_strength(&self.saham_sky_of(chart)?, which, year_lord, rules)
    }

    /// Which of an annual chart's seven are **retrograde** and which
    /// **combust** — the two things its longitudes cannot say.
    ///
    /// Read from the founded chart's own graha states, under this
    /// context's combustion table (`state.combustion_orbs`), so a
    /// consumer who wants another table changes the setting and not
    /// this call.
    ///
    /// # Errors
    ///
    /// A combustion table the SDK does not ship, named by its setting.
    pub fn annual_states(self, annual: &Document) -> Result<AnnualStates, Error> {
        let mut states = AnnualStates::default();
        for one in state(&annual.foundation, self.context.settings())? {
            // The nodes are not among the seven the sixteen read.
            if !SEVEN.contains(&one.graha) {
                continue;
            }
            if one.motion.retrograde {
                states.retrograde.push(one.graha);
            }
            if one.combustion.is_combust() {
                states.combust.push(one.graha);
            }
        }
        states.check()
    }

    /// How a planet of an annual chart stands to the source's
    /// **afflictions** — retrograde, combust, debilitated, in the 6th,
    /// 8th or 12th, or under malefic influence — clause by clause.
    ///
    /// Rudda holds where either of a pair is afflicted at all, and
    /// Durapha reads three of the five; carrying the clauses is what lets
    /// a reader asking *why* be answered.
    ///
    /// # Errors
    ///
    /// A body outside the seven, named `graha`; as
    /// [`ChartArea::annual_states`].
    pub fn affliction(self, annual: &Document, graha: Graha) -> Result<Affliction, Error> {
        teistro_tajika::affliction(
            graha,
            annual.foundation.lagna_deg,
            &Self::sky_of(annual)?,
            &self.annual_states(annual)?,
        )
    }

    /// Whether a planet of an annual chart is **unqualified**, clause by
    /// clause, in the source's own sense of the word.
    ///
    /// "Neither exalted nor debilitated, nor aspected/associated, nor in
    /// its own Hudda, Drekkana or Navamsha." Khallasara and
    /// Gairi-Kamboola both turn on it, and it is strict: Tajika counts
    /// eight of the twelve sign relations as an aspect, so a planet
    /// nothing aspects needs the other six inside the four neutral
    /// houses at once.
    ///
    /// Every clause is carried rather than collapsed into the verdict,
    /// so a reader asking why a yoga did not hold gets the clause.
    ///
    /// It needs **no ephemeris**: the chart is already founded.
    ///
    /// # Errors
    ///
    /// A body outside the seven, named `graha`; an annual chart that
    /// does not place it.
    pub fn qualification(self, annual: &Document, graha: Graha) -> Result<Qualification, Error> {
        teistro_tajika::qualification(graha, &Self::sky_of(annual)?)
    }

    /// How a planet of an annual chart stands to **strong** and **weak**,
    /// clause by clause.
    ///
    /// The source treats its three as alternatives — "exalted, in its own
    /// house or otherwise strong" — so strength is a disjunction and
    /// weakness is its denial, with no third state between them.
    ///
    /// The floor of the third clause is the one number the source does
    /// **not** give for the yogas: it floors strength at five Vishwa
    /// units for the office-bearers when choosing the year lord, and
    /// says nothing here. That figure is the default and
    /// [`Chart::strength_with_rules`] moves it; crux C116 records why,
    /// and `03-design/muntha-measured.md` §11 measures what moving it
    /// costs.
    ///
    /// It needs **no ephemeris**: the chart is already founded.
    ///
    /// # Errors
    ///
    /// A body outside the seven, named `graha`; an annual chart that
    /// does not place it.
    pub fn strength(self, annual: &Document, graha: Graha) -> Result<Strength, Error> {
        teistro_tajika::strength(graha, &Self::sky_of(annual)?)
    }

    /// [`Chart::strength`] with the floor between strong and weak chosen.
    ///
    /// # Errors
    ///
    /// A body outside the seven, named `graha`; an annual chart that
    /// does not place it.
    pub fn strength_with_rules(
        self,
        annual: &Document,
        graha: Graha,
        rules: YogaRules,
    ) -> Result<Strength, Error> {
        teistro_tajika::strength_with_rules(graha, &Self::sky_of(annual)?, rules)
    }

    /// How a planet of an annual chart stands to what **Kuttha** asks of
    /// each lord — powerful, in a kendra or a panaphara, under a benefic's
    /// aspect and no malefic's — clause by clause.
    ///
    /// Kuttha holds where both lords are favoured. A reader asking why it
    /// did not hold for a matter asks this of the two lords, and gets the
    /// clause that stopped it (crux C117).
    ///
    /// It needs **no ephemeris** and no retrograde or combustion: the
    /// chart is already founded, and Kuttha reads neither.
    ///
    /// # Errors
    ///
    /// A body outside the seven, named `graha`; an annual chart that
    /// does not place it.
    pub fn favour(self, annual: &Document, graha: Graha) -> Result<Favour, Error> {
        self.favour_with_rules(annual, graha, YogaRules::default())
    }

    /// [`Chart::favour`] with the strength floors and the Moon's reading
    /// chosen.
    ///
    /// # Errors
    ///
    /// As [`Chart::favour`]; and floors that would let a planet be both,
    /// named `strong_from`.
    pub fn favour_with_rules(
        self,
        annual: &Document,
        graha: Graha,
        rules: YogaRules,
    ) -> Result<Favour, Error> {
        teistro_tajika::favour(
            graha,
            annual.foundation.lagna_deg,
            &Self::sky_of(annual)?,
            rules,
        )
    }

    /// One year's **annual dasha**: the Mudda, the Varsha Yogini or the
    /// Patyayini, from a birth chart and an annual chart **you founded**
    /// (`03-design/annual-dashas.md`).
    ///
    /// The year opens at the annual chart's instant, which is the return.
    /// Under the default clock it closes on the next return, each of its
    /// 360 units the Sun's motion through one degree. The answer carries
    /// the ring the year runs round, the year itself, and every period to
    /// the rules' depth; the readings the sources differ on are
    /// [`AnnualDashaRules`], each named on the answer.
    ///
    /// Asking for more than one system of the same year?
    /// [`ChartArea::annual_dashas`] reads the Sun over the year once for
    /// all of them.
    ///
    /// ```no_run
    /// # use teistro::{AnnualDashaRules, ChartRequest, Context, Document, Ephemeris};
    /// # use teistro::catalogue::DashaSystem;
    /// # use teistro::tajika::Reading;
    /// # fn main() -> Result<(), teistro::Error> {
    /// # let sdk = Context::builder().ephemeris([Ephemeris::Builtin]).build()?;
    /// # let natal: Document = todo!();
    /// # let request: ChartRequest = todo!();
    /// let annual = sdk.chart().annual(&natal, Reading::Sidereal, 40, &request)?;
    /// let mudda = sdk.chart().annual_dasha(
    ///     &natal,
    ///     &annual.value,
    ///     40,
    ///     DashaSystem::Mudda,
    ///     AnnualDashaRules::default(),
    /// )?;
    /// let first = &mudda.periods[0];
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// As [`ChartArea::annual_dashas`].
    pub fn annual_dasha(
        self,
        natal: &Document,
        annual: &Document,
        completed_years: u16,
        system: DashaSystem,
        rules: AnnualDashaRules,
    ) -> Result<AnnualDasha, Error> {
        self.annual_dashas(natal, annual, completed_years, &[system], rules)?
            .pop()
            .ok_or_else(|| Error::internal("one system asked is one answered"))
    }

    /// Several annual dashas of **one** year, in the order asked, over one
    /// clock: the Sun's crossings of the year are found once however many
    /// systems read them.
    ///
    /// ```no_run
    /// # use teistro::{AnnualDashaRules, Context, Document, Ephemeris};
    /// # use teistro::tajika::ANNUAL_DASHAS;
    /// # fn main() -> Result<(), teistro::Error> {
    /// # let sdk = Context::builder().ephemeris([Ephemeris::Builtin]).build()?;
    /// # let (natal, annual): (Document, Document) = todo!();
    /// let all_three =
    ///     sdk.chart().annual_dashas(&natal, &annual, 40, &ANNUAL_DASHAS, AnnualDashaRules::default())?;
    /// assert_eq!(all_three.len(), 3);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// A system other than the three, named `system` with the three in the
    /// hint, before anything is searched; a `completed_years` past two
    /// hundred, named `completed_years`; no ephemeris, for a clock that
    /// reads the Sun or a balance measured by time; what the rules' clock
    /// refuses; a chart that does not place one of the seven.
    pub fn annual_dashas(
        self,
        natal: &Document,
        annual: &Document,
        completed_years: u16,
        systems: &[DashaSystem],
        rules: AnnualDashaRules,
    ) -> Result<Vec<AnnualDasha>, Error> {
        if completed_years > teistro_tajika::MOST_YEARS {
            return Err(Error::invalid_arg(format!(
                "an annual dasha is read for 0 to {} completed years, not {completed_years}",
                teistro_tajika::MOST_YEARS
            ))
            .with_field("completed_years"));
        }
        if let Some(stranger) = systems
            .iter()
            .find(|system| !teistro_tajika::ANNUAL_DASHAS.contains(system))
        {
            let built: Vec<&str> = teistro_tajika::ANNUAL_DASHAS
                .iter()
                .map(|one| one.key())
                .collect();
            return Err(
                Error::invalid_arg(format!("{} is not an annual dasha", stranger.key()))
                    .with_field("system")
                    .with_hint(format!("the annual dashas are {}", built.join(", "))),
            );
        }
        if systems.is_empty() {
            return Ok(Vec::new());
        }
        let year = &annual.foundation;
        let knots = match rules.clock.divisions() {
            Some(divisions) => {
                let sun = year
                    .graha(Graha::Sun)
                    .ok_or_else(|| Error::internal("a founded chart places the Sun"))?
                    .longitude_deg;
                Some(self.in_chart_sky(year, |longitudes, zodiac| {
                    teistro_tajika::sun_knots(longitudes, zodiac, year.instant, sun, divisions)
                })?)
            }
            None => None,
        };
        let clock = teistro_tajika::year_clock(rules.clock, year.instant, knots)?;
        systems
            .iter()
            .map(|&system| {
                let (ring, seed) =
                    self.annual_ring(natal, annual, completed_years, system, rules)?;
                let dasha = YearDasha::new(ring, clock.clone(), rules.birth_period)?;
                Ok(AnnualDasha::of(
                    &dasha,
                    system,
                    completed_years,
                    rules,
                    seed,
                ))
            })
            .collect()
    }

    /// The ring one annual dasha runs round in a year, and the birth
    /// nakshatra it is seeded from when it is a nakshatra year.
    fn annual_ring(
        self,
        natal: &Document,
        annual: &Document,
        completed_years: u16,
        system: DashaSystem,
        rules: AnnualDashaRules,
    ) -> Result<(YearRing, Option<Nakshatra>), Error> {
        let moon_of = |foundation: &ChartFoundation| {
            foundation
                .graha(Graha::Moon)
                .ok_or_else(|| Error::internal("a founded chart places the Moon"))
                .and_then(|moon| Ok(Nas::try_from_degrees(moon.longitude_deg)?))
        };
        if system == DashaSystem::Patyayini {
            let sky = Self::sky_of(annual)?;
            let strengths = teistro_tajika::panchavargiya(&sky)?;
            let ring =
                teistro_tajika::patyayini_ring(&sky, annual.foundation.lagna_deg, &strengths)?;
            return Ok((ring, None));
        }
        let birth_moon = moon_of(&natal.foundation)?;
        let remaining = match rules.balance {
            MuddaBalance::Whole => None,
            MuddaBalance::NatalMoon | MuddaBalance::EntryMoon => {
                let foundation = if rules.balance == MuddaBalance::NatalMoon {
                    &natal.foundation
                } else {
                    &annual.foundation
                };
                Some(match rules.measure() {
                    Balance::Temporal => teistro_tajika::remaining_by_time(
                        foundation.instant,
                        self.moon_span(foundation, Wheel::Nakshatras)?,
                    )?,
                    _ => teistro_tajika::remaining_by_arc(moon_of(foundation)?),
                })
            }
        };
        let ring = teistro_tajika::nakshatra_ring(system, birth_moon, completed_years, remaining)?;
        Ok((ring, Some(birth_moon.nakshatra())))
    }

    /// Where the seven stand in a founded chart, which both the strengths
    /// and the aspects read.
    fn sky_of(annual: &Document) -> Result<AnnualSky, Error> {
        let at = |graha: Graha| Self::longitude_of(annual, graha);
        Ok(AnnualSky {
            sun_deg: at(Graha::Sun)?,
            moon_deg: at(Graha::Moon)?,
            mars_deg: at(Graha::Mars)?,
            mercury_deg: at(Graha::Mercury)?,
            jupiter_deg: at(Graha::Jupiter)?,
            venus_deg: at(Graha::Venus)?,
            saturn_deg: at(Graha::Saturn)?,
        })
    }

    /// The Sun where it stood at birth, in both zodiacs, as a return needs
    /// it.
    ///
    /// Both are read off the founded chart rather than one being rebuilt
    /// from the other through the ayanamsha: the chart already answered
    /// that question and a second answer to it is a second thing to get
    /// wrong.
    fn natal_of(foundation: &ChartFoundation) -> Result<Natal, Error> {
        let sun = foundation
            .graha(Graha::Sun)
            .ok_or_else(|| Error::internal("a founded chart places the Sun"))?;
        Ok(Natal {
            instant: foundation.instant,
            sidereal_sun_deg: sun.longitude_deg,
            tropical_sun_deg: sun.tropical_deg,
        })
    }
}

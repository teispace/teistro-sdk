package com.teispace.teistro;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.Collections;
import java.util.List;
import java.util.Optional;

import com.teispace.teistro.blob.Charts;

/**
 * The Western readings of a chart: each reads its batch's sections once,
 * kept by {@link ChartBatch#cached}, and answers this chart's entry, or
 * empty when its request did not ask for the reading.
 */
final class WesternReads {
    private WesternReads() {}

    /** {@code antiscia.cusp_system} where no cusps were asked. */
    private static final int NO_HOUSE_SYSTEM = 0xFFFF;

    /**
     * The seven planets' essential dignities and the chart's sect, with
     * everything that made them; empty unless {@code dignities} asked for
     * them ({@code 03-design/essential-dignities.md}).
     */
    static Optional<Dignities> dignities(Chart chart) {
        return Reads.at(chart.batch().cached("western.dignities", () -> dignities(chart.batch().decoded())),
                chart.index());
    }

    /**
     * Both halves of Lilly's table, the essential dignities and the
     * accidental fortitudes, with everything that made them; empty unless
     * {@code fortitudes} asked for them.
     */
    static Optional<Fortitudes> fortitudes(Chart chart) {
        ChartBatch batch = chart.batch();
        return Reads.at(batch.cached("western.fortitudes",
                () -> fortitudes(batch.decoded(), batch.cached("western.dignities", () -> dignities(batch.decoded())))),
                chart.index());
    }

    /**
     * Valens's fourteen lots, with the chart's sect and the rules they were
     * read under; empty unless {@code lots} asked for them.
     */
    static Optional<Lots> lots(Chart chart) {
        return Reads.at(chart.batch().cached("western.lots", () -> lots(chart.batch().decoded())), chart.index());
    }

    /**
     * Lilly's considerations before judgement, read from the chart's
     * fortitudes and its planetary hour; empty unless {@code considerations}
     * asked for them.
     */
    static Optional<Considerations> considerations(Chart chart) {
        return Reads.at(chart.batch().cached("western.considerations",
                () -> considerations(chart.batch().decoded())), chart.index());
    }

    /**
     * Whether a horary matter is brought to pass, weighed on the chart's
     * fortitudes and searched on the ephemeris; empty unless
     * {@code perfection} asked.
     */
    static Optional<Matter> perfection(Chart chart) {
        return Reads.at(chart.batch().cached("western.perfection", () -> perfections(chart.batch().decoded())),
                chart.index());
    }

    /**
     * The birth read through its progressions: the progressed chart and the
     * direction at {@code at}, the contacts in the window; empty unless
     * {@code progressions} asked.
     */
    static Optional<Progressions> progressions(Chart chart) {
        return Reads.at(chart.batch().cached("western.progressions", () -> progressions(chart.batch().decoded())),
                chart.index());
    }

    /**
     * The Western aspect table, closest first: Leo's nine under his orbs by
     * default (C240); empty unless {@code western_aspects} asked.
     */
    static Optional<List<WesternAspectRow>> westernAspects(Chart chart) {
        return Reads.at(chart.batch().cached("western.aspects", () -> westernAspects(chart.batch().decoded())),
                chart.index());
    }

    /**
     * The chart's distances from the equator, its planets' and its angles';
     * empty unless {@code parallels} asked.
     */
    static Optional<Declinations> declinations(Chart chart) {
        return Reads.at(chart.batch().cached("western.declinations", () -> declinations(chart.batch().decoded())),
                chart.index());
    }

    /**
     * The parallels among the chart's planets, closest first: each pair the
     * same distance from the equator within the orb (Leo's 1° by default),
     * on either side of it (C243); empty unless {@code parallels} asked.
     */
    static Optional<List<ParallelRow>> parallels(Chart chart) {
        return Reads.at(chart.batch().cached("western.parallels", () -> parallels(chart.batch().decoded())),
                chart.index());
    }

    /**
     * The chart's antiscia: each planet's reflection about the solstices and
     * the equinoxes, and the pairs standing in one within the orbs, Lilly's
     * moieties by default (C244); empty unless {@code antiscia} asked.
     */
    static Optional<Antiscia> antiscia(Chart chart) {
        return Reads.at(chart.batch().cached("western.antiscia", () -> antiscia(chart.batch().decoded())),
                chart.index());
    }

    /**
     * The chart's harmonic chart: each planet, the ascendant and the
     * midheaven at its longitude multiplied, in its equal house from the
     * harmonic ascendant (C254), and every pair meeting within the orb, 12°
     * by default (C252), closest first; empty unless {@code harmonic} asked.
     */
    static Optional<HarmonicChart> harmonic(Chart chart) {
        return Reads.at(chart.batch().cached("western.harmonic", () -> harmonics(chart.batch().decoded())),
                chart.index());
    }

    /**
     * The chart's Western houses: the cusps of the asked division, else the
     * profile's for the module, else Placidus (C249), and each planet's
     * house and whether Leo reads it with the ascendant (C250); empty unless
     * {@code western_houses} asked.
     */
    static Optional<WesternHouses> westernHouses(Chart chart) {
        return Reads.at(chart.batch().cached("western.houses", () -> westernHouses(chart.batch().decoded())),
                chart.index());
    }

    /**
     * The chart's equal distances, closest first: each planet within the orb
     * of the axis through two others' midpoint, 0.5° by default (C245), on
     * the shorter arc's midpoint or opposite it (C246); empty unless
     * {@code midpoints} asked.
     */
    static Optional<List<MidpointRow>> midpoints(Chart chart) {
        return Reads.at(chart.batch().cached("western.midpoints", () -> midpoints(chart.batch().decoded())),
                chart.index());
    }

    /**
     * The Western aspects between this chart and the partner's, closest
     * first; empty unless {@code synastry} asked.
     */
    static Optional<List<SynastryRow>> synastry(Chart chart) {
        return Reads.at(chart.batch().cached("western.synastry", () -> synastries(chart.batch().decoded())),
                chart.index());
    }

    /**
     * The parallels between this chart and the partner's, closest first;
     * empty unless {@code synastry} asked for {@code parallels}.
     */
    static Optional<List<SynastryParallelRow>> synastryParallels(Chart chart) {
        return Reads.at(chart.batch().cached("western.synastry.parallels",
                () -> synastryParallels(chart.batch().decoded())), chart.index());
    }

    /**
     * The antiscia between this chart and the partner's, closest first: a
     * planet of this chart whose tropical longitude and one of the
     * partner's sum to 180°, or 0° for the contrantiscion, within the orb
     * read at the conjunction, Lilly's moieties by default (C244); empty
     * unless {@code synastry} asked for {@code antiscia}.
     */
    static Optional<List<AntiscionRow>> synastryAntiscia(Chart chart) {
        return Reads.at(chart.batch().cached("western.synastry.antiscia",
                () -> synastryAntiscia(chart.batch().decoded())), chart.index());
    }

    /**
     * The equal distances between this chart and the partner's, closest
     * first: a planet of one chart within the orb of the axis through two
     * of the other's, on the shorter arc's midpoint or opposite it, 0.5° by
     * default (C245, C246); empty unless {@code synastry} asked for
     * {@code midpoints}.
     */
    static Optional<List<SynastryMidpointRow>> synastryMidpoints(Chart chart) {
        return Reads.at(chart.batch().cached("western.synastry.midpoints",
                () -> synastryMidpoints(chart.batch().decoded())), chart.index());
    }

    /**
     * The composite of this chart and the partner's: each planet at the near
     * midpoint of its two places, moving at the mean of its two speeds, the
     * midheaven at the near midpoint of the two, and the lagna at theirs,
     * turned by 180° when it stood before the midheaven (C247), in the
     * synastry's zodiac; empty unless {@code synastry} asked for
     * {@code composite}.
     */
    static Optional<Composite> synastryComposite(Chart chart) {
        return Reads.at(chart.batch().cached("western.synastry.composite",
                () -> synastryComposites(chart.batch().decoded())), chart.index());
    }

    /**
     * The Davison birth of this chart and the partner: the mean of the two
     * instants, of the two latitudes and altitudes, of the two longitudes
     * the shorter way round, and of the two clocks, this chart's read on the
     * request's (C248); empty unless {@code synastry} asked for
     * {@code davison}.
     */
    static Optional<DavisonBirth> synastryDavison(Chart chart) {
        return Reads.at(chart.batch().cached("western.synastry.davison",
                () -> synastryDavisons(chart.batch().decoded())), chart.index());
    }

    // The batch readers: each decodes every chart's entry once.

    /** A natal point from a {@code to_lagna} and a {@code to_graha} column's cells. */
    private static NatalPoint pointAt(int toLagna, int toGraha) {
        return toLagna != 0 ? new NatalPoint("LAGNA", null) : new NatalPoint("GRAHA", Graha.of(toGraha));
    }

    /**
     * Every chart's essential dignities; empty when none were asked for.
     * {@code dignities} holds a row a chart, {@code dignity_planets} seven a
     * chart, in the Chaldean order, and {@code dignity_receptions} each
     * chart's receptions, ragged by {@code dignities.reception_count}.
     */
    private static List<Dignities> dignities(Charts decoded) {
        Charts.Dignities c = decoded.dignities();
        Charts.DignityPlanets p = decoded.dignityPlanets();
        Charts.DignityReceptions r = decoded.dignityReceptions();
        int charts = Reads.charts(decoded);
        if (c.length() == 0) {
            return List.of();
        }
        if (c.length() != charts || p.length() != 7 * charts) {
            throw Reads.internal("dignities has " + c.length() + " rows and dignity_planets " + p.length() + " for "
                    + charts + " charts; they are one and seven a chart");
        }
        long receptions = Reads.sum(c.length(), c::receptionCount);
        if (receptions != r.length()) {
            throw Reads.internal("dignity_receptions has " + r.length() + " rows and the charts count " + receptions);
        }
        int[] starts = Reads.starts(c.length(), c::receptionCount);
        List<Dignities> out = new ArrayList<>(charts);
        for (int chart = 0; chart < charts; chart += 1) {
            int k = chart;
            out.add(new Dignities(
                    Sect.of(c.sect(k)),
                    SectRule.of(c.sectRule(k)),
                    new AppliedDignityRules(Terms.of(c.terms(k)), Triplicities.of(c.triplicities(k))),
                    new DignityScores(c.scoreHouse(k), c.scoreExaltation(k), c.scoreTriplicity(k), c.scoreTerm(k),
                            c.scoreFace(k), c.scoreDetriment(k), c.scoreFall(k), c.scorePeregrine(k)),
                    Reads.rows(7 * k, 7 * k + 7, row -> {
                        EssentialDignity dignity = new EssentialDignity(p.house(row) == 1, p.exaltation(row) == 1,
                                p.triplicity(row) == 1, p.term(row) == 1, p.face(row) == 1, p.detriment(row) == 1,
                                p.fall(row) == 1);
                        return new PlanetDignity(Graha.of(p.planet(row)), p.longitude(row), dignity,
                                dignity.held().isEmpty(), p.score(row), p.reception(row));
                    }),
                    Reads.rows(starts[k], starts[k + 1], row -> new Reception(
                            List.of(Graha.of(r.first(row)), Graha.of(r.second(row))),
                            new EssentialDignity(r.firstInHouse(row) == 1, r.firstInExaltation(row) == 1,
                                    r.firstInTriplicity(row) == 1, r.firstInTerm(row) == 1, r.firstInFace(row) == 1,
                                    r.firstInDetriment(row) == 1, r.firstInFall(row) == 1),
                            new EssentialDignity(r.secondInHouse(row) == 1, r.secondInExaltation(row) == 1,
                                    r.secondInTriplicity(row) == 1, r.secondInTerm(row) == 1,
                                    r.secondInFace(row) == 1, r.secondInDetriment(row) == 1,
                                    r.secondInFall(row) == 1)))));
        }
        return Collections.unmodifiableList(out);
    }

    /**
     * Every chart's accidental fortitudes; empty when none were asked for.
     * {@code fortitudes} holds a row a chart, {@code fortitude_houses}
     * twelve a chart, {@code fortitude_planets} seven in the Chaldean order,
     * and {@code fortitude_accidents} each planet's lines, ragged by its
     * {@code accident_count}. The essential half is the batch's dignities.
     */
    private static List<Fortitudes> fortitudes(Charts decoded, List<Dignities> essential) {
        Charts.Fortitudes c = decoded.fortitudes();
        Charts.FortitudeHouses h = decoded.fortitudeHouses();
        Charts.FortitudePlanets p = decoded.fortitudePlanets();
        Charts.FortitudeAccidents a = decoded.fortitudeAccidents();
        int charts = Reads.charts(decoded);
        if (c.length() == 0) {
            return List.of();
        }
        if (c.length() != charts || h.length() != 12 * charts || p.length() != 7 * charts) {
            throw Reads.internal("fortitudes has " + c.length() + " rows, fortitude_houses " + h.length()
                    + " and fortitude_planets " + p.length() + " for " + charts
                    + " charts; they are one, twelve and seven a chart");
        }
        long accidents = Reads.sum(p.length(), row -> p.accidentCount(row));
        if (accidents != a.length()) {
            throw Reads.internal("fortitude_accidents has " + a.length() + " rows and the planets count " + accidents);
        }
        if (essential.size() != charts) {
            throw Reads.internal("fortitudes has " + charts + " charts and dignities " + essential.size());
        }
        int[] starts = Reads.starts(p.length(), row -> p.accidentCount(row));
        List<Fortitudes> out = new ArrayList<>(charts);
        for (int chart = 0; chart < charts; chart += 1) {
            int k = chart;
            Dignities own = essential.get(k);
            List<PlanetAccidents> planets = Reads.rows(0, 7, n -> {
                int row = 7 * k + n;
                PlanetDignity dignity = own.planets().get(n);
                return new PlanetAccidents(
                        Graha.of(p.planet(row)),
                        p.house(row),
                        Reads.rows(starts[row], starts[row + 1],
                                at -> new AccidentLine(Accident.of(a.accident(at)), a.points(at))),
                        p.fortitude(row),
                        p.debility(row),
                        dignity.score() + dignity.reception() + p.fortitude(row) - p.debility(row));
            });
            List<Graha> seven = planets.stream().map(PlanetAccidents::planet).toList();
            Almutens almutens = new Almutens(
                    new AlmutenRules(PlaceReading.of(c.almutenPlace(k)), FortuneRule.of(c.almutenFortune(k))),
                    c.fortune(k),
                    Almuten.of(seven, planets.stream().map(PlanetAccidents::net).toList()),
                    Almuten.of(seven, Reads.rows(7 * k, 7 * k + 7, p::places)),
                    Reads.rows(12 * k, 12 * k + 12, row -> Almuten.of(seven, List.of(
                            h.almutenSaturn(row), h.almutenJupiter(row), h.almutenMars(row), h.almutenSun(row),
                            h.almutenVenus(row), h.almutenMercury(row), h.almutenMoon(row)))));
            out.add(new Fortitudes(
                    own,
                    new AccidentalSky(
                            HouseSystem.of(c.houses(k)),
                            Reads.rows(12 * k, 12 * k + 12, h::cusp),
                            c.ascendant(k),
                            c.midheaven(k),
                            Reads.rows(7 * k, 7 * k + 7, p::speed),
                            c.northNode(k),
                            c.regulus(k),
                            c.spica(k),
                            c.algol(k)),
                    new AccidentalRules(
                            c.combustionOrb(k),
                            c.combustionInSign(k) == 1,
                            c.beamsOrb(k),
                            c.cazimiOrb(k),
                            c.cuspOrb(k),
                            c.starOrb(k),
                            Partile.of(c.partile(k)),
                            c.partileOrb(k),
                            Siege.of(c.siege(k)),
                            c.siegeSpan(k),
                            Reads.rows(7 * k, 7 * k + 7, p::meanMotion)),
                    new AccidentalScores(
                            Reads.rows(12 * k, 12 * k + 12, h::score),
                            c.scoreDirect(k), c.scoreRetrograde(k), c.scoreSwift(k), c.scoreSlow(k),
                            c.scoreSuperiorOriental(k), c.scoreSuperiorOccidental(k), c.scoreInferiorOriental(k),
                            c.scoreInferiorOccidental(k), c.scoreIncreasing(k), c.scoreDecreasing(k),
                            c.scoreFreeFromCombustion(k), c.scoreCazimi(k), c.scoreCombust(k), c.scoreUnderBeams(k),
                            c.scoreConjunctBenefic(k), c.scoreConjunctNorthNode(k), c.scoreTrineBenefic(k),
                            c.scoreSextileBenefic(k), c.scoreConjunctMalefic(k), c.scoreConjunctSouthNode(k),
                            c.scoreOpposedMalefic(k), c.scoreSquareMalefic(k), c.scoreBesieged(k), c.scoreRegulus(k),
                            c.scoreSpica(k), c.scoreAlgol(k)),
                    planets,
                    almutens));
        }
        return Collections.unmodifiableList(out);
    }

    /**
     * Every chart's lots; empty when none were asked for. {@code lots} holds
     * a row a chart and {@code lot_places} the catalogue's lots for each
     * chart, in its order.
     */
    private static List<Lots> lots(Charts decoded) {
        Charts.Lots c = decoded.lots();
        Charts.LotPlaces p = decoded.lotPlaces();
        int charts = Reads.charts(decoded);
        if (c.length() == 0) {
            return List.of();
        }
        int per = Lot.values().length;
        if (c.length() != charts || p.length() != per * charts) {
            throw Reads.internal("lots has " + c.length() + " rows and lot_places " + p.length() + " for " + charts
                    + " charts; they are one and " + per + " a chart");
        }
        List<Lots> out = new ArrayList<>(charts);
        for (int chart = 0; chart < charts; chart += 1) {
            out.add(new Lots(
                    Sect.of(c.sect(chart)),
                    new LotRules(SectRule.of(c.sectRule(chart)), FortuneRule.of(c.fortune(chart))),
                    c.fortuneReversed(chart) == 1,
                    Reads.rows(per * chart, per * chart + per, row -> new PlacedLot(
                            Lot.of(p.lot(row)),
                            new LotPlace(p.longitudeDeg(row), Rashi.of(p.sign(row)), Graha.of(p.lord(row)),
                                    p.house(row))))));
        }
        return Collections.unmodifiableList(out);
    }

    /**
     * Every chart's considerations; empty when none were asked for.
     * {@code considerations} holds a row a chart,
     * {@code consideration_perfections} two and {@code consideration_orbs}
     * seven.
     */
    private static List<Considerations> considerations(Charts decoded) {
        Charts.Considerations c = decoded.considerations();
        Charts.ConsiderationPerfections p = decoded.considerationPerfections();
        Charts.ConsiderationOrbs o = decoded.considerationOrbs();
        int charts = Reads.charts(decoded);
        if (c.length() == 0) {
            return List.of();
        }
        if (c.length() != charts || p.length() != 2 * charts || o.length() != 7 * charts) {
            throw Reads.internal("considerations has " + c.length() + " rows, consideration_perfections "
                    + p.length() + " and consideration_orbs " + o.length() + " for " + charts
                    + " charts; they are one, two and seven a chart");
        }
        List<RadicalGround> grounds = Arrays.asList(RadicalGround.values());
        List<Considerations> out = new ArrayList<>(charts);
        for (int k = 0; k < charts; k += 1) {
            out.add(new Considerations(
                    new Radicality(Graha.of(c.hourLord(k)), Graha.of(c.ascendantLord(k)),
                            Reads.members(c.radicalGrounds(k), grounds)),
                    new AscendantClause(Rashi.of(c.ascendantSign(k)), c.ascendantDegree(k), c.ascendantEarly(k) == 1,
                            c.ascendantLate(k) == 1, c.shortAscension(k) == 1),
                    new MoonClause(Rashi.of(c.moonSign(k)), c.moonDegree(k), c.moonLate(k) == 1,
                            c.moonLateSign(k) == 1, c.viaCombusta(k) == 1,
                            new MoonCourse(perfection(p, 2 * k), perfection(p, 2 * k + 1), c.daysInSign(k),
                                    c.eased(k) == 1)),
                    new SeventhClause(c.seventhCuspDeg(k), Graha.of(c.seventhLord(k)),
                            Reads.members(c.seventhInfortunes(k), Reads.SEVEN), c.seventhLordRetrograde(k) == 1,
                            c.seventhLordCombust(k) == 1, c.seventhLordInFall(k) == 1,
                            c.seventhLordInInfortuneTerm(k) == 1, c.seventhLordNet(k)),
                    c.saturnHouse(k),
                    c.saturnRetrograde(k) == 1,
                    c.ascendantLordCombust(k) == 1,
                    new ConsiderationRules(c.moonLateFromDeg(k), Reads.rows(7 * k, 7 * k + 7, o::orbDeg))));
        }
        return Collections.unmodifiableList(out);
    }

    private static Perfection perfection(Charts.ConsiderationPerfections p, int row) {
        if (p.present(row) != 1) {
            return null;
        }
        return new Perfection(Graha.of(p.planet(row)), PtolemaicAspect.of(p.aspect(row)), p.days(row), p.gapDeg(row));
    }

    /**
     * Every chart's perfection; empty when none was asked for.
     * {@code perfection} holds a row a chart, its impediments, translations
     * and collections ragged by that row's counts, and
     * {@code perfection_orbs} seven a chart.
     */
    private static List<Matter> perfections(Charts decoded) {
        Charts.Perfection m = decoded.perfection();
        Charts.PerfectionImpediments i = decoded.perfectionImpediments();
        Charts.PerfectionTranslations t = decoded.perfectionTranslations();
        Charts.PerfectionCollections c = decoded.perfectionCollections();
        Charts.PerfectionOrbs o = decoded.perfectionOrbs();
        int charts = Reads.charts(decoded);
        if (m.length() == 0) {
            return List.of();
        }
        if (m.length() != charts || o.length() != 7 * charts) {
            throw Reads.internal("perfection has " + m.length() + " rows and perfection_orbs " + o.length() + " for "
                    + charts + " charts; they are one and seven a chart");
        }
        checkRagged("perfection_impediments", i.length(), Reads.sum(m.length(), m::impedimentCount));
        checkRagged("perfection_translations", t.length(), Reads.sum(m.length(), m::translationCount));
        checkRagged("perfection_collections", c.length(), Reads.sum(m.length(), m::collectionCount));
        int[] impediments = Reads.starts(m.length(), m::impedimentCount);
        int[] translations = Reads.starts(m.length(), m::translationCount);
        int[] collections = Reads.starts(m.length(), m::collectionCount);
        List<Graha> infortunes = Reads.SEVEN;
        List<Way> ways = Arrays.asList(Way.values());
        List<Matter> out = new ArrayList<>(charts);
        for (int chart = 0; chart < charts; chart += 1) {
            int k = chart;
            Application application = m.applicationPresent(k) != 1 ? null : new Application(
                    PtolemaicAspect.of(m.applicationAspect(k)),
                    m.applicationDays(k),
                    Graha.of(m.applying(k)),
                    ApplicationKind.of(m.applicationKind(k)),
                    m.gapDeg(k),
                    m.withinMoieties(k) == 1);
            Separation separation = m.separationPresent(k) != 1 ? null
                    : new Separation(PtolemaicAspect.of(m.separationAspect(k)), m.separationPastDeg(k));
            double horizonRule = m.horizonRuleDays(k);
            out.add(new Matter(
                    Graha.of(m.querent(k)),
                    Graha.of(m.quesited(k)),
                    application,
                    separation,
                    Reads.rows(impediments[k], impediments[k + 1], at -> new Impediment(
                            ImpedimentKind.of(i.kind(at)),
                            Graha.of(i.significator(at)),
                            i.thirdPresent(at) == 1 ? Graha.of(i.third(at)) : null,
                            PtolemaicAspect.of(i.aspect(at)),
                            i.days(at))),
                    Reads.rows(translations[k], translations[k + 1], at -> new Translation(
                            Graha.of(t.translator(at)),
                            Graha.of(t.from(at)),
                            Graha.of(t.to(at)),
                            new Separation(PtolemaicAspect.of(t.separatingAspect(at)), t.separatingPastDeg(at)),
                            PtolemaicAspect.of(t.aspect(at)),
                            t.days(at),
                            EssentialDignity.ofBits(t.received(at)))),
                    Reads.rows(collections[k], collections[k + 1], at -> new Collection(
                            Graha.of(c.collector(at)),
                            new ContactAhead(PtolemaicAspect.of(c.fromQuerentAspect(at)), c.fromQuerentDays(at)),
                            new ContactAhead(PtolemaicAspect.of(c.fromQuesitedAspect(at)), c.fromQuesitedDays(at)),
                            EssentialDignity.ofBits(c.collectorInQuerent(at)),
                            EssentialDignity.ofBits(c.collectorInQuesited(at)),
                            EssentialDignity.ofBits(c.querentInCollector(at)),
                            EssentialDignity.ofBits(c.quesitedInCollector(at)))),
                    new Ways(
                            new SignificatorPlace(Graha.of(m.querent(k)), m.querentHouse(k),
                                    EssentialDignity.ofBits(m.querentDignity(k))),
                            new SignificatorPlace(Graha.of(m.quesited(k)), m.quesitedHouse(k),
                                    EssentialDignity.ofBits(m.quesitedDignity(k))),
                            m.mutualByHouse(k) == 1,
                            Reads.members(m.infortunesBetween(k), infortunes),
                            m.moonRelays(k) == 1,
                            m.quesitedInAscendant(k) == 1,
                            Reads.members(m.waysHeld(k), ways)),
                    m.horizonDays(k),
                    new PerfectionRules(
                            Reads.rows(7 * k, 7 * k + 7, o::orbDeg),
                            Double.isNaN(horizonRule) ? null : horizonRule,
                            m.withinSignRule(k) == 1)));
        }
        return Collections.unmodifiableList(out);
    }

    private static void checkRagged(String name, int rows, long counted) {
        if (rows != counted) {
            throw Reads.internal(name + " has " + rows + " rows and the charts count " + counted);
        }
    }

    /**
     * Every chart's progressions; empty when none were asked for.
     * {@code progressions} holds a row a chart, the progressed and directed
     * planets graha-count rows a chart when an instant was asked, and the
     * contacts are ragged by the row's count.
     */
    private static List<Progressions> progressions(Charts decoded) {
        Charts.Progressions p = decoded.progressions();
        Charts.ProgressedGrahas g = decoded.progressedGrahas();
        Charts.DirectedGrahas d = decoded.directedGrahas();
        Charts.ProgressedContacts c = decoded.progressedContacts();
        int charts = Reads.charts(decoded);
        if (p.length() == 0) {
            return List.of();
        }
        boolean asked = !Double.isNaN(p.life(0));
        int perChart = asked ? g.length() / charts : 0;
        if (p.length() != charts || d.length() != g.length() || g.length() != perChart * charts
                || c.length() != Reads.sum(p.length(), p::contactCount)) {
            throw Reads.internal("progressions has " + p.length() + " rows, progressed_grahas " + g.length()
                    + ", directed_grahas " + d.length() + " and progressed_contacts " + c.length() + " for "
                    + charts + " charts");
        }
        List<Progressions> out = new ArrayList<>(charts);
        int contact = 0;
        for (int chart = 0; chart < charts; chart += 1) {
            int k = chart;
            int from = k * perChart;
            int to = (k + 1) * perChart;
            int count = Math.toIntExact(p.contactCount(k));
            List<ProgressedContact> contacts = null;
            if (p.contactsAsked(k) == 1) {
                contacts = Reads.rows(contact, contact + count, at -> new ProgressedContact(
                        c.life(at), c.sky(at), Graha.of(c.graha(at)), pointAt(c.toLagna(at), c.toGraha(at)),
                        c.angle(at), Motion.of(c.motion(at))));
            }
            contact += count;
            out.add(new Progressions(
                    asked ? new Progressed(
                            p.life(k),
                            p.sky(k),
                            p.armcDeg(k),
                            new ProgressedAngles(p.ascendantDeg(k), p.midheavenDeg(k)),
                            Reads.rows(from, to, at -> new ProgressedPlanet(Graha.of(g.graha(at)), g.longitudeDeg(at),
                                    g.tropicalDeg(at), g.speedDegPerDay(at))))
                            : null,
                    asked ? new Directed(
                            p.life(k),
                            p.arcDeg(k),
                            p.directedAscendantDeg(k),
                            p.directedMidheavenDeg(k),
                            Reads.rows(from, to, at -> new DirectedPlanet(Graha.of(d.graha(at)), d.longitudeDeg(at))))
                            : null,
                    contacts));
        }
        return Collections.unmodifiableList(out);
    }

    /**
     * Every chart's Western aspect table; empty when none was asked for.
     * {@code western_aspects} holds a row a chart and
     * {@code western_aspect_rows} is ragged by its count.
     */
    private static List<List<WesternAspectRow>> westernAspects(Charts decoded) {
        Charts.WesternAspects a = decoded.westernAspects();
        Charts.WesternAspectRows r = decoded.westernAspectRows();
        return Reads.ragged(Reads.charts(decoded), a.length(), a::count, r.length(),
                "western_aspects and western_aspect_rows",
                at -> new WesternAspectRow(Graha.of(r.first(at)), Graha.of(r.second(at)), WesternAspect.of(r.aspect(at)),
                        r.apartDeg(at), r.fromExactDeg(at), r.orbDeg(at), r.applying(at) == 1));
    }

    /**
     * Every chart's synastry with the partner; empty when none was asked
     * for. {@code synastry} holds a row a chart and {@code synastry_rows} is
     * ragged by its count.
     */
    private static List<List<SynastryRow>> synastries(Charts decoded) {
        Charts.Synastry s = decoded.synastry();
        Charts.SynastryRows r = decoded.synastryRows();
        return Reads.ragged(Reads.charts(decoded), s.length(), s::count, r.length(), "synastry and synastry_rows",
                at -> new SynastryRow(pointAt(r.firstLagna(at), r.firstGraha(at)),
                        pointAt(r.secondLagna(at), r.secondGraha(at)), WesternAspect.of(r.aspect(at)),
                        r.apartDeg(at), r.fromExactDeg(at), r.orbDeg(at)));
    }

    /**
     * Every chart's antiscia; empty when none were asked for.
     * {@code antiscia} holds a row a chart, and {@code antiscion_points} and
     * {@code antiscion_rows} are ragged by its two counts.
     */
    private static List<Antiscia> antiscia(Charts decoded) {
        int charts = Reads.charts(decoded);
        Charts.Antiscia row = decoded.antiscia();
        Charts.AntiscionPoints p = decoded.antiscionPoints();
        Charts.AntiscionRows r = decoded.antiscionRows();
        Charts.AntiscionCuspRows c = decoded.antiscionCuspRows();
        List<List<Integer>> points = Reads.ragged(charts, row.length(), row::pointCount, p.length(),
                "antiscia and antiscion_points", at -> at);
        List<List<AntiscionRow>> pairs = Reads.ragged(charts, row.length(), row::pairCount, r.length(),
                "antiscia and antiscion_rows",
                at -> new AntiscionRow(Graha.of(r.first(at)), Graha.of(r.second(at)), r.contrary(at) == 1,
                        r.apartDeg(at), r.orbDeg(at)));
        List<List<CuspAntiscion>> onCusps = Reads.ragged(charts, row.length(), row::cuspCount, c.length(),
                "antiscia and antiscion_cusp_rows",
                at -> new CuspAntiscion(Graha.of(c.graha(at)), c.house(at), c.contrary(at) == 1));
        List<Antiscia> out = new ArrayList<>(points.size());
        for (int k = 0; k < points.size(); k += 1) {
            List<Integer> rows = points.get(k);
            List<Antiscion> reflections = new ArrayList<>(rows.size());
            List<Graha> unpaired = new ArrayList<>();
            for (int at : rows) {
                reflections.add(new Antiscion(Graha.of(p.graha(at)), p.antiscionDeg(at), p.contrantiscionDeg(at)));
                if (p.paired(at) == 0) {
                    unpaired.add(Graha.of(p.graha(at)));
                }
            }
            out.add(new Antiscia(reflections, pairs.get(k), unpaired, onCusps.isEmpty() ? List.of() : onCusps.get(k),
                    row.cuspSystem(k) == NO_HOUSE_SYSTEM ? null : HouseSystem.of(row.cuspSystem(k))));
        }
        return Collections.unmodifiableList(out);
    }

    /**
     * Every chart's harmonic chart; empty when none was asked for.
     * {@code harmonics} holds a row a chart, and {@code harmonic_points} and
     * {@code harmonic_rows} are ragged by its two counts.
     */
    private static List<HarmonicChart> harmonics(Charts decoded) {
        int charts = Reads.charts(decoded);
        Charts.Harmonics h = decoded.harmonics();
        Charts.HarmonicPoints p = decoded.harmonicPoints();
        Charts.HarmonicRows r = decoded.harmonicRows();
        List<List<HarmonicPlaced>> points = Reads.ragged(charts, h.length(), h::pointCount, p.length(),
                "harmonics and harmonic_points",
                at -> new HarmonicPlaced(HarmonicPoint.of(p.angle(at), p.graha(at)), p.longitudeDeg(at), p.house(at)));
        List<List<HarmonicRow>> rows = Reads.ragged(charts, h.length(), h::rowCount, r.length(),
                "harmonics and harmonic_rows",
                at -> new HarmonicRow(HarmonicPoint.of(r.firstAngle(at), r.firstGraha(at)),
                        HarmonicPoint.of(r.secondAngle(at), r.secondGraha(at)), r.apartDeg(at), r.multiple(at),
                        r.orbDeg(at)));
        List<HarmonicChart> out = new ArrayList<>(points.size());
        for (int k = 0; k < points.size(); k += 1) {
            out.add(new HarmonicChart(h.number(k), points.get(k), rows.get(k)));
        }
        return Collections.unmodifiableList(out);
    }

    /**
     * Every chart's Western houses; empty when none were asked for.
     * {@code western_houses} holds a row a chart,
     * {@code western_house_cusps} twelve a chart, and
     * {@code western_house_planets} is ragged by its count.
     */
    private static List<WesternHouses> westernHouses(Charts decoded) {
        int charts = Reads.charts(decoded);
        Charts.WesternHouses h = decoded.westernHouses();
        Charts.WesternHousePlanets g = decoded.westernHousePlanets();
        Charts.WesternHouseCusps u = decoded.westernHouseCusps();
        List<List<Double>> cusps = Reads.ragged(charts, h.length(), k -> 12, u.length(),
                "western_houses and western_house_cusps", u::cuspDeg);
        List<List<WesternHousePlacement>> planets = Reads.ragged(charts, h.length(), h::planetCount, g.length(),
                "western_houses and western_house_planets",
                at -> new WesternHousePlacement(Graha.of(g.graha(at)), g.house(at), g.withAscendant(at) == 1));
        List<WesternHouses> out = new ArrayList<>(planets.size());
        for (int k = 0; k < planets.size(); k += 1) {
            out.add(new WesternHouses(HouseSystem.of(h.system(k)), cusps.get(k), h.ascendantDeg(k), h.reachDeg(k),
                    planets.get(k)));
        }
        return Collections.unmodifiableList(out);
    }

    /**
     * Every chart's equal distances; empty when none were asked for.
     * {@code midpoints} holds a row a chart and {@code midpoint_rows} is
     * ragged by its count.
     */
    private static List<List<MidpointRow>> midpoints(Charts decoded) {
        Charts.Midpoints m = decoded.midpoints();
        Charts.MidpointRows r = decoded.midpointRows();
        return Reads.ragged(Reads.charts(decoded), m.length(), m::count, r.length(), "midpoints and midpoint_rows",
                at -> new MidpointRow(Graha.of(r.first(at)), Graha.of(r.second(at)), Graha.of(r.middle(at)),
                        r.far(at) == 1, r.distanceDeg(at), r.fromAxisDeg(at), r.orbDeg(at)));
    }

    /**
     * Every chart's equal distances with the partner; empty when none were
     * asked for. {@code synastry_midpoints} holds a row a chart and
     * {@code synastry_midpoint_rows} is ragged by its count.
     */
    private static List<List<SynastryMidpointRow>> synastryMidpoints(Charts decoded) {
        Charts.SynastryMidpoints m = decoded.synastryMidpoints();
        Charts.SynastryMidpointRows r = decoded.synastryMidpointRows();
        return Reads.ragged(Reads.charts(decoded), m.length(), m::count, r.length(),
                "synastry_midpoints and synastry_midpoint_rows",
                at -> new SynastryMidpointRow(Graha.of(r.first(at)), Graha.of(r.second(at)), Graha.of(r.middle(at)),
                        r.far(at) == 1, r.distanceDeg(at), r.fromAxisDeg(at), r.orbDeg(at), r.partnersPair(at) == 1));
    }

    /**
     * Every chart's antiscia with the partner; empty when none were asked
     * for. {@code synastry_antiscia} holds a row a chart and
     * {@code synastry_antiscion_rows} is ragged by its count.
     */
    private static List<List<AntiscionRow>> synastryAntiscia(Charts decoded) {
        Charts.SynastryAntiscia s = decoded.synastryAntiscia();
        Charts.SynastryAntiscionRows r = decoded.synastryAntiscionRows();
        return Reads.ragged(Reads.charts(decoded), s.length(), s::count, r.length(),
                "synastry_antiscia and synastry_antiscion_rows",
                at -> new AntiscionRow(Graha.of(r.first(at)), Graha.of(r.second(at)), r.contrary(at) == 1,
                        r.apartDeg(at), r.orbDeg(at)));
    }

    /**
     * Every chart's composite with the partner; empty when none was asked
     * for. {@code synastry_composites} holds a row a chart and
     * {@code synastry_composite_rows} is ragged by its count.
     */
    private static List<Composite> synastryComposites(Charts decoded) {
        int charts = Reads.charts(decoded);
        Charts.SynastryComposites c = decoded.synastryComposites();
        Charts.SynastryCompositeRows r = decoded.synastryCompositeRows();
        Charts.SynastryCompositeCusps u = decoded.synastryCompositeCusps();
        List<List<CompositePlanet>> planets = Reads.ragged(charts, c.length(), c::count, r.length(),
                "synastry_composites and synastry_composite_rows",
                at -> new CompositePlanet(Graha.of(r.graha(at)), r.longitudeDeg(at), r.speedDegPerDay(at)));
        List<List<Double>> cusps = Reads.ragged(charts, c.length(), c::cuspCount, u.length(),
                "synastry_composites and synastry_composite_cusps", u::cuspDeg);
        List<Composite> out = new ArrayList<>(planets.size());
        for (int k = 0; k < planets.size(); k += 1) {
            List<Double> own = cusps.isEmpty() ? List.of() : cusps.get(k);
            out.add(new Composite(planets.get(k), c.lagnaDeg(k), c.midheavenDeg(k), c.lagnaTurned(k) == 1,
                    own.isEmpty() ? null : own));
        }
        return Collections.unmodifiableList(out);
    }

    /**
     * Every chart's Davison birth with the partner; empty when none was
     * asked for, and a row a chart otherwise.
     */
    private static List<DavisonBirth> synastryDavisons(Charts decoded) {
        Charts.SynastryDavisons b = decoded.synastryDavisons();
        int charts = Reads.charts(decoded);
        if (b.length() != 0 && b.length() != charts) {
            throw Reads.internal("synastry_davisons has " + b.length() + " rows for " + charts
                    + " charts; it is one a chart or none");
        }
        return Reads.rows(0, b.length(), k -> new DavisonBirth(
                b.instant(k),
                new Observer(new Longitude(b.longitudeDeg(k)), new Latitude(b.latitudeDeg(k)),
                        new Altitude(b.altitudeM(k))),
                b.utcOffsetSeconds(k)));
    }

    /**
     * Every chart's parallels with the partner; empty when none were asked
     * for. {@code synastry_parallels} holds a row a chart and
     * {@code synastry_parallel_rows} is ragged by its count.
     */
    private static List<List<SynastryParallelRow>> synastryParallels(Charts decoded) {
        Charts.SynastryParallels s = decoded.synastryParallels();
        Charts.SynastryParallelRows r = decoded.synastryParallelRows();
        return Reads.ragged(Reads.charts(decoded), s.length(), s::count, r.length(),
                "synastry_parallels and synastry_parallel_rows",
                at -> new SynastryParallelRow(pointAt(r.firstLagna(at), r.firstGraha(at)),
                        pointAt(r.secondLagna(at), r.secondGraha(at)), r.contrary(at) == 1, r.apartDeg(at),
                        r.orbDeg(at)));
    }

    /**
     * Every chart's declinations; empty when none were asked for.
     * {@code declinations} holds a row a chart, and {@code declination_rows}
     * is ragged by its {@code graha_count}.
     */
    private static List<Declinations> declinations(Charts decoded) {
        Charts.Declinations row = decoded.declinations();
        Charts.DeclinationRows rows = decoded.declinationRows();
        List<List<Declined>> grahas = Reads.ragged(Reads.charts(decoded), row.length(), row::grahaCount,
                rows.length(), "declinations and declination_rows",
                at -> new Declined(Graha.of(rows.graha(at)), rows.declinationDeg(at)));
        List<Declinations> out = new ArrayList<>(grahas.size());
        for (int k = 0; k < grahas.size(); k += 1) {
            out.add(new Declinations(row.obliquityDeg(k), grahas.get(k), row.lagnaDeg(k), row.midheavenDeg(k)));
        }
        return Collections.unmodifiableList(out);
    }

    /**
     * Every chart's parallels; empty when none were asked for.
     * {@code parallel_rows} is ragged by {@code declinations.parallel_count}.
     */
    private static List<List<ParallelRow>> parallels(Charts decoded) {
        Charts.Declinations row = decoded.declinations();
        Charts.ParallelRows p = decoded.parallelRows();
        return Reads.ragged(Reads.charts(decoded), row.length(), row::parallelCount, p.length(),
                "declinations and parallel_rows",
                at -> new ParallelRow(Graha.of(p.first(at)), Graha.of(p.second(at)), p.contrary(at) == 1,
                        p.apartDeg(at), p.orbDeg(at)));
    }
}

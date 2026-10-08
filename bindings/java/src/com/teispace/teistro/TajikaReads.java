package com.teispace.teistro;

import java.util.ArrayList;
import java.util.List;
import java.util.function.IntToDoubleFunction;
import java.util.function.IntToLongFunction;
import java.util.function.IntUnaryOperator;

import com.teispace.teistro.blob.Charts;

/**
 * A chart's Tajika readings: its own sahams, and its annual charts' instants
 * with each year's chart read down to what Tajika reads from it, as the
 * Python façade's {@code Chart.sahams} and {@code Chart.praveshas} read them.
 */
final class TajikaReads {
    private TajikaReads() {
    }

    /**
     * The birth chart's own sahams, each with its strength clause by clause,
     * which has no year lord, in the order the varsha request's {@code sahams}
     * named them; empty unless it asked ({@code 03-design/tajika-saham-strength.md}).
     * Ports {@code Chart.sahams}.
     */
    static List<TajikaSaham> sahams(Chart chart) {
        Charts decoded = chart.batch().decoded();
        Charts.Cast cast = decoded.cast();
        int start = VedicReads.start(cast::natalSahamCount, chart.index());
        int count = Math.toIntExact(cast.natalSahamCount(chart.index()));
        Charts.NatalSahams c = decoded.natalSahams();
        Charts.NatalSahamSeven s = decoded.natalSahamSeven();
        SahamColumns columns = new SahamColumns(c::saham, c::longitudeDeg, c::sign, c::lord, c::house, c::addedSign,
                c::strong, c::weak, c::lordVishwa, c::lordHarsha, c::nodeAxis);
        SevenColumns seven = new SevenColumns(s::graha, s::drishti, s::relation, s::company);
        List<TajikaSaham> out = new ArrayList<>(count);
        for (int k = start; k < start + count; k += 1) {
            out.add(sahamAt(columns, seven, k));
        }
        return List.copyOf(out);
    }

    /**
     * The annual charts' instants: the Sun's returns to where it stood at
     * birth, in year order; empty unless {@code varsha} asked for them
     * ({@code 03-design/annual-chart.md}). The section is <b>ragged</b>: the
     * request settles how many returns are wanted and the ephemeris how many
     * there are, so read the length rather than the number asked for. The
     * place is the caller's: pass the instant to {@code found} yourself.
     * Ports {@code Chart.praveshas}.
     */
    static List<Pravesha> praveshas(Chart chart) {
        Charts decoded = chart.batch().decoded();
        Charts.Cast cast = decoded.cast();
        int start = VedicReads.start(cast::praveshaCount, chart.index());
        int count = Math.toIntExact(cast.praveshaCount(chart.index()));
        Charts.Praveshas columns = decoded.praveshas();
        Starts starts = chart.batch().cached("tajikaStarts", () -> Starts.of(decoded));
        List<Pravesha> out = new ArrayList<>(count);
        for (int i = start; i < start + count; i += 1) {
            out.add(new Pravesha(
                    columns.year(i),
                    columns.jd(i),
                    new Muntha(Rashi.of(columns.munthaSign(i)), Graha.of(columns.munthaLord(i)), columns.munthaDeg(i)),
                    annualChart(decoded, i, starts)));
        }
        return List.copyOf(out);
    }

    /**
     * Where each row's block starts in each ragged section under the annual
     * charts: {@code starts[row]} to {@code starts[row + 1]}. Computed once for
     * the batch rather than once a year, so reading them is linear.
     */
    private record Starts(int[] claims, int[] yogas, int[] matters, int[] held, int[] legs, int[] sahams,
            int[] dashas, int[] shares, int[] periods) {
        static int[] running(IntToLongFunction counts, int length) {
            int[] starts = new int[length + 1];
            long sum = 0;
            for (int at = 0; at < length; at += 1) {
                sum += counts.applyAsLong(at);
                starts[at + 1] = Math.toIntExact(sum);
            }
            return starts;
        }

        static Starts of(Charts decoded) {
            Charts.AnnualCharts charts = decoded.annualCharts();
            Charts.YearMatters matters = decoded.yearMatters();
            Charts.MatterYogas yogas = decoded.matterYogas();
            Charts.YearDashas dashas = decoded.yearDashas();
            return new Starts(
                    running(charts::claimCount, charts.length()),
                    running(charts::yogaCount, charts.length()),
                    running(charts::matterCount, charts.length()),
                    running(matters::heldCount, matters.length()),
                    running(yogas::legCount, yogas.length()),
                    running(charts::sahamCount, charts.length()),
                    running(charts::dashaCount, charts.length()),
                    running(dashas::shareCount, dashas.length()),
                    running(dashas::periodCount, dashas.length()));
        }
    }

    /** The columns of a saham section, the years' or the births', which share a layout. */
    private record SahamColumns(IntUnaryOperator saham, IntToDoubleFunction longitudeDeg, IntUnaryOperator sign,
            IntUnaryOperator lord, IntUnaryOperator house, IntUnaryOperator addedSign, IntUnaryOperator strong,
            IntUnaryOperator weak, IntUnaryOperator lordVishwa, IntUnaryOperator lordHarsha,
            IntUnaryOperator nodeAxis) {
    }

    /** The columns of a saham's seven rows, the years' or the births'. */
    private record SevenColumns(IntUnaryOperator graha, IntUnaryOperator drishti, IntUnaryOperator relation,
            IntUnaryOperator company) {
    }

    /** A strength from the integer sub-sub units the boundary carries. */
    static Bala bala(int subSub) {
        int units = Math.floorDiv(subSub, 3600);
        int rest = Math.floorMod(subSub, 3600);
        return new Bala(units, rest / 60, rest % 60, subSub);
    }

    /**
     * One saham row, from the years' sections or the births': where it fell,
     * its strength clause by clause, and the seven rows under it. One decoder
     * for both, so they cannot drift.
     */
    private static TajikaSaham sahamAt(SahamColumns cols, SevenColumns seven, int k) {
        int axis = cols.nodeAxis().applyAsInt(k);
        List<SahamSeven> sevens = new ArrayList<>(7);
        for (int at = 7 * k; at < 7 * k + 7; at += 1) {
            sevens.add(new SahamSeven(
                    Graha.of(seven.graha().applyAsInt(at)),
                    TajikaDrishti.of(seven.drishti().applyAsInt(at)),
                    TajikaRelation.of(seven.relation().applyAsInt(at)),
                    seven.company().applyAsInt(at) == 1));
        }
        return new TajikaSaham(
                Saham.of(cols.saham().applyAsInt(k)),
                cols.longitudeDeg().applyAsDouble(k),
                Rashi.of(cols.sign().applyAsInt(k)),
                Graha.of(cols.lord().applyAsInt(k)),
                cols.house().applyAsInt(k),
                cols.addedSign().applyAsInt(k) == 1,
                VedicReads.members(cols.strong().applyAsInt(k), SahamStrong.values()),
                VedicReads.members(cols.weak().applyAsInt(k), SahamWeak.values()),
                bala(cols.lordVishwa().applyAsInt(k)),
                HarshaGrade.of(cols.lordHarsha().applyAsInt(k)),
                // 2 is the boundary's "the chart placed no nodes to read".
                axis == 2 ? null : Boolean.valueOf(axis == 1),
                sevens);
    }

    /** How two planets stand, from a section carrying the pair columns: a matter's lords or a yoga's legs. */
    private static TajikaBetween between(int faster, int slower, int drishti, int yoga, boolean yogaPresent,
            double orbDeg, double apartDeg) {
        return new TajikaBetween(Graha.of(faster), Graha.of(slower), TajikaDrishti.of(drishti),
                yogaPresent ? TajikaYoga.of(yoga) : null, orbDeg, apartDeg);
    }

    private static TajikaBetween leg(Charts.MatterLegs legs, int k) {
        return between(legs.faster(k), legs.slower(k), legs.drishti(k), legs.yoga(k), legs.yogaPresent(k) == 1,
                legs.orbDeg(k), legs.apartDeg(k));
    }

    /**
     * A year's matters, each with its question, the lords' pair, what it could
     * not answer, and every yoga that held, ragged three deep
     * ({@code 03-design/tajika-yogas.md}).
     */
    private static List<TajikaMatter> matters(Charts decoded, int row, Starts starts) {
        Charts.YearMatters matters = decoded.yearMatters();
        Charts.MatterYogas held = decoded.matterYogas();
        Charts.MatterLegs legs = decoded.matterLegs();
        Affliction[] afflictions = Affliction.values();
        List<TajikaMatter> found = new ArrayList<>();
        for (int m = starts.matters()[row]; m < starts.matters()[row + 1]; m += 1) {
            TajikaBetween between = matters.sameLord(m) == 1 ? null
                    : between(matters.pairFaster(m), matters.pairSlower(m), matters.pairDrishti(m),
                            matters.pairYoga(m), matters.pairYogaPresent(m) == 1, matters.pairOrbDeg(m),
                            matters.pairApartDeg(m));
            List<HeldYearYoga> yogas = new ArrayList<>();
            for (int h = starts.held()[m]; h < starts.held()[m + 1]; h += 1) {
                yogas.add(new HeldYearYoga(
                        YearYoga.of(held.yoga(h)),
                        held.byPair(h) == 1 ? between : null,
                        held.throughPresent(h) == 1 ? Graha.of(held.through(h)) : null,
                        held.enteringPresent(h) == 1 ? Graha.of(held.entering(h)) : null,
                        held.legCount(h) == 2
                                ? List.of(leg(legs, starts.legs()[h]), leg(legs, starts.legs()[h] + 1))
                                : null,
                        held.afflictionsPresent(h) == 1
                                ? new Afflictions(VedicReads.members(held.lagneshaAfflictions(h), afflictions),
                                        VedicReads.members(held.karyeshaAfflictions(h), afflictions))
                                : null));
            }
            found.add(new TajikaMatter(
                    matters.house(m),
                    Rashi.of(matters.sign(m)),
                    Graha.of(matters.lagnesha(m)),
                    Graha.of(matters.karyesha(m)),
                    matters.sameLord(m) == 1,
                    between,
                    yogas,
                    VedicReads.members(matters.unanswered(m), YearYoga.values())));
        }
        return List.copyOf(found);
    }

    /**
     * A year's annual dashas, ragged by {@code dasha_count}, each with its ring
     * and its periods ragged under it by {@code share_count} and
     * {@code period_count} ({@code 03-design/annual-dashas.md}).
     */
    private static List<AnnualDasha> annualDashas(Charts decoded, int row, Starts starts) {
        Charts.YearDashas rows = decoded.yearDashas();
        Charts.YearDashaShares shares = decoded.yearDashaShares();
        Charts.YearDashaPeriods cells = decoded.yearDashaPeriods();
        List<AnnualDasha> found = new ArrayList<>();
        for (int k = starts.dashas()[row]; k < starts.dashas()[row + 1]; k += 1) {
            double remaining = rows.remaining(k);
            List<AnnualDashaShare> ring = new ArrayList<>();
            for (int i = starts.shares()[k]; i < starts.shares()[k + 1]; i += 1) {
                ring.add(new AnnualDashaShare(Graha.of(shares.lord(i)),
                        shares.hasSign(i) == 1 ? Rashi.of(shares.sign(i)) : null, shares.weight(i)));
            }
            found.add(new AnnualDasha(
                    DashaSystem.of(rows.system(k)),
                    rows.seeded(k) == 1 ? Nakshatra.of(rows.seed(k)) : null,
                    new DashaRing(ring, rows.first(k), Double.isNaN(remaining) ? null : remaining),
                    new Interval(rows.fromJd(k), rows.toJd(k)),
                    DashaReads.periods(cells::level, cells::index, cells::sign, cells::lord, cells::fromJd,
                            cells::toJd, starts.periods()[k], Math.toIntExact(rows.periodCount(k)),
                            i -> cells.hasSign(i) == 1)));
        }
        return List.copyOf(found);
    }

    /** A year's sahams, ragged by {@code saham_count} ({@code 03-design/tajika-sahams.md}). */
    private static List<TajikaSaham> yearSahams(Charts decoded, int row, Starts starts) {
        Charts.YearSahams c = decoded.yearSahams();
        Charts.YearSahamSeven s = decoded.yearSahamSeven();
        SahamColumns columns = new SahamColumns(c::saham, c::longitudeDeg, c::sign, c::lord, c::house, c::addedSign,
                c::strong, c::weak, c::lordVishwa, c::lordHarsha, c::nodeAxis);
        SevenColumns seven = new SevenColumns(s::graha, s::drishti, s::relation, s::company);
        List<TajikaSaham> found = new ArrayList<>();
        for (int k = starts.sahams()[row]; k < starts.sahams()[row + 1]; k += 1) {
            found.add(sahamAt(columns, seven, k));
        }
        return List.copyOf(found);
    }

    /** A founded year's Harsha bala: seven rows a year, fixed, in the catalogue's order. */
    private static List<HarshaBala> harsha(Charts decoded, int row) {
        Charts.YearHarsha h = decoded.yearHarsha();
        List<HarshaBala> found = new ArrayList<>(7);
        for (int at = 7 * row; at < 7 * row + 7; at += 1) {
            found.add(new HarshaBala(Graha.of(h.graha(at)), h.house(at), h.sthana(at) == 1,
                    h.uchchaSwakshetra(at) == 1, h.striPurusha(at) == 1, h.dinaRatri(at) == 1, h.total(at),
                    HarshaGrade.of(h.grade(at))));
        }
        return List.copyOf(found);
    }

    /**
     * Row {@code row} of {@code annual_charts}, which runs beside
     * {@code praveshas} row for row or is empty; anything between is a layout
     * this layer cannot pair, and it says so rather than giving a year another
     * year's chart.
     */
    private static AnnualChart annualChart(Charts decoded, int row, Starts starts) {
        Charts.AnnualCharts charts = decoded.annualCharts();
        if (charts.length() == 0) {
            return null;
        }
        if (charts.length() != decoded.praveshas().length()) {
            throw new IllegalStateException("annual_charts has " + charts.length() + " rows beside "
                    + decoded.praveshas().length() + " returns; it is all of them or none");
        }
        // The claims are ragged by `claim_count`, as the returns are by
        // `pravesha_count`: this year's block starts where the ones before end.
        int start = starts.claims()[row];
        int count = charts.claimCount(row);
        Charts.YearClaims claims = decoded.yearClaims();
        List<YearClaim> claimed = new ArrayList<>(count);
        for (int i = start; i < start + count; i += 1) {
            claimed.add(new YearClaim(Graha.of(claims.graha(i)), bala(claims.vishwa(i)), claims.portfolios(i),
                    claims.aspectsLagna(i) == 1));
        }
        int yogaStart = starts.yogas()[row];
        Charts.YearYogas pairs = decoded.yearYogas();
        List<TajikaPair> yogas = new ArrayList<>();
        for (int i = yogaStart; i < yogaStart + charts.yogaCount(row); i += 1) {
            yogas.add(new TajikaPair(Graha.of(pairs.faster(i)), Graha.of(pairs.slower(i)),
                    TajikaDrishti.of(pairs.drishti(i)), TajikaYoga.of(pairs.yoga(i)), pairs.orbDeg(i),
                    pairs.apartDeg(i)));
        }
        Graha[] seven = VedicReads.grahaIds(7);
        return new AnnualChart(
                charts.lagnaDeg(row),
                charts.daylight(row) == 1,
                new OfficeBearers(
                        Graha.of(decoded.praveshas().munthaLord(row)),
                        Graha.of(charts.janmaLagnaLord(row)),
                        Graha.of(charts.varshaLagnaLord(row)),
                        Graha.of(charts.triRashiLord(row)),
                        Graha.of(charts.dinaRatriLord(row))),
                new YearLord(
                        Graha.of(charts.yearLord(row)),
                        VarsheshaChosen.of(charts.yearLordChosen(row)),
                        bala(charts.yearLordVishwa(row)),
                        charts.moonPassedOver(row) == 1,
                        claimed),
                yogas,
                VedicReads.members(charts.retrograde(row), seven),
                VedicReads.members(charts.combust(row), seven),
                matters(decoded, row, starts),
                yearSahams(decoded, row, starts),
                harsha(decoded, row),
                annualDashas(decoded, row, starts));
    }
}

package com.teispace.teistro;

import java.util.ArrayList;
import java.util.EnumMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.function.IntFunction;
import java.util.function.IntToLongFunction;

import com.teispace.teistro.blob.Charts;
import com.teispace.teistro.blob.MatchingKootas;
import com.teispace.teistro.blob.Matchings;
import com.teispace.teistro.blob.PoruthamRows;
import com.teispace.teistro.blob.Poruthams;

/**
 * A chart matched with a partner's birth: the Ashta Koota, the ten
 * considerations, the Kuja dosha on each side and the marriage doshas, as
 * the Python façade's {@code Chart.matching}, {@code Chart.porutham},
 * {@code Chart.kuja} and {@code Chart.marriage_doshas} read them. The
 * Ashta Koota and the ten are read from shapes any blob carrying them shares,
 * a chart batch's or a naam blob's, through {@link #matchingsIn} and
 * {@link #poruthamsIn}.
 */
final class MatchReads {
    private MatchReads() {
    }

    /**
     * A per-chart table from a count column and the rows it is ragged by, for
     * {@code charts} charts: each chart's rows, or none at all when the count
     * column is empty because nothing was asked.
     *
     * @param counts the count column, {@code countLength} rows
     * @param rows the ragged section's rows
     * @param names the two sections, for a refusal
     */
    static <T> List<List<T>> raggedIn(int charts, IntToLongFunction counts, int countLength, int rows, String names,
            IntFunction<T> read) {
        if (countLength == 0) {
            return List.of();
        }
        long total = VedicReads.sum(counts, countLength);
        if (countLength != charts || rows != total) {
            throw JsonRead.internal(
                    names + ": " + countLength + " counts and " + rows + " rows for " + charts + " charts");
        }
        List<List<T>> tables = new ArrayList<>(countLength);
        int start = 0;
        for (int at = 0; at < countLength; at += 1) {
            int count = Math.toIntExact(counts.applyAsLong(at));
            List<T> table = new ArrayList<>(count);
            for (int row = start; row < start + count; row += 1) {
                table.add(read.apply(row));
            }
            tables.add(List.copyOf(table));
            start += count;
        }
        return List.copyOf(tables);
    }

    /**
     * The chart matched with a partner's birth by the Ashta Koota of
     * <i>Muhurta Chintamani</i> VI.21 to 34: each koota's points and what it
     * read, in the verse's order, and the total out of 36. Never a verdict:
     * the doshas and their exceptions are clauses; empty unless
     * {@code matching} asked ({@code 03-design/matching.md}). Ports {@code Chart.matching}.
     */
    static Optional<AshtaKoota> matching(Chart chart) {
        Charts decoded = chart.batch().decoded();
        return VedicReads.at(chart.batch().cached("matchings",
                () -> matchingsIn(decoded.matchings(), decoded.matchingKootas(), decoded.cast().length())),
                chart.index());
    }

    /**
     * The chart matched with the same partner by the ten considerations of
     * <i>Kalaprakasika</i> XIII: whether each agrees and what it read, in the
     * chapter's order, how many agree, how many of the chief five, and the
     * p. 76 exception's clauses. Never a verdict; empty unless asked.
     * Ports {@code Chart.porutham}.
     */
    static Optional<Porutham> porutham(Chart chart) {
        Charts decoded = chart.batch().decoded();
        return VedicReads.at(chart.batch().cached("poruthams",
                () -> poruthamsIn(decoded.poruthams(), decoded.poruthamRows(), decoded.cast().length())),
                chart.index());
    }

    /**
     * The chart's Kuja dosha beside the same partner's (<i>Manasagari</i>,
     * jayabhava v. 4): Mars's house by sign from the lagna, the Moon and Venus
     * on each side, whether each side carries the dosha under the rules and
     * whether both do. Nothing is lifted; empty unless asked. Ports {@code Chart.kuja}.
     */
    static Optional<Kuja> kuja(Chart chart) {
        return VedicReads.at(chart.batch().cached("kujas", () -> kujas(chart.batch().decoded())), chart.index());
    }

    /**
     * Every marriage dosha the chart's match with the same partner carries,
     * as one list in the answers' own order: the Ashta Koota's Bhakoot, Nadi,
     * Gana and the lords' enmity, each of the ten that disagrees or agrees
     * only by the p. 76 exception, and each side's Kuja dosha, each with
     * whether it is lifted. No severity; empty unless {@code matching} asked.
     * Ports {@code Chart.marriage_doshas}.
     */
    static Optional<List<MarriageDosha>> marriageDoshas(Chart chart) {
        return VedicReads.at(chart.batch().cached("marriageDoshas", () -> marriageDoshasOf(chart.batch().decoded())),
                chart.index());
    }

    /**
     * The Ashta Koota read from the {@code matchings} and {@code matching_kootas}
     * shapes of any blob carrying them, {@code charts} rows: a chart batch's,
     * or a naam blob's one match; empty when none was asked for.
     * {@code matchings} holds a row a match with what each koota read, and
     * {@code matching_kootas} eight rows a match, each koota's points in the
     * verse's order.
     */
    static List<AshtaKoota> matchingsIn(Matchings m, MatchingKootas k, int charts) {
        List<List<KootaRowCells>> kootas = raggedIn(charts, at -> 8, m.length(), k.length(),
                "matchings and matching_kootas",
                row -> new KootaRowCells(Koota.of(k.koota(row)), k.points(row), k.maxPoints(row)));
        List<AshtaKoota> matched = new ArrayList<>(kootas.size());
        for (int at = 0; at < kootas.size(); at += 1) {
            Map<Koota, KootaReading> read = kootaReadings(m, at);
            List<KootaRow> rows = new ArrayList<>(8);
            for (KootaRowCells cells : kootas.get(at)) {
                rows.add(new KootaRow(cells.points(), cells.most(), reading(read, cells.koota())));
            }
            matched.add(new AshtaKoota(rows, m.total(at)));
        }
        return List.copyOf(matched);
    }

    /** One {@code matching_kootas} row before its reading is joined to it. */
    private record KootaRowCells(Koota koota, double points, double most) {
    }

    /** What a koota read, refusing a koota the readings do not carry as Python's mapping would. */
    private static <R> R reading(Map<Koota, R> read, Koota koota) {
        R found = read.get(koota);
        if (found == null) {
            throw JsonRead.internal("a match row names the koota " + koota + ", which no reading carries");
        }
        return found;
    }

    private static Map<Koota, KootaReading> kootaReadings(Matchings m, int at) {
        BhakootDosha dosha = BhakootDosha.of(m.bhakootDosha(at));
        List<KootaReading> read = List.of(
                new VarnaKoota(Varna.of(m.brideVarna(at)), Varna.of(m.groomVarna(at))),
                new VashyaKoota(VashyaRelation.of(m.vashya(at))),
                new TaraKoota(m.taraBrideToGroom(at), m.taraGroomToBride(at)),
                new YoniKoota(Yoni.of(m.brideYoni(at)), Yoni.of(m.groomYoni(at)), YoniRelation.of(m.yoni(at))),
                new MaitriKoota(Graha.of(m.brideLord(at)), Graha.of(m.groomLord(at)), MaitriRelation.of(m.maitri(at)),
                        m.maitriLifted(at) == 1),
                new GanaKoota(Gana.of(m.brideGana(at)), Gana.of(m.groomGana(at)), m.ganaDosha(at) == 1,
                        m.ganaLifted(at) == 1),
                new BhakootKoota(
                        m.bhakootApart(at),
                        dosha == BhakootDosha.NONE ? null : dosha,
                        new BhakootExceptions(
                                m.bhakootOneLord(at) == 1,
                                m.bhakootLordsFriends(at) == 1,
                                m.bhakootNavamshaLordsFriends(at) == 1,
                                m.bhakootTaraPure(at) == 1,
                                m.bhakootVashya(at) == 1),
                        m.bhakootLifted(at) == 1),
                new NadiKoota(Nadi.of(m.brideNadi(at)), Nadi.of(m.groomNadi(at)), m.nadiDosha(at) == 1,
                        m.nadiLifted(at) == 1));
        Map<Koota, KootaReading> byKoota = new EnumMap<>(Koota.class);
        for (KootaReading one : read) {
            byKoota.put(one.koota(), one);
        }
        return byKoota;
    }

    /**
     * The ten considerations read from the {@code poruthams} and
     * {@code porutham_rows} shapes of any blob carrying them, {@code charts}
     * rows; empty when none was asked for. {@code poruthams} holds a row a
     * match with what each read, and {@code porutham_rows} ten rows a match,
     * whether each agrees in the chapter's order.
     */
    static List<Porutham> poruthamsIn(Poruthams p, PoruthamRows r, int charts) {
        List<List<PoruthamRowCells>> rows = raggedIn(charts, at -> 10, p.length(), r.length(),
                "poruthams and porutham_rows",
                row -> new PoruthamRowCells(Koota.of(r.koota(row)), r.agrees(row) == 1, r.lifted(row) == 1));
        List<Porutham> matched = new ArrayList<>(rows.size());
        for (int at = 0; at < rows.size(); at += 1) {
            Map<Koota, PoruthamReading> read = poruthamReadings(p, at);
            List<PoruthamRow> ten = new ArrayList<>(10);
            for (PoruthamRowCells cells : rows.get(at)) {
                ten.add(new PoruthamRow(cells.agrees(), cells.lifted(), reading(read, cells.koota())));
            }
            matched.add(new Porutham(ten, p.agreeing(at), p.chiefAgreeing(at),
                    new PoruthamException(p.oneLord(at) == 1, p.lordsFriendly(at) == 1, p.opposite(at) == 1)));
        }
        return List.copyOf(matched);
    }

    /** One {@code porutham_rows} row before its reading is joined to it. */
    private record PoruthamRowCells(Koota koota, boolean agrees, boolean lifted) {
    }

    private static Map<Koota, PoruthamReading> poruthamReadings(Poruthams p, int at) {
        List<PoruthamReading> read = List.of(
                new DhinamPorutham(p.count(at), DhinamRule.of(p.dhinamRule(at))),
                new GanamPorutham(Gana.of(p.brideGana(at)), Gana.of(p.groomGana(at)), p.ganaDiminished(at) == 1),
                new MahendraPorutham(p.count(at)),
                new DeerghaPorutham(p.count(at)),
                new YoniPorutham(Yoni.of(p.brideYoni(at)), Yoni.of(p.groomYoni(at)), p.yoniHostile(at) == 1),
                new RasiPorutham(p.apart(at)),
                new RasyadhipathiPorutham(Graha.of(p.brideLord(at)), Graha.of(p.groomLord(at)),
                        p.brideCallsFriend(at) == 1, p.groomCallsFriend(at) == 1),
                new VasyamPorutham(p.brideToGroom(at) == 1, p.groomToBride(at) == 1),
                new RajjuPorutham(Rajju.of(p.brideRajju(at)), Rajju.of(p.groomRajju(at))),
                new VedhaiPorutham(p.pierced(at) == 1));
        Map<Koota, PoruthamReading> byKoota = new EnumMap<>(Koota.class);
        for (PoruthamReading one : read) {
            byKoota.put(one.koota(), one);
        }
        return byKoota;
    }

    /** Every chart's Kuja dosha with the record's partner, decoded once; a row a chart. */
    private static List<Kuja> kujas(Charts decoded) {
        Charts.Kujas k = decoded.kujas();
        List<Kuja> out = new ArrayList<>(k.length());
        for (int at = 0; at < k.length(); at += 1) {
            KujaSide bride = new KujaSide(List.of(
                    new KujaReading("LAGNA", k.brideLagnaHouse(at), k.brideLagnaInHouses(at) == 1),
                    new KujaReading("MOON", k.brideMoonHouse(at), k.brideMoonInHouses(at) == 1),
                    new KujaReading("VENUS", k.brideVenusHouse(at), k.brideVenusInHouses(at) == 1)),
                    k.brideDosha(at) == 1);
            KujaSide groom = new KujaSide(List.of(
                    new KujaReading("LAGNA", k.groomLagnaHouse(at), k.groomLagnaInHouses(at) == 1),
                    new KujaReading("MOON", k.groomMoonHouse(at), k.groomMoonInHouses(at) == 1),
                    new KujaReading("VENUS", k.groomVenusHouse(at), k.groomVenusInHouses(at) == 1)),
                    k.groomDosha(at) == 1);
            out.add(new Kuja(bride, groom, k.both(at) == 1));
        }
        return List.copyOf(out);
    }

    /**
     * Every chart's marriage doshas, decoded once; empty when none was asked
     * for. {@code marriage_doshas} holds a count a chart and
     * {@code marriage_dosha_rows} the entries ragged under it.
     */
    private static List<List<MarriageDosha>> marriageDoshasOf(Charts decoded) {
        Charts.MarriageDoshaRows r = decoded.marriageDoshaRows();
        Charts.MarriageDoshas counts = decoded.marriageDoshas();
        return raggedIn(decoded.cast().length(), counts::count, counts.length(), r.length(),
                "marriage_doshas and marriage_dosha_rows", row -> {
                    DoshaSystem system = DoshaSystem.of(r.system(row));
                    boolean kuja = system == DoshaSystem.KUJA;
                    return new MarriageDosha(
                            system,
                            kuja ? null : Koota.of(r.koota(row)),
                            kuja ? MatchRole.of(r.side(row)) : null,
                            r.lifted(row) == 1);
                });
    }
}

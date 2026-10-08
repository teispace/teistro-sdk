package com.teispace.teistro;

import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.function.IntPredicate;
import java.util.function.IntToDoubleFunction;
import java.util.function.IntUnaryOperator;

import com.teispace.teistro.blob.Charts;

/**
 * A chart's dashas, and the period decoding the birth's dashas and a year's
 * annual dashas share, as the Python façade's {@code Chart.dashas} reads them.
 */
final class DashaReads {
    private DashaReads() {
    }

    /**
     * The dashas asked for, in the order asked; empty unless {@code dashas}
     * named some ({@code 03-design/dasha-kernels.md}). Ports {@code Chart.dashas}.
     */
    static List<Dasha> dashas(Chart chart) {
        ChartBatch batch = chart.batch();
        List<List<Dasha>> parsed = batch.cached("dashas", () -> dashasOf(batch.decoded(), batch.dashaNames()));
        return chart.index() < parsed.size() ? parsed.get(chart.index()) : List.of();
    }

    /**
     * Every chart's dashas, decoded once however many charts read them. The
     * periods are <b>ragged</b> by each dasha's {@code period_count}, so a
     * chart's begin where the one before it ends.
     */
    private static List<List<Dasha>> dashasOf(Charts decoded, Map<Integer, String> names) {
        int per = Math.toIntExact(decoded.dashaCount());
        if (per == 0) {
            return List.of();
        }
        Charts.Dashas rows = decoded.dashas();
        List<List<Dasha>> out = new ArrayList<>();
        int start = 0;
        for (int chart = 0; chart < rows.length() / per; chart += 1) {
            List<Dasha> dashas = new ArrayList<>(per);
            for (int j = 0; j < per; j += 1) {
                int row = chart * per + j;
                int count = Math.toIntExact(rows.periodCount(row));
                dashas.add(dasha(decoded, row, start, count, names));
                start += count;
            }
            out.add(List.copyOf(dashas));
        }
        return List.copyOf(out);
    }

    /** A dasha row's system: a registered one's key, or the catalogue's member. */
    private static Object system(int id, Map<Integer, String> names) {
        String registered = names.get(id);
        return registered != null ? registered : DashaSystem.of(id);
    }

    /**
     * One dasha row and its periods. The system is the catalogue's member,
     * or the {@code dasha_system.*} key of one the context registered.
     */
    private static Dasha dasha(Charts decoded, int row, int start, int count, Map<Integer, String> names) {
        Charts.Dashas rows = decoded.dashas();
        Charts.DashaPeriods cells = decoded.dashaPeriods();
        boolean seeded = rows.seeded(row) != 0;
        boolean signed = rows.signed(row) != 0;
        List<DashaPeriod> periods = periods(cells::level, cells::index, cells::sign, cells::lord, cells::fromJd,
                cells::toJd, start, count, at -> signed);
        double spanFrom = rows.moonSpanFrom(row);
        return new Dasha(
                system(rows.system(row), names),
                seeded ? Nakshatra.of(rows.seed(row)) : null,
                Graha.of(rows.firstLord(row)),
                rows.overflow(row) != 0,
                seeded
                        ? new DashaBalance(
                                Balance.of(rows.balance(row)),
                                rows.remaining(row),
                                rows.balanceDays(row),
                                new WrittenBalance(rows.balanceYears(row), rows.balanceMonths(row),
                                        rows.balanceDayCount(row), rows.balanceHours(row), rows.balanceMinutes(row)))
                        : null,
                Double.isNaN(spanFrom) ? null : new Interval(spanFrom, rows.moonSpanTo(row)),
                rows.depth(row),
                periods);
    }

    /**
     * A dasha's periods from a period section, the births' {@code dasha_periods}
     * or the years' {@code year_dasha_periods}, which share a layout, so a
     * period is decoded in one place. A period's path is its index below the
     * nearest earlier period one level up, so it is rebuilt by truncating the
     * path to the level before it.
     */
    static List<DashaPeriod> periods(IntUnaryOperator level, IntUnaryOperator index, IntUnaryOperator sign,
            IntUnaryOperator lord, IntToDoubleFunction fromJd, IntToDoubleFunction toJd, int start, int count,
            IntPredicate signed) {
        List<DashaPeriod> periods = new ArrayList<>(count);
        List<String> path = new ArrayList<>();
        for (int i = start; i < start + count; i += 1) {
            int depth = level.applyAsInt(i);
            // Python's `del path[level - 1:]`: keep what lies above this level.
            int keep = depth - 1 < 0 ? Math.max(0, path.size() + depth - 1) : Math.min(path.size(), depth - 1);
            path.subList(keep, path.size()).clear();
            path.add(String.valueOf(index.applyAsInt(i)));
            periods.add(new DashaPeriod(
                    String.join("/", path),
                    depth,
                    signed.test(i) ? Rashi.of(sign.applyAsInt(i)) : null,
                    Graha.of(lord.applyAsInt(i)),
                    new Interval(fromJd.applyAsDouble(i), toJd.applyAsDouble(i))));
        }
        return List.copyOf(periods);
    }

    /**
     * The periods running at a Julian day (UTC), from the mahadasha down.
     * Depth first order means a period's children follow it, so one walk
     * that takes the next level's running period finds the chain.
     */
    static List<DashaPeriod> chainAt(List<DashaPeriod> periods, double jd) {
        List<DashaPeriod> chain = new ArrayList<>();
        for (DashaPeriod period : periods) {
            if (period.level() == chain.size() + 1 && period.span().fromJd() <= jd && jd < period.span().toJd()) {
                chain.add(period);
            }
        }
        return List.copyOf(chain);
    }
}

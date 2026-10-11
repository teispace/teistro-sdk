package com.teispace.teistro;

import java.util.List;

/**
 * What the purifier of BPHS ch. 2 vv. 67–78 leaves standing of the window around a chart's
 * instant ({@code 03-design/rectification.md}).
 *
 * @param intervals The maximal runs no bar removed, in order. Under the {@code WEIGHT} rule every
 *     run is here, its verdict saying whether it is pure.
 * @param removed The runs a bar removed, in order: each one's verdict names every clause that
 *     failed.
 * @param edges Every instant inside the window where a clause changes, as Julian days (UTC).
 * @param grid The seed grid.
 */
public record Purified(
        List<Run> intervals,
        List<Run> removed,
        List<Double> edges,
        Grid grid) {
    /** The value, its lists copied and unmodifiable. */
    public Purified {
        intervals = List.copyOf(intervals);
        removed = List.copyOf(removed);
        edges = List.copyOf(edges);
    }

    /**
     * One run of the window between two edges, with what was judged in it.
     *
     * @param from Where it starts, a Julian day (UTC): the window's start or a clause edge.
     * @param to Where it ends: a clause edge or the window's end.
     * @param verdict The verdict every instant of it shares.
     */
    public record Run(
            double from,
            double to,
            Verdict verdict) {
    }

    /**
     * The grid that seeded the edges, so a run reproduces.
     *
     * @param stepDays The seed step, in days.
     * @param cells How many cells the window was cut into.
     */
    public record Grid(
            double stepDays,
            int cells) {
    }

    /**
     * What the purifier finds at an instant: every clause, and whether any held.
     *
     * @param clauses Every clause judged, in the order pranapada, Gulika, Moon.
     * @param pure Whether at least one counted clause held (v. 75).
     */
    public record Verdict(
            List<Clause> clauses,
            boolean pure) {
        /** The value, its lists copied and unmodifiable. */
        public Verdict {
            clauses = List.copyOf(clauses);
        }
    }

    /**
     * One test of the lagna against one point of one purifier.
     *
     * @param purifier The purifier read: {@code PRANAPADA}, {@code GULIKA} or {@code MOON}.
     * @param reference Which of its points: {@code ITSELF}, or v. 76's {@code SEVENTH},
     *     {@code NAVAMSHA} or {@code NAVAMSHA_SEVENTH} (Gulika only).
     * @param sign The sign that point stands in.
     * @param lagna The lagna's sign.
     * @param house The lagna's house counted from that sign, one to twelve.
     * @param held Whether the house is one that purifies this native.
     * @param counted Whether the clause counts toward the verdict: false only for v. 76's extension
     *     while the pranapada or the Moon holds, under {@code WHEN_TWO_FAIL}.
     */
    public record Clause(
            String purifier,
            String reference,
            Rashi sign,
            Rashi lagna,
            int house,
            boolean held,
            boolean counted) {
    }
}

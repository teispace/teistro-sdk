package com.teispace.teistro;

import java.util.List;

/**
 * A dasha of a founded chart: its periods, and for a nakshatra-seeded one its seed and balance at
 * birth. A sign-based dasha has neither, and its periods name their signs.
 *
 * @param system Which system: a {@code DashaSystem}, or a registered one by its full key as a
 *     {@code String} ({@code dasha_system.ACME_SAPTAKA}).
 * @param seed The nakshatra the Moon stood in, which seeds it; null for a sign-based dasha. May be
 *     null.
 * @param firstLord The lord it starts with.
 * @param overflow Whether the seed lay outside a conditional system's nakshatras.
 * @param balance What remained of the first period at birth; null for a sign-based dasha, whose
 *     first period runs whole from birth. May be null.
 * @param moonSpan The Moon's stay in its nakshatra, when the balance read one. May be null.
 * @param depth How many levels the periods go down.
 * @param periods Every period of the birth cycle to {@code depth}, depth first in time order: a
 *     mahadasha, then its antardashas and theirs, then the next.
 */
public record Dasha(
        Object system,
        Nakshatra seed,
        Graha firstLord,
        boolean overflow,
        DashaBalance balance,
        Interval moonSpan,
        int depth,
        List<DashaPeriod> periods) {
    /** The value, its lists copied and unmodifiable. */
    public Dasha {
        periods = List.copyOf(periods);
    }

    /**
     * The periods running at a Julian day (UTC), from the mahadasha down to
     * {@code depth}; empty before birth and past the end of the cycle.
     *
     * @param jd the instant, a UTC Julian day
     * @return the running chain, outermost first
     */
    public List<DashaPeriod> at(double jd) {
        return DashaReads.chainAt(periods, jd);
    }
}

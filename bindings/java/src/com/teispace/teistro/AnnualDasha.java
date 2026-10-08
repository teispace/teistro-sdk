package com.teispace.teistro;

import java.util.List;

/**
 * One annual dasha of a year: its ring, the year it divides, and its periods
 * ({@code 03-design/annual-dashas.md}).
 *
 * @param system Which of the three: {@code PATYAYINI}, {@code MUDDA} or {@code VARSHA_YOGINI}.
 * @param seed The birth Moon's nakshatra, which seeds a nakshatra year; null for the Patyayini.
 *     May be null.
 * @param ring The lords the year runs round.
 * @param year The year: from its return to where the clock closes it, under the default clock the
 *     next return.
 * @param periods Every period to the rules' depth, depth first in time order; a period that runs
 *     for no time is not listed.
 */
public record AnnualDasha(
        DashaSystem system,
        Nakshatra seed,
        DashaRing ring,
        Interval year,
        List<DashaPeriod> periods) {
    /** The value, its lists copied and unmodifiable. */
    public AnnualDasha {
        periods = List.copyOf(periods);
    }

    /**
     * The lord the year opens with.
     *
     * @return the lord
     */
    public Graha firstLord() {
        return ring.shares().get(ring.first()).lord();
    }

    /**
     * The periods running at a Julian day (UTC), from the mahadasha down;
     * empty outside the year.
     *
     * @param jd the instant, a UTC Julian day
     * @return the running chain, outermost first
     */
    public List<DashaPeriod> at(double jd) {
        return DashaReads.chainAt(periods, jd);
    }
}

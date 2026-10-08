package com.teispace.teistro;

/**
 * A Tajika strength, exact. The boundary carries it as an integer count of <b>sub-sub units</b>,
 * 3600 to a unit, because two office-bearers a sub-sub unit apart decide a year between them.
 *
 * @param units Whole units, at most twenty: the figure a reader compares.
 * @param subUnits The sub-units after those, 0 to 59.
 * @param subSub The sub-sub units after those, 0 to 59.
 * @param total The whole of it in sub-sub units: what to compare and sum.
 */
public record Bala(
        int units,
        int subUnits,
        int subSub,
        int total) {
    /**
     * The strength as the sources write one: {@code 14:20:15}.
     *
     * @return the units, sub-units and sub-sub units
     */
    @Override
    public String toString() {
        return String.format("%02d:%02d:%02d", units, subUnits, subSub);
    }
}

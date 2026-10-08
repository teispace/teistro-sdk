package com.teispace.teistro;

import java.util.List;

/**
 * What the lagna's lordships make of one graha.
 *
 * @param graha Which graha.
 * @param houses The houses it lords, counted from the lagna.
 * @param clauses The clauses that made its nature.
 * @param nature {@code YOGAKARAKA}, {@code BENEFIC}, {@code NEUTRAL} or {@code MALEFIC}.
 */
public record FunctionalRow(
        Graha graha,
        List<Integer> houses,
        List<FunctionalClause> clauses,
        String nature) {
    /** The value, its lists copied and unmodifiable. */
    public FunctionalRow {
        houses = List.copyOf(houses);
        clauses = List.copyOf(clauses);
    }
}

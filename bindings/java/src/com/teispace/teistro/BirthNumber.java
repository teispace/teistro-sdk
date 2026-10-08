package com.teispace.teistro;

import java.util.List;

/**
 * Balliett's birth number.
 *
 * @param month the month, reduced
 * @param day the day, reduced
 * @param year the year, reduced
 * @param sum the parts that are not masters, added and reduced; may be
 *     null, when every part is a master
 * @param apart the parts that are masters, which stand apart from the sum
 *     (Balliett p. 90), in the order month, day, year
 */
public record BirthNumber(Reduction month, Reduction day, Reduction year, Reduction sum, List<Integer> apart) {
    /** Keeps the list unmodifiable. */
    public BirthNumber {
        apart = List.copyOf(apart);
    }
}

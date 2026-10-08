package com.teispace.teistro;

import java.util.List;

/**
 * One region of a drawn chart.
 *
 * @param outline The region's outline in the unit square.
 * @param sign The sign the cell shows; for a house between cusps, its cusp's sign.
 * @param house The house the cell shows, 1 to 12.
 * @param lagna Whether the lagna stands in this cell.
 * @param ring The ring, innermost 0; a grid's cells are all 0.
 * @param label Where the sign or house number is drawn.
 * @param anchor Where the cell's bodies are stacked about.
 * @param bodies The bodies in the cell, as catalogue keys ({@code graha.SUN}).
 */
public record DrawnCell(
        Outline outline,
        Rashi sign,
        int house,
        boolean lagna,
        int ring,
        UnitPoint label,
        UnitPoint anchor,
        List<String> bodies) {
    /** The value, its lists copied and unmodifiable. */
    public DrawnCell {
        bodies = List.copyOf(bodies);
    }
}

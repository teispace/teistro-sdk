package com.teispace.teistro;

import java.util.List;

/**
 * The eclipses whose greatest moment falls in an almanac's days, each kind in order.
 *
 * @param lunar the lunar eclipses
 * @param solar the solar eclipses
 */
public record EclipsesFound(List<LunarEclipseHere> lunar, List<SolarEclipseHere> solar) {
    /** Keeps the lists unmodifiable. */
    public EclipsesFound {
        lunar = List.copyOf(lunar);
        solar = List.copyOf(solar);
    }
}

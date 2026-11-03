package com.teispace.teistro;

import java.util.List;

import com.teispace.teistro.record.Provenance;

/**
 * A chart's instant recast at each place asked.
 *
 * @param instant the instant, a Julian day (UTC)
 * @param obliquityDeg the obliquity of the ecliptic at it
 * @param places each place, in the order asked
 * @param provenance what computed them, and under what
 */
public record Relocation(double instant, double obliquityDeg, List<RelocatedChart> places, Provenance provenance) {
    /** Keeps the places unmodifiable. */
    public Relocation {
        places = List.copyOf(places);
    }
}

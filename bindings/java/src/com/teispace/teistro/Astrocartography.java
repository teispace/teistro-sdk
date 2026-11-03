package com.teispace.teistro;

import java.util.List;

import com.teispace.teistro.record.Provenance;

/**
 * The astrocartography lines at an instant, and what they were drawn under
 * ({@code 03-design/astrocartography.md}).
 *
 * @param instant the instant, a Julian day (UTC)
 * @param projection {@code IN_MUNDO} or {@code ZODIACAL}
 * @param stepDeg the step of latitude
 * @param latitudeLimitDeg how far north and south the lines run
 * @param toleranceDeg how close every point is to its angle
 * @param greenwichRamcDeg the right ascension of the meridian at Greenwich
 * @param obliquityDeg the obliquity of the ecliptic
 * @param lines each body's four lines, the bodies in the order asked
 * @param provenance what computed them, and under what
 */
public record Astrocartography(
        double instant,
        String projection,
        double stepDeg,
        double latitudeLimitDeg,
        double toleranceDeg,
        double greenwichRamcDeg,
        double obliquityDeg,
        List<AstroLine> lines,
        Provenance provenance) {
    /** Keeps the lines unmodifiable. */
    public Astrocartography {
        lines = List.copyOf(lines);
    }
}

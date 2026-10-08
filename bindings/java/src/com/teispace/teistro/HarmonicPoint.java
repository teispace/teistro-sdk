package com.teispace.teistro;

/**
 * A point of a harmonic chart: a planet, the ascendant or the midheaven.
 *
 * @param point {@code "GRAHA"}, {@code "ASCENDANT"} or {@code "MIDHEAVEN"}
 * @param graha which planet, for a {@code "GRAHA"} point; may be null, for an angle
 */
public record HarmonicPoint(String point, Graha graha) {
    /** The ascendant. */
    public static final HarmonicPoint ASCENDANT = new HarmonicPoint("ASCENDANT", null);

    /** The midheaven. */
    public static final HarmonicPoint MIDHEAVEN = new HarmonicPoint("MIDHEAVEN", null);

    /**
     * A planet's point.
     *
     * @param graha the planet
     * @return its point
     */
    public static HarmonicPoint graha(Graha graha) {
        return new HarmonicPoint("GRAHA", graha);
    }

    /**
     * A harmonic chart's point from its two cells: 0 and a graha's id for a
     * planet, 1 for the ascendant and 2 for the midheaven.
     */
    static HarmonicPoint of(int angle, int graha) {
        return switch (angle) {
            case 0 -> graha(Graha.of(graha));
            case 1 -> ASCENDANT;
            case 2 -> MIDHEAVEN;
            default -> throw Reads.internal("a harmonic point's angle is 0, 1 or 2, and the blob holds " + angle);
        };
    }
}

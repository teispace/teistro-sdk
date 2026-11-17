package com.teispace.teistro;

/**
 * The Sun's and the Moon's sidereal longitudes at an instant as the panchanga reads them
 * ({@code 03-design/panchanga-reach.md} §3).
 *
 * @param at the instant, a UTC Julian day
 * @param sunDeg the Sun's sidereal longitude, degrees
 * @param moonDeg the Moon's, degrees
 * @param elongationDeg the Moon less the Sun: the tithi's and the karana's angle
 * @param yogaDeg the Moon plus the Sun: the yoga's sum
 */
public record LimbAngles(double at, double sunDeg, double moonDeg, double elongationDeg, double yogaDeg) {

    /**
     * The tithi running, read off the elongation.
     *
     * @return the tithi
     */
    public Tithi tithi() {
        return Tithi.of(index(elongationDeg, 30));
    }

    /**
     * The Moon's nakshatra, read off her longitude.
     *
     * @return the nakshatra
     */
    public Nakshatra nakshatra() {
        return Nakshatra.of(index(moonDeg, 27));
    }

    /**
     * The yoga running, read off the sum.
     *
     * @return the yoga
     */
    public Yoga yoga() {
        return Yoga.of(index(yogaDeg, 27));
    }

    /** The arc of a limb of {@code count} equal arcs holding an angle; the last for one a rounding puts at 360. */
    private static int index(double degrees, int count) {
        return Math.min((int) Math.floor(degrees / (360.0 / count)), count - 1);
    }
}

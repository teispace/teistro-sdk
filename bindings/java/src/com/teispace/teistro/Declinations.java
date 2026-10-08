package com.teispace.teistro;

import java.util.List;
import java.util.Optional;

/**
 * A chart's distances from the equator, degrees north
 * ({@code 03-design/western-declinations.md}).
 *
 * @param obliquityDeg the true obliquity at the chart's instant, which turned every one
 * @param grahas the planets, in the catalogue's order
 * @param lagnaDeg the lagna's: the Sun's at that degree (Leo, p. 141)
 * @param midheavenDeg the midheaven's, read the same way
 */
public record Declinations(double obliquityDeg, List<Declined> grahas, double lagnaDeg, double midheavenDeg) {
    /** Keeps the list unmodifiable. */
    public Declinations {
        grahas = List.copyOf(grahas);
    }

    /**
     * One planet's declination, when the chart placed it.
     *
     * @param graha the planet
     * @return its declination, degrees north
     */
    public Optional<Double> graha(Graha graha) {
        return grahas.stream().filter(one -> one.graha() == graha).map(Declined::declinationDeg).findFirst();
    }
}

package com.teispace.teistro;

/**
 * One planet's dignities and its score.
 *
 * @param planet the planet
 * @param longitudeDeg degrees of the chart's zodiac
 * @param dignity the dignities and debilities it holds
 * @param peregrine in none of its five dignities, whatever its debilities
 * @param score its score from its own dignities alone
 * @param reception what Lilly's table adds for mutual reception (p. 115):
 *     the house's score when received by house, the exaltation's when by
 *     exaltation, nothing for a mixed reception or one by a lesser dignity
 *     (C210); a total is {@code score + reception}
 */
public record PlanetDignity(
        Graha planet, double longitudeDeg, EssentialDignity dignity, boolean peregrine, int score, int reception) {}

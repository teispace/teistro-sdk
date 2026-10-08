package com.teispace.teistro;

/**
 * A planet of a composite chart: the near midpoint of its two places,
 * degrees in the synastry's zodiac, moving at the mean of its two speeds,
 * degrees a day, negative when retrograde.
 *
 * @param graha the planet
 * @param longitudeDeg the near midpoint of its two places
 * @param speedDegPerDay the mean of its two speeds
 */
public record CompositePlanet(Graha graha, double longitudeDeg, double speedDegPerDay) {}

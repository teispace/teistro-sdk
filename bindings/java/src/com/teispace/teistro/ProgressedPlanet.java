package com.teispace.teistro;

/**
 * A planet of the progressed chart.
 *
 * @param graha the planet
 * @param longitudeDeg its longitude in the chart's zodiac
 * @param tropicalDeg its tropical longitude
 * @param speedDegPerDay degrees a day at the instant of sky; below zero when retrograde
 */
public record ProgressedPlanet(Graha graha, double longitudeDeg, double tropicalDeg, double speedDegPerDay) {}

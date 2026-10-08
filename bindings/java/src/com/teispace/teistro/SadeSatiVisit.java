package com.teispace.teistro;

/**
 * One stay of Saturn's in a house, half-open.
 *
 * @param from When Saturn entered, a UTC Julian day; null before the ephemeris's coverage. May be
 *     null.
 * @param to When it left; null after the ephemeris's coverage. May be null.
 */
public record SadeSatiVisit(
        Double from,
        Double to) {
}

package com.teispace.teistro;

/**
 * Two progressed planets exactly in aspect with each other: a line of Leo's
 * "P. to P." list (<i>The Progressed Horoscope</i>, p. 41).
 *
 * @param life the instant of life it falls due, a UTC Julian day
 * @param sky the instant of sky it is exact at
 * @param first the first planet, in the request's order
 * @param second the second
 * @param angle a whole degree 0 to 180
 */
public record MutualContact(double life, double sky, Graha first, Graha second, int angle) {}

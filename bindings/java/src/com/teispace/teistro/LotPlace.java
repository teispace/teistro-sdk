package com.teispace.teistro;

/**
 * Where a lot fell, in the chart's zodiac.
 *
 * @param longitudeDeg degrees in [0, 360)
 * @param sign the sign
 * @param lord the sign's lord, the lot's ruler
 * @param house 1 to 12, counted in whole signs from the ascendant's sign
 */
public record LotPlace(double longitudeDeg, Rashi sign, Graha lord, int house) {}

package com.teispace.teistro;

/**
 * Where a planet is counted among a chart's Western houses.
 *
 * @param graha the planet
 * @param house the house whose cusp it has passed and whose next cusp it has not
 * @param withAscendant whether Leo counts it with the ascendant (C250): in
 *     the first house, or above the ascendant no further than the degree
 *     that rose one sidereal hour before; its house is never moved for it
 */
public record WesternHousePlacement(Graha graha, int house, boolean withAscendant) {}

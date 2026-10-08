package com.teispace.teistro;

/**
 * What the Sun does to a body.
 *
 * @param burning How badly it burns.
 * @param fromSunDeg How far from the Sun it stands, degrees, or null when the chart carries no
 *     Sun, in which case nothing is burnt and this says why rather than claiming the sky is clear.
 *     May be null.
 * @param orbDeg Combust inside this, degrees, or null for a body that does not burn at all. May be
 *     null.
 * @param deepOrbDeg Deeply combust inside this, degrees, where the table gives one. May be null.
 */
public record Combustion(
        Burning burning,
        Double fromSunDeg,
        Double orbDeg,
        Double deepOrbDeg) {
}

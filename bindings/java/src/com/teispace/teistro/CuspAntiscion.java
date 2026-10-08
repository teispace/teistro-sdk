package com.teispace.teistro;

/**
 * A planet's reflection upon a cusp's very degree, its own sign and whole
 * degree (Lilly, <i>Christian Astrology</i>, p. 165; C251).
 *
 * @param graha the planet
 * @param house the house whose cusp it falls upon, 1 to 12
 * @param contrary whether it is the contrantiscion, the reflection about the equinoxes
 */
public record CuspAntiscion(Graha graha, int house, boolean contrary) {}

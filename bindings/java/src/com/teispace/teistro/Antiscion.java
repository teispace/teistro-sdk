package com.teispace.teistro;

/**
 * A planet's two reflections, tropical degrees
 * ({@code 03-design/western-antiscia.md}).
 *
 * @param graha the planet
 * @param antiscionDeg its reflection about the solstices: 180° less its longitude
 * @param contrantiscionDeg its reflection about the equinoxes: 360° less its longitude
 */
public record Antiscion(Graha graha, double antiscionDeg, double contrantiscionDeg) {}

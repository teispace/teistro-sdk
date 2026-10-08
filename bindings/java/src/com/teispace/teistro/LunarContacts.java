package com.teispace.teistro;

/**
 * A lunar eclipse's contacts with the penumbra and umbra, UT1 Julian days;
 * an umbral contact the eclipse never reaches is null.
 *
 * @param p1 the first penumbral contact
 * @param u1 the first umbral contact; may be null
 * @param u2 the start of totality; may be null
 * @param u3 the end of totality; may be null
 * @param u4 the last umbral contact; may be null
 * @param p4 the last penumbral contact
 */
public record LunarContacts(double p1, Double u1, Double u2, Double u3, Double u4, double p4) {}

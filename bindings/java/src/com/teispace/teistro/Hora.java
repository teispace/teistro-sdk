package com.teispace.teistro;

/**
 * One hora, from sunrise.
 *
 * @param number its number, 1 to 24
 * @param lord the graha that rules it
 * @param start when it begins, as a Julian day (UTC)
 * @param end when it ends
 */
public record Hora(int number, Graha lord, double start, double end) {}

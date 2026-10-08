package com.teispace.teistro;

/**
 * One of the thirty muhurtas.
 *
 * @param at when it runs
 * @param daylight whether it is one of the fifteen of the daylight
 */
public record Muhurta(Interval at, boolean daylight) {}

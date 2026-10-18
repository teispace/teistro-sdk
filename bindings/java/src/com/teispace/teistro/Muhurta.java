package com.teispace.teistro;

/**
 * One of the thirty muhurtas.
 *
 * @param at when it runs
 * @param daylight whether it is one of the fifteen of the daylight
 * @param durmuhurta whether the day's vara censures it under {@code panchanga.durmuhurta} (crux C398)
 */
public record Muhurta(Interval at, boolean daylight, boolean durmuhurta) {}

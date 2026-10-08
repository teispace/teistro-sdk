package com.teispace.teistro;

/**
 * What the baseline's score reads of the reference day's panchanga at sunrise.
 *
 * @param tithi the tithi
 * @param yoga the nitya yoga
 * @param muhurtaYogas how many muhurta yogas held
 */
public record RashifalPanchanga(Tithi tithi, Yoga yoga, int muhurtaYogas) {}

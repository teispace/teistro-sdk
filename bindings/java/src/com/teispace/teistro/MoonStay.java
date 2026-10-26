package com.teispace.teistro;

/**
 * The Moon's stay in the nakshatra it holds at a chart's instant, and the two counts a janma
 * patrika prints of it in ghatis of twenty-four minutes (C756): the bhayat, how much had passed,
 * and the bhabhoga, the whole stay, which can run past sixty ghatis.
 *
 * @param entered when the Moon entered the nakshatra, a UTC Julian day
 * @param leaves when it leaves it, a UTC Julian day
 * @param bhayat the stay passed at the instant
 * @param bhabhoga the whole stay
 */
public record MoonStay(double entered, double leaves, GhatiPala bhayat, GhatiPala bhabhoga) {}

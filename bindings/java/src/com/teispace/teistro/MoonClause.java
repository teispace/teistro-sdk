package com.teispace.teistro;

/**
 * The Moon's place and course (pp. 112, 122).
 *
 * @param sign her sign
 * @param degree degrees within it
 * @param late whether she is late in it
 * @param lateSign Gemini, Scorpio or Capricorn
 * @param viaCombusta Libra 15° to Scorpio 15°
 * @param course where she is going
 */
public record MoonClause(
        Rashi sign, double degree, boolean late, boolean lateSign, boolean viaCombusta, MoonCourse course) {}

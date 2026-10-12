package com.teispace.teistro;

/**
 * The sign rising at a chart's instant, and its span (C378).
 *
 * @param sign the sign
 * @param rose when it began to rise, a UTC Julian day found to a second; null when it was already
 *     rising a day before, which only a place inside a polar circle sees
 * @param sets when the next sign begins to rise, a UTC Julian day; null likewise
 */
public record RisingSign(Rashi sign, Double rose, Double sets) {}

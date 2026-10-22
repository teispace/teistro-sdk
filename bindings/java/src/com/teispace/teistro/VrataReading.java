package com.teispace.teistro;

/**
 * One reading of when a vrata's fast is broken
 * ({@code 03-design/festival-rules.md} §8.7).
 *
 * @param kind {@code TITHI_END} (after the tithi's end, and the
 *     nakshatra's when it joins it; in the morning past {@code yamas}),
 *     {@code WITHIN} (inside the tithi, within {@code yamas} of the
 *     daylight), {@code SUNRISE}, or {@code AFTER_WINDOW} (after
 *     {@code at} on the fast day)
 * @param nakshatra the nakshatra whose end also counts; null when none
 * @param yamas yamas of the daylight, 1 to 4; null when the reading has none
 * @param at the window an {@code AFTER_WINDOW} follows; null otherwise
 */
public record VrataReading(String kind, Nakshatra nakshatra, Integer yamas, FestivalWindow at) {}

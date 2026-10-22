package com.teispace.teistro;

/**
 * One reading of when a vrata's fast is broken
 * ({@code 03-design/festival-rules.md} §8.7).
 *
 * @param kind {@code TITHI_END} (after the tithi's end, and the
 *     nakshatra's when it joins it; in the morning past {@code yamas}),
 *     {@code WITHIN} (inside the tithi, within {@code yamas} of the
 *     daylight), {@code SUNRISE}, {@code AFTER_WINDOW} (after
 *     {@code at} on the fast day), {@code NEXT_TITHI} (the first daylight
 *     of the tithi after the fast's), or {@code AFTER_RITE} (after the
 *     rule {@code rite}'s closing rite; with {@code remnant} ghatis, inside
 *     the fast's tithi left at the next sunrise when the rite falls a day
 *     later)
 * @param nakshatra the nakshatra whose end also counts; null when none
 * @param yamas yamas of the daylight, 1 to 4; null when the reading has none
 * @param at the window an {@code AFTER_WINDOW} follows; null otherwise
 * @param rite the rule key an {@code AFTER_RITE} follows; null otherwise
 * @param remnant the ghatis of an {@code AFTER_RITE}'s remnant, 0 to 60;
 *     null when it always waits for the rite
 */
public record VrataReading(
        String kind, Nakshatra nakshatra, Integer yamas, FestivalWindow at, String rite, Integer remnant) {
    /** A reading that follows no rite. */
    public VrataReading(String kind, Nakshatra nakshatra, Integer yamas, FestivalWindow at) {
        this(kind, nakshatra, yamas, at, null, null);
    }
}

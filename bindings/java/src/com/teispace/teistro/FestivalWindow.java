package com.teispace.teistro;

/**
 * A rite's window as a rule spells it ({@code 03-design/festival-rules.md}
 * §7).
 *
 * @param window {@code SUNRISE}, {@code SUNSET}, {@code PART},
 *     {@code PRADOSHA}, {@code NISHITHA}, {@code NIGHT_MUHURTA},
 *     {@code DAY_MUHURTA} or {@code MOONRISE}
 * @param part the fifth of the daylight for a {@code PART}; null otherwise
 * @param muhurta the night's muhurta for a {@code NIGHT_MUHURTA}, 1 to 15;
 *     null otherwise
 * @param fromMuhurta the first muhurta of a {@code DAY_MUHURTA}, 1 to 15;
 *     null otherwise
 * @param toMuhurta its last; null otherwise
 */
public record FestivalWindow(String window, String part, Integer muhurta, Integer fromMuhurta,
        Integer toMuhurta) {}

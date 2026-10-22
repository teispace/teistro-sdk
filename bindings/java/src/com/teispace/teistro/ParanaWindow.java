package com.teispace.teistro;

/**
 * A daylight window a parana is kept in, as a rule spells it
 * ({@code 03-design/festival-rules.md} §8.5).
 *
 * @param window {@code PART} or {@code DAY_MUHURTA}
 * @param part the fifth of the daylight for a {@code PART}, {@code PRATAH},
 *     {@code APARAHNA} and the rest; null otherwise
 * @param fromMuhurta the first muhurta of a {@code DAY_MUHURTA}, 1 to 15; null otherwise
 * @param toMuhurta its last; null otherwise
 */
public record ParanaWindow(String window, String part, Integer fromMuhurta, Integer toMuhurta) {}

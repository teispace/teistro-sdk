package com.teispace.teistro;

import java.util.List;

/**
 * A graha in the 12th from the karakamsha or the amatya, or joined to the amatya, the deities its
 * verse names (in the 1923 print's ch. 9), and whether Ketu stands with it (C355).
 *
 * @param graha The graha.
 * @param deities The deities its verse names.
 * @param verse The verse.
 * @param withKetu Whether Ketu stands with it.
 */
public record Devotion(
        Graha graha,
        List<String> deities,
        int verse,
        boolean withKetu) {
    /** The value, its lists copied and unmodifiable. */
    public Devotion {
        deities = List.copyOf(deities);
    }
}

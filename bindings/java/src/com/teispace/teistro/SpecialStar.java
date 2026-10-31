package com.teispace.teistro;

import java.util.Optional;

/**
 * A star the navatara chakra names besides its tara, by its count from the
 * birth star (<i>Muhurta Chintamani</i>, Nakshatra-prakarana v. 43;
 * {@code 03-design/navatara-chakra.md}).
 */
public enum SpecialStar {
    /** The 1st, the birth star itself. */
    JANMA,
    /** The 10th. */
    KARMA,
    /** The 16th. */
    SANGHATIKA,
    /** The 18th. */
    SAMUDAYA,
    /** The 23rd. */
    VAINASHIKA,
    /** The 25th. */
    MANASA;

    /**
     * The key the wire spells it with.
     *
     * @return the key, such as {@code "KARMA"}
     */
    public String key() {
        return name();
    }

    /**
     * The special star a key names, if any does.
     *
     * @param key the key
     * @return the special star
     */
    public static Optional<SpecialStar> byKey(String key) {
        for (SpecialStar star : values()) {
            if (star.name().equals(key)) {
                return Optional.of(star);
            }
        }
        return Optional.empty();
    }
}

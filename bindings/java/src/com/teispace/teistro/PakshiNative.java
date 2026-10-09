package com.teispace.teistro;

import java.util.Objects;

/**
 * Whose bird a Pancha Pakshi reading follows ({@code 03-design/pakshi.md}): the bird itself, or the one
 * a birth star and paksha give under a rule (P1).
 */
public sealed interface PakshiNative permits PakshiNative.Bird, PakshiNative.Star {
    /**
     * A native named by its bird.
     *
     * @param bird {@code VULTURE}, {@code OWL}, {@code CROW}, {@code COCK} or {@code PEACOCK}
     * @return the native
     */
    static PakshiNative bird(String bird) {
        return new Bird(bird);
    }

    /**
     * A native by birth star and paksha, the dark half reversing the birds.
     *
     * @param nakshatra the Moon's nakshatra at birth
     * @param paksha the paksha at birth
     * @return the native
     */
    static PakshiNative star(Nakshatra nakshatra, Paksha paksha) {
        return new Star(nakshatra, paksha, "BY_PAKSHA");
    }

    /**
     * The bird named outright.
     *
     * @param bird the bird's key
     */
    record Bird(String bird) implements PakshiNative {
        /** The value, its bird required. */
        public Bird {
            Objects.requireNonNull(bird, "bird");
        }
    }

    /**
     * The bird of a birth star in a paksha.
     *
     * @param nakshatra the Moon's nakshatra at birth
     * @param paksha the paksha at birth
     * @param rule {@code BY_PAKSHA} (the dark half reverses the birds) or {@code SINGLE} (one table)
     */
    record Star(Nakshatra nakshatra, Paksha paksha, String rule) implements PakshiNative {
        /** The value, every part required. */
        public Star {
            Objects.requireNonNull(nakshatra, "nakshatra");
            Objects.requireNonNull(paksha, "paksha");
            Objects.requireNonNull(rule, "rule");
        }
    }
}

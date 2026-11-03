package com.teispace.teistro;

import java.util.Optional;

/** What an aayadi reckoning's remainder names ({@code 03-design/aayadi.md}). */
public enum AayadiQuantity {
    /** The income. */
    AAYA,
    /** The expense. */
    VYAYA,
    /** The eight from the banner. */
    YONI,
    /** The house's star. */
    NAKSHATRA,
    /** The weekday. */
    VARA,
    /** The tithi. */
    TITHI,
    /** <i>Muhurta Chintamani</i>'s share. */
    AMSHA,
    /** Wealth. */
    DHANA,
    /** Debt. */
    RINA,
    /** The yoga. */
    YOGA,
    /** The age. */
    AYU;

    /**
     * The key the wire spells it with.
     *
     * @return the key, such as {@code "AAYA"}
     */
    public String key() {
        return name();
    }

    /**
     * The member a key names, if any does.
     *
     * @param key the key
     * @return the member
     */
    public static Optional<AayadiQuantity> byKey(String key) {
        for (AayadiQuantity member : values()) {
            if (member.name().equals(key)) {
                return Optional.of(member);
            }
        }
        return Optional.empty();
    }

    static AayadiQuantity read(Object raw) {
        String key = Reads.string(raw);
        return byKey(key).orElseThrow(() -> Reads.internal("the library drew a AayadiQuantity this build does not know: " + key));
    }
}

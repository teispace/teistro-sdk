package com.teispace.teistro;

import java.util.Optional;

/** A condition a vastu text judges a measure by ({@code 03-design/aayadi.md}). */
public enum AayadiClauseKey {
    /** The yoni or eightfold aaya is a good one. */
    YONI_GOOD,
    /** The aaya's count above the vyaya's. */
    AAYA_EXCEEDS_VYAYA,
    /** A good weekday. */
    VARA_GOOD,
    /** <i>Samarangana</i>'s lesser vyaya. */
    VYAYA_LESS_THAN_AAYA,
    /** <i>Samarangana</i>'s vyaya is not none. */
    VYAYA_PRESENT,
    /** The tara is not the 3rd, 5th or 7th. */
    TARA_NOT_AVOIDED,
    /** The house's gana the owner's. */
    GANA_SAME,
    /** An odd aaya. */
    AAYA_ODD,
    /** Wealth above debt. */
    DHANA_EXCEEDS_RINA,
    /** Not a rikta tithi. */
    TITHI_NOT_RIKTA,
    /** The house's star is not the owner's own. */
    OWNER_STAR_DIFFERS;

    /**
     * The key the wire spells it with.
     *
     * @return the key, such as {@code "YONI_GOOD"}
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
    public static Optional<AayadiClauseKey> byKey(String key) {
        for (AayadiClauseKey member : values()) {
            if (member.name().equals(key)) {
                return Optional.of(member);
            }
        }
        return Optional.empty();
    }

    static AayadiClauseKey read(Object raw) {
        String key = Reads.string(raw);
        return byKey(key).orElseThrow(() -> Reads.internal("the library drew a AayadiClauseKey this build does not know: " + key));
    }
}

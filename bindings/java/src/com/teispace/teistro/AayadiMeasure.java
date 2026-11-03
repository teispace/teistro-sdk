package com.teispace.teistro;

import java.util.Optional;

/** The measure an aayadi reckoning starts from ({@code 03-design/aayadi.md}). */
public enum AayadiMeasure {
    /** The length and the breadth together. */
    SUM,
    /** The length. */
    LENGTH,
    /** The breadth. */
    BREADTH,
    /** Twice the sum. */
    PERIMETER,
    /** The length times the breadth. */
    AREA,
    /** The house's nakshatra count (the vyaya of <i>Samarangana</i> and <i>Muhurta Chintamani</i>). */
    HOUSE_STAR;

    /**
     * The key the wire spells it with.
     *
     * @return the key, such as {@code "SUM"}
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
    public static Optional<AayadiMeasure> byKey(String key) {
        for (AayadiMeasure member : values()) {
            if (member.name().equals(key)) {
                return Optional.of(member);
            }
        }
        return Optional.empty();
    }

    static AayadiMeasure read(Object raw) {
        String key = Reads.string(raw);
        return byKey(key).orElseThrow(() -> Reads.internal("the library drew a AayadiMeasure this build does not know: " + key));
    }
}

package com.teispace.teistro;

import java.util.Optional;

/**
 * Where the largest phase of an eclipse a place sees falls, and so why it is or is not the eclipse's
 * maximum there ({@code 03-design/eclipses.md} §4.5, crux C1980).
 */
public enum SeenAt {
    /** The body is up at the maximum, so the maximum is seen. */
    MAXIMUM,
    /** The body rises after the maximum: the eclipse rises in progress, its largest phase seen at sunrise or moonrise. */
    RISING,
    /** The body sets before the maximum: the largest phase seen is at sunset or moonset. */
    SETTING,
    /** As {@link #RISING}, against a custom altitude limit rather than a horizon. */
    RISING_PAST_LIMIT,
    /** As {@link #SETTING}, against a custom altitude limit. */
    SETTING_PAST_LIMIT,
    /** The body is never up while the eclipse lasts: nothing is seen. */
    NOT_SEEN;

    /**
     * The key the wire spells it with.
     *
     * @return the key, such as {@code "NOT_SEEN"}
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
    public static Optional<SeenAt> byKey(String key) {
        for (SeenAt member : values()) {
            if (member.name().equals(key)) {
                return Optional.of(member);
            }
        }
        return Optional.empty();
    }

    static SeenAt read(Object raw) {
        String key = Reads.string(raw);
        return byKey(key).orElseThrow(() -> Reads.internal("the library drew a SeenAt this build does not know: " + key));
    }
}

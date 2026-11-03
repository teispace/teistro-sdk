package com.teispace.teistro;

import java.util.Optional;

/** What a vastu text asks of a measure that the reckoning is not given ({@code 03-design/aayadi.md}). */
public enum AayadiUnjudged {
    /** The owner's and the site's stars (<i>Mayamata</i> IX v. 19). */
    OWNER_STARS,
    /** The nakshatra against the owner's (<i>Manasara</i> IX). */
    NAKSHATRA_AGAINST_OWNER,
    /** Indra, Yama or Raja (crux C1196). */
    AMSHA,
    /** The house's name against the owner's. */
    HOUSE_NAME,
    /** The signs of the two stars (crux C1198). */
    SIGNS,
    /** <i>Samarangana</i>'s life (crux C1197). */
    LIFE,
    /** The six good karanas. */
    KARANAS,
    /** The new moon among the fifteen tithis. */
    NEW_MOON,
    /** The yogas the gloss does not list. */
    YOGAS;

    /**
     * The key the wire spells it with.
     *
     * @return the key, such as {@code "OWNER_STARS"}
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
    public static Optional<AayadiUnjudged> byKey(String key) {
        for (AayadiUnjudged member : values()) {
            if (member.name().equals(key)) {
                return Optional.of(member);
            }
        }
        return Optional.empty();
    }

    static AayadiUnjudged read(Object raw) {
        String key = Reads.string(raw);
        return byKey(key).orElseThrow(() -> Reads.internal("the library drew a AayadiUnjudged this build does not know: " + key));
    }
}

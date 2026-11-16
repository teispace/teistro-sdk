package com.teispace.teistro;

import java.util.Optional;

/**
 * Which month rule placed an observance in a year with an adhika or a
 * kshaya month (SR-93, {@code 03-design/festival-malamasa.md}).
 */
public enum MalamasaClause {
    /** The year doubles the rule's month and keeps it in the nija month, after the adhika one. */
    NIJA,
    /** Kept in the adhika month, by the rule's own {@code inAdhika}, which its source cites. */
    ADHIKA,
    /** In a kshaya lunation, under the first of the two names it joins. */
    KSHAYA_FIRST,
    /** In a kshaya lunation, under the second of its two names, whose festivals keep their tithis in it. */
    KSHAYA_SECOND;

    /**
     * The key the wire spells it with.
     *
     * @return the key, such as {@code "KSHAYA_SECOND"}
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
    public static Optional<MalamasaClause> byKey(String key) {
        for (MalamasaClause member : values()) {
            if (member.name().equals(key)) {
                return Optional.of(member);
            }
        }
        return Optional.empty();
    }

    static MalamasaClause read(Object raw) {
        String key = Reads.string(raw);
        return byKey(key).orElseThrow(() -> Reads.internal("the library drew a MalamasaClause this build does not know: " + key));
    }
}

package com.teispace.teistro;

/**
 * What an event study's null keeps ({@code 03-design/research.md} §1.4). None is a default: each
 * keeps different margins, and a dasha reads the age that a date shuffle moves.
 */
public enum ResearchEventShuffle {
    /** Event dates move among people: the calendar of events is kept, each person's age is not. */
    EVENT_DATES,
    /** Ages at the event move among people: the ages are kept, the calendar is not. */
    AGES_AT_EVENT,
    /**
     * Each person is their own control, read at their own {@link ResearchReferents}: nobody's event moves
     * to anybody else. Asked by the {@code timed} that takes referents.
     */
    SELF_MATCHED;

    /**
     * The spelling the record carries.
     *
     * @return the key, as {@code "AGES_AT_EVENT"}
     */
    public String key() {
        return name();
    }
}

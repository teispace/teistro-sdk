package com.teispace.teistro;

/**
 * What an event study's null keeps ({@code 03-design/research.md} §1.4). Neither is a default: the two
 * keep different margins, and a dasha reads the age that a date shuffle moves.
 */
public enum ResearchEventShuffle {
    /** Event dates move among people: the calendar of events is kept, each person's age is not. */
    EVENT_DATES,
    /** Ages at the event move among people: the ages are kept, the calendar is not. */
    AGES_AT_EVENT;

    /**
     * The spelling the record carries.
     *
     * @return the key, as {@code "AGES_AT_EVENT"}
     */
    public String key() {
        return name();
    }
}

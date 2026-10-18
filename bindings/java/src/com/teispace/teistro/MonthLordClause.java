package com.teispace.teistro;

/** A clause of vv. 12–13 about one of the month's lords ({@code 03-design/varshesha.md}). */
public enum MonthLordClause {
    /** In the 6th, 8th or 12th from the month lagna. */
    TRIKA,
    /** Sharing a sign with a malefic: the Sun, Mars, Saturn or the waning Moon. */
    WITH_MALEFIC,
    /** Aspected by a malefic with an inimical aspect. */
    MALEFIC_INIMICAL,
    /** Strong, by dignity or Vishwa bala. */
    STRONG,
    /** In a kendra, a trikona or the 11th. */
    WELL_PLACED;

    /**
     * The spelling the section carries.
     *
     * @return the key, as {@code "WITH_MALEFIC"}
     */
    public String key() {
        return name();
    }
}

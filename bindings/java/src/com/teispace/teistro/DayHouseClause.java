package com.teispace.teistro;

/** A clause of vv. 20–21 about one house's navamsha lord ({@code 03-design/varshesha.md}). */
public enum DayHouseClause {
    /** Sharing a sign with a benefic. */
    WITH_BENEFIC,
    /** Aspected by a benefic with a friendly aspect. */
    BENEFIC_FRIENDLY,
    /** Aspected by the Moon with a friendly aspect. */
    MOON_FRIENDLY,
    /** Sharing a sign with a malefic. */
    WITH_MALEFIC,
    /** Aspected by a malefic with an inimical aspect. */
    MALEFIC_INIMICAL;

    /**
     * The spelling the section carries.
     *
     * @return the key, as {@code "WITH_BENEFIC"}
     */
    public String key() {
        return name();
    }
}

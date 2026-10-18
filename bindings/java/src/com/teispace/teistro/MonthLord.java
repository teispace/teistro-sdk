package com.teispace.teistro;

/** The month's lords its results read ({@code 03-design/varshesha.md}, "The month's results"). */
public enum MonthLord {
    /** The lord of the year chart's lagna. */
    VARSHA_LAGNA,
    /** The Masesha, the month's lord. */
    MASESHA,
    /** The Varshesha, the year's lord. */
    VARSHESHA,
    /** The lord of the month chart's lagna. */
    MASA_LAGNA,
    /** The lord of the Muntha's sign this month. */
    MUNTHA;

    /**
     * The spelling the section carries.
     *
     * @return the key, as {@code "VARSHA_LAGNA"}
     */
    public String key() {
        return name();
    }
}

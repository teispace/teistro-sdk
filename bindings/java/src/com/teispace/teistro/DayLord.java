package com.teispace.teistro;

/** The day's lords v. 19 judges ({@code 03-design/varshesha.md}). */
public enum DayLord {
    /** The day's lord. */
    DINESHA,
    /** The year's lord. */
    VARSHESHA,
    /** The month's lord. */
    MASESHA,
    /** The lord of the Muntha's sign this day. */
    MUNTHA;

    /**
     * The spelling the section carries.
     *
     * @return the key, as {@code "DINESHA"}
     */
    public String key() {
        return name();
    }
}

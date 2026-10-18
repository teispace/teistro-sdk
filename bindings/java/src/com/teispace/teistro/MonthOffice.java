package com.teispace.teistro;

/**
 * The six portfolios of a month, in the verse's order ({@code 03-design/varshesha.md}, "The
 * month's lord"; <i>Tajika Nilakanthi</i>, Varsha Tantra, vv. 3–5).
 */
public enum MonthOffice {
    /** The lord of the month chart's lagna. */
    MASA_LAGNA,
    /** The lord of the Muntha's sign, the Muntha moved on 2°30′ a month. */
    MUNTHA,
    /** The lord of the birth chart's lagna. */
    JANMA_LAGNA,
    /** The month lagna's triplicity lord, by day or by night. */
    TRI_RASHI,
    /** The lord of the Sun's sign by day, of the Moon's by night. */
    DINA_RATRI,
    /** The lord of the year chart's lagna. */
    VARSHA_LAGNA;

    /**
     * The spelling the section carries.
     *
     * @return the key, as {@code "MASA_LAGNA"}
     */
    public String key() {
        return name();
    }
}

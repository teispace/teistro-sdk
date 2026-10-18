package com.teispace.teistro;

/**
 * A clause of vv. 7–11 about one house of a month (<i>Tajika Nilakanthi</i>, Varsha Tantra, the
 * month's results; {@code 03-design/varshesha.md}).
 */
public enum MonthHouseClause {
    /** The house's navamsha lord aspected by the lord's navamsha lord with a friendly aspect, or sharing its sign. */
    FRIENDLY,
    /** The Moon aspecting both with a friendly aspect. */
    MOON_FRIENDLY,
    /** The two in an inimical aspect, the 4th, 7th or 10th. */
    INIMICAL,
    /** The Moon aspecting both with an inimical aspect. */
    MOON_INIMICAL,
    /** One debilitated, the other combust. */
    LOW_AND_SET,
    /** Both debilitated. */
    BOTH_LOW,
    /** Both combust. */
    BOTH_SET,
    /** The two one planet, so no clause about the pair holds. */
    ONE_PLANET,
    /** The house's navamsha lord strong. */
    STRONG,
    /** The house's navamsha lord weak. */
    WEAK;

    /**
     * The spelling the section carries.
     *
     * @return the key, as {@code "MOON_FRIENDLY"}
     */
    public String key() {
        return name();
    }
}

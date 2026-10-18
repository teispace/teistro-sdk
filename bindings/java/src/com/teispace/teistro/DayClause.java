package com.teispace.teistro;

/** A clause of vv. 18 and 26–29 about the day's chart as a whole ({@code 03-design/varshesha.md}). */
public enum DayClause {
    /** Every benefic strong in a kendra, a trikona or the 11th from the lagna. */
    BENEFICS_WELL_PLACED_FROM_LAGNA,
    /** The same from the Moon, she not counted. */
    BENEFICS_WELL_PLACED_FROM_MOON,
    /** Every malefic in the 3rd, 6th or 11th from the lagna. */
    MALEFICS_WELL_PLACED_FROM_LAGNA,
    /** The same from the Moon, she not counted. */
    MALEFICS_WELL_PLACED_FROM_MOON,
    /** Malefics in both the 2nd and the 12th. */
    MALEFICS_FLANK,
    /** A benefic in the 12th. */
    BENEFIC_TWELFTH,
    /** A direct malefic in the 12th and a retrograde one in the 2nd. */
    PAPA_KARTARI,
    /** The same of benefics. */
    SHUBHA_KARTARI,
    /** The waning Moon in the 1st or 8th, joined or aspected by a malefic. */
    WANING_MOON_AFFLICTED,
    /** The Moon with Mars in the 6th or 8th. */
    MOON_MARS_DUSTHANA,
    /** A malefic in the 4th. */
    MALEFIC_FOURTH,
    /** A benefic in the 7th. */
    BENEFIC_SEVENTH,
    /** A benefic in the 2nd. */
    BENEFIC_SECOND,
    /** A benefic in the 9th. */
    BENEFIC_NINTH;

    /**
     * The spelling the section carries.
     *
     * @return the key, as {@code "BENEFICS_WELL_PLACED_FROM_LAGNA"}
     */
    public String key() {
        return name();
    }
}

package com.teispace.teistro;

import java.util.List;
import java.util.Optional;

import com.teispace.teistro.record.Provenance;

/**
 * A range's calendar cells ({@code 03-design/month-cells.md}): what a month
 * grid prints of each date.
 *
 * @param cells one cell a date, in order
 * @param observances the observances the cells' {@code festivals} name; empty
 *     when none were asked
 * @param provenance what computed it, and the clock it was asked on
 */
public record DayCells(List<Cell> cells, Optional<FestivalAnswer> observances, Provenance provenance) {
    /** Copies the cells. */
    public DayCells {
        cells = List.copyOf(cells);
    }

    /** How a cell's tithis stand against its sunrises. */
    public enum Kind {
        /** The sunrise tithi is not the one the sunrise before held, and none is skipped. */
        PLAIN,
        /** A tithi began after this sunrise and ended before the next: it names no date. */
        KSHAYA,
        /** The tithi at this sunrise held the sunrise before as well: the second of two dates it names. */
        VRIDDHI,
        /** Both at once. */
        VRIDDHI_AND_KSHAYA;

        /**
         * The kind an answer spells as {@code key}.
         *
         * @param key the wire spelling
         * @return the kind
         */
        static Kind of(String key) {
            for (Kind kind : values()) {
                if (kind.name().equals(key)) {
                    return kind;
                }
            }
            throw Reads.internal("the library drew a cell kind this build does not know: " + key);
        }
    }

    /**
     * A tithi that ends after a cell's sunrise and at or before the next.
     *
     * @param tithi the tithi
     * @param at the instant it ends, a UTC Julian day
     * @param kshaya whether it also began after the cell's sunrise: a kshaya
     *     tithi, which names no date
     */
    public record TithiEnd(Tithi tithi, double at, boolean kshaya) {
    }

    /**
     * What a month grid prints of one date.
     *
     * @param date the civil date, in the calendar asked
     * @param vara its weekday, from this sunrise to the next
     * @param sunrise its sunrise, a UTC Julian day: the full day's, to the bit
     * @param nextSunrise the next sunrise, a UTC Julian day
     * @param normal whether the Sun rose and set; a polar day is read under the
     *     profile's policy
     * @param tithi the tithi at the sunrise: the date's name
     * @param paksha that tithi's paksha
     * @param month the lunar month at the sunrise, with its kind
     * @param ends every tithi ending after the sunrise and at or before the
     *     next, in order
     * @param kind whether a tithi is skipped or repeated here
     * @param festivals the keys of what is kept on this date; empty unless
     *     festivals were asked
     */
    public record Cell(CalendarDate date, Vara vara, double sunrise, double nextSunrise, boolean normal,
            Tithi tithi, Paksha paksha, Month month, List<TithiEnd> ends, Kind kind, List<String> festivals) {
        /** Copies the lists. */
        public Cell {
            ends = List.copyOf(ends);
            festivals = List.copyOf(festivals);
        }
    }
}

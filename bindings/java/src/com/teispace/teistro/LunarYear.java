package com.teispace.teistro;

import java.util.List;

/**
 * One lunar year: the name it carries, its numbers and bounds, and the
 * Jovian years that ran in it ({@code 03-design/calendar-indian-lunisolar.md} §10).
 *
 * @param samvatsara the name the year carries under {@code calendars.samvatsara}
 * @param count which count named it: {@code BARHASPATYA},
 *     {@code BARHASPATYA_RUNNING} or {@code CHANDRAMANA}
 * @param vikrama the Vikrama year
 * @param shaka the Shaka year, whose number the southern count reads
 * @param opened the new moon that opened the year's first Chaitra, a UTC Julian day
 * @param began the sunrise of Chaitra Shukla Pratipada, where the name is
 *     read, a UTC Julian day
 * @param ended the next year's first sunrise, which ends this one, a UTC Julian day
 * @param jovian the Jovian years running between {@code began} and {@code ended}, in order
 * @param lupta the Jovian year that began and ended inside this one and so
 *     names no year; may be null
 */
public record LunarYear(
        Samvatsara samvatsara, String count, int vikrama, int shaka, double opened, double began, double ended,
        List<JovianYear> jovian, Samvatsara lupta) {
    /** Keeps the list unmodifiable. */
    public LunarYear {
        jovian = List.copyOf(jovian);
    }
}

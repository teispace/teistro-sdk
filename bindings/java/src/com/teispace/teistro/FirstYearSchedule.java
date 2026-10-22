package com.teispace.teistro;

import java.util.List;

/**
 * A death's first year ({@code 03-design/shraddha.md} §3.3): the <i>Dharmasindhu</i>'s sixteen
 * shraddhas, the first abdika and the sapindana, each on its day.
 *
 * @param key the request's key
 * @param origin the death's lunar date, as an anniversary's origin is found
 * @param dayOne the first day of the count, as the ashauch's
 * @param dayOneBy why it is that day
 * @param eleventhDay the eleventh day: {@code dayOne} plus ten
 * @param sapindana the sapindana's day
 * @param rites every rite, in the order of their own days, then of the sixteen
 */
public record FirstYearSchedule(
        String key, AnniversaryDay.Origin origin, CalendarDate dayOne, AshauchDayOneBy dayOneBy,
        CalendarDate eleventhDay, CalendarDate sapindana, List<FirstYearRite> rites) {
    /** Keeps the rites unmodifiable. */
    public FirstYearSchedule {
        rites = List.copyOf(rites);
    }
}

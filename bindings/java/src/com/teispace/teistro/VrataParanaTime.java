package com.teispace.teistro;

/**
 * When a vrata's fast is broken under one reading
 * ({@code 03-design/festival-rules.md} §8.7).
 *
 * @param reading the reading
 * @param day the day after the observance
 * @param fromJd when the meal may begin, a UTC Julian day
 * @param fromBy what set it: {@code SUNRISE}, {@code TITHI_END},
 *     {@code NAKSHATRA_END} or {@code WINDOW_END}
 * @param toJd when it must be taken by, for a reading with a window; null
 *     otherwise
 * @param toBy what set that, {@code TITHI_END} or {@code YAMAS}; null
 *     otherwise
 */
public record VrataParanaTime(VrataReading reading, CalendarDate day, double fromJd, String fromBy,
        Double toJd, String toBy) {}

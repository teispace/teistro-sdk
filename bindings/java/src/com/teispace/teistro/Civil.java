package com.teispace.teistro;

/**
 * Civil dates, times and zones, without naming the fields a call fills in:
 * what Python's {@code date}, {@code at} and {@code iana_zone} are, for a
 * static import.
 *
 * <pre>{@code
 * import static com.teispace.teistro.Civil.*;
 *
 * ZoneResolution resolved = ctx.time().resolve(
 *     at(date(Calendar.GREGORIAN, 1986, 1, 1), 0, 20), ianaZone("Asia/Kathmandu"));
 * }</pre>
 */
public final class Civil {
    private Civil() {}

    /**
     * A date in a calendar. The era and the era year are what the call
     * resolves them to, and the resolution is {@code DEFINED}, which is what
     * a date a caller states means.
     *
     * @param calendar the calendar the date is in
     * @param year the astronomical year
     * @param month the month, 1-based
     * @param day the day, 1-based
     * @return the date
     */
    public static CalendarDate date(Calendar calendar, int year, int month, int day) {
        return new CalendarDate(calendar, null, year, 0, month, day, Resolution.DEFINED, 0, 0);
    }

    /**
     * A date at a time of day, to the minute.
     *
     * @param day the date
     * @param hour the hour, 0 to 23
     * @param minute the minute, 0 to 59
     * @return the date and time
     */
    public static CivilDateTime at(CalendarDate day, int hour, int minute) {
        return at(day, hour, minute, 0);
    }

    /**
     * A date at a time of day, to the second.
     *
     * @param day the date
     * @param hour the hour, 0 to 23
     * @param minute the minute, 0 to 59
     * @param second the second, 0 to 60
     * @return the date and time
     */
    public static CivilDateTime at(CalendarDate day, int hour, int minute, int second) {
        return new CivilDateTime(day, new CivilTime(hour, minute, second, true, 0));
    }

    /**
     * A date whose time of day is unknown. Nothing guesses one: unless the
     * profile sets {@code time.unknown_time}, a resolution refuses it by
     * name and the hint says what to choose.
     *
     * @param day the date
     * @return the date, with no time
     */
    public static CivilDateTime whenUnknown(CalendarDate day) {
        return new CivilDateTime(day, new CivilTime(0, 0, 0, false, 0));
    }

    /**
     * A zone of the embedded database, by its IANA name.
     *
     * @param name the name, such as {@code Asia/Kathmandu}
     * @return the zone
     */
    public static ZoneSpec ianaZone(String name) {
        return new ZoneSpec(ZoneKind.IANA, 0, new Longitude(0), name);
    }

    /**
     * A fixed offset from UTC.
     *
     * @param offsetSeconds the offset east of UTC, in seconds
     * @return the zone
     */
    public static ZoneSpec fixedZone(int offsetSeconds) {
        return new ZoneSpec(ZoneKind.FIXED, offsetSeconds, new Longitude(0), null);
    }

    /**
     * Local mean time at a longitude, which is what a chart from before the
     * zone existed is cast in.
     *
     * @param longitude the longitude, east-positive
     * @return the zone
     */
    public static ZoneSpec localMeanZone(Longitude longitude) {
        return new ZoneSpec(ZoneKind.LOCAL_MEAN, 0, longitude, null);
    }
}

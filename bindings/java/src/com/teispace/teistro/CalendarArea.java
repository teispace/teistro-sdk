package com.teispace.teistro;

/**
 * {@code sky.calendar()}: the calendars, and the fixed day they share.
 *
 * <pre>{@code
 * CalendarDate bs = sky.calendar().dateOf(Calendar.BIKRAM_SAMBAT, 739_000);
 * CalendarDate ad = sky.calendar().convert(bs, Calendar.GREGORIAN);
 * }</pre>
 */
public final class CalendarArea {
    private final Context context;

    CalendarArea(Context context) {
        this.context = context;
    }

    /**
     * The date a fixed day number is, in a calendar.
     *
     * @param calendar the calendar
     * @param fixed the fixed day number (Rata Die: 1 January 1 CE is 1)
     * @return the date
     */
    public CalendarDate dateOf(Calendar calendar, long fixed) {
        return context.locked((lib, raw) -> Calls.calendarFromFixed(lib, raw, calendar, fixed));
    }

    /**
     * The fixed day number a date is.
     *
     * @param date the date
     * @return its fixed day number
     */
    public long fixedOf(CalendarDate date) {
        return context.locked((lib, raw) -> Calls.calendarToFixed(lib, raw, date));
    }

    /**
     * The same day in another calendar.
     *
     * @param date the date
     * @param into the calendar to express it in
     * @return the date in {@code into}
     */
    public CalendarDate convert(CalendarDate date, Calendar into) {
        return context.locked((lib, raw) -> Calls.calendarConvert(lib, raw, date, into));
    }

    /**
     * The weekday of a date as its ISO number: Monday {@code 1}, Sunday
     * {@code 7}. Not the catalogue's {@link Vara}, which counts from Sunday:
     * a vara is {@code weekdayOf(day) % 7}.
     *
     * @param date the date
     * @return its ISO weekday
     */
    public int weekdayOf(CalendarDate date) {
        return context.locked((lib, raw) -> Calls.calendarWeekday(lib, raw, date));
    }

    /**
     * How many days a month has.
     *
     * @param calendar the calendar
     * @param year the astronomical year
     * @param month the month, 1-based
     * @return its length in days
     */
    public int monthLength(Calendar calendar, int year, int month) {
        return context.locked((lib, raw) -> Calls.calendarMonthLength(lib, raw, calendar, year, month));
    }

    /**
     * Whether a year is a leap year in a calendar.
     *
     * @param calendar the calendar
     * @param year the astronomical year
     * @return true for a leap year
     */
    public boolean isLeap(Calendar calendar, int year) {
        return context.locked((lib, raw) -> Calls.calendarIsLeap(lib, raw, calendar, year)) != 0;
    }

    /**
     * A Tibetan year's names ({@code calendars/tibetan.md}): its rabjung cycle and place in it, 1 to 60,
     * with the Indian cycle's name for that place, and the element, gender and animal it is called by.
     *
     * <pre>{@code
     * TibetanAnimal animal = sdk.calendar().tibetanYear(2026).animal(); // HORSE
     * }</pre>
     *
     * @param year the Gregorian year the Tibetan year begins in, 1027 to 9998
     * @return its names
     */
    public TibetanYear tibetanYear(int year) {
        java.util.Map<?, ?> raw = Reads.object(Json.read(
                context.locked((lib, handle) -> Calls.calendarTibetanYear(lib, handle, year))));
        return new TibetanYear(
                Reads.integer(raw, "year"),
                Reads.integer(raw, "rabjung"),
                Reads.integer(raw, "yearInRabjung"),
                Reads.member(Samvatsara.byKey(Reads.string(raw, "samvatsara")), "Samvatsara", raw.get("samvatsara")),
                Reads.member(TibetanElement.byKey(Reads.string(raw, "element")), "TibetanElement", raw.get("element")),
                Reads.member(Gender.byKey(Reads.string(raw, "gender")), "Gender", raw.get("gender")),
                Reads.member(TibetanAnimal.byKey(Reads.string(raw, "animal")), "TibetanAnimal", raw.get("animal")));
    }
}

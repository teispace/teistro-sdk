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
     * The date {@code years}, {@code months} and {@code days} after {@code date}, in its own calendar
     * ({@code 03-design/calendar-bikram-sambat.md} §11): the years and months first, a day number the
     * month lacks resolved by {@code overflow}, then the days. A Bikram Sambat month runs 29 to 32 days,
     * so the 32nd of Jestha plus a month is the 31st of a 31-day Ashadh under {@link Overflow#CONSTRAIN},
     * the 1st of Shrawan under {@link Overflow#CARRY}, and refused under {@link Overflow#REJECT}.
     *
     * <pre>{@code
     * CalendarDate on = sdk.calendar().add(Civil.date(Calendar.BIKRAM_SAMBAT, 2081, 2, 32), 0, 1, 0,
     *         Overflow.CONSTRAIN); // Ashadh 31
     * }</pre>
     *
     * @param date the date
     * @param years the years to add, signed
     * @param months the months to add, signed
     * @param days the days to add, signed
     * @param overflow how a day the month lacks is resolved
     * @return the date reached
     */
    public CalendarDate add(CalendarDate date, int years, int months, int days, Overflow overflow) {
        return context.locked((lib, raw) -> Calls.calendarAdd(lib, raw, date, years, months, days, overflow));
    }

    /**
     * The years, months and days from {@code from} to {@code to}, both in one calendar, counted as an
     * age is: the most whole months whose anniversary of {@code from} is not past {@code to}, then the
     * days from it, so {@link #add} of the span gives {@code to}. One born on the 32nd of Jestha
     * completes a year on the 31st of a 31-day Jestha under {@link Overflow#CONSTRAIN} (C806). Earlier
     * than {@code from}, each part is negative or zero.
     *
     * <pre>{@code
     * DateSpan age = sdk.calendar().between(Civil.date(Calendar.BIKRAM_SAMBAT, 2050, 2, 32),
     *         Civil.date(Calendar.BIKRAM_SAMBAT, 2082, 2, 31), Overflow.CONSTRAIN); // 32 years
     * }</pre>
     *
     * @param from the first date
     * @param to the last date
     * @param overflow how an anniversary on a day its month lacks is resolved
     * @return the span
     */
    public DateSpan between(CalendarDate from, CalendarDate to, Overflow overflow) {
        return context.locked((lib, raw) -> Calls.calendarBetween(lib, raw, from, to, overflow));
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

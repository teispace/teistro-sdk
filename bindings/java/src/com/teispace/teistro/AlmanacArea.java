package com.teispace.teistro;

import java.util.List;
import java.util.Map;

import com.teispace.teistro.blob.Panchanga;
import com.teispace.teistro.ffi.Native;

/**
 * {@code sky.almanac()}: a day, or a run of days, with its limbs.
 *
 * <p>The boundary calls this {@code panchanga}; the area takes the
 * consumer's word, because an almanac is what the operation answers and a
 * panchanga is one tradition's name for five of its limbs
 * ({@code 03-design/surface-areas.md}).
 *
 * <pre>{@code
 * Almanac month = sky.almanac().of(first, last, kathmandu, 20_700);
 * List<Span<Tithi>> tithis = month.at(0).tithi();
 * Almanac marriage = sky.almanac().of(first, last, kathmandu, 20_700,
 *         Map.of("rules", "RAMAN_MARRIAGE"), null, false, false, false);
 * }</pre>
 */
public final class AlmanacArea {
    private final Context context;

    /**
     * The almanac area of a context.
     *
     * @param context the context every call goes through
     */
    AlmanacArea(Context context) {
        this.context = context;
    }

    /**
     * The almanac of every day in a range, at one place, with the days alone.
     *
     * @param fromDate the first day; its calendar is the range's
     * @param toDate the last day
     * @param place where
     * @param utcOffsetSeconds the local clock's offset from UTC, east positive
     * @return the almanac
     */
    public Almanac of(CalendarDate fromDate, CalendarDate toDate, Observer place, int utcOffsetSeconds) {
        return of(fromDate, toDate, place, utcOffsetSeconds, null, null, false, false, false);
    }

    /**
     * The almanac of every day in a range, at one place.
     *
     * <p>A <b>range</b> rather than a list of dates, because consecutive days
     * share a boundary (day <i>n</i>'s next sunrise is day <i>n+1</i>'s
     * sunrise), so a month of days costs much less than thirty days computed
     * separately. A range holding more than a year and a day is refused by
     * name. {@code muhurta} runs a search over the same days, answered as
     * {@link Almanac#muhurta()}, and {@code festivals} the rules whose days
     * fall in them, answered as {@link Almanac#festivals()}; the days are
     * founded once for both.
     *
     * <p>A request is a {@code Map} as {@link Json#write} writes it (a
     * member written as its full key, a clause or a bar an answer gave
     * handed back as it stands), or its JSON text.
     *
     * @param fromDate the first day; its calendar is the range's
     * @param toDate the last day
     * @param place where
     * @param utcOffsetSeconds the local clock's offset from UTC, east positive
     * @param muhurta the muhurta search to run, such as
     *     {@code Map.of("rules", "RAMAN_MARRIAGE")}; may be null for none
     * @param festivals the festival rules to reckon, such as
     *     {@code Map.of("rules", "DHARMASINDHU")}; may be null for none
     * @param years whether to answer the lunar years the days fall in, as {@link Almanac#years()}
     * @param eclipses whether to answer the eclipses whose greatest moment
     *     falls in the days, with how the place sees each, as {@link Almanac#eclipses()}
     * @param nepalSambat whether to answer each day's Nepal Sambat date, as
     *     {@link Almanac#nepalSambat()}
     * @return the almanac
     */
    public Almanac of(CalendarDate fromDate, CalendarDate toDate, Observer place, int utcOffsetSeconds,
            Object muhurta, Object festivals, boolean years, boolean eclipses, boolean nepalSambat) {
        long sections = (years ? Native.TS_PANCHANGA_YEARS : 0)
                | (eclipses ? Native.TS_PANCHANGA_ECLIPSES : 0)
                | (nepalSambat ? Native.TS_PANCHANGA_NEPAL_SAMBAT : 0);
        PanchangaRequest request = new PanchangaRequest(
                fromDate.calendar(),
                fromDate.year(),
                fromDate.month(),
                fromDate.day(),
                toDate.month(),
                toDate.day(),
                toDate.year(),
                place.latitudeDeg().value(),
                place.longitudeDeg().value(),
                place.altitudeM().value(),
                utcOffsetSeconds,
                sections,
                Reads.recordJson(muhurta, "muhurta", "Map.of(\"rules\", \"RAMAN_MARRIAGE\")"),
                Reads.recordJson(festivals, "festivals", "Map.of(\"rules\", \"DHARMASINDHU\")"));
        return new Almanac(Panchanga.decode(context.locked((lib, raw) -> Calls.panchangaDays(lib, raw, request))));
    }

    /**
     * The almanac of one day, which is the range of one unwrapped.
     *
     * @param date the day
     * @param place where
     * @param utcOffsetSeconds the local clock's offset from UTC, east positive
     * @return the day
     */
    public AlmanacDay day(CalendarDate date, Observer place, int utcOffsetSeconds) {
        return of(date, date, place, utcOffsetSeconds).at(0);
    }

    /**
     * A native's bird read over every day from {@code from} to {@code to} under Pancha Pakshi
     * ({@code 03-design/pakshi.md}), under the texts' default rules: each day's ten yamas from the
     * almanac's own sunrise, sunset and next sunrise, with the bird's activity and its timed
     * sub-periods. A day the Sun does not both rise and set has a null reading.
     *
     * <pre>{@code
     * PakshiDays days = sky.almanac().pakshi(date, date, madras, 19_800,
     *         PakshiNative.star(Nakshatra.UTTARA_ASHADHA, Paksha.SHUKLA));
     * PakshiDay.Yama second = days.value().get(0).reading().yamas().get(1);
     * }</pre>
     *
     * @param from the first day
     * @param to the last day, both ends included
     * @param place where
     * @param utcOffsetSeconds the local clock's offset from UTC, east positive
     * @param whose the native, by bird or by birth star and paksha
     * @return one entry a day, in order, with the provenance that sealed the request
     */
    public PakshiDays pakshi(CalendarDate from, CalendarDate to, Observer place, int utcOffsetSeconds,
            PakshiNative whose) {
        return PakshiReads.pakshi(context, from, to, place, utcOffsetSeconds, whose, null);
    }

    /**
     * As {@link #pakshi(CalendarDate, CalendarDate, Observer, int, PakshiNative)}, under {@code rules}:
     * {@code clock} ({@code STRETCHED} or {@code NAZHIGAI}), {@code subs} and {@code relations}
     * ({@code AGASTYA} or {@code PULIPPANI}), each optional, as
     * {@code Map.of("subs", "PULIPPANI")}; a key it does not read is refused as
     * {@code pakshi.rules.<key>}.
     *
     * @param from the first day
     * @param to the last day, both ends included
     * @param place where
     * @param utcOffsetSeconds the local clock's offset from UTC, east positive
     * @param whose the native, by bird or by birth star and paksha
     * @param rules what the days are read under
     * @return one entry a day, in order, with the provenance that sealed the request
     */
    public PakshiDays pakshi(CalendarDate from, CalendarDate to, Observer place, int utcOffsetSeconds,
            PakshiNative whose, Map<String, ?> rules) {
        return PakshiReads.pakshi(context, from, to, place, utcOffsetSeconds, whose, rules);
    }
}

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
     * <p>A request is its typed record, built or read from a map with
     * {@code of} (a member written as its full key, a clause or a bar an
     * answer gave handed back as it stands), or from JSON text with
     * {@code MuhurtaRequest.of(Json.object(text))}. A rule set the
     * consumer writes is a map in {@code rules}, as the SDK reads it.
     *
     * @param fromDate the first day; its calendar is the range's
     * @param toDate the last day
     * @param place where
     * @param utcOffsetSeconds the local clock's offset from UTC, east positive
     * @param muhurta the muhurta search to run, such as
     *     {@code MuhurtaRequest.builder().rules("RAMAN_MARRIAGE").build()}; may be null for none
     * @param festivals the festival rules to reckon, such as
     *     {@code new FestivalRequest("DHARMASINDHU")}; may be null for none
     * @param years whether to answer the lunar years the days fall in, as {@link Almanac#years()}
     * @param eclipses whether to answer the eclipses whose greatest moment
     *     falls in the days, with how the place sees each, as {@link Almanac#eclipses()}
     * @param nepalSambat whether to answer each day's Nepal Sambat date, as
     *     {@link Almanac#nepalSambat()}
     * @return the almanac
     */
    public Almanac of(CalendarDate fromDate, CalendarDate toDate, Observer place, int utcOffsetSeconds,
            com.teispace.teistro.request.MuhurtaRequest muhurta,
            com.teispace.teistro.request.FestivalRequest festivals,
            boolean years, boolean eclipses, boolean nepalSambat) {
        return asked(fromDate, toDate, place, utcOffsetSeconds, muhurta, festivals,
                (years ? Native.TS_PANCHANGA_YEARS : 0)
                        | (eclipses ? Native.TS_PANCHANGA_ECLIPSES : 0)
                        | (nepalSambat ? Native.TS_PANCHANGA_NEPAL_SAMBAT : 0));
    }

    /**
     * The days a set of festival rules falls on in a range, at one place
     * ({@code 03-design/festival-rules.md} §7.5), <b>without the days</b>:
     * each day founded as only what the rules read, so a year costs a
     * fraction of the year's almanac and answers exactly what {@code of}
     * with {@code festivals} does. Each observance's
     * {@link FestivalObservance#marginSeconds()} says how far the place can
     * move before its day can change; keep the answer under its provenance's
     * answer key.
     *
     * @param fromDate the first day; its calendar is the range's
     * @param toDate the last day
     * @param place where
     * @param utcOffsetSeconds the local clock's offset from UTC, east positive
     * @param festivals the festival rules to reckon, such as
     *     {@code new FestivalRequest("NEPAL")}
     * @return the observances
     */
    public FestivalAnswer festivals(CalendarDate fromDate, CalendarDate toDate, Observer place,
            int utcOffsetSeconds, com.teispace.teistro.request.FestivalRequest festivals) {
        return asked(fromDate, toDate, place, utcOffsetSeconds, null,
                java.util.Objects.requireNonNull(festivals, "festivals"), Native.TS_PANCHANGA_WITHOUT_DAYS)
                .festivals()
                .orElseThrow(() -> Reads.internal("a request with festivals answered none"));
    }

    private Almanac asked(CalendarDate fromDate, CalendarDate toDate, Observer place, int utcOffsetSeconds,
            com.teispace.teistro.request.MuhurtaRequest muhurta,
            com.teispace.teistro.request.FestivalRequest festivals, long sections) {
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
                muhurta == null ? null : Json.write(muhurta),
                festivals == null ? null : Json.write(festivals));
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
     * {@code PakshiRules.builder().subs(SubLengths.PULIPPANI).build()}, or
     * {@code PakshiRules.of(map)} from a map, which refuses a member it does
     * not read by its field, as {@code PakshiRules.clock}.
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
            PakshiNative whose, com.teispace.teistro.request.PakshiRules rules) {
        return PakshiReads.pakshi(context, from, to, place, utcOffsetSeconds, whose, rules);
    }

    /**
     * <b>Where this context's reckoning and {@code other}'s disagree</b>, and why
     * ({@code 03-design/setup-disagreements.md}): the official day an app shows beside the modern one.
     * {@link Disagreements#days()} holds only the days on which some limb's member at sunrise differs,
     * each limb with the member each side names, its span, its sunrise and the instant the two members
     * meet under that sky; {@link Disagreements#festivals()} each occurrence whose day or month differs,
     * or which one side alone gives. Each is null when not asked; a request comparing nothing is
     * refused as {@code disagreements.days}.
     *
     * <pre>{@code
     * Disagreements found = official.almanac().disagreements(modern, first, last, kathmandu, 20_700,
     *         true, new FestivalRequest("NEPAL"));
     * }</pre>
     *
     * @param other the reckoning compared with this one
     * @param from the first day
     * @param to the last day, both ends included
     * @param place where both are reckoned
     * @param utcOffsetSeconds the local clock's offset from UTC, east positive
     * @param days whether the days are compared
     * @param festivals the festival rules compared, such as {@code new FestivalRequest("NEPAL")}, or null for
     *     none
     * @return each comparison asked, sealed by this context's provenance
     */
    public Disagreements disagreements(Context other, CalendarDate from, CalendarDate to, Observer place,
            int utcOffsetSeconds, boolean days, com.teispace.teistro.request.FestivalRequest festivals) {
        if (other == null) {
            throw Reads.invalid("other is the Context compared with this one", "other");
        }
        if (from == null || to == null) {
            throw Reads.invalid("from and to are CalendarDates", from == null ? "from" : "to");
        }
        if (place == null) {
            throw Reads.invalid("place is an Observer", "place");
        }
        Map<String, Object> asked = new java.util.LinkedHashMap<>();
        asked.put("calendar", from.calendar().key());
        asked.put("first", Map.of("year", from.year(), "month", from.month(), "day", from.day()));
        asked.put("last", Map.of("year", to.year(), "month", to.month(), "day", to.day()));
        asked.put("latitudeDeg", place.latitudeDeg().value());
        asked.put("longitudeDeg", place.longitudeDeg().value());
        asked.put("altitudeM", place.altitudeM().value());
        asked.put("utcOffsetSeconds", utcOffsetSeconds);
        asked.put("days", days);
        if (festivals != null) {
            asked.put("festivals", festivals);
        }
        String json = Json.write(asked);
        return AlmanacReads.disagreements(context.lockedWith(other,
                (lib, ours, theirs) -> Calls.almanacDisagreements(lib, ours, theirs, json)));
    }
}

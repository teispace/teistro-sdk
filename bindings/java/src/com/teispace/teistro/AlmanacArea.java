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
     *     {@code new FestivalRequest(FestivalRules.DHARMASINDHU)}; may be null for none
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
        return of(fromDate, toDate, place, utcOffsetSeconds, muhurta, festivals, years, eclipses, nepalSambat,
                false);
    }

    /**
     * As {@link #of(CalendarDate, CalendarDate, Observer, int, com.teispace.teistro.request.MuhurtaRequest,
     * com.teispace.teistro.request.FestivalRequest, boolean, boolean, boolean)}, with each day's lagna table when
     * {@code lagnas} asks: every sign rising from sunrise to the next sunrise, from the ascendant
     * {@code panchanga.lagna_table} names, as {@link AlmanacDay#lagnas()}. Some four hundred ascendants a day, so only
     * on asking.
     *
     * @param fromDate the first day; its calendar is the range's
     * @param toDate the last day
     * @param place where
     * @param utcOffsetSeconds the local clock's offset from UTC, east positive
     * @param muhurta the muhurta search to run; may be null for none
     * @param festivals the festival rules to reckon; may be null for none
     * @param years whether to answer the lunar years the days fall in
     * @param eclipses whether to answer the eclipses of the days
     * @param nepalSambat whether to answer each day's Nepal Sambat date
     * @param lagnas whether to give each day its lagna table
     * @return the almanac
     */
    public Almanac of(CalendarDate fromDate, CalendarDate toDate, Observer place, int utcOffsetSeconds,
            com.teispace.teistro.request.MuhurtaRequest muhurta,
            com.teispace.teistro.request.FestivalRequest festivals,
            boolean years, boolean eclipses, boolean nepalSambat, boolean lagnas) {
        return of(fromDate, toDate, place, utcOffsetSeconds, muhurta, festivals, years, eclipses, nepalSambat, lagnas,
                false);
    }

    /**
     * As {@link #of(CalendarDate, CalendarDate, Observer, int, com.teispace.teistro.request.MuhurtaRequest,
     * com.teispace.teistro.request.FestivalRequest, boolean, boolean, boolean, boolean)}, with each day's homa ahuti
     * when {@code homaAhuti} asks, as {@link AlmanacDay#homaAhuti()}. The Sun's stars cost a search a day does not
     * otherwise make, so only on asking.
     *
     * @param fromDate the first day; its calendar is the range's
     * @param toDate the last day
     * @param place where
     * @param utcOffsetSeconds the local clock's offset from UTC, east positive
     * @param muhurta the muhurta search to run; may be null for none
     * @param festivals the festival rules to reckon; may be null for none
     * @param years whether to answer the lunar years the days fall in
     * @param eclipses whether to answer the eclipses of the days
     * @param nepalSambat whether to answer each day's Nepal Sambat date
     * @param lagnas whether to give each day its lagna table
     * @param homaAhuti whether to give each day its homa ahuti
     * @return the almanac
     */
    public Almanac of(CalendarDate fromDate, CalendarDate toDate, Observer place, int utcOffsetSeconds,
            com.teispace.teistro.request.MuhurtaRequest muhurta,
            com.teispace.teistro.request.FestivalRequest festivals,
            boolean years, boolean eclipses, boolean nepalSambat, boolean lagnas, boolean homaAhuti) {
        return of(fromDate, toDate, place, utcOffsetSeconds, muhurta, festivals, years, eclipses, nepalSambat, lagnas,
                homaAhuti, false, 0);
    }

    /**
     * {@link #of(CalendarDate, CalendarDate, Observer, int, com.teispace.teistro.request.MuhurtaRequest,
     * com.teispace.teistro.request.FestivalRequest, boolean, boolean, boolean, boolean, boolean)} and,
     * when {@code sky} asks, each day's sky, as {@link Almanac#sky()}: the Sun's and the Moon's paths, rises,
     * transits and sets, and the Moon's lit shape ({@code 03-design/day-sky.md}).
     *
     * @param fromDate the first day; its calendar is the range's
     * @param toDate the last day
     * @param place where
     * @param utcOffsetSeconds the local clock's offset from UTC, east positive
     * @param muhurta a muhurta search over the same days, or null
     * @param festivals festival rules to reckon over the same days, or null
     * @param years whether to answer the lunar years the days fall in
     * @param eclipses whether to answer the eclipses of the days
     * @param nepalSambat whether to answer each day's Nepal Sambat date
     * @param lagnas whether to give each day its lagna table
     * @param homaAhuti whether to give each day its homa ahuti
     * @param sky whether to answer each day's sky
     * @param skyStepMinutes the step of its paths, 1 to 240, zero for ten
     * @return the almanac
     */
    public Almanac of(CalendarDate fromDate, CalendarDate toDate, Observer place, int utcOffsetSeconds,
            com.teispace.teistro.request.MuhurtaRequest muhurta,
            com.teispace.teistro.request.FestivalRequest festivals,
            boolean years, boolean eclipses, boolean nepalSambat, boolean lagnas, boolean homaAhuti,
            boolean sky, int skyStepMinutes) {
        return of(fromDate, toDate, place, utcOffsetSeconds, muhurta, festivals, years, eclipses, nepalSambat, lagnas,
                homaAhuti, sky, skyStepMinutes, false);
    }

    /**
     * {@link #of(CalendarDate, CalendarDate, Observer, int, com.teispace.teistro.request.MuhurtaRequest,
     * com.teispace.teistro.request.FestivalRequest, boolean, boolean, boolean, boolean, boolean, boolean, int)} and,
     * when {@code markers} asks, each day's ganda mula, gandanta and vinchudo, as {@link AlmanacDay#markers()}
     * ({@code 03-design/day-markers.md} §3).
     *
     * @param fromDate the first day; its calendar is the range's
     * @param toDate the last day
     * @param place where
     * @param utcOffsetSeconds the local clock's offset from UTC, east positive
     * @param muhurta a muhurta search over the same days, or null
     * @param festivals festival rules to reckon over the same days, or null
     * @param years whether to answer the lunar years the days fall in
     * @param eclipses whether to answer the eclipses of the days
     * @param nepalSambat whether to answer each day's Nepal Sambat date
     * @param lagnas whether to give each day its lagna table
     * @param homaAhuti whether to give each day its homa ahuti
     * @param sky whether to answer each day's sky
     * @param skyStepMinutes the step of its paths, 1 to 240, zero for ten
     * @param markers whether to give each day its markers
     * @return the almanac
     */
    public Almanac of(CalendarDate fromDate, CalendarDate toDate, Observer place, int utcOffsetSeconds,
            com.teispace.teistro.request.MuhurtaRequest muhurta,
            com.teispace.teistro.request.FestivalRequest festivals,
            boolean years, boolean eclipses, boolean nepalSambat, boolean lagnas, boolean homaAhuti,
            boolean sky, int skyStepMinutes, boolean markers) {
        return of(fromDate, toDate, place, utcOffsetSeconds, muhurta, festivals, years, eclipses, nepalSambat, lagnas,
                homaAhuti, sky, skyStepMinutes, markers, false);
    }

    /**
     * {@link #of(CalendarDate, CalendarDate, Observer, int, com.teispace.teistro.request.MuhurtaRequest,
     * com.teispace.teistro.request.FestivalRequest, boolean, boolean, boolean, boolean, boolean, boolean, int,
     * boolean)} and, when {@code gowri} asks, each day's Gowri panchangam, as {@link AlmanacDay#gowri()}
     * ({@code 03-design/gowri-panchangam.md}).
     *
     * @param fromDate the first day; its calendar is the range's
     * @param toDate the last day
     * @param place where
     * @param utcOffsetSeconds the local clock's offset from UTC, east positive
     * @param muhurta a muhurta search over the same days, or null
     * @param festivals festival rules to reckon over the same days, or null
     * @param years whether to answer the lunar years the days fall in
     * @param eclipses whether to answer the eclipses of the days
     * @param nepalSambat whether to answer each day's Nepal Sambat date
     * @param lagnas whether to give each day its lagna table
     * @param homaAhuti whether to give each day its homa ahuti
     * @param sky whether to answer each day's sky
     * @param skyStepMinutes the step of its paths, 1 to 240, zero for ten
     * @param markers whether to give each day its markers
     * @param gowri whether to give each day its Gowri panchangam
     * @return the almanac
     */
    public Almanac of(CalendarDate fromDate, CalendarDate toDate, Observer place, int utcOffsetSeconds,
            com.teispace.teistro.request.MuhurtaRequest muhurta,
            com.teispace.teistro.request.FestivalRequest festivals,
            boolean years, boolean eclipses, boolean nepalSambat, boolean lagnas, boolean homaAhuti,
            boolean sky, int skyStepMinutes, boolean markers, boolean gowri) {
        return asked(fromDate, toDate, place, utcOffsetSeconds, null, muhurta, festivals,
                (years ? Native.TS_PANCHANGA_YEARS : 0)
                        | (eclipses ? Native.TS_PANCHANGA_ECLIPSES : 0)
                        | (nepalSambat ? Native.TS_PANCHANGA_NEPAL_SAMBAT : 0)
                        | (lagnas ? Native.TS_PANCHANGA_LAGNAS : 0)
                        | (homaAhuti ? Native.TS_PANCHANGA_HOMA_AHUTI : 0)
                        | (sky ? Native.TS_PANCHANGA_SKY : 0)
                        | (markers ? Native.TS_PANCHANGA_MARKERS : 0)
                        | (gowri ? Native.TS_PANCHANGA_GOWRI : 0),
                skyStepMinutes);
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
     *     {@code new FestivalRequest(FestivalRules.NEPAL)}
     * @return the observances
     */
    public FestivalAnswer festivals(CalendarDate fromDate, CalendarDate toDate, Observer place,
            int utcOffsetSeconds, com.teispace.teistro.request.FestivalRequest festivals) {
        return asked(fromDate, toDate, place, utcOffsetSeconds, null,
                java.util.Objects.requireNonNull(festivals, "festivals"), Native.TS_PANCHANGA_WITHOUT_DAYS)
                .festivals()
                .orElseThrow(() -> Reads.internal("a request with festivals answered none"));
    }

    /**
     * A published set read from a signed pack
     * ({@code 03-design/published-overrides.md} §5), ready for a
     * {@code FestivalRequest}'s {@code published}. The publisher's key is the
     * app's to fix at its own build; the SDK embeds none. A version under
     * {@code atLeast}, the newest the app has applied, is refused, so a
     * replayed pack cannot roll a correction back.
     *
     * @param pack the set's exact bytes
     * @param signature the publisher's detached Ed25519 signature over them, 64 bytes
     * @param key the publisher's public key, 32 bytes
     * @param atLeast the newest version applied; 0 takes any
     * @return the set
     * @throws TeistroException {@code INVALID_ARG} naming {@code key}, {@code signature},
     *     {@code version} or the set's own field
     */
    public com.teispace.teistro.request.PublishedSet publishedFromSigned(byte[] pack, byte[] signature, byte[] key,
            long atLeast) {
        byte[] bytes = pack.clone();
        byte[] signed = signature.clone();
        byte[] publisher = key.clone();
        String json = context.locked((lib, raw) -> Calls.publishedVerify(lib, raw, bytes, signed, publisher, atLeast));
        return com.teispace.teistro.request.PublishedSet.of(Json.object(json));
    }

    /**
     * {@code of} on a zone's clock: each day on the offset of its own noon,
     * and the provenance names the zone's offset, source and era at the
     * first day ({@code 03-design/time-and-timezone.md} §3.4).
     *
     * @param fromDate the first day; its calendar is the range's
     * @param toDate the last day
     * @param place where
     * @param zone the zone, from {@code Civil.ianaZone} and its neighbours
     * @return the days
     */
    public Almanac inZone(CalendarDate fromDate, CalendarDate toDate, Observer place, ZoneSpec zone) {
        return asked(fromDate, toDate, place, 0, zoneJson(zone), null, null, 0, 0);
    }

    /** A zone as the record the boundary reads beside a range or a chart's births. */
    static String zoneJson(ZoneSpec zone) {
        return Json.write(zoneRecord(zone));
    }

    /**
     * A request's clock as the SDK reads it, an offset: the one field the
     * request's JSON carries for it.
     */
    static Map<String, Object> clock(int utcOffsetSeconds) {
        Map<String, Object> written = new java.util.LinkedHashMap<>();
        written.put("utcOffsetSeconds", utcOffsetSeconds);
        return written;
    }

    /**
     * A request's clock as the SDK reads it, a zone read at each instant's
     * offset then ({@code 03-design/time-and-timezone.md} §3.4).
     */
    static Map<String, Object> clock(ZoneSpec zone) {
        if (zone == null) {
            throw Reads.invalid("zone is a ZoneSpec", "zone");
        }
        Map<String, Object> written = new java.util.LinkedHashMap<>();
        written.put("zone", zoneRecord(zone));
        return written;
    }

    /** A zone as the record the SDK reads in a request's JSON. */
    static Map<String, Object> zoneRecord(ZoneSpec zone) {
        Map<String, Object> written = new java.util.LinkedHashMap<>();
        switch (zone.kind()) {
            case IANA -> {
                written.put("kind", "IANA");
                written.put("zone", zone.zone());
            }
            case FIXED -> {
                written.put("kind", "FIXED");
                written.put("offset", zone.offsetSeconds());
            }
            default -> {
                written.put("kind", "LOCAL_MEAN");
                written.put("longitude", zone.longitudeDeg().value());
            }
        }
        return written;
    }

    /**
     * Every instant in a range at which the tithi, nakshatra, yoga, karana,
     * the Moon's or Sun's sign or the weekday changes, sorted, as one list
     * ({@code 03-design/change-points.md}). The range is asked on
     * {@code utcOffsetSeconds} or a {@code zone}, and every point is the
     * same instant either way.
     *
     * @param request the range, its clock and the kinds wanted
     * @return the points, the window and the provenance
     */
    public ChangePoints changes(com.teispace.teistro.request.ChangesRequest request) {
        String json = Json.write(request);
        Map<?, ?> envelope = Reads.object(Json.read(context.locked((lib, raw) -> Calls.almanacChanges(lib, raw, json))));
        Map<?, ?> value = Reads.object(envelope, "value");
        return new ChangePoints(
                Reads.each(value, "points", raw -> {
                    Map<?, ?> point = Reads.object(raw);
                    return new ChangePoints.Point(Reads.number(point, "at"), Reads.string(point, "kind"),
                            Reads.string(point, "from"), Reads.string(point, "to"));
                }),
                Reads.interval(Reads.field(value, "window")),
                com.teispace.teistro.record.Provenance.of(Reads.field(envelope, "provenance")));
    }

    private Almanac asked(CalendarDate fromDate, CalendarDate toDate, Observer place, int utcOffsetSeconds,
            com.teispace.teistro.request.MuhurtaRequest muhurta,
            com.teispace.teistro.request.FestivalRequest festivals, long sections) {
        return asked(fromDate, toDate, place, utcOffsetSeconds, null, muhurta, festivals, sections, 0);
    }

    private Almanac asked(CalendarDate fromDate, CalendarDate toDate, Observer place, int utcOffsetSeconds,
            String zoneJson, com.teispace.teistro.request.MuhurtaRequest muhurta,
            com.teispace.teistro.request.FestivalRequest festivals, long sections, int skyStepMinutes) {
        PanchangaRequest request = new PanchangaRequest(
                fromDate.calendar(),
                skyStepMinutes,
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
                festivals == null ? null : Json.write(festivals),
                zoneJson);
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
     * The almanac of one day, with its lagna table when {@code lagnas} asks, as {@link AlmanacDay#lagnas()}.
     *
     * @param date the day
     * @param place where
     * @param utcOffsetSeconds the local clock's offset from UTC, east positive
     * @param lagnas whether to give the day its lagna table
     * @return the day
     */
    public AlmanacDay day(CalendarDate date, Observer place, int utcOffsetSeconds, boolean lagnas) {
        return day(date, place, utcOffsetSeconds, lagnas, false);
    }

    /**
     * The almanac of one day, with its lagna table when {@code lagnas} asks and its homa ahuti when
     * {@code homaAhuti} does, as {@link AlmanacDay#lagnas()} and {@link AlmanacDay#homaAhuti()}.
     *
     * @param date the day
     * @param place where
     * @param utcOffsetSeconds the local clock's offset from UTC, east positive
     * @param lagnas whether to give the day its lagna table
     * @param homaAhuti whether to give the day its homa ahuti
     * @return the day
     */
    public AlmanacDay day(CalendarDate date, Observer place, int utcOffsetSeconds, boolean lagnas,
            boolean homaAhuti) {
        return of(date, date, place, utcOffsetSeconds, null, null, false, false, false, lagnas, homaAhuti).at(0);
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
        return PakshiReads.pakshi(context, from, to, place, clock(utcOffsetSeconds), whose, null);
    }

    /**
     * As {@link #pakshi(CalendarDate, CalendarDate, Observer, int, PakshiNative)}, under {@code rules}:
     * {@code clock} ({@code STRETCHED} or {@code NAZHIGAI}), {@code subs} and {@code relations}
     * ({@code AGASTYA} or {@code PULIPPANI}), each optional, as
     * {@code PakshiRules.builder().subs(PakshiSubLengths.PULIPPANI).build()}, or
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
        return PakshiReads.pakshi(context, from, to, place, clock(utcOffsetSeconds), whose, rules);
    }

    /**
     * {@link #pakshi(CalendarDate, CalendarDate, Observer, int, PakshiNative)} on a zone's clock: each
     * day read at the offset the zone had then ({@code 03-design/time-and-timezone.md} §3.4).
     *
     * <pre>{@code
     * PakshiDays days = sky.almanac().pakshi(date, date, madras, Civil.ianaZone("Asia/Kolkata"),
     *         PakshiNative.bird("OWL"));
     * }</pre>
     *
     * @param from the first day
     * @param to the last day, both ends included
     * @param place where
     * @param zone the zone each day is read on, from {@code Civil.ianaZone} and its neighbours
     * @param whose the native, by bird or by birth star and paksha
     * @return one entry a day, in order, with the provenance that sealed the request
     */
    public PakshiDays pakshi(CalendarDate from, CalendarDate to, Observer place, ZoneSpec zone,
            PakshiNative whose) {
        return PakshiReads.pakshi(context, from, to, place, clock(zone), whose, null);
    }

    /**
     * {@link #pakshi(CalendarDate, CalendarDate, Observer, int, PakshiNative,
     * com.teispace.teistro.request.PakshiRules)} on a zone's clock: each day read at the offset the
     * zone had then.
     *
     * @param from the first day
     * @param to the last day, both ends included
     * @param place where
     * @param zone the zone each day is read on, from {@code Civil.ianaZone} and its neighbours
     * @param whose the native, by bird or by birth star and paksha
     * @param rules what the days are read under
     * @return one entry a day, in order, with the provenance that sealed the request
     */
    public PakshiDays pakshi(CalendarDate from, CalendarDate to, Observer place, ZoneSpec zone,
            PakshiNative whose, com.teispace.teistro.request.PakshiRules rules) {
        return PakshiReads.pakshi(context, from, to, place, clock(zone), whose, rules);
    }

    /**
     * <b>The time a sankalpa recites</b> at an instant and a place ({@code 03-design/sankalpa.md}): the
     * samvatsara, ayana, ritu, lunar month and vara as the sunrise of the day holding the instant has
     * them, and the paksha, tithi, nakshatra, yoga, karana and the Sun's, Moon's and Jupiter's signs as
     * they stand at the instant. A rite before dawn recites the day before's.
     *
     * <pre>{@code
     * SankalpaAnswer found = sdk.almanac().sankalpa(2_461_055.593_75, kathmandu, 20_700, AyanaReckoning.SIDEREAL);
     * Samvatsara year = found.value().year().samvatsara();
     * }</pre>
     *
     * @param at the instant of the rite, a UTC Julian day
     * @param place where it is performed
     * @param utcOffsetSeconds the local clock's offset from UTC, east positive
     * @param ayana how the ayana turns: {@code SIDEREAL}, at Makara and Karka sankranti, or
     *     {@code TROPICAL}, at the solstices; null for {@code SIDEREAL}
     * @return the time recited, sealed
     */
    public SankalpaAnswer sankalpa(double at, Observer place, int utcOffsetSeconds,
            com.teispace.teistro.request.AyanaReckoning ayana) {
        return sankalpa(at, place, clock(utcOffsetSeconds), ayana);
    }

    /**
     * {@link #sankalpa(double, Observer, int, com.teispace.teistro.request.AyanaReckoning)} on a zone's
     * clock: the day and the instant read at the offset the zone had then.
     *
     * @param at the instant of the rite, a UTC Julian day
     * @param place where it is performed
     * @param zone the zone each instant is read on, at its offset then, from {@code Civil.ianaZone} and its
     *     neighbours
     * @param ayana how the ayana turns; null for {@code SIDEREAL}
     * @return the time recited, sealed
     */
    public SankalpaAnswer sankalpa(double at, Observer place, ZoneSpec zone,
            com.teispace.teistro.request.AyanaReckoning ayana) {
        return sankalpa(at, place, clock(zone), ayana);
    }

    private SankalpaAnswer sankalpa(double at, Observer place, Map<String, Object> clock,
            com.teispace.teistro.request.AyanaReckoning ayana) {
        if (place == null) {
            throw Reads.invalid("place is an Observer", "place");
        }
        Map<String, Object> asked = new java.util.LinkedHashMap<>();
        asked.put("at", at);
        asked.put("latitudeDeg", place.latitudeDeg().value());
        asked.put("longitudeDeg", place.longitudeDeg().value());
        asked.put("altitudeM", place.altitudeM().value());
        asked.putAll(clock);
        if (ayana != null) {
            asked.put("ayana", ayana.key());
        }
        String json = Json.write(asked);
        return AlmanacReads.sankalpaAnswer(context.locked((lib, raw) -> Calls.almanacSankalpa(lib, raw, json)));
    }

    /**
     * <b>The sankalpa in words</b> ({@code 03-design/sankalpa.md} §5): a tradition's template
     * ({@code tradition}, {@code sdk.sankalpa.himavatkhande} when left out, or a key a loaded pack
     * carries) filled with the time {@link #sankalpa(double, Observer, int,
     * com.teispace.teistro.request.AyanaReckoning)} reckons at the instant and place, and the performer,
     * gotra, purpose, river, holy ground and pitris in {@code slots}, in the request's locale or the
     * context's. A refusal is named under {@code sankalpaWords}.
     *
     * <pre>{@code
     * SankalpaWordsAnswer said = sky.almanac().sankalpaWords(SankalpaWordsRequest.builder()
     *         .at(2_461_055.593_75).latitudeDeg(27.7172).longitudeDeg(85.324).utcOffsetSeconds(20_700)
     *         .slots(new SankalpaFor("कौण्डिन्य", "हरिनाथशर्मा", "गणेशपूजनम्")).locale("sa-Deva").build());
     * String text = said.value().text(); // हरिः ॐ तत्सत् …
     * }</pre>
     *
     * @param request the instant, the place, its clock, the slots, and optionally the tradition and locale
     * @return the words, sealed
     */
    public SankalpaWordsAnswer sankalpaWords(com.teispace.teistro.request.SankalpaWordsRequest request) {
        String json = Json.write(request);
        Map<?, ?> envelope = Reads.object(Json.read(
                context.locked((lib, raw) -> Calls.almanacSankalpaWords(lib, raw, json))));
        Map<?, ?> value = Reads.object(envelope, "value");
        return new SankalpaWordsAnswer(
                new SankalpaWords(
                        Reads.string(value, "tradition"),
                        Reads.string(value, "locale"),
                        java.util.Collections.unmodifiableMap(Reads.object(value, "params")),
                        Reads.string(value, "text")),
                com.teispace.teistro.record.Provenance.of(Reads.field(envelope, "provenance")));
    }

    /**
     * <b>The period of impurity after a birth or a death</b> at a place ({@code 03-design/ashauch.md}):
     * from when, until when, and by which clause. The span is a count of sunrise days, the day the
     * event falls in the first and an event at night given to a day by the table's night rule; {@code n}
     * days end at the sunrise that begins day {@code n + 1}. A period already running goes in the
     * request's {@code during}, each one an earlier answer's {@link Ashauch#running()}.
     *
     * <pre>{@code
     * AshauchAnswer found = sdk.almanac().ashauch(AshauchRequest.builder()
     *         .key("grandfather").relation(AshauchRelation.SAPINDA)
     *         .event(AshauchEventDeath.builder().at(2_461_051.677_083).build()) // 2026-01-11 10:00 NPT
     *         .build(), kathmandu, 20_700);
     * Integer days = found.value().days(); // 10
     * }</pre>
     *
     * @param request the event, the observer and the periods already running, its instants UTC Julian
     *     days
     * @param place where the days are counted
     * @param utcOffsetSeconds the local clock's offset from UTC, east positive
     * @param table the tradition's table, every row data; null for the <i>Dharmasindhu</i>'s
     * @return the period, sealed
     */
    public AshauchAnswer ashauch(com.teispace.teistro.request.AshauchRequest request, Observer place,
            int utcOffsetSeconds, com.teispace.teistro.request.AshauchTable table) {
        return ashauch(request, place, clock(utcOffsetSeconds), "table", table);
    }

    /**
     * {@link #ashauch(com.teispace.teistro.request.AshauchRequest, Observer, int,
     * com.teispace.teistro.request.AshauchTable)} on a zone's clock: each instant read at the offset the
     * zone had then.
     *
     * @param request the event, the observer and the periods already running
     * @param place where the days are counted
     * @param zone the zone each instant is read on, at its offset then, from {@code Civil.ianaZone} and its
     *     neighbours
     * @param table the tradition's table; null for the <i>Dharmasindhu</i>'s
     * @return the period, sealed
     */
    public AshauchAnswer ashauch(com.teispace.teistro.request.AshauchRequest request, Observer place,
            ZoneSpec zone, com.teispace.teistro.request.AshauchTable table) {
        return ashauch(request, place, clock(zone), "table", table);
    }

    /**
     * The period under the <i>Dharmasindhu</i>'s table: {@link #ashauch(com.teispace.teistro.request.AshauchRequest,
     * Observer, int, com.teispace.teistro.request.AshauchTable)} with no table.
     *
     * @param request the event, the observer and the periods already running
     * @param place where the days are counted
     * @param utcOffsetSeconds the local clock's offset from UTC, east positive
     * @return the period, sealed
     */
    public AshauchAnswer ashauch(com.teispace.teistro.request.AshauchRequest request, Observer place,
            int utcOffsetSeconds) {
        return ashauch(request, place, clock(utcOffsetSeconds), "table", null);
    }

    /**
     * The period under a table the SDK ships, named by its tradition ({@code 03-design/ashauch.md} §2.10).
     *
     * @param request the event, the observer and the periods already running
     * @param place where the days are counted
     * @param utcOffsetSeconds the local clock's offset from UTC, east positive
     * @param tradition the table, such as {@link com.teispace.teistro.request.AshauchTradition#MITAKSHARA}
     * @return the period, sealed
     */
    public AshauchAnswer ashauch(com.teispace.teistro.request.AshauchRequest request, Observer place,
            int utcOffsetSeconds, com.teispace.teistro.request.AshauchTradition tradition) {
        if (tradition == null) {
            throw Reads.invalid("tradition is an AshauchTradition", "tradition");
        }
        return ashauch(request, place, clock(utcOffsetSeconds), "tradition", tradition);
    }

    private AshauchAnswer ashauch(com.teispace.teistro.request.AshauchRequest request, Observer place,
            Map<String, Object> clock, String key, Object table) {
        if (request == null) {
            throw Reads.invalid("request is an AshauchRequest", "request");
        }
        if (place == null) {
            throw Reads.invalid("place is an Observer", "place");
        }
        Map<String, Object> asked = new java.util.LinkedHashMap<>();
        asked.put("latitudeDeg", place.latitudeDeg().value());
        asked.put("longitudeDeg", place.longitudeDeg().value());
        asked.put("altitudeM", place.altitudeM().value());
        asked.putAll(clock);
        if (table != null) {
            asked.put(key, table);
        }
        asked.put("request", request);
        String json = Json.write(asked);
        return AlmanacReads.ashauchAnswer(context.locked((lib, raw) -> Calls.almanacAshauch(lib, raw, json)));
    }

    /**
     * <b>The navatara chakra</b> of a birth star
     * ({@code 03-design/navatara-chakra.md}): the 27 stars in the order the
     * count from the birth star reaches them, each with its count, tara,
     * round of nine (1 janma, 2 anujanma, 3 trijanma) and the special star
     * <i>Muhurta Chintamani</i> names at that count, or null. It reads no sky.
     *
     * <pre>{@code
     * List<NavataraRow> chakra = sky.almanac().navatara(Nakshatra.ASHWINI);
     * SpecialStar karma = chakra.get(9).special(); // Magha, the 10th
     * }</pre>
     *
     * @param birth the birth star
     * @return the 27 rows
     * @throws TeistroException naming {@code birth} when it is null
     */
    public List<NavataraRow> navatara(Nakshatra birth) {
        if (birth == null) {
            throw Reads.invalid("birth is a Nakshatra, such as Nakshatra.ASHWINI", "birth");
        }
        String json = Json.write(Map.of("birth", birth.key()));
        return Reads.array(Json.read(context.locked((lib, raw) -> Calls.navatara(lib, raw, json)))).stream()
                .map(NavataraRow::read)
                .toList();
    }

    /**
     * <b>A building's measure read by remainders</b> under one vastu text
     * ({@code 03-design/aayadi.md}): the two sides in whole units of the
     * caller's choosing (1 to 1 000 000), the owner's star for the clauses
     * that read it, and for {@code MANASARA} only the measure its six
     * reckonings take. It answers each quantity the system reckons, the
     * vyaya's kind and the tara where it reads them, and each clause it
     * judges, never one verdict. It reads no sky.
     *
     * <pre>{@code
     * Aayadi reading = sky.almanac().aayadi(AayadiSystem.CHINTAMANI, 29, 7, Nakshatra.ANURADHA, null);
     * Nakshatra star = reading.get(AayadiQuantity.NAKSHATRA).nakshatra(); // Rohini
     * }</pre>
     *
     * @param system the text whose reckoning is read
     * @param length the length
     * @param breadth the breadth, in the same unit
     * @param owner the owner's star; may be null
     * @param measure for {@code MANASARA} only, the measure; may be null for the split
     * @return the reading
     * @throws TeistroException naming {@code system} when it is null, {@code aayadi.length} or
     *     {@code aayadi.breadth} for a side out of range, and {@code aayadi.measure} for a measure
     *     asked of another system
     */
    public Aayadi aayadi(
            com.teispace.teistro.request.AayadiSystem system,
            long length,
            long breadth,
            Nakshatra owner,
            com.teispace.teistro.request.ManasaraMeasure measure) {
        if (system == null) {
            throw Reads.invalid("system is an AayadiSystem, such as AayadiSystem.CHINTAMANI", "system");
        }
        Map<String, Object> asked = new java.util.LinkedHashMap<>();
        asked.put("system", system.key());
        asked.put("length", length);
        asked.put("breadth", breadth);
        if (owner != null) {
            asked.put("owner", owner.key());
        }
        if (measure != null) {
            asked.put("measure", measure.key());
        }
        String json = Json.write(asked);
        return Aayadi.read(Json.read(context.locked((lib, raw) -> Calls.aayadi(lib, raw, json))));
    }

    /**
     * <b>A household's periods of impurity</b> at a place ({@code 03-design/ashauch.md} §11): several
     * births and deaths and several observers in one request. Each observer's ties name an event by its
     * key and the observer's relation to it; they are read in the order the news reached that observer,
     * each against the ones before it (DS p. 305: the news, not the event, decides).
     *
     * <pre>{@code
     * HouseholdAnswer found = sdk.almanac().household(Household.builder()
     *         .events(List.of(HouseholdEvent.builder().key("grandfather")
     *                 .event(AshauchEventDeath.builder().at(2_461_051.677_083).build()).build()))
     *         .observers(List.of(HouseholdObserver.builder().key("son")
     *                 .ties(List.of(HouseholdTie.builder().event("grandfather").relation(AshauchRelation.CHILD).build()))
     *                 .build()))
     *         .build(), kathmandu, 20_700, null);
     * double pure = found.value().observers().get(0).pureAt();
     * }</pre>
     *
     * @param household the events and the observers, its instants UTC Julian days
     * @param place where the days are counted
     * @param utcOffsetSeconds the local clock's offset from UTC, east positive
     * @param table the tradition's table, every row data; null for the <i>Dharmasindhu</i>'s
     * @return each observer's periods, sealed
     */
    public HouseholdAnswer household(com.teispace.teistro.request.Household household, Observer place,
            int utcOffsetSeconds, com.teispace.teistro.request.AshauchTable table) {
        return household(household, place, clock(utcOffsetSeconds), table);
    }

    /**
     * {@link #household(com.teispace.teistro.request.Household, Observer, int, com.teispace.teistro.request.AshauchTable)}
     * on a zone's clock: each instant read at the offset the zone had then.
     *
     * @param household the events and the observers
     * @param place where the days are counted
     * @param zone the zone each instant is read on, at its offset then, from {@code Civil.ianaZone} and its
     *     neighbours
     * @param table the tradition's table; null for the <i>Dharmasindhu</i>'s
     * @return each observer's periods, sealed
     */
    public HouseholdAnswer household(com.teispace.teistro.request.Household household, Observer place,
            ZoneSpec zone, com.teispace.teistro.request.AshauchTable table) {
        return household(household, place, clock(zone), table);
    }

    private HouseholdAnswer household(com.teispace.teistro.request.Household household, Observer place,
            Map<String, Object> clock, com.teispace.teistro.request.AshauchTable table) {
        if (household == null) {
            throw Reads.invalid("household is a Household", "household");
        }
        if (place == null) {
            throw Reads.invalid("place is an Observer", "place");
        }
        Map<String, Object> asked = new java.util.LinkedHashMap<>();
        asked.put("latitudeDeg", place.latitudeDeg().value());
        asked.put("longitudeDeg", place.longitudeDeg().value());
        asked.put("altitudeM", place.altitudeM().value());
        asked.putAll(clock);
        if (table != null) {
            asked.put("table", table);
        }
        asked.put("household", household);
        String json = Json.write(asked);
        return AlmanacReads.householdAnswer(context.locked((lib, raw) -> Calls.almanacHousehold(lib, raw, json)));
    }

    /**
     * <b>The ashauch relation a path in a family gives</b> ({@code 03-design/ashauch.md} §10): the steps
     * from the observer to the one who died or was born, read as the <i>Dharmasindhu</i> counts kin (pp.
     * 299–302). Needs no place or ephemeris; a path that steps back is refused as
     * {@code kin.kin.path[n]}.
     *
     * <pre>{@code
     * Kin kin = sdk.almanac().ashauchKin(KinQuery.builder().kin(KinAsked.builder()
     *         .path(List.of(KinStep.FATHER, KinStep.SON))
     *         .observer(KinObserver.builder().sex(AshauchSex.MALE).build()).build()).build());
     * AshauchRelation relation = kin.relation(); // SAPINDA
     * }</pre>
     *
     * @param query the path, the facts it reads, and the table (the <i>Dharmasindhu</i>'s when left out)
     * @return the relation, what is kept and the clause
     */
    public Kin ashauchKin(com.teispace.teistro.request.KinQuery query) {
        if (query == null) {
            throw Reads.invalid("query is a KinQuery", "kin");
        }
        String json = Json.write(query);
        return AlmanacReads.kin(context.locked((lib, raw) -> Calls.almanacAshauchKin(lib, raw, json)));
    }

    /**
     * <b>Whether a rite may go on during a period of impurity</b> ({@code 03-design/ashauch.md} §12):
     * goes on, in part, or waits, what opens the rite and the clause, from the <i>Dharmasindhu</i>'s
     * exceptions (pp. 309–310) when the query names no table. Needs no place or ephemeris.
     *
     * <pre>{@code
     * AshauchRiteAnswer answer = sdk.almanac().ashauchRite(RiteQuery.builder().rite(AshauchRiteAsked.builder()
     *         .rite(AshauchRite.MARRIAGE).begun(true).noOtherWay(true).build()).build());
     * RiteVerdict verdict = answer.verdict(); // GOES_ON
     * }</pre>
     *
     * @param query the rite, the facts it reads, and the table
     * @return the verdict and its clause
     */
    public AshauchRiteAnswer ashauchRite(com.teispace.teistro.request.RiteQuery query) {
        if (query == null) {
            throw Reads.invalid("query is a RiteQuery", "rite");
        }
        String json = Json.write(query);
        return AlmanacReads.ashauchRite(context.locked((lib, raw) -> Calls.almanacAshauchRite(lib, raw, json)));
    }

    /**
     * <b>The death-time check</b> at a place ({@code 03-design/death-time.md}): at the death and at the
     * burning, the nakshatra, the tithi, the vara and the Moon's sign, whether it is the panchaka, the
     * nakshatra's feet and any pushkara yoga; and the rites the <i>Dharmasindhu</i> ties to them (pp.
     * 334–335), each with its clause. With no burning given it is taken to fall as the death did; one
     * before the death is refused as {@code deathTime.crematedAt}.
     *
     * <pre>{@code
     * DeathTimeAnswer found = sdk.almanac().deathTime(2_461_051.677_083, null, kathmandu, 20_700, null);
     * boolean panchaka = found.value().death().panchaka();
     * }</pre>
     *
     * @param at the death, a UTC Julian day
     * @param crematedAt the burning, a UTC Julian day; null when it falls as the death did
     * @param place where the day and the sky are read
     * @param utcOffsetSeconds the local clock's offset from UTC, east positive
     * @param table the tradition's lists, every one data; null for the <i>Dharmasindhu</i>'s
     * @return the readings and the rites, sealed
     */
    public DeathTimeAnswer deathTime(double at, Double crematedAt, Observer place, int utcOffsetSeconds,
            com.teispace.teistro.request.DeathTimeTable table) {
        return deathTime(at, crematedAt, place, clock(utcOffsetSeconds), table);
    }

    /**
     * {@link #deathTime(double, Double, Observer, int, com.teispace.teistro.request.DeathTimeTable)}
     * on a zone's clock: each instant read at the offset the zone had then.
     *
     * @param at the death, a UTC Julian day
     * @param crematedAt the burning, a UTC Julian day; null when it falls as the death did
     * @param place where the day and the sky are read
     * @param zone the zone each instant is read on, at its offset then, from {@code Civil.ianaZone} and its
     *     neighbours
     * @param table the tradition's lists; null for the <i>Dharmasindhu</i>'s
     * @return the readings and the rites, sealed
     */
    public DeathTimeAnswer deathTime(double at, Double crematedAt, Observer place, ZoneSpec zone,
            com.teispace.teistro.request.DeathTimeTable table) {
        return deathTime(at, crematedAt, place, clock(zone), table);
    }

    private DeathTimeAnswer deathTime(double at, Double crematedAt, Observer place, Map<String, Object> clock,
            com.teispace.teistro.request.DeathTimeTable table) {
        if (place == null) {
            throw Reads.invalid("place is an Observer", "place");
        }
        Map<String, Object> asked = new java.util.LinkedHashMap<>();
        asked.put("latitudeDeg", place.latitudeDeg().value());
        asked.put("longitudeDeg", place.longitudeDeg().value());
        asked.put("altitudeM", place.altitudeM().value());
        asked.putAll(clock);
        asked.put("at", at);
        if (crematedAt != null) {
            asked.put("crematedAt", crematedAt);
        }
        if (table != null) {
            asked.put("table", table);
        }
        String json = Json.write(asked);
        return AlmanacReads.deathTimeAnswer(context.locked((lib, raw) -> Calls.almanacDeathTime(lib, raw, json)));
    }

    /**
     * <b>The sky at a place and an instant</b> ({@code 03-design/sky-view.md}): where each graha and each
     * nakshatra's junction star stands, topocentric, apparent and of date, its altitude airless and refracted and
     * its azimuth from north through east; and the ecliptic, with the ascendant and the descendant, and the
     * horizon as point lists. {@code grahas} and {@code stars} are the nine and the twenty-seven yogataras when
     * null; each step is 0.1 to 90 degrees, five when null; the air is 1010 hPa and 10 degrees Celsius when null,
     * and a pressure of 0 turns refraction off. A context over a classical astronomy is refused (C1185).
     *
     * <pre>{@code
     * SkyViewAnswer sky = sdk.almanac().skyView(2_461_345.25, kathmandu, null, null, null, null, null, null);
     * double moon = sky.value().graha(Graha.MOON).orElseThrow().azimuthDeg();
     * }</pre>
     *
     * @param at the instant, a UTC Julian day, read as UT1
     * @param place where the sky is seen
     * @param grahas the grahas placed, in this order; the nine when null
     * @param stars the stars placed, in this order; the twenty-seven yogataras when null
     * @param eclipticStepDeg the ecliptic's step in longitude; 5 when null
     * @param horizonStepDeg the horizon's step in azimuth; 5 when null
     * @param pressureHpa the air's pressure; 1010 when null
     * @param temperatureC the air's temperature; 10 when null
     * @return the sky, sealed
     */
    public SkyViewAnswer skyView(double at, Observer place, List<Graha> grahas, List<Star> stars,
            Double eclipticStepDeg, Double horizonStepDeg, Double pressureHpa, Double temperatureC) {
        if (place == null) {
            throw Reads.invalid("place is an Observer", "place");
        }
        Map<String, Object> asked = new java.util.LinkedHashMap<>();
        asked.put("latitudeDeg", place.latitudeDeg().value());
        asked.put("longitudeDeg", place.longitudeDeg().value());
        asked.put("altitudeM", place.altitudeM().value());
        asked.put("at", at);
        if (grahas != null) {
            asked.put("grahas", grahas.stream().map(Graha::key).toList());
        }
        if (stars != null) {
            asked.put("stars", stars.stream().map(Star::key).toList());
        }
        if (eclipticStepDeg != null) {
            asked.put("eclipticStepDeg", eclipticStepDeg);
        }
        if (horizonStepDeg != null) {
            asked.put("horizonStepDeg", horizonStepDeg);
        }
        if (pressureHpa != null) {
            asked.put("pressureHpa", pressureHpa);
        }
        if (temperatureC != null) {
            asked.put("temperatureC", temperatureC);
        }
        String json = Json.write(asked);
        return AlmanacReads.skyViewAnswer(context.locked((lib, raw) -> Calls.almanacSkyView(lib, raw, json)));
    }

    /**
     * <b>A death's first year</b> at a place ({@code 03-design/shraddha.md} §3.3): the
     * <i>Dharmasindhu</i>'s sixteen shraddhas, the first abdika and the sapindana, each on its day. The
     * eleventh day is the ashauch's day one plus ten, and every day kept on a tithi is the festival
     * evaluator's, so a monthly, an anniversary and an ashauch never judge a day differently. Every
     * reading the request leaves out is the <i>Dharmasindhu</i>'s.
     *
     * <pre>{@code
     * FirstYearAnswer found = sky.almanac().firstYear(FirstYearRequest.builder()
     *         .key("GRANDFATHER").at(2_461_333.677_083)
     *         .firstAbdika(FirstAbdika.ELEVENTH_DAY).firstYearAdhika(FirstYearAdhika.NIJA) // Nepal's
     *         .build(), kathmandu, 20_700);
     * CalendarDate sapindana = found.value().sapindana();
     * }</pre>
     *
     * @param request the death, a UTC Julian day, and the readings
     * @param place where the days are kept
     * @param utcOffsetSeconds the local clock's offset from UTC, east positive
     * @return the schedule, sealed
     */
    public FirstYearAnswer firstYear(com.teispace.teistro.request.FirstYearRequest request, Observer place,
            int utcOffsetSeconds) {
        return firstYear(request, place, clock(utcOffsetSeconds));
    }

    /**
     * {@link #firstYear(com.teispace.teistro.request.FirstYearRequest, Observer, int)}
     * on a zone's clock: each instant read at the offset the zone had then.
     *
     * @param request the death, a UTC Julian day, and the readings
     * @param place where the days are kept
     * @param zone the zone each instant is read on, at its offset then, from {@code Civil.ianaZone} and its
     *     neighbours
     * @return the schedule, sealed
     */
    public FirstYearAnswer firstYear(com.teispace.teistro.request.FirstYearRequest request, Observer place,
            ZoneSpec zone) {
        return firstYear(request, place, clock(zone));
    }

    private FirstYearAnswer firstYear(com.teispace.teistro.request.FirstYearRequest request, Observer place,
            Map<String, Object> clock) {
        if (request == null) {
            throw Reads.invalid("request is a FirstYearRequest", "request");
        }
        if (place == null) {
            throw Reads.invalid("place is an Observer", "place");
        }
        Map<String, Object> asked = new java.util.LinkedHashMap<>();
        asked.put("latitudeDeg", place.latitudeDeg().value());
        asked.put("longitudeDeg", place.longitudeDeg().value());
        asked.put("altitudeM", place.altitudeM().value());
        asked.putAll(clock);
        asked.put("request", request);
        String json = Json.write(asked);
        return AlmanacReads.firstYearAnswer(context.locked((lib, raw) -> Calls.almanacFirstYear(lib, raw, json)));
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
     *         true, new FestivalRequest(FestivalRules.NEPAL));
     * }</pre>
     *
     * @param other the reckoning compared with this one
     * @param from the first day
     * @param to the last day, both ends included
     * @param place where both are reckoned
     * @param utcOffsetSeconds the local clock's offset from UTC, east positive
     * @param days whether the days are compared
     * @param festivals the festival rules compared, such as {@code new FestivalRequest(FestivalRules.NEPAL)}, or null for
     *     none
     * @return each comparison asked, sealed by this context's provenance
     */
    public Disagreements disagreements(Context other, CalendarDate from, CalendarDate to, Observer place,
            int utcOffsetSeconds, boolean days, com.teispace.teistro.request.FestivalRequest festivals) {
        return disagreements(other, from, to, place, clock(utcOffsetSeconds), days, festivals);
    }

    /**
     * {@link #disagreements(Context, CalendarDate, CalendarDate, Observer, int, boolean,
     * com.teispace.teistro.request.FestivalRequest)} on a zone's clock: each day read at the offset the
     * zone had then ({@code 03-design/time-and-timezone.md} §3.4).
     *
     * <pre>{@code
     * Disagreements found = official.almanac().disagreements(modern, first, last, kathmandu,
     *         Civil.ianaZone("Asia/Kathmandu"), true, null);
     * }</pre>
     *
     * @param other the reckoning compared with this one
     * @param from the first day
     * @param to the last day, both ends included
     * @param place where both are reckoned
     * @param zone the zone each day is read on, from {@code Civil.ianaZone} and its neighbours
     * @param days whether the days are compared
     * @param festivals the festival rules compared, or null for none
     * @return each comparison asked, sealed by this context's provenance
     */
    public Disagreements disagreements(Context other, CalendarDate from, CalendarDate to, Observer place,
            ZoneSpec zone, boolean days, com.teispace.teistro.request.FestivalRequest festivals) {
        return disagreements(other, from, to, place, clock(zone), days, festivals);
    }

    private Disagreements disagreements(Context other, CalendarDate from, CalendarDate to, Observer place,
            Map<String, Object> clock, boolean days, com.teispace.teistro.request.FestivalRequest festivals) {
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
        asked.putAll(clock);
        asked.put("days", days);
        if (festivals != null) {
            asked.put("festivals", festivals);
        }
        String json = Json.write(asked);
        return AlmanacReads.disagreements(context.lockedWith(other,
                (lib, ours, theirs) -> Calls.almanacDisagreements(lib, ours, theirs, json)));
    }

    /**
     * <b>Why each published day differs from the rule's</b> ({@code 03-design/published-overrides.md} §4).
     * The festival request, with its {@code published} sets, is asked again under one change at a time:
     * each of {@code places} the publisher may have reckoned at, then {@code sky} (another context: the
     * committee's Surya Siddhanta beside a modern sky), then each shipped pack's rule of the same key. The
     * first change whose rule's day is the published one is the reason, else {@code PUBLISHER}; every
     * change tried is listed.
     *
     * <pre>{@code
     * PublishedReasons found = modern.almanac().publishedReasons(first, last, kathmandu, 20_700,
     *         new FestivalRequest(List.of(set), FestivalRules.NEPAL), List.of(), committee);
     * }</pre>
     *
     * @param from the first day
     * @param to the last day, both ends included
     * @param place where the days are reckoned
     * @param utcOffsetSeconds the local clock's offset from UTC, east positive
     * @param festivals the festival request with the published sets to explain
     * @param places the places tried, in order
     * @param sky the publisher's sky, or null to try none
     * @return each published day explained, sealed by this context's provenance
     */
    public PublishedReasons publishedReasons(CalendarDate from, CalendarDate to, Observer place,
            int utcOffsetSeconds, com.teispace.teistro.request.FestivalRequest festivals,
            List<PublishedReasons.Place> places, Context sky) {
        if (from == null || to == null) {
            throw Reads.invalid("from and to are CalendarDates", from == null ? "from" : "to");
        }
        if (place == null) {
            throw Reads.invalid("place is an Observer", "place");
        }
        if (festivals == null) {
            throw Reads.invalid("festivals is a FestivalRequest", "festivals");
        }
        Map<String, Object> asked = new java.util.LinkedHashMap<>();
        asked.put("calendar", from.calendar().key());
        asked.put("first", Map.of("year", from.year(), "month", from.month(), "day", from.day()));
        asked.put("last", Map.of("year", to.year(), "month", to.month(), "day", to.day()));
        asked.put("latitudeDeg", place.latitudeDeg().value());
        asked.put("longitudeDeg", place.longitudeDeg().value());
        asked.put("altitudeM", place.altitudeM().value());
        asked.put("utcOffsetSeconds", utcOffsetSeconds);
        asked.put("festivals", festivals);
        asked.put("places", (places == null ? List.<PublishedReasons.Place>of() : places).stream()
                .map(p -> Map.of("name", p.name(), "latitudeDeg", p.place().latitudeDeg().value(),
                        "longitudeDeg", p.place().longitudeDeg().value(), "altitudeM", p.place().altitudeM().value(),
                        "utcOffsetSeconds", p.utcOffsetSeconds()))
                .toList());
        String json = Json.write(asked);
        // This context as the other tries no sky.
        return AlmanacReads.publishedReasons(context.lockedWith(sky == null ? context : sky,
                (lib, ours, theirs) -> Calls.almanacPublishedReasons(lib, ours, theirs, json)));
    }

    /**
     * <b>A range's observances as an iCalendar feed</b> (RFC 5545, {@code 03-design/icalendar.md}) a
     * calendar app subscribes to: one whole-day event for each observance and Ekadashi fast, its summary
     * the festival's name in this context's locale, its description the rule's source and the answer's
     * provenance, its {@code UID} stable for the rule, the day and the place. The same request writes the
     * same bytes.
     *
     * <pre>{@code
     * String ics = sdk.almanac().festivalCalendar(first, last, kathmandu, 20_700,
     *         new FestivalRequest(FestivalRules.NEPAL_PANCHANGA), FeedOptions.defaults().withName("Nepal 2026"));
     * Files.writeString(Path.of("nepal-2026.ics"), ics);
     * }</pre>
     *
     * @param from the first day
     * @param to the last day, both ends included
     * @param place where the days are reckoned
     * @param utcOffsetSeconds the local clock's offset from UTC, east positive
     * @param festivals the festival rules, such as {@code new FestivalRequest(FestivalRules.NEPAL)}
     * @param feed what the feed says of itself
     * @return the feed, CRLF lines
     */
    public String festivalCalendar(CalendarDate from, CalendarDate to, Observer place, int utcOffsetSeconds,
            com.teispace.teistro.request.FestivalRequest festivals, FeedOptions feed) {
        if (from == null || to == null) {
            throw Reads.invalid("from and to are CalendarDates", from == null ? "from" : "to");
        }
        if (place == null) {
            throw Reads.invalid("place is an Observer", "place");
        }
        if (festivals == null) {
            throw Reads.invalid("festivals is a FestivalRequest", "festivals");
        }
        FeedOptions options = feed == null ? FeedOptions.defaults() : feed;
        Map<String, Object> asked = new java.util.LinkedHashMap<>();
        asked.put("calendar", from.calendar().key());
        asked.put("first", Map.of("year", from.year(), "month", from.month(), "day", from.day()));
        asked.put("last", Map.of("year", to.year(), "month", to.month(), "day", to.day()));
        asked.put("latitudeDeg", place.latitudeDeg().value());
        asked.put("longitudeDeg", place.longitudeDeg().value());
        asked.put("altitudeM", place.altitudeM().value());
        asked.put("utcOffsetSeconds", utcOffsetSeconds);
        asked.put("festivals", festivals);
        asked.put("stampUnixSeconds", options.stampUnixSeconds() == null
                ? System.currentTimeMillis() / 1000 : options.stampUnixSeconds());
        if (options.name() != null) {
            asked.put("name", options.name());
        }
        if (options.product() != null) {
            asked.put("product", options.product());
        }
        if (options.noRefresh()) {
            asked.put("refreshMinutes", 0);
        } else if (options.refreshMinutes() != null) {
            asked.put("refreshMinutes", options.refreshMinutes());
        }
        String json = Json.write(asked);
        return context.locked((lib, raw) -> Calls.almanacFestivalCalendar(lib, raw, json));
    }

    /**
     * A feed from the whole request record, for what {@link FeedOptions} does not hold: the caller's own
     * events, a filter, hints, an alarm, categories and a colour ({@code 03-design/icalendar.md} §6).
     *
     * <pre>{@code
     * String ics = sdk.almanac().festivalCalendar(FestivalCalendarRequest.builder()
     *         .first(first).latitudeDeg(27.7172).longitudeDeg(85.324).utcOffsetSeconds(20_700)
     *         .festivals(festivals).stampUnixSeconds(1_767_225_600L).color("gold").build());
     * }</pre>
     *
     * @param asked the request record
     * @return the feed, CRLF lines
     */
    public String festivalCalendar(com.teispace.teistro.request.FestivalCalendarRequest asked) {
        if (asked == null) {
            throw Reads.invalid("asked is a FestivalCalendarRequest", "asked");
        }
        String json = Json.write(asked.toJson());
        return context.locked((lib, raw) -> Calls.almanacFestivalCalendar(lib, raw, json));
    }

    /**
     * <b>The N-day digest</b> ({@code 03-design/digest-format.md}): the days from {@code first}, every limb
     * and window, the Moon and the observances, as bytes a process with no engine reads with
     * {@code bindings/c/digest}, and {@link com.teispace.teistro.blob.Digest#decode(byte[])} reads here. The
     * same request writes the same bytes on every platform.
     *
     * <pre>{@code
     * byte[] bytes = sdk.almanac().digest(DigestRequest.builder()
     *         .first(first).latitudeDeg(27.7172).longitudeDeg(85.324).utcOffsetSeconds(20_700).build());
     * }</pre>
     *
     * @param asked the request record
     * @return the digest's bytes
     */
    public byte[] digest(com.teispace.teistro.request.DigestRequest asked) {
        if (asked == null) {
            throw Reads.invalid("asked is a DigestRequest", "asked");
        }
        String json = Json.write(asked.toJson());
        return context.locked((lib, raw) -> Calls.almanacDigest(lib, raw, json));
    }

    /**
     * As {@link #festivalCalendar(CalendarDate, CalendarDate, Observer, int,
     * com.teispace.teistro.request.FestivalRequest, FeedOptions)}, the feed saying of itself what the SDK
     * says: "Festivals", refreshed daily, stamped now.
     *
     * @param from the first day
     * @param to the last day, both ends included
     * @param place where the days are reckoned
     * @param utcOffsetSeconds the local clock's offset from UTC, east positive
     * @param festivals the festival rules
     * @return the feed, CRLF lines
     */
    public String festivalCalendar(CalendarDate from, CalendarDate to, Observer place, int utcOffsetSeconds,
            com.teispace.teistro.request.FestivalRequest festivals) {
        return festivalCalendar(from, to, place, utcOffsetSeconds, festivals, FeedOptions.defaults());
    }

    /**
     * Every rule of a festival pack with what it is: {@code {pack, rules}}, each
     * row {@code {key, form, source, meta, unsourced}}, {@code meta} its kind,
     * tradition, communities, regions and fast and {@code unsourced} the fields no
     * source gives ({@code 03-design/festival-metadata.md}).
     *
     * <pre>{@code
     * Map<?, ?> listed = sdk.almanac().festivalPack("INDIA_CENTRAL");
     * }</pre>
     *
     * @param pack a pack the SDK ships
     * @return the pack's rows
     * @throws TeistroException {@code INVALID_ARG} for a pack it does not ship
     */
    public Map<?, ?> festivalPack(String pack) {
        if (pack == null) {
            throw Reads.invalid("pack is a pack's name", "pack");
        }
        String json = Json.write(Map.of("pack", pack));
        return Reads.object(Json.read(context.locked((lib, raw) -> Calls.almanacFestivalPack(lib, raw, json))));
    }

    /**
     * <b>Anniversaries kept by the tithi over a span of years</b>, for many people in one call
     * ({@code 03-design/anniversary-span.md}). The span's days are founded once, and every
     * person's are read from them. Each day's {@code dayRule} says whether its tithi was
     * {@code KSHAYA} or {@code VRIDDHI}, the window read, the clause and its source.
     *
     * <pre>{@code
     * AnniversariesRequest asked = AnniversariesRequest.of(Map.of("fromYear", 2026, "years", 10,
     *         "latitudeDeg", 27.7172, "longitudeDeg", 85.324, "utcOffsetSeconds", 20_700,
     *         "anniversaries", List.of(Map.of("key", "GRANDFATHER",
     *                 "origin", Map.of("by", "LUNAR", "month", "KARTIKA", "tithi", "SHUKLA_DASHAMI")))));
     * FestivalAnswer span = sdk.almanac().anniversaries(asked);
     * System.out.println(span.anniversaries().get(0).dayRule().clause());
     * }</pre>
     *
     * @param request the span in {@code calendar}, the place, and at most 1000 anniversaries
     * @return the festival answer, its anniversaries holding each person's days
     * @throws TeistroException {@code OUT_OF_RANGE} naming {@code years}, {@code LIMIT} naming
     *     {@code anniversaries}
     */
    public FestivalAnswer anniversaries(com.teispace.teistro.request.AnniversariesRequest request) {
        String json = Json.write(request);
        return AlmanacReads.festivalsAnswer(context.locked((lib, raw) -> Calls.almanacAnniversaries(lib, raw, json)));
    }

    /**
     * <b>The next day a rule or a kind of day falls on</b> after an instant at a place, with no year
     * asked ({@code 03-design/next-occurrence.md}). The civil day holding the instant counts. Give a
     * {@code kind}, {@code EKADASHI}, {@code PURNIMA}, {@code AMAVASYA} or {@code SANKRANTI}, read by
     * rules the SDK states, or a {@code rule} with the {@code festivals} that carry it; the search looks
     * {@code limitDays} ahead, 400 when null and at most 3660.
     *
     * <pre>{@code
     * NextOccurrence next = sdk.almanac().next(2461345.5, kathmandu, 20_700, "EKADASHI", null, null, null, null);
     * System.out.println(next.keys().get(0) + " on " + next.day());
     * }</pre>
     *
     * @param after the instant, a UTC Julian day
     * @param place where the days are reckoned
     * @param utcOffsetSeconds the local clock's offset from UTC, east positive
     * @param kind a kind of day, or null with a rule
     * @param rule a rule's key in {@code festivals}, or null with a kind
     * @param festivals the festival rules carrying {@code rule}, null with a kind
     * @param calendar the calendar the days are written in, null for Gregorian
     * @param limitDays how many days to look ahead, null for 400
     * @return the day found
     * @throws TeistroException {@code LIMIT} naming {@code limitDays} when nothing falls within it
     */
    public NextOccurrence next(double after, Observer place, int utcOffsetSeconds, String kind, String rule,
            com.teispace.teistro.request.FestivalRequest festivals, Calendar calendar, Integer limitDays) {
        return next(after, place, clock(utcOffsetSeconds), kind, rule, festivals, calendar, limitDays);
    }

    /**
     * {@link #next(double, Observer, int, String, String, com.teispace.teistro.request.FestivalRequest,
     * Calendar, Integer)} on a zone's clock: each day read at the offset the zone had then.
     *
     * <pre>{@code
     * NextOccurrence next = sdk.almanac().next(2461345.5, kathmandu, Civil.ianaZone("Asia/Kathmandu"),
     *         "EKADASHI", null, null, null, null);
     * }</pre>
     *
     * @param after the instant, a UTC Julian day
     * @param place where the days are reckoned
     * @param zone the zone each day is read on, from {@code Civil.ianaZone} and its neighbours
     * @param kind a kind of day, or null with a rule
     * @param rule a rule's key in {@code festivals}, or null with a kind
     * @param festivals the festival rules carrying {@code rule}, null with a kind
     * @param calendar the calendar the days are written in, null for Gregorian
     * @param limitDays how many days to look ahead, null for 400
     * @return the day found
     */
    public NextOccurrence next(double after, Observer place, ZoneSpec zone, String kind, String rule,
            com.teispace.teistro.request.FestivalRequest festivals, Calendar calendar, Integer limitDays) {
        return next(after, place, clock(zone), kind, rule, festivals, calendar, limitDays);
    }

    private NextOccurrence next(double after, Observer place, Map<String, Object> clock, String kind, String rule,
            com.teispace.teistro.request.FestivalRequest festivals, Calendar calendar, Integer limitDays) {
        if (place == null) {
            throw Reads.invalid("place is an Observer", "place");
        }
        Map<String, Object> asked = new java.util.LinkedHashMap<>();
        asked.put("after", after);
        asked.put("latitudeDeg", place.latitudeDeg().value());
        asked.put("longitudeDeg", place.longitudeDeg().value());
        asked.put("altitudeM", place.altitudeM().value());
        asked.putAll(clock);
        if (calendar != null) {
            asked.put("calendar", calendar.key());
        }
        if (kind != null) {
            asked.put("kind", kind);
        }
        if (rule != null) {
            asked.put("rule", rule);
        }
        if (festivals != null) {
            asked.put("festivals", festivals);
        }
        if (limitDays != null) {
            asked.put("limitDays", limitDays);
        }
        String json = Json.write(asked);
        Map<?, ?> envelope = Reads.object(Json.read(context.locked((lib, raw) -> Calls.almanacNext(lib, raw, json))));
        Map<?, ?> value = Reads.object(envelope, "value");
        Map<String, Object> section = new java.util.LinkedHashMap<>();
        section.put("value", value.get("festivals"));
        section.put("provenance", envelope.get("provenance"));
        return new NextOccurrence(
                Reads.serdeDate(Reads.field(value, "day")),
                Reads.each(value, "keys", Reads::string),
                AlmanacReads.festivalsAnswer(Json.write(section)),
                Reads.serdeDate(Reads.field(value, "from")),
                Reads.integer(value, "limitDays"),
                Reads.integer(value, "searchedDays"));
    }

    /**
     * <b>A day in sentences</b> ({@code 03-design/day-in-sentences.md}): one date's facts as short sentences
     * from templates, every value from the day's answer.
     *
     * <pre>{@code
     * DaySentences day = sdk.almanac().sentences(date, kathmandu, 20_700, "ne-Deva-NP");
     * System.out.println(day.sentences().get(0).text());
     * }</pre>
     *
     * @param date the day
     * @param place where the day is reckoned
     * @param utcOffsetSeconds the local clock's offset from UTC, east positive
     * @param locale the locale, null for the context's
     * @return the sentences
     * @throws TeistroException {@code UNSUPPORTED} naming {@code locale} for a locale without the templates
     */
    public DaySentences sentences(CalendarDate date, Observer place, int utcOffsetSeconds, String locale) {
        return sentences(date, place, clock(utcOffsetSeconds), locale);
    }

    /**
     * {@link #sentences(CalendarDate, Observer, int, String)} on a zone's clock: the day read at the
     * offset the zone had then.
     *
     * <pre>{@code
     * DaySentences day = sdk.almanac().sentences(date, kathmandu, Civil.ianaZone("Asia/Kathmandu"), "en-Latn");
     * }</pre>
     *
     * @param date the day
     * @param place where the day is reckoned
     * @param zone the zone the day is read on, from {@code Civil.ianaZone} and its neighbours
     * @param locale the locale, null for the context's
     * @return the sentences
     */
    public DaySentences sentences(CalendarDate date, Observer place, ZoneSpec zone, String locale) {
        return sentences(date, place, clock(zone), locale);
    }

    private DaySentences sentences(CalendarDate date, Observer place, Map<String, Object> clock, String locale) {
        if (date == null) {
            throw Reads.invalid("date is a CalendarDate", "date");
        }
        if (place == null) {
            throw Reads.invalid("place is an Observer", "place");
        }
        Map<String, Object> day = new java.util.LinkedHashMap<>();
        day.put("year", date.year());
        day.put("month", date.month());
        day.put("day", date.day());
        Map<String, Object> asked = new java.util.LinkedHashMap<>();
        asked.put("calendar", date.calendar().key());
        asked.put("date", day);
        asked.put("latitudeDeg", place.latitudeDeg().value());
        asked.put("longitudeDeg", place.longitudeDeg().value());
        asked.put("altitudeM", place.altitudeM().value());
        asked.putAll(clock);
        if (locale != null) {
            asked.put("locale", locale);
        }
        String json = Json.write(asked);
        Map<?, ?> envelope =
                Reads.object(Json.read(context.locked((lib, raw) -> Calls.almanacSentences(lib, raw, json))));
        Map<?, ?> value = Reads.object(envelope, "value");
        return new DaySentences(
                Reads.string(value, "locale"),
                Reads.flag(value, "draft"),
                Reads.each(value, "sentences", row -> {
                    Map<?, ?> sentence = Reads.object(row);
                    return new DaySentences.Sentence(
                            Reads.string(sentence, "key"),
                            Reads.object(sentence, "params"),
                            Reads.string(sentence, "text"));
                }),
                Reads.each(value, "withheld", Reads::string));
    }

    /**
     * <b>The times a muhurta rule set leaves, ranked</b> ({@code 03-design/election-search.md}): the
     * search {@code days} runs with {@code muhurta}, asked on its own, each window ranked from 1 with
     * the clauses for it ({@code holds}) and against it ({@code fails}). The record keeps the wire's
     * spelling: catalogue members are bare keys.
     *
     * @param request the days, the place and the rule set
     * @return the ranked windows, sealed
     */
    public FinderAnswer elect(com.teispace.teistro.request.ElectRequest request) {
        if (request == null) {
            throw Reads.invalid("request is an ElectRequest", "request");
        }
        String json = Json.write(request);
        return FinderAnswer.of(context.locked((lib, raw) -> Calls.almanacElect(lib, raw, json)));
    }

    /**
     * <b>When an observance falls under each tradition</b> ({@code 03-design/observance-finder.md}):
     * each tradition's rule over a year or a range, the occurrences paired by their tithi, and where
     * they part what differs and why ({@code RULE}, {@code SKY}, {@code RULE_AND_SKY},
     * {@code PUBLISHED}). A tradition reckoned {@code OTHER} is reckoned under {@code other}.
     *
     * @param request the range, the place and the traditions
     * @param other the context an {@code OTHER} tradition is reckoned under, or null for none
     * @return the occurrences, sealed by the first tradition's reckoning
     */
    public FinderAnswer observance(com.teispace.teistro.request.ObservanceRequest request, Context other) {
        if (request == null) {
            throw Reads.invalid("request is an ObservanceRequest", "request");
        }
        String json = Json.write(request);
        // This context as the other is no other.
        return FinderAnswer.of(context.lockedWith(other == null ? context : other,
                (lib, ours, theirs) -> Calls.almanacObservance(lib, ours, theirs, json)));
    }

    /**
     * <b>A day for many people at once</b> ({@code 03-design/people-digest.md}): each birth's running
     * dasha, every graha's gochar from their Moon, the day's transit hits on their chart, the day's tara
     * and the Moon's house from theirs, the day running sunrise to sunrise at the place.
     *
     * @param request the day, the place and the births
     * @return each person's day, sealed
     */
    public FinderAnswer people(com.teispace.teistro.request.PeopleRequest request) {
        if (request == null) {
            throw Reads.invalid("request is a PeopleRequest", "request");
        }
        String json = Json.write(request);
        return FinderAnswer.of(context.locked((lib, raw) -> Calls.almanacPeople(lib, raw, json)));
    }
}

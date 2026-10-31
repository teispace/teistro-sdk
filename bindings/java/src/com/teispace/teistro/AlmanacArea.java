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
        return asked(fromDate, toDate, place, utcOffsetSeconds, null, muhurta, festivals,
                (years ? Native.TS_PANCHANGA_YEARS : 0)
                        | (eclipses ? Native.TS_PANCHANGA_ECLIPSES : 0)
                        | (nepalSambat ? Native.TS_PANCHANGA_NEPAL_SAMBAT : 0)
                        | (lagnas ? Native.TS_PANCHANGA_LAGNAS : 0)
                        | (homaAhuti ? Native.TS_PANCHANGA_HOMA_AHUTI : 0)
                        | (sky ? Native.TS_PANCHANGA_SKY : 0),
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

    /** A zone as the record the boundary reads beside a range. */
    private static String zoneJson(ZoneSpec zone) {
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
        return Json.write(written);
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
        return PakshiReads.pakshi(context, from, to, place, utcOffsetSeconds, whose, null);
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
        return PakshiReads.pakshi(context, from, to, place, utcOffsetSeconds, whose, rules);
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
        if (place == null) {
            throw Reads.invalid("place is an Observer", "place");
        }
        Map<String, Object> asked = new java.util.LinkedHashMap<>();
        asked.put("at", at);
        asked.put("latitudeDeg", place.latitudeDeg().value());
        asked.put("longitudeDeg", place.longitudeDeg().value());
        asked.put("altitudeM", place.altitudeM().value());
        asked.put("utcOffsetSeconds", utcOffsetSeconds);
        if (ayana != null) {
            asked.put("ayana", ayana.key());
        }
        String json = Json.write(asked);
        return AlmanacReads.sankalpaAnswer(context.locked((lib, raw) -> Calls.almanacSankalpa(lib, raw, json)));
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
        return ashauch(request, place, utcOffsetSeconds, "table", table);
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
        return ashauch(request, place, utcOffsetSeconds, "table", null);
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
        return ashauch(request, place, utcOffsetSeconds, "tradition", tradition);
    }

    private AshauchAnswer ashauch(com.teispace.teistro.request.AshauchRequest request, Observer place,
            int utcOffsetSeconds, String key, Object table) {
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
        asked.put("utcOffsetSeconds", utcOffsetSeconds);
        if (table != null) {
            asked.put(key, table);
        }
        asked.put("request", request);
        String json = Json.write(asked);
        return AlmanacReads.ashauchAnswer(context.locked((lib, raw) -> Calls.almanacAshauch(lib, raw, json)));
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
        asked.put("utcOffsetSeconds", utcOffsetSeconds);
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
     * tradition, communities and regions and {@code unsourced} the fields no
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
        if (place == null) {
            throw Reads.invalid("place is an Observer", "place");
        }
        Map<String, Object> asked = new java.util.LinkedHashMap<>();
        asked.put("after", after);
        asked.put("latitudeDeg", place.latitudeDeg().value());
        asked.put("longitudeDeg", place.longitudeDeg().value());
        asked.put("altitudeM", place.altitudeM().value());
        asked.put("utcOffsetSeconds", utcOffsetSeconds);
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
        asked.put("utcOffsetSeconds", utcOffsetSeconds);
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
}

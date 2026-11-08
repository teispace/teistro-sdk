package com.teispace.teistro;

import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.Set;

import com.teispace.teistro.blob.Charts;

/**
 * {@code sky.chart()}: charts founded at an instant and a place.
 *
 * <pre>{@code
 * Observer kathmandu = new Observer(new Longitude(85.324), new Latitude(27.7172), new Altitude(1400));
 * Chart chart = sky.chart().found(2_460_000.0, kathmandu, 20_700,
 *         ChartOptions.builder().state(true).vargas(Varga.D9).build());
 * double lagna = chart.lagnaDeg();
 * }</pre>
 */
public final class ChartArea {
    /** The JSON request fields of {@code ts_chart_request}, in its order. */
    private static final List<String> READINGS = List.of(
            "theme_json", "rules_json", "interpret_json", "varsha_json", "gochar_json", "hits_json",
            "sade_sati_json", "kp_json", "dignities_json", "fortitudes_json", "lots_json",
            "considerations_json", "perfection_json", "progressions_json", "western_aspects_json",
            "synastry_json", "parallels_json", "antiscia_json", "midpoints_json", "western_houses_json",
            "harmonic_json", "matching_json", "prashna_json", "remedies_json", "rectification_json",
            "lalkitab_json", "sarvatobhadra_json", "tithi_pravesha_json",
            "dossier_json", "masa_pravesha_json", "western_configurations_json");

    private final Context context;

    ChartArea(Context context) {
        this.context = context;
    }

    /**
     * Founds a chart at an instant and a place. Everything but these is the
     * context's settings, so two charts founded under one context are
     * comparable and the settings hash says why.
     *
     * @param instant the instant, a UTC Julian day
     * @param place where
     * @param utcOffsetSeconds the local clock's offset from UTC, east positive:
     *     the clock the day's date is read in
     * @param options what to compute beside the foundation
     * @return the chart
     */
    public Chart found(double instant, Observer place, int utcOffsetSeconds, ChartOptions options) {
        return foundMany(new double[] {instant}, place, utcOffsetSeconds, options).at(0);
    }

    /**
     * {@link #found(double, Observer, int, ChartOptions)} on a zone's clock: the
     * birth read at the offset the zone had then, which
     * {@link Chart#utcOffsetSeconds()} says ({@code 03-design/time-and-timezone.md}
     * §3.4).
     *
     * @param instant the instant, a UTC Julian day
     * @param place where
     * @param zone the zone, from {@code Civil.ianaZone} and its neighbours
     * @param options what to compute beside the foundation
     * @return the chart
     */
    public Chart found(double instant, Observer place, ZoneSpec zone, ChartOptions options) {
        return foundMany(new double[] {instant}, place, zone, options).at(0);
    }

    /**
     * Founds a chart at each of many instants, at one place, in one crossing:
     * the settings and the solar model are shared across the batch, so a
     * hundred instants cost one setup. A batch of none is empty.
     *
     * @param instants the instants, UTC Julian days
     * @param place where
     * @param utcOffsetSeconds the local clock's offset from UTC, east positive
     * @param options what to compute beside the foundation
     * @return the batch
     */
    public ChartBatch foundMany(double[] instants, Observer place, int utcOffsetSeconds, ChartOptions options) {
        return founded(request(instants, place, utcOffsetSeconds, null, options));
    }

    /**
     * {@link #foundMany(double[], Observer, int, ChartOptions)} on a zone's clock:
     * each birth read at the offset the zone had then, so a batch across a
     * clock change gives each chart its own, {@link Chart#utcOffsetSeconds()},
     * and the provenance states the first birth's
     * ({@code 03-design/time-and-timezone.md} §3.4).
     *
     * @param instants the instants, UTC Julian days
     * @param place where
     * @param zone the zone, from {@code Civil.ianaZone} and its neighbours
     * @param options what to compute beside the foundation
     * @return the batch
     */
    public ChartBatch foundMany(double[] instants, Observer place, ZoneSpec zone, ChartOptions options) {
        if (zone == null) {
            throw Reads.invalid("zone is the clock the charts are founded on", "zone");
        }
        return founded(request(instants, place, 0, AlmanacArea.zoneJson(zone), options));
    }

    private ChartBatch founded(ChartRequest request) {
        return new ChartBatch(Charts.decode(context.locked((lib, raw) -> Calls.chartFound(lib, raw, request))),
                context.dashaNames());
    }

    private ChartRequest request(double[] instants, Observer place, int utcOffsetSeconds, String zoneJson,
            ChartOptions options) {
        for (String field : options.readings().keySet()) {
            if (!READINGS.contains(field)) {
                throw new IllegalArgumentException("`" + field.replaceFirst("_json$", "")
                        + "` is not a reading a chart request carries; it carries "
                        + String.join(", ", READINGS.stream().map(f -> f.replaceFirst("_json$", "")).toList()));
            }
        }
        Map<String, String> json = options.readings();
        List<String> readings = new ArrayList<>();
        for (String field : READINGS) {
            readings.add(json.get(field));
        }
        return new ChartRequest(
                options.kind(),
                options.lilith(),
                instants,
                place.latitudeDeg().value(),
                place.longitudeDeg().value(),
                place.altitudeM().value(),
                utcOffsetSeconds,
                options.sections(),
                options.vargas(),
                drawings(options.drawings()),
                dashas(options.dashas()),
                readings.get(0), readings.get(1), readings.get(2), readings.get(3), readings.get(4),
                readings.get(5), readings.get(6), readings.get(7), readings.get(8), readings.get(9),
                readings.get(10), readings.get(11), readings.get(12), readings.get(13), readings.get(14),
                readings.get(15), readings.get(16), readings.get(17), readings.get(18), readings.get(19),
                readings.get(20), readings.get(21), readings.get(22), readings.get(23), readings.get(24),
                readings.get(25), readings.get(26), readings.get(27), readings.get(28), readings.get(29),
                readings.get(30), zoneJson);
    }

    /**
     * A layout this context can draw in, shipped or registered, as its row:
     * a fresh mapping to copy, give a key of its own and register.
     *
     * @param layout a {@link ChartLayout}, or a layout's key, bare
     *     ({@code NORTH_INDIAN}) or full ({@code chart_layout.ACME_KERALA})
     * @return the row, as {@link Json#read} reads it
     */
    public Map<String, Object> layout(Object layout) {
        String key = layout instanceof ChartLayout member ? member.fullKey() : String.valueOf(layout);
        if (!(Json.read(context.locked((lib, raw) -> Calls.chartLayoutRow(lib, raw, key)))
                instanceof Map<?, ?> row)) {
            throw new TeistroException(Status.INTERNAL, "a layout's row is an object", "", "layout", "", "");
        }
        Map<String, Object> copy = new java.util.LinkedHashMap<>();
        row.forEach((name, value) -> copy.put(String.valueOf(name), value));
        return copy;
    }

    /**
     * <b>Where two charts disagree</b> ({@code 03-design/chart-disagreements.md}): {@code first} founded
     * under this context, {@code second} (the first's birth again when null) under {@code other}, and only
     * the facts that differ, each with both sides: the lagna's sign, each graha's sign, the Moon's
     * nakshatra and pada, each dasha level's running lord at {@code at} (each birth's own when null) and
     * each rule one side holds and the other does not. Pass this context again as {@code other} to compare
     * two births under one setting.
     *
     * <pre>{@code
     * ChartDisagreements found = classical.chart().disagreements(kp,
     *         new ComparedBirth(2_447_000.25, kathmandu, 20_700), null, null);
     * }</pre>
     *
     * @param other the context the second chart is founded under
     * @param first the first birth
     * @param second the second, or null for the first's again
     * @param at the instant the dasha lords are compared at, a UTC Julian day, or null for each birth's own
     * @return the facts that differ, sealed by the first chart's provenance
     */
    public ChartDisagreements disagreements(Context other, ComparedBirth first, ComparedBirth second, Double at) {
        if (other == null) {
            throw Reads.invalid("other is the Context compared with this one", "other");
        }
        if (first == null) {
            throw Reads.invalid("first is the birth compared", "first");
        }
        Map<String, Object> asked = new java.util.LinkedHashMap<>();
        asked.put("first", first.asked("disagreements.first"));
        if (second != null) {
            asked.put("second", second.asked("disagreements.second"));
        }
        if (at != null) {
            asked.put("at", at);
        }
        String json = Json.write(asked);
        return ChartDisagreements.read(context.lockedWith(other,
                (lib, ours, theirs) -> Calls.chartDisagreements(lib, ours, theirs, json)));
    }

    /**
     * As {@link #disagreements(Context, ComparedBirth, ComparedBirth, Double)}, one birth under two
     * settings, its dasha lords at its own birth.
     *
     * @param other the context the second chart is founded under
     * @param birth the birth
     * @return the facts that differ
     */
    public ChartDisagreements disagreements(Context other, ComparedBirth birth) {
        return disagreements(other, birth, null, null);
    }

    /** A dasha system's id: a catalogued one's, or the id the context resolved a registered key to. */
    private int[] dashas(List<Object> systems) {
        int[] ids = new int[systems.size()];
        for (int at = 0; at < ids.length; at += 1) {
            ids[at] = member(systems.get(at), Kind.DASHA_SYSTEM, "dashas[" + at + "]",
                    "a DashaSystem, or the dasha_system.* key of a system this context registered");
        }
        return ids;
    }

    /** Each drawing as the boundary takes it: {@code layout << 16 | varga}. */
    private long[] drawings(List<ChartOptions.Drawing> drawings) {
        long[] bits = new long[drawings.size()];
        for (int at = 0; at < bits.length; at += 1) {
            ChartOptions.Drawing drawing = drawings.get(at);
            String field = "drawings[" + at + "]";
            if (drawing.varga() == null) {
                throw new IllegalArgumentException("`" + field + "` names no varga");
            }
            int layout = member(drawing.layout(), Kind.CHART_LAYOUT, field,
                    "a ChartLayout, or the chart_layout.* key of a layout this context registered");
            bits[at] = ((long) layout << 16) | Values.id(field, drawing.varga());
        }
        return bits;
    }

    private int member(Object given, Kind kind, String field, String wanted) {
        if (given instanceof Catalogued member && member.id() >= 0 && member.kind() == kind) {
            return member.id();
        }
        if (given instanceof String key) {
            long id = context.keys().id(key);
            if ((id >>> 16) == kind.id()) {
                return (int) (id & 0xFFFF);
            }
        }
        throw new IllegalArgumentException("`" + field + "` is " + given + ", not " + wanted);
    }

    /** The JSON request fields a chart request carries, less {@code _json}. */
    static Set<String> readings() {
        return Set.copyOf(READINGS.stream().map(f -> f.replaceFirst("_json$", "")).toList());
    }

    /**
     * One period read for each of the twelve signs.
     *
     * @param request the period, its place and what to read
     * @return the reading, with what sealed it
     */
    public RashifalSealed rashifal(RashifalRequest request) {
        return RashifalReads.rashifal(context, request, null);
    }

    /**
     * One period read for each of the twelve signs, with a baseline engine's
     * scores beside the SDK's.
     *
     * @param request the period, its place and what to read
     * @param baseline the baseline's name
     * @return the reading, with what sealed it
     */
    public RashifalSealed rashifal(RashifalRequest request, String baseline) {
        return RashifalReads.rashifal(context, request, baseline);
    }

    /**
     * Several periods read in one call.
     *
     * @param requests the periods
     * @param baseline the baseline's name, or null for none
     * @return one reading per request, in order, with the batch's provenance
     */
    public RashifalAnswers rashifalMany(List<RashifalRequest> requests, String baseline) {
        return RashifalReads.rashifalMany(context, requests, baseline);
    }

    /**
     * A mundane chart ({@code 03-design/mundane-charts.md}): the moment its
     * kind names (an ingress, the lunation before one, an eclipse, a great
     * conjunction, or an event on record), found by the SDK's own searches,
     * and the chart cast for it at the place, {@link ChartKind#MUNDANE}
     * ({@link ChartKind#EVENT} for an event).
     *
     * @param kind which moment
     * @param place where the chart is cast
     * @param year the year whose moment is asked, or null to give {@code from}
     * @param from the instant the search starts at, a Julian day (UTC), or null
     * @param ingress for a lunation, the ingress it precedes; null for Mesha's
     * @param event for {@link MundaneKind#EVENT}, the event on record; otherwise null
     * @param utcOffsetSeconds the local clock's offset from UTC, east positive
     * @return the moment and the chart cast for it
     */
    public MundaneChart mundane(MundaneKind kind, Observer place, Integer year, Double from, MundaneKind ingress,
            MundaneEvent event, int utcOffsetSeconds) {
        return MapReads.mundane(context, this, kind, place, year, from, ingress, event, utcOffsetSeconds);
    }

    /**
     * A chart's instant recast at other places
     * ({@code 03-design/astrocartography.md}): each place's ascendant,
     * midheaven and twelve cusps, in the context's zodiac.
     *
     * @param instant the chart's instant, a Julian day (UTC)
     * @param places one to 256 places
     * @param houseSystem the house division, or null for the Western default
     * @return each place's angles and cusps
     */
    public Relocation relocated(double instant, List<Observer> places, HouseSystem houseSystem) {
        return MapReads.relocated(context, instant, places, houseSystem);
    }

    /**
     * The astrocartography lines at an instant
     * ({@code 03-design/astrocartography.md}): each body's MC, IC, ASC and
     * DSC lines as {@code [longitudeDeg, latitudeDeg]} points, south to
     * north, every point on its angle to {@code toleranceDeg}.
     *
     * @param instant a Julian day (UTC)
     * @param bodies the bodies, or null for the seven and the outer planets placed
     * @param projection {@code IN_MUNDO} or {@code ZODIACAL}, or null for in mundo
     * @param stepDeg the step of latitude in (0, 10], or null for 1
     * @param latitudeLimitDeg how far north and south in (0, 90), or null for 80
     * @return the lines and what they were drawn under
     */
    public Astrocartography astrocartography(double instant, List<Graha> bodies, String projection, Double stepDeg,
            Double latitudeLimitDeg) {
        return MapReads.astrocartography(context, instant, bodies, projection, stepDeg, latitudeLimitDeg);
    }
}

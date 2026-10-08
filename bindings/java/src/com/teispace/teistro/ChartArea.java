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
            "harmonic_json", "matching_json", "prashna_json", "remedies_json");

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
        ChartRequest request = request(instants, place, utcOffsetSeconds, options);
        return new ChartBatch(Charts.decode(context.locked((lib, raw) -> Calls.chartFound(lib, raw, request))));
    }

    private ChartRequest request(double[] instants, Observer place, int utcOffsetSeconds, ChartOptions options) {
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
                readings.get(20), readings.get(21), readings.get(22), readings.get(23));
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
}

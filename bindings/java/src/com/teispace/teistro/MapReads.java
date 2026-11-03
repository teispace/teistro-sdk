package com.teispace.teistro;

import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

import com.teispace.teistro.record.Provenance;

/**
 * {@code ts_chart_mundane}, {@code ts_chart_relocated} and
 * {@code ts_chart_astrocartography} as {@link ChartArea} hands them out
 * ({@code 03-design/mundane-charts.md}, {@code 03-design/astrocartography.md}).
 */
final class MapReads {
    private MapReads() {
    }

    static MundaneChart mundane(Context context, ChartArea chart, MundaneKind kind, Observer place, Integer year,
            Double from, MundaneKind ingress, MundaneEvent event, int utcOffsetSeconds) {
        if (kind == null || place == null) {
            throw Reads.invalid("a mundane chart needs a kind and a place", kind == null ? "kind" : "place");
        }
        Map<String, Object> asked = new LinkedHashMap<>();
        asked.put("kind", kind.key());
        asked.put("latitudeDeg", place.latitudeDeg().value());
        asked.put("longitudeDeg", place.longitudeDeg().value());
        asked.put("altitudeM", place.altitudeM().value());
        asked.put("utcOffsetSeconds", utcOffsetSeconds);
        if (year != null) {
            asked.put("year", year);
        }
        if (from != null) {
            asked.put("from", from);
        }
        if (ingress != null) {
            asked.put("ingress", ingress.key());
        }
        if (event != null) {
            Map<String, Object> record = new LinkedHashMap<>();
            record.put("name", event.name());
            record.put("instant", event.instant());
            record.put("source", event.source());
            asked.put("event", record);
        }
        String json = Json.write(asked);
        Map<?, ?> envelope = Reads.object(Json.read(context.locked((lib, raw) -> Calls.chartMundane(lib, raw, json))));
        Map<?, ?> value = Reads.object(envelope, "value");
        MundaneKind found = mundaneKind(Reads.field(value, "kind"));
        Observer at = new Observer(new Longitude(Reads.number(value, "longitudeDeg")),
                new Latitude(Reads.number(value, "latitudeDeg")), new Altitude(Reads.number(value, "altitudeM")));
        Chart cast = chart.found(Reads.number(value, "instant"), at, Reads.integer(value, "utcOffsetSeconds"),
                ChartOptions.builder().kind(found == MundaneKind.EVENT ? ChartKind.EVENT : ChartKind.MUNDANE).build());
        Object before = value.get("ingress");
        Object solar = value.get("solarEclipse");
        Object lunar = value.get("lunarEclipse");
        return new MundaneChart(
                found,
                Reads.number(value, "instant"),
                before == null ? null
                        : new MundaneIngress(mundaneKind(Reads.field(Reads.object(before), "kind")),
                                Reads.number(Reads.object(before), "instant")),
                solar == null ? null
                        : Reads.member(SolarEclipseKind.byKey(Reads.string(solar)), "SolarEclipseKind", solar),
                lunar == null ? null
                        : Reads.member(LunarEclipseKind.byKey(Reads.string(lunar)), "LunarEclipseKind", lunar),
                Reads.optionalString(value, "name"),
                Reads.optionalString(value, "source"),
                cast,
                Provenance.of(Reads.field(envelope, "provenance")));
    }

    static Relocation relocated(Context context, double instant, List<Observer> places, HouseSystem houseSystem) {
        if (places == null) {
            throw Reads.invalid("places is a list of one to 256 places", "places");
        }
        List<Object> asking = new ArrayList<>(places.size());
        for (Observer place : places) {
            Map<String, Object> one = new LinkedHashMap<>();
            one.put("latitudeDeg", place.latitudeDeg().value());
            one.put("longitudeDeg", place.longitudeDeg().value());
            one.put("altitudeM", place.altitudeM().value());
            asking.add(one);
        }
        Map<String, Object> asked = new LinkedHashMap<>();
        asked.put("instant", instant);
        asked.put("places", asking);
        if (houseSystem != null) {
            asked.put("houseSystem", houseSystem.key());
        }
        String json = Json.write(asked);
        Map<?, ?> envelope = Reads.object(Json.read(context.locked((lib, raw) -> Calls.chartRelocated(lib, raw, json))));
        Map<?, ?> value = Reads.object(envelope, "value");
        return new Relocation(Reads.number(value, "instant"), Reads.number(value, "obliquityDeg"),
                Reads.each(value, "places", raw -> {
                    Map<?, ?> one = Reads.object(raw);
                    Object system = Reads.field(one, "houseSystem");
                    return new RelocatedChart(Reads.number(one, "latitudeDeg"), Reads.number(one, "longitudeDeg"),
                            Reads.number(one, "altitudeM"), Reads.number(one, "ascendantDeg"),
                            Reads.number(one, "midheavenDeg"), Reads.each(one, "cuspsDeg", Reads::number),
                            Reads.member(HouseSystem.byKey(Reads.string(system)), "HouseSystem", system));
                }),
                Provenance.of(Reads.field(envelope, "provenance")));
    }

    static Astrocartography astrocartography(Context context, double instant, List<Graha> bodies, String projection,
            Double stepDeg, Double latitudeLimitDeg) {
        Map<String, Object> asked = new LinkedHashMap<>();
        asked.put("instant", instant);
        if (bodies != null) {
            asked.put("bodies", bodies.stream().map(Graha::key).toList());
        }
        if (projection != null) {
            asked.put("projection", projection);
        }
        if (stepDeg != null) {
            asked.put("stepDeg", stepDeg);
        }
        if (latitudeLimitDeg != null) {
            asked.put("latitudeLimitDeg", latitudeLimitDeg);
        }
        String json = Json.write(asked);
        Map<?, ?> envelope = Reads.object(
                Json.read(context.locked((lib, raw) -> Calls.chartAstrocartography(lib, raw, json))));
        Map<?, ?> value = Reads.object(envelope, "value");
        return new Astrocartography(Reads.number(value, "instant"), Reads.string(value, "projection"),
                Reads.number(value, "stepDeg"), Reads.number(value, "latitudeLimitDeg"),
                Reads.number(value, "toleranceDeg"), Reads.number(value, "greenwichRamcDeg"),
                Reads.number(value, "obliquityDeg"),
                Reads.each(value, "lines", raw -> {
                    Map<?, ?> line = Reads.object(raw);
                    Object body = Reads.field(line, "body");
                    return new AstroLine(Reads.member(Graha.byKey(Reads.string(body)), "Graha", body),
                            Reads.string(line, "angle"),
                            Reads.each(line, "points", point -> Reads.array(point).stream().map(Reads::number).toList()),
                            Reads.optionalNumber(line, "polarLimitDeg"));
                }),
                Provenance.of(Reads.field(envelope, "provenance")));
    }

    private static MundaneKind mundaneKind(Object key) {
        return Reads.member(MundaneKind.byKey(Reads.string(key)), "MundaneKind", key);
    }
}

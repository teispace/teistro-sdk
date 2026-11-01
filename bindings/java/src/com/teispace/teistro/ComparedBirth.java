package com.teispace.teistro;

import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

/**
 * One birth of a chart comparison ({@link ChartArea#disagreements}): the instant, the place and the
 * clock, and of the options the chart kind, the dasha systems and the rules, which are what a
 * comparison reads. A side naming no dasha system is read under Vimshottari.
 *
 * <pre>{@code
 * ComparedBirth birth = new ComparedBirth(2_447_000.25, kathmandu, 20_700,
 *         ChartOptions.builder().dashas(DashaSystem.VIMSHOTTARI).build());
 * }</pre>
 *
 * @param instant the instant, a UTC Julian day
 * @param place where
 * @param utcOffsetSeconds the local clock's offset from UTC, east positive
 * @param options the kind, dashas and rules compared; null for none
 */
public record ComparedBirth(double instant, Observer place, int utcOffsetSeconds, ChartOptions options) {
    /** The birth, its options never null. */
    public ComparedBirth {
        if (place == null) {
            throw Reads.invalid("place is an Observer", "place");
        }
        options = options == null ? ChartOptions.none() : options;
    }

    /**
     * A birth with nothing compared beyond the foundation and Vimshottari.
     *
     * @param instant the instant, a UTC Julian day
     * @param place where
     * @param utcOffsetSeconds the local clock's offset from UTC, east positive
     */
    public ComparedBirth(double instant, Observer place, int utcOffsetSeconds) {
        this(instant, place, utcOffsetSeconds, null);
    }

    /** The birth as the boundary reads it; a reading a comparison does not read is refused by name. */
    Map<String, Object> asked(String side) {
        for (String field : options.readings().keySet()) {
            if (!field.equals("rules_json")) {
                throw Reads.invalid("a comparison reads the kind, the dashas and the rules of a birth, not `"
                        + field.replaceFirst("_json$", "") + "`", side + "." + field.replaceFirst("_json$", ""));
            }
        }
        Map<String, Object> asked = new LinkedHashMap<>();
        asked.put("instant", instant);
        asked.put("latitudeDeg", place.latitudeDeg().value());
        asked.put("longitudeDeg", place.longitudeDeg().value());
        asked.put("altitudeM", place.altitudeM().value());
        asked.put("utcOffsetSeconds", utcOffsetSeconds);
        asked.put("kind", options.kind().fullKey());
        List<Object> systems = options.dashas();
        if (!systems.isEmpty()) {
            asked.put("dashas", systems.stream()
                    .map(system -> system instanceof DashaSystem member ? member.fullKey() : String.valueOf(system))
                    .toList());
        }
        String rules = options.readings().get("rules_json");
        if (rules != null) {
            asked.put("rules", Json.read(rules));
        }
        return asked;
    }
}

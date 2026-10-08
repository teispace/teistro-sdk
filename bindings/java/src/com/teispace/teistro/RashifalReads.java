package com.teispace.teistro;

import java.util.ArrayList;
import java.util.Collections;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

/**
 * The rashifal: one period of civil days at a place read for each of the
 * twelve signs ({@code 03-design/rashifal.md}), its request written as
 * {@code ts_rashifal} reads it and its answer read with its keys made members.
 */
final class RashifalReads {
    private RashifalReads() {}

    /**
     * One period of civil days at a place read for each of the twelve signs:
     * the sky at sunrise on the middle day, or at {@code snapshot}'s clock
     * time; each sign's gochar from Phaladeepika ch. 26, Saturn's standing
     * from it, and every ingress and station of the period counted from it.
     * With {@code baseline} ({@code "DAILY"}, {@code "WEEKLY"},
     * {@code "MONTHLY"} or {@code "YEARLY"}), each sign's score as the
     * baseline engine reckons it, {@code BASELINE} and unsourced (C361).
     *
     * @param context the context to read it under
     * @param request the period
     * @param baseline the baseline period to score; may be null for none
     * @return the answer
     */
    static RashifalAnswer rashifal(Context context, RashifalRequest request, String baseline) {
        return rashifalMany(context, List.of(request), baseline).get(0);
    }

    /**
     * Many periods, each read as {@link #rashifal} reads it alone, under one founder.
     *
     * @param context the context to read them under
     * @param requests the periods
     * @param baseline the baseline period to score; may be null for none
     * @return one answer a period, in their order
     */
    static List<RashifalAnswer> rashifalMany(Context context, List<RashifalRequest> requests, String baseline) {
        if (requests == null) {
            throw Reads.invalid("requests is a list of rashifal periods, such as "
                    + "List.of(RashifalRequest.of(date, place, 20_700))", "requests");
        }
        List<Object> periods = new ArrayList<>(requests.size());
        for (RashifalRequest request : requests) {
            periods.add(asked(request));
        }
        Map<String, Object> asked = new LinkedHashMap<>();
        asked.put("periods", periods);
        if (baseline != null) {
            asked.put("baseline", baseline);
        }
        String json = Json.write(asked);
        String answered = context.locked((lib, raw) -> Calls.rashifal(lib, raw, json));
        List<RashifalAnswer> out = new ArrayList<>();
        for (Object one : Reads.array(Json.read(answered))) {
            out.add(answer(Reads.object(one)));
        }
        return Collections.unmodifiableList(out);
    }

    private static Map<String, Integer> parts(CalendarDate date) {
        Map<String, Integer> out = new LinkedHashMap<>();
        out.put("year", date.year());
        out.put("month", date.month());
        out.put("day", date.day());
        return out;
    }

    /**
     * A rashifal period as {@code ts_rashifal} reads it: the days by their
     * parts, the place by its, a graha written as its full key.
     */
    private static Map<String, Object> asked(RashifalRequest request) {
        if (request == null) {
            throw Reads.invalid("a rashifal period is a RashifalRequest, such as RashifalRequest.of(date, place, 20_700)",
                    "rashifal");
        }
        if (request.first() == null) {
            throw Reads.invalid("first is a CalendarDate", "first");
        }
        if (request.place() == null) {
            throw Reads.invalid("place is an Observer", "place");
        }
        Map<String, Object> asked = new LinkedHashMap<>();
        asked.put("utcOffsetSeconds", request.utcOffsetSeconds());
        if (request.snapshot() != null) {
            asked.put("snapshot", request.snapshot());
        }
        if (request.events() != null) {
            asked.put("events", request.events());
        }
        if (request.spells() != null) {
            asked.put("spells", request.spells());
        }
        asked.put("first", parts(request.first()));
        asked.put("calendar", request.first().calendar().key());
        asked.put("latitudeDeg", request.place().latitudeDeg().value());
        asked.put("longitudeDeg", request.place().longitudeDeg().value());
        asked.put("altitudeM", request.place().altitudeM().value());
        if (request.last() != null) {
            asked.put("last", parts(request.last()));
        }
        return asked;
    }

    private static <E extends Member> E member(Map<?, ?> object, String key, String kind,
            java.util.function.Function<String, java.util.Optional<E>> byKey) {
        String text = Reads.string(object, key);
        return Reads.member(byKey.apply(text), kind, text);
    }

    private static Graha graha(Object key) {
        return Reads.member(Graha.byKey(Reads.string(key)), "Graha", key);
    }

    private static Transit transit(Object raw) {
        Map<?, ?> one = Reads.object(raw);
        return new Transit(member(one, "sign", "Rashi", Rashi::byKey), Reads.number(one, "degrees"));
    }

    private static Hit hit(Object raw) {
        Map<?, ?> one = Reads.object(raw);
        Map<?, ?> event = Reads.object(one, "event");
        double instant = Reads.number(one, "instant");
        Graha graha = graha(Reads.field(one, "graha"));
        String kind = Reads.string(event, "kind");
        return switch (kind) {
            case "SIGN_INGRESS" -> new Hit(instant, graha,
                    new SignIngress(member(event, "into", "Rashi", Rashi::byKey),
                            member(event, "motion", "Motion", Motion::byKey)));
            case "STATION" -> new Hit(instant, graha, new Station(member(event, "turns", "Motion", Motion::byKey)));
            default -> throw Reads.internal("a rashifal reported an event this build does not know: " + kind);
        };
    }

    private static GocharReading gochar(Object raw, double instant) {
        Map<?, ?> one = Reads.object(raw);
        Map<?, ?> rules = Reads.object(one, "rules");
        Map<?, ?> reference = Reads.object(one, "reference");
        return new GocharReading(
                instant,
                new GocharReference(member(reference, "from", "GocharFrom", GocharFrom::byKey),
                        member(reference, "sign", "Rashi", Rashi::byKey)),
                new GocharRules(member(rules, "nodeVedha", "NodeVedha", NodeVedha::byKey),
                        member(rules, "nodeObstruction", "NodeObstruction", NodeObstruction::byKey),
                        member(rules, "ashtakavargaGoodFrom", "AshtakavargaGoodFrom", AshtakavargaGoodFrom::byKey)),
                Reads.each(one, "grahas", g -> {
                    Map<?, ?> o = Reads.object(g);
                    return new GrahaGochar(
                            graha(Reads.field(o, "graha")),
                            transit(Reads.field(o, "transit")),
                            Reads.integer(o, "house"),
                            Reads.flag(o, "goodHouse"),
                            Reads.optionalInteger(o, "vedhaHouse"),
                            Reads.each(o, "obstructedBy", RashifalReads::graha),
                            member(o, "verdict", "GocharVerdict", GocharVerdict::byKey),
                            member(o, "fruition", "Fruition", Fruition::byKey),
                            Reads.flag(o, "fruitfulNow"));
                }),
                null);
    }

    private static RashiReading reading(Object raw, double instant) {
        Map<?, ?> one = Reads.object(raw);
        Map<?, ?> saturn = Reads.object(one, "saturn");
        return new RashiReading(
                member(one, "rashi", "Rashi", Rashi::byKey),
                gochar(Reads.field(one, "gochar"), instant),
                new SaturnStanding(Reads.integer(saturn, "house"), Reads.optionalString(saturn, "sadeSati"),
                        Reads.flag(saturn, "spell")),
                Reads.each(one, "events", e -> {
                    Map<?, ?> o = Reads.object(e);
                    Map<?, ?> event = Reads.object(o, "event");
                    return new RashifalEvent(hit(Reads.field(event, "hit")), member(event, "sign", "Rashi", Rashi::byKey),
                            Reads.integer(o, "house"), Reads.flag(o, "goodHouse"));
                }));
    }

    private static BaselineScore score(Object raw) {
        Map<?, ?> one = Reads.object(raw);
        Map<?, ?> lucky = Reads.object(one, "lucky");
        return new BaselineScore(
                Reads.integer(one, "overall"),
                Reads.each(one, "areas", pair -> {
                    List<?> both = Reads.array(pair);
                    if (both.size() != 2) {
                        throw Reads.internal("a baseline area is a name and a score, and the library wrote " + both);
                    }
                    return Map.entry(Reads.string(both.get(0)), Reads.integer(both.get(1)));
                }),
                Reads.each(one, "keyInfluences", k -> {
                    Map<?, ?> o = Reads.object(k);
                    return new KeyInfluence(graha(Reads.field(o, "graha")), Reads.integer(o, "house"),
                            member(o, "verdict", "GocharVerdict", GocharVerdict::byKey));
                }),
                new LuckyElements(Reads.string(lucky, "colour"), Reads.integer(lucky, "number"),
                        member(lucky, "day", "Vara", Vara::byKey),
                        member(lucky, "direction", "Direction", Direction::byKey)));
    }

    /** One period's answer from {@code ts_rashifal}'s JSON, its keys made members. */
    private static RashifalAnswer answer(Map<?, ?> raw) {
        Map<?, ?> period = Reads.object(raw, "period");
        Map<?, ?> limbs = Reads.object(period, "panchanga");
        double instant = Reads.number(period, "instant");
        Object baseline = raw.get("baseline");
        return new RashifalAnswer(
                new RashifalPeriod(
                        Reads.serdeDate(Reads.field(period, "first")),
                        Reads.serdeDate(Reads.field(period, "last")),
                        Reads.serdeDate(Reads.field(period, "reference")),
                        instant,
                        Reads.each(period, "transits", RashifalReads::transit),
                        Reads.each(period, "retrograde", flag -> {
                            if (flag instanceof Boolean backwards) {
                                return backwards;
                            }
                            throw Reads.internal("a retrograde flag is true or false, and the library wrote " + flag);
                        }),
                        new RashifalPanchanga(member(limbs, "tithi", "Tithi", Tithi::byKey),
                                member(limbs, "yoga", "Yoga", Yoga::byKey), Reads.integer(limbs, "muhurtaYogas")),
                        Reads.each(period, "readings", one -> reading(one, instant))),
                baseline == null ? null : Reads.array(baseline).stream().map(RashifalReads::score).toList());
    }
}

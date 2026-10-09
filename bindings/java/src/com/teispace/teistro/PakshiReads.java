package com.teispace.teistro;

import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

import com.teispace.teistro.record.Provenance;

/**
 * Pancha Pakshi's days, the request written as {@code ts_pakshi} reads it and the answer read with its
 * catalogue keys made members.
 */
final class PakshiReads {
    private PakshiReads() {}

    private static Map<String, Integer> parts(CalendarDate date) {
        Map<String, Integer> out = new LinkedHashMap<>();
        out.put("year", date.year());
        out.put("month", date.month());
        out.put("day", date.day());
        return out;
    }

    /** The native as {@code ts_pakshi} reads it. */
    private static Map<String, Object> whose(PakshiNative whose) {
        Map<String, Object> out = new LinkedHashMap<>();
        switch (whose) {
            case PakshiNative.Bird one -> out.put("bird", one.bird());
            case PakshiNative.Star one -> {
                out.put("nakshatra", one.nakshatra().fullKey());
                out.put("paksha", one.paksha().fullKey());
                out.put("rule", one.rule());
            }
        }
        return out;
    }

    static PakshiDays pakshi(Context context, CalendarDate from, CalendarDate to, Observer place,
            int utcOffsetSeconds, PakshiNative whose, Map<String, ?> rules) {
        if (from == null) {
            throw Reads.invalid("from is a CalendarDate", "from");
        }
        if (place == null) {
            throw Reads.invalid("place is an Observer", "place");
        }
        if (whose == null) {
            throw Reads.invalid("native is a PakshiNative, such as PakshiNative.bird(\"OWL\")", "native");
        }
        Map<String, Object> asked = new LinkedHashMap<>();
        asked.put("calendar", from.calendar().key());
        asked.put("first", parts(from));
        if (to != null) {
            asked.put("last", parts(to));
        }
        asked.put("latitudeDeg", place.latitudeDeg().value());
        asked.put("longitudeDeg", place.longitudeDeg().value());
        asked.put("altitudeM", place.altitudeM().value());
        asked.put("utcOffsetSeconds", utcOffsetSeconds);
        asked.put("native", whose(whose));
        if (rules != null) {
            asked.put("rules", rules);
        }
        String json = Json.write(asked);
        String answered = context.locked((lib, raw) -> Calls.pakshi(lib, raw, json));
        Map<?, ?> envelope = Reads.object(Json.read(answered));
        return new PakshiDays(Reads.each(envelope, "value", one -> day(Reads.object(one))),
                Provenance.of(Reads.field(envelope, "provenance")));
    }

    private static PakshiDay day(Map<?, ?> raw) {
        CalendarDate date = Reads.serdeDate(Reads.field(raw, "date"));
        Object reading = Reads.field(raw, "reading");
        if (reading == null) {
            return new PakshiDay(date, null);
        }
        Map<?, ?> read = Reads.object(reading);
        Map<?, ?> bounds = Reads.object(read, "day");
        return new PakshiDay(date, new PakshiDay.Reading(
                new PakshiDay.Bounds(
                        Reads.number(bounds, "sunrise"),
                        Reads.number(bounds, "sunset"),
                        Reads.number(bounds, "nextSunrise"),
                        Reads.member(Vara.byKey(Reads.string(bounds, "vara")), "Vara", bounds.get("vara")),
                        Reads.member(Paksha.byKey(Reads.string(bounds, "paksha")), "Paksha", bounds.get("paksha"))),
                Reads.string(read, "bird"),
                Reads.string(read, "deathBird"),
                Reads.flag(read, "deadToday"),
                Reads.each(read, "eaters", Reads::string),
                Reads.each(read, "yamas", PakshiReads::yama)));
    }

    private static PakshiDay.Yama yama(Object raw) {
        Map<?, ?> one = Reads.object(raw);
        return new PakshiDay.Yama(
                Reads.string(one, "half"),
                Reads.integer(one, "yama"),
                Reads.interval(Reads.field(one, "span")),
                Reads.string(one, "activity"),
                Reads.string(one, "quality"),
                Reads.each(one, "subs", PakshiReads::sub));
    }

    private static PakshiDay.Sub sub(Object raw) {
        Map<?, ?> one = Reads.object(raw);
        return new PakshiDay.Sub(
                Reads.string(one, "activity"),
                Reads.string(one, "owner"),
                Reads.integer(one, "share"),
                Reads.string(one, "ownerIs"),
                Reads.interval(Reads.field(one, "span")));
    }
}

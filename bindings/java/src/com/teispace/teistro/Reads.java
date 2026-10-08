package com.teispace.teistro;

import java.util.ArrayList;
import java.util.Collections;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.function.IntFunction;
import java.util.function.IntToLongFunction;

/**
 * What the readers of a blob's sections and of a JSON answer share: a
 * per-chart table from a count column, a bit set read as members, a
 * catalogue member read from its key, and JSON values read by what they
 * must be.
 */
final class Reads {
    private Reads() {}

    /** The seven grahas, the Sun to Saturn, as a bit set over them reads. */
    static final List<Graha> SEVEN = List.of(
            Graha.SUN, Graha.MOON, Graha.MARS, Graha.MERCURY, Graha.JUPITER, Graha.VENUS, Graha.SATURN);

    /** A refusal for a blob or an answer the library should never have written. */
    static TeistroException internal(String message) {
        return new TeistroException(Status.INTERNAL, message, "", "", "", "");
    }

    /** A refusal of a caller's argument, naming it. */
    static TeistroException invalid(String message, String field) {
        return new TeistroException(Status.INVALID_ARG, message, "", field, "", "");
    }

    /** How many charts a decoded blob holds, as its {@code cast} section counts them. */
    static int charts(com.teispace.teistro.blob.Charts decoded) {
        return decoded.cast().length();
    }

    /** The rows {@code from} to {@code to}, each read, as an unmodifiable list. */
    static <T> List<T> rows(int from, int to, IntFunction<T> read) {
        List<T> out = new ArrayList<>(Math.max(0, to - from));
        for (int at = from; at < to; at += 1) {
            out.add(read.apply(at));
        }
        return Collections.unmodifiableList(out);
    }

    /** Where each row's run begins: {@code starts[k]} to {@code starts[k + 1]} are row {@code k}'s. */
    static int[] starts(int rows, IntToLongFunction count) {
        int[] starts = new int[rows + 1];
        for (int k = 0; k < rows; k += 1) {
            starts[k + 1] = Math.toIntExact(starts[k] + count.applyAsLong(k));
        }
        return starts;
    }

    /** The sum of a count column. */
    static long sum(int rows, IntToLongFunction count) {
        long total = 0;
        for (int k = 0; k < rows; k += 1) {
            total += count.applyAsLong(k);
        }
        return total;
    }

    /**
     * A per-chart table from a count column and the rows it is ragged by,
     * for {@code charts} charts: each chart's rows, or none at all when the
     * column is empty because nothing was asked.
     */
    static <T> List<List<T>> ragged(int charts, int counts, IntToLongFunction count, int rows, String names,
            IntFunction<T> read) {
        if (counts == 0) {
            return List.of();
        }
        long total = sum(counts, count);
        if (counts != charts || rows != total) {
            throw internal(names + ": " + counts + " counts and " + rows + " rows for " + charts + " charts");
        }
        List<List<T>> tables = new ArrayList<>(counts);
        int start = 0;
        for (int k = 0; k < counts; k += 1) {
            int n = Math.toIntExact(count.applyAsLong(k));
            tables.add(rows(start, start + n, read));
            start += n;
        }
        return Collections.unmodifiableList(tables);
    }

    /** One chart's entry of a per-chart list, or empty when the list holds none for it. */
    static <T> Optional<T> at(List<T> parsed, int index) {
        return index < parsed.size() ? Optional.of(parsed.get(index)) : Optional.empty();
    }

    /**
     * The members of a bit set over a small closed list, in the list's
     * order: bit {@code n} is the member with id {@code n}. A member this
     * build does not know is never one.
     */
    static <E extends Member> List<E> members(long bits, List<E> of) {
        List<E> out = new ArrayList<>();
        for (E member : of) {
            int id = member.id();
            if (id >= 0 && id < 64 && (bits >>> id & 1L) != 0) {
                out.add(member);
            }
        }
        return Collections.unmodifiableList(out);
    }

    /** A catalogue member by its key, or a refusal naming both. */
    static <E extends Member> E member(Optional<E> found, String kind, Object key) {
        return found.orElseThrow(() -> internal("the library drew a " + kind + " this build does not know: " + key));
    }

    /** A JSON object, or a refusal. */
    static Map<?, ?> object(Object value) {
        if (value instanceof Map<?, ?> map) {
            return map;
        }
        throw internal("the library wrote " + value + " where an object belongs");
    }

    /** A JSON array, or a refusal. */
    static List<?> array(Object value) {
        if (value instanceof List<?> list) {
            return list;
        }
        throw internal("the library wrote " + value + " where an array belongs");
    }

    /** A field the answer always writes, or a refusal naming it. */
    static Object field(Map<?, ?> object, String key) {
        if (!object.containsKey(key)) {
            throw internal("the library's answer has no `" + key + "`");
        }
        return object.get(key);
    }

    /** A field's object. */
    static Map<?, ?> object(Map<?, ?> object, String key) {
        return object(field(object, key));
    }

    /** A field's array. */
    static List<?> array(Map<?, ?> object, String key) {
        return array(field(object, key));
    }

    /** A field's string. */
    static String string(Map<?, ?> object, String key) {
        return string(field(object, key));
    }

    /** A string, or a refusal. */
    static String string(Object value) {
        if (value instanceof String text) {
            return text;
        }
        throw internal("the library wrote " + value + " where a string belongs");
    }

    /** A field's string, or null when it is null or absent. */
    static String optionalString(Map<?, ?> object, String key) {
        Object value = object.get(key);
        return value == null ? null : string(value);
    }

    /** A field's number as a double. */
    static double number(Map<?, ?> object, String key) {
        return number(field(object, key));
    }

    /** A number as a double. */
    static double number(Object value) {
        if (value instanceof Number number) {
            return number.doubleValue();
        }
        throw internal("the library wrote " + value + " where a number belongs");
    }

    /** A field's number as a double, or null when it is null or absent. */
    static Double optionalNumber(Map<?, ?> object, String key) {
        Object value = object.get(key);
        return value == null ? null : number(value);
    }

    /** A field's whole number. */
    static int integer(Map<?, ?> object, String key) {
        return integer(field(object, key));
    }

    /** A whole number. */
    static int integer(Object value) {
        if (value instanceof Long whole) {
            return Math.toIntExact(whole);
        }
        if (value instanceof Number number && number.doubleValue() == Math.rint(number.doubleValue())) {
            return Math.toIntExact((long) number.doubleValue());
        }
        throw internal("the library wrote " + value + " where a whole number belongs");
    }

    /** A field's whole number, or null when it is null or absent. */
    static Integer optionalInteger(Map<?, ?> object, String key) {
        Object value = object.get(key);
        return value == null ? null : integer(value);
    }

    /** A field's truth value. */
    static boolean flag(Map<?, ?> object, String key) {
        if (field(object, key) instanceof Boolean flag) {
            return flag;
        }
        throw internal("the library wrote " + object.get(key) + " where `" + key + "` is true or false");
    }

    /** Each element of a field's array, read. */
    static <T> List<T> each(Map<?, ?> object, String key, java.util.function.Function<Object, T> read) {
        List<?> raw = array(object, key);
        List<T> out = new ArrayList<>(raw.size());
        for (Object one : raw) {
            out.add(read.apply(one));
        }
        return Collections.unmodifiableList(out);
    }

    /** An interval as a JSON section writes one: {@code from} and {@code to}. */
    static Interval interval(Object raw) {
        Map<?, ?> object = object(raw);
        return new Interval(number(object, "from"), number(object, "to"));
    }

    /**
     * A date as the Rust types serialise it inside a JSON section (a muhurta
     * answer's closed day, a festival's day), in this binding's own shape:
     * the era beside its year, the resolution by name with a divergent
     * one's computed day.
     */
    static CalendarDate serdeDate(Object raw) {
        Map<?, ?> date = object(raw);
        Map<?, ?> resolution = object(date, "resolution");
        boolean divergent = "DIVERGENT".equals(field(resolution, "kind"));
        Object eraField = date.get("era");
        Map<?, ?> era = eraField == null ? null : object(eraField);
        String calendar = string(date, "calendar");
        String kind = string(resolution, "kind");
        Map<?, ?> computed = divergent ? object(resolution, "computed") : null;
        return new CalendarDate(
                member(Calendar.byKey(calendar), "Calendar", calendar),
                era == null ? null : member(Era.byKey(string(era, "era")), "Era", era.get("era")),
                integer(date, "year"),
                era == null ? 0 : integer(era, "year"),
                integer(date, "month"),
                integer(date, "day"),
                member(Resolution.byKey(kind), "Resolution", kind),
                computed == null ? 0 : integer(computed, "month"),
                computed == null ? 0 : integer(computed, "day"));
    }

    /**
     * A request value as JSON writes it: a muhurta clause as its tag and
     * fields, a record an answer gave as a request writes it, a map's and a
     * list's values each in turn, and anything else as {@link Json#write}
     * writes it.
     */
    static Object written(Object value) {
        return switch (value) {
            case null -> null;
            case MuhurtaClauseKind clause -> clause.request();
            case MuhurtaBar bar -> bar.request();
            case MuhurtaPada pada -> pada.request();
            case TaraReading reading -> reading.request();
            case AccidentalRules rules -> rules.request();
            case AccidentalScores scores -> scores.request();
            case AlmutenRules rules -> rules.request();
            case LotRules rules -> rules.request();
            case ConsiderationRules rules -> rules.request();
            case PerfectionRules rules -> rules.request();
            case Map<?, ?> map -> {
                Map<Object, Object> out = new LinkedHashMap<>();
                for (Map.Entry<?, ?> entry : map.entrySet()) {
                    out.put(entry.getKey(), written(entry.getValue()));
                }
                yield out;
            }
            case List<?> list -> {
                List<Object> out = new ArrayList<>(list.size());
                for (Object one : list) {
                    out.add(written(one));
                }
                yield out;
            }
            default -> value;
        };
    }

    /**
     * A request option that crosses as a JSON record, written down; null
     * where it was not given. Anything but a map is refused here, named and
     * shown, rather than across the boundary.
     */
    static String recordJson(Object value, String field, String example) {
        if (value == null) {
            return null;
        }
        if (value instanceof String json) {
            return json;
        }
        if (value instanceof Map<?, ?>) {
            return Json.write(written(value));
        }
        throw invalid(field + " is a request record, such as " + example, field);
    }
}

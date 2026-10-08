package com.teispace.teistro;

import java.util.ArrayList;
import java.util.Collections;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.function.Function;

/**
 * Reading the values {@link Json#read} answers as the records a JSON section
 * holds: a field of an object, a list, a number, a catalogue member by its
 * bare key. A value of the wrong shape is the library's fault, never the
 * caller's, so each refusal is an {@link Status#INTERNAL} one naming what
 * was found.
 */
final class JsonRead {
    private JsonRead() {
    }

    /** A refusal for a section the library wrote in a shape this layer cannot read. */
    static TeistroException internal(String message) {
        return new TeistroException(Status.INTERNAL, message, "", "", "", "");
    }

    /** The field {@code key} of an object, which must be present (it may be JSON null). */
    static Object at(Object object, String key) {
        if (!(object instanceof Map<?, ?> map)) {
            throw internal("expected a JSON object holding `" + key + "`, found " + object);
        }
        if (!map.containsKey(key)) {
            throw internal("the JSON object has no `" + key + "`");
        }
        return map.get(key);
    }

    /** A JSON array. */
    static List<?> list(Object value) {
        if (!(value instanceof List<?> list)) {
            throw internal("expected a JSON array, found " + value);
        }
        return list;
    }

    /** A JSON object, copied into a string-keyed map in its own order, unmodifiable. */
    static Map<String, Object> object(Object value) {
        if (!(value instanceof Map<?, ?> map)) {
            throw internal("expected a JSON object, found " + value);
        }
        Map<String, Object> copy = new LinkedHashMap<>();
        for (Map.Entry<?, ?> entry : map.entrySet()) {
            copy.put(String.valueOf(entry.getKey()), entry.getValue());
        }
        return Collections.unmodifiableMap(copy);
    }

    /** A JSON string. */
    static String text(Object value) {
        if (!(value instanceof String text)) {
            throw internal("expected a JSON string, found " + value);
        }
        return text;
    }

    /** A JSON string, or null for JSON null. */
    static String textOrNull(Object value) {
        return value == null ? null : text(value);
    }

    /** A JSON array of strings. */
    static List<String> texts(Object value) {
        return list(value).stream().map(JsonRead::text).toList();
    }

    /** A JSON boolean. */
    static boolean bool(Object value) {
        if (!(value instanceof Boolean flag)) {
            throw internal("expected a JSON boolean, found " + value);
        }
        return flag;
    }

    /** A JSON boolean, or null for JSON null. */
    static Boolean boolOrNull(Object value) {
        return value == null ? null : bool(value);
    }

    /** A JSON integer that fits a long. */
    static long whole(Object value) {
        if (!(value instanceof Long number)) {
            throw internal("expected a JSON integer, found " + value);
        }
        return number;
    }

    /** A JSON integer that fits an int. */
    static int integer(Object value) {
        long number = whole(value);
        if (number < Integer.MIN_VALUE || number > Integer.MAX_VALUE) {
            throw internal("expected a JSON integer that fits 32 bits, found " + number);
        }
        return (int) number;
    }

    /** A JSON integer that fits an int, or null for JSON null. */
    static Integer integerOrNull(Object value) {
        return value == null ? null : integer(value);
    }

    /** A JSON array of integers that fit an int. */
    static List<Integer> integers(Object value) {
        return list(value).stream().map(JsonRead::integer).toList();
    }

    /** A JSON number, integer or not. */
    static double decimal(Object value) {
        if (!(value instanceof Number number)) {
            throw internal("expected a JSON number, found " + value);
        }
        return number.doubleValue();
    }

    /**
     * A catalogue member by its bare key, or a refusal naming both, since a
     * key this build does not know is the library drawing past the binding.
     */
    static <E> E member(Function<String, Optional<E>> byKey, String kind, Object key) {
        String bare = text(key);
        return byKey.apply(bare).orElseThrow(
                () -> internal("the library drew a " + kind + " this build does not know: " + bare));
    }

    /** A catalogue member by its bare key, or null for JSON null. */
    static <E> E memberOrNull(Function<String, Optional<E>> byKey, String kind, Object key) {
        return key == null ? null : member(byKey, kind, key);
    }

    /** A JSON array of bare keys, each a catalogue member. */
    static <E> List<E> members(Function<String, Optional<E>> byKey, String kind, Object keys) {
        List<E> found = new ArrayList<>();
        for (Object key : list(keys)) {
            found.add(member(byKey, kind, key));
        }
        return List.copyOf(found);
    }
}

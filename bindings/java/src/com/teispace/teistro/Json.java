package com.teispace.teistro;

import java.util.ArrayList;
import java.util.Collections;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

/**
 * JSON as the library writes it, read into plain Java values: an object is
 * a {@code Map<String, Object>} in the order it was written, an array a
 * {@code List<Object>}, a string a {@code String}, a number a {@code Long}
 * when it is an integer that fits and a {@code Double} otherwise, and
 * {@code true}, {@code false} and {@code null} themselves. The JDK has no
 * JSON reader, and the binding takes no dependency for one.
 *
 * <p>Strict, as the library's own reader is: text after the value, a key
 * given twice, and anything RFC 8259 does not allow are refused.
 */
public final class Json {
    private final String text;
    private int at;

    private Json(String text) {
        this.text = text;
    }

    /**
     * Reads one JSON value.
     *
     * @param text the JSON text
     * @return the value
     * @throws IllegalArgumentException naming where the text stopped being JSON
     */
    public static Object read(String text) {
        Json reader = new Json(text);
        reader.space();
        Object value = reader.value();
        reader.space();
        if (reader.at != text.length()) {
            throw reader.refused("text after the value");
        }
        return value;
    }

    /**
     * Reads one JSON object.
     *
     * @param text the JSON text
     * @return the object's members, in the order they were written
     * @throws IllegalArgumentException for text that is not one object
     */
    @SuppressWarnings("unchecked")
    public static Map<String, Object> object(String text) {
        if (read(text) instanceof Map<?, ?> map) {
            return (Map<String, Object>) map;
        }
        throw new IllegalArgumentException("the JSON is not an object");
    }

    private IllegalArgumentException refused(String why) {
        return new IllegalArgumentException("not JSON at character " + at + ": " + why);
    }

    private void space() {
        while (at < text.length()) {
            char c = text.charAt(at);
            if (c != ' ' && c != '\t' && c != '\n' && c != '\r') {
                return;
            }
            at += 1;
        }
    }

    private Object value() {
        if (at >= text.length()) {
            throw refused("the text ends where a value belongs");
        }
        char c = text.charAt(at);
        return switch (c) {
            case '{' -> members();
            case '[' -> elements();
            case '"' -> string();
            case 't' -> word("true", Boolean.TRUE);
            case 'f' -> word("false", Boolean.FALSE);
            case 'n' -> word("null", null);
            default -> number();
        };
    }

    private Object word(String word, Object value) {
        if (!text.startsWith(word, at)) {
            throw refused("expected `" + word + "`");
        }
        at += word.length();
        return value;
    }

    private Map<String, Object> members() {
        Map<String, Object> members = new LinkedHashMap<>();
        at += 1;
        space();
        if (peek('}')) {
            at += 1;
            return Collections.unmodifiableMap(members);
        }
        while (true) {
            space();
            if (!peek('"')) {
                throw refused("expected a key");
            }
            String key = string();
            space();
            expect(':');
            space();
            if (members.containsKey(key)) {
                throw refused("the key `" + key + "` is given twice");
            }
            members.put(key, value());
            space();
            if (peek(',')) {
                at += 1;
                continue;
            }
            expect('}');
            return Collections.unmodifiableMap(members);
        }
    }

    private List<Object> elements() {
        List<Object> elements = new ArrayList<>();
        at += 1;
        space();
        if (peek(']')) {
            at += 1;
            return Collections.unmodifiableList(elements);
        }
        while (true) {
            space();
            elements.add(value());
            space();
            if (peek(',')) {
                at += 1;
                continue;
            }
            expect(']');
            return Collections.unmodifiableList(elements);
        }
    }

    private String string() {
        at += 1;
        StringBuilder out = new StringBuilder();
        while (at < text.length()) {
            char c = text.charAt(at);
            at += 1;
            if (c == '"') {
                return out.toString();
            }
            if (c < 0x20) {
                throw refused("a control character inside a string");
            }
            if (c != '\\') {
                out.append(c);
                continue;
            }
            if (at >= text.length()) {
                break;
            }
            char escaped = text.charAt(at);
            at += 1;
            switch (escaped) {
                case '"' -> out.append('"');
                case '\\' -> out.append('\\');
                case '/' -> out.append('/');
                case 'b' -> out.append('\b');
                case 'f' -> out.append('\f');
                case 'n' -> out.append('\n');
                case 'r' -> out.append('\r');
                case 't' -> out.append('\t');
                case 'u' -> {
                    if (at + 4 > text.length()) {
                        throw refused("a short `\\u` escape");
                    }
                    try {
                        out.append((char) Integer.parseInt(text.substring(at, at + 4), 16));
                    } catch (NumberFormatException e) {
                        throw refused("a `\\u` escape that is not hexadecimal");
                    }
                    at += 4;
                }
                default -> throw refused("an unknown escape `\\" + escaped + "`");
            }
        }
        throw refused("the text ends inside a string");
    }

    private Object number() {
        int start = at;
        if (peek('-')) {
            at += 1;
        }
        if (peek('0')) {
            at += 1;
        } else if (!digits()) {
            throw refused("expected a value");
        }
        boolean integer = true;
        if (peek('.')) {
            at += 1;
            integer = false;
            if (!digits()) {
                throw refused("a fraction without digits");
            }
        }
        if (peek('e') || peek('E')) {
            at += 1;
            integer = false;
            if (peek('+') || peek('-')) {
                at += 1;
            }
            if (!digits()) {
                throw refused("an exponent without digits");
            }
        }
        String written = text.substring(start, at);
        if (integer) {
            try {
                return Long.parseLong(written);
            } catch (NumberFormatException e) {
                // An integer too wide for a long reads as the double nearest it.
            }
        }
        return Double.parseDouble(written);
    }

    private boolean digits() {
        int start = at;
        while (at < text.length() && text.charAt(at) >= '0' && text.charAt(at) <= '9') {
            at += 1;
        }
        return at > start;
    }

    private boolean peek(char c) {
        return at < text.length() && text.charAt(at) == c;
    }

    private void expect(char c) {
        if (!peek(c)) {
            throw refused("expected `" + c + "`");
        }
        at += 1;
    }
}

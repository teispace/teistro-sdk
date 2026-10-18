package com.teispace.teistro;

import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.function.Function;

/**
 * The {@code sarvatobhadra} section parsed once a batch into {@link Sarvatobhadra} records, its
 * catalogue keys made members, as the Python façade's {@code Chart.sarvatobhadra} reads it.
 */
final class SarvatobhadraReads {
    private SarvatobhadraReads() {
    }

    /**
     * The chakra at each of the record's instants; empty unless {@code sarvatobhadra} asked for it
     * ({@code 03-design/gochar.md} §8). Ports {@code Chart.sarvatobhadra}.
     */
    static Optional<List<Sarvatobhadra>> sarvatobhadra(Chart chart) {
        return VedicReads.at(chart.batch().cached("sarvatobhadra", () -> {
            String text = chart.batch().decoded().sarvatobhadra();
            if (text.isEmpty()) {
                return List.<List<Sarvatobhadra>>of();
            }
            return each(Json.read(text), one -> each(one, SarvatobhadraReads::readingOf));
        }), chart.index());
    }

    private static Object at(Object raw, String key) {
        return JsonRead.at(raw, key);
    }

    /** A member the boundary leaves out where it does not apply, or null. */
    private static Object optional(Object raw, String key) {
        return JsonRead.object(raw).get(key);
    }

    private static Graha graha(Object key) {
        return JsonRead.member(Graha::byKey, "Graha", key);
    }

    private static List<Graha> grahas(Object keys) {
        return JsonRead.members(Graha::byKey, "Graha", keys);
    }

    private static Nakshatra nakshatra(Object key) {
        return JsonRead.member(Nakshatra::byKey, "Nakshatra", key);
    }

    private static <T> List<T> each(Object raw, Function<Object, T> read) {
        List<T> out = new ArrayList<>();
        for (Object one : JsonRead.list(raw)) {
            out.add(read.apply(one));
        }
        return List.copyOf(out);
    }

    private static Sarvatobhadra.Star star(Object raw) {
        return new Sarvatobhadra.Star(
                "NAKSHATRA".equals(JsonRead.text(at(raw, "star"))) ? nakshatra(at(raw, "nakshatra")) : null);
    }

    private static Sarvatobhadra.Cell cell(Object raw) {
        String kind = JsonRead.text(at(raw, "cell"));
        return new Sarvatobhadra.Cell(
                kind,
                "STAR".equals(kind) ? star(at(raw, "star")) : null,
                JsonRead.textOrNull(optional(raw, "vowel")),
                JsonRead.textOrNull(optional(raw, "letter")),
                "RASHI".equals(kind) ? JsonRead.member(Rashi::byKey, "Rashi", at(raw, "rashi")) : null,
                "TITHI".equals(kind) ? JsonRead.member(TithiClass::byKey, "TithiClass", at(raw, "class")) : null);
    }

    private static Sarvatobhadra.Reach reach(Object raw) {
        Object cell = optional(raw, "cell");
        return new Sarvatobhadra.Reach(
                JsonRead.text(at(raw, "by")),
                JsonRead.textOrNull(optional(raw, "line")),
                cell == null ? null : cell(cell));
    }

    private static Sarvatobhadra.GrahaVedha vedha(Object raw) {
        Object place = at(raw, "place");
        return new Sarvatobhadra.GrahaVedha(
                graha(at(raw, "graha")),
                new Sarvatobhadra.Place(star(at(place, "star")), nakshatra(at(place, "nakshatra")),
                        JsonRead.integer(at(place, "pada"))),
                JsonRead.text(at(raw, "motion")),
                JsonRead.texts(at(raw, "lines")),
                JsonRead.text(at(raw, "nature")),
                JsonRead.bool(at(raw, "waning")),
                grahas(at(raw, "joinedBy")),
                each(at(raw, "cells"), SarvatobhadraReads::reach),
                each(at(raw, "points"), p -> new Sarvatobhadra.PointStrike(
                        JsonRead.text(at(p, "point")), each(at(p, "by"), SarvatobhadraReads::reach))));
    }

    private static Sarvatobhadra readingOf(Object raw) {
        Object rules = at(raw, "rules");
        Object native_ = at(raw, "native");
        Object name = optional(native_, "name");
        Map<String, Object> named = name == null ? null : JsonRead.object(name);
        return new Sarvatobhadra(
                new Sarvatobhadra.Rules(JsonRead.text(at(rules, "front")), JsonRead.text(at(rules, "sight")),
                        JsonRead.text(at(rules, "corner"))),
                new Sarvatobhadra.Native(
                        nakshatra(at(native_, "nakshatra")),
                        JsonRead.member(Rashi::byKey, "Rashi", at(native_, "rashi")),
                        JsonRead.member(Tithi::byKey, "Tithi", at(native_, "tithi")),
                        JsonRead.member(Vara::byKey, "Vara", at(native_, "vara")),
                        named == null ? null
                                : new Sarvatobhadra.Name(JsonRead.text(named.get("letter")),
                                        JsonRead.text(named.get("vowel")))),
                each(at(raw, "grahas"), SarvatobhadraReads::vedha),
                each(at(raw, "points"), p -> new Sarvatobhadra.PointVedha(JsonRead.text(at(p, "point")),
                        grahas(at(p, "malefics")), grahas(at(p, "benefics")))),
                JsonRead.integer(at(raw, "maleficPoints")));
    }
}

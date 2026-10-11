package com.teispace.teistro;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import java.util.function.Function;

/**
 * The {@code lalkitab} section parsed once a batch into {@link LalKitab} records, its grahas made
 * catalogue members, as the Python façade's {@code Chart.lalkitab} reads it.
 */
final class LalKitabReads {
    private LalKitabReads() {
    }

    /**
     * The chart read as Lal Kitab reads it; empty unless {@code lalkitab} asked for it
     * ({@code 03-design/lalkitab.md}). Ports {@code Chart.lalkitab}.
     */
    static Optional<LalKitab> lalkitab(Chart chart) {
        return VedicReads.at(chart.batch().cached("lalkitab", () -> {
            String text = chart.batch().decoded().lalkitab();
            if (text.isEmpty()) {
                return List.<LalKitab>of();
            }
            return each(Json.read(text), LalKitabReads::lalkitabOf);
        }), chart.index());
    }

    private static Object at(Object raw, String key) {
        return JsonRead.at(raw, key);
    }

    private static Graha graha(Object key) {
        return JsonRead.member(Graha::byKey, "Graha", key);
    }

    private static List<Graha> grahas(Object keys) {
        return JsonRead.members(Graha::byKey, "Graha", keys);
    }

    private static <T> List<T> each(Object raw, Function<Object, T> read) {
        List<T> out = new ArrayList<>();
        for (Object one : JsonRead.list(raw)) {
            out.add(read.apply(one));
        }
        return List.copyOf(out);
    }

    private static LalKitab.Planet planet(Object one) {
        return new LalKitab.Planet(
                graha(at(one, "graha")),
                JsonRead.integer(at(one, "house")),
                JsonRead.texts(at(one, "dignities")),
                each(at(one, "owners"),
                        o -> new LalKitab.Owner(graha(at(o, "owner")), JsonRead.text(at(o, "regard")))),
                JsonRead.bool(at(one, "awake")),
                JsonRead.bool(at(one, "kayam")),
                each(at(one, "casts"), c -> new LalKitab.Cast(JsonRead.integer(at(c, "to")),
                        JsonRead.text(at(c, "strength")), grahas(at(c, "onto")))));
    }

    private static LalKitab.House house(Object one) {
        return new LalKitab.House(
                JsonRead.integer(at(one, "house")),
                grahas(at(one, "occupants")),
                each(at(one, "lookedAtBy"),
                        l -> new LalKitab.Look(JsonRead.integer(at(l, "from")), JsonRead.text(at(l, "strength")))),
                JsonRead.bool(at(one, "awake")),
                graha(at(one, "waker")));
    }

    /** A teva's reading from the section's JSON. */
    private static LalKitab.Reading reading(Object raw) {
        Object flags = at(raw, "flags");
        return new LalKitab.Reading(
                each(at(raw, "planets"), LalKitabReads::planet),
                each(at(raw, "houses"), LalKitabReads::house),
                each(at(raw, "masnui"), m -> new LalKitab.Pair(grahas(at(m, "pair")),
                        JsonRead.integer(at(m, "house")), JsonRead.text(at(m, "countsAs")))),
                each(at(raw, "rinas"), r -> new LalKitab.Debt(JsonRead.text(at(r, "rin")), graha(at(r, "of")),
                        each(at(r, "seated"),
                                s -> new LalKitab.Seat(graha(at(s, "enemy")), JsonRead.integer(at(s, "house")))))),
                each(at(raw, "pitri"),
                        p -> new LalKitab.Pitri(graha(at(p, "ninth")), JsonRead.integer(at(p, "mercury")))),
                new LalKitab.Flags(
                        JsonRead.bool(at(flags, "ratandha")),
                        JsonRead.bool(at(flags, "nabalig")),
                        grahas(at(flags, "dharmi")),
                        each(at(flags, "sathi"), LalKitabReads::grahas)));
    }

    /** A chart's Lal Kitab from the section's JSON, its grahas made members. */
    private static LalKitab lalkitabOf(Object raw) {
        Object cycle = at(raw, "cycle");
        Object year = at(raw, "year");
        Object annual = year == null ? null : at(year, "annual");
        return new LalKitab(
                reading(at(raw, "reading")),
                new LalKitab.Cycle(graha(at(cycle, "planet")), JsonRead.integer(at(cycle, "year"))),
                each(at(raw, "periods"), p -> new LalKitab.Period(graha(at(p, "planet")),
                        JsonRead.integer(at(p, "from")), JsonRead.integer(at(p, "to")))),
                year == null ? null
                        : new LalKitab.Year(
                                JsonRead.integer(at(year, "year")),
                                graha(at(year, "ruler")),
                                grahas(at(year, "thirds")),
                                annual == null ? null : reading(annual)));
    }
}

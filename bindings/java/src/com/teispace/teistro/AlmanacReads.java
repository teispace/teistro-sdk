package com.teispace.teistro;

import java.util.List;
import java.util.Map;

import com.teispace.teistro.record.Provenance;

/**
 * The almanac's JSON sections, each an envelope of a value and its
 * provenance, read into this binding's records with their keys made members.
 */
final class AlmanacReads {
    private AlmanacReads() {}

    private static Provenance provenance(Map<?, ?> envelope) {
        return Provenance.of(Reads.field(envelope, "provenance"));
    }

    private static <E extends Member> E member(Map<?, ?> object, String key, String kind,
            java.util.function.Function<String, java.util.Optional<E>> byKey) {
        String text = Reads.string(object, key);
        return Reads.member(byKey.apply(text), kind, text);
    }

    /** The {@code muhurta} section: the envelope's value, its members resolved, with the provenance beside it. */
    static MuhurtaAnswer muhurtaAnswer(String text) {
        Map<?, ?> envelope = Reads.object(Json.read(text));
        Map<?, ?> value = Reads.object(envelope, "value");
        return new MuhurtaAnswer(
                Reads.each(value, "windows", AlmanacReads::window),
                Reads.each(value, "closed", raw -> {
                    Map<?, ?> d = Reads.object(raw);
                    return new ClosedDay(Reads.serdeDate(Reads.field(d, "date")), Reads.each(d, "by",
                            k -> Reads.member(BlackoutKind.byKey(Reads.string(k)), "BlackoutKind", k)));
                }),
                Reads.integer(value, "daysJudged"),
                Reads.integer(value, "daysCut"),
                Reads.integer(value, "windowsBlackedOut"),
                Reads.string(value, "ranking"),
                Reads.each(value, "unjudged", raw -> {
                    Map<?, ?> u = Reads.object(raw);
                    return new MuhurtaUnjudged(Reads.string(u, "what"), Reads.string(u, "why"));
                }),
                provenance(envelope));
    }

    private static MuhurtaWindow window(Object raw) {
        Map<?, ?> w = Reads.object(raw);
        Object scored = Reads.field(w, "score");
        MuhurtaScore score = null;
        if (scored != null) {
            Map<?, ?> s = Reads.object(scored);
            score = new MuhurtaScore(
                    Reads.integer(s, "value"),
                    Reads.each(s, "factors", one -> {
                        Map<?, ?> f = Reads.object(one);
                        Object graha = Reads.field(f, "graha");
                        return new MuhurtaFactor(Reads.string(f, "dimension"), Reads.integer(f, "weight"),
                                graha == null ? null : Reads.member(Graha.byKey(Reads.string(graha)), "Graha", graha));
                    }),
                    Reads.optionalInteger(s, "cappedAt"));
        }
        return new MuhurtaWindow(
                Reads.interval(Reads.field(w, "at")),
                Reads.each(w, "clauses", c -> new MuhurtaClause(MuhurtaClauseKind.read(c),
                        Reads.interval(Reads.field(Reads.object(c), "at")))),
                Reads.each(w, "barredBy", MuhurtaBar::read),
                score);
    }

    /** The {@code festivals} section: the envelope's value, its dates in this binding's shape. */
    static FestivalAnswer festivalsAnswer(String text) {
        Map<?, ?> envelope = Reads.object(Json.read(text));
        Map<?, ?> value = Reads.object(envelope, "value");
        return new FestivalAnswer(
                Reads.each(value, "observances", AlmanacReads::observance),
                Reads.each(value, "ekadashis", AlmanacReads::fast),
                Reads.each(value, "unjudged", raw -> {
                    Map<?, ?> u = Reads.object(raw);
                    return new FestivalUnjudged(Reads.string(u, "rule"), Reads.interval(Reads.field(u, "tithi")),
                            Reads.string(u, "why"));
                }),
                provenance(envelope));
    }

    private static FestivalExtent extent(Object raw) {
        Map<?, ?> e = Reads.object(raw);
        return new FestivalExtent(Reads.serdeDate(Reads.field(e, "day")), Reads.interval(Reads.field(e, "window")),
                Reads.number(e, "held"));
    }

    private static FestivalObservance observance(Object raw) {
        Map<?, ?> o = Reads.object(raw);
        List<FestivalExtent> extents = Reads.each(o, "extents", AlmanacReads::extent);
        if (extents.size() != 2) {
            throw Reads.internal("an observance has two extents, and the library wrote " + extents.size());
        }
        Map<?, ?> decided = Reads.object(o, "decidedBy");
        return new FestivalObservance(
                Reads.string(o, "rule"),
                Reads.serdeDate(Reads.field(o, "day")),
                Reads.interval(Reads.field(o, "tithi")),
                member(o, "month", "Masa", Masa::byKey),
                Reads.flag(o, "adhika"),
                Reads.string(o, "case"),
                extents,
                new FestivalDecided(Reads.string(decided, "by"), Reads.optionalInteger(decided, "index"),
                        Reads.optionalString(decided, "rule"), Reads.optionalInteger(decided, "days")),
                Reads.string(o, "choice"));
    }

    private static EkadashiFast fast(Object raw) {
        Map<?, ?> f = Reads.object(raw);
        List<Interval> tithis = Reads.each(f, "tithis", Reads::interval);
        List<CalendarDate> days = Reads.each(f, "days", Reads::serdeDate);
        if (tithis.size() != 3 || days.size() != 2) {
            throw Reads.internal("an Ekadashi fast has three tithis and two days, and the library wrote "
                    + tithis.size() + " and " + days.size());
        }
        return new EkadashiFast(
                Reads.string(f, "rule"),
                member(f, "tithi", "Tithi", Tithi::byKey),
                member(f, "month", "Masa", Masa::byKey),
                Reads.flag(f, "adhika"),
                tithis,
                days,
                Reads.optionalString(f, "piercedAt"),
                Reads.flag(f, "pierced"),
                Reads.string(f, "excess"),
                Reads.string(f, "choice"),
                Reads.serdeDate(Reads.field(f, "day")));
    }

    /** The {@code years} section: the envelope's years, with the provenance beside them. */
    static LunarYears yearsAnswer(String text) {
        Map<?, ?> envelope = Reads.object(Json.read(text));
        return new LunarYears(Reads.each(envelope, "value", raw -> {
            Map<?, ?> y = Reads.object(raw);
            Object lupta = Reads.field(y, "lupta");
            return new LunarYear(
                    member(y, "samvatsara", "Samvatsara", Samvatsara::byKey),
                    Reads.string(y, "count"),
                    Reads.integer(y, "vikrama"),
                    Reads.integer(y, "shaka"),
                    Reads.number(y, "opened"),
                    Reads.number(y, "began"),
                    Reads.number(y, "ended"),
                    Reads.each(y, "jovian", one -> {
                        Map<?, ?> j = Reads.object(one);
                        return new JovianYear(member(j, "member", "Samvatsara", Samvatsara::byKey),
                                Reads.integer(j, "count"), Reads.number(j, "from"), Reads.number(j, "to"));
                    }),
                    lupta == null ? null : Reads.member(Samvatsara.byKey(Reads.string(lupta)), "Samvatsara", lupta));
        }), provenance(envelope));
    }

    /** The {@code nepal_sambat} section: one date a day, with the provenance beside them. */
    static NepalSambatDates nepalSambatAnswer(String text) {
        Map<?, ?> envelope = Reads.object(Json.read(text));
        return new NepalSambatDates(Reads.each(envelope, "value", raw -> {
            Map<?, ?> d = Reads.object(raw);
            return new NepalSambatDate(Reads.integer(d, "year"), Reads.integer(d, "month"),
                    member(d, "kind", "MonthKind", MonthKind::byKey), member(d, "paksha", "Paksha", Paksha::byKey));
        }), provenance(envelope));
    }

    private static EclipseMoment moment(Object raw) {
        Map<?, ?> m = Reads.object(raw);
        return new EclipseMoment(Reads.number(m, "at"), Reads.number(m, "altitudeDeg"));
    }

    private static EclipseMoment reached(Map<?, ?> object, String key) {
        Object raw = Reads.field(object, key);
        return raw == null ? null : moment(raw);
    }

    private static EclipseSeen seen(Map<?, ?> object, String key) {
        Object raw = Reads.field(object, key);
        if (raw == null) {
            return null;
        }
        Map<?, ?> s = Reads.object(raw);
        return new EclipseSeen(Reads.number(s, "from"), Reads.number(s, "to"));
    }

    /** The {@code eclipses} section: the envelope's eclipses, with the provenance beside them. */
    static Eclipses eclipsesAnswer(String text) {
        Map<?, ?> envelope = Reads.object(Json.read(text));
        Map<?, ?> value = Reads.object(envelope, "value");
        return new Eclipses(new EclipsesFound(
                Reads.each(value, "lunar", AlmanacReads::lunar),
                Reads.each(value, "solar", AlmanacReads::solar)), provenance(envelope));
    }

    private static LunarEclipseHere lunar(Object raw) {
        Map<?, ?> both = Reads.object(raw);
        Map<?, ?> e = Reads.object(both, "eclipse");
        Map<?, ?> h = Reads.object(both, "here");
        Map<?, ?> c = Reads.object(e, "contacts");
        return new LunarEclipseHere(
                new LunarEclipse(
                        Reads.number(e, "greatest"),
                        member(e, "kind", "LunarEclipseKind", LunarEclipseKind::byKey),
                        Reads.number(e, "gamma"),
                        Reads.number(e, "umbralMagnitude"),
                        Reads.number(e, "penumbralMagnitude"),
                        new LunarContacts(Reads.number(c, "p1"), Reads.optionalNumber(c, "u1"),
                                Reads.optionalNumber(c, "u2"), Reads.optionalNumber(c, "u3"),
                                Reads.optionalNumber(c, "u4"), Reads.number(c, "p4")),
                        Reads.string(e, "shadow")),
                new LunarEclipseView(
                        moment(Reads.field(h, "p1")),
                        reached(h, "u1"),
                        reached(h, "u2"),
                        moment(Reads.field(h, "greatest")),
                        reached(h, "u3"),
                        reached(h, "u4"),
                        moment(Reads.field(h, "p4")),
                        seen(h, "seen"),
                        seen(h, "umbralSeen")));
    }

    private static SolarEclipseHere solar(Object raw) {
        Map<?, ?> both = Reads.object(raw);
        Map<?, ?> e = Reads.object(both, "eclipse");
        Map<?, ?> point = Reads.object(e, "point");
        Object here = Reads.field(both, "here");
        SolarEclipseView view = null;
        if (here != null) {
            Map<?, ?> h = Reads.object(here);
            view = new SolarEclipseView(
                    member(h, "kind", "SolarEclipseKind", SolarEclipseKind::byKey),
                    Reads.number(h, "magnitude"),
                    Reads.number(h, "obscuration"),
                    moment(Reads.field(h, "first")),
                    reached(h, "second"),
                    reached(h, "third"),
                    moment(Reads.field(h, "fourth")),
                    moment(Reads.field(h, "maximum")),
                    seen(h, "seen"));
        }
        return new SolarEclipseHere(
                new SolarEclipse(
                        Reads.number(e, "greatest"),
                        member(e, "kind", "SolarEclipseKind", SolarEclipseKind::byKey),
                        Reads.number(e, "gamma"),
                        Reads.number(e, "magnitude"),
                        new EclipsePoint(Reads.number(point, "latitude"), Reads.number(point, "longitude"))),
                view);
    }
}

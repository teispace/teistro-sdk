package com.teispace.teistro;

import java.util.List;
import java.util.Map;

import com.teispace.teistro.record.Provenance;
import com.teispace.teistro.request.AshauchKind;
import com.teispace.teistro.request.AshauchLength;
import com.teispace.teistro.request.AshauchRelation;
import com.teispace.teistro.request.AshauchSex;
import com.teispace.teistro.request.NightRule;

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
                // Left out of the answer when no dated rule gave a day.
                value.containsKey("dated")
                        ? Reads.each(value, "dated", raw -> {
                            Map<?, ?> d = Reads.object(raw);
                            return new DatedObservance(Reads.string(d, "rule"),
                                    Reads.serdeDate(Reads.field(d, "day")), Reads.serdeDate(Reads.field(d, "on")),
                                    published(d));
                        })
                        : java.util.List.of(),
                // Left out of the answer when none was asked.
                value.containsKey("anniversaries")
                        ? Reads.each(value, "anniversaries", AlmanacReads::anniversary)
                        : java.util.List.of(),
                // Each left out when no published set was laid over the answer.
                value.containsKey("added")
                        ? Reads.each(value, "added", raw -> {
                            Map<?, ?> a = Reads.object(raw);
                            Map<String, String> names = new java.util.TreeMap<>();
                            if (a.get("names") != null) {
                                Reads.object(a, "names").forEach((k, v) -> names.put(Reads.string(k), Reads.string(v)));
                            }
                            return new Published.Added(Reads.string(a, "key"), Reads.serdeDate(Reads.field(a, "day")),
                                    names, published(a));
                        })
                        : java.util.List.of(),
                value.containsKey("unplaced")
                        ? Reads.each(value, "unplaced", raw -> {
                            Map<?, ?> u = Reads.object(raw);
                            return new Published.Unplaced(Reads.string(u, "set"), Reads.integer(u, "entry"),
                                    Reads.string(u, "rule"), Reads.serdeDate(Reads.field(u, "day")));
                        })
                        : java.util.List.of(),
                value.containsKey("applied")
                        ? Reads.each(value, "applied", raw -> {
                            Map<?, ?> a = Reads.object(raw);
                            return new Published.Applied(Reads.string(a, "id"), Reads.integer(a, "version"),
                                    Reads.string(a, "publisher"), Reads.string(a, "hash"));
                        })
                        : java.util.List.of(),
                provenance(envelope));
    }

    /** What a published entry did to a day, or null when none touched it. */
    private static Published published(Map<?, ?> holder) {
        Object raw = holder.get("published");
        if (raw == null) {
            return null;
        }
        Map<?, ?> p = Reads.object(raw);
        return new Published(Reads.string(p, "status"), Reads.string(p, "publisher"), Reads.string(p, "set"),
                Reads.integer(p, "version"), Reads.string(p, "source"),
                p.get("computed") == null ? null : Reads.serdeDate(Reads.field(p, "computed")),
                p.get("window") == null ? null : Reads.interval(Reads.field(p, "window")));
    }

    private static AnniversaryDay anniversary(Object raw) {
        Map<?, ?> a = Reads.object(raw);
        Map<?, ?> o = Reads.object(a, "origin");
        return new AnniversaryDay(
                Reads.string(a, "key"),
                Reads.optionalInteger(a, "count"),
                Reads.string(a, "kind"),
                origin(o),
                Reads.string(a, "monthBy"),
                observance(Reads.field(a, "observance")));
    }

    /** An anniversary's origin as found, as an anniversary and a first year write it. */
    private static AnniversaryDay.Origin origin(Map<?, ?> o) {
        return new AnniversaryDay.Origin(
                member(o, "tithi", "Tithi", Tithi::byKey),
                member(o, "month", "Masa", Masa::byKey),
                Reads.string(o, "kind"),
                Reads.string(o, "half"),
                o.get("nakshatra") == null ? null : member(o, "nakshatra", "Nakshatra", Nakshatra::byKey),
                Reads.optionalNumber(o, "at"),
                Reads.flag(o, "assumedSunrise"),
                o.containsKey("candidates")
                        ? Reads.each(o, "candidates", t -> {
                            String text = String.valueOf(t);
                            return Reads.member(Tithi.byKey(text), "Tithi", text);
                        })
                        : List.of());
    }

    /** {@code ts_almanac_published_reasons}'s answer, with the provenance beside it. */
    static PublishedReasons publishedReasons(String text) {
        Map<?, ?> envelope = Reads.object(Json.read(text));
        Map<?, ?> value = Reads.object(envelope, "value");
        return new PublishedReasons(Reads.each(value, "reasons", raw -> {
            Map<?, ?> r = Reads.object(raw);
            return new PublishedReasons.Reason(Reads.string(r, "rule"), Reads.string(r, "set"),
                    Reads.serdeDate(Reads.field(r, "day")), Reads.serdeDate(Reads.field(r, "computed")),
                    cause(Reads.object(r, "reason")),
                    Reads.each(r, "tried", t -> {
                        Map<?, ?> tried = Reads.object(t);
                        // Absent when the change gives no day within seven.
                        Object day = tried.get("day");
                        return new PublishedReasons.Tried(cause(Reads.object(tried, "change")),
                                day == null ? null : Reads.serdeDate(day), Reads.flag(tried, "reproduces"));
                    }));
        }), provenance(envelope));
    }

    private static PublishedReasons.Cause cause(Map<?, ?> raw) {
        return new PublishedReasons.Cause(Reads.string(raw, "by"), Reads.optionalString(raw, "name"),
                Reads.optionalString(raw, "profile"), Reads.optionalString(raw, "settingsHash"),
                Reads.optionalString(raw, "pack"));
    }

    /** {@code ts_almanac_disagreements}'s answer: each comparison asked, its members resolved, or null. */
    static Disagreements disagreements(String text) {
        Map<?, ?> answer = Reads.object(Json.read(text));
        Object days = Reads.field(answer, "days");
        Object festivals = Reads.field(answer, "festivals");
        return new Disagreements(days == null ? null : dayDisagreements(Reads.object(days)),
                festivals == null ? null : festivalDisagreements(Reads.object(festivals)));
    }

    private static Disagreements.Days dayDisagreements(Map<?, ?> envelope) {
        Map<?, ?> value = Reads.object(envelope, "value");
        return new Disagreements.Days(Reads.string(value, "otherProfile"), Reads.string(value, "otherSettingsHash"),
                Reads.integer(value, "compared"),
                Reads.each(value, "days", raw -> {
                    Map<?, ?> d = Reads.object(raw);
                    return new Disagreements.Day(Reads.serdeDate(Reads.field(d, "date")),
                            limb(d, "tithi", "Tithi", Tithi::byKey),
                            limb(d, "nakshatra", "Nakshatra", Nakshatra::byKey),
                            limb(d, "yoga", "Yoga", Yoga::byKey),
                            limb(d, "karana", "Karana", Karana::byKey));
                }),
                provenance(envelope));
    }

    private static <E extends Member> Disagreements.Limb<E> limb(Map<?, ?> day, String key, String kind,
            java.util.function.Function<String, java.util.Optional<E>> byKey) {
        Object raw = Reads.field(day, key);
        if (raw == null) {
            return null;
        }
        Map<?, ?> both = Reads.object(raw);
        return new Disagreements.Limb<>(side(Reads.object(both, "official"), kind, byKey),
                side(Reads.object(both, "other"), kind, byKey));
    }

    private static <E extends Member> Disagreements.Side<E> side(Map<?, ?> raw, String kind,
            java.util.function.Function<String, java.util.Optional<E>> byKey) {
        Object change = Reads.field(raw, "change");
        Disagreements.Change at = null;
        if (change != null) {
            Map<?, ?> c = Reads.object(change);
            at = new Disagreements.Change(Reads.number(c, "at"), Reads.number(c, "minutesFromSunrise"));
        }
        return new Disagreements.Side<>(member(raw, "member", kind, byKey), Reads.interval(Reads.field(raw, "span")),
                Reads.number(raw, "sunrise"), at);
    }

    private static Disagreements.Festivals festivalDisagreements(Map<?, ?> envelope) {
        Map<?, ?> value = Reads.object(envelope, "value");
        return new Disagreements.Festivals(Reads.string(value, "otherProfile"),
                Reads.string(value, "otherSettingsHash"), Reads.integer(value, "compared"),
                Reads.each(value, "observances", raw -> {
                    Map<?, ?> o = Reads.object(raw);
                    Object official = Reads.field(o, "official");
                    Object other = Reads.field(o, "other");
                    return new Disagreements.Festival(Reads.string(o, "rule"), member(o, "month", "Masa", Masa::byKey),
                            Reads.flag(o, "adhika"), official == null ? null : observance(official),
                            other == null ? null : observance(other));
                }),
                Reads.each(value, "ekadashis", raw -> {
                    Map<?, ?> f = Reads.object(raw);
                    Object official = Reads.field(f, "official");
                    Object other = Reads.field(f, "other");
                    return new Disagreements.Ekadashi(Reads.string(f, "rule"), member(f, "tithi", "Tithi", Tithi::byKey),
                            member(f, "month", "Masa", Masa::byKey), Reads.flag(f, "adhika"),
                            official == null ? null : fast(official), other == null ? null : fast(other));
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
                Reads.string(o, "choice"),
                Reads.number(o, "marginSeconds"),
                o.get("parana") == null ? List.of() : Reads.each(o, "parana", AlmanacReads::vrataParana),
                published(o),
                o.get("sait") == null ? null : sait(Reads.object(o, "sait")));
    }

    /** A sait as a festival answer writes it. */
    private static FestivalSait sait(Map<?, ?> s) {
        return new FestivalSait(
                Reads.integer(s, "window"),
                Reads.interval(Reads.field(s, "within")),
                Reads.each(s, "candidates", raw -> {
                    Map<?, ?> c = Reads.object(raw);
                    return new FestivalSait.Candidate(
                            Reads.interval(Reads.field(c, "span")),
                            Reads.flag(c, "whole"),
                            c.get("lagna") == null ? null : member(c, "lagna", "Rashi", Rashi::byKey),
                            c.get("navamsa") == null ? null : member(c, "navamsa", "Rashi", Rashi::byKey));
                }),
                Reads.optionalInteger(s, "chosen"),
                s.get("at") == null ? null : Reads.number(s, "at"));
    }

    /** A vrata's parana under one reading, as a festival answer writes it. */
    private static VrataParanaTime vrataParana(Object raw) {
        Map<?, ?> p = Reads.object(raw);
        Map<?, ?> reading = Reads.object(p, "reading");
        return new VrataParanaTime(
                new VrataReading(
                        Reads.string(reading, "kind"),
                        reading.get("nakshatra") == null ? null
                                : member(reading, "nakshatra", "Nakshatra", Nakshatra::byKey),
                        Reads.optionalInteger(reading, "yamas"),
                        reading.get("at") == null ? null : festivalWindow(Reads.object(reading, "at")),
                        Reads.optionalString(reading, "rite"),
                        Reads.optionalInteger(reading, "remnant")),
                Reads.serdeDate(Reads.field(p, "day")),
                Reads.number(p, "from"),
                Reads.string(p, "fromBy"),
                Reads.optionalNumber(p, "to"),
                Reads.optionalString(p, "toBy"));
    }

    /** A rite's window as a rule writes it. */
    private static FestivalWindow festivalWindow(Map<?, ?> at) {
        return new FestivalWindow(
                Reads.string(at, "window"),
                Reads.optionalString(at, "part"),
                Reads.optionalInteger(at, "muhurta"),
                Reads.optionalInteger(at, "from"),
                Reads.optionalInteger(at, "to"));
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
                Reads.serdeDate(Reads.field(f, "day")),
                Reads.number(f, "marginSeconds"),
                f.get("parana") == null ? null : parana(Reads.object(f, "parana")),
                f.get("mahadvadashi") == null ? null : mahadvadashi(Reads.object(f, "mahadvadashi")),
                published(f));
    }

    /** A fast's mahadvadashi as a festival answer writes it. */
    private static Mahadvadashi mahadvadashi(Map<?, ?> m) {
        Map<?, ?> joined = m.get("joined") == null ? null : Reads.object(m, "joined");
        return new Mahadvadashi(
                Reads.string(m, "kind"),
                Reads.each(m, "held", Reads::string),
                Reads.string(m, "clause"),
                joined == null ? null : member(joined, "nakshatra", "Nakshatra", Nakshatra::byKey),
                joined == null ? null : Reads.interval(Reads.field(joined, "span")));
    }

    /** A fast's parana as a festival answer writes it. */
    private static Parana parana(Map<?, ?> p) {
        Map<?, ?> kept = p.get("kept") == null ? null : Reads.object(p, "kept");
        return new Parana(
                Reads.serdeDate(Reads.field(p, "day")),
                member(p, "tithi", "Tithi", Tithi::byKey),
                Reads.optionalNumber(p, "hariVasaraEnds"),
                kept == null ? null : new ParanaKept(
                        Reads.interval(Reads.field(kept, "span")),
                        paranaWindow(Reads.object(kept, "at")),
                        Reads.string(kept, "fromBy"),
                        Reads.string(kept, "toBy")),
                p.get("barred") == null ? null : paranaBarred(Reads.object(p, "barred")));
    }

    /** A parana's barred share as a festival answer writes it. */
    private static ParanaBarred paranaBarred(Map<?, ?> b) {
        Map<?, ?> bar = Reads.object(b, "bar");
        Map<?, ?> portion = bar.get("portion") == null ? null : Reads.object(bar, "portion");
        return new ParanaBarred(
                new NakshatraBar(
                        member(bar, "tithi", "Tithi", Tithi::byKey),
                        member(bar, "month", "Masa", Masa::byKey),
                        member(bar, "nakshatra", "Nakshatra", Nakshatra::byKey),
                        portion == null ? 1 : Reads.integer(portion, "parts"),
                        portion == null ? 1 : Reads.integer(portion, "which")),
                Reads.interval(Reads.field(b, "span")),
                Reads.flag(b, "setAside"));
    }

    private static ParanaWindow paranaWindow(Map<?, ?> at) {
        return new ParanaWindow(
                Reads.string(at, "window"),
                Reads.optionalString(at, "part"),
                Reads.optionalInteger(at, "from"),
                Reads.optionalInteger(at, "to"));
    }

    /** The {@code years} section: the envelope's years, with the provenance beside them. */
    static LunarYears yearsAnswer(String text) {
        Map<?, ?> envelope = Reads.object(Json.read(text));
        return new LunarYears(Reads.each(envelope, "value", AlmanacReads::lunarYear), provenance(envelope));
    }

    /** One lunar year as a JSON answer writes it. */
    private static LunarYear lunarYear(Object raw) {
        Map<?, ?> y = Reads.object(raw);
        Object lupta = Reads.field(y, "lupta");
        return new LunarYear(
                member(y, "samvatsara", "Samvatsara", Samvatsara::byKey),
                Reads.string(y, "count"),
                Reads.integer(y, "vikrama"),
                Reads.integer(y, "shaka"),
                Reads.integer(y, "kali"),
                Reads.number(y, "opened"),
                Reads.number(y, "began"),
                Reads.number(y, "ended"),
                Reads.each(y, "jovian", one -> {
                    Map<?, ?> j = Reads.object(one);
                    return new JovianYear(member(j, "member", "Samvatsara", Samvatsara::byKey),
                            Reads.integer(j, "count"), Reads.number(j, "from"), Reads.number(j, "to"));
                }),
                lupta == null ? null : Reads.member(Samvatsara.byKey(Reads.string(lupta)), "Samvatsara", lupta));
    }

    /** {@code ts_almanac_sankalpa}'s answer: the time recited, its members resolved, with the provenance beside it. */
    static SankalpaAnswer sankalpaAnswer(String text) {
        Map<?, ?> envelope = Reads.object(Json.read(text));
        Map<?, ?> v = Reads.object(envelope, "value");
        Map<?, ?> m = Reads.object(v, "month");
        return new SankalpaAnswer(new Sankalpa(
                Reads.number(v, "at"),
                Reads.serdeDate(Reads.field(v, "date")),
                member(v, "vara", "Vara", Vara::byKey),
                Reads.number(v, "sunrise"),
                Reads.number(v, "nextSunrise"),
                lunarYear(Reads.field(v, "year")),
                member(v, "ayana", "Ayana", Ayana::byKey),
                Reads.string(v, "ayanaReckoning"),
                member(v, "ritu", "Ritu", Ritu::byKey),
                Reads.string(v, "rituReckoning"),
                new Month(
                        member(m, "month", "Masa", Masa::byKey),
                        member(m, "amanta", "Masa", Masa::byKey),
                        member(m, "purnimanta", "Masa", Masa::byKey),
                        member(m, "paksha", "Paksha", Paksha::byKey),
                        member(m, "convention", "LunarMonth", LunarMonth::byKey),
                        member(m, "kind", "MonthKind", MonthKind::byKey)),
                member(v, "paksha", "Paksha", Paksha::byKey),
                member(v, "tithi", "Tithi", Tithi::byKey),
                member(v, "nakshatra", "Nakshatra", Nakshatra::byKey),
                member(v, "yoga", "Yoga", Yoga::byKey),
                member(v, "karana", "Karana", Karana::byKey),
                member(v, "sun", "Rashi", Rashi::byKey),
                member(v, "moon", "Rashi", Rashi::byKey),
                member(v, "jupiter", "Rashi", Rashi::byKey)), provenance(envelope));
    }

    /** {@code ts_almanac_ashauch}'s answer: the period, its members resolved, with the provenance beside it. */
    static AshauchAnswer ashauchAnswer(String text) {
        Map<?, ?> envelope = Reads.object(Json.read(text));
        Map<?, ?> v = Reads.object(envelope, "value");
        // The clause's fields are spelt as Rust names them: no rename reaches
        // inside a variant, so `within_days` keeps its underscore.
        Map<?, ?> c = Reads.object(v, "clause");
        Object sex = v.get("sex");
        Object lastDay = v.get("lastDay");
        Object overlap = v.get("overlap");
        Map<?, ?> o = overlap == null ? null : Reads.object(overlap);
        return new AshauchAnswer(new Ashauch(
                Reads.string(v, "key"),
                AshauchKind.of(Reads.field(v, "kind")),
                AshauchRelation.of(Reads.field(v, "relation")),
                sex == null ? null : AshauchSex.of(sex),
                Reads.number(v, "start"),
                Reads.serdeDate(Reads.field(v, "dayOne")),
                dayOneBy(Reads.object(v, "dayOneBy")),
                Reads.number(v, "end"),
                lastDay == null ? null : Reads.serdeDate(lastDay),
                AshauchLength.of(Reads.field(v, "length")),
                Reads.optionalInteger(v, "days"),
                new AshauchClause(
                        Reads.string(c, "clause"),
                        Reads.optionalInteger(c, "row"),
                        Reads.optionalString(c, "said"),
                        Reads.optionalInteger(c, "within_days"),
                        c.get("abroad") == null ? null : Reads.flag(c, "abroad")),
                o == null ? null : new AshauchOverlapped(
                        Reads.string(o, "overlap"), Reads.string(o, "earlier"), Reads.optionalInteger(o, "days")),
                Reads.optionalNumber(v, "ritesFrom"),
                Reads.string(v, "table")), provenance(envelope));
    }

    /** Why a count's first day is the day it is, as an ashauch and a first year write it. */
    private static AshauchDayOneBy dayOneBy(Map<?, ?> by) {
        Object rule = by.get("rule");
        return new AshauchDayOneBy(Reads.string(by, "by"), rule == null ? null : NightRule.of(rule));
    }

    /** {@code ts_almanac_first_year}'s answer: the schedule, its members resolved, with the provenance beside it. */
    static FirstYearAnswer firstYearAnswer(String text) {
        Map<?, ?> envelope = Reads.object(Json.read(text));
        Map<?, ?> v = Reads.object(envelope, "value");
        return new FirstYearAnswer(new FirstYearSchedule(
                Reads.string(v, "key"),
                origin(Reads.object(v, "origin")),
                Reads.serdeDate(Reads.field(v, "dayOne")),
                dayOneBy(Reads.object(v, "dayOneBy")),
                Reads.serdeDate(Reads.field(v, "eleventhDay")),
                Reads.serdeDate(Reads.field(v, "sapindana")),
                Reads.each(v, "rites", AlmanacReads::firstYearRite)), provenance(envelope));
    }

    /** One rite of a first year: each member the answer leaves out when it has none read as null. */
    private static FirstYearRite firstYearRite(Object raw) {
        Map<?, ?> r = Reads.object(raw);
        Object drawnTo = r.get("drawnTo");
        Object observance = r.get("observance");
        return new FirstYearRite(
                Reads.string(r, "rite"),
                Reads.optionalInteger(r, "place"),
                Reads.optionalInteger(r, "month"),
                Reads.optionalInteger(r, "year"),
                Reads.flag(r, "adhika"),
                Reads.serdeDate(Reads.field(r, "day")),
                Reads.optionalString(r, "form"),
                drawnTo == null ? null : Reads.serdeDate(drawnTo),
                // Left out of the answer when empty.
                r.get("bars") == null ? List.of() : Reads.each(r, "bars", Reads::string),
                observance == null ? null : observance(observance),
                Reads.string(r, "clause"));
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
                        seen(h, "umbralSeen")),
                seen(both, "vedha"));
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
                view,
                seen(both, "vedha"));
    }
}

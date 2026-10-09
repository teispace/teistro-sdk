package com.teispace.teistro;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import java.util.function.Function;

import com.teispace.teistro.blob.Charts;

/**
 * The chart readings the library writes as JSON sections, KP, the prashna,
 * the remedies and the rectification, parsed once a batch into records with
 * their keys made catalogue members, as the Python façade's
 * {@code Chart.kp}, {@code Chart.prashna}, {@code Chart.remedies} and
 * {@code Chart.rectification} read them.
 */
final class KpReads {
    private KpReads() {
    }

    /** Every chart's record of a JSON section that holds a list of them, parsed once; empty for no text. */
    private static <T> List<T> parsed(String text, Function<Object, T> read) {
        if (text.isEmpty()) {
            return List.of();
        }
        List<T> out = new ArrayList<>();
        for (Object raw : JsonRead.list(Json.read(text))) {
            out.add(read.apply(raw));
        }
        return List.copyOf(out);
    }

    private static Charts decoded(Chart chart) {
        return chart.batch().decoded();
    }

    private static Object at(Object raw, String key) {
        return JsonRead.at(raw, key);
    }

    private static Graha graha(Object key) {
        return JsonRead.member(Graha::byKey, "Graha", key);
    }

    private static Graha grahaOrNull(Object key) {
        return JsonRead.memberOrNull(Graha::byKey, "Graha", key);
    }

    private static List<Graha> grahas(Object keys) {
        return JsonRead.members(Graha::byKey, "Graha", keys);
    }

    private static Rashi rashi(Object key) {
        return JsonRead.member(Rashi::byKey, "Rashi", key);
    }

    // ---- KP ----

    /**
     * The chart read as KP: its cusps and planets to the sub-sub lord, its
     * significators in Reader VI's order and the ruling planets of its moment,
     * under the settings' {@code kp} group; empty unless {@code kp} asked for
     * it ({@code 03-design/kp.md}). A longitude and a lord's span are integers
     * in nanoarcseconds, exact; for a horary {@code number} the cusps are the
     * number's and the ruling planets still the moment's own. Ports {@code Chart.kp}.
     */
    static Optional<KpReading> kp(Chart chart) {
        return VedicReads.at(chart.batch().cached("kps", () -> parsed(decoded(chart).kp(), KpReads::kpReading)),
                chart.index());
    }

    private static KpLevel level(Object raw) {
        Object span = at(raw, "span");
        return new KpLevel(graha(at(raw, "lord")),
                new KpSpan(JsonRead.whole(at(span, "start")), JsonRead.whole(at(span, "end"))));
    }

    private static KpLords lords(Object raw) {
        return new KpLords(graha(at(raw, "sign")), level(at(raw, "star")), level(at(raw, "sub")),
                level(at(raw, "subSub")));
    }

    private static KpRejection rejection(Object raw) {
        return raw == null ? null : new KpRejection(graha(at(raw, "retrograde")), JsonRead.bool(at(raw, "byStar")));
    }

    private static KpReason reason(Object raw) {
        String kind = JsonRead.text(at(raw, "kind"));
        if (kind.equals("AGENT")) {
            return new KpReason("AGENT", graha(at(raw, "of")), JsonRead.text(at(raw, "by")));
        }
        return new KpReason(kind, null, null);
    }

    /** A chart's KP reading from the {@code kp} section's JSON, its keys made members. */
    private static KpReading kpReading(Object raw) {
        Object chart = at(raw, "chart");
        Object significators = at(raw, "significators");
        Object ruling = at(raw, "ruling");
        List<KpCusp> cusps = new ArrayList<>();
        for (Object c : JsonRead.list(at(chart, "cusps"))) {
            cusps.add(new KpCusp(JsonRead.integer(at(c, "house")), JsonRead.whole(at(c, "longitude")),
                    lords(at(c, "lords"))));
        }
        List<KpPlanet> planets = new ArrayList<>();
        for (Object p : JsonRead.list(at(chart, "planets"))) {
            planets.add(new KpPlanet(graha(at(p, "graha")), JsonRead.whole(at(p, "longitude")),
                    JsonRead.bool(at(p, "retrograde")), JsonRead.integer(at(p, "house")), lords(at(p, "lords"))));
        }
        List<KpHouseSignificators> houses = new ArrayList<>();
        for (Object h : JsonRead.list(at(significators, "houses"))) {
            houses.add(new KpHouseSignificators(
                    JsonRead.integer(at(h, "house")),
                    grahas(at(h, "inOccupantsStars")),
                    grahas(at(h, "occupants")),
                    grahas(at(h, "inLordsStar")),
                    graha(at(h, "lord")),
                    grahas(at(h, "conjoined")),
                    grahas(at(h, "aspected")),
                    JsonRead.members(Rashi::byKey, "Rashi", at(h, "intercepted"))));
        }
        List<KpNodeAgency> nodes = new ArrayList<>();
        for (Object n : JsonRead.list(at(significators, "nodes"))) {
            nodes.add(new KpNodeAgency(graha(at(n, "node")), grahas(at(n, "conjoined")), graha(at(n, "starLord")),
                    grahas(at(n, "aspecting")), graha(at(n, "signLord"))));
        }
        List<KpRuler> rulers = new ArrayList<>();
        for (Object r : JsonRead.list(at(ruling, "rulers"))) {
            List<KpReason> reasons = new ArrayList<>();
            for (Object why : JsonRead.list(at(r, "reasons"))) {
                reasons.add(reason(why));
            }
            rulers.add(new KpRuler(graha(at(r, "graha")), reasons, JsonRead.bool(at(r, "retrograde")),
                    rejection(at(r, "rejectedBy")), rejection(at(r, "rejectedBySub"))));
        }
        Object rules = at(ruling, "rules");
        return new KpReading(
                new KpChart(JsonRead.member(HouseSystem::byKey, "HouseSystem", at(chart, "system")), cusps, planets),
                new KpSignificators(houses, nodes),
                new KpRuling(rulers, new KpRulingRules(JsonRead.text(at(rules, "count")),
                        JsonRead.text(at(rules, "nodeRulers")), JsonRead.text(at(rules, "retrogradeRejection")))));
    }

    // ---- the prashna ----

    /**
     * The chart read as a prashna, the chart of the moment a question was
     * asked: the verdict's clauses with <i>Shatpanchashika</i> I.4's three
     * outcomes, whether the matter stays, when, what an unspoken question is
     * about, the Tajika links when a house is asked and the Moon's
     * weaknesses; the baseline engine's points only under
     * {@code rules={"score": "BASELINE"}}. Empty unless {@code prashna} asked
     * for it ({@code 03-design/prashna.md}). Ports {@code Chart.prashna}.
     */
    static Optional<Prashna> prashna(Chart chart) {
        return VedicReads.at(
                chart.batch().cached("prashnas", () -> parsed(decoded(chart).prashna(), KpReads::prashnaOf)),
                chart.index());
    }

    /**
     * A chart's prashna from the {@code prashna} section's JSON, its keys made
     * members and its links a year's matter.
     */
    private static Prashna prashnaOf(Object raw) {
        Object timing = at(raw, "timing");
        Object mook = at(raw, "mook");
        Object links = at(raw, "links");
        Object score = at(raw, "score");
        Object sign = at(raw, "numberSign");
        Object verdict = at(raw, "verdict");
        List<PrashnaClause> clauses = new ArrayList<>();
        for (Object c : JsonRead.list(at(verdict, "clauses"))) {
            clauses.add(new PrashnaClause(JsonRead.text(at(c, "kind")), grahaOrNull(at(c, "graha")),
                    JsonRead.text(at(c, "favour"))));
        }
        PrashnaScore scored = null;
        if (score != null) {
            List<PrashnaFactor> factors = new ArrayList<>();
            for (Object f : JsonRead.list(at(score, "factors"))) {
                factors.add(new PrashnaFactor(JsonRead.text(at(f, "kind")), JsonRead.integer(at(f, "points"))));
            }
            scored = new PrashnaScore(JsonRead.integer(at(score, "points")), JsonRead.text(at(score, "answer")),
                    factors, JsonRead.bool(at(score, "void")), grahaOrNull(at(score, "applyingTo")));
        }
        Object moon = at(raw, "moon");
        return new Prashna(
                JsonRead.object(at(raw, "rules")),
                new PrashnaVerdict(clauses, JsonRead.text(at(verdict, "outcome"))),
                JsonRead.text(at(raw, "change")),
                new PrashnaTiming(
                        JsonRead.text(at(timing, "rule")),
                        graha(at(timing, "graha")),
                        JsonRead.bool(at(timing, "tie")),
                        JsonRead.integer(at(timing, "count")),
                        JsonRead.integer(at(timing, "multiplier")),
                        JsonRead.integerOrNull(at(timing, "amount")),
                        JsonRead.text(at(timing, "unit")),
                        grahas(at(timing, "between"))),
                new PrashnaMook(
                        JsonRead.text(at(mook, "rule")),
                        graha(at(mook, "graha")),
                        JsonRead.bool(at(mook, "tie")),
                        JsonRead.integer(at(mook, "house")),
                        JsonRead.textOrNull(at(mook, "person")),
                        JsonRead.text(at(mook, "thought"))),
                links == null ? null : links(links),
                new MoonWeakness(JsonRead.object(at(moon, "rules")), JsonRead.texts(at(moon, "clauses"))),
                scored,
                sign == null ? null : rashi(sign));
    }

    /** Two planets of an annual chart as the boundary's JSON writes them, or null for JSON null. */
    private static TajikaBetween pair(Object one) {
        if (one == null) {
            return null;
        }
        Object yoga = at(one, "yoga");
        return new TajikaBetween(
                graha(at(one, "faster")),
                graha(at(one, "slower")),
                JsonRead.member(TajikaDrishti::byKey, "TajikaDrishti", at(one, "drishti")),
                JsonRead.memberOrNull(TajikaYoga::byKey, "TajikaYoga", yoga),
                JsonRead.decimal(at(one, "orbDeg")),
                JsonRead.decimal(at(one, "apartDeg")));
    }

    /** A lord's afflictions from their clause flags, in the catalogue's order. */
    private static List<Affliction> afflicted(Object one) {
        String[][] clauses = {
            {"RETROGRADE", "retrograde"}, {"COMBUST", "combust"}, {"DEBILITATED", "debilitated"},
            {"TRIKA", "trika"}, {"UNDER_MALEFIC", "underMalefic"}};
        List<Affliction> found = new ArrayList<>();
        for (String[] clause : clauses) {
            if (JsonRead.bool(at(one, clause[1]))) {
                found.add(JsonRead.member(Affliction::byKey, "Affliction", clause[0]));
            }
        }
        return List.copyOf(found);
    }

    private static HeldYearYoga held(Object one) {
        Object legs = at(one, "legs");
        Object afflictions = at(one, "afflictions");
        TajikaBetween first = null;
        TajikaBetween second = null;
        if (legs != null) {
            List<?> two = JsonRead.list(legs);
            first = pair(two.get(0));
            second = pair(two.get(1));
        }
        Afflictions afflicted = null;
        if (afflictions != null) {
            List<?> two = JsonRead.list(afflictions);
            afflicted = new Afflictions(afflicted(two.get(0)), afflicted(two.get(1)));
        }
        return new HeldYearYoga(
                JsonRead.member(YearYoga::byKey, "YearYoga", at(one, "yoga")),
                pair(at(one, "between")),
                grahaOrNull(at(one, "through")),
                grahaOrNull(at(one, "entering")),
                first == null || second == null ? null : List.of(first, second),
                afflicted);
    }

    /** Tajika's sixteen yogas for one matter as the boundary's JSON writes them, in the shape a year's take. */
    private static PrashnaLinks links(Object raw) {
        Object states = at(raw, "states");
        List<HeldYearYoga> held = new ArrayList<>();
        for (Object one : JsonRead.list(at(raw, "held"))) {
            held.add(held(one));
        }
        return new PrashnaLinks(
                JsonRead.integer(at(raw, "house")),
                rashi(at(raw, "sign")),
                graha(at(raw, "lagnesha")),
                graha(at(raw, "karyesha")),
                JsonRead.bool(at(raw, "sameLord")),
                pair(at(raw, "between")),
                held,
                JsonRead.members(YearYoga::byKey, "YearYoga", at(raw, "unanswered")),
                states == null ? null
                        : new AnnualStatesRead(grahas(at(states, "retrograde")), grahas(at(states, "combust"))));
    }

    // ---- the remedies ----

    /**
     * The chart's remedies: what the lagna's lordships make of the seven
     * grahas, whom a remedy is for and why, each subject's graha-shanti, the
     * running antardasha's shanti under {@code at}, and the ishta-devata in
     * both charts. Empty unless {@code remedies} asked for them
     * ({@code 03-design/remedies.md}). Ports {@code Chart.remedies}.
     */
    static Optional<Remedies> remedies(Chart chart) {
        return VedicReads.at(
                chart.batch().cached("remedies", () -> parsed(decoded(chart).remedies(), KpReads::remediesOf)),
                chart.index());
    }

    private static List<Devotion> devotions(Object raw) {
        List<Devotion> found = new ArrayList<>();
        for (Object d : JsonRead.list(raw)) {
            found.add(new Devotion(graha(at(d, "graha")), JsonRead.texts(at(d, "deities")),
                    JsonRead.integer(at(d, "verse")), JsonRead.bool(at(d, "withKetu"))));
        }
        return List.copyOf(found);
    }

    private static IshtaDevata devata(Object one) {
        return new IshtaDevata(JsonRead.object(at(one, "rules")), rashi(at(one, "sign")),
                devotions(at(one, "devotions")), grahas(at(one, "minor")));
    }

    private static AmatyaDevata amatyaDevata(Object one) {
        return new AmatyaDevata(devata(at(one, "twelfth")), rashi(at(one, "sign")), JsonRead.integer(at(one, "house")),
                devotions(at(one, "joined")));
    }

    /** The running antardasha's shanti from the {@code remedies} section's JSON. */
    private static AntardashaShanti antardashaShanti(Object raw) {
        Object printed = at(raw, "shanti");
        List<Boolean> holds = new ArrayList<>();
        for (Object one : JsonRead.list(at(raw, "holds"))) {
            holds.add(JsonRead.boolOrNull(one));
        }
        return new AntardashaShanti(
                new DashaShanti(
                        graha(at(printed, "mahadasha")),
                        graha(at(printed, "antardasha")),
                        JsonRead.integer(at(printed, "chapter")),
                        JsonRead.text(at(printed, "verses")),
                        JsonRead.integer(at(printed, "page")),
                        JsonRead.texts(at(printed, "conditions")),
                        JsonRead.texts(at(printed, "remedies"))),
                holds);
    }

    /** One graha's shanti from the {@code remedies} section's JSON. */
    private static Shanti shanti(Object raw) {
        return new Shanti(
                graha(at(raw, "graha")),
                JsonRead.text(at(raw, "image")),
                JsonRead.text(at(raw, "rik")),
                JsonRead.integer(at(raw, "japaThousands")),
                JsonRead.text(at(raw, "samidh")),
                JsonRead.text(at(raw, "food")),
                JsonRead.text(at(raw, "dakshina")),
                JsonRead.text(at(raw, "gem")),
                JsonRead.textOrNull(at(raw, "substance")),
                JsonRead.memberOrNull(Direction::byKey, "Direction", at(raw, "direction")),
                JsonRead.text(at(raw, "mandala")));
    }

    /** A chart's remedies from the {@code remedies} section's JSON, its keys made members. */
    private static Remedies remediesOf(Object raw) {
        Object functional = at(raw, "functional");
        Object subjects = at(raw, "subjects");
        Object antardasha = at(subjects, "antardasha");
        Object devatas = at(raw, "ishtaDevata");
        Object amatya = at(devatas, "amatya");
        Object badhaka = at(functional, "badhaka");
        List<FunctionalRow> rows = new ArrayList<>();
        for (Object row : JsonRead.list(at(functional, "rows"))) {
            List<FunctionalClause> clauses = new ArrayList<>();
            for (Object c : JsonRead.list(at(row, "clauses"))) {
                clauses.add(new FunctionalClause(JsonRead.text(at(c, "kind")), JsonRead.integer(at(c, "house"))));
            }
            rows.add(new FunctionalRow(graha(at(row, "graha")), JsonRead.integers(at(row, "houses")), clauses,
                    JsonRead.text(at(row, "nature"))));
        }
        List<RemedySubject> subjected = new ArrayList<>();
        for (Object one : JsonRead.list(at(subjects, "subjects"))) {
            subjected.add(new RemedySubject(graha(at(one, "graha")), JsonRead.texts(at(one, "reasons"))));
        }
        List<Shanti> shantis = new ArrayList<>();
        for (Object one : JsonRead.list(at(raw, "shantis"))) {
            shantis.add(shanti(one));
        }
        return new Remedies(
                JsonRead.object(at(raw, "rules")),
                new Functional(
                        rashi(at(functional, "lagna")),
                        JsonRead.text(at(functional, "scheme")),
                        rows,
                        grahas(at(functional, "yogakarakas")),
                        grahas(at(functional, "marakas")),
                        new Badhaka(JsonRead.integer(at(badhaka, "house")), graha(at(badhaka, "lord")))),
                new RemedySubjects(subjected, antardasha == null ? null : antardashaShanti(antardasha)),
                shantis,
                new IshtaDevatas(
                        graha(at(devatas, "atmakaraka")),
                        rashi(at(devatas, "karakamsha")),
                        devata(at(devatas, "inRasi")),
                        devata(at(devatas, "inNavamsha")),
                        new AmatyaDevatas(
                                graha(at(amatya, "graha")),
                                rashi(at(amatya, "amsha")),
                                amatyaDevata(at(amatya, "inRasi")),
                                amatyaDevata(at(amatya, "inNavamsha")))));
    }

    // ---- the rectification ----

    /**
     * The chart read as a birth time to rectify, the chart's instant the time
     * on record: what the purifier of BPHS ch. 2 leaves standing of the
     * window, the conception reports, <i>Brihat Jataka</i> ch. V's
     * circumstances and the baseline engine's cascade, each only where the
     * request asked for it. Empty unless {@code rectification} asked for it
     * ({@code 03-design/rectification.md}). Ports {@code Chart.rectification}.
     */
    static Optional<Rectification> rectification(Chart chart) {
        return VedicReads.at(
                chart.batch().cached("rectifications",
                        () -> parsed(decoded(chart).rectification(), KpReads::rectificationOf)),
                chart.index());
    }

    /** The field {@code key} of an object, or null where it is absent or JSON null. */
    private static Object absent(Object raw, String key) {
        return JsonRead.object(raw).get(key);
    }

    private static Nakshatra nakshatra(Object key) {
        return JsonRead.member(Nakshatra::byKey, "Nakshatra", key);
    }

    private static List<Double> decimals(Object raw) {
        List<Double> found = new ArrayList<>();
        for (Object one : JsonRead.list(raw)) {
            found.add(JsonRead.decimal(one));
        }
        return List.copyOf(found);
    }

    /** The purifier's verdict at an instant, every clause it judged. */
    private static Purified.Verdict verdict(Object raw) {
        List<Purified.Clause> clauses = new ArrayList<>();
        for (Object c : JsonRead.list(at(raw, "clauses"))) {
            clauses.add(new Purified.Clause(
                    JsonRead.text(at(c, "purifier")),
                    JsonRead.text(at(c, "reference")),
                    rashi(at(c, "sign")),
                    rashi(at(c, "lagna")),
                    JsonRead.integer(at(c, "house")),
                    JsonRead.bool(at(c, "held")),
                    JsonRead.bool(at(c, "counted"))));
        }
        return new Purified.Verdict(clauses, JsonRead.bool(at(raw, "pure")));
    }

    private static List<Purified.Run> runs(Object raw) {
        List<Purified.Run> found = new ArrayList<>();
        for (Object r : JsonRead.list(raw)) {
            found.add(new Purified.Run(JsonRead.decimal(at(r, "from")), JsonRead.decimal(at(r, "to")),
                    verdict(at(r, "verdict"))));
        }
        return List.copyOf(found);
    }

    private static Purified purified(Object raw) {
        Object grid = at(raw, "grid");
        return new Purified(runs(at(raw, "intervals")), runs(at(raw, "removed")), decimals(at(raw, "edges")),
                new Purified.Grid(JsonRead.decimal(at(grid, "stepDays")), JsonRead.integer(at(grid, "cells"))));
    }

    private static Conception conception(Object raw) {
        Object house = at(raw, "pranapadaHouse");
        Object nisheka = at(raw, "nisheka");
        Object count = at(nisheka, "count");
        Object points = at(count, "points");
        Object span = at(count, "span");
        Object written = at(span, "written");
        Object added = absent(span, "moonAddedDeg");
        Object moon = at(raw, "moon");
        Object predicted = at(moon, "predicted");
        Object predictedNakshatra = absent(predicted, "nakshatra");
        Object moonNakshatra = absent(moon, "moonNakshatra");
        return new Conception(
                JsonRead.decimal(at(raw, "birth")),
                new Conception.PranapadaHouse(
                        JsonRead.decimal(at(house, "pranapadaDeg")),
                        JsonRead.decimal(at(house, "lagnaDeg")),
                        JsonRead.integer(at(house, "house")),
                        JsonRead.bool(at(house, "auspicious"))),
                new Conception.Nisheka(
                        new Conception.NishekaCount(
                                new Conception.NishekaPoints(
                                        JsonRead.decimal(at(points, "mandiDeg")),
                                        JsonRead.decimal(at(points, "saturnDeg")),
                                        JsonRead.decimal(at(points, "lagnaDeg")),
                                        JsonRead.decimal(at(points, "ninthDeg")),
                                        JsonRead.decimal(at(points, "lagnaLordDeg")),
                                        JsonRead.decimal(at(points, "moonDeg"))),
                                new Conception.NishekaSpan(
                                        JsonRead.decimal(at(span, "saturnToMandiDeg")),
                                        JsonRead.decimal(at(span, "lagnaToNinthDeg")),
                                        Optional.ofNullable(added).map(JsonRead::decimal),
                                        JsonRead.decimal(at(span, "arcDeg")),
                                        new Conception.MonthsBefore(
                                                JsonRead.integer(at(written, "months")),
                                                JsonRead.integer(at(written, "days")),
                                                JsonRead.integer(at(written, "ghatis")),
                                                JsonRead.integer(at(written, "palas"))),
                                        JsonRead.decimal(at(span, "daysBefore"))),
                                JsonRead.decimal(at(count, "instant")),
                                JsonRead.decimal(at(count, "daysPerBirthMinute"))),
                        JsonRead.decimal(at(nisheka, "lagnaDeg")),
                        verdict(at(nisheka, "verdict"))),
                new Conception.Moon(
                        new Conception.MoonCount(
                                JsonRead.integer(at(predicted, "dvadashamsha")),
                                rashi(at(predicted, "sign")),
                                Optional.ofNullable(predictedNakshatra).map(KpReads::nakshatra)),
                        rashi(at(moon, "moonSign")),
                        Optional.ofNullable(moonNakshatra).map(KpReads::nakshatra),
                        JsonRead.bool(at(moon, "signAgrees")),
                        Optional.ofNullable(JsonRead.boolOrNull(absent(moon, "nakshatraAgrees"))),
                        rashi(at(moon, "rising")),
                        JsonRead.text(at(moon, "predictedPart")),
                        JsonRead.bool(at(moon, "bornByDay")),
                        JsonRead.bool(at(moon, "partAgrees")),
                        JsonRead.decimal(at(moon, "risenFraction")),
                        JsonRead.decimal(at(moon, "elapsedFraction"))));
    }

    private static Circumstance circumstance(Object raw) {
        Object sky = at(raw, "sky");
        Object father = at(raw, "father");
        Object presentation = at(raw, "presentation");
        Object lamp = at(raw, "lamp");
        Object attending = at(raw, "attending");
        List<Circumstance.Weight> weights = new ArrayList<>();
        for (Object w : JsonRead.list(at(raw, "weights"))) {
            weights.add(new Circumstance.Weight(JsonRead.text(at(w, "indication")), JsonRead.bool(at(w, "agrees"))));
        }
        return new Circumstance(
                new Circumstance.Sky(JsonRead.decimal(at(sky, "lagnaDeg")), decimals(at(sky, "grahasDeg")),
                        JsonRead.bool(at(sky, "lordRetrograde"))),
                new Circumstance.Father(
                        JsonRead.member(Strength::byKey, "Strength", at(father, "moonAspect")),
                        JsonRead.bool(at(father, "unseen")),
                        JsonRead.bool(at(father, "saturnRising")),
                        JsonRead.bool(at(father, "marsSetting")),
                        JsonRead.bool(at(father, "moonHemmed")),
                        JsonRead.bool(at(father, "away")),
                        Optional.ofNullable(JsonRead.textOrNull(absent(father, "whereabouts"))),
                        JsonRead.integer(at(father, "sunHouse"))),
                new Circumstance.Presentation(
                        JsonRead.text(at(presentation, "by")),
                        JsonRead.member(Rising::byKey, "Rising", at(presentation, "rising")),
                        graha(at(presentation, "lord")),
                        JsonRead.bool(at(presentation, "lordRetrograde")),
                        JsonRead.text(at(presentation, "foretold"))),
                new Circumstance.Lamp(
                        JsonRead.decimal(at(lamp, "oil")),
                        JsonRead.text(at(lamp, "oilLevel")),
                        JsonRead.decimal(at(lamp, "wick")),
                        JsonRead.text(at(lamp, "wickLevel"))),
                new Circumstance.Attending(
                        grahas(at(attending, "between")),
                        grahas(at(attending, "visible")),
                        JsonRead.integer(at(attending, "inside")),
                        JsonRead.integer(at(attending, "outside"))),
                weights);
    }

    private static Interval interval(Object raw) {
        return new Interval(JsonRead.decimal(at(raw, "from")), JsonRead.decimal(at(raw, "to")));
    }

    /** One stage note from its tagged JSON. */
    private static BaselineNote note(Object raw) {
        String kind = JsonRead.text(at(raw, "kind"));
        return switch (kind) {
            case "TATTVA_SEX" -> new BaselineNote.TattvaSex(JsonRead.text(at(raw, "sex")),
                    JsonRead.decimal(at(raw, "admittedMinutes")), JsonRead.integer(at(raw, "penalised")),
                    JsonRead.integer(at(raw, "of")));
            case "REPORTED_TIME" -> new BaselineNote.ReportedTime(JsonRead.text(at(raw, "accuracy")),
                    JsonRead.decimal(at(raw, "uncertaintyMinutes")));
            case "EVENT_FIT" -> new BaselineNote.EventFit(JsonRead.integer(at(raw, "event")),
                    Optional.ofNullable(JsonRead.textOrNull(absent(raw, "id"))), JsonRead.text(at(raw, "eventKind")),
                    grahas(at(raw, "lords")), JsonRead.decimal(at(raw, "contribution")));
            default -> throw JsonRead.internal("the library wrote a stage note this build does not know: " + kind);
        };
    }

    private static BaselineRectification baseline(Object raw) {
        List<Interval> intervals = new ArrayList<>();
        for (Object one : JsonRead.list(at(raw, "intervals"))) {
            intervals.add(interval(one));
        }
        List<BaselineRectification.Ranked> candidates = new ArrayList<>();
        for (Object c : JsonRead.list(at(raw, "candidates"))) {
            candidates.add(new BaselineRectification.Ranked(
                    JsonRead.decimal(at(c, "at")),
                    JsonRead.decimal(at(c, "probability")),
                    JsonRead.decimal(at(c, "logPosterior")),
                    rashi(at(c, "lagna")),
                    nakshatra(at(c, "lagnaNakshatra"))));
        }
        List<BaselineRectification.Stage> stages = new ArrayList<>();
        for (Object s : JsonRead.list(at(raw, "stages"))) {
            List<BaselineNote> notes = new ArrayList<>();
            for (Object n : JsonRead.list(at(s, "notes"))) {
                notes.add(note(n));
            }
            stages.add(new BaselineRectification.Stage(
                    JsonRead.text(at(s, "stage")),
                    JsonRead.bool(at(s, "applied")),
                    JsonRead.bool(at(s, "flat")),
                    JsonRead.decimal(at(s, "resolutionMinutes")),
                    notes));
        }
        List<BaselineRectification.HoldOut> held = new ArrayList<>();
        for (Object h : JsonRead.list(at(raw, "holdOut"))) {
            held.add(new BaselineRectification.HoldOut(
                    JsonRead.integer(at(h, "event")),
                    JsonRead.text(at(h, "kind")),
                    JsonRead.decimal(at(h, "scoreAtFit")),
                    JsonRead.decimal(at(h, "baseline")),
                    JsonRead.bool(at(h, "supported"))));
        }
        return new BaselineRectification(
                interval(at(raw, "window")),
                JsonRead.decimal(at(raw, "sunrise")),
                intervals,
                JsonRead.decimal(at(raw, "intervalWidthMinutes")),
                JsonRead.decimal(at(raw, "resolutionMinutes")),
                JsonRead.decimal(at(raw, "suggested")),
                JsonRead.decimal(at(raw, "concentration")),
                candidates,
                stages,
                JsonRead.integer(at(raw, "eventsUsed")),
                JsonRead.integer(at(raw, "eventsHeldOut")),
                held);
    }

    /**
     * A chart's rectification from the {@code rectification} section's JSON,
     * its keys made members and each reading not asked for empty.
     */
    private static Rectification rectificationOf(Object raw) {
        return new Rectification(
                Optional.ofNullable(absent(raw, "purified")).map(KpReads::purified),
                Optional.ofNullable(absent(raw, "conception")).map(KpReads::conception),
                Optional.ofNullable(absent(raw, "circumstance")).map(KpReads::circumstance),
                Optional.ofNullable(absent(raw, "baseline")).map(KpReads::baseline));
    }
}

package com.teispace.teistro;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import java.util.function.Function;

import com.teispace.teistro.blob.Charts;

/**
 * The chart readings the library writes as JSON sections, KP, the prashna and
 * the remedies, parsed once a batch into records with their keys made
 * catalogue members, as the Python façade's {@code Chart.kp},
 * {@code Chart.prashna} and {@code Chart.remedies} read them.
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
}

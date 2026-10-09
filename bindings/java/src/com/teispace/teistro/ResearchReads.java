package com.teispace.teistro;

import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

import com.teispace.teistro.record.Provenance;

/** A study, the request written as {@code ts_research} reads it and the answer read into records. */
final class ResearchReads {
    private ResearchReads() {}

    private static Map<String, Object> birth(ResearchBirth birth) {
        Map<String, Object> out = new LinkedHashMap<>();
        out.put("instant", birth.instant());
        out.put("latitudeDeg", birth.place().latitudeDeg().value());
        out.put("longitudeDeg", birth.place().longitudeDeg().value());
        out.put("altitudeM", birth.place().altitudeM().value());
        out.put("utcOffsetSeconds", birth.utcOffsetSeconds());
        out.put("uncertaintyMinutes", birth.uncertaintyMinutes());
        return out;
    }

    static List<Map<String, Object>> births(List<ResearchBirth> births) {
        if (births == null) {
            throw Reads.invalid("births is a list of ResearchBirth", "births");
        }
        return births.stream().map(ResearchReads::birth).toList();
    }

    static List<Map<String, Object>> subjects(List<ResearchSubject> subjects) {
        if (subjects == null) {
            throw Reads.invalid("subjects is a list of ResearchSubject", "subjects");
        }
        return subjects.stream().map(subject -> {
            Map<String, Object> out = new LinkedHashMap<>();
            out.put("birth", birth(subject.birth()));
            out.put("event", subject.event());
            return out;
        }).toList();
    }

    /**
     * The study's answer: every field given crosses, and one the study does not read is refused by
     * name there rather than dropped here; {@code options} adds {@code holds} and, for an event study,
     * {@code depth}, {@code shuffle} and {@code strata}.
     */
    static Map<?, ?> run(Context context, String study, Map<String, ?> rules, Map<String, Object> fields,
            Map<String, ?> options) {
        Map<String, Object> asked = new LinkedHashMap<>();
        asked.put("study", study);
        String written = Reads.recordJson(rules, "rules", "Map.of(\"shipped\", List.of(\"YOGAS\"))");
        asked.put("rules", written == null ? Map.of() : Json.read(written));
        for (Map.Entry<String, Object> field : fields.entrySet()) {
            if (field.getValue() != null) {
                asked.put(field.getKey(), Reads.written(field.getValue()));
            }
        }
        if (options != null) {
            for (Map.Entry<String, ?> option : options.entrySet()) {
                asked.put(option.getKey(), Reads.written(option.getValue()));
            }
        }
        String json = Json.write(asked);
        return Reads.object(Json.read(context.locked((lib, raw) -> Calls.research(lib, raw, json))));
    }

    private static ResearchCounts.Group group(Object raw) {
        Map<?, ?> one = Reads.object(raw);
        return new ResearchCounts.Group(
                Reads.integer(one, "present"),
                Reads.integer(one, "absent"),
                Reads.integer(one, "unreadable"),
                Reads.integer(one, "unstable"));
    }

    static ResearchCounts counts(Map<?, ?> answer) {
        Map<?, ?> value = Reads.object(answer, "value");
        return new ResearchCounts(
                Reads.each(value, "rows", raw -> {
                    Map<?, ?> one = Reads.object(raw);
                    return new ResearchCounts.Row(Reads.string(one, "predicate"),
                            Reads.each(one, "counts", ResearchReads::group));
                }),
                Provenance.of(Reads.field(answer, "provenance")));
    }

    private static ResearchTested.Estimate estimate(Object raw) {
        Map<?, ?> one = Reads.object(raw);
        return new ResearchTested.Estimate(Reads.number(one, "estimate"), Reads.number(one, "low"),
                Reads.number(one, "high"));
    }

    private static ResearchTested.Effect effect(Object raw) {
        if (raw == null) {
            return null;
        }
        Map<?, ?> one = Reads.object(raw);
        Object ratio = one.get("riskRatio");
        return new ResearchTested.Effect(
                estimate(Reads.field(one, "riskCase")),
                estimate(Reads.field(one, "riskRest")),
                estimate(Reads.field(one, "riskDifference")),
                ratio == null ? null : estimate(ratio),
                Reads.optionalNumber(one, "oddsRatio"),
                Reads.number(one, "cohenH"));
    }

    private static ResearchTested.Row row(Object raw) {
        Map<?, ?> one = Reads.object(raw);
        Map<?, ?> p = Reads.object(one, "p");
        Map<?, ?> adjusted = Reads.object(one, "adjusted");
        Object expected = one.get("expected");
        Object under = one.get("underAlpha");
        return new ResearchTested.Row(
                Reads.string(one, "predicate"),
                Reads.each(one, "counts", ResearchReads::group),
                Reads.optionalNumber(one, "observed"),
                new ResearchTested.PValue(Reads.integer(p, "exceed"), Reads.number(p, "value"),
                        Reads.number(p, "low"), Reads.number(p, "high")),
                Reads.optionalNumber(one, "exact"),
                new ResearchTested.Adjusted(Reads.number(adjusted, "maxT"), Reads.number(adjusted, "holm"),
                        Reads.number(adjusted, "bonferroni"), Reads.number(adjusted, "bh"),
                        Reads.number(adjusted, "by")),
                effect(one.get("effect")),
                expected == null ? null : expectation(Reads.object(expected)),
                under == null ? null : underAlpha(Reads.object(under)));
    }

    private static ResearchTested.Expectation expectation(Map<?, ?> one) {
        return new ResearchTested.Expectation(Reads.number(one, "observed"), Reads.number(one, "expected"),
                Reads.optionalNumber(one, "ratio"));
    }

    private static ResearchTested.UnderAlpha underAlpha(Map<?, ?> one) {
        return new ResearchTested.UnderAlpha(Reads.flag(one, "raw"), Reads.flag(one, "maxT"),
                Reads.flag(one, "holm"), Reads.flag(one, "bonferroni"), Reads.flag(one, "bh"),
                Reads.flag(one, "by"));
    }

    static ResearchTested tested(Map<?, ?> answer) {
        Map<?, ?> value = Reads.object(answer, "value");
        return new ResearchTested(
                Reads.each(value, "rows", ResearchReads::row),
                Reads.integer(value, "permutations"),
                Reads.number(value, "resolution"),
                Reads.string(value, "shuffle"),
                Provenance.of(Reads.field(answer, "provenance")));
    }
}

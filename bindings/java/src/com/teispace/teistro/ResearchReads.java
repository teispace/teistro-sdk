package com.teispace.teistro;

import java.math.BigInteger;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

import com.teispace.teistro.record.Provenance;
import com.teispace.teistro.request.RuleRequest;

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

    /** A seed as the boundary reads it: a decimal string, which carries every 64-bit seed exactly. */
    private static String seed(BigInteger seed) {
        return seed.toString();
    }

    private static Map<String, Object> contrast(ResearchContrast contrast) {
        Map<String, Object> out = new LinkedHashMap<>();
        switch (contrast) {
            case ResearchContrast.CaseVsRest cases -> {
                out.put("kind", "CASE_VS_REST");
                out.put("case", cases.cases());
            }
            case ResearchContrast.AnyDifference any -> out.put("kind", "ANY_DIFFERENCE");
        }
        return out;
    }

    /** The fields every test reads: its direction, its level, and alpha and threads when given. */
    private static void reading(Map<String, Object> out, ResearchAlternative alternative, double level,
            Double alpha, Integer threads) {
        out.put("alternative", alternative.key());
        out.put("level", level);
        if (alpha != null) {
            out.put("alpha", alpha);
        }
        if (threads != null) {
            out.put("parallelism", Map.of("THREADS", threads));
        }
    }

    static Map<String, Object> test(ResearchGroupTest test) {
        if (test == null) {
            throw Reads.invalid("test is a ResearchGroupTest, such as ResearchGroupTest.builder(7, 999, "
                    + "ResearchContrast.caseVsRest(1)).build()", "test");
        }
        Map<String, Object> out = new LinkedHashMap<>();
        out.put("seed", seed(test.seed()));
        out.put("permutations", test.permutations());
        out.put("contrast", contrast(test.contrast()));
        reading(out, test.alternative(), test.level(), test.alpha(), test.threads());
        return out;
    }

    static Map<String, Object> test(ResearchEventTest test) {
        if (test == null) {
            throw Reads.invalid("test is a ResearchEventTest, such as ResearchEventTest.builder(7, 999).build()",
                    "test");
        }
        Map<String, Object> out = new LinkedHashMap<>();
        out.put("seed", seed(test.seed()));
        out.put("permutations", test.permutations());
        out.put("afterBirth", test.restrictPairings() ? "RESTRICT_PAIRINGS" : "REFUSE");
        reading(out, test.alternative(), test.level(), test.alpha(), test.threads());
        return out;
    }

    static Map<String, Object> test(ResearchReplicateTest test) {
        if (test == null) {
            return null;
        }
        Map<String, Object> out = new LinkedHashMap<>();
        reading(out, test.alternative(), test.level(), test.alpha(), null);
        return out;
    }

    static Map<String, Object> referents(ResearchReferents referents) {
        if (referents == null) {
            throw Reads.invalid("referents is a ResearchReferents, such as new ResearchReferents(365.25, 30)",
                    "referents");
        }
        Map<String, Object> out = new LinkedHashMap<>();
        out.put("windowDays", referents.windowDays());
        out.put("stepDays", referents.stepDays());
        return out;
    }

    static Map<String, Object> control(ResearchControl control) {
        if (control == null) {
            throw Reads.invalid("control is a ResearchControl, such as new ResearchControl(7, 1000)", "control");
        }
        Map<String, Object> out = new LinkedHashMap<>();
        out.put("seed", seed(control.seed()));
        out.put("replicates", control.replicates());
        if (control.strata() != null) {
            out.put("strata", control.strata());
        }
        return out;
    }

    /**
     * The study's answer: every field given crosses, and one the study does not read is refused by
     * name there rather than dropped here; {@code options}, when given, adds its fields.
     */
    static Map<?, ?> run(Context context, String study, RuleRequest rules, Map<String, Object> fields,
            ResearchOptions options) {
        Map<String, Object> asked = new LinkedHashMap<>();
        asked.put("study", study);
        asked.put("rules", rules == null ? Map.of() : rules);
        for (Map.Entry<String, Object> field : fields.entrySet()) {
            if (field.getValue() != null) {
                asked.put(field.getKey(), Reads.written(field.getValue()));
            }
        }
        if (options != null) {
            asked.putAll(options.fields());
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

    private static ResearchTested.Estimate maybeEstimate(Object raw) {
        return raw == null ? null : estimate(raw);
    }

    private static ResearchTested.Pooled pooled(Object raw) {
        if (raw == null) {
            return null;
        }
        Map<?, ?> one = Reads.object(raw);
        return new ResearchTested.Pooled(
                maybeEstimate(one.get("riskRatio")),
                maybeEstimate(one.get("oddsRatio")),
                Reads.optionalNumber(one, "homogeneityP"));
    }

    private static ResearchTested.Effect effect(Object raw) {
        if (raw == null) {
            return null;
        }
        Map<?, ?> one = Reads.object(raw);
        return new ResearchTested.Effect(
                estimate(Reads.field(one, "riskCase")),
                estimate(Reads.field(one, "riskRest")),
                estimate(Reads.field(one, "riskDifference")),
                maybeEstimate(one.get("riskRatio")),
                Reads.optionalNumber(one, "oddsRatio"),
                Reads.number(one, "cohenH"),
                pooled(one.get("pooled")));
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
                Reads.flag(value, "enumerated"),
                Reads.string(value, "shuffle"),
                Provenance.of(Reads.field(answer, "provenance")));
    }
}

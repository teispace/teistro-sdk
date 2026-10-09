package com.teispace.teistro;

import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

/**
 * {@code sky.research()}: counts and permutation tests over a batch of births
 * ({@code 03-design/research.md}). Every rule of {@code rules} is a predicate, read once on every chart,
 * and the predicates are one family for the corrections. A study's provenance seals it: its
 * {@code inputHash} is the pre-registration a study publishes before its data are collected.
 *
 * <p>The records cross as {@link Json#write} writes them: {@code rules} as a chart request's rules,
 * {@code design} as {@code {groups, strata}}, and each test and control as the design page names
 * them, a seed a {@code Long} or, past {@code Long.MAX_VALUE}, a {@code BigInteger}. {@code options}
 * adds {@code holds} ({@code "STANDING"} or {@code "FORMED"}) and, for an event study, {@code depth},
 * {@code shuffle} and {@code strata}; it may be null.
 *
 * <pre>{@code
 * ResearchTested tested = sky.research().compare(births, Map.of("shipped", List.of("YOGAS")),
 *         Map.of("groups", groups),
 *         Map.of("seed", 7, "permutations", 9999, "contrast", Map.of("kind", "CASE_VS_REST", "case", 1)),
 *         null);
 * }</pre>
 */
public final class ResearchArea {
    private final Context context;

    /**
     * The research area of a context.
     *
     * @param context the context every call goes through
     */
    ResearchArea(Context context) {
        this.context = context;
    }

    private static Map<String, Object> fields(Object... pairs) {
        Map<String, Object> out = new LinkedHashMap<>();
        for (int at = 0; at + 1 < pairs.length; at += 2) {
            out.put((String) pairs[at], pairs[at + 1]);
        }
        return out;
    }

    /**
     * How often each rule holds in each group, the charts it cannot be read on and those it is
     * unstable on counted apart. No null and no shuffle.
     *
     * @param births the births, in the order the design labels them
     * @param rules the rules, as a chart request's rules record
     * @param design {@code {groups, strata}}, one group per birth
     * @param options {@code holds}; may be null
     * @return the counts
     * @throws TeistroException naming the field, as {@code research.design}
     */
    public ResearchCounts counts(List<ResearchBirth> births, Map<String, ?> rules, Map<String, ?> design,
            Map<String, ?> options) {
        return ResearchReads.counts(ResearchReads.run(context, "COUNTS", rules,
                fields("births", ResearchReads.births(births), "design", design), options));
    }

    /**
     * Whether the design's groups differ on each rule, the labels permuted (within strata when the
     * design has them), with the family's corrections and the effect sizes.
     *
     * @param births the births, in the order the design labels them
     * @param rules the rules, as a chart request's rules record
     * @param design {@code {groups, strata}}, one group per birth
     * @param test {@code {seed, permutations, contrast, alternative, level, alpha, parallelism}}
     * @param options {@code holds}; may be null
     * @return the rows
     * @throws TeistroException naming the field, as {@code research.test.seed}
     */
    public ResearchTested compare(List<ResearchBirth> births, Map<String, ?> rules, Map<String, ?> design,
            Map<String, ?> test, Map<String, ?> options) {
        return ResearchReads.tested(ResearchReads.run(context, "COMPARE", rules,
                fields("births", ResearchReads.births(births), "design", design, "test", test), options));
    }

    /**
     * Whether each rule is commoner (or rarer) in this sample than in its own recombined population:
     * the sample refounded with clock times shuffled among its births, date and place kept.
     *
     * @param births the births
     * @param rules the rules, as a chart request's rules record
     * @param control {@code {seed, replicates, strata}}
     * @param test {@code {alternative, level, alpha}}; may be null
     * @param options {@code holds}; may be null
     * @return the rows
     * @throws TeistroException naming the field, as {@code research.control}
     */
    public ResearchTested expected(List<ResearchBirth> births, Map<String, ?> rules, Map<String, ?> control,
            Map<String, ?> test, Map<String, ?> options) {
        return ResearchReads.tested(ResearchReads.run(context, "EXPECTED", rules,
                fields("births", ResearchReads.births(births), "control", control, "test", test), options));
    }

    /**
     * Whether each rule is delivered by a dasha's running periods at the subjects' own events more (or
     * less) often than at events shuffled among them.
     *
     * @param subjects the subjects
     * @param rules the rules, as a chart request's rules record
     * @param dasha the dasha whose periods deliver
     * @param test {@code {seed, permutations, alternative, afterBirth, level, alpha, parallelism}}
     * @param options {@code holds}, {@code depth} (2 when left out), {@code shuffle}
     *     ({@code "EVENT_DATES"} or {@code "AGES_AT_EVENT"}) and {@code strata}; may be null
     * @return the rows
     * @throws TeistroException naming the field, as {@code research.subjects}
     */
    public ResearchTested timed(List<ResearchSubject> subjects, Map<String, ?> rules, DashaSystem dasha,
            Map<String, ?> test, Map<String, ?> options) {
        if (dasha == null) {
            throw Reads.invalid("dasha is a DashaSystem, such as DashaSystem.VIMSHOTTARI", "dasha");
        }
        return ResearchReads.tested(ResearchReads.run(context, "TIMED", rules,
                fields("subjects", ResearchReads.subjects(subjects), "dasha", dasha.fullKey(), "test", test),
                options));
    }
}

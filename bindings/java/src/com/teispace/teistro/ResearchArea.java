package com.teispace.teistro;

import com.teispace.teistro.request.Design;
import com.teispace.teistro.request.RuleRequest;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

/**
 * {@code sky.research()}: counts and permutation tests over a batch of births
 * ({@code 03-design/research.md}). Every rule of {@code rules} is a predicate, read once on every chart,
 * and the predicates are one family for the corrections. A study's provenance seals it: its
 * {@code inputHash} is the pre-registration a study publishes before its data are collected.
 *
 * <p>The rules are a chart request's {@link RuleRequest}, the design the generated {@link Design}, each
 * test a {@link ResearchGroupTest}, {@link ResearchEventTest} or {@link ResearchReplicateTest}, and the
 * options {@link ResearchOptions}. A seed is a {@code long} or, past {@code Long.MAX_VALUE}, a
 * {@code BigInteger}, and the same draws in every binding.
 *
 * <pre>{@code
 * RuleRequest rules = RuleRequest.builder().shipped(List.of(ShippedRules.YOGAS)).build();
 * ResearchTested tested = sky.research().compare(births, rules, new Design(groups, null),
 *         ResearchGroupTest.builder(7, 9999, ResearchContrast.caseVsRest(1)).build(),
 *         ResearchOptions.builder().holds(Holds.FORMED).build());
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
     * @param rules the rules, as a chart request's rules
     * @param design one group per birth, and the strata the labels move within
     * @param options {@code holds} and {@code uncertaintyStepMinutes}; may be null
     * @return the counts
     * @throws TeistroException naming the field, as {@code research.design}
     */
    public ResearchCounts counts(List<ResearchBirth> births, RuleRequest rules, Design design,
            ResearchOptions options) {
        return ResearchReads.counts(ResearchReads.run(context, "COUNTS", rules,
                fields("births", ResearchReads.births(births), "design", design), options));
    }

    /**
     * Whether the design's groups differ on each rule, the labels permuted (within strata when the
     * design has them), with the family's corrections and the effect sizes.
     *
     * @param births the births, in the order the design labels them
     * @param rules the rules, as a chart request's rules
     * @param design one group per birth, and the strata the labels move within
     * @param test the seed, the permutations and what is compared
     * @param options {@code holds} and {@code uncertaintyStepMinutes}; may be null
     * @return the rows
     * @throws TeistroException naming the field, as {@code research.test.seed}
     */
    public ResearchTested compare(List<ResearchBirth> births, RuleRequest rules, Design design,
            ResearchGroupTest test, ResearchOptions options) {
        return ResearchReads.tested(ResearchReads.run(context, "COMPARE", rules,
                fields("births", ResearchReads.births(births), "design", design, "test", ResearchReads.test(test)),
                options));
    }

    /**
     * Whether each rule is commoner (or rarer) in this sample than in its own recombined population:
     * the sample refounded with clock times shuffled among its births, date and place kept.
     *
     * @param births the births
     * @param rules the rules, as a chart request's rules
     * @param control the seed, the replicates and the strata
     * @param test how the sample is read against its replicates; {@link ResearchReplicateTest#DEFAULT}
     *     when null
     * @param options {@code holds} and {@code uncertaintyStepMinutes}; may be null
     * @return the rows
     * @throws TeistroException naming the field, as {@code research.control}
     */
    public ResearchTested expected(List<ResearchBirth> births, RuleRequest rules, ResearchControl control,
            ResearchReplicateTest test, ResearchOptions options) {
        return ResearchReads.tested(ResearchReads.run(context, "EXPECTED", rules,
                fields("births", ResearchReads.births(births), "control", ResearchReads.control(control), "test",
                        ResearchReads.test(test)),
                options));
    }

    /**
     * Whether each rule is delivered by a dasha's running periods at the subjects' own events more (or
     * less) often than at events shuffled among them.
     *
     * @param subjects the subjects
     * @param rules the rules, as a chart request's rules
     * @param dasha the dasha whose periods deliver
     * @param shuffle what the null keeps; there is no default
     * @param test the seed, the permutations and whether pairings are restricted
     * @param options {@code holds}, {@code depth} (2 when left out) and {@code strata}; may be null
     * @return the rows
     * @throws TeistroException naming the field, as {@code research.subjects}
     */
    public ResearchTested timed(List<ResearchSubject> subjects, RuleRequest rules, DashaSystem dasha,
            ResearchEventShuffle shuffle, ResearchEventTest test, ResearchOptions options) {
        if (dasha == null) {
            throw Reads.invalid("dasha is a DashaSystem, such as DashaSystem.VIMSHOTTARI", "dasha");
        }
        if (shuffle == null) {
            throw Reads.invalid("shuffle is a ResearchEventShuffle, such as AGES_AT_EVENT", "shuffle");
        }
        return ResearchReads.tested(ResearchReads.run(context, "TIMED", rules,
                fields("subjects", ResearchReads.subjects(subjects), "dasha", dasha.fullKey(), "shuffle",
                        shuffle.key(), "test", ResearchReads.test(test)),
                options));
    }
}

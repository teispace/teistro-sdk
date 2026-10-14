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
     * less) often than at events shuffled among them. A rule that reads transits holds under the sky at
     * each instant instead, and the test's {@code maxPairs} caps how many skies that is.
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
        return timed(subjects, rules, dashaKey(dasha), shuffle, test, options);
    }

    /**
     * The same study under a dasha system the context registered, by its full key.
     *
     * @param subjects the subjects
     * @param rules the rules, as a chart request's rules
     * @param dasha the full key of a system the context registered, as {@code dasha_system.ACME_SAPTAKA}
     * @param shuffle what the null keeps; there is no default
     * @param test the seed, the permutations and whether pairings are restricted
     * @param options {@code holds}, {@code depth} (2 when left out) and {@code strata}; may be null
     * @return the rows
     * @throws TeistroException naming the field, as {@code dasha} for a key the context did not register
     */
    public ResearchTested timed(List<ResearchSubject> subjects, RuleRequest rules, String dasha,
            ResearchEventShuffle shuffle, ResearchEventTest test, ResearchOptions options) {
        if (dasha == null) {
            throw Reads.invalid(DASHA, "dasha");
        }
        if (shuffle == null) {
            throw Reads.invalid("shuffle is a ResearchEventShuffle, such as AGES_AT_EVENT", "shuffle");
        }
        return ResearchReads.tested(ResearchReads.run(context, "TIMED", rules,
                fields("subjects", ResearchReads.subjects(subjects), "dasha", dasha, "shuffle",
                        shuffle.key(), "test", ResearchReads.test(test)),
                options));
    }

    /**
     * Whether each rule is delivered by a dasha's running periods at the subjects' own events more (or
     * less) often than at their own {@code referents}: each person their own control, the event drawn
     * among the person's instants ({@link ResearchEventShuffle#SELF_MATCHED}).
     *
     * <pre>{@code
     * ResearchTested matched = ctx.research().timed(subjects, rules, DashaSystem.VIMSHOTTARI,
     *         new ResearchReferents(365.25, 30), ResearchEventTest.builder(7, 999).build(), null);
     * }</pre>
     *
     * @param subjects the subjects
     * @param rules the rules, as a chart request's rules
     * @param dasha the dasha whose periods deliver
     * @param referents where each person is read besides their event
     * @param test the seed and the permutations
     * @param options {@code holds} and {@code depth} (2 when left out); {@code strata} is refused, a person
     *     already matched to themselves; may be null
     * @return the rows
     * @throws TeistroException naming the field, as {@code referents.stepDays}
     */
    public ResearchTested timed(List<ResearchSubject> subjects, RuleRequest rules, DashaSystem dasha,
            ResearchReferents referents, ResearchEventTest test, ResearchOptions options) {
        return timed(subjects, rules, dashaKey(dasha), referents, test, options);
    }

    /**
     * The same self-matched study under a dasha system the context registered, by its full key.
     *
     * @param subjects the subjects
     * @param rules the rules, as a chart request's rules
     * @param dasha the full key of a system the context registered, as {@code dasha_system.ACME_SAPTAKA}
     * @param referents where each person is read besides their event
     * @param test the seed and the permutations
     * @param options {@code holds} and {@code depth} (2 when left out); {@code strata} is refused
     * @return the rows
     * @throws TeistroException naming the field, as {@code dasha} for a key the context did not register
     */
    public ResearchTested timed(List<ResearchSubject> subjects, RuleRequest rules, String dasha,
            ResearchReferents referents, ResearchEventTest test, ResearchOptions options) {
        if (dasha == null) {
            throw Reads.invalid(DASHA, "dasha");
        }
        return ResearchReads.tested(ResearchReads.run(context, "TIMED", rules,
                fields("subjects", ResearchReads.subjects(subjects), "dasha", dasha, "shuffle",
                        ResearchEventShuffle.SELF_MATCHED.key(), "referents", ResearchReads.referents(referents),
                        "test", ResearchReads.test(test)),
                options));
    }

    /** What a study's dasha is, as a refusal says it. */
    private static final String DASHA =
            "dasha is a DashaSystem, such as DashaSystem.VIMSHOTTARI, or the dasha_system.* key of a system this context registered";

    /** A catalogued system's full key, refused when there is none. */
    private static String dashaKey(DashaSystem dasha) {
        if (dasha == null) {
            throw Reads.invalid(DASHA, "dasha");
        }
        return dasha.fullKey();
    }
}

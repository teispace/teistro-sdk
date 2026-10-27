package com.teispace.teistro;

import com.teispace.teistro.request.ResearchHolds;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

/**
 * What a study may say beside its births, rules and test
 * ({@code 03-design/research.md}); every field optional, an unset one leaving the SDK's default.
 * A field the study does not read is refused by name, not dropped: {@code depth} and {@code strata}
 * are an event study's, {@code uncertaintyStepMinutes} a study of births'.
 *
 * <pre>{@code
 * ResearchOptions options = ResearchOptions.builder().holds(ResearchHolds.FORMED).uncertaintyStepMinutes(5.0).build();
 * }</pre>
 *
 * @param holds when a rule counts as holding: standing (the default) or merely formed
 * @param uncertaintyStepMinutes how far apart, in minutes, an uncertain birth's window is read, out to
 *     both edges; 2 when unset
 * @param depth how deep an event study reads its dasha, 1 (the mahadasha) to 6; 2 when unset
 * @param strata each subject's stratum, inside which an event study's shuffle moves
 */
public record ResearchOptions(ResearchHolds holds, Double uncertaintyStepMinutes, Integer depth, List<Long> strata) {
    /** The value, its list copied. */
    public ResearchOptions {
        strata = strata == null ? null : List.copyOf(strata);
    }

    /**
     * A builder, every field unset.
     *
     * @return the builder
     */
    public static Builder builder() {
        return new Builder();
    }

    /** The fields given, under the names the request record reads. */
    Map<String, Object> fields() {
        Map<String, Object> out = new LinkedHashMap<>();
        if (holds != null) {
            out.put("holds", holds);
        }
        if (uncertaintyStepMinutes != null) {
            out.put("uncertaintyStepMinutes", uncertaintyStepMinutes);
        }
        if (depth != null) {
            out.put("depth", depth);
        }
        if (strata != null) {
            out.put("strata", strata);
        }
        return out;
    }

    /** Builds {@link ResearchOptions} a field at a time. */
    public static final class Builder {
        private ResearchHolds holds;
        private Double uncertaintyStepMinutes;
        private Integer depth;
        private List<Long> strata;

        private Builder() {}

        /**
         * Sets {@code holds}.
         *
         * @param holds when a rule counts as holding
         * @return this builder
         */
        public Builder holds(ResearchHolds holds) {
            this.holds = holds;
            return this;
        }

        /**
         * Sets {@code uncertaintyStepMinutes}.
         *
         * @param minutes how far apart an uncertain birth's window is read
         * @return this builder
         */
        public Builder uncertaintyStepMinutes(double minutes) {
            this.uncertaintyStepMinutes = minutes;
            return this;
        }

        /**
         * Sets {@code depth}.
         *
         * @param depth how deep an event study reads its dasha, 1 to 6
         * @return this builder
         */
        public Builder depth(int depth) {
            this.depth = depth;
            return this;
        }

        /**
         * Sets {@code strata}.
         *
         * @param strata each subject's stratum
         * @return this builder
         */
        public Builder strata(List<Long> strata) {
            this.strata = strata;
            return this;
        }

        /**
         * The options.
         *
         * @return the options
         */
        public ResearchOptions build() {
            return new ResearchOptions(holds, uncertaintyStepMinutes, depth, strata);
        }
    }
}

package com.teispace.teistro;

import java.util.List;
import java.util.Optional;

/**
 * What a stage of the baseline rectification says it did. Each kind is a record named for its tag
 * ({@link #kind()}), holding that kind's fields; a {@code switch} on the record reads it.
 *
 * <pre>{@code
 * for (BaselineNote note : stage.notes()) {
 *     if (note instanceof BaselineNote.EventFit fit) {
 *         System.out.println(fit.event() + " " + fit.lords());
 *     }
 * }
 * }</pre>
 */
public sealed interface BaselineNote {
    /**
     * The kind's tag, as the answer spells it.
     *
     * @return the tag
     */
    String kind();

    /**
     * The tattva prior: how many candidates a tattva of the other sex penalised.
     *
     * @param sex the sex, {@code MALE} or {@code FEMALE}
     * @param admittedMinutes the minutes of every 90 that admit it
     * @param penalised the candidates penalised, not excluded
     * @param of the candidates
     */
    record TattvaSex(String sex, double admittedMinutes, int penalised, int of) implements BaselineNote {
        /**
         * The kind's tag.
         *
         * @return {@code "TATTVA_SEX"}
         */
        @Override
        public String kind() {
            return "TATTVA_SEX";
        }
    }

    /**
     * The reported time's prior.
     *
     * @param accuracy how far it is trusted: {@code EXACT}, {@code APPROXIMATE},
     *     {@code RECTIFIED} or {@code UNKNOWN}
     * @param uncertaintyMinutes its uncertainty, minutes
     */
    record ReportedTime(String accuracy, double uncertaintyMinutes) implements BaselineNote {
        /**
         * The kind's tag.
         *
         * @return {@code "REPORTED_TIME"}
         */
        @Override
        public String kind() {
            return "REPORTED_TIME";
        }
    }

    /**
     * One event's fit.
     *
     * @param event its index in the request
     * @param id the caller's name for it; empty where it was given none
     * @param eventKind what happened, as {@code MARRIAGE}
     * @param lords the period lords that fit it, mahadasha first, where any did
     * @param contribution its best score over every candidate, weighted
     */
    record EventFit(int event, Optional<String> id, String eventKind, List<Graha> lords, double contribution)
            implements BaselineNote {
        /** Keeps the lists unmodifiable. */
        public EventFit {
            lords = List.copyOf(lords);
        }

        /**
         * The kind's tag.
         *
         * @return {@code "EVENT_FIT"}
         */
        @Override
        public String kind() {
            return "EVENT_FIT";
        }
    }
}

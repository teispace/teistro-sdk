package com.teispace.teistro;

import java.util.List;

/**
 * The baseline engine's unsourced cascade around a chart's instant: the reported time's prior and
 * the tattva of the sex, then the dated events against the dasha's periods
 * ({@code 03-design/rectification.md}). Instants are Julian days (UTC).
 *
 * @param window The window searched.
 * @param sunrise The sunrise the tattva cycle counted from.
 * @param intervals The intervals holding the request's {@code coverage} of the posterior.
 * @param intervalWidthMinutes Their width in all, minutes, never finer than the resolution.
 * @param resolutionMinutes The finest the stages that told candidates apart can tell, minutes.
 * @param suggested The posterior's mode.
 * @param concentration How concentrated the posterior is, 0 (flat) to 1.
 * @param candidates The most probable candidates, most probable first.
 * @param stages Each stage's outcome.
 * @param eventsUsed The events fitted.
 * @param eventsHeldOut The events held out.
 * @param holdOut The held-out events, tested.
 */
public record BaselineRectification(
        Interval window,
        double sunrise,
        List<Interval> intervals,
        double intervalWidthMinutes,
        double resolutionMinutes,
        double suggested,
        double concentration,
        List<Ranked> candidates,
        List<Stage> stages,
        int eventsUsed,
        int eventsHeldOut,
        List<HoldOut> holdOut) {
    /** The value, its lists copied and unmodifiable. */
    public BaselineRectification {
        intervals = List.copyOf(intervals);
        candidates = List.copyOf(candidates);
        stages = List.copyOf(stages);
        holdOut = List.copyOf(holdOut);
    }

    /**
     * One candidate, ranked.
     *
     * @param at The instant.
     * @param probability Its share of the posterior.
     * @param logPosterior Its log-posterior, the stages summed.
     * @param lagna Its lagna's sign.
     * @param lagnaNakshatra Its lagna's nakshatra.
     */
    public record Ranked(
            double at,
            double probability,
            double logPosterior,
            Rashi lagna,
            Nakshatra lagnaNakshatra) {
    }

    /**
     * One stage's outcome.
     *
     * @param stage The stage: {@code PRIOR} or {@code DASHA_BOUNDARY}.
     * @param applied Whether it ran.
     * @param flat Whether it told no candidate from another.
     * @param resolutionMinutes The finest it can tell, minutes.
     * @param notes What it says it did.
     */
    public record Stage(
            String stage,
            boolean applied,
            boolean flat,
            double resolutionMinutes,
            List<BaselineNote> notes) {
        /** The value, its lists copied and unmodifiable. */
        public Stage {
            notes = List.copyOf(notes);
        }
    }

    /**
     * A held-out event tested against the fit.
     *
     * @param event Its index in the request.
     * @param kind What happened, as {@code ACCIDENT}.
     * @param scoreAtFit Its score at the posterior's mode.
     * @param baseline Its mean score over candidates spread across the grid.
     * @param supported Whether the mode fits it by more than the margin over the spread.
     */
    public record HoldOut(
            int event,
            String kind,
            double scoreAtFit,
            double baseline,
            boolean supported) {
    }
}

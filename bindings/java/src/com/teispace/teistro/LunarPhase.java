package com.teispace.teistro;

/**
 * The Moon reached a principal phase, in the chart's frame (C419).
 *
 * @param phase Which of the four.
 */
public record LunarPhase(
        MoonPhase phase) implements HitEvent {
    /**
     * Which kind of event this is.
     *
     * @return {@link HitKind#LUNAR_PHASE}
     */
    @Override
    public HitKind kind() {
        return HitKind.LUNAR_PHASE;
    }
}

package com.teispace.teistro;

/**
 * The Moon's elongation from the Sun came back to the natal chart's (C420).
 */
public record PhaseReturn() implements HitEvent {
    /**
     * Which kind of event this is.
     *
     * @return {@link HitKind#PHASE_RETURN}
     */
    @Override
    public HitKind kind() {
        return HitKind.PHASE_RETURN;
    }
}

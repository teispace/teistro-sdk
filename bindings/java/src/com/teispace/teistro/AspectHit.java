package com.teispace.teistro;

/**
 * The graha aspected a natal point, or came within or left its orb.
 *
 * @param to The natal point aspected.
 * @param angle The angle, 0 to 180 degrees, either side of the natal point (C145).
 * @param phase Where in the orb's window (C146); always exact without an orb.
 * @param motion How the graha was moving.
 */
public record AspectHit(
        NatalPoint to,
        int angle,
        AspectPhase phase,
        Motion motion) implements HitEvent {
    /**
     * Which kind of event this is.
     *
     * @return {@link HitKind#ASPECT}
     */
    @Override
    public HitKind kind() {
        return HitKind.ASPECT;
    }
}

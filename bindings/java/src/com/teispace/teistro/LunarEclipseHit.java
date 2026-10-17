package com.teispace.teistro;

/**
 * A lunar eclipse fell on a natal point, within the request's eclipse orb (C421).
 *
 * @param to The natal point the eclipsed Moon fell on.
 * @param eclipse Which kind of eclipse.
 */
public record LunarEclipseHit(
        NatalPoint to,
        LunarEclipseKind eclipse) implements HitEvent {
    /**
     * Which kind of event this is.
     *
     * @return {@link HitKind#LUNAR_ECLIPSE}
     */
    @Override
    public HitKind kind() {
        return HitKind.LUNAR_ECLIPSE;
    }
}

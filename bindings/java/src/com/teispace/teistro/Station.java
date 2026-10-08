package com.teispace.teistro;

/**
 * The graha stood still in longitude.
 *
 * @param turns The motion it turned to.
 */
public record Station(
        Motion turns) implements HitEvent {
    /**
     * Which kind of event this is.
     *
     * @return {@link HitKind#STATION}
     */
    @Override
    public HitKind kind() {
        return HitKind.STATION;
    }
}

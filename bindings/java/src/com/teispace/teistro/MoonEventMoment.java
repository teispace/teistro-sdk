package com.teispace.teistro;

/**
 * A moonrise or a moonset. Python's {@code MoonEvent}; the generated
 * {@link MoonEvent} enum names which of the two it is.
 *
 * @param rise true for a rise, false for a set
 * @param instant when, as a Julian day (UTC)
 */
public record MoonEventMoment(boolean rise, double instant) {
    /**
     * Which of the two it is, as the catalogue names it.
     *
     * @return {@link MoonEvent#RISE} or {@link MoonEvent#SET}
     */
    public MoonEvent event() {
        return rise ? MoonEvent.RISE : MoonEvent.SET;
    }
}

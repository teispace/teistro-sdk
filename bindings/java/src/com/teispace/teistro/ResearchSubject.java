package com.teispace.teistro;

import java.util.Objects;

/**
 * One subject of an event study: a birth and the event of its life.
 *
 * @param birth the birth
 * @param event when the event happened, a Julian day in UTC
 */
public record ResearchSubject(ResearchBirth birth, double event) {
    /** The value, its birth required. */
    public ResearchSubject {
        Objects.requireNonNull(birth, "birth");
    }
}

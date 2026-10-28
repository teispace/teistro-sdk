package com.teispace.teistro;

/** Whose text a clause of a name in daily use is reported from (C774). */
public enum ClauseSource {
    /** Pāraskara's gṛhyasūtra. */
    PARASKARA,
    /** Āśvalāyana's gṛhyasūtra. */
    ASHVALAYANA,
    /** Āpastamba and Hiraṇyakeśin, as <i>Dharmasindhu</i> reports them. */
    APASTAMBA_HIRANYAKESHIN;

    static ClauseSource of(Object raw) {
        return valueOf(Reads.string(raw));
    }
}

package com.teispace.teistro;

/**
 * How an initial reaches a pada's cell (<code>03-design/namakarana.md</code>,
 * C294, C770).
 */
public enum InitialBy {
    /** The cell's own letter. */
    CELL,
    /** A letter paired with it: ब for व, श for स (C294). */
    PAIRED,
    /** The patro's substitute for a nasal no name begins with (C770). */
    SUBSTITUTE;

    static InitialBy of(Object raw) {
        return valueOf(Reads.string(raw));
    }
}

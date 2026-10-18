package com.teispace.teistro;

/** The Moon's twelve states within a sign, 2°30′ each, in the gloss's order (vv. 30–33; {@code 03-design/varshesha.md}). */
public enum MoonState {
    /** A stay abroad. */
    PRAVASA,
    /** Loss of wealth. */
    NASHTA,
    /** Fear of death. */
    MRITA,
    /** Victory. */
    JAYA,
    /** Pleasure. */
    HASYA,
    /** Union. */
    RATI,
    /** Happiness. */
    KRIDITA,
    /** Sleep and quarrel. */
    PRASUPTA,
    /** Bodily pain. */
    BHUKTA,
    /** Fear. */
    JVARA,
    /** Loss. */
    KAMPITA,
    /** Ease. */
    SUSTHITA;

    /**
     * The spelling the section carries.
     *
     * @return the key, as {@code "PRAVASA"}
     */
    public String key() {
        return name();
    }
}

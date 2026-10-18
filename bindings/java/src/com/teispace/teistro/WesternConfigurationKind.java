package com.teispace.teistro;

/** Which aspect configuration a shape is ({@code 03-design/western-configurations.md}). */
public enum WesternConfigurationKind {
    /** Three bodies, each trine the other two. */
    GRAND_TRINE,
    /** Two bodies in opposition and a third, the apex, square to both. */
    T_SQUARE,
    /** Four bodies in two oppositions, each square to both ends of the other. */
    GRAND_CROSS,
    /** Two bodies in sextile, both quincunx a third, the apex. */
    YOD,
    /** A grand trine with a fourth body, the apex, opposite one of its three and sextile the other two. */
    KITE,
    /** Four bodies in two oppositions whose ends are joined alternately by trines and sextiles. */
    MYSTIC_RECTANGLE,
    /** Bodies enough in one tropical sign. */
    STELLIUM;

    /**
     * The spelling the section carries.
     *
     * @return the key, as {@code "T_SQUARE"}
     */
    public String key() {
        return name();
    }
}

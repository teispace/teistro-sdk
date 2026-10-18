package com.teispace.teistro;

import java.util.List;

/**
 * One sthira karaka of a chart (BPHS ch. 32 vv. 18 to 20;
 * {@code 03-design/jaimini-significators.md} §7).
 *
 * @param signification What it signifies.
 * @param candidates The grahas the verse names for it, in id order: one, or the pair the stronger is
 *     taken of.
 * @param graha The karaka.
 * @param chosenBy How it was chosen (C408, C409).
 */
public record SthiraKaraka(Signification signification, List<Graha> candidates, Graha graha, ChosenBy chosenBy) {
    /** The value, its list copied and unmodifiable. */
    public SthiraKaraka {
        candidates = List.copyOf(candidates);
    }
}

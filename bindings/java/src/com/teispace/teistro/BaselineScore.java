package com.teispace.teistro;

import java.util.List;
import java.util.Map;

/**
 * The baseline engine's score of one sign's reading, {@code BASELINE} and
 * unsourced (C361).
 *
 * @param overall 0 to 100
 * @param areas the eight life areas and their scores, each 0 to 100, in the
 *     baseline's order ({@code "OVERALL"} first)
 * @param keyInfluences the grahas the score names
 * @param lucky the sign lord's lucky elements
 */
public record BaselineScore(
        int overall, List<Map.Entry<String, Integer>> areas, List<KeyInfluence> keyInfluences, LuckyElements lucky) {
    /** Keeps the lists unmodifiable. */
    public BaselineScore {
        areas = List.copyOf(areas);
        keyInfluences = List.copyOf(keyInfluences);
    }
}

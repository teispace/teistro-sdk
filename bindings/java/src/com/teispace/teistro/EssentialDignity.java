package com.teispace.teistro;

import java.util.ArrayList;
import java.util.Collections;
import java.util.List;

/**
 * The dignities and debilities a planet holds where it stands.
 *
 * @param house in its own house
 * @param exaltation in its exaltation
 * @param triplicity in its triplicity
 * @param term in its term
 * @param face in its face
 * @param detriment in its detriment
 * @param fall in its fall
 */
public record EssentialDignity(
        boolean house, boolean exaltation, boolean triplicity, boolean term, boolean face, boolean detriment,
        boolean fall) {

    /** The five essential dignities, by the name each flag has, strongest first. */
    static final List<String> KINDS = List.of("house", "exaltation", "triplicity", "term", "face");

    /**
     * The flags from a bit set: bit 0 the house, then the exaltation, the
     * triplicity, the term, the face, the detriment and the fall.
     */
    static EssentialDignity ofBits(long bits) {
        return new EssentialDignity((bits & 1) != 0, (bits >> 1 & 1) != 0, (bits >> 2 & 1) != 0,
                (bits >> 3 & 1) != 0, (bits >> 4 & 1) != 0, (bits >> 5 & 1) != 0, (bits >> 6 & 1) != 0);
    }

    /**
     * Whether it holds one of the five essential dignities, by its name.
     *
     * @param kind {@code house}, {@code exaltation}, {@code triplicity}, {@code term} or {@code face}
     * @return whether it holds that one
     * @throws IllegalArgumentException for a name that is not one of the five
     */
    public boolean holds(String kind) {
        return switch (kind) {
            case "house" -> house;
            case "exaltation" -> exaltation;
            case "triplicity" -> triplicity;
            case "term" -> term;
            case "face" -> face;
            default -> throw new IllegalArgumentException(
                    "`" + kind + "` is not an essential dignity; they are " + String.join(", ", KINDS));
        };
    }

    /**
     * The five essential dignities it holds, strongest first.
     *
     * @return the names of the ones it holds
     */
    public List<String> held() {
        List<String> out = new ArrayList<>();
        for (String kind : KINDS) {
            if (holds(kind)) {
                out.add(kind);
            }
        }
        return Collections.unmodifiableList(out);
    }
}

package com.teispace.teistro;

import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

/**
 * The orbs and limits Lilly's accidental fortitudes were judged by
 * ({@code 03-design/essential-dignities.md} §Accidental fortitudes). Its
 * {@link #request()} handed back as a request's {@code rules} asks for the
 * same again.
 *
 * @param combustionDeg combust within this many degrees of the Sun
 * @param combustionInSign whether combustion also asks for the Sun's sign (C211)
 * @param beamsDeg under the beams within this many degrees (C212)
 * @param cazimiDeg cazimi within this many degrees
 * @param cuspOrbDeg a planet this near the next cusp is in its house (p. 33, C214)
 * @param starOrbDeg with a star within this many degrees
 * @param partile by the same degree, Lilly's, or within {@code partileOrbDeg}
 *     of the exact aspect (C216)
 * @param partileOrbDeg the orb of {@link Partile#WITHIN}; 0 for {@code SAME_DEGREE}
 * @param siege within one sign, Lilly's example, or on an arc no wider than
 *     {@code siegeSpanDeg} (C215)
 * @param siegeSpanDeg the span of {@link Siege#WITHIN}; 0 for {@code SAME_SIGN}
 * @param meanMotionDeg the mean daily motions swift and slow are judged
 *     against, the seven in the Chaldean order
 */
public record AccidentalRules(
        double combustionDeg, boolean combustionInSign, double beamsDeg, double cazimiDeg, double cuspOrbDeg,
        double starOrbDeg, Partile partile, double partileOrbDeg, Siege siege, double siegeSpanDeg,
        List<Double> meanMotionDeg) {
    /** Keeps the list unmodifiable. */
    public AccidentalRules {
        meanMotionDeg = List.copyOf(meanMotionDeg);
    }

    /**
     * The rules as a request writes them, to hand back as a fortitude
     * request's {@code rules}.
     *
     * @return the request record, for {@link Json#write}
     */
    public Map<String, Object> request() {
        Map<String, Object> out = new LinkedHashMap<>();
        out.put("combustionDeg", combustionDeg);
        out.put("combustionInSign", combustionInSign);
        out.put("beamsDeg", beamsDeg);
        out.put("cazimiDeg", cazimiDeg);
        out.put("cuspOrbDeg", cuspOrbDeg);
        out.put("starOrbDeg", starOrbDeg);
        out.put("partile", withOrb(partile, "orbDeg", partileOrbDeg));
        out.put("siege", withOrb(siege, "spanDeg", siegeSpanDeg));
        out.put("meanMotionDeg", meanMotionDeg);
        return out;
    }

    private static Object withOrb(Member member, String field, double orb) {
        return "WITHIN".equals(member.key()) ? Map.of("WITHIN", Map.of(field, orb)) : member.key();
    }
}

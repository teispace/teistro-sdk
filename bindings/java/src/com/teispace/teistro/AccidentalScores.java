package com.teispace.teistro;

import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

/**
 * What each of Lilly's accidental lines was worth (p. 115): positive for a
 * fortitude, negative for a debility. Its {@link #request()} handed back
 * as a request's {@code scores} asks for the same again.
 *
 * @param houses the first house to the twelfth
 * @param direct what the line "direct" scores
 * @param retrograde what the line "retrograde" scores
 * @param swift what the line "swift" scores
 * @param slow what the line "slow" scores
 * @param superiorOriental Saturn, Jupiter or Mars oriental
 * @param superiorOccidental Saturn, Jupiter or Mars occidental
 * @param inferiorOriental Venus or Mercury oriental
 * @param inferiorOccidental Venus or Mercury occidental
 * @param increasing what the line "increasing" scores
 * @param decreasing what the line "decreasing" scores
 * @param freeFromCombustion what the line "free from combustion" scores
 * @param cazimi what the line "cazimi" scores
 * @param combust what the line "combust" scores
 * @param underBeams what the line "under beams" scores
 * @param conjunctBenefic what the line "conjunct benefic" scores
 * @param conjunctNorthNode what the line "conjunct north node" scores
 * @param trineBenefic what the line "trine benefic" scores
 * @param sextileBenefic what the line "sextile benefic" scores
 * @param conjunctMalefic what the line "conjunct malefic" scores
 * @param conjunctSouthNode what the line "conjunct south node" scores
 * @param opposedMalefic what the line "opposed malefic" scores
 * @param squareMalefic what the line "square malefic" scores
 * @param besieged what the line "besieged" scores
 * @param regulus what the line "regulus" scores
 * @param spica what the line "spica" scores
 * @param algol what the line "algol" scores
 */
public record AccidentalScores(
        List<Integer> houses, int direct, int retrograde, int swift, int slow, int superiorOriental, int superiorOccidental, int inferiorOriental, int inferiorOccidental, int increasing, int decreasing, int freeFromCombustion, int cazimi, int combust, int underBeams, int conjunctBenefic, int conjunctNorthNode, int trineBenefic, int sextileBenefic, int conjunctMalefic, int conjunctSouthNode, int opposedMalefic, int squareMalefic, int besieged, int regulus, int spica, int algol) {
    /** Keeps the list unmodifiable. */
    public AccidentalScores {
        houses = List.copyOf(houses);
    }

    /**
     * The scores as a request writes them, to hand back as a fortitude
     * request's {@code scores}.
     *
     * @return the request record, for {@link Json#write}
     */
    public Map<String, Object> request() {
        Map<String, Object> out = new LinkedHashMap<>();
        out.put("houses", houses);
        out.put("direct", direct);
        out.put("retrograde", retrograde);
        out.put("swift", swift);
        out.put("slow", slow);
        out.put("superiorOriental", superiorOriental);
        out.put("superiorOccidental", superiorOccidental);
        out.put("inferiorOriental", inferiorOriental);
        out.put("inferiorOccidental", inferiorOccidental);
        out.put("increasing", increasing);
        out.put("decreasing", decreasing);
        out.put("freeFromCombustion", freeFromCombustion);
        out.put("cazimi", cazimi);
        out.put("combust", combust);
        out.put("underBeams", underBeams);
        out.put("conjunctBenefic", conjunctBenefic);
        out.put("conjunctNorthNode", conjunctNorthNode);
        out.put("trineBenefic", trineBenefic);
        out.put("sextileBenefic", sextileBenefic);
        out.put("conjunctMalefic", conjunctMalefic);
        out.put("conjunctSouthNode", conjunctSouthNode);
        out.put("opposedMalefic", opposedMalefic);
        out.put("squareMalefic", squareMalefic);
        out.put("besieged", besieged);
        out.put("regulus", regulus);
        out.put("spica", spica);
        out.put("algol", algol);
        return out;
    }
}

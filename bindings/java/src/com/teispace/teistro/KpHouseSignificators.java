package com.teispace.teistro;

import java.util.List;

/**
 * A house's significators in KP Reader VI's order, strongest first (C154).
 *
 * @param house The house, 1 to 12.
 * @param inOccupantsStars (a) Planets in the stars of the house's occupants.
 * @param occupants (b) The occupants.
 * @param inLordsStar (c) Planets in the star of the house's lord.
 * @param lord (d) The house's lord.
 * @param conjoined (e) Planets joined to a significator above.
 * @param aspected (f) Planets aspecting the house under the settings' node aspects.
 * @param intercepted Signs wholly inside the house.
 */
public record KpHouseSignificators(
        int house,
        List<Graha> inOccupantsStars,
        List<Graha> occupants,
        List<Graha> inLordsStar,
        Graha lord,
        List<Graha> conjoined,
        List<Graha> aspected,
        List<Rashi> intercepted) {
    /** The value, its lists copied and unmodifiable. */
    public KpHouseSignificators {
        inOccupantsStars = List.copyOf(inOccupantsStars);
        occupants = List.copyOf(occupants);
        inLordsStar = List.copyOf(inLordsStar);
        conjoined = List.copyOf(conjoined);
        aspected = List.copyOf(aspected);
        intercepted = List.copyOf(intercepted);
    }
}

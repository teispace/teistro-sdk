package com.teispace.teistro;

import java.util.List;

/**
 * The ways of perfection (pp. 125–127): what they weigh, and which hold.
 *
 * @param querent where the querent's significator stands
 * @param quesited where the quesited's stands
 * @param mutualByHouse each stands in the other's house
 * @param infortunesBetween Saturn and Mars among the thirds that come
 *     between the significators before they perfect
 * @param moonRelays the Moon, neither significator, separating from the
 *     quesited's and coming next to the querent's
 * @param quesitedInAscendant whether the quesited's stands in the ascendant
 * @param held the ways the figure holds, in {@link Way}'s order
 */
public record Ways(
        SignificatorPlace querent, SignificatorPlace quesited, boolean mutualByHouse, List<Graha> infortunesBetween,
        boolean moonRelays, boolean quesitedInAscendant, List<Way> held) {
    /** Keeps the lists unmodifiable. */
    public Ways {
        infortunesBetween = List.copyOf(infortunesBetween);
        held = List.copyOf(held);
    }
}

package com.teispace.teistro;

import java.util.List;

/**
 * One graha's transit, read from the reference sign.
 *
 * @param graha Which graha.
 * @param transit Where it stands.
 * @param house Its house from the reference sign, 1 to 12.
 * @param goodHouse Whether v. 2 makes a transit of this house good.
 * @param vedhaHouse The house whose occupant obstructs it (vv. 3 to 8); null where nothing can.
 *     May be null.
 * @param obstructedBy The grahas standing in the vedha house that obstruct it, the verses'
 *     exemptions left out, in id order.
 * @param verdict The verdict on the transit.
 * @param fruition The decanate in which its transit bears fruit (v. 25).
 * @param fruitfulNow Whether it stands in that decanate now.
 */
public record GrahaGochar(
        Graha graha,
        Transit transit,
        int house,
        boolean goodHouse,
        Integer vedhaHouse,
        List<Graha> obstructedBy,
        GocharVerdict verdict,
        Fruition fruition,
        boolean fruitfulNow) {
    /** The value, its lists copied and unmodifiable. */
    public GrahaGochar {
        obstructedBy = List.copyOf(obstructedBy);
    }
}
